use cosmwasm_std::StdResult;
use url::Url;

pub fn verify_tree_uri(tree_uri: &Option<String>) -> StdResult<()> {
    if tree_uri.is_some() {
        let res = Url::parse(tree_uri.as_ref().unwrap());
        if res.is_err() {
            return Err(cosmwasm_std::StdError::GenericErr {
                msg: "Invalid tree uri".to_string(),
            });
        }
    }
    Ok(())
}
