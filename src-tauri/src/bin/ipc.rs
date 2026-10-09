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

#[cfg(feature = "integration-test")]
#[path = "../../tests/support/binance_market_stream_fixture.rs"]
mod binance_market_stream_fixture;
#[cfg(feature = "integration-test")]
#[path = "../../tests/support/quote_stream_fixture.rs"]
mod quote_stream_fixture;
#[cfg(feature = "integration-test")]
#[derive(Clone, Default)]
struct SharedVault(Arc<Mutex<fixtures::Vault>>);
#[cfg(feature = "integration-test")]
impl tradex::provider_io::CredentialVault for SharedVault {
    fn put(&self, r: &str, c: &tradex::provider_io::Credentials) -> tradex::protocol::Result<()> {
        self.0
            .lock()
            .map_err(|_| tradex::protocol::TradeXError::new("CREDENTIAL_UNAVAILABLE"))?
            .put(r, c)
    }
    fn get(&self, r: &str) -> tradex::protocol::Result<tradex::provider_io::Credentials> {
        self.0
            .lock()
            .map_err(|_| tradex::protocol::TradeXError::new("CREDENTIAL_UNAVAILABLE"))?
            .get(r)
    }
    fn remove(&self, r: &str) -> tradex::protocol::Result<()> {
        self.0
            .lock()
            .map_err(|_| tradex::protocol::TradeXError::new("CREDENTIAL_UNAVAILABLE"))?
            .remove(r)
    }
}
#[cfg(feature = "integration-test")]
#[derive(Default)]
struct QuoteHttp(Mutex<fixtures::Http>);
#[cfg(feature = "integration-test")]
impl tradex::provider_io::ProviderHttp for QuoteHttp {
    fn get(
        &self,
        e: tradex::provider_io::ProviderEndpoint,
        p: &str,
        h: reqwest::header::HeaderMap,
    ) -> tradex::protocol::Result<Vec<u8>> {
        self.0
            .lock()
            .map_err(|_| tradex::protocol::TradeXError::new("PROVIDER_UNAVAILABLE"))?
            .get(e, p, h)
    }
    fn request(
        &self,
        e: tradex::provider_io::ProviderEndpoint,
        m: tradex::provider_io::ProviderHttpMethod,
        p: &str,
        h: reqwest::header::HeaderMap,
        b: Option<&Value>,
    ) -> tradex::protocol::Result<tradex::provider_io::ProviderHttpResponse> {
        self.0
            .lock()
            .map_err(|_| tradex::protocol::TradeXError::new("PROVIDER_UNAVAILABLE"))?
            .request(e, m, p, h, b)
    }
}
fn main() -> io::Result<()> {
    #[cfg(feature = "integration-test")]
    let (vault, http) = {
        let mut http = fixtures::Http::default();
        http.fx_ui = std::env::var_os("TRADEX_FX_HTTP_FIXTURE").is_some();
        http.binance_rules_ui = std::env::var_os("TRADEX_BINANCE_RULE_HTTP_FIXTURE").is_some();
        (Arc::new(SharedVault::default()), http)
    };
    let Some(path) = std::env::args_os().nth(1) else {
        eprintln!("Usage: tradex-ipc <isolated-workspace-directory>");
        std::process::exit(2);
    };
    let control = Arc::new(Mutex::new(ControlPlane::new(PathBuf::from(path))));
    #[cfg(feature = "integration-test")]
    let quote_hot = tradex::quote_source::hot::QuoteHotSupervisor::new();
    #[cfg(feature = "integration-test")]
    let quote_http = Arc::new(QuoteHttp::default());
    #[cfg(feature = "integration-test")]
    let quote_connector = if std::env::var_os("TRADEX_QUOTE_STREAM_FIXTURE").is_some() {
        Some(quote_stream_fixture::start()?)
    } else {
        None
    };
    #[cfg(feature = "integration-test")]
    let binance_market_fixture =
        if std::env::var_os("TRADEX_BINANCE_MARKET_STREAM_FIXTURE").is_some() {
            Some(binance_market_stream_fixture::start()?)
        } else {
            None
        };
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
            if command == Some("binance.live.capacity.fixture") {
                // Only external HTTP responses change. No Control Plane capacity
                // or financial authority is populated by this disposable UI seam.
                let mode = match request["payload"]["scenario"].as_str() {
                    Some("NORMAL") => Some(fixtures::BinanceCapacityFixture::Normal),
                    Some("BASE_LISTS") => Some(fixtures::BinanceCapacityFixture::BaseLists),
                    Some("MALFORMED") => Some(fixtures::BinanceCapacityFixture::Malformed),
                    Some("DELAYED") => Some(fixtures::BinanceCapacityFixture::Delayed),
                    Some("COMPLETE_COVERAGE") => Some(fixtures::BinanceCapacityFixture::CompleteCoverage),
                    Some("ORDER_RATE_EXHAUSTED") => Some(fixtures::BinanceCapacityFixture::OrderRateExhausted),
                    _ => None,
                };
                let reply = if let Some(mode) = mode.filter(|_| http.binance_rules_ui) {
                    http.binance_capacity_ui.set(mode);
                    json!({"requestId":request["requestId"],"schemaVersion":1,"ok":true,"data":{}})
                } else {
                    json!({"requestId":request["requestId"],"schemaVersion":1,"ok":false,"error":tradex::protocol::TradeXError::new("IPC_PAYLOAD_INVALID")})
                };
                write_frame(&output, &json!({"kind":"result","result":reply}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            #[cfg(feature = "integration-test")]
            if command == Some("binance.live.rules.fixture") {
                let p = &request["payload"];
                let result = match (p["scenario"].as_str(), p["symbol"].as_str()) {
                    (
                        Some(
                            "NORMAL" | "HALT" | "BREAK" | "MALFORMED" | "PERCENT_REFERENCE"
                            | "PRICE_REJECT" | "PRICE_RANGE_PARTIAL" | "PRICE_RANGE_NULL",
                        ),
                        Some(symbol @ ("BTCUSDT" | "ETHUSDT")),
                    ) if http.binance_rules_ui => {
                        let now_ms = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map_or(0, |elapsed| elapsed.as_millis() as u64);
                        let mut external = fixtures::binance_spot_exchange_info(symbol);
                        *http.binance_execution_rules.borrow_mut() = None;
                        *http.binance_reference_price.borrow_mut() = None;
                        match p["scenario"].as_str().unwrap() {
                            "HALT" | "BREAK" => {
                                external["symbols"][0]["status"] = p["scenario"].clone()
                            }
                            "MALFORMED" => {
                                external["symbols"][0]["filters"][1]["minQty"] = json!("200")
                            }
                            "PERCENT_REFERENCE" => external["symbols"][0]["filters"].as_array_mut().unwrap().push(json!({"filterType":"PERCENT_PRICE","multiplierDown":"0.8","multiplierUp":"1.2","avgPriceMins":5})),
                            "PRICE_REJECT" => external["symbols"][0]["filters"][0]["tickSize"] = json!("7"),
                            // Documented partial PRICE_RANGE configuration: only one multiplier.
                            "PRICE_RANGE_PARTIAL" => *http.binance_execution_rules.borrow_mut() = Some(json!({"symbolRules":[{"symbol":symbol,"rules":[{"ruleType":"PRICE_RANGE","bidLimitMultUp":"1.0001"}]}]})),
                            // Documented explicit null reference price: not enforced, not a failure.
                            "PRICE_RANGE_NULL" => *http.binance_reference_price.borrow_mut() = Some(json!({"symbol":symbol,"referencePrice":null,"timestamp":now_ms})),
                            _ => (),
                        }
                        *http.binance_exchange_info.borrow_mut() = if p["scenario"] == "NORMAL" {
                            None
                        } else {
                            Some(external)
                        };
                        json!({"requestId":request["requestId"],"schemaVersion":1,"ok":true,"data":{}})
                    }
                    _ => {
                        json!({"requestId":request["requestId"],"schemaVersion":1,"ok":false,"error":{"code":"IPC_PAYLOAD_INVALID"}})
                    }
                };
                write_frame(&output, &json!({"kind":"result","result":result}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            #[cfg(feature = "integration-test")]
            if command == Some("binance.live.trading-status.fixture") {
                let scenario = request
                    .get("payload")
                    .and_then(|p| p.get("scenario"))
                    .and_then(Value::as_str);
                let reply = match scenario {
                    Some("NORMAL" | "MAINTENANCE" | "LOCKED" | "MALFORMED") => {
                        *http.binance_system_status.borrow_mut() = Some(
                            json!({"status": if scenario == Some("MAINTENANCE") {1} else {0}}),
                        );
                        *http.binance_api_trading_status.borrow_mut() = Some(
                            if scenario == Some("MALFORMED") {
                                json!({"data":{}})
                            } else {
                                json!({"data":{"isLocked": scenario == Some("LOCKED"),"plannedRecoverTime":0,"updateTime":1547630471725u64}})
                            },
                        );
                        json!({"requestId":request["requestId"],"schemaVersion":1,"ok":true,"data":{}})
                    }
                    _ => {
                        json!({"requestId":request["requestId"],"schemaVersion":1,"ok":false,"error":{"code":"IPC_PAYLOAD_INVALID"}})
                    }
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
            if command == Some("model.gateway.fixture") {
                let payload = request.get("payload").unwrap_or(&Value::Null);
                let seeded = (|| {
                    let workspace_id = payload
                        .get("workspaceId")
                        .and_then(Value::as_str)
                        .ok_or_else(|| tradex::protocol::TradeXError::new("IPC_PAYLOAD_INVALID"))?;
                    let status = serde_json::from_value(
                        payload.get("status").cloned().unwrap_or(Value::Null),
                    )
                    .map_err(|_| tradex::protocol::TradeXError::new("IPC_PAYLOAD_INVALID"))?;
                    control
                        .lock()
                        .map_err(|_| {
                            tradex::protocol::TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE")
                        })?
                        .seed_browser_model_gateway(workspace_id, status)
                })();
                let reply = match seeded {
                    Ok(()) => {
                        json!({"requestId":request["requestId"],"schemaVersion":1,"ok":true,"data":{"seeded":true}})
                    }
                    Err(error) => {
                        json!({"requestId":request["requestId"],"schemaVersion":1,"ok":false,"error":error})
                    }
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
                                vault
                                    .0
                                    .lock()
                                    .unwrap()
                                    .present
                                    .borrow_mut()
                                    .insert(account.credential_ref());
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
                                    vault
                                        .0
                                        .lock()
                                        .unwrap()
                                        .present
                                        .borrow_mut()
                                        .insert(account.credential_ref());
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
                                vault
                                    .0
                                    .lock()
                                    .unwrap()
                                    .present
                                    .borrow_mut()
                                    .insert(account.credential_ref());
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
            #[cfg(feature = "integration-test")]
            if command == Some("binance.market.fixture.gap") {
                let result = match (
                    binance_market_fixture.as_ref(),
                    request["payload"]["enabled"].as_bool(),
                ) {
                    (Some(fixture), Some(enabled)) => {
                        fixture.set_gap(enabled);
                        json!({"requestId":request["requestId"],"schemaVersion":1,"ok":true,"data":{}})
                    }
                    _ => {
                        json!({"requestId":request["requestId"],"schemaVersion":1,"ok":false,"error":tradex::protocol::TradeXError::new("IPC_ACCESS_DENIED")})
                    }
                };
                write_frame(&output, &json!({"kind":"result","result":result}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            #[cfg(feature = "integration-test")]
            if command == Some("binance.market.fixture.inspect") {
                let result = match binance_market_fixture.as_ref() {
                    Some(fixture) => {
                        json!({"requestId":request["requestId"],"schemaVersion":1,"ok":true,"data":fixture.inspect()})
                    }
                    None => {
                        json!({"requestId":request["requestId"],"schemaVersion":1,"ok":false,"error":tradex::protocol::TradeXError::new("IPC_ACCESS_DENIED")})
                    }
                };
                write_frame(&output, &json!({"kind":"result","result":result}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            #[cfg(feature = "integration-test")]
            if matches!(command, Some("market.hot.acquire" | "market.hot.release")) {
                let instrument = request["payload"]["instrumentId"].as_str();
                let binance = instrument.is_some_and(|id| id.starts_with("crypto:"));
                let result = if binance && binance_market_fixture.is_some() {
                    let fixture = binance_market_fixture.as_ref().unwrap();
                    match fixture.connector(instrument.unwrap()) {
                        Ok(connector) => quote_hot.dispatch_with(
                            &control,
                            &request,
                            "stdio",
                            Arc::new(binance_market_stream_fixture::NoTradingKey),
                            fixture.http.clone(),
                            tradex::quote_source::hot::QuoteStreamConnectors {
                                stock: Default::default(),
                                binance: connector,
                            },
                        ),
                        Err(error) => {
                            json!({"requestId":request["requestId"],"schemaVersion":1,"ok":false,"error":error})
                        }
                    }
                } else if command == Some("market.hot.release") && binance_market_fixture.is_some()
                {
                    // Release carries lease identity only; the manager owns its transport.
                    quote_hot.dispatch_with(
                        &control,
                        &request,
                        "stdio",
                        Arc::new(binance_market_stream_fixture::NoTradingKey),
                        binance_market_fixture.as_ref().unwrap().http.clone(),
                        tradex::quote_source::hot::QuoteStreamConnectors::default(),
                    )
                } else if !binance && let Some(connector) = quote_connector.as_ref() {
                    quote_hot.dispatch_with(
                        &control,
                        &request,
                        "stdio",
                        vault.clone(),
                        quote_http.clone(),
                        connector.clone(),
                    )
                } else {
                    json!({"requestId":request["requestId"],"schemaVersion":1,"ok":false,"error":tradex::protocol::TradeXError::new("IPC_ACCESS_DENIED")})
                };
                write_frame(&output, &json!({"kind":"result","result":result}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            #[cfg(feature = "integration-test")]
            if command == Some("market.get") {
                let result = tradex::quote_source::execute_market(
                    &control,
                    &request,
                    "stdio",
                    vault.as_ref(),
                    &http,
                );
                write_frame(&output, &json!({"kind":"result","result":result}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            #[cfg(feature = "integration-test")]
            if matches!(
                command,
                Some(
                    "data.actions.refresh"
                        | "data.instrument.refresh"
                        | "data.fx.refresh"
                        | "data.binance_rules.refresh"
                        | "trade.spot_rules.refresh"
                        | "trade.spot_capacity.refresh"
                        | "trade.spot_order_intervals.refresh"
                )
            ) {
                let result = tradex::financial_sources::execute_refresh(
                    &control,
                    &request,
                    "stdio",
                    vault.as_ref(),
                    &http,
                );
                write_frame(&output, &json!({"kind":"result","result":result}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            #[cfg(feature = "integration-test")]
            if command == Some("data.calendar.refresh") {
                let result = tradex::calendar_source::execute_refresh(
                    &control,
                    &request,
                    "stdio",
                    vault.as_ref(),
                    &http,
                );
                write_frame(&output, &json!({"kind":"result","result":result}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            if command == Some("data.source.probe") {
                #[cfg(feature = "integration-test")]
                let result = tradex::quote_source::execute_probe(
                    &control,
                    &request,
                    "stdio",
                    vault.as_ref(),
                    &http,
                );
                #[cfg(not(feature = "integration-test"))]
                let result = {
                    let prepared = match control.lock() {
                        Ok(mut control) => control.prepare_data_source_probe(&request),
                        Err(_) => Err(tradex::protocol::TradeXError::new(
                            "IPC_CONTROL_PLANE_UNAVAILABLE",
                        )),
                    };
                    let result = match prepared {
                        Ok(Some(job)) => {
                            let outcome = tradex::data_sources::probe(
                                &job.input.source_id,
                                job.source.clone(),
                            );
                            match control.lock() {
                                Ok(mut control) => {
                                    control.complete_data_source_probe(&job, outcome)
                                }
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
                    result
                };
                write_frame(&output, &json!({"kind":"result", "result":result}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            #[cfg(feature = "integration-test")]
            if matches!(
                command,
                Some("data.source.configure" | "data.source.disconnect" | "data.source.cleanup")
            ) {
                let result = tradex::quote_source::execute_configuration(
                    &control,
                    &request,
                    "stdio",
                    vault.as_ref(),
                    fixtures::credentials,
                );
                write_frame(&output, &json!({"kind":"result", "result":result}))?;
                frame.clear();
                oversized = false;
                continue;
            }
            #[cfg(feature = "integration-test")]
            let prepared = match control.lock() {
                Ok(mut control) => control.prepare_provider_for(&request, "stdio"),
                Err(_) => Err(tradex::protocol::TradeXError::new(
                    "IPC_CONTROL_PLANE_UNAVAILABLE",
                )),
            };
            #[cfg(feature = "integration-test")]
            let result = match prepared {
                Ok(Some(job)) => {
                    let outcome = job.run(
                        vault.as_ref(),
                        |definition| {
                            if definition.provider_id == "bitget" {
                                fixtures::bitget::credentials()
                            } else {
                                fixtures::credentials()
                            }
                        },
                        &http,
                        || {
                            control
                                .lock()
                                .is_ok_and(|control| control.provider_job_current(&job))
                        },
                    );
                    match control.lock() {
                        Ok(mut control) => {
                            let reply = control.complete_provider(&job, outcome);
                            if let Some(cleanup) =
                                job.cleanup_after_failed_commit(&reply, vault.as_ref())
                            {
                                control.record_credential_cleanup(&job, cleanup);
                            }
                            reply
                        }
                        Err(_) => json!({
                            "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                            "error":tradex::protocol::TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE")
                        }),
                    }
                }
                Ok(None) => match control.lock() {
                    Ok(mut control) => {
                        control.dispatch_with_events(request.clone(), "stdio", Some(sink.clone()))
                    }
                    Err(_) => json!({
                        "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                        "error":tradex::protocol::TradeXError::new("IPC_CONTROL_PLANE_UNAVAILABLE")
                    }),
                },
                Err(error) => json!({
                    "requestId":request["requestId"],"schemaVersion":1,"ok":false,
                    "error":error
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
    #[cfg(feature = "integration-test")]
    quote_hot.stop_all();
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
