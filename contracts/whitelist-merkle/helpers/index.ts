import fs from "fs";
import path from "path";

import { MerkleTree } from "merkletreejs";
import CryptoJS from "crypto-js";

import whitelist from "./testdata/whitelist.json";

const leaves = whitelist.map((x) => CryptoJS.SHA256(x));
const tree = new MerkleTree(leaves, CryptoJS.SHA256, { sort: true });
const root = tree.getRoot().toString("hex");

const data = whitelist.map((address) => {
  const proof = tree.getProof(CryptoJS.SHA256(address));
  return {
    address,
    proof: proof.map((x) => x.data.toString("hex")),
  };
});

fs.writeFileSync(
  path.join(__dirname, "testdata", "proofs.json"),
  JSON.stringify(
    {
      root,
      data,
    },
    null,
    2
  )
);
