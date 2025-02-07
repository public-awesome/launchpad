#[cfg(test)]
mod tests {
    use cosmwasm_std::Addr;
    use cw_multi_test::Executor;

    use open_edition_factory::helpers::FactoryContract;
    use open_edition_factory::msg::InstantiateMsg;
    use open_edition_factory::state::OpenEditionMinterParams;

    use crate::common_setup::contract_boxes::{
        contract_open_edition_factory, custom_mock_app, App,
    };
    use crate::common_setup::setup_minter::open_edition_minter::mock_params::mock_params_proper;

    const GOVERNANCE: &str = "governance";

    fn proper_instantiate() -> (App, FactoryContract, OpenEditionMinterParams) {
        let mut app = custom_mock_app();
        let factory_id = app.store_code(contract_open_edition_factory());
        let minter_id = 2;

        let mut params = mock_params_proper();
        params.code_id = minter_id;

        let factory_contract_addr = app
            .instantiate_contract(
                factory_id,
                Addr::unchecked(GOVERNANCE),
                &InstantiateMsg {
                    params: params.clone(),
                },
                &[],
                "factory",
                None,
            )
            .unwrap();

        (app, FactoryContract(factory_contract_addr), params)
    }

    mod init {
        use open_edition_factory::msg::ParamsResponse;
        use sg2::query::{AllowedCollectionCodeIdResponse, AllowedCollectionCodeIdsResponse};

        use super::*;

        #[test]
        fn query_params() {
            let (app, factory_contract, params) = proper_instantiate();
            let query_config_msg = sg2::query::Sg2QueryMsg::Params {};
            let res: ParamsResponse = app
                .wrap()
                .query_wasm_smart(factory_contract.0.to_string(), &query_config_msg)
                .unwrap();
            assert_eq!(
                res.params.allowed_sg721_code_ids,
                params.allowed_sg721_code_ids
            );
            assert_eq!(res.params.creation_fee, params.creation_fee);
            assert_eq!(res.params.code_id, params.code_id);
            assert_eq!(res.params.frozen, params.frozen);
            assert_eq!(res.params.mint_fee_bps, params.mint_fee_bps);
            assert_eq!(res.params.min_mint_price, params.min_mint_price);
            assert_eq!(
                res.params.max_trading_offset_secs,
                params.max_trading_offset_secs
            );
            assert_eq!(
                res.params.extension.max_per_address_limit,
                params.extension.max_per_address_limit
            );
            assert_eq!(
                res.params.extension.airdrop_mint_price,
                params.extension.airdrop_mint_price
            );
            assert_eq!(
                res.params.extension.airdrop_mint_fee_bps,
                params.extension.airdrop_mint_fee_bps
            );
            assert_eq!(
                res.params.extension.dev_fee_address,
                params.extension.dev_fee_address
            );
            assert_eq!(
                res.params.extension.max_token_limit,
                params.extension.max_token_limit
            );
        }

        #[test]
        fn query_allowed_collection_code_ids_test() {
            let (app, factory_contract, params) = proper_instantiate();
            let query_config_msg = sg2::query::Sg2QueryMsg::AllowedCollectionCodeIds {};
            let res: AllowedCollectionCodeIdsResponse = app
                .wrap()
                .query_wasm_smart(factory_contract.0.to_string(), &query_config_msg)
                .unwrap();
            assert_eq!(res.code_ids, params.allowed_sg721_code_ids);
        }

        #[test]
        fn query_allowed_collection_code_id_test() {
            let (app, factory_contract, params) = proper_instantiate();
            // Valid code id
            assert!(params.allowed_sg721_code_ids.contains(&1u64));
            let query_config_msg = sg2::query::Sg2QueryMsg::AllowedCollectionCodeId(1);
            let res: AllowedCollectionCodeIdResponse = app
                .wrap()
                .query_wasm_smart(factory_contract.0.to_string(), &query_config_msg)
                .unwrap();
            assert!(res.allowed);

            // Invalid code id
            assert!(!params.allowed_sg721_code_ids.contains(&11u64));
            let query_config_msg = sg2::query::Sg2QueryMsg::AllowedCollectionCodeId(11);
            let res: AllowedCollectionCodeIdResponse = app
                .wrap()
                .query_wasm_smart(factory_contract.0.to_string(), &query_config_msg)
                .unwrap();
            assert!(!res.allowed);
        }
    }
}
