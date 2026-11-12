use crate::{wire::Fields, Event};
use aes::cipher::{block_padding::Pkcs7, BlockDecrypt, BlockDecryptMut, KeyInit, KeyIvInit};
use anyhow::{bail, ensure, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use futures::{stream, StreamExt, TryStreamExt};
use sha1::{Digest, Sha1};
use std::{
    collections::HashSet,
    io::{Cursor, Read, Seek, SeekFrom, Write},
    path::{Component, Path},
};

fn decrypt(data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    ensure!(
        data.len() >= 32 && data.len() % 16 == 0 && key.len() == 32,
        "Invalid encrypted Steam content"
    );
    let aes = aes::Aes256::new_from_slice(key).map_err(|_| anyhow::anyhow!("Invalid depot key"))?;
    let mut iv = aes::cipher::Block::<aes::Aes256>::clone_from_slice(&data[..16]);
    aes.decrypt_block(&mut iv);
    cbc::Decryptor::<aes::Aes256>::new_from_slices(key, &iv)
        .map_err(|_| anyhow::anyhow!("Invalid depot key"))?
        .decrypt_padded_vec_mut::<Pkcs7>(&data[16..])
        .map_err(|_| anyhow::anyhow!("Steam content decryption failed"))
}
fn unzip(data: &[u8], limit: u64) -> Result<Vec<u8>> {
    let mut archive = zip::ZipArchive::new(Cursor::new(data))?;
    ensure!(archive.len() == 1, "Unexpected Steam archive");
    let f = archive.by_index(0)?;
    ensure!(f.size() <= limit, "Steam archive exceeds size limit");
    let mut out = vec![];
    f.take(limit + 1).read_to_end(&mut out)?;
    ensure!(
        out.len() as u64 <= limit,
        "Steam archive exceeds size limit"
    );
    Ok(out)
}
async fn fetch(
    client: &reqwest::Client,
    hosts: &[String],
    path: &str,
    limit: usize,
) -> Result<Vec<u8>> {
    for attempt in 0..3 {
        for host in hosts.iter().take(4) {
            let request = client.get(format!("https://{host}/{path}"));
            let Ok(r) = request.send().await else {
                continue;
            };
            if !r.status().is_success() {
                continue;
            }
            if r.content_length().is_some_and(|n| n > limit as u64) {
                continue;
            }
            let mut stream = r.bytes_stream();
            let mut b = Vec::new();
            let mut failed = false;
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(chunk) => {
                        ensure!(
                            chunk.len() <= limit.saturating_sub(b.len()),
                            "Steam content exceeds size limit"
                        );
                        b.extend_from_slice(&chunk);
                    }
                    Err(_) => {
                        failed = true;
                        break;
                    }
                }
            }
            if !failed {
                return Ok(b);
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(1 << attempt)).await;
    }
    bail!("Steam content download failed after retries")
}
struct Chunk {
    sha: Vec<u8>,
    offset: u64,
    size: u64,
    compressed: u64,
}
struct File {
    path: String,
    size: u64,
    sha: Vec<u8>,
    flags: u64,
    chunks: Vec<Chunk>,
}
fn parse(data: &[u8], key: &[u8], depot: u32, manifest: u64) -> Result<Vec<File>> {
    let data = if data.starts_with(b"PK") {
        unzip(data, 32 * 1024 * 1024)?
    } else {
        data.to_vec()
    };
    let mut pos = 0;
    let mut payload = None;
    let mut metadata = None;
    let mut ended = false;
    while pos + 4 <= data.len() {
        let magic = u32::from_le_bytes(data[pos..pos + 4].try_into()?);
        pos += 4;
        if magic == 0x32C415AB {
            ended = true;
            break;
        }
        ensure!(pos + 4 <= data.len(), "Truncated Steam manifest");
        let n = u32::from_le_bytes(data[pos..pos + 4].try_into()?) as usize;
        pos += 4;
        ensure!(n <= data.len() - pos, "Truncated Steam manifest");
        match magic {
            0x71F617D0 => payload = Some(Fields::parse(&data[pos..pos + n])?),
            0x1F4812BE => metadata = Some(Fields::parse(&data[pos..pos + n])?),
            0x1B81B817 => {}
            _ => bail!("Unknown Steam manifest section"),
        };
        pos += n;
    }
    ensure!(ended && pos == data.len(), "Incomplete Steam manifest");
    let meta = metadata.context("Missing Steam manifest metadata")?;
    ensure!(
        meta.n(1) == depot as u64 && meta.n(2) == manifest,
        "Steam manifest identity mismatch"
    );
    let mut files = vec![];
    let mut seen = HashSet::new();
    for mapping in payload
        .context("Missing Steam manifest payload")?
        .repeated(1)
    {
        let f = Fields::parse(mapping)?;
        let flags = f.n(3);
        ensure!(
            flags & 512 == 0 && f.b(7).is_empty(),
            "Steam symbolic links are not supported by this installer"
        );
        let name = f.s(1)?;
        let name = if meta.n(4) != 0 {
            let b = decrypt(
                &STANDARD.decode(
                    name.bytes()
                        .filter(|b| !b.is_ascii_whitespace())
                        .collect::<Vec<_>>(),
                )?,
                key,
            )?;
            String::from_utf8(b.into_iter().take_while(|b| *b != 0).collect())?
        } else {
            name
        };
        let name = name.replace('\\', "/");
        ensure!(
            !name.is_empty()
                && !name.contains(':')
                && !name.contains('\0')
                && Path::new(&name)
                    .components()
                    .all(|c| matches!(c, Component::Normal(_))),
            "Unsafe path in Steam manifest"
        );
        ensure!(
            seen.insert(name.to_lowercase()),
            "Duplicate Steam manifest path"
        );
        let mut chunks = vec![];
        let mut offset = 0u64;
        for c in f.repeated(6) {
            let c = Fields::parse(c)?;
            let size = c.n(4);
            ensure!(
                c.b(1).len() == 20 && size <= 16 * 1024 * 1024 && c.n(5) <= 32 * 1024 * 1024,
                "Invalid Steam chunk layout"
            );
            chunks.push(Chunk {
                sha: c.b(1).to_vec(),
                offset: c.n(3),
                size,
                compressed: c.n(5),
            });
        }
        chunks.sort_by_key(|chunk| chunk.offset);
        for chunk in &chunks {
            ensure!(
                chunk.offset == offset,
                "Steam file has overlapping or missing chunks"
            );
            offset = offset
                .checked_add(chunk.size)
                .context("Steam file too large")?;
        }
        if flags & 64 == 0 {
            ensure!(
                offset == f.n(2) && f.b(5).len() == 20,
                "Incomplete Steam file mapping"
            );
        }
        files.push(File {
            path: name,
            size: f.n(2),
            sha: f.b(5).to_vec(),
            flags,
            chunks,
        });
    }
    Ok(files)
}
fn decode(data: &[u8], key: &[u8], chunk: &Chunk) -> Result<Vec<u8>> {
    let b = decrypt(data, key)?;
    let mut out = Vec::new();
    if b.starts_with(b"PK") {
        out = unzip(&b, chunk.size)?;
    } else if b.starts_with(b"VZa") && b.len() >= 22 {
        ensure!(&b[b.len() - 2..] == b"zv", "Invalid LZMA footer");
        let mut lz = Vec::new();
        lz.extend(&b[7..12]);
        lz.extend(chunk.size.to_le_bytes());
        lz.extend(&b[12..b.len() - 10]);
        lzma_rs::lzma_decompress_with_options(
            &mut Cursor::new(lz),
            &mut out,
            &lzma_rs::decompress::Options {
                memlimit: Some(32 * 1024 * 1024),
                ..Default::default()
            },
        )?;
    } else if b.starts_with(b"VSZa") && b.len() >= 23 {
        zstd::stream::read::Decoder::new(&b[8..b.len() - 15])?
            .take(chunk.size + 1)
            .read_to_end(&mut out)?;
    } else {
        bail!("Unsupported Steam chunk compression")
    }
    ensure!(
        out.len() as u64 == chunk.size && Sha1::digest(&out).as_slice() == chunk.sha,
        "Steam chunk checksum mismatch"
    );
    Ok(out)
}
pub async fn download(
    client: &reqwest::Client,
    hosts: &[String],
    key: &[u8],
    depot: u32,
    manifest: u64,
    code: u64,
    target: &Path,
    emit: &(impl Fn(Event) + Send + Sync),
) -> Result<()> {
    let raw = fetch(
        client,
        hosts,
        &format!("depot/{depot}/manifest/{manifest}/5/{code}"),
        32 * 1024 * 1024,
    )
    .await?;
    let files = parse(&raw, key, depot, manifest)?;
    let total = files
        .iter()
        .flat_map(|f| &f.chunks)
        .map(|c| c.compressed)
        .sum();
    let downloaded = std::sync::atomic::AtomicU64::new(0);
    let connections = tokio::sync::Semaphore::new(16);
    stream::iter(files.into_iter().map(Ok::<_, anyhow::Error>))
        .try_for_each_concurrent(16, |f| {
            let downloaded = &downloaded;
            let connections = &connections;
            async move {
                let path = target.join(&f.path);
                if f.flags & 64 != 0 {
                    std::fs::create_dir_all(&path)?;
                    return Ok(());
                }
                if let Some(p) = path.parent() {
                    std::fs::create_dir_all(p)?;
                }
                let mut file = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&path)?;
                file.set_len(f.size)?;
                let mut chunks = stream::iter(f.chunks.into_iter().map(|c| async move {
                    // Hold the permit until the verified bytes have been written, bounding
                    // both network concurrency and buffered decoded chunks across files.
                    let permit = connections.acquire().await?;
                    let sha = c.sha.iter().map(|b| format!("{b:02x}")).collect::<String>();
                    let b = fetch(
                        client,
                        hosts,
                        &format!("depot/{depot}/chunk/{sha}"),
                        32 * 1024 * 1024,
                    )
                    .await?;
                    let size = b.len();
                    let owned_key = key.to_vec();
                    let offset = c.offset;
                    let decoded =
                        tokio::task::spawn_blocking(move || decode(&b, &owned_key, &c)).await??;
                    Ok::<_, anyhow::Error>((offset, decoded, size, permit))
                }))
                .buffer_unordered(8);
                while let Some((offset, b, size, _permit)) = chunks.try_next().await? {
                    file.seek(SeekFrom::Start(offset))?;
                    file.write_all(&b)?;
                    let completed = downloaded
                        .fetch_add(size as u64, std::sync::atomic::Ordering::Relaxed)
                        + size as u64;
                    emit(Event::Progress {
                        downloaded_bytes: completed,
                        total_bytes: total,
                    });
                }
                file.sync_all()?;
                drop(file);
                let mut input = std::fs::File::open(&path)?;
                let mut hash = Sha1::new();
                let mut buf = [0; 65536];
                loop {
                    let n = input.read(&mut buf)?;
                    if n == 0 {
                        break;
                    }
                    hash.update(&buf[..n]);
                }
                ensure!(
                    hash.finalize().as_slice() == f.sha,
                    "Steam file checksum mismatch: {}",
                    f.path
                );
                #[cfg(unix)]
                if f.flags & 32 != 0 {
                    use std::os::unix::fs::PermissionsExt;
                    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))?;
                }
                Ok(())
            }
        })
        .await?;
    Ok(())
}
