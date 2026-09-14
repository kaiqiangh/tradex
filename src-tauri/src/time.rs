use crate::protocol::{Remediation, Result, TimeConfidence, TimeStatus, TradeXError};
use ::time::{OffsetDateTime, format_description::well_known::Rfc3339};
use std::time::Instant;

/// Wall and monotonic readings may differ by this amount before trust is lost.
pub const CLOCK_JUMP_TOLERANCE_MS: i128 = 2_000;
/// Provider/server samples outside this offset are not safe for authority use.
pub const MAX_PROVIDER_OFFSET_MS: i64 = 5_000;
const MAX_MONOTONIC_MS: u128 = 9_007_199_254_740_991;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Reading {
    wall_ms: i128,
    monotonic_ms: u64,
    provider_offset_ms: Option<i64>,
}

pub struct TimeService {
    workspace_id: Option<String>,
    baseline: Option<Reading>,
    last: Option<Reading>,
    last_provider_offset_ms: Option<i64>,
    confidence: TimeConfidence,
    reason: &'static str,
    needs_revalidation: bool,
    started: Instant,
}

impl Default for TimeService {
    fn default() -> Self {
        Self {
            workspace_id: None,
            baseline: None,
            last: None,
            last_provider_offset_ms: None,
            confidence: TimeConfidence::ClockUncertain,
            reason: "Open a workspace and synchronize time before Live authority decisions.",
            needs_revalidation: true,
            started: Instant::now(),
        }
    }
}

