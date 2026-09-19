use serde_json::{Value, json};
use tradex::ControlPlane;

fn command(control: &mut ControlPlane, name: &str, payload: Value) -> Value {
    control.dispatch(json!({
        "requestId": format!("screener-persistence-{name}"),
        "schemaVersion": 1,
        "command": name,
        "payload": payload,
    }))
}

fn boundary_state(control: &mut ControlPlane, workspace_id: &str) -> Value {
    // The public IPC surface has no outbox, approval, arming, or credential read
    // command. Domain snapshots plus these sanitized projections are the complete
    // observable boundary for this read-only slice; the screener commands do not
    // have a path to mutate the private credential store or those future domains.
    json!({
        "domain": command(control, "domain.snapshot", json!({
            "aggregateType": "workspace",
            "aggregateId": workspace_id
        })),
        "accounts": command(control, "account.list", json!({"workspaceId": workspace_id})),
        "risk": command(control, "risk.get_policy", json!({"workspaceId": workspace_id})),
        "gateway": command(control, "model.get_gateway", json!({"workspaceId": workspace_id})),
        "model": command(control, "model.get", json!({"workspaceId": workspace_id}))
    })
}

fn parse(control: &mut ControlPlane, workspace_id: &str, natural_language: &str) -> Value {
    command(
        control,
        "market.screen",
        json!({
            "workspaceId": workspace_id,
            "operation": "PARSE",
            "naturalLanguage": natural_language,
            "focus": "EQUITY"
        }),
    )
}

fn definition(parsed: &Value) -> Value {
    json!({
        "naturalLanguage": parsed["data"]["naturalLanguage"],
        "focus": parsed["data"]["focus"],
        "filterSpec": parsed["data"]["filterSpec"],
        "rankSpec": parsed["data"]["rankSpec"],
        "revision": parsed["data"]["revision"],
        "limit": 10
    })
}

#[test]
fn screener_library_is_workspace_scoped_versioned_and_reopenable() {
    let directory = tempfile::tempdir().unwrap();
    let workspace_path = directory.path().join("workspace");
    let mut control = ControlPlane::new(workspace_path.clone());
    let opened = command(&mut control, "workspace.open", json!({}));
    assert_eq!(opened["ok"], true, "{opened}");
    let workspace_id = opened["data"]["workspaceId"].as_str().unwrap();
    let initial = command(
        &mut control,
        "screener.list",
        json!({"workspaceId": workspace_id}),
    );
    assert_eq!(initial["ok"], true, "{initial}");
    assert_eq!(initial["data"]["screeners"].as_array().unwrap().len(), 0);
    assert!(
        initial["data"]["stateVersion"]
            .as_str()
            .unwrap()
            .ends_with(":0")
    );
    let boundary_before = boundary_state(&mut control, workspace_id);

    let natural_language = "US large-cap technology stocks with revenue growth above 15% and positive estimate revisions.";
    let parsed = parse(&mut control, workspace_id, natural_language);
    assert_eq!(parsed["data"]["state"], "PARSED", "{parsed}");
    let saved = command(
        &mut control,
        "screener.save",
        json!({
            "workspaceId": workspace_id,
            "name": "Growth leaders",
            "definition": definition(&parsed),
            "state": "PARSED",
            "expectedStateVersion": initial["data"]["stateVersion"]
        }),
    );
    assert_eq!(saved["ok"], true, "{saved}");
    assert_eq!(
        boundary_state(&mut control, workspace_id),
        boundary_before,
        "screener projection writes must not mutate financial/runtime boundaries"
    );
    assert_eq!(saved["data"]["screeners"].as_array().unwrap().len(), 1);
    let saved_version = saved["data"]["stateVersion"].as_str().unwrap().to_owned();
    let screener_id = saved["data"]["screeners"][0]["screenerId"]
        .as_str()
        .unwrap()
        .to_owned();

    let updated_natural_language = "US large-cap technology stocks with revenue growth above 20% and positive estimate revisions.";
    let updated_parse = parse(&mut control, workspace_id, updated_natural_language);
    assert_eq!(updated_parse["data"]["state"], "PARSED", "{updated_parse}");
    let updated = command(
        &mut control,
        "screener.update",
        json!({
            "workspaceId": workspace_id,
            "screenerId": screener_id,
            "name": "Growth leaders revised",
            "definition": definition(&updated_parse),
            "state": "PARSED",
            "expectedStateVersion": saved_version
        }),
    );
    assert_eq!(updated["ok"], true, "{updated}");
    assert_eq!(
        updated["data"]["screeners"][0]["name"],
        "Growth leaders revised"
    );
    assert_ne!(
        updated["data"]["screeners"][0]["definition"]["revision"],
        parsed["data"]["revision"]
    );

    let stale = command(
        &mut control,
        "screener.update",
        json!({
            "workspaceId": workspace_id,
            "screenerId": screener_id,
            "name": "Should fail",
            "definition": definition(&parsed),
            "state": "PARSED",
            "expectedStateVersion": saved_version
        }),
    );
    assert_eq!(stale["ok"], false, "{stale}");
    assert_eq!(stale["error"]["code"], "STATE_VERSION_CONFLICT");

    drop(control);
    let mut reopened_control = ControlPlane::new(workspace_path);
    let reopened = command(&mut reopened_control, "workspace.open", json!({}));
    assert_eq!(reopened["ok"], true, "{reopened}");
    let persisted = command(
        &mut reopened_control,
        "screener.list",
        json!({"workspaceId": workspace_id}),
    );
    assert_eq!(persisted["ok"], true, "{persisted}");
    assert_eq!(persisted["data"]["screeners"].as_array().unwrap().len(), 1);
    assert_eq!(
        persisted["data"]["screeners"][0]["definition"]["naturalLanguage"],
        updated_natural_language
    );
}

