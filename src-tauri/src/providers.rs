use crate::protocol::{Result, TradeXError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderField {
    pub id: String,
    pub label: String,
    pub input_type: String,
    pub required: bool,
    pub secret: bool,
    pub max_length: usize,
    pub help_text: String,
    pub environment: String,
}

#[derive(Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderDefinition {
    pub provider_id: String,
    pub display_name: String,
    pub environment: String,
    pub available: bool,
    pub help_text: String,
    pub fields: Vec<ProviderField>,
    pub required_permissions: Vec<String>,
    pub optional_permissions: Vec<String>,
    pub forbidden_permissions: Vec<String>,
}

#[derive(Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProviderCatalog {
    pub providers: Vec<ProviderDefinition>,
}

pub fn catalog() -> ProviderCatalog {
    ProviderCatalog {
        providers: [
            ("local-paper", "Local Paper", "LOCAL"),
            ("alpaca", "Alpaca Paper", "PAPER"),
            ("trading212", "Trading 212 Demo", "DEMO"),
            ("trading212", "Trading 212 Live", "LIVE"),
            ("binance", "Binance Spot Testnet", "TESTNET"),
            ("binance", "Binance Spot Live", "LIVE"),
            ("bitget", "Bitget Spot Demo", "DEMO"),
            ("bitget", "Bitget Spot Live", "LIVE"),
        ].into_iter().map(|(id, label, environment)| {
            let available = matches!(id, "alpaca" | "trading212" | "binance");
            ProviderDefinition {
                provider_id: id.into(), display_name: label.into(), environment: environment.into(), available,
                help_text: if id == "local-paper" { "Built-in; no external credentials. Simulation is not configured yet." }
                    else if id == "trading212" { "Use the API key and secret for this exact Demo or Live account. Invest/Stocks ISA only; the API does not expose the subtype. Account values use the primary currency; prices retain instrument currency. Scope and IP restrictions cannot be fully inspected. Connection testing only reads data and never arms Live execution." }
                    else if id == "binance" { "Use a separate HMAC API key and secret for this exact Spot Testnet or Live account. Native asset balances are not USD valuations. Live key scope is inspected separately; Testnet scope remains unverified. Withdrawals, transfers and unsupported margin/derivative permissions block confirmation. Testing only reads data; Live stays disarmed." }
                    else if available { "Use separate Alpaca Paper credentials. TradeX reads account, positions and open orders. Key scope cannot be fully inspected. No withdrawals, transfers, custody, margin borrowing or leverage management are required or implemented." }
                    else { "This provider connection is not available in this build." }.into(),
                fields: if available { [("apiKey", "Paper API key ID"), ("secret", "Paper API secret")].into_iter().map(|(id,label)| ProviderField {
                    id:id.into(), label:if id == "apiKey" && environment != "PAPER" { "API key".into() } else if environment != "PAPER" { "API secret".into() } else { label.into() }, input_type:"password".into(), required:true, secret:true, max_length:512,
                    help_text:format!("Enter the value for this {environment} account in the native secure window. Never reuse another environment’s credentials."), environment:environment.into(),
                }).collect() } else { vec![] },
                required_permissions: vec!["account.read".into(),"positions.read".into(),"orders.read".into()],
                optional_permissions: vec![],
                forbidden_permissions: vec!["withdrawal".into(),"transfer".into(),"custody".into(),"margin.borrow".into(),"leverage.manage".into()],
            }
        }).collect(),
    }
}

