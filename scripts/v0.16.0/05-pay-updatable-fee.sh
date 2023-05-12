ADMIN=stars1wh3wjjgprxeww4cgqyaw8k75uslzh3sd3s2yfk
SG721=stars1naqvlqkh8yccuwv6a566hdt4flrrz5933lhva0jmsp2kgjhuh3zqf6avr4

# set start time 100 seconds from now
TIME=$(date -v+100S +%s)

MSG=$(cat <<EOF
{
	"enable_updatable": {}
}
EOF
)

echo $MSG

starsd tx wasm execute $SG721 "$MSG" --amount 500000000ustars \
  --gas-prices 0.025ustars --gas 2000000 --gas-adjustment 1.9 \
  --from $ADMIN -y -b block -o json | jq .