starsd config node $NODE
starsd config chain-id $CHAIN_ID
starsd config output json

KEY=$(starsd keys show $ADMIN | jq -r .name)

# minter params
# MinterParams<T> {
#     /// The minter code id
#     pub code_id: u64,
#     pub creation_fee: Coin,
#     pub min_mint_price: Coin,
#     pub mint_fee_bps: u64,
#     pub max_trading_offset_secs: u64,
#     pub extension: {
        # pub max_token_limit: u32,
        # pub max_per_address_limit: u32,
        # pub airdrop_mint_price: Coin,
        # pub airdrop_mint_fee_bps: u64,
        # pub shuffle_fee: Coin,
# }
MSG=$(cat <<EOF
{
    "params": {
        "code_id": $MINTER_CODE_ID,
        "creation_fee": "5000000000ustars",
        "min_mint_price": "50000000ustars",
        "mint_fee_bps": 1000,
        "max_trading_offset_secs": 604800,
        "extension": {
            "max_token_limit": 10000,
            "max_per_address_limit": 50,
            "airdrop_mint_price": "0ustars",
            "airdrop_mint_fee_bps": 10000,
            "shuffle_fee": "500000000ustars"
        }
    }
	
}
EOF
)


# starsd tx wasm instantiate 1653 "$MSG" --label "Factory" --admin stars10w5eulj60qp3cfqa0hkmke78qdy2feq6x9xdmd --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 --from stars-dev -y -b block -o json | jq .

starsd tx wasm instantiate $FACTORY_CODE_ID "$MSG" --label "Factory" \
  --admin $ADMIN \
  --gas-prices 0.025ustars --gas 50000000 --gas-adjustment 1.9 \
  --from $KEY -y -b block -o json | jq .