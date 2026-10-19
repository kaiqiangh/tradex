//! Public Spot source selection; configuration performs no provider I/O.
mod book;
pub mod stream;
use crate::protocol::{
    BinanceLocalRetentionBoundary, BinanceLocalUseAssessment, BinanceLocalUseAssessmentState,
    BinanceLocalUseAttestation, BinanceLocalUseAttestationCommand, BinanceLocalUseDatasetProfile,
    BinanceLocalUseDestination, BinanceLocalUseEnvironment, BinanceLocalUseFeeSymbolScope,
    BinanceLocalUseProvenance, BinanceLocalUseScope, BinanceLocalUseWithdrawalCommand,
    BinanceMarketSourceConnection, CommandEnvelope, DataSourceEntry, DataSourceMutation,
    DataSourceProbeKind, DataSourceQuery, DataSourceStatus, MAX_SEQUENCE, Result, TradeXError,
};
use crate::{ControlPlane, payload, provider_order_consumer_allowed};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub const SOURCE_ID: &str = "BINANCE_SPOT_PUBLIC";
const LOCAL_USE_POLICY_VERSION: &str = "BINANCE_ORDINARY_SPOT_LOCAL_DECISION_SUPPORT_V1";
const BINANCE_TERMS_URL: &str = "https://www.binance.com/en/terms";
const BINANCE_TERMS_REVIEWED_AT: &str = "2026-10-08";
const KNOWN_RESTRICTED_JURISDICTIONS: &[&str] = &["US"];
const ISO_3166_1_ALPHA_2_CODES: &str = "AW AF AO AI AX AL AD AE AR AM AS AQ TF AG AU AT AZ BI BE BJ BQ BF BD BG BH BS BA BL BY BZ BM BO BR BB BN BT BV BW CF CA CC CH CL CN CI CM CD CG CK CO KM CV CR CU CW CX KY CY CZ DE DJ DM DO DZ EC EG ER EH ES EE ET FI FJ FK FR FO FM GA GB GE GG GI GN GP GM GW GQ GR GD GL GT GF GU GY HK HM HN HR HT HU ID IM IN IO IE IR IQ IS IL IT JM JE JO JP KZ KE KG KH KI KN KR KW LA LB LR LY LC LI LK LS LT LU LV MO MF MA MC MD MG MV MX MH MK ML MT MM ME MN MP MZ MR MS MQ MU MW MY YT NA NC NE NF NG NI NU NL NO NP NR NZ OM PK PA PN PE PH PW PL PR KP PT PY PS PF QA RE RO RU RW SA SD SN SG GS SH SJ SB SL SV SM SO PM RS SS ST SR SK SI SZ SX SC SY TC TD TG TH TJ TK TM TL TO TT TN TR TV TW TZ UG UA UM UY US UZ VA VC VE VG VI VN VU WF WS YE ZA ZM ZW";

/// Only configuration persists. No credential, quote, book or lease belongs here.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SavedMarketSource {
    pub generation: u64,
    pub configured: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_use_attestation: Option<BinanceLocalUseAttestation>,
}
impl SavedMarketSource {
    pub fn state_version(&self, workspace: &str) -> String {
        format!("binance-market:{workspace}:{}", self.generation)
    }
    pub fn validate(&self) -> Result<()> {
        if self.generation > MAX_SEQUENCE {
            return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
        }
        if let Some(attestation) = &self.local_use_attestation {
            validate_local_use_attestation(attestation)?;
        }
        Ok(())
    }
}

pub(crate) fn policy() -> DataSourceEntry {
    DataSourceEntry {
        source_id: SOURCE_ID.into(),
        provider: "Binance ordinary public Spot market data".into(),
        capabilities: vec!["BTC/USDT and ETH/USDT Hot quotes and bounded depth".into()],
        coverage: "Only canonical BTC/USDT and ETH/USDT on BINANCE; BASE depth, with explicit known price bands. USDT is not USD.".into(),
        latency: "Realtime diff-depth events; usable continuity and actual timestamps must be proven.".into(),
        entitlement: "Public endpoints require no trading credentials. Access alone does not prove permitted financial use.".into(),
        retention: "Ephemeral in-memory book only; permitted durable retention remains UNVERIFIED. No raw tick/book export is granted.".into(),
        redistribution: "UNVERIFIED; no redistribution permission is inferred from public access.".into(),
        commercial_use: "UNVERIFIED; no user-specific commercial or financial-use determination.".into(),
        jurisdictions: "UNVERIFIED for this user; current agreement, local terms and regional access restrictions apply. No host fallback or bypass.".into(),
        official_url: "https://developers.binance.com/en/docs/catalog/core-trading-spot-trading/api/ws-streams/~".into(),
        terms_url: BINANCE_TERMS_URL.into(),
        reviewed_at: BINANCE_TERMS_REVIEWED_AT.into(),
        checked_at: None,
        observed_at: None,
        verified_at: None,
        probe_kind: DataSourceProbeKind::PublicMetadata,
        status: DataSourceStatus::BlockedExternal,
        configured: false,
        availability_reason: "Select the ordinary public Binance Spot source. Saving does not start a network read or lease.".into(),
    }
}

