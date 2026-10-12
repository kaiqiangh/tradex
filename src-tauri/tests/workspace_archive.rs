use serde_json::{Value, json};
use std::{
    fs,
    fs::OpenOptions,
    io::{Read, Seek, SeekFrom, Write},
    time::{Duration, SystemTime},
};
use tradex::ControlPlane;

fn command(control: &mut ControlPlane, name: &str, payload: Value) -> Value {
    control.dispatch(json!({
        "requestId": format!("workspace-archive-{name}"),
        "schemaVersion": 1,
        "command": name,
        "payload": payload
    }))
}

#[test]
fn a_workspace_can_be_exported_previewed_and_restored_without_replacing_the_current_workspace() {
    let directory = tempfile::tempdir().unwrap();
    let current_path = directory.path().join("current");
    let archive_path = directory.path().join("backup.tradex");
    let mut control = ControlPlane::new(current_path.clone());
    let opened = command(&mut control, "workspace.open", json!({}));
    assert_eq!(opened["ok"], true, "{opened}");
    let workspace_id = opened["data"]["workspaceId"].as_str().unwrap();
    let database = rusqlite::Connection::open(current_path.join("workspace.sqlite3")).unwrap();
    database.execute(
        "INSERT INTO artifacts(artifact_id,workspace_id,kind,title,sequence,projection) VALUES('archive-research',?1,'RESEARCH','Archive research',1,?2)",
        rusqlite::params![workspace_id, json!({"createdAt":"2026-10-12T00:00:00Z","provenance":{}}).to_string()],
    ).unwrap();
    drop(database);

    let exported = command(
        &mut control,
        "workspace.archive.export",
        json!({
            "workspaceId": workspace_id,
            "destinationPath": archive_path.to_string_lossy()
        }),
    );
    assert_eq!(exported["ok"], true, "{exported}");
    assert!(archive_path.is_file());
    assert!(exported["data"]["bytes"].as_u64().unwrap() > 0);

    let preview = command(
        &mut control,
        "workspace.archive.preview",
        json!({
            "workspaceId": workspace_id,
            "archivePath": archive_path.to_string_lossy()
        }),
    );
    assert_eq!(preview["ok"], true, "{preview}");
    assert_eq!(preview["data"]["workspaceId"], workspace_id);
    assert!(preview["data"]["manifestHash"].as_str().is_some());
    assert_eq!(
        preview["data"]["dataCategories"].as_array().unwrap().len(),
        5
    );
    let research_records = preview["data"]["dataCategories"]
        .as_array()
        .unwrap()
        .iter()
        .find(|category| category["categoryId"] == "RESEARCH")
        .unwrap()["recordCount"]
        .as_u64()
        .unwrap();
    assert_eq!(research_records, 1);

    let restored = command(
        &mut control,
        "workspace.archive.restore",
        json!({
            "workspaceId": workspace_id,
            "archivePath": archive_path.to_string_lossy(),
            "destinationParent": directory.path().join("restored").to_string_lossy(),
            "manifestHash": preview["data"]["manifestHash"]
        }),
    );
    assert_eq!(restored["ok"], true, "{restored}");
    assert!(current_path.join("workspace.sqlite3").is_file());
    assert!(std::path::Path::new(restored["data"]["backupPath"].as_str().unwrap()).is_file());

    let restored_path = restored["data"]["restoredPath"].as_str().unwrap();
    assert!(
        std::path::Path::new(restored_path)
            .join("workspace.sqlite3")
            .is_file()
    );
    let reopened = command(
        &mut control,
        "workspace.open",
        json!({"path": restored_path}),
    );
    assert_eq!(reopened["ok"], true, "{reopened}");
    assert_eq!(reopened["data"]["workspaceId"], workspace_id);
}

