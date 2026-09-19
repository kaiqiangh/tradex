use crate::protocol::{
    DataSourceEntry, DataSourceStatus, FilterSpec, RankSpec, ResearchFreshness, ResearchQuality,
    Result, ScreenerCandidate, ScreenerDirection, ScreenerFeature, ScreenerFeatureField,
    ScreenerOperation, ScreenerOperator, ScreenerPredicate, ScreenerPredicateField,
    ScreenerProvenance, ScreenerRankField, ScreenerRequest, ScreenerResult, ScreenerResultState,
    ScreenerUniverse, TradeXError,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;

const FIXTURE_SOURCE: &str = "FX-SCREENER";
const FIXTURE_TIMESTAMP: &str = "2026-09-14T00:00:00Z";
const FIXTURE_LIMITATION: &str =
    "Synthetic screener fixture; provider entitlement and execution authority are not established.";

pub fn screen(
    request: &ScreenerRequest,
    sources: &[DataSourceEntry],
    received_timestamp: &str,
    fixture: bool,
) -> Result<ScreenerResult> {
    validate_request(request)?;
    let parsed = parse(request, received_timestamp)?;
    if request.operation == ScreenerOperation::Parse {
        return Ok(parsed);
    }
    if parsed.state != ScreenerResultState::Parsed {
        return Ok(parsed);
    }
    let Some(filter_spec) = request.filter_spec.as_ref() else {
        return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
    };
    let Some(rank_spec) = request.rank_spec.as_ref() else {
        return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
    };
    let expected_revision = parsed.revision.clone().expect("parsed revision");
    if request.revision.as_deref() != Some(expected_revision.as_str()) {
        return Err(TradeXError::new("SCREENER_REVISION_STALE"));
    }
    if parsed.filter_spec.as_ref() != Some(filter_spec)
        || parsed.rank_spec.as_ref() != Some(rank_spec)
    {
        return Err(TradeXError::new("SCREENER_REVISION_STALE"));
    }
    let limit = request
        .limit
        .or_else(|| parsed_limit(&parsed))
        .unwrap_or(10);
    if limit == 0 || limit > 50 {
        return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
    }
    if !fixture {
        return Ok(blocked_result(
            request,
            filter_spec,
            rank_spec,
            expected_revision,
            sources,
            received_timestamp,
        ));
    }
    let mut candidates = fixture_rows(filter_spec.universe)
        .into_iter()
        .filter(|row| matches_predicates(row, &filter_spec.predicates))
        .map(|row| candidate(row, rank_spec))
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| compare_candidates(left, right, rank_spec));
    for (index, candidate) in candidates.iter_mut().enumerate() {
        candidate.rank = (index + 1) as u32;
    }
    candidates.truncate(limit as usize);
    let state = if candidates.is_empty() {
        ScreenerResultState::Empty
    } else {
        ScreenerResultState::Completed
    };
    let candidate_count = candidates.len() as u32;
    Ok(ScreenerResult {
        workspace_id: request.workspace_id.clone(),
        operation: request.operation,
        state,
        natural_language: request.natural_language.clone(),
        focus: request.focus.clone(),
        filter_spec: Some(filter_spec.clone()),
        rank_spec: Some(rank_spec.clone()),
        revision: Some(expected_revision),
        applied_conditions: applied_conditions(filter_spec, rank_spec, limit),
        candidates,
        candidate_count,
        source_ids: vec![FIXTURE_SOURCE.into()],
        provider_timestamp: Some(FIXTURE_TIMESTAMP.into()),
        received_timestamp: FIXTURE_TIMESTAMP.into(),
        availability_reason: "Synthetic fixture rows are available for contract verification only."
            .into(),
        limitations: vec![FIXTURE_LIMITATION.into()],
        fixture_label: Some("SYNTHETIC_SCREENER_FIXTURE".into()),
    })
}

