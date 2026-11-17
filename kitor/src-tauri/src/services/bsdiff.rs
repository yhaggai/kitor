//! Apply BSDIFF40 deltas to a staging file using bounded buffers.
use bzip2::read::BzDecoder;
use std::{
    fs::File,
    io::{self, BufWriter, Read, Seek, SeekFrom, Write},
    path::Path,
};

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn number(bytes: [u8; 8]) -> i64 {
    let raw = u64::from_le_bytes(bytes);
    let magnitude = (raw & (i64::MAX as u64)) as i64;
    if raw >> 63 == 0 {
        magnitude
    } else {
        -magnitude
    }
}

fn read_number(mut input: impl Read) -> io::Result<i64> {
    let mut bytes = [0; 8];
    input.read_exact(&mut bytes)?;
    Ok(number(bytes))
}

fn length(input: impl Read) -> io::Result<u64> {
    read_number(input)?
        .try_into()
        .map_err(|_| invalid("Negative BSDIFF length"))
}

fn section(path: &Path, offset: u64, size: u64) -> io::Result<BzDecoder<io::Take<File>>> {
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(offset))?;
    Ok(BzDecoder::new(file.take(size)))
}

pub(super) fn apply(original: &Path, patch: &Path, output: &Path) -> io::Result<()> {
    let mut header = File::open(patch)?;
    let patch_len = header.metadata()?.len();
    let mut magic = [0; 8];
    header.read_exact(&mut magic)?;
    if &magic != b"BSDIFF40" {
        return Err(invalid("Invalid BSDIFF40 header"));
    }
    let control_len = length(&mut header)?;
    let diff_len = length(&mut header)?;
    let mut remaining = length(&mut header)?;
    let diff_offset = 32 + control_len;
    let extra_offset = diff_offset
        .checked_add(diff_len)
        .filter(|offset| *offset <= patch_len)
        .ok_or_else(|| invalid("BSDIFF sections exceed patch size"))?;
    let mut control = section(patch, 32, control_len)?;
    let mut diff = section(patch, diff_offset, diff_len)?;
    let mut extra = section(patch, extra_offset, patch_len - extra_offset)?;
    let mut old = File::open(original)?;
    let old_len = i128::from(old.metadata()?.len());
    let mut output = BufWriter::new(File::create(output)?);
    let mut data = [0; 32768];
    let mut source = [0; 32768];
    let mut old_pos = 0_i128;

    while remaining != 0 {
        let mut diff_count = length(&mut control)?;
        let extra_count = length(&mut control)?;
        let seek = read_number(&mut control)?;
        let written = diff_count + extra_count;
        remaining = remaining
            .checked_sub(written)
            .ok_or_else(|| invalid("BSDIFF instruction exceeds output size"))?;

        while diff_count != 0 {
            let count = diff_count.min(data.len() as u64) as usize;
            diff.read_exact(&mut data[..count])?;
            let end = old_pos + count as i128;
            // Original positions outside the file contribute zero.
            let lo = old_pos.max(0);
            let hi = end.min(old_len);
            if lo < hi {
                let start = (lo - old_pos) as usize;
                let len = (hi - lo) as usize;
                old.seek(SeekFrom::Start(lo as u64))?;
                old.read_exact(&mut source[..len])?;
                for (byte, original) in data[start..start + len].iter_mut().zip(&source[..len]) {
                    *byte = byte.wrapping_add(*original);
                }
            }
            output.write_all(&data[..count])?;
            old_pos = end;
            diff_count -= count as u64;
        }

        if io::copy(&mut (&mut extra).take(extra_count), &mut output)? != extra_count {
            return Err(invalid("Truncated BSDIFF extra data"));
        }
        old_pos = old_pos
            .checked_add(i128::from(seek))
            .ok_or_else(|| invalid("BSDIFF source offset overflow"))?;
    }

    // Finish all streams so bzip2 verifies their checksums.
    for stream in [&mut control, &mut diff, &mut extra] {
        io::copy(stream, &mut io::sink())?;
    }
    output.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bzip2::{write::BzEncoder, Compression};

    fn encode_number(value: i64) -> [u8; 8] {
        (value.unsigned_abs() | (u64::from(value < 0) << 63)).to_le_bytes()
    }

    fn compress(bytes: &[u8]) -> Vec<u8> {
        let mut encoder = BzEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(bytes).unwrap();
        encoder.finish().unwrap()
    }

    #[test]
    fn applies_legacy_delta_and_rejects_corrupt_stream() {
        let fixture = tempfile::tempdir().unwrap();
        let original = fixture.path().join("original");
        let patch = fixture.path().join("delta");
        let output = fixture.path().join("output");
        std::fs::write(&original, b"ABC").unwrap();

        let instructions: Vec<u8> = [2, 1, -3, 5, 1, 0]
            .into_iter()
            .flat_map(encode_number)
            .collect();
        let control = compress(&instructions);
        let diff = compress(&[0, 1, b'!', 0, 0, 0, b'?']);
        let extra = compress(b"-.");
        let mut bytes = b"BSDIFF40".to_vec();
        for length in [control.len() as i64, diff.len() as i64, 9] {
            bytes.extend(encode_number(length));
        }
        bytes.extend(control);
        bytes.extend(diff);
        bytes.extend(extra);
        std::fs::write(&patch, &bytes).unwrap();
        apply(&original, &patch, &output).unwrap();
        assert_eq!(std::fs::read(&output).unwrap(), b"AC-!ABC?.");

        bytes.truncate(bytes.len() - 4);
        std::fs::write(&patch, bytes).unwrap();
        assert!(apply(&original, &patch, &output).is_err());
    }
}
