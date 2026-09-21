use serde_json::{Value, json};
use tradex::ControlPlane;

fn command(control: &mut ControlPlane, name: &str, payload: Value) -> Value {
    control.dispatch(json!({
        "requestId": format!("watchlist-{name}"),
        "schemaVersion": 1,
        "command": name,
        "payload": payload,
    }))
}

#[test]
fn watchlists_are_versioned_ordered_idempotent_and_persistent() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("workspace");
    let mut control = ControlPlane::new(path.clone());
    let opened = command(&mut control, "workspace.open", json!({}));
    assert_eq!(opened["ok"], true, "{opened}");
    let workspace_id = opened["data"]["workspaceId"].as_str().unwrap().to_owned();

    let empty = command(
        &mut control,
        "watchlist.list",
        json!({"workspaceId": workspace_id}),
    );
    assert_eq!(empty["ok"], true, "{empty}");
    assert_eq!(empty["data"]["watchlists"].as_array().unwrap().len(), 0);

    let created = command(
        &mut control,
        "watchlist.create",
        json!({"workspaceId": workspace_id, "name": "Core equities"}),
    );
    assert_eq!(created["ok"], true, "{created}");
    let watchlist_id = created["data"]["watchlistId"].as_str().unwrap().to_owned();
    let version_one = created["data"]["stateVersion"].as_str().unwrap().to_owned();
    let duplicate = command(
        &mut control,
        "watchlist.create",
        json!({"workspaceId": workspace_id, "name": "core EQUITIES"}),
    );
    assert_eq!(duplicate["error"]["code"], "WATCHLIST_NAME_CONFLICT");

    let add_aapl = command(
        &mut control,
        "watchlist.add",
        json!({"workspaceId": workspace_id, "watchlistId": watchlist_id, "instrumentId": "equity:US:AAPL", "expectedStateVersion": version_one}),
    );
    assert_eq!(add_aapl["ok"], true, "{add_aapl}");
    let version_two = add_aapl["data"]["stateVersion"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(
        add_aapl["data"]["items"][0]["instrumentId"],
        "equity:US:AAPL"
    );

    let duplicate_add = command(
        &mut control,
        "watchlist.add",
        json!({"workspaceId": workspace_id, "watchlistId": watchlist_id, "instrumentId": "equity:US:AAPL", "expectedStateVersion": version_two}),
    );
    assert_eq!(duplicate_add["ok"], true, "{duplicate_add}");
    assert_eq!(duplicate_add["data"]["stateVersion"], version_two);

    let add_msft = command(
        &mut control,
        "watchlist.add",
        json!({"workspaceId": workspace_id, "watchlistId": watchlist_id, "instrumentId": "equity:US:MSFT", "expectedStateVersion": version_two}),
    );
    assert_eq!(
        add_msft["data"]["items"][1]["instrumentId"],
        "equity:US:MSFT"
    );
    let version_three = add_msft["data"]["stateVersion"]
        .as_str()
        .unwrap()
        .to_owned();

    let stale = command(
        &mut control,
        "watchlist.remove",
        json!({"workspaceId": workspace_id, "watchlistId": watchlist_id, "instrumentId": "equity:US:AAPL", "expectedStateVersion": version_two}),
    );
    assert_eq!(stale["error"]["code"], "STATE_VERSION_CONFLICT");
    let after_stale = command(
        &mut control,
        "watchlist.list",
        json!({"workspaceId": workspace_id}),
    );
    assert_eq!(
        after_stale["data"]["watchlists"][0]["stateVersion"],
        version_three
    );
    assert_eq!(
        after_stale["data"]["watchlists"][0]["items"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    let remove_aapl = command(
        &mut control,
        "watchlist.remove",
        json!({"workspaceId": workspace_id, "watchlistId": watchlist_id, "instrumentId": "equity:US:AAPL", "expectedStateVersion": version_three}),
    );
    let version_four = remove_aapl["data"]["stateVersion"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(
        remove_aapl["data"]["items"][0]["instrumentId"],
        "equity:US:MSFT"
    );
    let absent_remove = command(
        &mut control,
        "watchlist.remove",
        json!({"workspaceId": workspace_id, "watchlistId": watchlist_id, "instrumentId": "equity:US:AAPL", "expectedStateVersion": version_four}),
    );
    assert_eq!(absent_remove["ok"], true, "{absent_remove}");
    assert_eq!(absent_remove["data"]["stateVersion"], version_four);

    let renamed = command(
        &mut control,
        "watchlist.rename",
        json!({"workspaceId": workspace_id, "watchlistId": watchlist_id, "name": "Core + Microsoft", "expectedStateVersion": version_four}),
    );
    assert_eq!(renamed["data"]["name"], "Core + Microsoft");
    let renamed_version = renamed["data"]["stateVersion"].as_str().unwrap().to_owned();
    let unknown_instrument = command(
        &mut control,
        "watchlist.add",
        json!({"workspaceId": workspace_id, "watchlistId": watchlist_id, "instrumentId": "equity:US:ZZZZ", "expectedStateVersion": renamed_version}),
    );
    assert_eq!(
        unknown_instrument["error"]["code"],
        "MARKET_INSTRUMENT_NOT_FOUND"
    );

    drop(control);
    let mut reopened = ControlPlane::new(path.clone());
    let reopened_workspace = command(&mut reopened, "workspace.open", json!({}));
    assert_eq!(reopened_workspace["ok"], true, "{reopened_workspace}");
    let persisted = command(
        &mut reopened,
        "watchlist.list",
        json!({"workspaceId": workspace_id}),
    );
    assert_eq!(persisted["ok"], true, "{persisted}");
    assert_eq!(persisted["data"]["watchlists"].as_array().unwrap().len(), 1);
    assert_eq!(
        persisted["data"]["watchlists"][0]["name"],
        "Core + Microsoft"
    );
    assert_eq!(
        persisted["data"]["watchlists"][0]["items"][0]["instrumentId"],
        "equity:US:MSFT"
    );
    let persisted_version = persisted["data"]["watchlists"][0]["stateVersion"]
        .as_str()
        .unwrap()
        .to_owned();
    let deleted = command(
        &mut reopened,
        "watchlist.delete",
        json!({"workspaceId": workspace_id, "watchlistId": watchlist_id, "expectedStateVersion": persisted_version}),
    );
    assert_eq!(deleted["ok"], true, "{deleted}");
    assert!(deleted["data"]["watchlists"].as_array().unwrap().is_empty());
    let missing = command(
        &mut reopened,
        "watchlist.delete",
        json!({"workspaceId": workspace_id, "watchlistId": watchlist_id, "expectedStateVersion": persisted_version}),
    );
    assert_eq!(missing["error"]["code"], "WATCHLIST_NOT_FOUND");
    drop(reopened);

    let mut after_delete = ControlPlane::new(path);
    let reopened_workspace = command(&mut after_delete, "workspace.open", json!({}));
    assert_eq!(reopened_workspace["ok"], true, "{reopened_workspace}");
    let reopened_lists = command(
        &mut after_delete,
        "watchlist.list",
        json!({"workspaceId": workspace_id}),
    );
    assert_eq!(reopened_lists["ok"], true, "{reopened_lists}");
    assert!(
        reopened_lists["data"]["watchlists"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn watchlist_schema_rejects_unknown_fields_and_invalid_names() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("workspace");
    let mut control = ControlPlane::new(path.clone());
    let opened = command(&mut control, "workspace.open", json!({}));
    let workspace_id = opened["data"]["workspaceId"].clone();
    let invalid = command(
        &mut control,
        "watchlist.create",
        json!({"workspaceId": workspace_id, "name": "\n"}),
    );
    assert_eq!(invalid["error"]["code"], "IPC_PAYLOAD_INVALID");
    let unknown = command(
        &mut control,
        "watchlist.list",
        json!({"workspaceId": workspace_id, "extra": true}),
    );
    assert_eq!(unknown["error"]["code"], "IPC_PAYLOAD_INVALID");

    let unicode = command(
        &mut control,
        "watchlist.create",
        json!({"workspaceId": workspace_id, "name": "Ångström"}),
    );
    assert_eq!(unicode["ok"], true, "{unicode}");
    let unicode_duplicate = command(
        &mut control,
        "watchlist.create",
        json!({"workspaceId": workspace_id, "name": "ångström"}),
    );
    assert_eq!(
        unicode_duplicate["error"]["code"],
        "WATCHLIST_NAME_CONFLICT"
    );

    let created = command(
        &mut control,
        "watchlist.create",
        json!({"workspaceId": workspace_id, "name": "Integrity check"}),
    );
    assert_eq!(created["ok"], true, "{created}");
    drop(control);
    let database = path.join("workspace.sqlite3");
    let connection = rusqlite::Connection::open(database).unwrap();
    connection
        .execute(
            "UPDATE watchlists SET name='Tampered column' WHERE watchlist_id=?1",
            rusqlite::params![created["data"]["watchlistId"].as_str().unwrap()],
        )
        .unwrap();
    drop(connection);
    let mut tampered = ControlPlane::new(path);
    let reopened = command(&mut tampered, "workspace.open", json!({}));
    assert_eq!(reopened["ok"], true, "{reopened}");
    let listed = command(
        &mut tampered,
        "watchlist.list",
        json!({"workspaceId": workspace_id}),
    );
    assert_eq!(listed["error"]["code"], "WORKSPACE_INTEGRITY_FAILED");
}

#[test]
fn schema_six_workspaces_migrate_watchlists_transactionally() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("workspace");
    let mut control = ControlPlane::new(path.clone());
    assert_eq!(
        command(&mut control, "workspace.open", json!({}))["ok"],
        true
    );
    drop(control);
    let database = path.join("workspace.sqlite3");
    let connection = rusqlite::Connection::open(&database).unwrap();
    connection.execute("DROP TABLE watchlists", []).unwrap();
    connection.execute("DROP TABLE screeners", []).unwrap();
    connection.execute("DROP TABLE artifacts", []).unwrap();
    connection.execute("DROP TABLE order_drafts", []).unwrap();
    connection
        .execute("DROP TABLE order_proposal_events", [])
        .unwrap();
    connection
        .execute("DROP TABLE order_proposals", [])
        .unwrap();
    connection.pragma_update(None, "user_version", 6).unwrap();
    drop(connection);

    let mut migrated = ControlPlane::new(path);
    let opened = command(&mut migrated, "workspace.open", json!({}));
    assert_eq!(opened["ok"], true, "{opened}");
    assert_eq!(opened["data"]["storageSchemaVersion"], 14);
    let listed = command(
        &mut migrated,
        "watchlist.list",
        json!({"workspaceId": opened["data"]["workspaceId"]}),
    );
    assert_eq!(listed["ok"], true, "{listed}");
    assert!(listed["data"]["watchlists"].as_array().unwrap().is_empty());
    let artifacts = command(
        &mut migrated,
        "artifact.list",
        json!({"workspaceId": opened["data"]["workspaceId"]}),
    );
    assert_eq!(artifacts["ok"], true, "{artifacts}");
    assert!(
        artifacts["data"]["artifacts"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn schema_eight_workspaces_migrate_artifacts_table() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("workspace");
    let mut control = ControlPlane::new(path.clone());
    let opened = command(&mut control, "workspace.open", json!({}));
    assert_eq!(opened["ok"], true, "{opened}");
    let workspace_id = opened["data"]["workspaceId"].as_str().unwrap().to_owned();
    drop(control);

    let database = path.join("workspace.sqlite3");
    let connection = rusqlite::Connection::open(&database).unwrap();
    connection.execute("DROP TABLE artifacts", []).unwrap();
    connection.execute("DROP TABLE order_drafts", []).unwrap();
    connection
        .execute("DROP TABLE order_proposal_events", [])
        .unwrap();
    connection
        .execute("DROP TABLE order_proposals", [])
        .unwrap();
    connection.pragma_update(None, "user_version", 8).unwrap();
    drop(connection);

    let mut migrated = ControlPlane::new(path);
    let reopened = command(&mut migrated, "workspace.open", json!({}));
    assert_eq!(reopened["ok"], true, "{reopened}");
    assert_eq!(reopened["data"]["storageSchemaVersion"], 14);
    let artifacts = command(
        &mut migrated,
        "artifact.list",
        json!({"workspaceId": workspace_id}),
    );
    assert_eq!(artifacts["ok"], true, "{artifacts}");
    assert!(
        artifacts["data"]["artifacts"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}