#[test]
fn a_tampered_archive_is_rejected_before_a_restore_directory_is_created() {
    let directory = tempfile::tempdir().unwrap();
    let current_path = directory.path().join("current");
    let archive_path = directory.path().join("backup.tradex");
    let restore_parent = directory.path().join("restore-target");
    let mut control = ControlPlane::new(current_path);
    let opened = command(&mut control, "workspace.open", json!({}));
    let workspace_id = opened["data"]["workspaceId"].as_str().unwrap();
    let exported = command(
        &mut control,
        "workspace.archive.export",
        json!({"workspaceId":workspace_id,"destinationPath":archive_path.to_string_lossy()}),
    );
    assert_eq!(exported["ok"], true, "{exported}");
    let preview = command(
        &mut control,
        "workspace.archive.preview",
        json!({"workspaceId":workspace_id,"archivePath":archive_path.to_string_lossy()}),
    );
    assert_eq!(preview["ok"], true, "{preview}");

    let mut archive = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&archive_path)
        .unwrap();
    archive.seek(SeekFrom::End(-1)).unwrap();
    let mut last_byte = [0_u8; 1];
    archive.read_exact(&mut last_byte).unwrap();
    archive.seek(SeekFrom::End(-1)).unwrap();
    archive.write_all(&[last_byte[0] ^ 0x01]).unwrap();
    archive.sync_all().unwrap();

    let rejected = command(
        &mut control,
        "workspace.archive.restore",
        json!({
            "workspaceId":workspace_id,
            "archivePath":archive_path.to_string_lossy(),
            "destinationParent":restore_parent.to_string_lossy(),
            "manifestHash":preview["data"]["manifestHash"],
        }),
    );
    assert_eq!(rejected["ok"], false, "{rejected}");
    assert_eq!(rejected["error"]["code"], "WORKSPACE_ARCHIVE_HASH_MISMATCH");
    assert!(!restore_parent.exists());
}

#[test]
fn retention_expires_only_unlinked_artifacts_regular_cache_files_and_old_market_rows() {
    let directory = tempfile::tempdir().unwrap();
    let workspace_path = directory.path().join("current");
    let mut control = ControlPlane::new(workspace_path.clone());
    let opened = command(&mut control, "workspace.open", json!({}));
    assert_eq!(opened["ok"], true, "{opened}");
    let workspace_id = opened["data"]["workspaceId"].as_str().unwrap();

    let database = rusqlite::Connection::open(workspace_path.join("workspace.sqlite3")).unwrap();
    for (artifact_id, order_refs) in [
        ("expired-research", json!([])),
        ("linked-financial-evidence", json!(["order-123"])),
    ] {
        let projection = json!({
            "createdAt":"2000-01-01T00:00:00Z",
            "provenance":{"relatedOrderIds":order_refs},
        });
        database.execute(
            "INSERT INTO artifacts(artifact_id,workspace_id,kind,title,sequence,projection) VALUES(?1,?2,'RESEARCH',?3,1,?4)",
            rusqlite::params![artifact_id, workspace_id, artifact_id, projection.to_string()],
        ).unwrap();
    }
    drop(database);

    let cache = workspace_path.join("cache");
    fs::create_dir(&cache).unwrap();
    let expired_cache = cache.join("expired.cache");
    fs::write(&expired_cache, "old cache").unwrap();
    let file = OpenOptions::new().write(true).open(&expired_cache).unwrap();
    file.set_times(
        std::fs::FileTimes::new()
            .set_modified(SystemTime::now() - Duration::from_secs(10 * 24 * 60 * 60)),
    )
    .unwrap();
    let recent_cache = cache.join("recent.cache");
    fs::write(&recent_cache, "recent cache").unwrap();
    let external_cache = directory.path().join("external.cache");
    fs::write(&external_cache, "outside workspace").unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(&external_cache, cache.join("linked.cache")).unwrap();
    let nested = cache.join("nested");
    fs::create_dir(&nested).unwrap();
    fs::write(nested.join("nested.cache"), "nested cache").unwrap();

    let market_path = workspace_path.join("market.duckdb");
    let market = duckdb::Connection::open(&market_path).unwrap();
    market.execute_batch("DELETE FROM ohlcv_1m;").unwrap();
    market.execute("INSERT INTO ohlcv_1m(instrument_id,interval_start,open,high,low,close,volume,source,venue,provider_timestamp,received_timestamp,entitlement,freshness) VALUES ('equity:US:AAPL','2000-01-01T00:00:00Z','1','1','1','1','1','OD-001',NULL,'2000-01-01T00:00:00Z','2000-01-01T00:00:00Z','DELAYED','HISTORICAL')", []).unwrap();
    let current_time = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap();
    market.execute("INSERT INTO ohlcv_1m(instrument_id,interval_start,open,high,low,close,volume,source,venue,provider_timestamp,received_timestamp,entitlement,freshness) VALUES ('equity:US:AAPL',?1,'1','1','1','1','1','OD-001',NULL,?1,?1,'DELAYED','HISTORICAL')", [&current_time]).unwrap();
    drop(market);

    let settings = command(
        &mut control,
        "workspace.retention.get",
        json!({"workspaceId":workspace_id}),
    );
    assert_eq!(settings["ok"], true, "{settings}");
    let unapplied = command(
        &mut control,
        "workspace.retention.apply",
        json!({"workspaceId":workspace_id}),
    );
    assert_eq!(unapplied["ok"], true, "{unapplied}");
    assert_eq!(unapplied["data"]["lastDeletedArtifacts"], 0);
    assert!(expired_cache.is_file());
    let initial_artifact_count: i64 =
        rusqlite::Connection::open(workspace_path.join("workspace.sqlite3"))
            .unwrap()
            .query_row(
                "SELECT COUNT(*) FROM artifacts WHERE workspace_id=?1",
                [workspace_id],
                |row| row.get(0),
            )
            .unwrap();
    assert_eq!(initial_artifact_count, 2);
    let initial_market_count: i64 = duckdb::Connection::open(&market_path)
        .unwrap()
        .query_row("SELECT COUNT(*) FROM ohlcv_1m", [], |row| row.get(0))
        .unwrap();
    assert_eq!(initial_market_count, 2);
    let settings = command(
        &mut control,
        "workspace.retention.get",
        json!({"workspaceId":workspace_id}),
    );
    assert_eq!(settings["ok"], true, "{settings}");

    let saved = command(
        &mut control,
        "workspace.retention.set",
        json!({
            "workspaceId":workspace_id,
            "expectedStateVersion":settings["data"]["stateVersion"],
            "artifactRetentionDays":1,
            "cacheRetentionDays":1,
            "marketDataRetentionDays":1,
        }),
    );
    assert_eq!(saved["ok"], true, "{saved}");
    let applied = command(
        &mut control,
        "workspace.retention.apply",
        json!({"workspaceId":workspace_id}),
    );
    assert_eq!(applied["ok"], true, "{applied}");
    assert_eq!(applied["data"]["lastDeletedArtifacts"], 1);
    assert_eq!(applied["data"]["lastDeletedCacheFiles"], 1);
    assert_eq!(applied["data"]["lastDeletedMarketRows"], 1);
    assert!(!expired_cache.exists());
    assert!(recent_cache.is_file());
    assert!(external_cache.is_file());
    assert!(nested.join("nested.cache").is_file());
    #[cfg(unix)]
    assert!(
        cache
            .join("linked.cache")
            .symlink_metadata()
            .unwrap()
            .file_type()
            .is_symlink()
    );

    let remaining_artifacts: Vec<String> =
        rusqlite::Connection::open(workspace_path.join("workspace.sqlite3"))
            .unwrap()
            .prepare("SELECT artifact_id FROM artifacts WHERE workspace_id=?1 ORDER BY artifact_id")
            .unwrap()
            .query_map([workspace_id], |row| row.get(0))
            .unwrap()
            .collect::<std::result::Result<Vec<_>, _>>()
            .unwrap();
    assert_eq!(remaining_artifacts, ["linked-financial-evidence"]);
    let market = duckdb::Connection::open(market_path).unwrap();
    let remaining_market_rows: i64 = market
        .query_row("SELECT COUNT(*) FROM ohlcv_1m", [], |row| row.get(0))
        .unwrap();
    assert_eq!(remaining_market_rows, 1);
    let retention_events: i64 =
        rusqlite::Connection::open(workspace_path.join("workspace.sqlite3"))
            .unwrap()
            .query_row(
                "SELECT COUNT(*) FROM outbox WHERE aggregate_type='workspace-retention'",
                [],
                |row| row.get(0),
            )
            .unwrap();
    assert_eq!(retention_events, 3);
}