fn validate_request(request: &ScreenerRequest) -> Result<()> {
    if request.workspace_id.trim().is_empty()
        || request.workspace_id.chars().count() > 128
        || request.workspace_id.chars().any(char::is_control)
        || request.natural_language.trim().is_empty()
        || request.natural_language.chars().count() > 4_000
        || request.natural_language.chars().any(char::is_control)
        || request
            .limit
            .is_some_and(|limit| !(1..=50).contains(&limit))
        || request.revision.as_deref().is_some_and(|value| {
            value.is_empty() || value.len() > 80 || value.chars().any(char::is_control)
        })
    {
        return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
    }
    if let Some(spec) = &request.filter_spec {
        validate_filter(spec)?;
    }
    Ok(())
}

fn validate_filter(spec: &FilterSpec) -> Result<()> {
    if spec.predicates.len() > 8 {
        return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
    }
    for predicate in &spec.predicates {
        if predicate.threshold.is_empty()
            || predicate.threshold.len() > 64
            || predicate.threshold.chars().any(char::is_control)
        {
            return Err(TradeXError::new("IPC_PAYLOAD_INVALID"));
        }
        normalize_decimal(&predicate.threshold)?;
    }
    Ok(())
}

fn parse(request: &ScreenerRequest, received_timestamp: &str) -> Result<ScreenerResult> {
    let text = request.natural_language.to_lowercase();
    let universe = if text.contains("crypto") || text.contains("spot") {
        ScreenerUniverse::CryptoSpot
    } else if text.contains("technology") || text.contains("tech") {
        ScreenerUniverse::UsLargeCapTechnology
    } else {
        ScreenerUniverse::UsEquities
    };
    let mut predicates = Vec::new();
    let mut unsupported_reason = None;
    if let Some(reason) = add_predicate(
        &text,
        &["revenue growth", "growth"],
        ScreenerPredicateField::RevenueGrowth,
        "0.15",
        true,
        &mut predicates,
    )? {
        unsupported_reason = Some(reason);
    }
    if let Some(reason) = add_predicate(
        &text,
        &["estimate revision", "estimate revisions", "revisions"],
        ScreenerPredicateField::EstimateRevision,
        "0",
        true,
        &mut predicates,
    )? {
        unsupported_reason.get_or_insert(reason);
    }
    if let Some(reason) = add_predicate(
        &text,
        &["rsi"],
        ScreenerPredicateField::Rsi,
        "70",
        false,
        &mut predicates,
    )? {
        unsupported_reason.get_or_insert(reason);
    }
    if let Some(reason) = add_predicate(
        &text,
        &["price change", "momentum"],
        ScreenerPredicateField::PriceChange,
        "0",
        true,
        &mut predicates,
    )? {
        unsupported_reason.get_or_insert(reason);
    }
    unsupported_reason = unsupported_reason.or_else(|| unsupported_filter_reason(&text));
    let (rank_field, direction) = rank_from_text(&text);
    let rank_spec = RankSpec {
        field: rank_field,
        direction,
    };
    let limit = parse_limit(&text).unwrap_or(10);
    let parsed_filter_spec = FilterSpec {
        universe,
        predicates,
    };
    let filter_spec = request.filter_spec.clone().unwrap_or(parsed_filter_spec);
    validate_filter(&filter_spec)?;
    let rank_spec = request.rank_spec.clone().unwrap_or(rank_spec);
    let limit = request.limit.unwrap_or(limit);
    let (state, reason, limitations) = match unsupported_reason {
        Some(reason) => (
            ScreenerResultState::Failed,
            reason.clone(),
            vec![format!("SCREENER_FILTER_UNSUPPORTED: {reason}")],
        ),
        None if filter_spec.predicates.is_empty() => (
            ScreenerResultState::Failed,
            "No supported screener predicate was found in the request.".into(),
            vec!["SCREENER_FILTER_UNSUPPORTED".into()],
        ),
        None => (
            ScreenerResultState::Parsed,
            "Natural-language conditions were parsed into a bounded FilterSpec and RankSpec."
                .into(),
            Vec::new(),
        ),
    };
    let revision = revision_for(request, &filter_spec, &rank_spec, limit)?;
    Ok(ScreenerResult {
        workspace_id: request.workspace_id.clone(),
        operation: request.operation,
        state,
        natural_language: request.natural_language.clone(),
        focus: request.focus.clone(),
        filter_spec: Some(filter_spec.clone()),
        rank_spec: Some(rank_spec.clone()),
        revision: Some(revision),
        applied_conditions: applied_conditions(&filter_spec, &rank_spec, limit),
        candidates: Vec::new(),
        candidate_count: 0,
        source_ids: Vec::new(),
        provider_timestamp: None,
        received_timestamp: received_timestamp.to_owned(),
        availability_reason: reason,
        limitations,
        fixture_label: None,
    })
}

