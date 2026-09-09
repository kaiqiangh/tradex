use super::*;

pub(super) fn account_id(account: &Value) -> Result<String> {
    account["id"]
        .as_i64()
        .filter(|id| *id > 0)
        .map(|id| id.to_string())
        .ok_or_else(invalid)
}

// serde_json arbitrary_precision preserves the source number; never round through f64.
fn number(value: &Value) -> Result<String> {
    let raw = value.as_number().ok_or_else(invalid)?.to_string();
    let Some((mantissa, exponent)) = raw.split_once(['e', 'E']) else {
        return decimal(&Value::String(raw));
    };
    let exponent: i32 = exponent.parse().map_err(|_| invalid())?;
    if !(-128..=128).contains(&exponent) || mantissa.len() > 128 {
        return Err(invalid());
    }
    let (sign, mantissa) = mantissa
        .strip_prefix('-')
        .map_or(("", mantissa), |m| ("-", m));
    let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let digits = format!("{whole}{fraction}");
    if !digits.bytes().all(|b| b.is_ascii_digit()) {
        return Err(invalid());
    }
    let point = whole.len() as i32 + exponent;
    let expanded = if point <= 0 {
        format!("{sign}0.{}{digits}", "0".repeat((-point) as usize))
    } else if point as usize >= digits.len() {
        format!(
            "{sign}{digits}{}",
            "0".repeat(point as usize - digits.len())
        )
    } else {
        format!(
            "{sign}{}.{}",
            &digits[..point as usize],
            &digits[point as usize..]
        )
    };
    decimal(&Value::String(expanded))
}
fn optional(value: &Value, field: &str) -> Result<Option<String>> {
    match value.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => number(value).map(Some),
    }
}
fn currency(value: &Value) -> Result<String> {
    let s = text(value, "currency", 3)?;
    if s.len() != 3 || !s.bytes().all(|b| b.is_ascii_uppercase()) {
        return Err(invalid());
    }
    Ok(s)
}
fn ticker(value: &Value) -> Result<String> {
    let s = text(value, "ticker", 64)?;
    if !s
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b"._/-".contains(&b))
    {
        return Err(invalid());
    }
    Ok(s)
}
fn rows(value: &Value) -> Result<&Vec<Value>> {
    let rows = value.as_array().ok_or_else(invalid)?;
    if rows.len() > 10_000 {
        return Err(TradeXError::new("PROVIDER_DATA_INCOMPLETE"));
    }
    Ok(rows)
}

pub(super) fn observe(account: Value, positions: Value, orders: Value) -> Result<Observation> {
    let account_currency = currency(&account)?;
    let mut tickers = std::collections::HashSet::new();
    let positions = rows(&positions)?
        .iter()
        .map(|p| {
            let symbol = ticker(&p["instrument"])?;
            if !tickers.insert(symbol.clone()) {
                return Err(invalid());
            }
            let average_entry_price = optional(p, "averagePricePaid")?;
            let market_value = optional(&p["walletImpact"], "currentValue")?;
            let instrument_currency = Some(currency(&p["instrument"])?);
            let market_value_currency = if market_value.is_some() {
                Some(currency(&p["walletImpact"])?)
            } else {
                None
            };
            Ok(Position {
                symbol,
                quantity: number(&p["quantity"])?,
                market_value,
                average_entry_price,
                instrument_currency,
                market_value_currency,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let mut ids = std::collections::HashSet::new();
    let orders = rows(&orders)?
        .iter()
        .map(|o| {
            let broker_order_id = account_id(o)?;
            if !ids.insert(broker_order_id.clone()) {
                return Err(invalid());
            }
            let side = text(o, "side", 4)?;
            if !matches!(side.as_str(), "BUY" | "SELL") {
                return Err(invalid());
            }
            let status = text(o, "status", 32)?;
            if !matches!(
                status.as_str(),
                "LOCAL"
                    | "UNCONFIRMED"
                    | "CONFIRMED"
                    | "NEW"
                    | "CANCELLING"
                    | "CANCELLED"
                    | "PARTIALLY_FILLED"
                    | "FILLED"
                    | "REJECTED"
                    | "REPLACING"
                    | "REPLACED"
            ) {
                return Err(invalid());
            }
            let symbol = if o.get("instrument").is_some_and(Value::is_object) {
                ticker(&o["instrument"])?
            } else {
                ticker(o)?
            };
            if o.get("ticker").is_some() && ticker(o)? != symbol {
                return Err(invalid());
            }
            let strategy = text(o, "strategy", 8)?;
            let (quantity, notional, filled_quantity, filled_value) = match strategy.as_str() {
                "QUANTITY" => (
                    Some(number(&o["quantity"])?),
                    None,
                    optional(o, "filledQuantity")?,
                    None,
                ),
                "VALUE" => (
                    None,
                    Some(number(&o["value"])?),
                    None,
                    optional(o, "filledValue")?,
                ),
                _ => return Err(invalid()),
            };
            let limit_price = optional(o, "limitPrice")?;
            let currency =
                if notional.is_some() || limit_price.is_some() || o.get("currency").is_some() {
                    Some(currency(o)?)
                } else {
                    None
                };
            Ok(OpenOrder {
                kind: None,
                trigger_price: None,
                broker_order_id,
                symbol,
                side: side.to_ascii_lowercase(),
                quantity,
                notional,
                filled_quantity,
                filled_value,
                currency,
                status,
                limit_price,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let permissions = PermissionReview {
        detected: vec![
            "account.read".into(),
            "positions.read".into(),
            "orders.read".into(),
        ],
        ..PermissionReview::default()
    };
    Ok(Observation { data: AccountData {
        remote_account_id: account_id(&account)?,
        account_type: "Invest / Stocks ISA — subtype unavailable".into(),
        currency: Some(account_currency.clone()),
        balances: vec![Balance { locked: None, restricted_available: None, asset: account_currency, available: number(&account["cash"]["availableToTrade"])?, total: optional(&account,"totalValue")?, reserved: optional(&account["cash"],"reservedForOrders")?, in_pies: optional(&account["cash"],"inPies")? }],
        positions, open_orders: orders, capabilities: permissions.detected.clone(),
        limitations: vec![
            "Invest/Stocks ISA API only; the provider does not expose the exact account subtype or full key/IP scope. CFD and other account types are unsupported.".into(),
            "Cash and account value use the primary currency. Position prices use instrument currency; market values use wallet currency. No currency conversion is performed by TradeX.".into(),
            "API execution is restricted to primary account currency; API multi-currency conversion and value-order placement are unavailable. Orders shown here may have originated in other Trading 212 clients.".into(),
            "Manual refresh reads summary and pending orders at a limit of one request per five seconds per account (positions: one per second). A quota error requires a later manual retry.".into(),
            "TradeX execution, reconciliation and effective available funds are not configured. Successful reads do not grant trading authority.".into(),
        ],
    }, permissions })
}
