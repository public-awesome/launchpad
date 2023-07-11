KEY=$(starsd keys show $ADMIN | jq -r .name)
FACTORY=stars1dfqgqmvw077capmmwcynqrrarrxqje2a48gn8f4f0jn0ntgrc3kqxeaj9k
SG721_CODE_ID=2604
# SG721_INCREASE_ROYALTIES_CODE_ID=2628

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
# CollectionInfo {
    # pub creator: String,
    # pub description: String,
    # pub image: String,
    # pub external_link: Option<String>,
    # pub explicit_content: Option<bool>,
    # pub start_trading_time: Option<Timestamp>,
    # pub royalty_info: Option<T>,
# }

# add a few minutes buffer to start time
TIME=$(date -v+30S +%s)

MSG=$(cat <<EOF
{
    "create_minter": {
        "init_msg": {
            "base_token_uri": "ipfs://bafybeiey2heysue3px2tgc523cmjbfjlox5zfzzan5syzdooikdvimtxwq",
            "start_time": "$(echo $TIME)000000000",
            "num_tokens": 2000,
            "mint_price": { "amount": "0", "denom": "ustars" },
            "per_address_limit": 30
        },
        "collection_params": {
            "code_id": $SG721_CODE_ID,
            "name": "Test Collection yubo 2",
            "symbol": "YUBO1",
            "info": {
                "creator": "$ADMIN",
                "description": "Test Collection yubo 2",
                "image": "ipfs://bafybeiavall5udkxkdtdm4djezoxrmfc6o5fn2ug3ymrlvibvwmwydgrkm/1.jpg",
                "start_trading_time": "$(echo $TIME)000000000",
                "royalty_info": {
                    "payment_address": "$ADMIN",
                    "share": "0.05"

                }
            }
        }
    }
}
EOF
)

echo $MSG

starsd tx wasm execute $FACTORY "$MSG" --amount 1000000000ustars \
--gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
--from $KEY -y -b block -o json | jq .