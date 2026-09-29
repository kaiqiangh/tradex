pub(super) fn s27_stop_model(control: &mut ControlPlane, workspace_id: &str) {
    let gateway = control.store.as_ref().unwrap().gateway().unwrap();
    let job = control.prepare_gateway(&request("model.gateway", json!({
        "workspaceId":workspace_id,"expectedStateVersion":gateway.state_version,"action":"PROBE"
    }))).unwrap().unwrap();
    let mut stopped = job.state.clone();
    stopped.status = gateway::GatewayStatus::Stopped;
    stopped.model_available = false;
    stopped.error_code = Some("MODEL_UNAVAILABLE".into());
    assert_eq!(control.complete_gateway(&job, stopped)["ok"], true);
}

pub(super) fn s27_send_exact_cancellation(control: &mut ControlPlane, attempt_id: &str) {
    use provider_io::{ProviderEndpoint, ProviderHttp, ProviderHttpMethod, ProviderHttpResponse};
    struct Http {
        remote_id: u64,
        calls: std::cell::RefCell<Vec<(ProviderHttpMethod, String)>>,
    }
    impl ProviderHttp for Http {
        fn get(
            &self,
            endpoint: ProviderEndpoint,
            path: &str,
            headers: reqwest::header::HeaderMap,
        ) -> Result<Vec<u8>> {
            self.request(endpoint, ProviderHttpMethod::Get, path, headers, None)
                .map(|reply| reply.body)
        }
        fn request(
            &self,
            endpoint: ProviderEndpoint,
            method: ProviderHttpMethod,
            path: &str,
            headers: reqwest::header::HeaderMap,
            body: Option<&Value>,
        ) -> Result<ProviderHttpResponse> {
            assert_eq!(endpoint, ProviderEndpoint::Trading212Live);
            assert!(headers.contains_key(reqwest::header::AUTHORIZATION));
            assert!(body.is_none());
            self.calls.borrow_mut().push((method, path.into()));
            let (status, value) = match (method, path) {
                (ProviderHttpMethod::Get, "/api/v0/equity/account/summary") => {
                    (200, json!({"id":self.remote_id}))
                }
                (ProviderHttpMethod::Get, "/api/v0/equity/orders/123456") => (
                    200,
                    json!({
                        "id":123456,"ticker":"AAPL_US_EQ","side":"BUY","type":"LIMIT",
                        "timeInForce":"DAY","strategy":"QUANTITY","quantity":1,"filledQuantity":0,
                        "filledValue":0,"currency":"GBP","limitPrice":180,"status":"NEW",
                        "createdAt":"2026-09-28T11:00:00Z"
                    }),
                ),
                (ProviderHttpMethod::Delete, "/api/v0/equity/orders/123456") => (204, Value::Null),
                _ => panic!("unexpected cancellation request: {method:?} {path}"),
            };
            Ok(ProviderHttpResponse {
                status,
                body: serde_json::to_vec(&value).unwrap(),
            })
        }
    }
    let package = control
        .live_dispatch_package(attempt_id, "s27-model-offline-gateway")
        .unwrap();
    let order_gateway::GatewayDispatchIntent::Cancel(intent) = &package.intent else {
        panic!("expected exact cancellation")
    };
    let http = Http {
        remote_id: package
            .account
            .data
            .as_ref()
            .unwrap()
            .remote_account_id
            .parse()
            .unwrap(),
        calls: Default::default(),
    };
    let mutation = provider_io::prepare_trading212_live_mutation(
        &package.account,
        &package.credential_reference,
        provider_io::PrivilegedLiveOperation::Cancel(intent),
        &ResolutionVault,
        &http,
    )
    .unwrap();
    control
        .begin_live_execution_submission(&package.grant.grant_id, "s27-model-offline-gateway")
        .unwrap();
    let outcome = mutation.send(&http);
    let completed = control
        .complete_live_execution_submission(attempt_id, &outcome)
        .unwrap();
    assert_eq!(
        completed.attempt.state,
        protocol::ExecutionAttemptState::CancelPending
    );
    assert_eq!(completed.attempt.broker_order_id.as_deref(), Some("123456"));
    assert_eq!(
        *http.calls.borrow(),
        vec![
            (
                ProviderHttpMethod::Get,
                "/api/v0/equity/account/summary".into()
            ),
            (
                ProviderHttpMethod::Get,
                "/api/v0/equity/orders/123456".into()
            ),
            (
                ProviderHttpMethod::Delete,
                "/api/v0/equity/orders/123456".into()
            ),
        ]
    );
    assert!(completed.reservation.is_none());
}

