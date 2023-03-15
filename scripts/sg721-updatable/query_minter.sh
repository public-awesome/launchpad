MSG=$(cat <<EOF
{
  "config": {}
}
EOF
)

starsd q wasm contract-state smart $MINTER "$MSG"

MSG=$(cat <<EOF
{
  "start_time": {}
}
EOF
)

starsd q wasm contract-state smart $MINTER "$MSG"