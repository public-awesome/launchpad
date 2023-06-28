BUYER=stars1u5kav800kkkrzyvad67zhdmn4xajg6t5j7jm7k
MARKETPLACE=stars15hfylrz33mf839x80a0vpqdgkjm3gm0xmwpneg3cqu3ap32x7mssrcpfty
COLLECTION=stars1a98k05tfna7nxvpp0uench5rp2vvu7tu6h3jfvraksltjcunh86q6cn8xh

# ExecuteMsg::SetBid {
#     sale_type: SaleType::FixedPrice,
#     collection: collection.to_string(),
#     token_id,
#     finders_fee_bps: None,
#     expires: start_time.plus_seconds(MIN_EXPIRY + 1),
#     finder: None,
# }

TIME=$(date -v+10000S +%s)
MSG=$(cat <<EOF
{ "set_bid": {
    "sale_type": "fixed_price",
    "collection": "$COLLECTION",
    "token_id": $1,
    "price": {"amount": "111000000", "denom": "ustars"},
    "expires": "$(echo $TIME)000000000"
    }
}
EOF
)

echo $MSG
starsd tx wasm execute $MARKETPLACE "$MSG" --amount 111000000ustars \
  --gas-prices 0.025ustars --gas 500000 --gas-adjustment 1.9 \
  --from $BUYER -y -b block -o json | jq .