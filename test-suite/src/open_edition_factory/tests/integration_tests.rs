#[cfg(test)]
mod tests {

    mod init {
        use crate::common_setup::setup_minter::common::constants::MAX_TOKEN_LIMIT;
        use crate::open_edition_factory::tests::common::proper_instantiate;
        use open_edition_factory::msg::{
            OpenEditionUpdateParamsExtension, OpenEditionUpdateParamsMsg, ParamsResponse, SudoMsg,
        };
        // Assumption: CreateMinter method is validated at the minter level

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
            assert_eq!(res.params.extension.max_token_limit, MAX_TOKEN_LIMIT);

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
                    max_token_limit: Some(9_999u32),
                    max_per_address_limit: None,
                    min_mint_price: None,
                    airdrop_mint_fee_bps: None,
                    airdrop_mint_price: None,
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
            assert_eq!(res.params.extension.max_token_limit, 9_999u32);
        }
    }
}
