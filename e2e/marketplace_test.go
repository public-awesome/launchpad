package e2e_test

import (
	"fmt"
	"io"
	"io/ioutil"
	"net/http"
	"os"
	"testing"
	"time"

	wasmkeeper "github.com/CosmWasm/wasmd/x/wasm/keeper"
	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/public-awesome/stargaze/v4/testutil/simapp"
	"github.com/stretchr/testify/require"
	"github.com/tendermint/tendermint/crypto/secp256k1"
	tmproto "github.com/tendermint/tendermint/proto/tendermint/types"
)

var (
	instantiateSG721Template = `
		{
			"name": "%s",
			"symbol": "%s",
			"minter": "%s",
			"collection_info": {
				"creator": "%s",
				"description": "Description",
				"image": "https://example.com/image.png"
			}
		}
		`

	executeAskTemplate = `
		{
			"set_ask": {
				"collection": "%s",
				"token_id": %d,
				"price": {
					"amount": "%d",
					"denom": "ustars"
				},
				"expires": "%d"	
			}
		}
		`
	executeMintTemplate = `
		{
			"mint": {
				"token_id": "%d",
				"owner": "%s",
				"extension": {}
			}
		}
		`

	executeApproveTemplate = `
		{
			"approve": {
				"spender": "%s",
				"token_id": "%d",
				"expires": null
			}
		}
		`
)

func TestMarketplace(t *testing.T) {
	accs := GetAccounts()

	genAccs, balances := GetAccountsAndBalances(accs)

	app := simapp.SetupWithGenesisAccounts(t, t.TempDir(), genAccs, balances...)

	startDateTime, err := time.Parse(time.RFC3339Nano, "2022-03-11T20:59:00Z")
	require.NoError(t, err)
	ctx := app.BaseApp.NewContext(false, tmproto.Header{Height: 1, ChainID: "stargaze-1", Time: startDateTime})

	// wasm params
	wasmParams := app.WasmKeeper.GetParams(ctx)
	wasmParams.CodeUploadAccess = wasmtypes.AllowEverybody
	wasmParams.MaxWasmCodeSize = 1000 * 1024 * 4 // 4MB
	app.WasmKeeper.SetParams(ctx, wasmParams)

	priv1 := secp256k1.GenPrivKey()
	pub1 := priv1.PubKey()
	addr1 := sdk.AccAddress(pub1.Address())

	// sg721
	b, err := ioutil.ReadFile("contracts/sg721.wasm")
	require.NoError(t, err)

	msgServer := wasmkeeper.NewMsgServerImpl(wasmkeeper.NewDefaultPermissionKeeper(app.WasmKeeper))
	res, err := msgServer.StoreCode(sdk.WrapSDKContext(ctx), &wasmtypes.MsgStoreCode{
		Sender:       addr1.String(),
		WASMByteCode: b,
	})
	require.NoError(t, err)
	require.NotNil(t, res)
	require.Equal(t, res.CodeID, uint64(1))

	creator := accs[0]

	instantiateMsgRaw := []byte(
		fmt.Sprintf(instantiateSG721Template,
			"Collection Name",
			"COL",
			creator.Address.String(),
			creator.Address.String(),
		),
	)
	instantiateRes, err := msgServer.InstantiateContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgInstantiateContract{
		Sender: creator.Address.String(),
		Admin:  creator.Address.String(),
		CodeID: 1,
		Label:  "SG721",
		Msg:    instantiateMsgRaw,
		Funds:  sdk.NewCoins(sdk.NewInt64Coin("ustars", 1_000_000_000)),
	})
	require.NoError(t, err)
	require.NotNil(t, instantiateRes)
	require.NotEmpty(t, instantiateRes.Address)
	collectionAddress := instantiateRes.Address

	// mint an NFT
	executeMsgRaw := fmt.Sprintf(executeMintTemplate,
		1,
		creator.Address.String(),
	)
	_, err = msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: collectionAddress,
		Sender:   creator.Address.String(),
		Msg:      []byte(executeMsgRaw),
	})
	require.NoError(t, err)

	// download latest marketplace code
	out, err := os.Create("contracts/sg_marketplace.wasm")
	require.NoError(t, err)
	defer out.Close()
	resp, err := http.Get("https://github.com/public-awesome/marketplace/releases/latest/download/sg_marketplace.wasm")
	require.NoError(t, err)
	defer resp.Body.Close()
	_, err = io.Copy(out, resp.Body)
	require.NoError(t, err)

	// marketplace
	b, err = ioutil.ReadFile("contracts/sg_marketplace.wasm")
	require.NoError(t, err)

	res, err = msgServer.StoreCode(sdk.WrapSDKContext(ctx), &wasmtypes.MsgStoreCode{
		Sender:       addr1.String(),
		WASMByteCode: b,
	})
	require.NoError(t, err)
	require.NotNil(t, res)
	require.Equal(t, res.CodeID, uint64(2))

	instantiateMsgRaw = []byte(
		fmt.Sprintf(instantiateMarketplaceTemplate,
			2,
			86400,
			15552000,
			86400,
			15552000,
			creator.Address.String(),
		),
	)
	// instantiate marketplace
	instantiateRes, err = msgServer.InstantiateContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgInstantiateContract{
		Sender: addr1.String(),
		Admin:  addr1.String(),
		CodeID: 2,
		Label:  "Marketplace",
		Msg:    instantiateMsgRaw,
	})
	require.NoError(t, err)
	require.NotNil(t, instantiateRes)
	require.NotEmpty(t, instantiateRes.Address)
	marketplaceAddress := instantiateRes.Address
	require.NotEmpty(t, marketplaceAddress)

	// approve the NFT
	executeMsgRaw = fmt.Sprintf(executeApproveTemplate,
		marketplaceAddress,
		1,
	)
	_, err = msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: collectionAddress,
		Sender:   creator.Address.String(),
		Msg:      []byte(executeMsgRaw),
	})
	require.NoError(t, err)

	// execute an ask on the marketplace
	expires, _ := time.Parse(time.RFC3339Nano, "2025-03-11T21:00:00Z")
	executeMsgRaw = fmt.Sprintf(executeAskTemplate,
		collectionAddress,
		1,
		10000000,
		expires.UnixNano(),
	)
	_, err = msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: marketplaceAddress,
		Sender:   creator.Address.String(),
		Msg:      []byte(executeMsgRaw),
	})
	require.NoError(t, err)

}
