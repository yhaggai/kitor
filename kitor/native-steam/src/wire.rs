//! Minimal, bounded protobuf wire codec for Steam's protocol messages.
use anyhow::{bail, ensure, Result};
#[derive(Default, Clone)]
pub struct Message(pub Vec<u8>);
impl Message {
    pub fn fixed32(mut self, tag: u32, n: u32) -> Self {
        put(&mut self.0, ((tag as u64) << 3) | 5);
        self.0.extend(n.to_le_bytes());
        self
    }
    pub fn number(mut self, tag: u32, n: u64) -> Self {
        put(&mut self.0, (tag as u64) << 3);
        put(&mut self.0, n);
        self
    }
    pub fn fixed(mut self, tag: u32, n: u64) -> Self {
        put(&mut self.0, ((tag as u64) << 3) | 1);
        self.0.extend(n.to_le_bytes());
        self
    }
    pub fn bytes(mut self, tag: u32, b: impl AsRef<[u8]>) -> Self {
        let b = b.as_ref();
        put(&mut self.0, ((tag as u64) << 3) | 2);
        put(&mut self.0, b.len() as u64);
        self.0.extend(b);
        self
    }
}
fn put(out: &mut Vec<u8>, mut n: u64) {
    while n > 127 {
        out.push((n as u8) | 128);
        n >>= 7;
    }
    out.push(n as u8);
}
fn take<'a>(data: &mut &'a [u8], len: usize) -> Result<&'a [u8]> {
    ensure!(len <= data.len(), "Truncated Steam message");
    let (a, b) = data.split_at(len);
    *data = b;
    Ok(a)
}
fn varint(data: &mut &[u8]) -> Result<u64> {
    let mut n = 0;
    for shift in (0..70).step_by(7) {
        let b = take(data, 1)?[0];
        ensure!(shift < 63 || b < 2, "Invalid Steam integer");
        n |= ((b & 127) as u64) << shift;
        if b & 128 == 0 {
            return Ok(n);
        }
    }
    bail!("Invalid Steam integer")
}
#[derive(Clone)]
pub enum Value {
    Number(u64),
    Bytes(Vec<u8>),
}
#[derive(Default, Clone)]
pub struct Fields(pub Vec<(u32, Value)>);
impl Fields {
    pub fn parse(mut data: &[u8]) -> Result<Self> {
        ensure!(
            data.len() <= 32 * 1024 * 1024,
            "Steam message exceeds limit"
        );
        let mut f = Vec::new();
        while !data.is_empty() {
            let key = varint(&mut data)?;
            ensure!(key >> 3 > 0, "Invalid Steam field");
            let v = match key & 7 {
                0 => Value::Number(varint(&mut data)?),
                1 => Value::Number(u64::from_le_bytes(take(&mut data, 8)?.try_into()?)),
                2 => {
                    let len = usize::try_from(varint(&mut data)?)?;
                    Value::Bytes(take(&mut data, len)?.to_vec())
                }
                5 => Value::Number(u32::from_le_bytes(take(&mut data, 4)?.try_into()?) as u64),
                _ => bail!("Unsupported Steam field"),
            };
            f.push(((key >> 3) as u32, v));
            ensure!(f.len() < 1_000_000, "Too many Steam fields");
        }
        Ok(Self(f))
    }
    pub fn n(&self, tag: u32) -> u64 {
        self.0
            .iter()
            .find_map(|(t, v)| match v {
                Value::Number(n) if *t == tag => Some(*n),
                _ => None,
            })
            .unwrap_or(0)
    }
    pub fn b(&self, tag: u32) -> &[u8] {
        self.0
            .iter()
            .find_map(|(t, v)| match v {
                Value::Bytes(b) if *t == tag => Some(b.as_slice()),
                _ => None,
            })
            .unwrap_or_default()
    }
    pub fn s(&self, tag: u32) -> Result<String> {
        Ok(std::str::from_utf8(self.b(tag))?.to_owned())
    }
    pub fn repeated(&self, tag: u32) -> impl Iterator<Item = &[u8]> {
        self.0.iter().filter_map(move |(t, v)| match v {
            Value::Bytes(b) if *t == tag => Some(b.as_slice()),
            _ => None,
        })
    }
}
