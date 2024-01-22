use cosmwasm_std::StdError;
use sg_controllers::{HookError, Hooks};

// re-export the proc macros
pub use sg_mint_hooks_derive::{sg_mint_hooks_execute, sg_mint_hooks_query};

pub const PREMINT_HOOKS: Hooks = Hooks::new("premint-hooks");
pub const POSTMINT_HOOKS: Hooks = Hooks::new("postmint-hooks");

const PREMINT_HOOK_REPLY_ID: u64 = 6902;
const POSTMINT_HOOK_REPLY_ID: u64 = 6903;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum MintHookError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Hook(#[from] HookError),

    #[error("pre-mint hook failed")]
    PreMintHookFailed {},

    #[error("post-mint hook failed")]
    PostMintHookFailed {},
}

pub fn handle_reply(reply_id: u64) -> Result<(), MintHookError> {
    match reply_id {
        PREMINT_HOOK_REPLY_ID => return Err(MintHookError::PreMintHookFailed {}),
        POSTMINT_HOOK_REPLY_ID => return Err(MintHookError::PostMintHookFailed {}),
        _ => (),
    };

    Ok(())
}

pub mod pre {
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{to_json_binary, Addr, Binary, Deps, DepsMut, StdResult, WasmMsg};
    use sg_std::{Response, SubMsg};

    use crate::{MintHookError, PREMINT_HOOKS, PREMINT_HOOK_REPLY_ID};

    #[cw_serde]
    pub struct PreMintHookMsg {
        pub collection: String,
        pub token_id: Option<String>,
        pub buyer: String,
    }

    impl PreMintHookMsg {
        pub fn new(collection: String, token_id: Option<String>, buyer: String) -> Self {
            PreMintHookMsg {
                collection,
                token_id,
                buyer,
            }
        }

        /// serializes the message
        pub fn into_json_binary(self) -> StdResult<Binary> {
            let msg = PreMintExecuteHookMsg::PreMintHook(self);
            to_json_binary(&msg)
        }
    }

    // This is just a helper to properly serialize the above message
    #[cw_serde]
    pub enum PreMintExecuteHookMsg {
        PreMintHook(PreMintHookMsg),
    }

    pub fn add_premint_hook(deps: DepsMut, hook: String) -> Result<Response, MintHookError> {
        PREMINT_HOOKS.add_hook(deps.storage, deps.api.addr_validate(&hook)?)?;

        let res = Response::new()
            .add_attribute("action", "add_premint_hook")
            .add_attribute("hook", hook);
        Ok(res)
    }

    pub fn query_premint_hooks(deps: Deps) -> StdResult<Binary> {
        to_json_binary(&PREMINT_HOOKS.query_hooks(deps)?)
    }

    pub fn prepare_premint_hooks(
        deps: Deps,
        collection: Addr,
        token_id: Option<String>,
        buyer: String,
    ) -> StdResult<Vec<SubMsg>> {
        let submsgs = PREMINT_HOOKS.prepare_hooks(deps.storage, |h| {
            let msg = PreMintHookMsg {
                collection: collection.to_string(),
                token_id: token_id.clone(),
                buyer: buyer.clone(),
            };
            let execute = WasmMsg::Execute {
                contract_addr: h.to_string(),
                msg: msg.into_json_binary()?,
                funds: vec![],
            };
            Ok(SubMsg::reply_on_error(execute, PREMINT_HOOK_REPLY_ID))
        })?;

        Ok(submsgs)
    }
}

pub mod post {
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{to_json_binary, Addr, Binary, Deps, DepsMut, StdResult, WasmMsg};
    use sg_std::{Response, SubMsg};

    use crate::{MintHookError, POSTMINT_HOOKS, POSTMINT_HOOK_REPLY_ID};

    #[cw_serde]
    pub struct PostMintHookMsg {
        pub collection: String,
        pub token_id: Option<String>,
        pub buyer: String,
    }

    impl PostMintHookMsg {
        pub fn new(collection: String, token_id: Option<String>, buyer: String) -> Self {
            PostMintHookMsg {
                collection,
                token_id,
                buyer,
            }
        }

        /// serializes the message
        pub fn into_json_binary(self) -> StdResult<Binary> {
            let msg = PostMintExecuteHookMsg::PostMintHook(self);
            to_json_binary(&msg)
        }
    }

    // This is just a helper to properly serialize the above message
    #[cw_serde]
    pub enum PostMintExecuteHookMsg {
        PostMintHook(PostMintHookMsg),
    }

    pub fn add_postmint_hook(deps: DepsMut, hook: String) -> Result<Response, MintHookError> {
        POSTMINT_HOOKS.add_hook(deps.storage, deps.api.addr_validate(&hook)?)?;

        let res = Response::new()
            .add_attribute("action", "add_postmint_hook")
            .add_attribute("hook", hook);
        Ok(res)
    }

    pub fn query_postmint_hooks(deps: Deps) -> StdResult<Binary> {
        to_json_binary(&POSTMINT_HOOKS.query_hooks(deps)?)
    }

    pub fn prepare_postmint_hooks(
        deps: Deps,
        collection: Addr,
        token_id: Option<String>,
        buyer: String,
    ) -> StdResult<Vec<SubMsg>> {
        let submsgs = POSTMINT_HOOKS.prepare_hooks(deps.storage, |h| {
            let msg = PostMintHookMsg {
                collection: collection.to_string(),
                token_id: token_id.clone(),
                buyer: buyer.clone(),
            };
            let execute = WasmMsg::Execute {
                contract_addr: h.to_string(),
                msg: msg.into_json_binary()?,
                funds: vec![],
            };
            Ok(SubMsg::reply_on_error(execute, POSTMINT_HOOK_REPLY_ID))
        })?;

        Ok(submsgs)
    }
}
