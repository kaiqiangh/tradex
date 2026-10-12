use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::protocol::{Result, TradeXError};

const MAGIC: &[u8; 8] = b"TRDXAR01";
const MAX_MANIFEST_BYTES: usize = 256 * 1024;
pub(crate) const MAX_ARCHIVE_BYTES: u64 = 1_073_741_824;
const MEMBER_NAME: &str = "workspace.sqlite3";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Manifest {
    pub app_id: u32,
    pub format_version: u32,
    pub workspace_id: String,
    pub workspace_name: String,
    pub captured_at: String,
    pub storage_schema_version: u32,
    pub member_name: String,
    pub member_bytes: u64,
    pub member_sha256: String,
    pub warnings: Vec<String>,
}

pub(crate) struct ValidatedArchive {
    pub manifest: Manifest,
    pub archive_bytes: u64,
    pub manifest_hash: String,
    pub member_offset: u64,
    file: File,
}

pub(crate) fn write_archive(
    snapshot_path: &Path,
    destination: &Path,
    mut manifest: Manifest,
) -> Result<(u64, String)> {
    let metadata = fs::metadata(snapshot_path)
        .map_err(|_| TradeXError::new("WORKSPACE_ARCHIVE_EXPORT_FAILED"))?;
    if metadata.len() == 0 || metadata.len() > MAX_ARCHIVE_BYTES {
        return Err(TradeXError::new("WORKSPACE_ARCHIVE_TOO_LARGE"));
    }
    if destination.exists() || fs::symlink_metadata(destination).is_ok() {
        return Err(TradeXError::new("WORKSPACE_ARCHIVE_EXISTS"));
    }
    let parent = destination
        .parent()
        .filter(|parent| parent.is_dir())
        .ok_or_else(|| TradeXError::new("WORKSPACE_PATH_INVALID"))?;
    if parent.to_str().is_none() || destination.to_str().is_none() {
        return Err(TradeXError::new("WORKSPACE_PATH_INVALID"));
    }
    manifest.member_name = MEMBER_NAME.into();
    manifest.member_bytes = metadata.len();
    manifest.member_sha256 = hash_file(snapshot_path)?;
    let manifest_bytes = serde_json::to_vec(&manifest)
        .map_err(|_| TradeXError::new("WORKSPACE_ARCHIVE_EXPORT_FAILED"))?;
    if manifest_bytes.is_empty() || manifest_bytes.len() > MAX_MANIFEST_BYTES {
        return Err(TradeXError::new("WORKSPACE_ARCHIVE_EXPORT_FAILED"));
    }
    let final_bytes = 12_u64
        .checked_add(manifest_bytes.len() as u64)
        .and_then(|size| size.checked_add(metadata.len()))
        .filter(|size| *size <= MAX_ARCHIVE_BYTES)
        .ok_or_else(|| TradeXError::new("WORKSPACE_ARCHIVE_TOO_LARGE"))?;
    let temporary = parent.join(format!(".tradex-archive-{}.tmp", Uuid::new_v4()));
    let mut published = false;
    let result = (|| {
        let mut output = private_create_new(&temporary)?;
        output
            .write_all(MAGIC)
            .and_then(|_| output.write_all(&(manifest_bytes.len() as u32).to_be_bytes()))
            .and_then(|_| output.write_all(&manifest_bytes))
            .map_err(|_| TradeXError::new("WORKSPACE_ARCHIVE_EXPORT_FAILED"))?;
        let mut input = File::open(snapshot_path)
            .map_err(|_| TradeXError::new("WORKSPACE_ARCHIVE_EXPORT_FAILED"))?;
        std::io::copy(&mut input, &mut output)
            .map_err(|_| TradeXError::new("WORKSPACE_ARCHIVE_EXPORT_FAILED"))?;
        output
            .sync_all()
            .map_err(|_| TradeXError::new("WORKSPACE_ARCHIVE_EXPORT_FAILED"))?;
        drop(output);
        // A hard link publishes the complete file atomically without replacing an existing export.
        fs::hard_link(&temporary, destination)
            .map_err(|_| TradeXError::new("WORKSPACE_ARCHIVE_EXISTS"))?;
        published = true;
        fs::remove_file(&temporary)
            .map_err(|_| TradeXError::new("WORKSPACE_ARCHIVE_EXPORT_FAILED"))?;
        sync_directory(parent)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
        if published {
            let _ = fs::remove_file(destination);
        }
    }
    result?;
    Ok((final_bytes, hash_file(destination)?))
}

pub(crate) fn validate_archive(path: &Path, current_schema: u32) -> Result<ValidatedArchive> {
    let path_metadata =
        fs::symlink_metadata(path).map_err(|_| TradeXError::new("WORKSPACE_ARCHIVE_INVALID"))?;
    if path_metadata.file_type().is_symlink() || !path_metadata.is_file() {
        return Err(TradeXError::new("WORKSPACE_ARCHIVE_INVALID"));
    }
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let mut file = options
        .open(path)
        .map_err(|_| TradeXError::new("WORKSPACE_ARCHIVE_INVALID"))?;
    let opened_metadata = file
        .metadata()
        .map_err(|_| TradeXError::new("WORKSPACE_ARCHIVE_INVALID"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if path_metadata.dev() != opened_metadata.dev()
            || path_metadata.ino() != opened_metadata.ino()
        {
            return Err(TradeXError::new("WORKSPACE_ARCHIVE_INVALID"));
        }
    }
    let archive_bytes = opened_metadata.len();
    if archive_bytes < 13 || archive_bytes > MAX_ARCHIVE_BYTES {
        return Err(TradeXError::new("WORKSPACE_ARCHIVE_INVALID"));
    }
    let mut magic = [0_u8; 8];
    file.read_exact(&mut magic)
        .map_err(|_| TradeXError::new("WORKSPACE_ARCHIVE_INVALID"))?;
    if &magic != MAGIC {
        return Err(TradeXError::new("WORKSPACE_ARCHIVE_INVALID"));
    }
    let mut length = [0_u8; 4];
    file.read_exact(&mut length)
        .map_err(|_| TradeXError::new("WORKSPACE_ARCHIVE_INVALID"))?;
    let manifest_len = u32::from_be_bytes(length) as usize;
    if manifest_len == 0 || manifest_len > MAX_MANIFEST_BYTES {
        return Err(TradeXError::new("WORKSPACE_ARCHIVE_INVALID"));
    }
    let mut manifest_bytes = vec![0_u8; manifest_len];
    file.read_exact(&mut manifest_bytes)
        .map_err(|_| TradeXError::new("WORKSPACE_ARCHIVE_INVALID"))?;
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|_| TradeXError::new("WORKSPACE_ARCHIVE_INVALID"))?;
    let member_offset = 12 + manifest_len as u64;
    let expected_size = member_offset
        .checked_add(manifest.member_bytes)
        .ok_or_else(|| TradeXError::new("WORKSPACE_ARCHIVE_INVALID"))?;
    if manifest.app_id != 0x5452_5831
        || manifest.format_version != 1
        || manifest.workspace_id.is_empty()
        || manifest.workspace_id.len() > 128
        || manifest.workspace_name.trim().is_empty()
        || manifest.workspace_name.chars().count() > 120
        || manifest.workspace_name.chars().any(char::is_control)
        || time::OffsetDateTime::parse(
            &manifest.captured_at,
            &time::format_description::well_known::Rfc3339,
        )
        .is_err()
        || manifest.storage_schema_version == 0
        || manifest.storage_schema_version > current_schema
        || manifest.member_name != MEMBER_NAME
        || manifest.member_bytes == 0
        || manifest.member_bytes > MAX_ARCHIVE_BYTES
        || manifest.member_sha256.len() != 64
        || !manifest
            .member_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        || manifest.warnings.len() > 8
        || manifest.warnings.iter().any(|warning| {
            warning.trim().is_empty()
                || warning.len() > 512
                || warning.chars().any(char::is_control)
        })
        || expected_size != archive_bytes
    {
        return Err(TradeXError::new("WORKSPACE_ARCHIVE_INVALID"));
    }
    let member_hash = hash_range(&mut file, member_offset, manifest.member_bytes)?;
    if member_hash != manifest.member_sha256 {
        return Err(TradeXError::new("WORKSPACE_ARCHIVE_HASH_MISMATCH"));
    }
    let manifest_hash = hex::encode(Sha256::digest(&manifest_bytes));
    Ok(ValidatedArchive {
        manifest,
        archive_bytes,
        manifest_hash,
        member_offset,
        file,
    })
}

pub(crate) fn extract_member(archive: &mut ValidatedArchive, destination: &Path) -> Result<()> {
    archive
        .file
        .seek(SeekFrom::Start(archive.member_offset))
        .map_err(|_| TradeXError::new("WORKSPACE_ARCHIVE_INVALID"))?;
    let mut output = private_create_new(destination)?;
    let mut limited = Read::by_ref(&mut archive.file).take(archive.manifest.member_bytes);
    let copied = std::io::copy(&mut limited, &mut output)
        .map_err(|_| TradeXError::new("WORKSPACE_ARCHIVE_INVALID"))?;
    if copied != archive.manifest.member_bytes {
        return Err(TradeXError::new("WORKSPACE_ARCHIVE_INVALID"));
    }
    output
        .sync_all()
        .map_err(|_| TradeXError::new("WORKSPACE_ARCHIVE_INVALID"))?;
    drop(output);
    if hash_file(destination)? != archive.manifest.member_sha256 {
        let _ = fs::remove_file(destination);
        return Err(TradeXError::new("WORKSPACE_ARCHIVE_HASH_MISMATCH"));
    }
    Ok(())
}

fn hash_file(path: &Path) -> Result<String> {
    let mut file = File::open(path).map_err(|_| TradeXError::new("WORKSPACE_ARCHIVE_INVALID"))?;
    let mut hash = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|_| TradeXError::new("WORKSPACE_ARCHIVE_INVALID"))?;
        if read == 0 {
            break;
        }
        hash.update(&buffer[..read]);
    }
    Ok(hex::encode(hash.finalize()))
}

