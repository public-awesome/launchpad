# Stargaze Splits

A contract that distributes funds to a cw4-group based on their weights.

## How to create a split

Step 1: Instantiate a group contract with members and weights

Weights should be sufficiently large to allow for more granularity in payment distribution.

Step 2: Instantiate a splits contract with the group contract.

## How to use splits for Launchpad payments

Use the split contract address for the payment address.

## How to use splits for Marketplace payments (royalties)

Use the split contract for the royalty payment address.
