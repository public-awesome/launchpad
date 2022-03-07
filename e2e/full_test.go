package e2e_test

import (
	"fmt"
	"io/ioutil"
	"testing"
	"time"

	wasmkeeper "github.com/CosmWasm/wasmd/x/wasm/keeper"
	"github.com/CosmWasm/wasmd/x/wasm/types"
	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
	"github.com/public-awesome/stargaze/v3/testutil/simapp"
	"github.com/stretchr/testify/require"
	"github.com/tendermint/tendermint/crypto"
	"github.com/tendermint/tendermint/crypto/secp256k1"
	tmproto "github.com/tendermint/tendermint/proto/tendermint/types"
)

type Account struct {
	PrivKey secp256k1.PrivKey
	PubKey  crypto.PubKey
	Address sdk.AccAddress
}

func GetAccounts() []Account {
	accounts := make([]Account, 0, 150)
	for i := 0; i < 150; i++ {
		priv := secp256k1.GenPrivKey()
		pub := priv.PubKey()
		addr := sdk.AccAddress(pub.Address())
		acc := Account{
			PrivKey: priv,
			PubKey:  pub,
			Address: addr,
		}
		accounts = append(accounts, acc)
	}
	return accounts
}

func GetAccountsAndBalances(accs []Account) ([]authtypes.GenesisAccount, []banktypes.Balance) {
	genAccs := make([]authtypes.GenesisAccount, 0, len(accs))
	balances := make([]banktypes.Balance, 0, len(accs))
	for _, a := range accs {
		genAcc := authtypes.BaseAccount{
			Address: a.Address.String(),
		}
		balance := banktypes.Balance{
			Address: a.Address.String(),
			Coins:   sdk.NewCoins(sdk.NewInt64Coin("ustars", 2000_000_000)),
		}
		genAccs = append(genAccs, &genAcc)
		balances = append(balances, balance)
	}
	return genAccs, balances
}
func TestMinter(t *testing.T) {
	accs := GetAccounts()

	genAccs, balances := GetAccountsAndBalances(accs)

	app := simapp.SetupWithGenesisAccounts(t, t.TempDir(), genAccs, balances...)

	ctx := app.BaseApp.NewContext(false, tmproto.Header{Height: 1, ChainID: "stargaze-1", Time: time.Now().UTC()})

	// wasm params
	wasmParams := app.WasmKeeper.GetParams(ctx)
	wasmParams.CodeUploadAccess = wasmtypes.AllowEverybody
	wasmParams.MaxWasmCodeSize = 1000 * 1024 * 4 // 4MB
	app.WasmKeeper.SetParams(ctx, wasmParams)

	priv1 := secp256k1.GenPrivKey()
	pub1 := priv1.PubKey()
	addr1 := sdk.AccAddress(pub1.Address())

	// minter
	b, err := ioutil.ReadFile("contracts/minter.wasm")
	require.NoError(t, err)

	msgServer := wasmkeeper.NewMsgServerImpl(wasmkeeper.NewDefaultPermissionKeeper(app.WasmKeeper))
	res, err := msgServer.StoreCode(sdk.WrapSDKContext(ctx), &types.MsgStoreCode{
		Sender:       addr1.String(),
		WASMByteCode: b,
	})
	require.NoError(t, err)
	require.NotNil(t, res)
	require.Equal(t, res.CodeID, uint64(1))

	// whitelist
	b, err = ioutil.ReadFile("contracts/whitelist.wasm")
	require.NoError(t, err)

	res, err = msgServer.StoreCode(sdk.WrapSDKContext(ctx), &types.MsgStoreCode{
		Sender:       addr1.String(),
		WASMByteCode: b,
	})
	require.NoError(t, err)
	require.NotNil(t, res)
	require.Equal(t, res.CodeID, uint64(2))

	// sg721
	b, err = ioutil.ReadFile("contracts/sg721.wasm")
	require.NoError(t, err)

	res, err = msgServer.StoreCode(sdk.WrapSDKContext(ctx), &types.MsgStoreCode{
		Sender:       addr1.String(),
		WASMByteCode: b,
	})
	require.NoError(t, err)
	require.NotNil(t, res)
	require.Equal(t, res.CodeID, uint64(3))

	creator := accs[0]
	// minter

	instantiateMsgTemplate := `
	{
		"base_token_uri": "ipfs://...",
		"num_tokens": 100,
		"sg721_code_id": 2,
		"sg721_instantiate_msg": {
		  "name": "Collection Name",
		  "symbol": "SYM",
		  "minter": "%s",
		  "config": {
			"contract_uri": "ipfs://...",
			"creator": "%s",
			"royalties": {
			  "payment_address": "%s",
			  "share": "0.1"
			}
		  }
		},
		"start_time": {
		  "at_time": "1646258400000000000"
		},
		"unit_price": {
		  "amount": "100000000",
		  "denom": "ustars"
		}
	  }	  
	`
	instantiateMsgRaw := []byte(fmt.Sprintf(instantiateMsgTemplate, creator.Address.String(), creator.Address.String(), creator.Address.String()))
	instantiateRes, err := msgServer.InstantiateContract(sdk.WrapSDKContext(ctx), &types.MsgInstantiateContract{
		Sender: creator.Address.String(),
		Admin:  creator.Address.String(),
		CodeID: 1,
		Label:  "Minter",
		Msg:    instantiateMsgRaw,
		Funds:  sdk.NewCoins(sdk.NewInt64Coin("ustars", 1000_000_000)),
	})
	require.NoError(t, err)
	require.NotNil(t, instantiateRes)

}
