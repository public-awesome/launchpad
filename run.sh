INSTANTIATE='
{
    "members":["stars1paqkeyluuw47pflgwwqaaj8y679zj96aatg5a7"],
    "start_time": "1647061823000000000",
    "end_time": "1647148223000000000",
    "unit_price" : {"amount":"25000000", "denom":"ustars"},
    "per_address_limit": 2,
    "member_limit": 500
}'
echo $INSTANTIATE
starsd tx wasm instantiate 1 "$INSTANTIATE" --from validator --label "my whitelist"  --chain-id localnet-1 --admin stars1paqkeyluuw47pflgwwqaaj8y679zj96aatg5a7 -b block --amount 100000000ustars --gas auto --gas-adjustment 1.5
starsd q wasm  contract-state smart stars14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9srsl6sm "{\"members\":{}}"