pub(crate) fn connection(
    control: &ControlPlane,
    workspace: &str,
) -> Result<BinanceMarketSourceConnection> {
    control.require_workspace(workspace)?;
    let saved = control.store.as_ref().unwrap().binance_market_source()?;
    let version = saved.state_version(workspace);
    let mut source = policy();
    source.configured = saved.configured;
    if saved.configured {
        source.status = DataSourceStatus::Unverified;
        source.availability_reason = "Public Spot source selected; no continuous Hot book is verified. Open a supported Hot view to start a lease.".into();
        stream::project_source(control, &version, &mut source);
    }
    Ok(BinanceMarketSourceConnection {
        workspace_id: workspace.into(),
        state_version: version,
        configured: saved.configured,
        source,
        local_use_attestation: current_saved_attestation(&saved, workspace),
        local_use_assessment: local_use_assessment(&saved, workspace),
    })
}

pub(crate) fn is_known_restricted_jurisdiction(jurisdiction: &str) -> bool {
    KNOWN_RESTRICTED_JURISDICTIONS.contains(&jurisdiction)
}

fn is_supported_jurisdiction_code(jurisdiction: &str) -> bool {
    ISO_3166_1_ALPHA_2_CODES
        .split_ascii_whitespace()
        .any(|code| code == jurisdiction)
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct LocalUseRiskEvidence {
    pub source_state_version: String,
    pub configured: bool,
    pub current_attestation: Option<BinanceLocalUseAttestation>,
    pub restricted_jurisdiction: Option<String>,
    pub unsupported_jurisdiction: Option<String>,
    pub current_policy_version: String,
    pub current_terms_url: String,
    pub current_terms_reviewed_at: String,
}

pub(crate) fn local_use_risk_evidence(
    control: &ControlPlane,
    workspace: &str,
) -> Result<LocalUseRiskEvidence> {
    control.require_workspace(workspace)?;
    let saved = control
        .store
        .as_ref()
        .ok_or_else(|| TradeXError::new("IPC_AGGREGATE_NOT_FOUND"))?
        .binance_market_source()?;
    let source = policy();
    let restricted_jurisdiction = saved
        .local_use_attestation
        .as_ref()
        .filter(|attestation| {
            attestation.workspace_id == workspace
                && attestation.source_id == SOURCE_ID
                && is_known_restricted_jurisdiction(&attestation.declared_jurisdiction)
        })
        .map(|attestation| attestation.declared_jurisdiction.clone());
    Ok(LocalUseRiskEvidence {
        source_state_version: saved.state_version(workspace),
        configured: saved.configured,
        current_attestation: current_saved_attestation(&saved, workspace),
        restricted_jurisdiction,
        unsupported_jurisdiction: saved
            .local_use_attestation
            .as_ref()
            .filter(|attestation| {
                !is_supported_jurisdiction_code(&attestation.declared_jurisdiction)
            })
            .map(|attestation| attestation.declared_jurisdiction.clone()),
        current_policy_version: LOCAL_USE_POLICY_VERSION.into(),
        current_terms_url: source.terms_url,
        current_terms_reviewed_at: source.reviewed_at,
    })
}

fn current_saved_attestation(
    saved: &SavedMarketSource,
    workspace: &str,
) -> Option<BinanceLocalUseAttestation> {
    let attestation = saved.local_use_attestation.as_ref()?;
    if local_use_assessment(saved, workspace).state != BinanceLocalUseAssessmentState::Current {
        return None;
    }
    Some(attestation.clone())
}

fn local_use_assessment(saved: &SavedMarketSource, workspace: &str) -> BinanceLocalUseAssessment {
    let assessment = |state, reason: &str| BinanceLocalUseAssessment {
        state,
        reason: reason.into(),
    };
    if !saved.configured {
        return assessment(
            BinanceLocalUseAssessmentState::SourceNotSelected,
            "The ordinary Binance public Spot source is not selected. Select it before recording a local-use confirmation.",
        );
    }
    let Some(attestation) = saved.local_use_attestation.as_ref() else {
        return assessment(
            BinanceLocalUseAssessmentState::NotRecorded,
            "Public Spot technical collection establishes no user-specific financial-use, retention, redistribution, commercial or regional permission. No current matching local-use confirmation is recorded; provider entitlement remains UNVERIFIED.",
        );
    };
    if attestation.workspace_id != workspace {
        return assessment(
            BinanceLocalUseAssessmentState::Mismatched,
            "The saved local-use confirmation is bound to a different workspace. Record a new confirmation for this workspace.",
        );
    }
    if attestation.source_generation != saved.generation {
        return assessment(
            BinanceLocalUseAssessmentState::Stale,
            "The saved local-use confirmation is stale because the Binance source configuration generation changed. Review the current terms and record a new confirmation.",
        );
    }
    if !is_supported_jurisdiction_code(&attestation.declared_jurisdiction) {
        return assessment(
            BinanceLocalUseAssessmentState::Unsupported,
            "The saved local-use confirmation has an unrecognized jurisdiction code. Use an ISO 3166-1 alpha-2 country code and record a new confirmation.",
        );
    }
    if is_known_restricted_jurisdiction(&attestation.declared_jurisdiction) {
        return assessment(
            BinanceLocalUseAssessmentState::Restricted,
            &format!(
                "Current local policy identifies declared jurisdiction {} as restricted for this ordinary Binance Spot source. No provider entitlement is inferred.",
                attestation.declared_jurisdiction
            ),
        );
    }
    if attestation.dataset_profile
        != BinanceLocalUseDatasetProfile::OrdinaryPublicSpotQuotesAndBoundedDepth
        || attestation.symbols != ["BTCUSDT", "ETHUSDT"]
        || attestation.fee_symbol_scope
            != BinanceLocalUseFeeSymbolScope::BnbUsdtOnlyWhenRequiredForFeeValuation
    {
        return assessment(
            BinanceLocalUseAssessmentState::Unsupported,
            "The saved local-use confirmation uses a dataset or instrument profile outside the supported ordinary Binance Spot scope of BTC/USDT and ETH/USDT quotes and bounded depth.",
        );
    }
    if attestation.policy_version != LOCAL_USE_POLICY_VERSION
        || attestation.terms_url != BINANCE_TERMS_URL
        || attestation.terms_reviewed_at != BINANCE_TERMS_REVIEWED_AT
    {
        return assessment(
            BinanceLocalUseAssessmentState::PolicyUnresolved,
            "The saved local-use confirmation references an older or unresolved policy or terms revision. Review the current displayed terms before recording a new confirmation.",
        );
    }
    if attestation.provenance != BinanceLocalUseProvenance::UserAttestedLocalOnly
        || attestation.scope != BinanceLocalUseScope::LocalTradingDecisionSupport
        || attestation.source_id != SOURCE_ID
        || attestation.provider_id != "binance"
        || attestation.environment != BinanceLocalUseEnvironment::Ordinary
        || attestation.destination != BinanceLocalUseDestination::LocalOnly
        || attestation.retention_boundary
            != BinanceLocalRetentionBoundary::EphemeralLocalEvidenceNoQuoteOrBookExport
        || local_use_binding_digest(
            workspace,
            saved.generation,
            &attestation.declared_jurisdiction,
            &attestation.terms_url,
            &attestation.terms_reviewed_at,
            &attestation.policy_version,
        )
        .ok()
        .as_deref()
            != Some(attestation.binding_digest.as_str())
    {
        return assessment(
            BinanceLocalUseAssessmentState::Mismatched,
            "The saved local-use confirmation does not match the current source binding or its integrity digest. Record a new confirmation.",
        );
    }
    assessment(
        BinanceLocalUseAssessmentState::Current,
        &format!(
            "User-attested local-only decision support is bound to declared jurisdiction {} and the current Binance Spot source profile; provider entitlement remains UNVERIFIED.",
            attestation.declared_jurisdiction
        ),
    )
}

fn validate_local_use_attestation(attestation: &BinanceLocalUseAttestation) -> Result<()> {
    if attestation.workspace_id.is_empty()
        || attestation.workspace_id.len() > 128
        || attestation.source_generation == 0
        || attestation.source_id != SOURCE_ID
        || attestation.provider_id != "binance"
        || attestation.declared_jurisdiction.len() != 2
        || !attestation
            .declared_jurisdiction
            .bytes()
            .all(|byte| byte.is_ascii_uppercase())
        || attestation.policy_version.is_empty()
        || attestation.policy_version.len() > 64
        || attestation.terms_url.is_empty()
        || attestation.terms_url.len() > 512
        || attestation.terms_reviewed_at.len() < 10
        || attestation.terms_reviewed_at.len() > 32
        || attestation.confirmed_at.is_empty()
        || attestation.confirmed_at.len() > 64
        || attestation.binding_digest.len() != 64
        || !attestation
            .binding_digest
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(TradeXError::new("WORKSPACE_INTEGRITY_FAILED"));
    }
    Ok(())
}

