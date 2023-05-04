for d in ./*.wasm; do
    echo $d;
    starsd tx wasm store $d --from test --gas-prices 0.025ustars --gas-adjustment 1.7 --gas auto --chain-id elgafar-1 --node https://rpc.elgafar-1.stargaze-apis.com:443  -b block --yes -o json | jq '.logs' | grep -A 1 code_id
    echo "-----------------";
done

