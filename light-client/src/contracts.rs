//! Predicates used in components contracts.

use std::time::Duration;

use crate::{
    store::LightStore,
    verifier::types::{Height, LightBlock, Status, Time},
};

/// Whether or not the given light store contains a verified or
/// trusted block at the given target height.
pub fn trusted_store_contains_block_at_target_height(
    light_store: &dyn LightStore,
    target_height: Height,
) -> bool {
    light_store.get(target_height, Status::Verified).is_some()
        || light_store.get(target_height, Status::Trusted).is_some()
}

/// Whether or not the given block is within the given trusting period,
/// relative to the given time.
pub fn is_within_trust_period(
    light_block: &LightBlock,
    trusting_period: Duration,
    now: Time,
) -> bool {
    let header_time = light_block.signed_header.header.time;
    match now - trusting_period {
        Ok(start) => header_time > start,
        Err(_) => false,
    }
}

/// Whether or not the given light store contains a trusted block
/// within the trusting period.
///
/// See `is_within_trust_period`.
pub fn light_store_contains_block_within_trusting_period(
    light_store: &dyn LightStore,
    trusting_period: Duration,
    now: Time,
) -> bool {
    light_store
        .all(Status::Trusted)
        .any(|lb| is_within_trust_period(&lb, trusting_period, now))
}
