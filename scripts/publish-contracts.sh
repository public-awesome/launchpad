cd contracts/base-factory && cargo publish && cd ../..
sleep 10
cd contracts/sg721-base && cargo publish && cd ../..
sleep 10
cd contracts/sg721-metadata-onchain && cargo publish && cd ../..
sleep 10
cd contracts/sg721-nt && cargo publish && cd ../..
sleep 10
cd contracts/splits && cargo publish && cd ../..
sleep 10
cd contracts/base-minter && cargo publish && cd ../..
sleep 10
cd contracts/whitelist && cargo publish && cd ../..
sleep 10
cd contracts/vending-factory && cargo publish && cd ../..
sleep 10
cd contracts/vending-minter && cargo publish && cd ../..
sleep 10
cd contracts/whitelist-immutable && cargo publish && cd ../..
