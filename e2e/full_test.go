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

	startDateTime, err := time.Parse(time.RFC3339Nano, "2022-03-07T17:00:00Z")
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
		"sg721_code_id": 3,
		"sg721_instantiate_msg": {
		  "name": "Collection Name",
		  "symbol": "SYM",
		  "minter": "%s",
		  "collection_info": {
			"contract_uri": "ipfs://...",
			"creator": "%s",
			"description": "Stargaze Monkeys",
			"image": "https://example.com/image.png",
			"external_link" : "https://stargaze.zone",
			"royalty_info": {
			  "payment_address": "%s",
			  "share": "0.1"
			}
		  }
		},
		"start_time": "1647032400000000000",
		"per_address_limit": 1,
		"unit_price": {
		  "amount": "100000000",
		  "denom": "ustars"
		}
	  }	  
	`

	intialTotalSupply := app.BankKeeper.GetSupply(ctx, "ustars")

	instantiateMsgRaw := []byte(fmt.Sprintf(instantiateMsgTemplate, creator.Address.String(), creator.Address.String(), creator.Address.String()))
	instantiateRes, err := msgServer.InstantiateContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgInstantiateContract{
		Sender: creator.Address.String(),
		Admin:  creator.Address.String(),
		CodeID: 1,
		Label:  "Minter",
		Msg:    instantiateMsgRaw,
		Funds:  sdk.NewCoins(sdk.NewInt64Coin("ustars", 1000_000_000)),
	})
	require.NoError(t, err)
	require.NotNil(t, instantiateRes)
	require.NotEmpty(t, instantiateRes.Address)
	minterAddress := instantiateRes.Address

	// 500 STARS should have been burned
	require.Equal(t,
		intialTotalSupply.Amount.Sub(sdk.NewInt(500_000_000)).String(),
		app.BankKeeper.GetSupply(ctx, "ustars").Amount.String())

	// 500 STARS should have been transferred to community pool
	require.Equal(t,
		int64(500_000_000),
		app.DistrKeeper.GetFeePoolCommunityCoins(ctx).AmountOf("ustars").TruncateInt64(),
	)

	// Creator should have been charged 1000STARS
	require.Equal(t,
		app.BankKeeper.GetBalance(ctx, creator.Address, "ustars"),
		sdk.NewInt64Coin("ustars", 1000_000_000),
	)

	// mint has not started
	_, err = msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: minterAddress,
		Sender:   accs[1].Address.String(),
		Msg:      []byte(`{"mint":{}}`),
	})
	require.Error(t, err)
	require.Contains(t, err.Error(), "Minting has not started yet")

	afterGenesisMint, err := time.Parse(time.RFC3339Nano, "2022-03-11T21:00:01Z")
	require.NoError(t, err)
	ctx = app.BaseApp.NewContext(false, tmproto.Header{Height: 1, ChainID: "stargaze-1", Time: afterGenesisMint})

	// mint fails with no funds
	_, err = msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: minterAddress,
		Sender:   accs[1].Address.String(),
		Msg:      []byte(`{"mint":{}}`),
	})
	require.Error(t, err)
	require.Contains(t, err.Error(), "IncorrectPaymentAmount 0ustars != 100000000ustars")

	// mint succeeds
	_, err = msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: minterAddress,
		Sender:   accs[1].Address.String(),
		Msg:      []byte(`{"mint":{}}`),
		Funds:    sdk.NewCoins(sdk.NewInt64Coin("ustars", 100_000_000)),
	})
	require.NoError(t, err)

	// Buyer should have 100STARS less
	require.Equal(t,
		sdk.NewInt64Coin("ustars", 1900_000_000),
		app.BankKeeper.GetBalance(ctx, accs[1].Address, "ustars"),
	)

	// Creator should have earned 90%
	require.Equal(t,
		app.BankKeeper.GetBalance(ctx, creator.Address, "ustars"),
		sdk.NewInt64Coin("ustars", 1090_000_000),
	)

	// 505 STARS should have been burned so far
	require.Equal(t,
		intialTotalSupply.Amount.Sub(sdk.NewInt(505_000_000)).String(),
		app.BankKeeper.GetSupply(ctx, "ustars").Amount.String())

	// 505 STARS should have been transferred to community pool so far
	require.Equal(t,
		int64(505_000_000),
		app.DistrKeeper.GetFeePoolCommunityCoins(ctx).AmountOf("ustars").TruncateInt64(),
	)

	// // mint fails
	// _, err = msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
	// 	Contract: minterAddress,
	// 	Sender:   accs[1].Address.String(),
	// 	Msg:      []byte(`{"mint":{}}`),
	// 	Funds:    sdk.NewCoins(sdk.NewInt64Coin("ustars", 100_000_000)),
	// })
	// require.Error(t, err)
	// require.Contains(t, err.Error(), "Max minting limit per address exceeded")

	// // Buyer should have still have only 100STARS less
	// require.Equal(t,
	// 	sdk.NewInt64Coin("ustars", 1900_000_000).String(),
	// 	app.BankKeeper.GetBalance(ctx, accs[1].Address, "ustars").String(),
	// )
	count := 0
	for i := 2; i < 101; i++ {
		count++
		_, err = msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
			Contract: minterAddress,
			Sender:   accs[i].Address.String(),
			Msg:      []byte(`{"mint":{}}`),
			Funds:    sdk.NewCoins(sdk.NewInt64Coin("ustars", 100_000_000)),
		})
		require.NoError(t, err)
		// Buyer should have still have only 100STARS less
		require.Equal(t,
			sdk.NewInt64Coin("ustars", 1900_000_000).String(),
			app.BankKeeper.GetBalance(ctx, accs[i].Address, "ustars").String(),
		)
	}
	require.Equal(t, 99, count)

	// 500 +  (100 * 5) STARS should have been burned so far
	require.Equal(t,
		intialTotalSupply.Amount.Sub(sdk.NewInt(1000_000_000)).String(),
		app.BankKeeper.GetSupply(ctx, "ustars").Amount.String())

	// 500 +  (100 * 5) STARS should have been transferred to community pool so far
	require.Equal(t,
		int64(1000_000_000),
		app.DistrKeeper.GetFeePoolCommunityCoins(ctx).AmountOf("ustars").TruncateInt64(),
	)

	// Creator should have earned 90% of total sales
	// 1000 (balance) + (100 * 90 STARS)
	require.Equal(t,
		sdk.NewInt64Coin("ustars", 10_000_000_000),
		app.BankKeeper.GetBalance(ctx, creator.Address, "ustars"),
	)
}
