# Stargaze Mint Hooks

Mint hooks are a way to run custom code before and after minting. Creators may write custom hooks to perform tasks such as implementing burn-to-mint.

For example, a pre-mint hook could receive an NFT and check if its in the collection to be burned. Then after the mint is complete, the post-mint hook could do the actual burn operation.

The pre-mint action, mint, and post-mint actions are sub-messages that are implemented as a single atomic action. They are executed in order, and all rollback if one of them fails.

## How to add mint hooks to a minter contract

### Add macros for execute and query enums

```rs
#[sg_mint_hooks_execute]
#[cw_serde]
pub enum ExecuteMsg {}

#[sg_mint_hooks_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
```

### Add the pre and post submessages to mint execution

```rs
    let premint_hooks = prepare_premint_hooks(
        deps.as_ref(),
        collection.clone(),
        Some(token_id.clone()),
        info.sender.to_string(),
    )?;

    let postmint_hooks = prepare_postmint_hooks(
        deps.as_ref(),
        collection.clone(),
        Some(token_id.clone()),
        info.sender.to_string(),
    )?;

    let mint = WasmMsg::Execute { ... };

    Response::new()
        .add_submessages(premint_hooks)
        .add_submessage(SubMsg::reply_on_error(mint, MINT_REPLY_ID));
        .add_submessages(postmint_hooks);
```

### Handle the reply errors

```rs
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    handle_reply(msg.id)?;

    match msg.id {
        MINT_REPLY_ID => Err(ContractError::MintFailed {}),
    }

}
```