fn add_predicate(
    text: &str,
    keywords: &[&str],
    field: ScreenerPredicateField,
    default: &str,
    percent: bool,
    predicates: &mut Vec<ScreenerPredicate>,
) -> Result<Option<String>> {
    let Some(index) = keywords
        .iter()
        .filter_map(|keyword| text.find(keyword))
        .min()
    else {
        return Ok(None);
    };
    let suffix = &text[index..text.len().min(index + 96)];
    let clause = suffix.split([',', ';']).next().unwrap_or_default();
    if let Some(operator) = unsupported_operator(clause) {
        return Ok(Some(format!(
            "Unsupported operator '{operator}' for {field:?}."
        )));
    }
    let raw = match numeric_after(suffix) {
        Some(value) => value,
        None if has_explicit_operator(clause) => {
            return Ok(Some(format!("Missing threshold for {field:?}.")));
        }
        None => default.to_owned(),
    };
    let threshold = if percent || raw.ends_with('%') {
        percent_to_decimal(raw.trim_end_matches('%'))?
    } else {
        normalize_decimal(&raw)?
    };
    predicates.push(ScreenerPredicate {
        field,
        operator: operator_before(text, index),
        threshold,
    });
    Ok(None)
}

fn has_explicit_operator(text: &str) -> bool {
    [
        "above",
        "below",
        "over",
        "under",
        "greater",
        "less",
        "at least",
        "at most",
        "more than",
        "fewer than",
        ">",
        "<",
    ]
    .iter()
    .any(|operator| text.contains(operator))
}

fn unsupported_operator(text: &str) -> Option<&'static str> {
    [
        "around",
        "approximately",
        "roughly",
        "between",
        "equal",
        "not",
    ]
    .iter()
    .find(|operator| text.contains(**operator))
    .copied()
}

fn unsupported_filter_reason(text: &str) -> Option<String> {
    if let Some((field, operator)) = text.split([',', ';']).find_map(|clause| {
        let clause = clause.trim();
        let has_supported_field = [
            "revenue growth",
            "growth",
            "estimate revision",
            "revisions",
            "rsi",
            "price change",
            "momentum",
        ]
        .iter()
        .any(|field| clause.contains(field));
        if has_explicit_operator(clause) && !has_supported_field {
            let operator = [
                "above",
                "below",
                "over",
                "under",
                "greater",
                "less",
                "at least",
                "at most",
                "more than",
                "fewer than",
            ]
            .iter()
            .find(|operator| clause.contains(**operator))
            .copied()
            .unwrap_or("comparison");
            return Some((clause, operator));
        }
        None
    }) {
        return Some(format!(
            "Unsupported filter field in clause '{field}' ({operator})."
        ));
    }
    [
        ("p/e", "price-to-earnings"),
        ("pe ratio", "price-to-earnings"),
        ("price/earnings", "price-to-earnings"),
        ("price to earnings", "price-to-earnings"),
        ("market cap", "market capitalization"),
        ("market capitalization", "market capitalization"),
        ("dividend", "dividend"),
        ("volatility", "volatility"),
        ("volume", "volume"),
        ("sector", "sector"),
    ]
    .iter()
    .find_map(|(term, label)| {
        text.contains(term)
            .then(|| format!("Unsupported filter field: {label}."))
    })
}