pub fn definition(provider: &str, environment: &str) -> Result<ProviderDefinition> {
    catalog()
        .providers
        .into_iter()
        .find(|p| p.provider_id == provider && p.environment == environment && p.available)
        .ok_or_else(|| TradeXError::new("PROVIDER_UNSUPPORTED"))
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderSelection {
    pub provider_id: String,
    pub environment: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkspaceQuery {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AccountQuery {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub connection_id: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AccountMutation {
    #[schemars(length(min = 1, max = 128))]
    pub workspace_id: String,
    #[schemars(length(min = 1, max = 128))]
    pub connection_id: String,
    #[schemars(length(min = 1, max = 256))]
    pub expected_state_version: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "step", rename_all = "camelCase", deny_unknown_fields)]
pub enum Connect {
    #[serde(rename_all = "camelCase")]
    Test {
        workspace_id: String,
        provider_id: String,
        environment: String,
        label: String,
    },
    #[serde(rename_all = "camelCase")]
    Confirm {
        workspace_id: String,
        connection_id: String,
        expected_state_version: String,
        acknowledge_unverified: bool,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConnectionState {
    Connecting,
    ReviewRequired,
    Connected,
    Failed,
    Disconnected,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PermissionReview {
    #[schemars(extend("enum" = ["VERIFIED", "UNVERIFIED"]))]
    pub scope: String,
    pub detected: Vec<String>,
    pub forbidden: Vec<String>,
    pub unsupported: Vec<String>,
    pub acknowledged: bool,
    pub ip_allow_list_status: String,
}

impl Default for PermissionReview {
    fn default() -> Self {
        Self {
            scope: "UNVERIFIED".into(),
            detected: vec![],
            forbidden: vec![],
            unsupported: vec![],
            acknowledged: false,
            ip_allow_list_status: "UNKNOWN".into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AccountHealth {
    pub connection: String,
    pub authentication: String,
    pub credential: String,
    pub private_stream: String,
    pub reconciliation: String,
    pub execution_eligibility: String,
    pub arming: String,
    pub reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Balance {
    pub asset: String,
    pub available: String,
    pub total: Option<String>,
    pub reserved: Option<String>,
    pub in_pies: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Position {
    pub symbol: String,
    pub quantity: String,
    pub market_value: Option<String>,
    pub average_entry_price: Option<String>,
    pub instrument_currency: Option<String>,
    pub market_value_currency: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OpenOrder {
    pub broker_order_id: String,
    pub symbol: String,
    pub side: String,
    pub quantity: Option<String>,
    pub notional: Option<String>,
    pub filled_quantity: Option<String>,
    pub filled_value: Option<String>,
    pub currency: Option<String>,
    pub status: String,
    pub limit_price: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AccountData {
    pub remote_account_id: String,
    pub account_type: String,
    pub currency: Option<String>,
    pub balances: Vec<Balance>,
    pub positions: Vec<Position>,
    pub open_orders: Vec<OpenOrder>,
    pub capabilities: Vec<String>,
    pub limitations: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AccountConnection {
    #[schemars(length(min = 1))]
    pub connection_id: String,
    #[schemars(length(min = 1))]
    pub workspace_id: String,
    pub provider_id: String,
    pub environment: String,
    pub label: String,
    pub created_at: String,
    pub updated_at: String,
    #[schemars(length(min = 1))]
    pub state_version: String,
    pub connection_state: ConnectionState,
    pub health: AccountHealth,
    pub permissions: PermissionReview,
    pub data: Option<AccountData>,
    pub last_successful_sync: Option<String>,
}

#[derive(Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Accounts {
    pub accounts: Vec<AccountConnection>,
}

impl AccountConnection {
    pub fn new(
        workspace_id: String,
        provider_id: String,
        environment: String,
        label: String,
    ) -> Result<Self> {
        let now = crate::storage::timestamp()?;
        Ok(Self {
            connection_id: uuid::Uuid::new_v4().to_string(),
            workspace_id,
            provider_id,
            environment: environment.clone(),
            label,
            created_at: now.clone(),
            updated_at: now,
            state_version: String::new(),
            connection_state: ConnectionState::Connecting,
            health: AccountHealth {
                connection: "CONNECTING".into(),
                authentication: "UNVERIFIED".into(),
                credential: "UNCHECKED".into(),
                private_stream: "NOT_CONFIGURED".into(),
                reconciliation: "NOT_RUN".into(),
                execution_eligibility: "BLOCKED".into(),
                arming: if environment == "LIVE" {
                    "DISARMED"
                } else {
                    "NOT_APPLICABLE"
                }
                .into(),
                reason: "Complete connection testing and permission review.".into(),
            },
            permissions: PermissionReview::default(),
            data: None,
            last_successful_sync: None,
        })
    }
    pub fn credential_ref(&self) -> String {
        format!(
            "{}/{}/{}/{}",
            self.workspace_id, self.provider_id, self.environment, self.connection_id
        )
    }
    pub fn blocked_permissions(&self) -> bool {
        !self.permissions.forbidden.is_empty() || !self.permissions.unsupported.is_empty()
    }
}
