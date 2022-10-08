import { MerkleTree } from "merkletreejs";
import SHA256 from "crypto-js/sha256";

import whitelist from "./testdata/whitelist.json";

const leaves = whitelist.map((x) => SHA256(x));
const tree = new MerkleTree(leaves, SHA256);
const root = tree.getRoot().toString("hex");
console.log({ root });

console.log({ whitelist });
const proofs = whitelist.map((x) => tree.getProof(SHA256(x)));

for (const address of whitelist) {
  console.log("Proof:", address);
  const proof = tree.getProof(SHA256(address));
  console.log(proof.map((x) => x.data.toString("hex")));
}
// console.log(proofs[0].map((x) => x.data.toString("hex")));

// console.log(tree.verify(proofs[0], leaves[0], root)); // true
// console.log(tree.verify(proofs[1], leaves[1], root)); // true
// console.log(tree.verify(proofs[2], leaves[2], root)); // true
// console.log(
//   tree.verify(proofs[2], "stars16epdu6c7h8apxrnuu06yzfxflrede0mtu4qqz4", root)
// ); // false

// console.log(
//   tree.getProof(SHA256("stars16epdu6c7h8apxrnuu06yzfxflrede0mtu4qqz4"))
// );