#[test]
fn schema_37_workspaces_migrate_to_retention_schema_38() {
    let directory = tempfile::tempdir().unwrap();
    let workspace_path = directory.path().join("current");
    let mut control = ControlPlane::new(workspace_path.clone());
    let opened = command(&mut control, "workspace.open", json!({}));
    assert_eq!(opened["ok"], true, "{opened}");
    drop(control);
    let database = rusqlite::Connection::open(workspace_path.join("workspace.sqlite3")).unwrap();
    database
        .execute_batch("DROP TABLE workspace_retention; PRAGMA user_version=37;")
        .unwrap();
    drop(database);

    let mut reopened = ControlPlane::new(workspace_path.clone());
    let opened = command(
        &mut reopened,
        "workspace.open",
        json!({"path":workspace_path.to_string_lossy()}),
    );
    assert_eq!(opened["ok"], true, "{opened}");
    assert_eq!(opened["data"]["storageSchemaVersion"], 38);
    let retention = command(
        &mut reopened,
        "workspace.retention.get",
        json!({"workspaceId":opened["data"]["workspaceId"]}),
    );
    assert_eq!(retention["ok"], true, "{retention}");
    assert_eq!(
        retention["data"]["stateVersion"],
        format!(
            "workspace-retention:{}:0",
            opened["data"]["workspaceId"].as_str().unwrap()
        )
    );
}
