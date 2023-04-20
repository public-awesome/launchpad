#[cfg(test)]
mod tests {
    use cosmwasm_std::Addr;
    use cw_multi_test::Executor;

    use open_edition_factory::helpers::FactoryContract;
    use open_edition_factory::msg::InstantiateMsg;
    use sg_multi_test::StargazeApp;

    use crate::common_setup::contract_boxes::{contract_open_edition_factory, custom_mock_app};
    use crate::common_setup::setup_minter::open_edition_minter::mock_params::mock_params_proper;

    const GOVERNANCE: &str = "governance";

    fn proper_instantiate() -> (StargazeApp, FactoryContract) {
        let mut app = custom_mock_app();
        let factory_id = app.store_code(contract_open_edition_factory());
        let minter_id = 2;

        let mut params = mock_params_proper();
        params.code_id = minter_id;

        let factory_contract_addr = app
            .instantiate_contract(
                factory_id,
                Addr::unchecked(GOVERNANCE),
                &InstantiateMsg { params },
                &[],
                "factory",
                None,
            )
            .unwrap();

        (app, FactoryContract(factory_contract_addr))
    }

    mod init {
        use open_edition_factory::msg::{OpenEditionUpdateParamsExtension, OpenEditionUpdateParamsMsg, ParamsResponse, SudoMsg};

        use super::*;

// We assume that the CreateMinter method is validated at the minter level

        #[test]
        fn can_init() {
            let (_, factory_contract) = proper_instantiate();
            assert_eq!(factory_contract.addr().to_string(), "contract0");
        }

        #[test]
        fn sudo_update_params() {
            let (mut app, factory_contract) = proper_instantiate();
            let query_config_msg = sg2::query::Sg2QueryMsg::Params {};
            let res: ParamsResponse = app
                .wrap()
                .query_wasm_smart(factory_contract.0.to_string(), &query_config_msg)
                .unwrap();
            assert_eq!(res.params.allowed_sg721_code_ids, vec![1, 3, 5, 6]);
            assert!(!res.params.frozen);
            assert_eq!(res.params.mint_fee_bps, 1000);
            assert_eq!(res.params.extension.dev_fee_bps, 200);
            assert_eq!(res.params.extension.token_id_prefix_length, 30);

            let update_msg = OpenEditionUpdateParamsMsg {
                add_sg721_code_ids: Some(vec![12, 24]),
                rm_sg721_code_ids: Some(vec![1]),
                frozen: Some(true),
                code_id: None,
                creation_fee: None,
                min_mint_price: None,
                mint_fee_bps: Some(2000),
                max_trading_offset_secs: None,
                extension: OpenEditionUpdateParamsExtension {
                    token_id_prefix_length: Some(15),
                    abs_max_mint_per_address: None,
                    min_mint_price: None,
                    airdrop_mint_fee_bps: None,
                    airdrop_mint_price: None,
                    dev_fee_bps: None,
                    dev_fee_address: None,
                },
            };
            let sudo_msg = SudoMsg::UpdateParams(Box::new(update_msg));
            let _res = app.wasm_sudo(factory_contract.clone().0, &sudo_msg);
            let res: ParamsResponse = app
                .wrap()
                .query_wasm_smart(factory_contract.0.to_string(), &query_config_msg)
                .unwrap();
            assert_eq!(res.params.allowed_sg721_code_ids, vec![3, 5, 6, 12, 24]);
            assert!(res.params.frozen);
            assert_eq!(res.params.mint_fee_bps, 2000);
            assert_eq!(res.params.extension.dev_fee_bps, 200);
            assert_eq!(res.params.extension.token_id_prefix_length, 15);
        }

    }
}
