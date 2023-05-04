ADMIN=stars1wh3wjjgprxeww4cgqyaw8k75uslzh3sd3s2yfk
SG721=stars1naqvlqkh8yccuwv6a566hdt4flrrz5933lhva0jmsp2kgjhuh3zqf6avr4

MSG=$(cat <<EOF
{
	"update_token_metadata": {
		"token_id": "98",
		"token_uri": "ipfs://bafybeiaf2qzkva4tnxak4k5trnnyzuinzzoxrookm7t4wa753rdarsoetm/metadata/1"
	}
}
EOF
)

echo $MSG

starsd tx wasm execute $SG721 "$MSG" \
  --gas-prices 0.025ustars --gas 2000000 --gas-adjustment 1.9 \
  --from $ADMIN -y -b block -o json | jq .