fn numeric_after(text: &str) -> Option<String> {
    let mut started = false;
    let mut value = String::new();
    for character in text.chars() {
        if !started && (character.is_ascii_digit() || character == '-') {
            started = true;
        }
        if !started && matches!(character, ',' | ';') {
            break;
        }
        if started && (character.is_ascii_digit() || matches!(character, '.' | '-' | '%')) {
            value.push(character);
        } else if started {
            break;
        }
    }
    if value.ends_with('.') {
        value.pop();
    }
    (!value.is_empty() && value != "-").then_some(value)
}

fn operator_before(text: &str, index: usize) -> ScreenerOperator {
    let before = &text[index.saturating_sub(36)..index];
    let after = text[index..].split([',', ';']).next().unwrap_or_default();
    if before.contains("below")
        || before.contains("under")
        || before.contains("less")
        || before.contains("fewer than")
        || before.contains('<')
        || after.contains("below")
        || after.contains("under")
        || after.contains("less")
        || after.contains("fewer than")
        || after.contains('<')
    {
        ScreenerOperator::LessThan
    } else if before.contains("at least")
        || before.contains("or more")
        || before.contains(">=")
        || after.contains("at least")
        || after.contains("or more")
        || after.contains(">=")
    {
        ScreenerOperator::GreaterOrEqual
    } else if before.contains("at most")
        || before.contains("or less")
        || after.contains("at most")
        || after.contains("or less")
    {
        ScreenerOperator::LessOrEqual
    } else {
        ScreenerOperator::GreaterThan
    }
}

fn rank_from_text(text: &str) -> (ScreenerRankField, ScreenerDirection) {
    let field = if text.contains("momentum") || text.contains("price change") {
        ScreenerRankField::Momentum
    } else if text.contains("revision") {
        ScreenerRankField::RevisionStrength
    } else {
        ScreenerRankField::Quality
    };
    let direction = if text.contains("ascending")
        || text.contains("lowest")
        || text.contains("smallest")
        || text.contains(" asc")
    {
        ScreenerDirection::Asc
    } else {
        ScreenerDirection::Desc
    };
    (field, direction)
}

fn parse_limit(text: &str) -> Option<u32> {
    let index = text.find("top ")?;
    let number = text[index + 4..]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>();
    number.parse().ok().filter(|value| (1..=50).contains(value))
}

fn percent_to_decimal(value: &str) -> Result<String> {
    let normalized = normalize_decimal(value)?;
    let (negative, digits) = normalized
        .strip_prefix('-')
        .map_or((false, normalized.as_str()), |value| (true, value));
    let mut digits = digits.replace('.', "");
    let scale = normalized
        .split_once('.')
        .map_or(0, |(_, fraction)| fraction.len());
    let target_scale = scale + 2;
    if digits.len() <= target_scale {
        digits = format!("{}{}", "0".repeat(target_scale + 1 - digits.len()), digits);
    }
    let split = digits.len() - target_scale;
    let whole = digits[..split].trim_start_matches('0');
    let whole = if whole.is_empty() { "0" } else { whole };
    let fraction = digits[split..].trim_end_matches('0');
    let value = if fraction.is_empty() {
        whole.to_owned()
    } else {
        format!("{whole}.{fraction}")
    };
    Ok(if negative && value != "0" {
        format!("-{value}")
    } else {
        value
    })
}

