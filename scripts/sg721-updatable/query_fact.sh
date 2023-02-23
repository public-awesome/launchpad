FACTORY=stars1csq2m3gpca9syyq386v6rsfq5r3cp8llee9eyx5uj4wcmxcmg98sqx5xzg

MSG=$(cat <<EOF
{
  "params": {}
}
EOF
)

starsd q wasm contract-state smart $FACTORY "$MSG"