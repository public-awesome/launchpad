ADMIN=stars10w5eulj60qp3cfqa0hkmke78qdy2feq6x9xdmd
# ADMIN=stars1wh3wjjgprxeww4cgqyaw8k75uslzh3sd3s2yfk
KEY=$(starsd keys show $ADMIN | jq -r .name)
# MINTER=stars1ms5z58yrp757sq498dsw35zzq5w9t6zq56jz9xvc8lv488e0e7dsze6eln
MINTER=stars1d4wn70v5kjtewrqu8tv3cc6hava3zxalwxnvclyqed09lvnahw4sf9cj2r

MSG=$(cat <<EOF
{ "mint": {} }
EOF
)

echo $MSG

starsd tx wasm execute $MINTER "$MSG" \
  --gas-prices 0.025ustars --gas 500000 --gas-adjustment 1.9 \
  --from $ADMIN -y -b block -o json | jq -r '.txhash, (.logs[0].events | map(select(.type=="wasm"))[].attributes | map(select(.key=="token_id"))[].value)'