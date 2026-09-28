use serde_json::{Value, json};
use std::{
    io::{self, BufRead, Write},
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tradex::{
    BacktestSupervisor, ControlPlane, RuntimeSupervisor, StrategySupervisor, protocol::EventSink,
};

#[cfg(all(feature = "integration-test", target_os = "macos"))]
#[path = "../../tests/support/fake_live_provider.rs"]
mod fake_live_provider;
#[cfg(feature = "integration-test")]
#[path = "../../tests/support/provider_fixtures.rs"]
mod fixtures;

fn main() -> io::Result<()> {
    #[cfg(feature = "integration-test")]
    let (vault, http) = (fixtures::Vault::default(), fixtures::Http::default());
    let Some(path) = std::env::args_os().nth(1) else {
        eprintln!("Usage: tradex-ipc <isolated-workspace-directory>");
        std::process::exit(2);
    };
    let control = Arc::new(Mutex::new(ControlPlane::new(PathBuf::from(path))));
    let supervisor = RuntimeSupervisor::new();
    let strategy_supervisor = StrategySupervisor::new();
    let backtest_supervisor = BacktestSupervisor::new();
    #[cfg(all(feature = "integration-test", target_os = "macos"))]
    let live_provider = if std::env::var_os("TRADEX_INTEGRATION_ORDER_GATEWAY").is_some() {
        Some(fake_live_provider::FakeLiveProvider::start()?)
    } else {
        None
    };
    #[cfg(all(feature = "integration-test", target_os = "macos"))]
    let mut order_gateway: Option<tradex::order_gateway::OrderGatewayHost> = None;
    #[cfg(all(feature = "integration-test", target_os = "macos"))]
    let runtime_path =
        std::env::temp_dir().join(format!("tradex-model-ui-{}", uuid::Uuid::new_v4()));
    #[cfg(all(feature = "integration-test", target_os = "macos"))]
    let mut gateway = tradex::gateway_process::GatewayHost::new(runtime_path.clone());
    #[cfg(all(feature = "integration-test", target_os = "macos"))]
    let model_vault = tradex::model_credentials::MemoryModelVault::default();
    let output = Arc::new(Mutex::new(io::stdout()));
    let event_output = output.clone();
    let sink: EventSink = Arc::new(move |event| {
        write_frame(&event_output, &json!({"kind":"event", "event":event})).is_ok()
    });
    let mut input = io::stdin().lock();
    let mut frame = Vec::new();
    let mut oversized = false;
    loop {
        let bytes = input.fill_buf()?;
        if bytes.is_empty() {
            break;
        }
        let newline = bytes.iter().position(|b| *b == b'\n');
        let length = newline.map_or(bytes.len(), |n| n + 1);
        if frame.len() + length > 65_536 {
            oversized = true;
        }
        if !oversized {
            frame.extend_from_slice(&bytes[..length]);
        }
        input.consume(length);
        if newline.is_some() {
            let request: Value = if oversized {
                Value::Null
            } else {
                serde_json::from_slice(&frame).unwrap_or(Value::Null)
            };
            let command = request.get("command").and_then(Value::as_str);
            #[cfg(all(feature = "integration-test", target_os = "macos"))]
            if command == Some("live.gateway.fixture.set_result") {
                let payload = request.get("payload").unwrap_or(&Value::Null);
                let scenario = payload.get("scenario").and_then(Value::as_str);
                let result = match (live_provider.as_ref(), scenario) {
                    (Some(provider), Some("ACCEPTED")) => {
                        provider.set_next_result(fake_live_provider::MutationResult::Accepted);
                        Ok(())
                    }
                    (Some(provider), Some("REJECTED")) => {
                        provider.set_next_result(fake_live_provider::MutationResult::Rejected);
                        Ok(())
                    }
                    (Some(provider), Some("UNKNOWN")) => {
                        provider.set_next_result(fake_live_provider::MutationResult::Unknown);
                        Ok(())
                    }
                    (None, _) => Err(tradex::protocol::TradeXError::new("IPC_COMMAND_UNKNOWN")),
                    _ => Err(tradex::protocol::TradeXError::new("IPC_PAYLOAD_INVALID")),
                };
                let reply = match result {
                    Ok(()) => json!({
                        "requestId":request["requestId"],"schemaVersion":1,"ok":true,
                        "data":{"configured":true}
                    }),
                    Err(error) => json!({
                        "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                        "error":error
                    }),
                };
                write_frame(&output, &json!({"kind":"result", "result":reply}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            #[cfg(all(feature = "integration-test", target_os = "macos"))]
            if command == Some("live.gateway.fixture.inspect") {
                let reply = match live_provider.as_ref() {
                    Some(provider) => json!({
                        "requestId":request["requestId"],"schemaVersion":1,"ok":true,
                        "data":{"requests":provider.requests().into_iter().map(|request| json!({
                            "method":request.method,
                            "path":request.path,
                            "body":request.body,
                            "authorizationPresent":request.authorization_present
                        })).collect::<Vec<_>>()}
                    }),
                    None => json!({
                        "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                        "error":tradex::protocol::TradeXError::new("IPC_COMMAND_UNKNOWN")
                    }),
                };
                write_frame(&output, &json!({"kind":"result", "result":reply}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            #[cfg(feature = "integration-test")]
            if command == Some("alpaca.paper.stream.fixture") {
                let payload = request.get("payload").unwrap_or(&Value::Null);
                let connection_id = payload.get("connectionId").and_then(Value::as_str);
                let state_version = payload
                    .get("expectedConnectionStateVersion")
                    .and_then(Value::as_str);
                let remote_account_id = payload.get("remoteAccountId").and_then(Value::as_str);
                let stream_frame = payload.get("frame");
                let reply = match (
                    connection_id,
                    state_version,
                    remote_account_id,
                    stream_frame,
                ) {
                    (
                        Some(connection_id),
                        Some(state_version),
                        Some(remote_account_id),
                        Some(frame),
                    ) => match control.lock() {
                        Ok(mut control) => match control.apply_alpaca_private_stream_frame(
                            connection_id,
                            state_version,
                            remote_account_id,
                            frame,
                            &[fixtures::KEY.into(), fixtures::SECRET.into()],
                        ) {
                            Ok(()) => json!({
                                "requestId":request["requestId"],"schemaVersion":1,"ok":true,
                                "data":{"accepted":true}
                            }),
                            Err(error) => json!({
                                "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                                "error":error
                            }),
                        },
                        Err(_) => json!({
                            "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                            "error":tradex::protocol::TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE")
                        }),
                    },
                    _ => json!({
                        "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                        "error":tradex::protocol::TradeXError::new("IPC_PAYLOAD_INVALID")
                    }),
                };
                write_frame(&output, &json!({"kind":"result", "result":reply}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            #[cfg(feature = "integration-test")]
            if command == Some("alpaca.paper.stream.disconnect.fixture") {
                let payload = request.get("payload").unwrap_or(&Value::Null);
                let connection_id = payload.get("connectionId").and_then(Value::as_str);
                let state_version = payload
                    .get("expectedConnectionStateVersion")
                    .and_then(Value::as_str);
                let reply = match (connection_id, state_version) {
                    (Some(connection_id), Some(state_version)) => match control.lock() {
                        Ok(mut control) => match control.mark_alpaca_private_stream_degraded(
                            connection_id,
                            state_version,
                            "DEGRADED",
                            "Alpaca Paper private stream fixture disconnected.",
                        ) {
                            Ok(()) => json!({
                                "requestId":request["requestId"],"schemaVersion":1,"ok":true,
                                "data":{"accepted":true}
                            }),
                            Err(error) => json!({
                                "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                                "error":error
                            }),
                        },
                        Err(_) => json!({
                            "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                            "error":tradex::protocol::TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE")
                        }),
                    },
                    _ => json!({
                        "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                        "error":tradex::protocol::TradeXError::new("IPC_PAYLOAD_INVALID")
                    }),
                };
                write_frame(&output, &json!({"kind":"result", "result":reply}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            #[cfg(feature = "integration-test")]
            if command == Some("binance.testnet.cancel.fixture") {
                let payload = request.get("payload").unwrap_or(&Value::Null);
                let scenario = payload.get("scenario").and_then(Value::as_str);
                let reply = match scenario {
                    Some("PREPARE_ORDER") => {
                        let mut orders = http
                            .binance_open_orders
                            .borrow()
                            .clone()
                            .filter(|orders| !orders.is_empty())
                            .unwrap_or_else(fixtures::default_binance_open_orders);
                        let updated_at = payload
                            .get("orderUpdatedAtMs")
                            .and_then(Value::as_u64)
                            .unwrap_or(1788849506000u64);
                        if !orders
                            .iter()
                            .any(|order| order["orderId"] == 9007199254740998u64)
                        {
                            orders.push(json!({
                                "symbol":"BTCUSDT","orderId":9007199254740998u64,
                                "clientOrderId":"fixture-private-stream-order","side":"BUY","type":"LIMIT",
                                "timeInForce":"GTC","status":"PARTIALLY_FILLED","price":"90","origQty":"0.25",
                                "origQuoteOrderQty":"0","executedQty":"0.1","cummulativeQuoteQty":"9",
                                "time":updated_at.saturating_sub(70000),"updateTime":updated_at
                            }));
                        }
                        if !orders
                            .iter()
                            .any(|order| order["orderId"] == 9007199254740997u64)
                        {
                            orders.push(json!({
                                "symbol":"ETHUSDT","orderId":9007199254740997u64,
                                "clientOrderId":"fixture-cancel-eth","side":"BUY","type":"LIMIT",
                                "timeInForce":"GTC","status":"NEW","price":"90","origQty":"0.5",
                                "origQuoteOrderQty":"0","executedQty":"0","cummulativeQuoteQty":"0",
                                "time":1788849503000u64,"updateTime":1788849503000u64
                            }));
                        }
                        if !orders
                            .iter()
                            .any(|order| order["orderId"] == 9007199254740999u64)
                        {
                            orders.push(json!({
                                "symbol":"ETHUSDT","orderId":9007199254740999u64,
                                "clientOrderId":"fixture-terminal-eth","side":"BUY","type":"LIMIT",
                                "timeInForce":"GTC","status":"NEW","price":"90","origQty":"0.25",
                                "origQuoteOrderQty":"0","executedQty":"0","cummulativeQuoteQty":"0",
                                "time":1788849504000u64,"updateTime":1788849504000u64
                            }));
                        }
                        if !orders
                            .iter()
                            .any(|order| order["orderId"] == 9007199254741001u64)
                        {
                            orders.push(json!({
                                "symbol":"BTCUSDT","orderId":9007199254741001u64,
                                "clientOrderId":"fixture-cancel-btc-retry","side":"SELL","type":"LIMIT",
                                "timeInForce":"GTC","status":"NEW","price":"100","origQty":"0.05",
                                "origQuoteOrderQty":"0","executedQty":"0","cummulativeQuoteQty":"0",
                                "time":1788849505000u64,"updateTime":1788849505000u64
                            }));
                        }
                        if !orders
                            .iter()
                            .any(|order| order["orderId"] == 9007199254741002u64)
                        {
                            orders.push(json!({
                                "symbol":"BTCUSDT","orderId":9007199254741002u64,
                                "clientOrderId":"fixture-cancel-btc-unknown","side":"BUY","type":"LIMIT",
                                "timeInForce":"GTC","status":"NEW","price":"100","origQty":"0.07",
                                "origQuoteOrderQty":"0","executedQty":"0","cummulativeQuoteQty":"0",
                                "time":1788849506000u64,"updateTime":1788849506000u64
                            }));
                        }
                        if !orders
                            .iter()
                            .any(|order| order["orderId"] == 9007199254741003u64)
                        {
                            orders.push(json!({
                                "symbol":"ETHUSDT","orderId":9007199254741003u64,
                                "clientOrderId":"fixture-cancel-eth-zero-remaining","side":"BUY","type":"LIMIT",
                                "timeInForce":"GTC","status":"PARTIALLY_FILLED","price":"90","origQty":"0.25",
                                "origQuoteOrderQty":"0","executedQty":"0.25","cummulativeQuoteQty":"22.5",
                                "time":1788849507000u64,"updateTime":1788849507000u64
                            }));
                        }
                        *http.binance_open_orders.borrow_mut() = Some(orders);
                        json!({"requestId":request["requestId"],"schemaVersion":1,"ok":true,"data":{"configured":true}})
                    }
                    Some("REJECTED") => {
                        http.binance_cancel_timeout.set(false);
                        http.binance_cancel_status.set(Some(400));
                        json!({"requestId":request["requestId"],"schemaVersion":1,"ok":true,"data":{"configured":true}})
                    }
                    Some("UNKNOWN") => {
                        http.binance_cancel_status.set(None);
                        http.binance_cancel_timeout.set(true);
                        json!({"requestId":request["requestId"],"schemaVersion":1,"ok":true,"data":{"configured":true}})
                    }
                    Some("RESET") => {
                        http.binance_cancel_status.set(None);
                        http.binance_cancel_timeout.set(false);
                        json!({"requestId":request["requestId"],"schemaVersion":1,"ok":true,"data":{"configured":true}})
                    }
                    Some("INVALID_QUANTITY") | Some("TERMINAL") => {
                        let mut orders = http
                            .binance_open_orders
                            .borrow()
                            .clone()
                            .unwrap_or_else(fixtures::default_binance_open_orders);
                        let order_id = if scenario == Some("TERMINAL") {
                            9007199254740999u64
                        } else {
                            9007199254740997u64
                        };
                        let order = orders.iter_mut().find(|order| order["orderId"] == order_id);
                        let configured = order.is_some();
                        if let Some(order) = order {
                            if scenario == Some("TERMINAL") {
                                order["origQty"] = "0.25".into();
                                order["executedQty"] = "0.25".into();
                                order["cummulativeQuoteQty"] = "22.5".into();
                                order["status"] = "FILLED".into();
                            } else {
                                order["origQty"] = "-0.5".into();
                                order["executedQty"] = "0".into();
                                order["cummulativeQuoteQty"] = "0".into();
                                order["status"] = "NEW".into();
                            }
                        }
                        *http.binance_open_orders.borrow_mut() = Some(orders);
                        json!({"requestId":request["requestId"],"schemaVersion":1,"ok":configured,"data":{"configured":configured}})
                    }
                    Some("REMOVE_ORDER") => {
                        let mut orders = http
                            .binance_open_orders
                            .borrow()
                            .clone()
                            .unwrap_or_else(fixtures::default_binance_open_orders);
                        let previous_count = orders.len();
                        orders.retain(|order| order["orderId"] != 9007199254740999u64);
                        let configured = orders.len() < previous_count;
                        *http.binance_open_orders.borrow_mut() = Some(orders);
                        json!({"requestId":request["requestId"],"schemaVersion":1,"ok":configured,"data":{"configured":configured}})
                    }
                    _ => json!({
                        "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                        "error":tradex::protocol::TradeXError::new("IPC_PAYLOAD_INVALID")
                    }),
                };
                write_frame(&output, &json!({"kind":"result", "result":reply}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            #[cfg(feature = "integration-test")]
            if command == Some("workspace.ready.fixture") {
                let payload = request.get("payload").unwrap_or(&Value::Null);
                let workspace_id = payload.get("workspaceId").and_then(Value::as_str);
                let reply = match workspace_id {
                    Some(workspace_id) => match control.lock() {
                        Ok(mut control) => match control.seed_browser_workspace_ready(workspace_id)
                        {
                            Ok(()) => json!({
                                "requestId":request["requestId"],"schemaVersion":1,"ok":true,
                                "data":{"ready":true}
                            }),
                            Err(error) => json!({
                                "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                                "error":error
                            }),
                        },
                        Err(_) => json!({
                            "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                            "error":tradex::protocol::TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE")
                        }),
                    },
                    None => json!({
                        "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                        "error":tradex::protocol::TradeXError::new("IPC_PAYLOAD_INVALID")
                    }),
                };
                write_frame(&output, &json!({"kind":"result", "result":reply}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            #[cfg(feature = "integration-test")]
            if command == Some("binance.testnet.stream.fixture") {
                let payload = request.get("payload").unwrap_or(&Value::Null);
                let connection_id = payload.get("connectionId").and_then(Value::as_str);
                let state_version = payload
                    .get("expectedConnectionStateVersion")
                    .and_then(Value::as_str);
                let remote_account_id = payload.get("remoteAccountId").and_then(Value::as_str);
                let subscription_id = payload.get("subscriptionId").and_then(Value::as_u64);
                let stream_frame = payload.get("frame");
                let reply = match (
                    connection_id,
                    state_version,
                    remote_account_id,
                    subscription_id,
                    stream_frame,
                ) {
                    (
                        Some(connection_id),
                        Some(state_version),
                        Some(remote_account_id),
                        Some(subscription_id),
                        Some(stream_frame),
                    ) => match control.lock() {
                        Ok(mut control) => match control.apply_binance_private_stream_frame(
                            connection_id,
                            state_version,
                            remote_account_id,
                            stream_frame,
                            subscription_id,
                            &[fixtures::KEY.into(), fixtures::SECRET.into()],
                        ) {
                            Ok(needs_reconciliation) => json!({
                                "requestId":request["requestId"],"schemaVersion":1,"ok":true,
                                "data":{"accepted":true,"needsReconciliation":needs_reconciliation}
                            }),
                            Err(error) => json!({
                                "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                                "error":error
                            }),
                        },
                        Err(_) => json!({
                            "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                            "error":tradex::protocol::TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE")
                        }),
                    },
                    _ => json!({
                        "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                        "error":tradex::protocol::TradeXError::new("IPC_PAYLOAD_INVALID")
                    }),
                };
                write_frame(&output, &json!({"kind":"result", "result":reply}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            #[cfg(feature = "integration-test")]
            if command == Some("binance.testnet.stream.disconnect.fixture") {
                let payload = request.get("payload").unwrap_or(&Value::Null);
                let connection_id = payload.get("connectionId").and_then(Value::as_str);
                let state_version = payload
                    .get("expectedConnectionStateVersion")
                    .and_then(Value::as_str);
                let reply = match (connection_id, state_version) {
                    (Some(connection_id), Some(state_version)) => match control.lock() {
                        Ok(mut control) => match control.update_binance_private_stream_health(
                            connection_id,
                            state_version,
                            "DEGRADED",
                            "DEGRADED",
                            "Binance Spot Testnet private stream fixture disconnected.",
                        ) {
                            Ok(()) => json!({
                                "requestId":request["requestId"],"schemaVersion":1,"ok":true,
                                "data":{"accepted":true}
                            }),
                            Err(error) => json!({
                                "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                                "error":error
                            }),
                        },
                        Err(_) => json!({
                            "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                            "error":tradex::protocol::TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE")
                        }),
                    },
                    _ => json!({
                        "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                        "error":tradex::protocol::TradeXError::new("IPC_PAYLOAD_INVALID")
                    }),
                };
                write_frame(&output, &json!({"kind":"result", "result":reply}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            #[cfg(feature = "integration-test")]
            if command == Some("binance.testnet.stream.reconcile.fixture") {
                let payload = request.get("payload").unwrap_or(&Value::Null);
                let connection_id = payload.get("connectionId").and_then(Value::as_str);
                let state_version = payload
                    .get("expectedConnectionStateVersion")
                    .and_then(Value::as_str);
                let remote_account_id = payload.get("remoteAccountId").and_then(Value::as_str);
                let reply = match (connection_id, state_version, remote_account_id) {
                    (Some(connection_id), Some(state_version), Some(remote_account_id)) => {
                        match control.lock() {
                            Ok(mut control) => {
                                match control.binance_private_stream_order_book(connection_id) {
                                    Ok(book) => {
                                        http.mirror_binance_private_stream_book(&book);
                                        match control.reconcile_binance_private_stream_fixture(
                                            connection_id,
                                            state_version,
                                            remote_account_id,
                                            &[fixtures::KEY.into(), fixtures::SECRET.into()],
                                            &http,
                                        ) {
                                            Ok(()) => json!({
                                                "requestId":request["requestId"],"schemaVersion":1,"ok":true,
                                                "data":{"accepted":true,"reconciled":true}
                                            }),
                                            Err(error) => json!({
                                                "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                                                "error":error
                                            }),
                                        }
                                    }
                                    Err(error) => json!({
                                        "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                                        "error":error
                                    }),
                                }
                            }
                            Err(_) => json!({
                                "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                                "error":tradex::protocol::TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE")
                            }),
                        }
                    }
                    _ => json!({
                        "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                        "error":tradex::protocol::TradeXError::new("IPC_PAYLOAD_INVALID")
                    }),
                };
                write_frame(&output, &json!({"kind":"result", "result":reply}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            #[cfg(feature = "integration-test")]
            if command == Some("trading212.live.cancel.fixture.seed_order") {
                http.seed_trading212_live_cancel_order();
                write_frame(
                    &output,
                    &json!({
                        "kind":"result",
                        "result":{"requestId":request["requestId"],"schemaVersion":1,"ok":true,"data":{"seeded":true}}
                    }),
                )?;
                frame.clear();
                oversized = false;
                continue;
            }
            #[cfg(feature = "integration-test")]
            if command == Some("trading212.live.cancel.fixture.fill_race") {
                http.fill_trading212_live_cancel_race();
                write_frame(
                    &output,
                    &json!({
                        "kind":"result",
                        "result":{"requestId":request["requestId"],"schemaVersion":1,"ok":true,"data":{"filled":true}}
                    }),
                )?;
                frame.clear();
                oversized = false;
                continue;
            }
            #[cfg(feature = "integration-test")]
            if command == Some("account.cancellation.fixture.seed") {
                let payload = request.get("payload").unwrap_or(&Value::Null);
                let workspace_id = payload.get("workspaceId").and_then(Value::as_str);
                let provider_id = payload.get("providerId").and_then(Value::as_str);
                let label = payload.get("label").and_then(Value::as_str);
                let reply = match (workspace_id, provider_id, label) {
                    (Some(workspace_id), Some(provider_id), Some(label)) => match control.lock() {
                        Ok(mut control) => match control.seed_live_cancellation_fixture(
                            workspace_id,
                            provider_id,
                            label,
                        ) {
                            Ok(account) => {
                                vault.present.borrow_mut().insert(account.credential_ref());
                                json!({
                                    "requestId":request["requestId"],"schemaVersion":1,"ok":true,
                                    "data":account
                                })
                            }
                            Err(error) => json!({
                                "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                                "error":error
                            }),
                        },
                        Err(_) => json!({
                            "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                            "error":tradex::protocol::TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE")
                        }),
                    },
                    _ => json!({
                        "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                        "error":tradex::protocol::TradeXError::new("IPC_PAYLOAD_INVALID")
                    }),
                };
                write_frame(&output, &json!({"kind":"result", "result":reply}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            #[cfg(feature = "integration-test")]
            if command == Some("account.arming.fixture.seed") {
                let payload = request.get("payload").unwrap_or(&Value::Null);
                let workspace_id = payload.get("workspaceId").and_then(Value::as_str);
                let provider_id = payload.get("providerId").and_then(Value::as_str);
                let label = payload.get("label").and_then(Value::as_str);
                let reply = match (workspace_id, provider_id, label) {
                    (Some(workspace_id), Some(provider_id), Some(label)) => match control.lock() {
                        Ok(mut control) => {
                            match control.seed_live_arming_fixture(workspace_id, provider_id, label)
                            {
                                Ok(account) => {
                                    vault.present.borrow_mut().insert(account.credential_ref());
                                    if account.provider_id == "trading212"
                                        && account.environment == "LIVE"
                                        && let Some(remote_id) =
                                            account.data.as_ref().and_then(|data| {
                                                data.remote_account_id.parse::<u64>().ok()
                                            })
                                    {
                                        http.trading212_identity.set(remote_id);
                                    }
                                    json!({
                                        "requestId":request["requestId"],"schemaVersion":1,"ok":true,
                                        "data":account
                                    })
                                }
                                Err(error) => json!({
                                    "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                                    "error":error
                                }),
                            }
                        }
                        Err(_) => json!({
                            "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                            "error":tradex::protocol::TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE")
                        }),
                    },
                    _ => json!({
                        "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                        "error":tradex::protocol::TradeXError::new("IPC_PAYLOAD_INVALID")
                    }),
                };
                write_frame(&output, &json!({"kind":"result", "result":reply}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            #[cfg(feature = "integration-test")]
            if command == Some("time.fixture.advance") {
                let payload = request.get("payload").unwrap_or(&Value::Null);
                let workspace_id = payload.get("workspaceId").and_then(Value::as_str);
                let elapsed_ms = payload.get("elapsedMs").and_then(Value::as_u64);
                let reply = match (workspace_id, elapsed_ms) {
                    (Some(workspace_id), Some(elapsed_ms)) => match control.lock() {
                        Ok(mut control) => {
                            match control.advance_test_clock_fixture(workspace_id, elapsed_ms) {
                                Ok(status) => json!({
                                    "requestId":request["requestId"],"schemaVersion":1,"ok":true,
                                    "data":status
                                }),
                                Err(error) => json!({
                                    "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                                    "error":error
                                }),
                            }
                        }
                        Err(_) => json!({
                            "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                            "error":tradex::protocol::TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE")
                        }),
                    },
                    _ => json!({
                        "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                        "error":tradex::protocol::TradeXError::new("IPC_PAYLOAD_INVALID")
                    }),
                };
                write_frame(&output, &json!({"kind":"result", "result":reply}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            #[cfg(feature = "integration-test")]
            if matches!(
                command,
                Some("binance.live.reconciliation.fixture.account.seed")
                    | Some("bitget.live.reconciliation.fixture.account.seed")
            ) {
                let provider_id =
                    if command == Some("binance.live.reconciliation.fixture.account.seed") {
                        "binance"
                    } else {
                        "bitget"
                    };
                let payload = request.get("payload").unwrap_or(&Value::Null);
                let workspace_id = payload.get("workspaceId").and_then(Value::as_str);
                let label = payload.get("label").and_then(Value::as_str);
                let reply = match (workspace_id, label) {
                    (Some(workspace_id), Some(label)) => match control.lock() {
                        Ok(mut control) => match control.seed_live_cancellation_fixture(
                            workspace_id,
                            provider_id,
                            label,
                        ) {
                            Ok(account) => {
                                vault.present.borrow_mut().insert(account.credential_ref());
                                if provider_id == "binance" {
                                    http.binance_uid.set(9_007_199_254_740_993);
                                }
                                json!({
                                    "requestId":request["requestId"],"schemaVersion":1,"ok":true,
                                    "data":account
                                })
                            }
                            Err(error) => json!({
                                "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                                "error":error
                            }),
                        },
                        Err(_) => json!({
                            "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                            "error":tradex::protocol::TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE")
                        }),
                    },
                    _ => json!({
                        "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                        "error":tradex::protocol::TradeXError::new("IPC_PAYLOAD_INVALID")
                    }),
                };
                write_frame(&output, &json!({"kind":"result", "result":reply}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            #[cfg(feature = "integration-test")]
            if matches!(
                command,
                Some("binance.live.reconciliation.fixture.attempt.seed")
                    | Some("bitget.live.reconciliation.fixture.attempt.seed")
            ) {
                let provider_id =
                    if command == Some("binance.live.reconciliation.fixture.attempt.seed") {
                        "binance"
                    } else {
                        "bitget"
                    };
                let payload = request.get("payload").unwrap_or(&Value::Null);
                let workspace_id = payload.get("workspaceId").and_then(Value::as_str);
                let proposal_id = payload.get("proposalId").and_then(Value::as_str);
                let approval_id = payload.get("approvalId").and_then(Value::as_str);
                let capacity = payload
                    .get("capacityProjection")
                    .cloned()
                    .and_then(|value| serde_json::from_value(value).ok());
                let reply = match (workspace_id, proposal_id, approval_id, capacity) {
                    (Some(workspace_id), Some(proposal_id), Some(approval_id), Some(capacity)) => {
                        match control.lock() {
                            Ok(mut control) => match if provider_id == "binance" {
                                control.seed_binance_live_unknown_attempt_fixture(
                                    workspace_id,
                                    proposal_id,
                                    approval_id,
                                    capacity,
                                )
                            } else {
                                control.seed_bitget_live_unknown_attempt_fixture(
                                    workspace_id,
                                    proposal_id,
                                    approval_id,
                                    capacity,
                                )
                            } {
                                Ok(attempt) => {
                                    let submitted_at = attempt
                                        .dispatch_started_at
                                        .as_deref()
                                        .and_then(|value| {
                                            time::OffsetDateTime::parse(
                                                value,
                                                &time::format_description::well_known::Rfc3339,
                                            )
                                            .ok()
                                        })
                                        .map(|value| value.unix_timestamp_nanos() / 1_000_000)
                                        .unwrap_or_default();
                                    if provider_id == "binance" {
                                        *http.binance_order_by_client_id.borrow_mut() = Some(
                                            json!({
                                                "orderId":987654321,
                                                "symbol":"BTCUSDT",
                                                "clientOrderId":attempt.provider_client_order_id.clone(),
                                                "side":"BUY",
                                                "type":"LIMIT",
                                                "timeInForce":"GTC",
                                                "origQty":"0.01",
                                                "origQuoteOrderQty":"0.00000000",
                                                "price":"50000.00000000",
                                                "status":"NEW",
                                                "time":submitted_at
                                            }),
                                        );
                                    } else {
                                        http.bitget.set_order_info(Some(json!({
                                            "userId":"9007199254740993",
                                            "symbol":"BTCUSDT",
                                            "orderId":"987654321",
                                            "clientOid":attempt.provider_client_order_id,
                                            "price":"50000",
                                            "size":"0.01",
                                            "orderType":"limit",
                                            "side":"buy",
                                            "status":"live",
                                            "force":"gtc",
                                            "tpslType":"normal",
                                            "cTime":submitted_at.to_string()
                                        })));
                                    }
                                    json!({
                                        "requestId":request["requestId"],"schemaVersion":1,"ok":true,
                                        "data":{"attempt":attempt}
                                    })
                                }
                                Err(error) => json!({
                                    "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                                    "error":error
                                }),
                            },
                            Err(_) => json!({
                                "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                                "error":tradex::protocol::TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE")
                            }),
                        }
                    }
                    _ => json!({
                        "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                        "error":tradex::protocol::TradeXError::new("IPC_PAYLOAD_INVALID")
                    }),
                };
                write_frame(&output, &json!({"kind":"result", "result":reply}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            #[cfg(feature = "integration-test")]
            if command == Some("bitget.live.reconciliation.fixture.set_scenario") {
                let payload = request.get("payload").unwrap_or(&Value::Null);
                let scenario = payload.get("scenario").and_then(Value::as_str);
                let client_oid = payload.get("clientOid").and_then(Value::as_str);
                let submitted_at = payload.get("submittedAt").and_then(Value::as_str);
                let configured = match (scenario, client_oid, submitted_at) {
                    (Some("EXACT"), Some(client_oid), Some(submitted_at)) => {
                        http.bitget.set_order_info(Some(json!({
                            "userId":"9007199254740993",
                            "symbol":"BTCUSDT",
                            "orderId":"987654321",
                            "clientOid":client_oid,
                            "price":"50000",
                            "size":"0.01",
                            "orderType":"limit",
                            "side":"buy",
                            "status":"live",
                            "force":"gtc",
                            "tpslType":"normal",
                            "cTime":submitted_at
                        })));
                        http.bitget.set_order_info_error(None);
                        http.bitget.set_order_info_transport_error(false);
                        true
                    }
                    (Some("EMPTY"), _, _) => {
                        http.bitget.set_order_info(None);
                        true
                    }
                    (Some("AUTH_ERROR"), _, _) => {
                        http.bitget.set_order_info_error(Some("40001".into()));
                        true
                    }
                    (Some("TRANSPORT_ERROR"), _, _) => {
                        http.bitget.set_order_info_transport_error(true);
                        true
                    }
                    (Some("MISMATCH"), Some(client_oid), Some(submitted_at)) => {
                        http.bitget.set_order_info(Some(json!({
                            "userId":"42",
                            "symbol":"BTCUSDT",
                            "orderId":"987654321",
                            "clientOid":client_oid,
                            "price":"50000",
                            "size":"0.01",
                            "orderType":"limit",
                            "side":"buy",
                            "status":"live",
                            "force":"gtc",
                            "tpslType":"normal",
                            "cTime":submitted_at
                        })));
                        true
                    }
                    (Some("WRONG_SIDE"), Some(client_oid), Some(submitted_at)) => {
                        http.bitget.set_order_info(Some(json!({
                            "userId":"9007199254740993",
                            "symbol":"BTCUSDT",
                            "orderId":"987654321",
                            "clientOid":client_oid,
                            "price":"50000",
                            "size":"0.01",
                            "orderType":"limit",
                            "side":"sell",
                            "status":"live",
                            "force":"gtc",
                            "tpslType":"normal",
                            "cTime":submitted_at
                        })));
                        true
                    }
                    _ => false,
                };
                let reply = json!({
                    "requestId":request["requestId"],"schemaVersion":1,"ok":configured,
                    "data":{"configured":configured}
                });
                write_frame(&output, &json!({"kind":"result", "result":reply}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            #[cfg(feature = "integration-test")]
            if matches!(
                command,
                Some("binance.live.reconciliation.fixture.inspect")
                    | Some("bitget.live.reconciliation.fixture.inspect")
            ) {
                let reply = json!({
                    "requestId":request["requestId"],"schemaVersion":1,"ok":true,
                    "data":{"providerOrderWrites":http.binance_posts.borrow().len() + http.bitget.order_writes()}
                });
                write_frame(&output, &json!({"kind":"result", "result":reply}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            #[cfg(feature = "integration-test")]
            if command == Some("account.capacity.fixture.stale") {
                let payload = request.get("payload").unwrap_or(&Value::Null);
                let workspace_id = payload.get("workspaceId").and_then(Value::as_str);
                let connection_id = payload.get("connectionId").and_then(Value::as_str);
                let reply = match (workspace_id, connection_id) {
                    (Some(workspace_id), Some(connection_id)) => match control.lock() {
                        Ok(mut control) => match control
                            .mark_live_capacity_fixture_stale(workspace_id, connection_id)
                        {
                            Ok(account) => json!({
                                "requestId":request["requestId"],"schemaVersion":1,"ok":true,
                                "data":account
                            }),
                            Err(error) => json!({
                                "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                                "error":error
                            }),
                        },
                        Err(_) => json!({
                            "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                            "error":tradex::protocol::TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE")
                        }),
                    },
                    _ => json!({
                        "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                        "error":tradex::protocol::TradeXError::new("IPC_PAYLOAD_INVALID")
                    }),
                };
                write_frame(&output, &json!({"kind":"result", "result":reply}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            #[cfg(feature = "integration-test")]
            if command == Some("account.delete.fixture.seed") {
                let payload = request.get("payload").unwrap_or(&Value::Null);
                let workspace_id = payload.get("workspaceId").and_then(Value::as_str);
                let label = payload.get("label").and_then(Value::as_str);
                let reply = match (workspace_id, label) {
                    (Some(workspace_id), Some(label)) => match control.lock() {
                        Ok(mut control) => match control
                            .seed_trading212_demo_deletion_fixture(workspace_id, label)
                        {
                            Ok(account) => json!({
                                "requestId":request["requestId"],"schemaVersion":1,"ok":true,
                                "data":account
                            }),
                            Err(error) => json!({
                                "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                                "error":error
                            }),
                        },
                        Err(_) => json!({
                            "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                            "error":tradex::protocol::TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE")
                        }),
                    },
                    _ => json!({
                        "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                        "error":tradex::protocol::TradeXError::new("IPC_PAYLOAD_INVALID")
                    }),
                };
                write_frame(&output, &json!({"kind":"result", "result":reply}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            if command == Some("workspace.open") {
                supervisor.stop_all();
                strategy_supervisor.stop_all();
                backtest_supervisor.stop_all();
            }
            if command == Some("turn.start") || command == Some("turn.retry") {
                let result = if command == Some("turn.start") {
                    supervisor.start(control.clone(), request, None)
                } else {
                    supervisor.retry(control.clone(), request, None)
                };
                write_frame(&output, &json!({"kind":"result", "result":result}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            if command == Some("turn.cancel") {
                let result = supervisor.cancel(control.clone(), request);
                write_frame(&output, &json!({"kind":"result", "result":result}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            if command == Some("strategy.run") {
                let result = strategy_supervisor.start(control.clone(), request);
                write_frame(&output, &json!({"kind":"result", "result":result}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            if command == Some("strategy.cancel") {
                let result = strategy_supervisor.cancel(control.clone(), request);
                write_frame(&output, &json!({"kind":"result", "result":result}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            if command == Some("backtest.run") {
                let result = backtest_supervisor.start(control.clone(), request);
                write_frame(&output, &json!({"kind":"result", "result":result}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            if command == Some("backtest.cancel") {
                let result = backtest_supervisor.cancel(control.clone(), request);
                write_frame(&output, &json!({"kind":"result", "result":result}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            if command == Some("data.source.probe") {
                let prepared = match control.lock() {
                    Ok(mut control) => control.prepare_data_source_probe(&request),
                    Err(_) => Err(tradex::protocol::TradeXError::new(
                        "IPC_CONTROL_PLANE_UNAVAILABLE",
                    )),
                };
                let result = match prepared {
                    Ok(Some(job)) => {
                        let outcome =
                            tradex::data_sources::probe(&job.input.source_id, job.source.clone());
                        match control.lock() {
                            Ok(mut control) => control.complete_data_source_probe(&job, outcome),
                            Err(_) => json!({
                                "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                                "error":tradex::protocol::TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE")
                            }),
                        }
                    }
                    Ok(None) => json!({
                        "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                        "error":tradex::protocol::TradeXError::new("IPC_COMMAND_UNKNOWN")
                    }),
                    Err(error) => json!({
                        "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                        "error":error
                    }),
                };
                write_frame(&output, &json!({"kind":"result", "result":result}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            #[cfg(feature = "integration-test")]
            let result = match control.lock() {
                Ok(mut control) => match control.prepare_provider_for(&request, "stdio") {
                    Ok(Some(job)) => {
                        let outcome = job.run(
                            &vault,
                            |definition| {
                                if definition.provider_id == "bitget" {
                                    fixtures::bitget::credentials()
                                } else {
                                    fixtures::credentials()
                                }
                            },
                            &http,
                            || control.provider_job_current(&job),
                        );
                        let reply = control.complete_provider(&job, outcome);
                        if let Some(cleanup) = job.cleanup_after_failed_commit(&reply, &vault) {
                            control.record_credential_cleanup(&job, cleanup);
                        }
                        reply
                    }
                    Ok(None) => {
                        control.dispatch_with_events(request.clone(), "stdio", Some(sink.clone()))
                    }
                    Err(error) => json!({
                        "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                        "error":error
                    }),
                },
                Err(_) => json!({
                    "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                    "error":tradex::protocol::TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE")
                }),
            };
            #[cfg(not(feature = "integration-test"))]
            let result = match control.lock() {
                Ok(mut control) => {
                    control.dispatch_with_events(request.clone(), "stdio", Some(sink.clone()))
                }
                Err(_) => json!({
                    "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                    "error":tradex::protocol::TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE")
                }),
            };
            #[cfg(all(feature = "integration-test", target_os = "macos"))]
            let result = match request.get("command").and_then(Value::as_str) {
                Some("model.gateway") => match control.lock() {
                    Ok(mut control) => match control.prepare_gateway(&request) {
                        Ok(Some(job)) => {
                            let outcome = gateway.run(&job, || control.gateway_job_current(&job));
                            control.complete_gateway(&job, outcome)
                        }
                        Ok(None) => result,
                        Err(error) => {
                            json!({"requestId":request["requestId"],"schemaVersion":1,"ok":false,"error":error})
                        }
                    },
                    Err(_) => json!({
                        "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                        "error":tradex::protocol::TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE")
                    }),
                },
                Some("model.configure_deepseek") | Some("model.verify_route") => {
                    match control.lock() {
                        Ok(mut control) => match control.prepare_model(&request) {
                            Ok(Some(job)) => {
                                let outcome = tradex::model::run_job(
                                    &job,
                                    &mut gateway,
                                    &model_vault,
                                    || {
                                        tradex::model_credentials::ModelKey::new(
                                            "integration-test-key".into(),
                                        )
                                    },
                                    || control.model_job_current(&job),
                                );
                                control.complete_model(&job, outcome)
                            }
                            Ok(None) => result,
                            Err(error) => {
                                json!({"requestId":request["requestId"],"schemaVersion":1,"ok":false,"error":error})
                            }
                        },
                        Err(_) => json!({
                            "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                            "error":tradex::protocol::TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE")
                        }),
                    }
                }
                Some("model.login_chatgpt") => json!({
                    "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                    "error":tradex::protocol::TradeXError::new("MODEL_NATIVE_REQUIRED")
                }),
                _ => result,
            };
            #[cfg(all(feature = "integration-test", target_os = "macos"))]
            let result = dispatch_fake_live_execution(
                result,
                &request,
                &control,
                live_provider.as_ref(),
                &mut order_gateway,
                &sink,
            );
            write_frame(&output, &json!({"kind":"result", "result":result}))?;
            frame.clear();
            oversized = false;
        }
    }
    supervisor.stop_all();
    backtest_supervisor.stop_all();
    #[cfg(all(feature = "integration-test", target_os = "macos"))]
    if gateway.stop() {
        let _ = std::fs::remove_dir_all(runtime_path);
    }
    Ok(())
}

#[cfg(all(feature = "integration-test", target_os = "macos"))]
fn dispatch_fake_live_execution(
    mut reply: Value,
    request: &Value,
    control: &Arc<Mutex<ControlPlane>>,
    provider: Option<&fake_live_provider::FakeLiveProvider>,
    gateway: &mut Option<tradex::order_gateway::OrderGatewayHost>,
    sink: &EventSink,
) -> Value {
    if provider.is_none()
        || request.get("command").and_then(Value::as_str) != Some("trade.execution.prepare")
        || reply["ok"] != true
        || reply["data"]["attempt"]["state"] != "RESERVED"
    {
        return reply;
    }
    let Some(provider) = provider else {
        return reply;
    };
    let attempt_id = reply["data"]["attempt"]["attemptId"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    let approval_id = reply["data"]["attempt"]["approvalId"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    let workspace_id = reply["data"]["attempt"]["workspaceId"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    let startup = (|| -> Result<(), &'static str> {
        if gateway.is_none() {
            let executable = std::env::current_exe()
                .map_err(|_| "GATEWAY_BINARY_INVALID")?
                .with_file_name("tradex-order-gateway");
            let pin = std::env::var("TRADEX_ORDER_GATEWAY_SHA256")
                .map_err(|_| "GATEWAY_BINARY_INVALID")?;
            *gateway = Some(tradex::order_gateway::OrderGatewayHost::new(
                executable, pin,
            ));
        }
        let gateway = gateway.as_mut().ok_or("GATEWAY_UNAVAILABLE")?;
        if !gateway.is_running() {
            gateway.start()?;
        }
        Ok(())
    })();
    let dispatch = startup.and_then(|()| {
        gateway
            .as_mut()
            .ok_or("GATEWAY_UNAVAILABLE")?
            .dispatch_attempt(
                &attempt_id,
                {
                    let control = control.clone();
                    move |attempt_id, session_id| {
                        let mut control = control
                            .lock()
                            .map_err(|_| "IPC_CONTROL_PLANE_UNAVAILABLE".to_owned())?;
                        let mut package = control
                            .live_dispatch_package(attempt_id, session_id)
                            .map_err(|error| error.code)?;
                        let remote_account_id = package
                            .account
                            .data
                            .as_ref()
                            .map(|data| data.remote_account_id.clone())
                            .ok_or_else(|| "PROVIDER_REVIEW_REQUIRED".to_owned())?;
                        let bitget_order = if package.account.provider_id == "bitget"
                            && package.attempt.operation == tradex::protocol::FinancialOperation::Cancel
                        {
                            let broker_order_id = package
                                .attempt
                                .broker_order_id
                                .as_deref()
                                .and_then(|value| value.strip_prefix("normal:"))
                                .ok_or_else(|| "ORDER_CHANGED_REVIEW_AGAIN".to_owned())?;
                            let data = package.account.data.as_ref().unwrap();
                            let order = data
                                .bitget_order_book
                                .as_ref()
                                .and_then(|book| book.orders.iter().find(|order| {
                                    order.kind == "NORMAL" && order.provider_order_id == broker_order_id
                                }))
                                .ok_or_else(|| "ORDER_CHANGED_REVIEW_AGAIN".to_owned())?;
                            let open = data.open_orders.iter().find(|row| {
                                row.broker_order_id == package.attempt.broker_order_id.as_deref().unwrap_or_default()
                            });
                            let timestamp_ms = |value: Option<&str>| {
                                value.and_then(|value| {
                                    time::OffsetDateTime::parse(
                                        value,
                                        &time::format_description::well_known::Rfc3339,
                                    )
                                    .ok()
                                })
                                .map(|value| (value.unix_timestamp_nanos() / 1_000_000).to_string())
                            };
                            Some(json!({
                                "userId":remote_account_id,
                                "orderId":order.provider_order_id,
                                "symbol":order.symbol,
                                "size":order.quantity.as_deref().or(order.notional.as_deref()).unwrap_or("0"),
                                "orderType":if order.notional.is_some() { "market" } else { "limit" },
                                "side":order.side.to_ascii_lowercase(),
                                "status":order.provider_status,
                                "tpslType":"normal",
                                "priceAvg":open.and_then(|row| row.limit_price.as_deref()).unwrap_or("1"),
                                "baseVolume":order.filled_quantity.as_deref().unwrap_or("0"),
                                "quoteVolume":order.filled_value.as_deref().unwrap_or("0"),
                                "quoteCoin":order.currency,
                                "cTime":timestamp_ms(order.created_at.as_deref()),
                                "uTime":timestamp_ms(order.updated_at.as_deref())
                                    .unwrap_or_else(|| std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis().to_string())
                            }))
                        } else {
                            None
                        };
                        drop(control);
                        provider.set_remote_account_id(remote_account_id);
                        if let Some(order) = bitget_order {
                            provider.set_bitget_order(order);
                        }
                        package.local_test_base_url = Some(provider.base_url().to_owned());
                        Ok(package)
                    }
                },
                {
                    let control = control.clone();
                    move |grant_id, session_id| {
                        control
                            .lock()
                            .map_err(|_| "IPC_CONTROL_PLANE_UNAVAILABLE".to_owned())?
                            .begin_live_execution_submission(grant_id, session_id)
                            .map(|_| ())
                            .map_err(|error| error.code)
                    }
                },
                {
                    let control = control.clone();
                    move |attempt_id, code| {
                        if let Ok(mut control) = control.lock() {
                            let _ = control.stop_live_dispatch_before_submission(attempt_id, code);
                        }
                    }
                },
                {
                    let control = control.clone();
                    move |attempt_id, outcome| {
                        control
                            .lock()
                            .map_err(|_| "IPC_CONTROL_PLANE_UNAVAILABLE".to_owned())?
                            .complete_live_execution_submission(attempt_id, outcome)
                            .map(|_| ())
                            .map_err(|error| error.code)
                    }
                },
            )
    });
    if dispatch.is_err() {
        if let Some(gateway) = gateway.as_mut() {
            gateway.stop();
        }
        if let Ok(mut control) = control.lock() {
            let _ =
                control.stop_live_dispatch_before_submission(&attempt_id, "GATEWAY_PROCESS_FAILED");
        }
    }
    let saved = match control.lock() {
        Ok(mut control) => control.dispatch_with_events(
            json!({
                "requestId":request["requestId"],
                "schemaVersion":1,
                "command":"trade.execution.preparation.get",
                "payload":{"workspaceId":workspace_id,"approvalId":approval_id}
            }),
            "stdio",
            Some(sink.clone()),
        ),
        Err(_) => return failed_reply(&reply, "IPC_CONTROL_PLANE_UNAVAILABLE"),
    };
    if saved["ok"] != true || saved["data"]["preparation"].is_null() {
        return failed_reply(&reply, "WORKSPACE_INTEGRITY_FAILED");
    }
    reply["data"] = saved["data"]["preparation"].clone();
    reply
}

#[cfg(all(feature = "integration-test", target_os = "macos"))]
fn failed_reply(reply: &Value, code: &str) -> Value {
    json!({
        "requestId":reply["requestId"],
        "schemaVersion":1,
        "ok":false,
        "error":tradex::protocol::TradeXError::new(code)
    })
}

fn write_frame(output: &Mutex<io::Stdout>, frame: &Value) -> io::Result<()> {
    let mut output = output
        .lock()
        .map_err(|_| io::Error::other("output unavailable"))?;
    serde_json::to_writer(&mut *output, frame)?;
    output.write_all(b"\n")?;
    output.flush()
}
