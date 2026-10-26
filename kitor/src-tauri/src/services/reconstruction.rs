//! Bounded, native source-range reconstruction. Recipes contain no executable code.
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Recipe {
    schema_version: u32,
    input_sha256: String,
    #[serde(default)]
    alternate_input_sha256: Vec<String>,
    output_size: u64,
    operations: Vec<Op>,
}
#[derive(Deserialize)]
#[serde(untagged)]
enum Op {
    Copy(Copy),
    Literal(Literal),
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Copy {
    offset: u64,
    length: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Literal {
    hex: String,
}
fn hex(s: &str) -> Result<Vec<u8>, String> {
    if !s.is_ascii() || s.len() % 2 != 0 {
        return Err("Invalid recipe literal".into());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| e.to_string()))
        .collect()
}
pub fn apply(recipe: &Path, input: &Path, output: &Path) -> Result<(), String> {
    let err = |e: std::io::Error| e.to_string();
    if std::fs::metadata(recipe).map_err(err)?.len() > 16 * 1024 * 1024 {
        return Err("Recipe too large".into());
    }
    let r: Recipe =
        serde_json::from_slice(&std::fs::read(recipe).map_err(err)?).map_err(|e| e.to_string())?;
    if r.schema_version != 1
        || r.output_size > 4 * 1024 * 1024 * 1024
        || r.operations.len() > 200_000
        || r.alternate_input_sha256.len() > 8
    {
        return Err("Unsupported recipe limits".into());
    }
    let mut source = File::open(input).map_err(err)?;
    let size = source.metadata().map_err(err)?.len();
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let n = source.read(&mut buffer).map_err(err)?;
        if n == 0 {
            break;
        }
        digest.update(&buffer[..n]);
    }
    let input_hash = format!("{:x}", digest.finalize());
    if input_hash != r.input_sha256 && !r.alternate_input_sha256.contains(&input_hash) {
        return Err("Unsupported game data checksum; use a supported original edition".into());
    }
    // Validate the entire plan before opening output or copying any bytes.
    let mut total = 0u64;
    for op in &r.operations {
        let n = match op {
            Op::Copy(c) => {
                if c.offset.checked_add(c.length).is_none_or(|end| end > size) {
                    return Err("Recipe range outside source".into());
                }
                c.length
            }
            Op::Literal(l) => hex(&l.hex)?.len() as u64,
        };
        total = total.checked_add(n).ok_or("Recipe length overflow")?;
        if total > r.output_size {
            return Err("Recipe exceeds declared output".into());
        }
    }
    if total != r.output_size {
        return Err("Recipe output length mismatch".into());
    }
    let mut out = File::create(output).map_err(err)?;
    for op in r.operations {
        match op {
            Op::Literal(l) => out.write_all(&hex(&l.hex)?).map_err(err)?,
            Op::Copy(c) => {
                source.seek(SeekFrom::Start(c.offset)).map_err(err)?;
                let mut left = c.length;
                while left > 0 {
                    let n = left.min(buffer.len() as u64) as usize;
                    source.read_exact(&mut buffer[..n]).map_err(err)?;
                    out.write_all(&buffer[..n]).map_err(err)?;
                    left -= n as u64;
                }
            }
        }
    }
    out.sync_all().map_err(err)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reconstructs_and_rejects_invalid_recipes_before_output() {
        let d = tempfile::tempdir().unwrap();
        let input = d.path().join("input");
        let out = d.path().join("out");
        let recipe = d.path().join("recipe");
        std::fs::write(&input, b"abcdefgh").unwrap();
        let mut r = serde_json::json!({"schema_version":1,"input_sha256":format!("{:x}",Sha256::digest(b"abcdefgh")),"output_size":5,"operations":[{"offset":2,"length":3},{"hex":"2121"}]});
        std::fs::write(&recipe, r.to_string()).unwrap();
        apply(&recipe, &input, &out).unwrap();
        assert_eq!(std::fs::read(&out).unwrap(), b"cde!!");
        for bad in [
            serde_json::json!({"offset":7,"length":3}),
            serde_json::json!({"hex":"zz"}),
            serde_json::json!({"offset":u64::MAX,"length":2}),
        ] {
            r["operations"][0] = bad;
            std::fs::write(&recipe, r.to_string()).unwrap();
            assert!(apply(&recipe, &input, &out).is_err());
            assert_eq!(std::fs::read(&out).unwrap(), b"cde!!");
        }
    }
}
