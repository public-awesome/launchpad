for W in ./target/wasm32-unknown-unknown/release/*.wasm
do
    echo -n "Checking `basename $W`... "
    cosmwasm-check --available-capabilities iterator,staking,stargate,stargaze,cosmwasm_1_1,cosmwasm_1_2,cosmwasm_1_3,cosmwasm_1_4 $W
done
