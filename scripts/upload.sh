echo "serial-print-factory";
starsd tx wasm store artifacts/serial_print_factory.wasm --from test --gas-prices 0.025ustars --gas-adjustment 1.7 --gas auto --chain-id elgafar-1 --node https://rpc.elgafar-1.stargaze-apis.com:443  -b block --yes -o json | jq '.logs' | grep -A 1 code_id
echo "sg721-base";
starsd tx wasm store artifacts/sg721_base.wasm --from test --gas-prices 0.025ustars --gas-adjustment 1.7 --gas auto --chain-id elgafar-1 --node https://rpc.elgafar-1.stargaze-apis.com:443 -b block --yes -o json | jq '.logs' | grep -A 1 code_id
echo "serial-print-minter";
starsd tx wasm store artifacts/serial_print_minter.wasm --from test --gas-prices 0.025ustars --gas-adjustment 1.7 --gas auto --chain-id elgafar-1 --node https://rpc.elgafar-1.stargaze-apis.com:443   -b block --yes -o json | jq '.logs' | grep -A 1 code_id
