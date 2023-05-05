KEY=$(starsd keys show $ADMIN | jq -r .name)
MINTER=stars10az3y4tp98llj43elvtkq38w9h3xsx3s36gm064mffuqqj9d7mfsjvzv8a
WHITELIST=stars1zjq0hazrn32jx9vruqk9efea49c5uz30fdsyaqj2rqz04gc0q7asx08n4t

# add a few minutes buffer to start time
TIME=$(date -v+5000S +%s)

MSG=$(cat <<EOF
{ "set_whitelist": { "whitelist": "$WHITELIST" } }
EOF
)

echo $MSG

starsd tx wasm execute $MINTER "$MSG" \
--gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 \
--from $KEY -y -b block -o json | jq .