MSG=$(cat <<EOF
{
  "tokens": {"owner": "$USER"}
}
EOF
)

starsd q wasm contract-state smart $COLLECTION "$MSG"
 
