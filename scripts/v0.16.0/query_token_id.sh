SG721=stars1naqvlqkh8yccuwv6a566hdt4flrrz5933lhva0jmsp2kgjhuh3zqf6avr4

MSG=$(cat <<EOF
{
  "nft_info": { "token_id": "$1" }
}
EOF
)
echo $MSG

starsd q wasm contract-state smart $SG721 "$MSG"
