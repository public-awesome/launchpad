for d in contracts/collections/*; do
  if [ -d "$d" ]; then
    cd $d
    cargo schema
    rm -rf schema/raw
    cd ../../..
  fi
done
for d in contracts/factories/*; do
  if [ -d "$d" ]; then
    cd $d
    cargo schema
    rm -rf schema/raw
    cd ../../..
  fi
done
for d in contracts/minters/*; do
  if [ -d "$d" ]; then
    cd $d
    cargo schema
    rm -rf schema/raw
    cd ../../..
  fi
done
for d in contracts/whitelists/*; do
  if [ -d "$d" ]; then
    cd $d
    cargo schema
    rm -rf schema/raw
    cd ../../..
  fi
done
cd contracts/sg-eth-airdrop && cargo schema && rm -rf schema/raw && cd ../..
cd contracts/splits && cargo schema && rm -rf schema/raw && cd ../..

cd ts && yarn install && yarn codegen

cd ..
