pub mod interface;
pub mod validators;

use crate::contract::MIN_MINT_PRICE;
use crate::state::{Config, Stage, CONFIG};
use crate::ContractError;
use cosmwasm_std::{ensure, Env, StdError, Storage};

pub fn fetch_active_stage(deps: &dyn Storage, env: &Env) -> Option<Stage> {
    let config: Config = CONFIG.load(deps).ok()?;
    let current_time = env.block.time;
    config
        .stages
        .iter()
        .find(|stage| stage.start_time <= current_time && current_time <= stage.end_time)
        .cloned()
}

pub fn fetch_active_stage_index(deps: &dyn Storage, env: &Env) -> Option<u32> {
    let config: Config = CONFIG.load(deps).ok()?;
    let current_time = env.block.time;
    config
        .stages
        .iter()
        .position(|stage| stage.start_time <= current_time && current_time <= stage.end_time)
        .map(|i| i as u32)
}

pub fn validate_stages(env: &Env, stages: &[Stage]) -> Result<(), ContractError> {
    ensure!(
        !stages.is_empty(),
        StdError::generic_err("Must have at least one stage")
    );
    ensure!(
        stages.len() < 4,
        StdError::generic_err("Cannot have more than 3 stages")
    );

    // Check mint price is valid
    if stages
        .iter()
        .any(|stage| stage.mint_price.amount.u128() < MIN_MINT_PRICE)
    {
        return Err(ContractError::InvalidUnitPrice(
            MIN_MINT_PRICE,
            stages
                .iter()
                .map(|s| s.mint_price.amount.u128())
                .min()
                .unwrap(),
        ));
    }

    ensure!(
        stages[0].start_time > env.block.time,
        StdError::generic_err("Stages must have a start time in the future")
    );
    for i in 0..stages.len() {
        let stage = &stages[i];
        ensure!(
            stage.start_time < stage.end_time,
            StdError::generic_err("Stage start time must be before the end time")
        );

        for other_stage in stages.iter().skip(i + 1) {
            ensure!(
                other_stage.start_time >= stage.end_time,
                StdError::generic_err("Stages must have non-overlapping times")
            );
        }
    }
    Ok(())
}