fn revision_for(
    request: &ScreenerRequest,
    filter_spec: &FilterSpec,
    rank_spec: &RankSpec,
    limit: u32,
) -> Result<String> {
    let bytes = serde_json::to_vec(&json!({
        "naturalLanguage": request.natural_language,
        "focus": request.focus,
        "filterSpec": filter_spec,
        "rankSpec": rank_spec,
        "limit": limit,
    }))
    .map_err(|_| TradeXError::new("SCREENER_REVISION_FAILED"))?;
    Ok(format!("sha256:{}", hex::encode(Sha256::digest(bytes))))
}

fn parsed_limit(result: &ScreenerResult) -> Option<u32> {
    result.applied_conditions.iter().find_map(|condition| {
        condition
            .strip_prefix("Limit: ")
            .and_then(|value| value.parse().ok())
    })
}

fn applied_conditions(filter: &FilterSpec, rank: &RankSpec, limit: u32) -> Vec<String> {
    let mut conditions = vec![format!("Universe: {:?}", filter.universe)];
    conditions.extend(filter.predicates.iter().map(|predicate| {
        format!(
            "{:?} {:?} {}",
            predicate.field, predicate.operator, predicate.threshold
        )
    }));
    conditions.push(format!("Rank: {:?} {:?}", rank.field, rank.direction));
    conditions.push(format!("Limit: {limit}"));
    conditions
}

fn blocked_result(
    request: &ScreenerRequest,
    filter: &FilterSpec,
    rank: &RankSpec,
    revision: String,
    sources: &[DataSourceEntry],
    received_timestamp: &str,
) -> ScreenerResult {
    let required = required_sources(filter);
    let missing = required
        .iter()
        .filter(|source_id| {
            sources
                .iter()
                .find(|source| source.source_id == **source_id)
                .is_none_or(|source| source.status != DataSourceStatus::Available)
        })
        .copied()
        .collect::<Vec<_>>();
    let reason = if missing.is_empty() {
        "Provider screener adapters are not configured; no network request was made.".into()
    } else {
        format!(
            "Required source(s) unavailable or unverified: {}.",
            missing.join(", ")
        )
    };
    let limit = request.limit.unwrap_or(10);
    ScreenerResult {
        workspace_id: request.workspace_id.clone(),
        operation: request.operation,
        state: ScreenerResultState::BlockedExternal,
        natural_language: request.natural_language.clone(),
        focus: request.focus.clone(),
        filter_spec: Some(filter.clone()),
        rank_spec: Some(rank.clone()),
        revision: Some(revision),
        applied_conditions: applied_conditions(filter, rank, limit),
        candidates: Vec::new(),
        candidate_count: 0,
        source_ids: required.into_iter().map(str::to_owned).collect(),
        provider_timestamp: None,
        received_timestamp: received_timestamp.into(),
        availability_reason: reason,
        limitations: vec!["No external provider call is made by this read-only slice.".into()],
        fixture_label: None,
    }
}

fn required_sources(filter: &FilterSpec) -> Vec<&'static str> {
    let mut required = Vec::new();
    if matches!(
        filter.universe,
        ScreenerUniverse::UsEquities | ScreenerUniverse::UsLargeCapTechnology
    ) && !filter.predicates.is_empty()
    {
        required.push("OD-001");
    }
    if filter.predicates.iter().any(|predicate| {
        matches!(
            predicate.field,
            ScreenerPredicateField::RevenueGrowth | ScreenerPredicateField::EstimateRevision
        )
    }) {
        required.push("OD-003");
    }
    required.sort_unstable();
    required.dedup();
    required
}

#[derive(Clone, Copy)]
struct FixtureRow {
    instrument_id: &'static str,
    symbol: &'static str,
    universe: ScreenerUniverse,
    revenue_growth: Option<&'static str>,
    estimate_revision: Option<&'static str>,
    rsi: Option<&'static str>,
    price_change: Option<&'static str>,
    quality: &'static str,
    revision_strength: &'static str,
    momentum: &'static str,
}

