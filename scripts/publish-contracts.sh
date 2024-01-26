cd contracts/factories/base-factory && cargo publish && cd ../../..
sleep 10
cd contracts/collections/sg721-base && cargo publish && cd ../../..
sleep 15
cd contracts/collections/sg721-metadata-onchain && cargo publish && cd ../../..
sleep 10
cd contracts/collections/sg721-nt && cargo publish && cd ../../..
sleep 10
cd contracts/collections/sg721-updatable && cargo publish && cd ../../..
sleep 10
cd contracts/splits && cargo publish && cd ../..
sleep 10
cd contracts/minters/base-minter && cargo publish && cd ../../..
sleep 10
cd contracts/whitelists/whitelist && cargo publish && cd ../../..
sleep 10
cd contracts/factories/vending-factory && cargo publish && cd ../../..
sleep 15
cd contracts/minters/vending-minter && cargo publish && cd ../../..
sleep 15
cd contracts/factories/open-edition-factory && cargo publish && cd ../../..
sleep 15
cd contracts/minters/open-edition-minter && cargo publish && cd ../../..
sleep 10
cd contracts/whitelists/whitelist-immutable && cargo publish && cd ../../..
sleep 15
cd contracts/whitelists/whitelist-flex && cargo publish && cd ../../..
sleep 15
cd contracts/whitelists/whitelist-merkletree && cargo publish && cd ../../..
sleep 15
cd contracts/minters/vending-minter-wl-flex && cargo publish && cd ../../..
sleep 15
cd contracts/minters/vending-minter-merkle-wl && cargo publish && cd ../../..

cd contracts/sg-eth-airdrop && cargo publish && cd ../..
sleep 15
cd test-suite && cargo publish && cd ..