impl TimeService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self, workspace_id: &str) {
        self.workspace_id = Some(workspace_id.to_owned());
        self.baseline = None;
        self.last = None;
        self.last_provider_offset_ms = None;
        self.confidence = TimeConfidence::ClockUncertain;
        self.reason =
            "Workspace opened or resumed; synchronize time before Live authority decisions.";
        self.needs_revalidation = true;
    }

    /// Resume hooks use the same fail-closed transition as workspace reopen.
    pub fn resume(&mut self, workspace_id: &str) {
        self.reset(workspace_id);
    }

    pub fn status(&mut self, workspace_id: &str) -> Result<TimeStatus> {
        self.ensure_workspace(workspace_id)?;
        Ok(self.observe_reading(workspace_id, self.reading()))
    }

    pub fn revalidate(&mut self, workspace_id: &str) -> Result<TimeStatus> {
        self.ensure_workspace(workspace_id)?;
        Ok(self.revalidate_reading(workspace_id, self.reading()))
    }

    /// Future approval, risk, and dispatch paths must use this shared gate.
    pub fn require_trusted(&self) -> Result<()> {
        if self.confidence == TimeConfidence::Trusted && !self.needs_revalidation {
            Ok(())
        } else {
            Err(TradeXError::new("CLOCK_SKEW"))
        }
    }

    fn ensure_workspace(&self, workspace_id: &str) -> Result<()> {
        if self.workspace_id.as_deref() == Some(workspace_id) {
            Ok(())
        } else {
            Err(TradeXError::new("IPC_AGGREGATE_NOT_FOUND"))
        }
    }

    fn reading(&self) -> Reading {
        Reading {
            wall_ms: OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000,
            monotonic_ms: self.started.elapsed().as_millis().min(MAX_MONOTONIC_MS) as u64,
            provider_offset_ms: None,
        }
    }

    fn observe_reading(&mut self, workspace_id: &str, reading: Reading) -> TimeStatus {
        debug_assert_eq!(self.workspace_id.as_deref(), Some(workspace_id));
        if let Some(offset) = reading.provider_offset_ms {
            self.last_provider_offset_ms = Some(offset);
            if i128::from(offset).abs() > i128::from(MAX_PROVIDER_OFFSET_MS) {
                self.invalidate(
                    TimeConfidence::Stale,
                    "Provider/server time offset exceeded the trusted tolerance.",
                    reading,
                );
                return self.status_from(workspace_id, reading);
            }
        }

        if let Some(previous) = self.last
            && reading.monotonic_ms < previous.monotonic_ms
        {
            self.invalidate(
                TimeConfidence::Stale,
                "The monotonic clock moved backwards; revalidate time before continuing.",
                reading,
            );
            return self.status_from(workspace_id, reading);
        }

        if !self.needs_revalidation {
            if let Some(baseline) = self.baseline {
                let wall_elapsed = reading.wall_ms - baseline.wall_ms;
                let monotonic_elapsed = i128::from(reading.monotonic_ms - baseline.monotonic_ms);
                if wall_elapsed < 0
                    || (wall_elapsed - monotonic_elapsed).abs() > CLOCK_JUMP_TOLERANCE_MS
                {
                    self.invalidate(
                        TimeConfidence::ClockUncertain,
                        "Wall-clock progression diverged from monotonic time; synchronize again.",
                        reading,
                    );
                    return self.status_from(workspace_id, reading);
                }
            }
            self.confidence = TimeConfidence::Trusted;
            self.reason = "Wall and monotonic time are within the trusted tolerance.";
        }

        self.last = Some(reading);
        self.status_from(workspace_id, reading)
    }

    fn revalidate_reading(&mut self, workspace_id: &str, reading: Reading) -> TimeStatus {
        debug_assert_eq!(self.workspace_id.as_deref(), Some(workspace_id));
        self.baseline = Some(reading);
        self.last = Some(reading);
        self.last_provider_offset_ms = reading.provider_offset_ms;
        self.needs_revalidation = false;
        self.confidence = TimeConfidence::Trusted;
        self.reason = "Time synchronized; Live authority decisions may use this trust result.";
        if reading
            .provider_offset_ms
            .is_some_and(|offset| i128::from(offset).abs() > i128::from(MAX_PROVIDER_OFFSET_MS))
        {
            self.invalidate(
                TimeConfidence::Stale,
                "Provider/server time offset exceeded the trusted tolerance.",
                reading,
            );
        }
        self.status_from(workspace_id, reading)
    }

    fn invalidate(&mut self, confidence: TimeConfidence, reason: &'static str, reading: Reading) {
        self.confidence = confidence;
        self.reason = reason;
        self.needs_revalidation = true;
        self.baseline = Some(reading);
        self.last = Some(reading);
    }

    fn status_from(&self, workspace_id: &str, reading: Reading) -> TimeStatus {
        let wall_clock = OffsetDateTime::from_unix_timestamp_nanos(reading.wall_ms * 1_000_000)
            .ok()
            .and_then(|time| time.format(&Rfc3339).ok())
            .unwrap_or_else(|| "UNAVAILABLE".into());
        TimeStatus {
            workspace_id: workspace_id.into(),
            confidence: self.confidence,
            wall_clock: wall_clock.clone(),
            monotonic_ms: reading.monotonic_ms,
            provider_offset_ms: reading.provider_offset_ms.or(self.last_provider_offset_ms),
            observed_at: wall_clock,
            reason: self.reason.into(),
            remediation: if self.confidence == TimeConfidence::Trusted {
                Remediation {
                    id: "none".into(),
                    label: "No action required".into(),
                }
            } else {
                Remediation {
                    id: "time_revalidate".into(),
                    label: "Synchronize time".into(),
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service() -> TimeService {
        let mut service = TimeService::new();
        service.reset("workspace-1");
        service
    }

    fn reading(wall_ms: i128, monotonic_ms: u64) -> Reading {
        Reading {
            wall_ms,
            monotonic_ms,
            provider_offset_ms: None,
        }
    }

    #[test]
    fn first_status_after_open_is_untrusted_until_explicit_revalidation() {
        let mut service = service();
        assert_eq!(
            service
                .observe_reading("workspace-1", reading(1_700_000_000_000, 100))
                .confidence,
            TimeConfidence::ClockUncertain
        );
        assert_eq!(
            service
                .revalidate_reading("workspace-1", reading(1_700_000_000_000, 101))
                .confidence,
            TimeConfidence::Trusted
        );
    }

    #[test]
    fn material_wall_jump_requires_revalidation() {
        let mut service = service();
        service.revalidate_reading("workspace-1", reading(1_700_000_000_000, 100));
        let status = service.observe_reading("workspace-1", reading(1_700_000_010_000, 200));
        assert_eq!(status.confidence, TimeConfidence::ClockUncertain);
        assert_eq!(status.remediation.id, "time_revalidate");
        assert!(service.require_trusted().is_err());
    }

    #[test]
    fn normal_wall_and_monotonic_elapsed_stays_trusted() {
        let mut service = service();
        service.revalidate_reading("workspace-1", reading(1_700_000_000_000, 100));
        let status = service.observe_reading("workspace-1", reading(1_700_000_001_000, 1_050));
        assert_eq!(status.confidence, TimeConfidence::Trusted);
        assert!(service.require_trusted().is_ok());
    }

    #[test]
    fn monotonic_rollback_is_stale() {
        let mut service = service();
        service.revalidate_reading("workspace-1", reading(1_700_000_000_000, 100));
        let status = service.observe_reading("workspace-1", reading(1_700_000_000_100, 99));
        assert_eq!(status.confidence, TimeConfidence::Stale);
        assert_eq!(
            status.reason,
            "The monotonic clock moved backwards; revalidate time before continuing."
        );
    }

    #[test]
    fn provider_offset_is_bounded_and_workspace_scoped() {
        let mut service = service();
        let status = service.revalidate_reading(
            "workspace-1",
            Reading {
                wall_ms: 1_700_000_000_000,
                monotonic_ms: 100,
                provider_offset_ms: Some(MAX_PROVIDER_OFFSET_MS + 1),
            },
        );
        assert_eq!(status.confidence, TimeConfidence::Stale);
        assert!(service.status("other").is_err());
    }

    #[test]
    fn resume_invalidates_an_existing_trusted_baseline() {
        let mut service = service();
        service.revalidate_reading("workspace-1", reading(1_700_000_000_000, 100));
        service.resume("workspace-1");
        assert_eq!(
            service
                .observe_reading("workspace-1", reading(1_700_000_000_001, 101))
                .confidence,
            TimeConfidence::ClockUncertain
        );
    }
}
