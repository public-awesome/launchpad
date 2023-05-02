ADMIN=stars1wh3wjjgprxeww4cgqyaw8k75uslzh3sd3s2yfk
MINTER=stars1hvfglvxy6ydh8lxsl5su0zmzw0y5rl6p2urvvps925m5mpq3rgvslzm0nv

MSG=$(cat <<EOF
{ "mint": {} }
EOF
)

echo $MSG

starsd tx wasm execute $MINTER "$MSG" --amount 10000000ustars \
  --gas-prices 0.025ustars --gas 500000 --gas-adjustment 1.9 \
  --from $ADMIN -y -b block -o json | jq .