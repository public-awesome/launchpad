use assert_matches::assert_matches;
use cosm_orc::orchestrator::error::CosmwasmError::TxError;
use cosm_orc::orchestrator::error::ProcessError;
use sg721::{CollectionInfo, InstantiateMsg};
use test_context::test_context;

use crate::helpers::{chain::Chain, helper::SG721_NAME};

#[test_context(Chain)]
#[test]
#[ignore]
fn test_unauthorized_sg721_instantiation(chain: &mut Chain) {
    let user = chain.cfg.users[0].clone();
    let user_addr = &user.account.address;

    let res = chain.orc.instantiate(
        SG721_NAME,
        "sg721_inst_err",
        &InstantiateMsg {
            name: "Collection Name".to_string(),
            symbol: "COL".to_string(),
            minter: user_addr.to_string(),
            collection_info: CollectionInfo {
                creator: user_addr.to_string(),
                description: "Description".to_string(),
                image: "https://example.com/image.png".to_string(),
                external_link: Some("https://github.com/public-awesome".to_string()),
                royalty_info: None,
                explicit_content: None,
                start_trading_time: None,
            },
        },
        &user.key,
        Some(user_addr.parse().unwrap()),
        vec![],
    );

    let err = res.unwrap_err();
    assert_matches!(err, ProcessError::CosmwasmError(TxError(..)));
    assert!(err
        .to_string()
        .contains("Unauthorized: instantiate wasm contract failed"));
}