fn hash_range(file: &mut File, offset: u64, length: u64) -> Result<String> {
    file.seek(SeekFrom::Start(offset))
        .map_err(|_| TradeXError::new("WORKSPACE_ARCHIVE_INVALID"))?;
    let mut limited = file.take(length);
    let mut hash = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    let mut remaining = length;
    while remaining > 0 {
        let buffer_len = buffer.len().min(remaining as usize);
        let read = limited
            .read(&mut buffer[..buffer_len])
            .map_err(|_| TradeXError::new("WORKSPACE_ARCHIVE_INVALID"))?;
        if read == 0 {
            return Err(TradeXError::new("WORKSPACE_ARCHIVE_INVALID"));
        }
        hash.update(&buffer[..read]);
        remaining -= read as u64;
    }
    Ok(hex::encode(hash.finalize()))
}

fn private_create_new(path: &Path) -> Result<File> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options
        .open(path)
        .map_err(|_| TradeXError::new("WORKSPACE_ARCHIVE_EXPORT_FAILED"))
}

fn sync_directory(path: &Path) -> Result<()> {
    File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(|_| TradeXError::new("WORKSPACE_ARCHIVE_EXPORT_FAILED"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn write_test_archive(path: &Path, manifest: &serde_json::Value, member: &[u8]) {
        let manifest = serde_json::to_vec(manifest).unwrap();
        let mut file = File::create(path).unwrap();
        file.write_all(MAGIC).unwrap();
        file.write_all(&(manifest.len() as u32).to_be_bytes())
            .unwrap();
        file.write_all(&manifest).unwrap();
        file.write_all(member).unwrap();
    }

    fn manifest(member: &[u8]) -> serde_json::Value {
        json!({
            "appId": 0x5452_5831_u32,
            "formatVersion": 1,
            "workspaceId": "workspace-test",
            "workspaceName": "Test workspace",
            "capturedAt": "2026-10-12T00:00:00Z",
            "storageSchemaVersion": 38,
            "memberName": MEMBER_NAME,
            "memberBytes": member.len(),
            "memberSha256": hex::encode(Sha256::digest(member)),
            "warnings": []
        })
    }

    #[test]
    fn rejects_traversal_unknown_manifest_fields_and_future_schemas() {
        let directory = tempfile::tempdir().unwrap();
        let member = b"bounded member";

        let traversal = directory.path().join("traversal.tradex");
        let mut traversal_manifest = manifest(member);
        traversal_manifest["memberName"] = "../workspace.sqlite3".into();
        write_test_archive(&traversal, &traversal_manifest, member);
        assert!(validate_archive(&traversal, 38).is_err());

        let unknown = directory.path().join("unknown-field.tradex");
        let mut unknown_manifest = manifest(member);
        unknown_manifest["unreviewed"] = true.into();
        write_test_archive(&unknown, &unknown_manifest, member);
        assert!(validate_archive(&unknown, 38).is_err());

        let future = directory.path().join("future-schema.tradex");
        let mut future_manifest = manifest(member);
        future_manifest["storageSchemaVersion"] = 39.into();
        write_test_archive(&future, &future_manifest, member);
        assert!(validate_archive(&future, 38).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_archive_paths_before_reading_the_target() {
        let directory = tempfile::tempdir().unwrap();
        let target = directory.path().join("target.tradex");
        fs::write(&target, b"not an archive").unwrap();
        let link = directory.path().join("linked.tradex");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        assert!(validate_archive(&link, 38).is_err());
    }

    #[test]
    fn export_never_replaces_an_existing_destination() {
        let directory = tempfile::tempdir().unwrap();
        let snapshot = directory.path().join("snapshot.sqlite3");
        fs::write(&snapshot, b"sqlite snapshot").unwrap();
        let destination = directory.path().join("existing.tradex");
        fs::write(&destination, b"keep this file").unwrap();
        let result = write_archive(
            &snapshot,
            &destination,
            serde_json::from_value(manifest(b"ignored")).unwrap(),
        );
        assert!(result.is_err());
        assert_eq!(fs::read(destination).unwrap(), b"keep this file");
    }
}
