# ex:
# "update_token_metadata": {
#     "token_id": "$1",
#     "token_uri": "ipfs://bafybeiey2heysue3px2tgc523cmjbfjlox5zfzzan5syzdooikdvimtxwq/407"
# }

KEY=$(starsd keys show $ADMIN | jq -r .name)
MSG=$(cat <<EOF
{
    "update_token_metadata": {
        "token_id": "$1",
        "token_uri": "$2"
    }
}
EOF
)

echo $MSG

starsd tx wasm execute $SG721 "$MSG" \
--gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
--from $KEY -y -b block -o json | jq .