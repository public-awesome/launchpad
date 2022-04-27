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
	tmproto "github.com/tendermint/tendermint/proto/tendermint/types"
)

func TestMarketplace(t *testing.T) {
	accs := GetAccounts()
	addr1 := accs[1].Address

	genAccs, balances := GetAccountsAndBalances(accs)

	app := simapp.SetupWithGenesisAccounts(t, t.TempDir(), genAccs, balances...)

	startDateTime, err := time.Parse(time.RFC3339Nano, "2022-03-11T20:59:00Z")
	require.NoError(t, err)
	ctx := app.BaseApp.NewContext(false, tmproto.Header{Height: 1, ChainID: "stargaze-1", Time: startDateTime})

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
	b, err := ioutil.ReadFile("contracts/sg_marketplace.wasm")
	require.NoError(t, err)

	msgServer := wasmkeeper.NewMsgServerImpl(wasmkeeper.NewDefaultPermissionKeeper(app.WasmKeeper))
	res, err := msgServer.StoreCode(sdk.WrapSDKContext(ctx), &wasmtypes.MsgStoreCode{
		Sender:       addr1.String(),
		WASMByteCode: b,
	})
	require.NoError(t, err)
	require.NotNil(t, res)
	require.Equal(t, res.CodeID, uint64(4))

	creator := accs[0]

	instantiateMsgRaw := []byte(
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
	instantiateRes, err := msgServer.InstantiateContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgInstantiateContract{
		Sender: addr1.String(),
		Admin:  addr1.String(),
		CodeID: 1,
		Label:  "Marketplace",
		Msg:    instantiateMsgRaw,
	})
	require.NoError(t, err)
	require.NotNil(t, instantiateRes)
	require.NotEmpty(t, instantiateRes.Address)
	marketplaceAddress := instantiateRes.Address
	require.NotEmpty(t, marketplaceAddress)

}
