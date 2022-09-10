import codegen from "@cosmwasm/ts-codegen";

codegen({
  contracts: [
    {
      name: "Splits",
      dir: "../contracts/splits/schema",
    },
    {
      name: "BaseFactory",
      dir: "../contracts/base-factory/schema",
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