fn local_use_binding_digest(
    workspace: &str,
    source_generation: u64,
    jurisdiction: &str,
    terms_url: &str,
    terms_reviewed_at: &str,
    policy_version: &str,
) -> Result<String> {
    let material = json!([
        workspace,
        source_generation,
        SOURCE_ID,
        "binance",
        "ORDINARY",
        BinanceLocalUseDatasetProfile::OrdinaryPublicSpotQuotesAndBoundedDepth,
        ["BTCUSDT", "ETHUSDT"],
        BinanceLocalUseFeeSymbolScope::BnbUsdtOnlyWhenRequiredForFeeValuation,
        jurisdiction,
        BinanceLocalUseScope::LocalTradingDecisionSupport,
        BinanceLocalUseDestination::LocalOnly,
        BinanceLocalRetentionBoundary::EphemeralLocalEvidenceNoQuoteOrBookExport,
        policy_version,
        terms_url,
        terms_reviewed_at,
    ]);
    let encoded = serde_json::to_vec(&material)
        .map_err(|_| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
    Ok(hex::encode(Sha256::digest(encoded)))
}

fn make_local_use_attestation(
    workspace: &str,
    next_generation: u64,
    jurisdiction: &str,
    confirmed_at: String,
) -> Result<BinanceLocalUseAttestation> {
    let source = policy();
    let binding_digest = local_use_binding_digest(
        workspace,
        next_generation,
        jurisdiction,
        &source.terms_url,
        &source.reviewed_at,
        LOCAL_USE_POLICY_VERSION,
    )?;
    Ok(BinanceLocalUseAttestation {
        workspace_id: workspace.into(),
        source_generation: next_generation,
        provenance: BinanceLocalUseProvenance::UserAttestedLocalOnly,
        scope: BinanceLocalUseScope::LocalTradingDecisionSupport,
        source_id: SOURCE_ID.into(),
        provider_id: "binance".into(),
        environment: BinanceLocalUseEnvironment::Ordinary,
        dataset_profile: BinanceLocalUseDatasetProfile::OrdinaryPublicSpotQuotesAndBoundedDepth,
        symbols: vec!["BTCUSDT".into(), "ETHUSDT".into()],
        fee_symbol_scope: BinanceLocalUseFeeSymbolScope::BnbUsdtOnlyWhenRequiredForFeeValuation,
        declared_jurisdiction: jurisdiction.into(),
        destination: BinanceLocalUseDestination::LocalOnly,
        retention_boundary:
            BinanceLocalRetentionBoundary::EphemeralLocalEvidenceNoQuoteOrBookExport,
        policy_version: LOCAL_USE_POLICY_VERSION.into(),
        terms_url: source.terms_url,
        terms_reviewed_at: source.reviewed_at,
        confirmed_at,
        binding_digest,
    })
}

pub(crate) fn selected_once(control: &ControlPlane) -> Result<bool> {
    Ok(control
        .store
        .as_ref()
        .ok_or_else(|| TradeXError::new("IPC_AGGREGATE_NOT_FOUND"))?
        .binance_market_source()?
        .generation
        > 0)
}

pub(crate) fn metadata(
    control: &mut ControlPlane,
    request: CommandEnvelope,
    consumer: &str,
) -> Result<(Value, Option<String>)> {
    if !provider_order_consumer_allowed(consumer) {
        return Err(TradeXError::new("IPC_ACCESS_DENIED"));
    }
    let workspace = if request.command == "data.binance_market.connection" {
        let input: DataSourceQuery = payload(request.payload)?;
        input.workspace_id
    } else if request.command == "data.binance_market.attest_local_use" {
        let input: BinanceLocalUseAttestationCommand = payload(request.payload)?;
        if !input.confirm_local_trading_decision_support
            || input.declared_jurisdiction.len() != 2
            || !input
                .declared_jurisdiction
                .bytes()
                .all(|byte| byte.is_ascii_uppercase())
        {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        control.require_workspace(&input.workspace_id)?;
        let store = control.store.as_mut().unwrap();
        let mut saved = store.binance_market_source()?;
        if saved.state_version(&input.workspace_id) != input.expected_state_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        if !saved.configured {
            return Err(TradeXError::new("BINANCE_LOCAL_USE_SOURCE_UNAVAILABLE"));
        }
        if !is_supported_jurisdiction_code(&input.declared_jurisdiction) {
            return Err(TradeXError::new(
                "BINANCE_LOCAL_USE_JURISDICTION_UNSUPPORTED",
            ));
        }
        if is_known_restricted_jurisdiction(&input.declared_jurisdiction) {
            return Err(TradeXError::new("BINANCE_LOCAL_USE_RESTRICTED"));
        }
        let source = policy();
        if source.terms_url != BINANCE_TERMS_URL
            || source.reviewed_at != BINANCE_TERMS_REVIEWED_AT
            || LOCAL_USE_POLICY_VERSION.is_empty()
        {
            return Err(TradeXError::new("BINANCE_LOCAL_USE_POLICY_UNRESOLVED"));
        }
        let next_generation = saved
            .generation
            .checked_add(1)
            .filter(|generation| *generation <= MAX_SEQUENCE)
            .ok_or_else(|| TradeXError::new("WORKSPACE_INTEGRITY_FAILED"))?;
        saved.local_use_attestation = Some(make_local_use_attestation(
            &input.workspace_id,
            next_generation,
            &input.declared_jurisdiction,
            crate::storage::timestamp()?,
        )?);
        store.save_binance_market_source(saved)?;
        input.workspace_id
    } else if request.command == "data.binance_market.withdraw_local_use" {
        let input: BinanceLocalUseWithdrawalCommand = payload(request.payload)?;
        control.require_workspace(&input.workspace_id)?;
        let store = control.store.as_mut().unwrap();
        let mut saved = store.binance_market_source()?;
        if saved.state_version(&input.workspace_id) != input.expected_state_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        saved.local_use_attestation = None;
        store.save_binance_market_source(saved)?;
        input.workspace_id
    } else {
        let input: DataSourceMutation = payload(request.payload)?;
        control.require_workspace(&input.workspace_id)?;
        let store = control.store.as_mut().unwrap();
        let mut saved = store.binance_market_source()?;
        if saved.state_version(&input.workspace_id) != input.expected_state_version {
            return Err(TradeXError::new("STATE_VERSION_CONFLICT"));
        }
        saved.configured = request.command == "data.binance_market.configure";
        saved.local_use_attestation = None;
        store.save_binance_market_source(saved)?;
        input.workspace_id
    };
    let projected = connection(control, &workspace)?;
    let version = projected.state_version.clone();
    Ok((json!(projected), Some(version)))
}

#[cfg(test)]
mod local_use_tests {
    use super::*;

    fn current_saved_source() -> SavedMarketSource {
        SavedMarketSource {
            generation: 2,
            configured: true,
            local_use_attestation: Some(
                make_local_use_attestation("workspace", 2, "CA", "2026-10-19T00:00:00Z".into())
                    .unwrap(),
            ),
        }
    }

    #[test]
    fn old_saved_source_projection_decodes_without_the_optional_attestation() {
        let saved: SavedMarketSource =
            serde_json::from_str(r#"{"generation":1,"configured":true}"#).unwrap();
        assert_eq!(saved.generation, 1);
        assert!(saved.configured);
        assert!(saved.local_use_attestation.is_none());
        saved.validate().unwrap();
    }

    #[test]
    fn stale_unknown_and_mismatched_bindings_have_distinct_backend_assessments() {
        let current = current_saved_source();
        assert_eq!(
            local_use_assessment(&current, "workspace").state,
            BinanceLocalUseAssessmentState::Current
        );

        let mut stale = current.clone();
        stale.generation += 1;
        assert_eq!(
            local_use_assessment(&stale, "workspace").state,
            BinanceLocalUseAssessmentState::Stale
        );

        let mut unsupported = current.clone();
        unsupported
            .local_use_attestation
            .as_mut()
            .unwrap()
            .declared_jurisdiction = "ZZ".into();
        assert_eq!(
            local_use_assessment(&unsupported, "workspace").state,
            BinanceLocalUseAssessmentState::Unsupported
        );

        let mut unresolved = current.clone();
        unresolved
            .local_use_attestation
            .as_mut()
            .unwrap()
            .terms_reviewed_at = "2020-01-01".into();
        assert_eq!(
            local_use_assessment(&unresolved, "workspace").state,
            BinanceLocalUseAssessmentState::PolicyUnresolved
        );

        let mut mismatched = current;
        mismatched
            .local_use_attestation
            .as_mut()
            .unwrap()
            .binding_digest = "0".repeat(64);
        assert_eq!(
            local_use_assessment(&mismatched, "workspace").state,
            BinanceLocalUseAssessmentState::Mismatched
        );
    }
}
