KEY=$(starsd keys show $ADMIN | jq -r .name)
FACTORY=stars1qwn5rk7lzmr0td07zafnnsqcha0hx3fmnkpfjm65kppnxj4mzcaqqv023l
SG721_CODE_ID=2604

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
            "num_tokens": 100,
            "mint_price": { "amount": "50000000", "denom": "ustars" },
            "per_address_limit": 3
        },
        "collection_params": {
            "code_id": $SG721_CODE_ID,
            "name": "Test Collection yubo2",
            "symbol": "YUBO2",
            "info": {
                "creator": "$ADMIN",
                "description": "Test Collection yubo2",
                "image": "ipfs://bafybeiavall5udkxkdtdm4djezoxrmfc6o5fn2ug3ymrlvibvwmwydgrkm/1.jpg",
                "start_trading_time": "$(echo $TIME)000000000"
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