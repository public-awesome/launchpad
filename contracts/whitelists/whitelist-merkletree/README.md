# Whitelist MerkleTree contract

A whitelist contract that relies on MerkleTree data structure for verifying inclusion of an address in a whitelist. 

Only merkle root (and optionaly URI of a tree) are stored within the state. Inclusion can be verified by submitting a user address and hex-encoded list of merklee proofs. This approach allows significant reduction of gas usage during storage phase with a downside of having actual data off-chain and reliance on 3rd parties for providing inclusiong proofs. 

Inclusion operation is a slighly more complex and costly when compared to the standard map-based whitelist. The contract uses **Sha256** for hashing concatenated proofs. Hashes are sorted on byte level prior to concatenation, which significantly simplifies the verification process by not requiring submission of leaf positions. 

**Important:** Make sure that your algorithm for merkle tree construction also sort the hashes. See example of extending `rs-merkle` library in `tests/hasher.rs`

## Gas Usage

The contracts for the merkletree based whitelist and the updated minter that supports it were both deployed to the testnet to measure actual gas usage in production. The contracts were instantiated and tested with two different whitelist sizes: **703** and **91,750,400** entries

#### Instantiating
Naturally due to only needing to store a merkle tree root in the state of the contract there is no  difference between instantiating a whitelist with the [smaller](https://testnet-explorer.publicawesome.dev/stargaze/tx/07BB768915A24C17C12982D3FE34ADF0453AA9231961197A8B4E5E228D5C6B54) and the [bigger](https://testnet-explorer.publicawesome.dev/stargaze/tx/14E2DFB03AFB2A711A6AF601FA43FAEADFC8D0BA8581DD9E02EEFFB582E8AFB7) list sizes and they both consume 190,350 units of gas.

#### Minting

Number of hashing operations required to check for inclusion of an address in a merkle tree is at most `Math.ceil[ log₂N ]` and in some cases even smaller depending on the depth of a leaf within a tree.

In case of the smaller tree with 704 records we had to submit 8 hash proofs and an example mint [transaction](https://testnet-explorer.publicawesome.dev/stargaze/tx/8692581537939E09BF5D81594B078436D4224F0944B515A421F096CEE480ECA9) took 635,345 units of gas

The bigger tree with ~90 million records [used](https://testnet-explorer.publicawesome.dev/stargaze/tx/670A76A64F0A64FB1A5077DADDB6C326A9A64B66999215345C47BA3F03265811) 647,448 units of gas and required 24 proofs only (up to 27 with deeper leaves).

The jump from computing 8 to computing 24 proofs (+16) only took additional 8 thousands units of gas. Keep in mind that another increase in 16 proofs allow us to check for inclusion in a tree with 1 trillion addresses.
