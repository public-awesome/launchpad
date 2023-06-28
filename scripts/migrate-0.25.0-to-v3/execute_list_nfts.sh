ADMIN=stars10w5eulj60qp3cfqa0hkmke78qdy2feq6x9xdmd
KEY=$(starsd keys show $ADMIN | jq -r .name)
MARKETPLACE=stars15hfylrz33mf839x80a0vpqdgkjm3gm0xmwpneg3cqu3ap32x7mssrcpfty
# COLLECTION=stars1tt0uclyhjfaksp7yzk835ffjydkh5y2sfxzq6skcj4r27jf8yj3sxkqwav
COLLECTION=stars1xf6grh6vtz8hcsz22nny9gl2jpwx4swz68n5lgnkwxp6xw4phzrqwk8nmv

# approve marketplace contract, no expiration
APPROVE_MSG=$(cat <<EOF 
 { 
    "approve": {
        "spender": "$MARKETPLACE",
        "token_id": "$1"
    }
 }
EOF
)

echo $APPROVE_MSG
starsd tx wasm execute $COLLECTION "$APPROVE_MSG" \
  --gas-prices 0.025ustars --gas 500000 --gas-adjustment 1.9 \
  --from $ADMIN -y -b block -o json | jq .


# list for sale
# ExecuteMsg::SetAsk {
#     sale_type: SaleType::FixedPrice,
#     collection: collection.to_string(),
#     token_id: token_id_0,
#     price: coin(110, NATIVE_DENOM),
#     funds_recipient: None,
#     reserve_for: None,
#     expires: start_time.plus_seconds(MIN_EXPIRY + 1),
#     finders_fee_bps: Some(0),
# }
TIME=$(date -v+10000S +%s)
MSG=$(cat <<EOF
{ "set_ask": {
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
starsd tx wasm execute $MARKETPLACE "$MSG" \
  --gas-prices 0.025ustars --gas 500000 --gas-adjustment 1.9 \
  --from $ADMIN -y -b block -o json | jq .