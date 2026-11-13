//! Parse the quoted KeyValues text in Steam PICS app responses.
use anyhow::{bail, ensure, Context, Result};
use serde_json::{Map, Value};
#[derive(PartialEq)]
enum Token {
    Text(String),
    Open,
    Close,
}
fn tokens(data: &[u8]) -> Result<Vec<Token>> {
    ensure!(
        data.len() <= 16 * 1024 * 1024,
        "Steam metadata exceeds size limit"
    );
    let text = std::str::from_utf8(data)?.trim_end_matches('\0');
    let mut chars = text.chars().peekable();
    let mut out = vec![];
    while let Some(c) = chars.next() {
        match c {
            c if c.is_whitespace() => {}
            '/' if chars.peek() == Some(&'/') => {
                for c in chars.by_ref() {
                    if c == '\n' {
                        break;
                    }
                }
            }
            '{' => out.push(Token::Open),
            '}' => out.push(Token::Close),
            '"' => {
                let mut s = String::new();
                let mut closed = false;
                while let Some(c) = chars.next() {
                    match c {
                        '"' => {
                            closed = true;
                            break;
                        }
                        '\\' => {
                            let next = chars.next().context("Truncated Steam metadata escape")?;
                            match next {
                                '"' | '\\' => s.push(next),
                                'n' => s.push('\n'),
                                't' => s.push('\t'),
                                _ => {
                                    s.push('\\');
                                    s.push(next);
                                }
                            }
                        }
                        _ => s.push(c),
                    }
                }
                ensure!(closed, "Unclosed Steam metadata string");
                out.push(Token::Text(s));
            }
            _ => {
                let mut s = String::from(c);
                while chars
                    .peek()
                    .is_some_and(|c| !c.is_whitespace() && *c != '{' && *c != '}')
                {
                    s.push(chars.next().unwrap());
                }
                out.push(Token::Text(s));
            }
        }
        ensure!(out.len() < 1_000_000, "Steam metadata has too many tokens");
    }
    Ok(out)
}
fn object(tokens: &mut std::collections::VecDeque<Token>, depth: usize) -> Result<Value> {
    ensure!(depth < 32, "Steam metadata nesting too deep");
    let mut map = Map::new();
    while let Some(token) = tokens.pop_front() {
        let key = match token {
            Token::Close if depth > 0 => return Ok(Value::Object(map)),
            Token::Text(s) => s,
            _ => bail!("Invalid Steam metadata key"),
        };
        let value = match tokens.pop_front() {
            Some(Token::Open) => object(tokens, depth + 1)?,
            Some(Token::Text(s)) => Value::String(s),
            _ => bail!("Missing Steam metadata value"),
        };
        map.insert(key, value);
    }
    ensure!(depth == 0, "Unclosed Steam metadata object");
    Ok(Value::Object(map))
}
pub fn manifest(data: &[u8], depot: u32) -> Result<u64> {
    let v = object(&mut tokens(data)?.into(), 0)?;
    let root = v.get("appinfo").unwrap_or(&v);
    let m = &root["depots"][depot.to_string()]["manifests"]["public"];
    let gid = m
        .get("gid")
        .unwrap_or(m)
        .as_str()
        .context("No public Steam manifest for this depot")?;
    Ok(gid.parse()?)
}

pub fn installed_build(data: &[u8], app_id: u32, install_dir: &str) -> Option<u64> {
    let manifest = object(&mut tokens(data).ok()?.into(), 0).ok()?;
    let state = manifest.get("AppState")?;
    let manifest_app_id = state.get("appid")?.as_str()?.parse::<u32>().ok()?;
    let manifest_dir = state.get("installdir")?.as_str()?;
    let matches_directory = if cfg!(windows) {
        manifest_dir.eq_ignore_ascii_case(install_dir)
    } else {
        manifest_dir == install_dir
    };
    if manifest_app_id != app_id || !matches_directory {
        return None;
    }
    state.get("buildid")?.as_str()?.parse().ok().filter(|id| *id > 0)
}

#[cfg(test)]
mod tests {
    use super::installed_build;

    #[test]
    fn installed_build_matches_the_game_and_ignores_the_pending_update() {
        let manifest = br#""AppState" {
            "appid" "391540"
            "installdir" "Undertale"
            "buildid" "123456"
            "TargetBuildID" "234567"
            "InstalledDepots" { "391541" { "manifest" "345678" } }
        }"#;
        assert_eq!(installed_build(manifest, 391540, "Undertale"), Some(123456));
        assert_eq!(installed_build(manifest, 391541, "Undertale"), None);
        assert_eq!(installed_build(manifest, 391540, "Other copy"), None);
    }
}
