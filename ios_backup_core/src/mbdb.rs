use std::{io::ErrorKind, path::PathBuf};

use anyhow::{Context as _, Result, anyhow};
use sha1::{Digest as _, Sha1};
use tokio::io::{AsyncRead, AsyncReadExt as _};

#[derive(Debug, Clone)]
pub struct FileRecord {
    pub domain: String,
    pub path: PathBuf,
    pub file_id: String,
    pub mode: u16,
    // pub inode: u64,
    // pub uid: u32,
    // pub gid: u32,
    // pub mtime: u32,
    // pub atime: u32,
    // pub ctime: u32,
    pub file_length: u64,
    // pub protection_class: u8,
    // pub properties: FileProperties,
}

pub async fn parse<T: AsyncRead + Unpin>(reader: &mut T) -> Result<Vec<FileRecord>> {
    let mut header = [0u8; 4];
    reader
        .read_exact(&mut header)
        .await
        .context("Failed to read header")?;

    if &header != b"mbdb" {
        return Err(anyhow!(
            "Invalid Manifest.mbdb file: incorrect magic number"
        ));
    }

    let _version = reader.read_u16().await.context("Failed to read version")?;

    let mut files = Vec::new();
    while let Some(record) = parse_file_record(reader).await? {
        if (record.mode & 0xF000) == 0x8000 {
            files.push(record);
        }
    }
    Ok(files)
}

async fn parse_file_record<T: AsyncRead + Unpin>(reader: &mut T) -> Result<Option<FileRecord>> {
    let domain_len = match reader.read_u16().await {
        Ok(len) => len,
        Err(error) if error.kind() == ErrorKind::UnexpectedEof => {
            return Ok(None);
        }
        Err(error) => return Err(error.into()),
    };

    if domain_len == 0 {
        println!("Reached end marker (domain_len=0)");
        return Ok(None);
    }
    let domain = read_string(reader, domain_len.into()).await?;
    let path = PathBuf::from(read_string_or_blank(reader).await?);
    let _linktarget = read_string_or_blank(reader).await?;
    let _datahash = read_string_or_blank(reader).await?;
    let _enckey = read_string_or_blank(reader).await?;
    let mode = reader.read_u16().await?;
    let _inode = reader.read_u64().await?;
    let _uid = reader.read_u32().await?;
    let _gid = reader.read_u32().await?;
    let _mtime = reader.read_u32().await?;
    let _atime = reader.read_u32().await?;
    let _ctime = reader.read_u32().await?;
    let file_length = reader.read_u64().await?;
    let _flag = reader.read_u8().await?;
    let num_props = reader.read_u8().await?;
    for _ in 0..num_props {
        let propname_len = reader.read_u16().await?;
        let _propname = read_string(reader, propname_len.into()).await?;
        let propval_len = reader.read_u16().await?;
        let mut propval_buf = vec![0u8; propval_len.into()];
        reader.read_exact(&mut propval_buf).await?;
    }

    let file_id = compute_file_id(&domain, &path.to_string_lossy());

    Ok(Some(FileRecord {
        domain,
        path,
        file_id,
        mode,
        file_length,
    }))
}

async fn read_string_or_blank<T: AsyncRead + Unpin>(reader: &mut T) -> Result<String> {
    let length = reader.read_u16().await?;

    if length == 0xFFFF {
        return Ok(String::new());
    }

    read_string(reader, length.into()).await
}

async fn read_string<T: AsyncRead + Unpin>(reader: &mut T, len: usize) -> Result<String> {
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).await?;
    Ok(String::from_utf8_lossy(&buf).to_string())
}

fn compute_file_id(domain: &str, path: &str) -> String {
    let combined = format!("{domain}-{path}");
    let mut hasher = Sha1::new();
    hasher.update(combined.as_bytes());
    let result = hasher.finalize();
    format!("{result:x}")
}
