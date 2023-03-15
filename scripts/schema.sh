for d in contracts/*; do
  if [ -d "$d" ]; then
    cd $d
    cargo schema
    rm -rf schema/raw
    cd ../..
  fi
done

<<<<<<< HEAD
cd ts && yarn codegen
=======
cd ts && yarn install && yarn codegen
>>>>>>> main