fn fixture_rows(universe: ScreenerUniverse) -> Vec<FixtureRow> {
    [
        FixtureRow {
            instrument_id: "equity:US:AAPL",
            symbol: "AAPL",
            universe: ScreenerUniverse::UsLargeCapTechnology,
            revenue_growth: Some("0.22"),
            estimate_revision: Some("0.08"),
            rsi: Some("62"),
            price_change: Some("0.12"),
            quality: "0.90",
            revision_strength: "0.08",
            momentum: "0.12",
        },
        FixtureRow {
            instrument_id: "equity:US:MSFT",
            symbol: "MSFT",
            universe: ScreenerUniverse::UsLargeCapTechnology,
            revenue_growth: Some("0.18"),
            estimate_revision: Some("0.12"),
            rsi: Some("55"),
            price_change: Some("0.10"),
            quality: "0.95",
            revision_strength: "0.12",
            momentum: "0.10",
        },
        FixtureRow {
            instrument_id: "crypto:BTC/USDT:spot",
            symbol: "BTC/USDT",
            universe: ScreenerUniverse::CryptoSpot,
            revenue_growth: None,
            estimate_revision: None,
            rsi: Some("58"),
            price_change: Some("0.20"),
            quality: "0.70",
            revision_strength: "0",
            momentum: "0.20",
        },
        FixtureRow {
            instrument_id: "crypto:ETH/USDT:spot",
            symbol: "ETH/USDT",
            universe: ScreenerUniverse::CryptoSpot,
            revenue_growth: None,
            estimate_revision: None,
            rsi: Some("64"),
            price_change: Some("0.07"),
            quality: "0.65",
            revision_strength: "0",
            momentum: "0.07",
        },
    ]
    .into_iter()
    .filter(|row| match universe {
        ScreenerUniverse::UsEquities | ScreenerUniverse::UsLargeCapTechnology => {
            matches!(row.universe, ScreenerUniverse::UsLargeCapTechnology)
        }
        ScreenerUniverse::CryptoSpot => row.universe == ScreenerUniverse::CryptoSpot,
    })
    .collect()
}

fn feature_value(row: FixtureRow, field: ScreenerPredicateField) -> Option<&'static str> {
    match field {
        ScreenerPredicateField::RevenueGrowth => row.revenue_growth,
        ScreenerPredicateField::EstimateRevision => row.estimate_revision,
        ScreenerPredicateField::Rsi => row.rsi,
        ScreenerPredicateField::PriceChange => row.price_change,
    }
}

fn matches_predicates(row: &FixtureRow, predicates: &[ScreenerPredicate]) -> bool {
    predicates.iter().all(|predicate| {
        feature_value(*row, predicate.field).is_some_and(|value| {
            let Ok(ordering) = compare_decimal(value, &predicate.threshold) else {
                return false;
            };
            match predicate.operator {
                ScreenerOperator::GreaterThan => ordering == Ordering::Greater,
                ScreenerOperator::GreaterOrEqual => ordering != Ordering::Less,
                ScreenerOperator::LessThan => ordering == Ordering::Less,
                ScreenerOperator::LessOrEqual => ordering != Ordering::Greater,
            }
        })
    })
}

