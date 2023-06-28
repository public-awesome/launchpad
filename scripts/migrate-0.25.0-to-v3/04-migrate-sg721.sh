ADMIN=stars10w5eulj60qp3cfqa0hkmke78qdy2feq6x9xdmd
SG721=stars1tt0uclyhjfaksp7yzk835ffjydkh5y2sfxzq6skcj4r27jf8yj3sxkqwav
# SG721=stars1xf6grh6vtz8hcsz22nny9gl2jpwx4swz68n5lgnkwxp6xw4phzrqwk8nmv
SG721_INCREASE_ROYALTIES_CODE_ID=2608

starsd tx wasm migrate $SG721 $SG721_INCREASE_ROYALTIES_CODE_ID "{}" \
  --gas-prices 0.025ustars --gas 2000000 --gas-adjustment 1.9 \
  --from $ADMIN -y -b block -o json | jq .