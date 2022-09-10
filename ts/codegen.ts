import codegen from "@cosmwasm/ts-codegen";

codegen({
  contracts: [
    {
      name: "BaseFactory",
      dir: "../contracts/base-factory/schema",
    },
    {
      name: "BaseMinter",
      dir: "../contracts/base-minter/schema",
    },
    {
      name: "Sg721Base",
      dir: "../contracts/sg721-base/schema",
    },
    {
      name: "Sg721MetadataOnchain",
      dir: "../contracts/sg721-metadata-onchain/schema",
    },
    {
      name: "Sg721Nt",
      dir: "../contracts/sg721-nt/schema",
    },
    {
      name: "Splits",
      dir: "../contracts/splits/schema",
    },
    {
      name: "VendingFactory",
      dir: "../contracts/vending-factory/schema",
    },
    {
      name: "VendingMinter",
      dir: "../contracts/vending-minter/schema",
    },
  ],
  outPath: "./src/",

  // options are completely optional ;)
  options: {
    bundle: {
      bundleFile: "index.ts",
      scope: "contracts",
    },
    types: {
      enabled: true,
    },
    client: {
      enabled: true,
    },
    reactQuery: {
      enabled: true,
      optionalClient: true,
      version: "v4",
      mutations: true,
      queryKeys: true,
    },
    recoil: {
      enabled: false,
    },
    messageComposer: {
      enabled: false,
    },
  },
}).then(() => {
  console.log("✨ all done!");
});