fn candidate(row: FixtureRow, _rank: &RankSpec) -> ScreenerCandidate {
    ScreenerCandidate {
        instrument_id: row.instrument_id.into(),
        symbol: row.symbol.into(),
        rank: 0,
        features: [
            (ScreenerFeatureField::RevenueGrowth, row.revenue_growth),
            (
                ScreenerFeatureField::EstimateRevision,
                row.estimate_revision,
            ),
            (ScreenerFeatureField::Rsi, row.rsi),
            (ScreenerFeatureField::PriceChange, row.price_change),
            (ScreenerFeatureField::Quality, Some(row.quality)),
            (
                ScreenerFeatureField::RevisionStrength,
                Some(row.revision_strength),
            ),
            (ScreenerFeatureField::Momentum, Some(row.momentum)),
        ]
        .into_iter()
        .filter_map(|(field, value)| {
            value.map(|value| ScreenerFeature {
                field,
                value: value.into(),
            })
        })
        .collect(),
        provenance: ScreenerProvenance {
            source_id: FIXTURE_SOURCE.into(),
            provider_timestamp: Some(FIXTURE_TIMESTAMP.into()),
            received_timestamp: FIXTURE_TIMESTAMP.into(),
            freshness: ResearchFreshness::Healthy,
            quality: ResearchQuality::Degraded,
        },
        limitation: Some(FIXTURE_LIMITATION.into()),
    }
}

fn compare_candidates(
    left: &ScreenerCandidate,
    right: &ScreenerCandidate,
    rank: &RankSpec,
) -> Ordering {
    let field = match rank.field {
        ScreenerRankField::Quality => ScreenerFeatureField::Quality,
        ScreenerRankField::RevisionStrength => ScreenerFeatureField::RevisionStrength,
        ScreenerRankField::Momentum => ScreenerFeatureField::Momentum,
    };
    let left_value = left
        .features
        .iter()
        .find(|feature| feature.field == field)
        .map(|feature| feature.value.as_str())
        .unwrap_or("0");
    let right_value = right
        .features
        .iter()
        .find(|feature| feature.field == field)
        .map(|feature| feature.value.as_str())
        .unwrap_or("0");
    let ordering = compare_decimal(left_value, right_value).unwrap_or(Ordering::Equal);
    if rank.direction == ScreenerDirection::Desc {
        ordering.reverse()
    } else {
        ordering
    }
}

fn normalize_decimal(value: &str) -> Result<String> {
    crate::provider_io::decimal(&Value::String(value.into()))
        .map_err(|_| TradeXError::new("IPC_PAYLOAD_INVALID"))
}

fn compare_decimal(left: &str, right: &str) -> Result<Ordering> {
    let left = normalize_decimal(left)?;
    let right = normalize_decimal(right)?;
    let (left_negative, left_digits, left_scale) = decimal_parts(&left);
    let (right_negative, right_digits, right_scale) = decimal_parts(&right);
    if left_negative != right_negative {
        return Ok(if left_negative {
            Ordering::Less
        } else {
            Ordering::Greater
        });
    }
    let scale = left_scale.max(right_scale);
    let left_digits = format!("{}{}", left_digits, "0".repeat(scale - left_scale));
    let right_digits = format!("{}{}", right_digits, "0".repeat(scale - right_scale));
    let ordering = left_digits
        .len()
        .cmp(&right_digits.len())
        .then_with(|| left_digits.cmp(&right_digits));
    Ok(if left_negative {
        ordering.reverse()
    } else {
        ordering
    })
}

