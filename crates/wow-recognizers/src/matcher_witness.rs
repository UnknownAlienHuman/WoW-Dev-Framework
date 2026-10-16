//! Bounded retention of original matcher outputs for real producer consumers.
use crate::{
    RecognizerError, RecognizerErrorCode, RecognizerMatch, RecognizerProposedAssertion,
    RecognizerResult, RecognizerRuleOutcome,
};
use serde::Serialize;
use std::{
    io::{self, Write},
    sync::atomic::{AtomicBool, Ordering},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OriginalMatcherWitness {
    pub output_id: Box<str>,
    pub matched: RecognizerMatch,
}
pub(crate) struct CapturedMatcherWitness {
    pub witness: OriginalMatcherWitness,
    pub rule_id: Box<str>,
    pub rule_version: u32,
    pub kind: wow_graph::GraphAssertionKind,
}
pub(crate) fn capture(
    outcome: &RecognizerRuleOutcome,
    proposal: &RecognizerProposedAssertion,
    used: &mut u64,
    stop: &AtomicBool,
) -> RecognizerResult<CapturedMatcherWitness> {
    checkpoint(stop)?;
    let (match_id, decisive_fact_ids) = match proposal {
        RecognizerProposedAssertion::Entity {
            match_id,
            decisive_fact_ids,
            ..
        }
        | RecognizerProposedAssertion::Relation {
            match_id,
            decisive_fact_ids,
            ..
        } => (match_id, decisive_fact_ids),
    };
    let index = outcome
        .matches()
        .binary_search_by(|item| item.match_id().cmp(match_id))
        .map_err(|_| failure(RecognizerErrorCode::AdapterBindingMissing))?;
    let matched = &outcome.matches()[index];
    if matched.decisive_fact_ids() != decisive_fact_ids
        || matched.confidence() != proposal.confidence()
    {
        return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
    }
    charge(&(proposal.output_id(), matched), used, stop)?;
    Ok(CapturedMatcherWitness {
        witness: OriginalMatcherWitness {
            output_id: proposal.output_id().into(),
            matched: matched.clone(),
        },
        rule_id: outcome.rule_id().into(),
        rule_version: outcome.rule_version(),
        kind: match proposal {
            RecognizerProposedAssertion::Entity { .. } => wow_graph::GraphAssertionKind::Entity,
            RecognizerProposedAssertion::Relation { .. } => wow_graph::GraphAssertionKind::Relation,
        },
    })
}
pub(crate) fn charge(
    value: &impl Serialize,
    used: &mut u64,
    stop: &AtomicBool,
) -> RecognizerResult<()> {
    checkpoint(stop)?;
    let mut writer = Counter {
        used,
        stop,
        failure: None,
    };
    if serde_json::to_writer(&mut writer, value).is_err() {
        checkpoint(stop)?;
        return Err(failure(
            writer
                .failure
                .unwrap_or(RecognizerErrorCode::AdapterBindingInvalid),
        ));
    }
    checkpoint(stop)
}
struct Counter<'a> {
    used: &'a mut u64,
    stop: &'a AtomicBool,
    failure: Option<RecognizerErrorCode>,
}
impl Write for Counter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let failed = if self.stop.load(Ordering::Acquire) {
            Some(RecognizerErrorCode::Cancelled)
        } else if let Some(next) = self
            .used
            .checked_add(bytes.len() as u64)
            .filter(|next| *next <= 16 * 1024 * 1024)
        {
            *self.used = next;
            None
        } else {
            Some(RecognizerErrorCode::BudgetExceeded)
        };
        if let Some(code) = failed {
            self.failure = Some(code);
            return Err(io::Error::other("matcher witness bound"));
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn checkpoint(stop: &AtomicBool) -> RecognizerResult<()> {
    if stop.load(Ordering::Acquire) {
        Err(failure(RecognizerErrorCode::Cancelled))
    } else {
        Ok(())
    }
}
fn failure(code: RecognizerErrorCode) -> RecognizerError {
    RecognizerError::new(code, "original matcher witness rejected")
}
