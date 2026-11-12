use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Row {
    pub key: String,
    pub character: String,
    pub context: String,
    pub english: String,
    pub hebrew: String,
}

pub fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn read(source: &str) -> Result<Vec<(csv::StringRecord, usize, usize)>, String> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(source.as_bytes());
    let mut result = Vec::new();
    let mut record = csv::StringRecord::new();
    let mut start = 0;
    while reader
        .read_record(&mut record)
        .map_err(|_| "Invalid translation CSV")?
    {
        let mut end = reader.position().byte() as usize;
        // csv-core may report its cursor between CR and LF. Own the complete
        // terminator so replacing a record cannot remove its neighbour's LF.
        if end > 0
            && source.as_bytes().get(end - 1) == Some(&b'\r')
            && source.as_bytes().get(end) == Some(&b'\n')
        {
            end += 1;
        }
        if record.len() != 6 {
            return Err("Expected six CSV columns".into());
        }
        result.push((record.clone(), start, end));
        start = end;
    }
    Ok(result)
}

fn key(record: &csv::StringRecord) -> String {
    hash(
        serde_json::to_string(&record.iter().take(5).collect::<Vec<_>>())
            .unwrap()
            .as_bytes(),
    )
}

pub fn rows(source: &str) -> Result<Vec<Row>, String> {
    let records = read(source)?;
    if records.len() < 4
        || records[0].0.iter().collect::<Vec<_>>()
            != ["Character", "ID", "File", "Context", "English", "Hebrew"]
    {
        return Err("Unsupported Drifter CSV header".into());
    }
    let mut seen = HashSet::new();
    records
        .into_iter()
        .skip(3)
        .map(|(record, _, _)| {
            let key = key(&record);
            if !seen.insert(key.clone()) {
                return Err("Duplicate translation identity".into());
            }
            Ok(Row {
                key,
                character: record[0].into(),
                context: format!("{} / {}", &record[2], &record[3]),
                english: record[4].into(),
                hebrew: record[5].into(),
            })
        })
        .collect()
}

fn tokens(text: &str) -> Vec<String> {
    let mut output = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '{' || chars[i] == '<' {
            let close = if chars[i] == '{' { '}' } else { '>' };
            if let Some(end) = chars[i + 1..].iter().position(|c| *c == close) {
                output.push(chars[i..=i + 1 + end].iter().collect());
                i += end + 2;
                continue;
            }
        }
        if chars[i] == '%' && i + 1 < chars.len() && "sdif".contains(chars[i + 1]) {
            output.push(chars[i..i + 2].iter().collect());
            i += 2;
            continue;
        }
        i += 1;
    }
    output.sort();
    output
}

/// Preserve raw bytes of untouched records, including multiline fields and CRLF.
pub fn apply(source: &str, edits: &BTreeMap<String, String>) -> Result<String, String> {
    if edits.len() > 200 {
        return Err("Submit at most 200 corrections at a time".into());
    }
    let allowed: BTreeMap<_, _> = rows(source)?
        .into_iter()
        .map(|r| (r.key, r.hebrew))
        .collect();
    for (key, value) in edits {
        let original = allowed.get(key).ok_or("Unknown translation key")?;
        if value.trim().is_empty() || value.len() > 16000 || value.contains('\0') {
            return Err("Corrections must contain text and be shorter than 16 KB".into());
        }
        if tokens(original) != tokens(value) {
            return Err("Keep the existing formatting and control tokens".into());
        }
    }
    let mut output = String::with_capacity(source.len());
    let mut end = 0;
    for (index, (record, start, stop)) in read(source)?.into_iter().enumerate() {
        let replacement = if index >= 3 {
            edits
                .get(&key(&record))
                .filter(|v| v.as_str() != &record[5])
        } else {
            None
        };
        if let Some(value) = replacement {
            let raw = &source[start..stop];
            let ending = if raw.ends_with("\r\n") {
                "\r\n"
            } else if raw.ends_with('\n') {
                "\n"
            } else {
                ""
            };
            let mut fields: Vec<String> = record.iter().map(str::to_owned).collect();
            fields[5] = value.clone();
            output.push_str(
                &fields
                    .iter()
                    .map(|f| {
                        if f.contains([',', '"', '\n', '\r']) {
                            format!("\"{}\"", f.replace('"', "\"\""))
                        } else {
                            f.clone()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(","),
            );
            output.push_str(ending);
        } else {
            output.push_str(&source[start..stop]);
        }
        end = stop;
    }
    output.push_str(&source[end..]);
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    const SOURCE: &str =
        include_str!("../../../../../patches-workspace/drifter/he/payload/translation.csv");
    #[test]
    fn full_roundtrip_and_single_change() {
        let rows = rows(SOURCE).unwrap();
        assert_eq!(rows.len(), 9799);
        assert_eq!(apply(SOURCE, &BTreeMap::new()).unwrap(), SOURCE);
        let row = rows
            .iter()
            .find(|r| r.english == "Ok" && r.context == "Room- Title / ContinueGame")
            .unwrap();
        let edits = BTreeMap::from([(row.key.clone(), "רדסב".into())]);
        let actual = apply(SOURCE, &edits).unwrap();
        let expected = SOURCE.replacen(
            ",1,Room- Title,ContinueGame,Ok,רושיא",
            ",1,Room- Title,ContinueGame,Ok,רדסב",
            1,
        );
        let first = actual
            .bytes()
            .zip(expected.bytes())
            .position(|(a, b)| a != b);
        assert!(
            actual == expected,
            "byte mismatch {:?}; lengths {} / {}; nearby {:?} / {:?}",
            first,
            actual.len(),
            expected.len(),
            first.map(|p| &actual.as_bytes()[p.saturating_sub(15)..(p + 20).min(actual.len())]),
            first.map(|p| &expected.as_bytes()[p.saturating_sub(15)..(p + 20).min(expected.len())])
        );
    }
    #[test]
    fn invalid_edits_and_quoted_newlines() {
        assert!(apply(SOURCE, &BTreeMap::from([("missing".into(), "x".into())])).is_err());
        let row = &rows(SOURCE).unwrap()[0];
        assert!(apply(SOURCE, &BTreeMap::from([(row.key.clone(), "".into())])).is_err());
        let value = "שורה, \"אחת\"\nושנייה";
        let output = apply(SOURCE, &BTreeMap::from([(row.key.clone(), value.into())])).unwrap();
        assert_eq!(rows(&output).unwrap()[0].hebrew, value);
        assert_eq!(tokens("a {name} <b>%s</b>"), tokens("<b>%s</b> {name} b"));
        assert_ne!(tokens("{name}"), tokens("{other}"));
    }
}
