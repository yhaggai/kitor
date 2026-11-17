use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashSet},
    sync::LazyLock,
};
use unicode_normalization::UnicodeNormalization;

#[derive(Deserialize)]
pub struct Catalog {
    pub id: String,
    pub path: String,
    header: Vec<String>,
    key: usize,
    speaker: Option<usize>,
    context: Vec<usize>,
    english: Option<usize>,
    editable: Vec<usize>,
    skip: usize,
    status: Option<usize>,
    #[serde(default)]
    locked_keys: Vec<String>,
    tokens: String,
}

static CATALOGS: LazyLock<Vec<Catalog>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../../../contribution-catalogs.json"))
        .expect("valid contribution catalog configuration")
});

pub fn catalog(id: &str) -> Result<&'static Catalog, String> {
    CATALOGS
        .iter()
        .find(|catalog| catalog.id == id)
        .ok_or_else(|| "Unknown translation catalog".into())
}

impl Catalog {
    fn drifter(&self) -> bool {
        matches!(self.id.as_str(), "drifter" | "drifter-logical")
    }

    fn row_key(&self, record: &csv::StringRecord, col: usize) -> String {
        if self.drifter() {
            key(record)
        } else {
            format!("{col}:{}", &record[self.key])
        }
    }

    fn locked(&self, record: &csv::StringRecord) -> bool {
        self.status.is_some_and(|col| &record[col] == "unchanged")
            || self
                .locked_keys
                .binary_search_by(|key| key.as_str().cmp(&record[self.key]))
                .is_ok()
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Row {
    pub key: String,
    pub character: String,
    pub context: String,
    pub english: String,
    pub hebrew: String,
    pub variant: Option<&'static str>,
}

pub fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn read(config: &Catalog, source: &str) -> Result<Vec<(csv::StringRecord, usize, usize)>, String> {
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
        if record.len() != config.header.len() {
            return Err("Unexpected number of translation CSV columns".into());
        }
        result.push((record.clone(), start, end));
        start = end;
    }
    if result.len() <= config.skip
        || result[0]
            .0
            .iter()
            .ne(config.header.iter().map(String::as_str))
    {
        return Err("Unsupported translation CSV header".into());
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

pub fn rows(config: &Catalog, source: &str) -> Result<Vec<Row>, String> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for (record, _, _) in read(config, source)?.into_iter().skip(config.skip) {
        let id = if config.drifter() {
            key(&record)
        } else {
            record[config.key].to_owned()
        };
        if id.is_empty() || !seen.insert(id) {
            return Err("Duplicate or missing translation identity".into());
        }
        if config.locked(&record) {
            continue;
        }
        for (index, &col) in config.editable.iter().enumerate() {
            result.push(Row {
                key: config.row_key(&record, col),
                character: config
                    .speaker
                    .map(|speaker| record[speaker].to_owned())
                    .unwrap_or_default(),
                context: config
                    .context
                    .iter()
                    .map(|&c| &record[c])
                    .collect::<Vec<_>>()
                    .join(" / "),
                english: config
                    .english
                    .map(|en| record[en].to_owned())
                    .unwrap_or_default(),
                hebrew: record[col].into(),
                variant: (config.editable.len() == 2).then_some(if index == 0 {
                    "Regular Hebrew"
                } else {
                    "Hebrew with niqqud"
                }),
            });
        }
    }
    Ok(result)
}

fn tokens<'a>(pattern: &Regex, text: &'a str) -> Vec<&'a str> {
    let mut result: Vec<_> = pattern.find_iter(text).map(|m| m.as_str()).collect();
    result.sort_unstable();
    result
}

fn is_mark(c: char) -> bool {
    matches!(c, '\u{0591}'..='\u{05bd}' | '\u{05bf}' | '\u{05c1}'..='\u{05c2}' | '\u{05c4}'..='\u{05c5}' | '\u{05c7}')
}

fn same_wording(config: &Catalog, regular: &str, pointed: &str) -> bool {
    if regular.chars().any(is_mark) {
        return false;
    }
    if config.id == "owlboy" {
        regular.chars().eq(pointed.chars().filter(|&c| !is_mark(c)))
    } else {
        regular.nfd().eq(pointed.nfd().filter(|&c| !is_mark(c)))
    }
}

/// Preserve raw bytes of untouched records, including multiline fields and CRLF.
/// In-progress drafts can save one cell at a time before paired Hebrew is ready.
pub fn apply(
    config: &Catalog,
    source: &str,
    edits: &BTreeMap<String, String>,
    review: bool,
) -> Result<String, String> {
    if edits.len() > 200 {
        return Err("Submit at most 200 corrections at a time".into());
    }
    let records = read(config, source)?;
    let pattern = Regex::new(&config.tokens).map_err(|_| "Invalid catalog token rule")?;
    let mut output = String::with_capacity(source.len());
    let mut end = 0;
    let mut found = HashSet::new();
    for (record, start, stop) in records.into_iter().skip(config.skip) {
        let mut fields: Vec<_> = record.iter().collect();
        let mut changed = false;
        for &col in &config.editable {
            let key = config.row_key(&record, col);
            let Some(value) = edits.get(&key) else {
                continue;
            };
            if !found.insert(key) {
                return Err("Duplicate translation identity".into());
            }
            let before = fields[col];
            if value == before {
                continue;
            }
            if config.locked(&record) {
                return Err("This line is not editable in the current patch".into());
            }
            if value.trim().is_empty() || value.len() > 16000 || value.contains('\0') {
                return Err("Corrections must contain text and be shorter than 16 KB".into());
            }
            let baseline = if before.is_empty() {
                config.english.map(|col| fields[col]).unwrap_or("")
            } else {
                before
            };
            if (config.id != "drifter"
                && value.contains([
                    '\u{202a}', '\u{202b}', '\u{202c}', '\u{202d}', '\u{202e}', '\u{2066}',
                    '\u{2067}', '\u{2068}', '\u{2069}',
                ]))
                || (config.id.starts_with("undertale") && value.contains(['\r', '\n']))
                || (config.id == "owlboy"
                    && before.split_once('|').map(|(_, suffix)| suffix)
                        != value.split_once('|').map(|(_, suffix)| suffix))
                || tokens(&pattern, baseline) != tokens(&pattern, value)
            {
                return Err("Keep existing formatting and control tokens".into());
            }
            fields[col] = value;
            changed = true;
        }
        if !changed {
            continue;
        }
        if review
            && config.editable.len() == 2
            && !same_wording(
                config,
                fields[config.editable[0]],
                fields[config.editable[1]],
            )
        {
            return Err(
                "Update both Hebrew versions so their wording matches without niqqud".into(),
            );
        }
        let raw = &source[start..stop];
        let ending = if raw.ends_with("\r\n") {
            "\r\n"
        } else if raw.ends_with('\n') {
            "\n"
        } else if raw.ends_with('\r') {
            "\r"
        } else {
            ""
        };
        output.push_str(&source[end..start]);
        for (col, field) in fields.iter().enumerate() {
            if col != 0 {
                output.push(',');
            }
            if field.contains([',', '"', '\n', '\r']) {
                output.push('"');
                output.push_str(&field.replace('"', "\"\""));
                output.push('"');
            } else {
                output.push_str(field);
            }
        }
        output.push_str(ending);
        end = stop;
    }
    if found.len() != edits.len() {
        return Err("Unknown translation key".into());
    }
    output.push_str(&source[end..]);
    Ok(output)
}

#[cfg(test)]
#[path = "csv_text_tests.rs"]
mod tests;
