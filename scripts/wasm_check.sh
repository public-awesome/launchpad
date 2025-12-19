for W in ./target/wasm32-unknown-unknown/release/*.wasm
do
    echo -n "Checking `basename $W`... "
    cosmwasm-check --available-capabilities iterator,staking,stargate,cosmwasm_1_1,cosmwasm_1_2,cosmwasm_1_3,cosmwasm_1_4,cosmwasm_2_0,cosmwasm_2_1,cosmwasm_2_2 $W
done