fn decimal_parts(value: &str) -> (bool, String, usize) {
    let (negative, value) = value
        .strip_prefix('-')
        .map_or((false, value), |value| (true, value));
    let (whole, fraction) = value.split_once('.').unwrap_or((value, ""));
    (negative, format!("{whole}{fraction}"), fraction.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::ResearchFocus;

    fn request(operation: ScreenerOperation) -> ScreenerRequest {
        ScreenerRequest {
            workspace_id: "ws".into(),
            operation,
            natural_language: "US large-cap technology stocks with revenue growth above 15%, positive estimate revisions, and RSI below 70".into(),
            focus: Some(ResearchFocus::Equity),
            filter_spec: None,
            rank_spec: None,
            revision: None,
            limit: Some(10),
        }
    }

    #[test]
    fn parse_is_deterministic_and_bounded() {
        let parsed = screen(
            &request(ScreenerOperation::Parse),
            &[],
            FIXTURE_TIMESTAMP,
            false,
        )
        .unwrap();
        assert_eq!(parsed.state, ScreenerResultState::Parsed);
        assert_eq!(parsed.filter_spec.as_ref().unwrap().predicates.len(), 3);
        assert_eq!(
            parsed.revision,
            screen(
                &request(ScreenerOperation::Parse),
                &[],
                FIXTURE_TIMESTAMP,
                false
            )
            .unwrap()
            .revision
        );
    }

    #[test]
    fn fixture_run_filters_and_ranks_candidates() {
        let parsed = screen(
            &request(ScreenerOperation::Parse),
            &[],
            FIXTURE_TIMESTAMP,
            true,
        )
        .unwrap();
        let mut run = request(ScreenerOperation::Run);
        run.filter_spec = parsed.filter_spec;
        run.rank_spec = parsed.rank_spec;
        run.revision = parsed.revision;
        let result = screen(&run, &[], FIXTURE_TIMESTAMP, true).unwrap();
        assert_eq!(result.state, ScreenerResultState::Completed);
        assert_eq!(
            result
                .candidates
                .iter()
                .map(|row| row.symbol.as_str())
                .collect::<Vec<_>>(),
            vec!["MSFT", "AAPL"]
        );
    }

    #[test]
    fn run_rejects_stale_revision_and_blocks_external_sources() {
        let parsed = screen(
            &request(ScreenerOperation::Parse),
            &[],
            FIXTURE_TIMESTAMP,
            false,
        )
        .unwrap();
        let mut run = request(ScreenerOperation::Run);
        run.filter_spec = parsed.filter_spec;
        run.rank_spec = parsed.rank_spec;
        run.revision = Some("sha256:stale".into());
        assert_eq!(
            screen(&run, &[], FIXTURE_TIMESTAMP, false)
                .unwrap_err()
                .code,
            "SCREENER_REVISION_STALE"
        );
        run.revision = parsed.revision;
        assert_eq!(
            screen(&run, &[], FIXTURE_TIMESTAMP, false).unwrap().state,
            ScreenerResultState::BlockedExternal
        );
    }

    #[test]
    fn unsupported_language_is_explicit() {
        let mut request = request(ScreenerOperation::Parse);
        request.natural_language = "find unusual companies".into();
        let parsed = screen(&request, &[], FIXTURE_TIMESTAMP, false).unwrap();
        assert_eq!(parsed.state, ScreenerResultState::Failed);
        assert!(
            parsed
                .limitations
                .iter()
                .any(|value| value.contains("SCREENER_FILTER_UNSUPPORTED"))
        );
        request.natural_language = "Find stocks with P/E below 20.".into();
        let field_failure = screen(&request, &[], FIXTURE_TIMESTAMP, false).unwrap();
        assert!(
            field_failure
                .availability_reason
                .contains("Unsupported filter field")
        );
        request.natural_language = "RSI fewer than 70".into();
        let fewer = screen(&request, &[], FIXTURE_TIMESTAMP, false).unwrap();
        assert_eq!(
            fewer.filter_spec.unwrap().predicates[0].operator,
            ScreenerOperator::LessThan
        );
        request.natural_language = "RSI around 70".into();
        let around = screen(&request, &[], FIXTURE_TIMESTAMP, false).unwrap();
        assert!(around.availability_reason.contains("Unsupported operator"));
    }

    #[test]
    fn run_cannot_bypass_failed_parse() {
        let mut run = request(ScreenerOperation::Run);
        run.natural_language = "find unusual companies".into();
        let parsed = screen(&run, &[], FIXTURE_TIMESTAMP, true).unwrap();
        run.filter_spec = parsed.filter_spec;
        run.rank_spec = parsed.rank_spec;
        run.revision = parsed.revision;
        let result = screen(&run, &[], FIXTURE_TIMESTAMP, true).unwrap();
        assert_eq!(result.state, ScreenerResultState::Failed);
        assert!(result.candidates.is_empty());
    }
}
