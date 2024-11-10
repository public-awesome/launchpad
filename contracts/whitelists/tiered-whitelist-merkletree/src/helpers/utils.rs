use crate::contract::MAX_PER_ADDRESS_LIMIT;
use crate::state::{Config, Stage, CONFIG};
use crate::ContractError;
use cosmwasm_std::{ensure, Env, StdError, StdResult, Storage};
use url::Url;

pub fn verify_tree_uri(tree_uri: &str) -> StdResult<()> {
    let res = Url::parse(tree_uri);
    if res.is_err() {
        return Err(cosmwasm_std::StdError::GenericErr {
            msg: "Invalid tree uri".to_string(),
        });
    }
    Ok(())
}

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

    // Check per address limit is valid
    if stages.iter().any(|stage| {
        stage.per_address_limit == 0 || stage.per_address_limit > MAX_PER_ADDRESS_LIMIT
    }) {
        return Err(ContractError::InvalidPerAddressLimit {
            max: MAX_PER_ADDRESS_LIMIT.to_string(),
            got: stages
                .iter()
                .map(|s| s.per_address_limit)
                .max()
                .unwrap()
                .to_string(),
        });
    }

    // Check stages have matching mint price denoms
    let mint_denom = stages[0].mint_price.denom.clone();
    ensure!(
        stages
            .iter()
            .all(|stage| stage.mint_price.denom == mint_denom),
        StdError::generic_err("All stages must have the same mint price denom")
    );

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
