# add new sg721 code id to factory
ADMIN=stars10w5eulj60qp3cfqa0hkmke78qdy2feq6x9xdmd
FACTORY=stars1dfqgqmvw077capmmwcynqrrarrxqje2a48gn8f4f0jn0ntgrc3kqxeaj9k
SG721_BASE_INCREASE_ROYALTIES_CODE_ID=2674

# update params msg
MSG=$(cat <<EOF
{
  "update_minter_params": {
    "add_sg721_code_ids": [$SG721_BASE_INCREASE_ROYALTIES_CODE_ID]
  }
}
EOF
)

# must submit gov proposal to change SUDO_PARAMS
starsd tx gov submit-proposal execute-contract $FACTORY "$MSG" \
  --title "Add SG721 code id to factory" \
  --description "Add SG721 code id to factory" \
  --amount 10001000000ustars \
  --gas-prices 0.025ustars --gas 500000 --gas-adjustment 1.9 \
  --from $ADMIN --run-as $ADMIN -y -b block -o json | jq .