#[test]
fn screener_attach_returns_only_selected_canonical_contexts() {
    let directory = tempfile::tempdir().unwrap();
    let mut control = ControlPlane::new(directory.path().join("workspace"));
    let opened = command(&mut control, "workspace.open", json!({}));
    let workspace_id = opened["data"]["workspaceId"].as_str().unwrap();
    let boundary_before = boundary_state(&mut control, workspace_id);
    let revision = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let attached = command(
        &mut control,
        "screener.attach",
        json!({
            "workspaceId": workspace_id,
            "revision": revision,
            "selectedInstrumentIds": ["equity:US:AAPL"]
        }),
    );
    assert_eq!(attached["ok"], true, "{attached}");
    assert_eq!(attached["data"]["contextRefs"].as_array().unwrap().len(), 1);
    assert_eq!(attached["data"]["contextRefs"][0]["kind"], "instrument");
    assert_eq!(attached["data"]["contextRefs"][0]["id"], "equity:US:AAPL");
    assert_eq!(
        boundary_state(&mut control, workspace_id),
        boundary_before,
        "screener attachment must not mutate financial/runtime boundaries"
    );
    let contexts = attached["data"]["contextRefs"].clone();
    let capability = command(
        &mut control,
        "agent.capabilities",
        json!({
            "workspaceId": workspace_id,
            "agentMode": "ASK",
            "executionContext": "NONE_READ_ONLY",
            "attachedContexts": contexts
        }),
    );
    assert_eq!(capability["ok"], true, "{capability}");

    let duplicate = command(
        &mut control,
        "screener.attach",
        json!({
            "workspaceId": workspace_id,
            "revision": revision,
            "selectedInstrumentIds": ["equity:US:AAPL", "equity:US:AAPL"]
        }),
    );
    assert_eq!(duplicate["ok"], false, "{duplicate}");
    assert_eq!(duplicate["error"]["code"], "IPC_PAYLOAD_INVALID");
    let forged = command(
        &mut control,
        "agent.capabilities",
        json!({
            "workspaceId": workspace_id,
            "agentMode": "ASK",
            "executionContext": "NONE_READ_ONLY",
            "attachedContexts": [{"kind":"instrument","id":"equity:US:AAPL","hash":"sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}]
        }),
    );
    assert_eq!(forged["ok"], false, "{forged}");
    assert_eq!(forged["error"]["code"], "TURN_CONTEXT_INVALID");
}
