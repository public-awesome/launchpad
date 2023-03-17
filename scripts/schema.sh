for d in contracts/*; do
  if [ -d "$d" ]; then
    cd $d
    cargo schema
    rm -rf schema/raw
    cd ../..
  fi
done

cd ts && yarn install && yarn codegen

npm version $1

npm publish --access public

cd ..
