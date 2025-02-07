for C in ./contracts/collections/*/
do
    echo "Compiling `basename $C`..."
    (cd $C && cargo build --release --lib --target wasm32-unknown-unknown --locked)
done
for C in ./contracts/factories/*/
do
    echo "Compiling `basename $C`..."
    (cd $C && cargo build --release --lib --target wasm32-unknown-unknown --locked)
done
for C in ./contracts/minters/*/
do
    echo "Compiling `basename $C`..."
    (cd $C && cargo build --release --lib --target wasm32-unknown-unknown --locked)
done
for C in ./contracts/sg-eth-airdrop/
do
    echo "Compiling `basename $C`..."
    (cd $C && cargo build --release --lib --target wasm32-unknown-unknown --locked)
done
for C in ./contracts/splits/
do
    echo "Compiling `basename $C`..."
    (cd $C && cargo build --release --lib --target wasm32-unknown-unknown --locked)
done
for C in ./contracts/whitelists/*/
do
    echo "Compiling `basename $C`..."
    (cd $C && cargo build --release --lib --target wasm32-unknown-unknown --locked)
done
