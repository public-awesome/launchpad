ADMIN=stars1wh3wjjgprxeww4cgqyaw8k75uslzh3sd3s2yfk
FACTORY=stars1z89ugky6tpu837v4nwyvsfdaa3f092xlkx4f5p3utd9336pems7qke68gx
SG721_CODE_ID=2045
KEY=$(starsd keys show $ADMIN | jq -r .name)

# set start time 100 seconds from now
TIME=$(date -v+100S +%s)

MSG=$(cat <<EOF
{ "create_minter":
    {
        "init_msg": {
            "base_token_uri": "ipfs://QmVnos4WEq5z2zLwX8CR5EkaHyBUiF2RZYQLDWL4gm5DDU",
            "start_time": "$(echo $TIME)000000000",
            "num_tokens": 100,
            "mint_price": {"amount": "10000000", "denom": "ustars"},
            "per_address_limit": 5
        },
        "collection_params": {
            "code_id": $SG721_CODE_ID,
            "name": "Collection Name",
            "symbol": "COL",
            "info": {
                "creator": "$ADMIN",
                "description": "Stargaze Moons",
                "image": "ipfs://QmPYqcz3p89SNzHnsdHt6JCbXdB7EceLckdVSQGZBqNZeX/1.png",
                "external_link": "https://example.com/external.html",
                "explicit_content": false,
                "royalty_info": {
                    "payment_address": "$ADMIN",
                    "share": "0.1"
                }
            }
        }
    }
}
EOF
)

echo $MSG

starsd tx wasm execute $FACTORY "$MSG" --amount 5000000000ustars \
  --gas-prices 0.025ustars --gas 2000000 --gas-adjustment 1.9 \
  --from $ADMIN -y -b block -o json | jq .