fn s27_begin_model_turn(
    control: &mut ControlPlane,
    workspace_id: &str,
    title: &str,
) -> PreparedTurn {
    let created = dispatch(
        control,
        "thread.create",
        json!({
            "workspaceId":workspace_id,"title":title,"defaultAgentMode":"ASK",
            "defaultExecutionContext":"NONE_READ_ONLY","linkedContexts":[]
        }),
    );
    assert_eq!(created["ok"], true, "{created}");
    control.begin_turn(serde_json::from_value(json!({
        "workspaceId":workspace_id,"threadId":created["data"]["threadId"],
        "expectedStateVersion":created["data"]["stateVersion"],"message":"Read-only model recovery test",
        "agentMode":"ASK","executionContext":"NONE_READ_ONLY"
    })).unwrap()).unwrap().0
}

#[test]
fn s27_model_failure_preserves_live_authority_and_reconciliation() {
    use gateway::GatewayStatus;

    for (status, code, category) in [
        (
            GatewayStatus::Stopped,
            "MODEL_UNAVAILABLE",
            "MODEL_UNAVAILABLE",
        ),
        (
            GatewayStatus::Unauthorized,
            "GATEWAY_UNAUTHORIZED",
            "MODEL_UNAVAILABLE",
        ),
        (
            GatewayStatus::PortConflict,
            "GATEWAY_PORT_CONFLICT",
            "MODEL_UNAVAILABLE",
        ),
        (
            GatewayStatus::Running,
            "MODEL_QUOTA_EXCEEDED",
            "QUOTA_EXCEEDED",
        ),
        (
            GatewayStatus::Running,
            "MODEL_OAUTH_EXPIRED",
            "OAUTH_EXPIRED",
        ),
    ] {
        let (folder, mut control, workspace_id, _, proposal, review) =
            reviewed_live_fixture_with_numeric_binance_id(true);
        let (unknown_id, unknown_account, _) =
            make_unknown_binance_live_attempt(&mut control, &workspace_id, &proposal, &review);
        let mut healthy = control
            .seed_live_arming_fixture(&workspace_id, "trading212", "Independent safety account")
            .unwrap();
        healthy.data.as_mut().unwrap().balances = vec![Balance {
            asset: "USD".into(),
            available: "1000".into(),
            total: Some("1000".into()),
            reserved: Some("0".into()),
            in_pies: None,
            locked: None,
            restricted_available: None,
        }];
        healthy.last_successful_sync = Some(control.time.status(&workspace_id).unwrap().wall_clock);
        control
            .store
            .as_mut()
            .unwrap()
            .save_account(healthy.clone())
            .unwrap();
        let healthy = control
            .store
            .as_ref()
            .unwrap()
            .account(&healthy.connection_id)
            .unwrap();
        // The shared proposal fixture starts its deterministic clock at 100ms.
        control.time.set_test_time(
            OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000,
            100,
        );
        control.time.revalidate(&workspace_id).unwrap();
        let _earlier_proposal = live_proposal(&mut control, &workspace_id, &healthy);
        let healthy = control
            .store
            .as_ref()
            .unwrap()
            .account(&healthy.connection_id)
            .unwrap();
        control.time.set_test_time(
            OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000,
            110,
        );
        assert_eq!(
            dispatch(
                &mut control,
                "time.revalidate",
                json!({"workspaceId":workspace_id})
            )["data"]["confidence"],
            "TRUSTED"
        );
        let healthy_proposal = live_proposal(&mut control, &workspace_id, &healthy);
        let healthy_review = dispatch_main(
            &mut control,
            "trade.request_approval",
            json!({
                "workspaceId":workspace_id,"proposalId":healthy_proposal["proposalId"]
            }),
        );
        assert_eq!(healthy_review["data"]["eligible"], true, "{healthy_review}");

        let route = model::ModelRoute {
            provider: model::ModelProvider::Chatgpt,
            model_id: "gpt-5.6-sol".into(),
            thinking_type: None,
            verified_at: Some(storage::timestamp().unwrap()),
        };
        let mut model = control.store.as_ref().unwrap().model().unwrap();
        model.chatgpt.configured = true;
        model.chatgpt.status = model::ModelHealth::Ready;
        model.chatgpt.routes = vec![route.clone()];
        model.default_route = Some(route.selection());
        model.current_route = Some(route.clone());
        model.deepseek.configured = true;
        model.deepseek.status = model::ModelHealth::Ready;
        model.deepseek.routes = vec![model::ModelRoute {
            provider: model::ModelProvider::Deepseek,
            model_id: "deepseek-v4-flash".into(),
            thinking_type: Some(model::ThinkingType::Disabled),
            verified_at: Some(storage::timestamp().unwrap()),
        }];
        assert!(!model.automatic_fallback);
        control
            .store
            .as_mut()
            .unwrap()
            .save_model(model.clone(), "model.provider.changed")
            .unwrap();
        let affected = s27_begin_model_turn(&mut control, &workspace_id, "Affected model turn");
        let unaffected =
            s27_begin_model_turn(&mut control, &workspace_id, "Independent model turn");
        let other_turn = serde_json::to_value(
            control
                .store
                .as_ref()
                .unwrap()
                .thread(&unaffected.thread_id)
                .unwrap(),
        )
        .unwrap();
        let account_snapshot =
            serde_json::to_value(control.store.as_ref().unwrap().accounts().unwrap()).unwrap();
        let policy_snapshot =
            serde_json::to_value(control.store.as_ref().unwrap().risk().unwrap()).unwrap();
        let capability_request = CapabilityQuery {
            workspace_id: workspace_id.clone(),
            agent_mode: protocol::AgentMode::Trade,
            execution_context: protocol::ExecutionContext::Trading212Live,
            account_id: Some(healthy.connection_id.clone()),
            requested_level: None,
            requested_tool: None,
            attached_contexts: Vec::new(),
        };
        let capability =
            serde_json::to_value(control.capability_decision(&capability_request).unwrap())
                .unwrap();
        let limited_account = format!("s27-limited-{}", uuid::Uuid::new_v4());
        provider_io::record_provider_retry_after(
            "trading212",
            Some(&limited_account),
            Some(u64::MAX),
        );
        let budget = provider_io::provider_retry_after_seconds("trading212", &limited_account);
        assert_eq!(budget, Some(u64::MAX));
        let gateway = control.store.as_ref().unwrap().gateway().unwrap();
        let job = control.prepare_gateway(&request("model.gateway", json!({
            "workspaceId":workspace_id,"expectedStateVersion":gateway.state_version,"action":"PROBE"
        }))).unwrap().unwrap();
        let mut failed_gateway = job.state.clone();
        failed_gateway.status = status;
        failed_gateway.model_available = false;
        failed_gateway.error_code = Some(code.into());
        assert_eq!(control.complete_gateway(&job, failed_gateway)["ok"], true);
        let error = TradeXError::new(code);
        assert_eq!(error.category, category);
        assert!(!error.remediation_actions.is_empty());
        control.finish_runtime_turn(&affected, Err(error)).unwrap();
        let failed = control
            .store
            .as_ref()
            .unwrap()
            .thread(&affected.thread_id)
            .unwrap();
        assert_eq!(failed.turns[0].status, protocol::TurnStatus::Failed);
        assert_eq!(
            failed.turns[0]
                .provider_attempts
                .last()
                .unwrap()
                .error_code
                .as_deref(),
            Some(code)
        );
        assert_eq!(
            serde_json::to_value(
                control
                    .store
                    .as_ref()
                    .unwrap()
                    .thread(&unaffected.thread_id)
                    .unwrap()
            )
            .unwrap(),
            other_turn
        );
        assert_eq!(
            serde_json::to_value(control.store.as_ref().unwrap().accounts().unwrap()).unwrap(),
            account_snapshot,
            "{code}"
        );
        assert_eq!(
            serde_json::to_value(control.store.as_ref().unwrap().risk().unwrap()).unwrap(),
            policy_snapshot
        );
        assert_eq!(
            serde_json::to_value(control.capability_decision(&capability_request).unwrap())
                .unwrap(),
            capability
        );
        assert_eq!(
            provider_io::provider_retry_after_seconds("trading212", &limited_account),
            budget
        );
        assert_eq!(
            provider_io::provider_retry_after_seconds("trading212", &healthy.connection_id),
            None
        );
        let persisted_model = control.store.as_ref().unwrap().model().unwrap();
        assert_eq!(persisted_model.default_route, model.default_route);
        assert_eq!(persisted_model.current_route, Some(route.clone()));
        assert!(!persisted_model.automatic_fallback);
        assert!(
            persisted_model.fallback_for(&route, category).is_none(),
            "Failure cannot silently choose the available DeepSeek route"
        );
        let opted_in = dispatch(
            &mut control,
            "model.set_fallback_policy",
            json!({
                "workspaceId":workspace_id,"expectedStateVersion":persisted_model.state_version,"automaticFallback":true
            }),
        );
        assert_eq!(opted_in["ok"], true, "{opted_in}");
        let opted_in = control.store.as_ref().unwrap().model().unwrap();
        assert_eq!(
            opted_in.fallback_for(&route, category).unwrap().provider,
            model::ModelProvider::Deepseek
        );
        assert_eq!(opted_in.default_route, persisted_model.default_route);
        assert_eq!(
            serde_json::to_value(control.capability_decision(&capability_request).unwrap())
                .unwrap(),
            capability
        );
        assert_eq!(
            dispatch(
                &mut control,
                "model.set_fallback_policy",
                json!({
                    "workspaceId":workspace_id,"expectedStateVersion":opted_in.state_version,"automaticFallback":false
                })
            )["ok"],
            true
        );

        let (_, _, prepared) = issue_and_prepare_live_place(
            &mut control,
            &workspace_id,
            &healthy_proposal,
            &healthy_review["data"],
            "offline-model-place",
        );
        assert_eq!(prepared["data"]["attempt"]["state"], "RESERVED");
        let unknown = control
            .store
            .as_ref()
            .unwrap()
            .execution_preparation_for_attempt(&workspace_id, &unknown_id)
            .unwrap();
        let job = control.prepare_provider_for(&request("trade.resolution_evidence.refresh", json!({
            "workspaceId":workspace_id,"executionAttemptId":unknown_id,
            "accountId":unknown_account.connection_id,"expectedAttemptStateVersion":unknown.attempt.state_version
        })), "main").unwrap().unwrap();
        let http = BinanceLiveResolutionHttp {
            remote_account_id: unknown_account
                .data
                .as_ref()
                .unwrap()
                .remote_account_id
                .clone(),
            order_status: 429,
            order: json!({}),
            calls: Default::default(),
        };
        let outcome = std::thread::scope(|scope| {
            let (started, ready) = mpsc::channel();
            let finished = Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let mut releases = Vec::new();
            for _ in 0..3 {
                let (release, wait) = mpsc::channel::<()>();
                releases.push(release);
                let started = started.clone();
                let finished = finished.clone();
                scope.spawn(move || {
                    provider_io::with_provider_slot(
                        "binance",
                        provider_io::ProviderPriority::P3,
                        &|| true,
                        || {
                            started.send(()).unwrap();
                            let _ = wait.recv_timeout(std::time::Duration::from_secs(10));
                            finished.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        },
                    )
                    .unwrap()
                });
            }
            for _ in 0..3 {
                ready
                    .recv_timeout(std::time::Duration::from_secs(10))
                    .unwrap();
            }
            // The exact P0 read must progress while all non-P0 slots are occupied.
            let outcome = job.run(
                &ResolutionVault,
                |_| unreachable!(),
                &http,
                || control.provider_job_current(&job),
            );
            assert_eq!(
                finished.load(std::sync::atomic::Ordering::SeqCst),
                0,
                "P0 reconciliation waited for low-priority work to release its slots"
            );
            drop(releases);
            outcome
        });
        let reconciled = control.complete_provider(&job, outcome);
        assert_eq!(reconciled["ok"], true, "{reconciled}");
        assert_eq!(
            reconciled["data"]["ledger"]["evidence"][0]["outcome"],
            "INCONCLUSIVE"
        );
        assert!(
            http.calls
                .borrow()
                .iter()
                .all(|(_, method, _)| *method == provider_io::ProviderHttpMethod::Get)
        );
        let unknown = control
            .store
            .as_ref()
            .unwrap()
            .execution_preparation_for_attempt(&workspace_id, &unknown_id)
            .unwrap();
        assert_eq!(
            unknown.attempt.state,
            protocol::ExecutionAttemptState::UnknownReconciling
        );
        assert_eq!(
            unknown.reservation.unwrap().status,
            protocol::ExecutionReservationStatus::Active
        );
        control.disarm_live_for_safety("SESSION_RESUMED").unwrap();
        let plan = control
            .prepare_startup_live_recovery_plan()
            .unwrap()
            .unwrap();
        for id in [&unknown_account.connection_id, &healthy.connection_id] {
            let account = control.store.as_ref().unwrap().account(id).unwrap();
            assert_eq!(account.health.arming, "DISARMED");
            assert!(plan.account_ids.contains(&account.connection_id));
        }
        assert_eq!(
            control
                .store
                .as_ref()
                .unwrap()
                .execution_preparation_for_attempt(&workspace_id, &unknown_id)
                .unwrap()
                .reservation
                .unwrap()
                .status,
            protocol::ExecutionReservationStatus::Active
        );
        drop(control);
        let mut reopened = ControlPlane::new(folder.path().to_path_buf());
        assert_eq!(
            dispatch(&mut reopened, "workspace.open", json!({}))["ok"],
            true
        );
        for id in [&unknown_account.connection_id, &healthy.connection_id] {
            assert_eq!(
                reopened
                    .store
                    .as_ref()
                    .unwrap()
                    .account(id)
                    .unwrap()
                    .health
                    .arming,
                "DISARMED"
            );
        }
        let unknown = reopened
            .store
            .as_ref()
            .unwrap()
            .execution_preparation_for_attempt(&workspace_id, &unknown_id)
            .unwrap();
        assert_eq!(
            unknown.attempt.state,
            protocol::ExecutionAttemptState::UnknownReconciling
        );
        assert_eq!(
            unknown.reservation.unwrap().status,
            protocol::ExecutionReservationStatus::Active
        );
        let gateway = reopened.store.as_ref().unwrap().gateway().unwrap();
        let job = reopened.prepare_gateway(&request("model.gateway", json!({
            "workspaceId":workspace_id,"expectedStateVersion":gateway.state_version,"action":"PROBE"
        }))).unwrap().unwrap();
        let mut recovered = job.state.clone();
        recovered.status = GatewayStatus::Running;
        recovered.model_available = true;
        recovered.error_code = None;
        assert_eq!(reopened.complete_gateway(&job, recovered)["ok"], true);
        for id in [&unknown_account.connection_id, &healthy.connection_id] {
            assert_eq!(
                reopened
                    .store
                    .as_ref()
                    .unwrap()
                    .account(id)
                    .unwrap()
                    .health
                    .arming,
                "DISARMED"
            );
        }
        let recovered_model = reopened.store.as_ref().unwrap().model().unwrap();
        assert_eq!(recovered_model.default_route, model.default_route);
        assert!(!recovered_model.automatic_fallback);
    }
}
