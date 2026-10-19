//! Background jobs: engine work that takes a while (lookups, downloads, merges) is wrapped in an
//! [`Effect::Job`] with an ID and a label, listed in the jobs panel (`J`) until
//! [`Action::JobFinished`](super::Action::JobFinished) arrives.

use super::Effect;
use crate::state::{Job, State};

/// `effect` as a job labelled `label`, recorded as running.
pub fn start(state: &mut State, label: impl Into<String>, effect: Effect) -> Effect {
    let id = state.next_job;
    state.next_job += 1;
    state.jobs.push(Job { id, label: label.into(), started: std::time::Instant::now() });
    Effect::Job { id, effect: Box::new(effect) }
}

/// The job `id` ended.
pub fn finished(state: &mut State, id: u64) {
    state.jobs.retain(|job| job.id != id);
}
