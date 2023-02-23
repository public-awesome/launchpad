KEY=$(starsd keys show $USER | jq -r .name)

# init msg
# VendingMinterInitMsgExtension {
#     pub base_token_uri: String,
#     pub payment_address: Option<String>,
#     pub start_time: Timestamp,
#     pub num_tokens: u32,
#     pub mint_price: Coin,
#     pub per_address_limit: u32,
#     pub whitelist: Option<String>,
# }

# collection params
# CollectionParams {
#     /// The collection code id
#     pub code_id: u64,
#     pub name: String,
#     pub symbol: String,
#     pub info: CollectionInfo<RoyaltyInfoResponse>,
# }

MSG=$(cat <<EOF
{
    "code_id": $MINTER_CODE_ID,
    "creation_fee": "5000000000ustars",
    "min_mint_price": "50000000ustars",
    "mint_fee_bps": "1000",
    "max_trading_offset_secs": 604800,
    extension: ParamsExtension {
        max_token_limit: 10000,
        max_per_address_limit: 50,
        airdrop_mint_price: 0ustars,
        airdrop_mint_fee_bps: 10000,
        shuffle_fee: 500000000ustars,
    },
}
EOF
)


if [ "$ADMIN_MULTISIG" = true ] ; then
  echo 'Using multisig'
  starsd tx wasm instantiate $MINTER_CODE_ID "$MSG" --label "Minter-Updatable" \
    --admin $ADMIN \
    --gas-prices 0.025ustars --gas 50000000 --gas-adjustment 1.9 \
    --from $ADMIN \
    --generate-only > unsignedTx.json

  starsd tx sign unsignedTx.json \
    --multisig=$ADMIN --from $USER --output-document=$KEY.json \
    --chain-id $CHAIN_ID
else
  echo 'Using single signer'
  starsd tx wasm instantiate $MINTER_CODE_ID "$MSG" --label "Minter-Updatable" \
    --admin $ADMIN \
    --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
    --from $ADMIN -y -b block -o json | jq .
fi