MSG=$(cat <<EOF
{
  "nft_info": {"token_id": "$1"}
}
EOF
)
echo $MSG $SG721

starsd q wasm contract-state smart $SG721 "$MSG"

