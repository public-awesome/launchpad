import codegen from "@cosmwasm/ts-codegen";

codegen({
  contracts: [
    {
      name: "BaseFactory",
      dir: "../contracts/factories/base-factory/schema",
    },
    {
      name: "BaseMinter",
      dir: "../contracts/minters/base-minter/schema",
    },
    {
      name: "Sg721Base",
      dir: "../contracts/collections/sg721-base/schema",
    },
    {
      name: "Sg721MetadataOnchain",
      dir: "../contracts/collections/sg721-metadata-onchain/schema",
    },
    {
      name: "Sg721Updatable",
      dir: "../contracts/collections/sg721-updatable/schema",
    },
    {
      name: "Sg721Nt",
      dir: "../contracts/collections/sg721-nt/schema",
    },
    {
      name: "Splits",
      dir: "../contracts/splits/schema",
    },
    {
      name: "VendingFactory",
      dir: "../contracts/factories/vending-factory/schema",
    },
    {
      name: "VendingMinter",
      dir: "../contracts/minters/vending-minter/schema",
    },
    {
      name: "VendingMinterWlFlex",
      dir: "../contracts/minters/vending-minter-wl-flex/schema",
    },
    {
      name: "WhitelistFlex",
      dir: "../contracts/whitelists/whitelist-flex/schema",
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
      enabled: false,
      optionalClient: true,
      version: "v4",
      mutations: true,
      queryKeys: true,
    },
    recoil: {
      enabled: false,
    },
    messageComposer: {
      enabled: true,
    },
  },
}).then(() => {
  console.log("✨ all done!");
});
