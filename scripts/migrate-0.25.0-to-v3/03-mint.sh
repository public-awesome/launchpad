ADMIN=stars1wh3wjjgprxeww4cgqyaw8k75uslzh3sd3s2yfk
KEY=$(starsd keys show $ADMIN | jq -r .name)
# MINTER=stars1wjzvkel5fdxhapu9m0243ddtydz0efcg5v8uq8zxkeeaydedplsq39maj7
MINTER=stars1d9dsqcyhkjcuv7s4k2xt648stdgzhxjpw4xkzl5sqyhlgm2g90ls8nffnu

MSG=$(cat <<EOF
{ "mint": {} }
EOF
)

echo $MSG

starsd tx wasm execute $MINTER "$MSG" --amount 50000000ustars \
  --gas-prices 0.025ustars --gas 500000 --gas-adjustment 1.9 \
  --from $ADMIN -y -b block -o json | jq .