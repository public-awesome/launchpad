cd contracts/factories/base-factory && cargo publish && cd ../../..
sleep 10
cd contracts/collections/sg721-base && cargo publish && cd ../../..
sleep 15
cd contracts/collections/sg721-metadata-onchain && cargo publish && cd ../../..
sleep 10
cd contracts/collections/sg721-nt && cargo publish && cd ../../..
sleep 10
cd contracts/splits && cargo publish && cd ../..
sleep 10
cd contracts/minters/base-minter && cargo publish && cd ../../..
sleep 10
cd contracts/whitelist && cargo publish && cd ../..
sleep 10
cd contracts/factories/vending-factory && cargo publish && cd ../../..
sleep 15
cd contracts/minters/vending-minter && cargo publish && cd ../../..
sleep 10
cd contracts/whitelists/whitelist-immutable && cargo publish && cd ../../..
sleep 15
cd contracts/sg-eth-airdrop && cargo publish && cd ../..
sleep 15
cd test-suite && cargo publish && cd ..
