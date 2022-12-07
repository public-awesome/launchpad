cd packages/sg721 && cargo publish && cd ../..
sleep 5
cd packages/sg-metadata && cargo publish && cd ../..
sleep 5
cd packages/sg4 && cargo publish && cd ../..
sleep 5
cd packages/sg2 && cargo publish && cd ../..
sleep 5
cd packages/sg-std && cargo publish && cd ../..
sleep 5
cd packages/sg-multi-test && cargo publish && cd ../..
sleep 5
cd packages/sg-utils && cargo publish && cd ../..
sleep 5
cd packages/sg1 && cargo publish && cd ../..
sleep 5
cd packages/controllers && cargo publish && cd ../..

