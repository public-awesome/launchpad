package e2e_test

import (
	"fmt"
	"io/ioutil"
	"strings"
	"testing"
	"time"

	wasmkeeper "github.com/CosmWasm/wasmd/x/wasm/keeper"
	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
	"github.com/public-awesome/stargaze/v5/testutil/simapp"
	"github.com/stretchr/testify/require"
	"github.com/tendermint/tendermint/crypto"
	"github.com/tendermint/tendermint/crypto/secp256k1"
	tmproto "github.com/tendermint/tendermint/proto/tendermint/types"
)

var (
	// whitelist
	instantiateWhiteListTemplate = `
		{
			"members":[%s],
			"start_time": "%d",
			"end_time": "%d",
			"unit_price": {
				"amount": "50000000",
				"denom": "ustars"
			},
			"per_address_limit": 1,
			"member_limit": 1000
		}
		`

	instantiateMinterTemplate = `
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
			"start_time": "%d",
			"whitelist" : %s, 
			"per_address_limit": %d,
			"unit_price": {
			  "amount": "100000000",
			  "denom": "ustars"
			}
		  }	  
		`
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
			Coins:   sdk.NewCoins(sdk.NewInt64Coin("ustars", 2_000_000_000)),
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

	// minter
	b, err := ioutil.ReadFile("contracts/minter.wasm")
	require.NoError(t, err)

	msgServer := wasmkeeper.NewMsgServerImpl(wasmkeeper.NewDefaultPermissionKeeper(app.WasmKeeper))
	res, err := msgServer.StoreCode(sdk.WrapSDKContext(ctx), &wasmtypes.MsgStoreCode{
		Sender:       addr1.String(),
		WASMByteCode: b,
	})
	require.NoError(t, err)
	require.NotNil(t, res)
	require.Equal(t, res.CodeID, uint64(1))

	// whitelist
	b, err = ioutil.ReadFile("contracts/whitelist.wasm")
	require.NoError(t, err)

	res, err = msgServer.StoreCode(sdk.WrapSDKContext(ctx), &wasmtypes.MsgStoreCode{
		Sender:       addr1.String(),
		WASMByteCode: b,
	})
	require.NoError(t, err)
	require.NotNil(t, res)
	require.Equal(t, res.CodeID, uint64(2))

	// sg721
	b, err = ioutil.ReadFile("contracts/sg721.wasm")
	require.NoError(t, err)

	res, err = msgServer.StoreCode(sdk.WrapSDKContext(ctx), &wasmtypes.MsgStoreCode{
		Sender:       addr1.String(),
		WASMByteCode: b,
	})
	require.NoError(t, err)
	require.NotNil(t, res)
	require.Equal(t, res.CodeID, uint64(3))

	creator := accs[0]
	// minter

	genesisMintDateTime, err := time.Parse(time.RFC3339Nano, "2022-03-11T21:00:00Z")
	require.NoError(t, err)
	intialTotalSupply := app.BankKeeper.GetSupply(ctx, "ustars")

	instantiateMsgRaw := []byte(
		fmt.Sprintf(instantiateMinterTemplate,
			creator.Address.String(),
			creator.Address.String(),
			creator.Address.String(),
			genesisMintDateTime.UnixNano(),
			"null",
			1, // limit 1
		),
	)
	instantiateRes, err := msgServer.InstantiateContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgInstantiateContract{
		Sender: creator.Address.String(),
		Admin:  creator.Address.String(),
		CodeID: 1,
		Label:  "Minter",
		Msg:    instantiateMsgRaw,
		Funds:  sdk.NewCoins(sdk.NewInt64Coin("ustars", 1_000_000_000)),
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
		sdk.NewInt64Coin("ustars", 1_000_000_000),
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
		sdk.NewInt64Coin("ustars", 1_900_000_000),
		app.BankKeeper.GetBalance(ctx, accs[1].Address, "ustars"),
	)

	// Creator should have the same amount they started with since it hasn't been withdrawn yet
	require.Equal(t,
		app.BankKeeper.GetBalance(ctx, creator.Address, "ustars"),
		sdk.NewInt64Coin("ustars", 1_000_000_000),
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
		intialTotalSupply.Amount.Sub(sdk.NewInt(1_000_000_000)).String(),
		app.BankKeeper.GetSupply(ctx, "ustars").Amount.String())

	// 500 +  (100 * 5) STARS should have been transferred to community pool so far
	require.Equal(t,
		int64(1_000_000_000),
		app.DistrKeeper.GetFeePoolCommunityCoins(ctx).AmountOf("ustars").TruncateInt64(),
	)

	// Creator should have the same balance before withdrawal
	require.Equal(t,
		sdk.NewInt64Coin("ustars", 1_000_000_000),
		app.BankKeeper.GetBalance(ctx, creator.Address, "ustars"),
	)

	// withdraw succeeds
	_, err = msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: minterAddress,
		Sender:   creator.Address.String(),
		Msg:      []byte(`{"withdraw":{}}`),
	})
	require.NoError(t, err)

	// Creator should have earned 90% of total sales
	// 1000 (balance) + (100 * 90 STARS)
	require.Equal(t,
		sdk.NewInt64Coin("ustars", 10_000_000_000),
		app.BankKeeper.GetBalance(ctx, creator.Address, "ustars"),
	)

}

func TestWhitelistMinter(t *testing.T) {
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

	// minter
	b, err := ioutil.ReadFile("contracts/minter.wasm")
	require.NoError(t, err)

	msgServer := wasmkeeper.NewMsgServerImpl(wasmkeeper.NewDefaultPermissionKeeper(app.WasmKeeper))
	res, err := msgServer.StoreCode(sdk.WrapSDKContext(ctx), &wasmtypes.MsgStoreCode{
		Sender:       addr1.String(),
		WASMByteCode: b,
	})
	require.NoError(t, err)
	require.NotNil(t, res)
	require.Equal(t, res.CodeID, uint64(1))

	// whitelist
	b, err = ioutil.ReadFile("contracts/whitelist.wasm")
	require.NoError(t, err)

	res, err = msgServer.StoreCode(sdk.WrapSDKContext(ctx), &wasmtypes.MsgStoreCode{
		Sender:       addr1.String(),
		WASMByteCode: b,
	})
	require.NoError(t, err)
	require.NotNil(t, res)
	require.Equal(t, res.CodeID, uint64(2))

	// sg721
	b, err = ioutil.ReadFile("contracts/sg721.wasm")
	require.NoError(t, err)

	res, err = msgServer.StoreCode(sdk.WrapSDKContext(ctx), &wasmtypes.MsgStoreCode{
		Sender:       addr1.String(),
		WASMByteCode: b,
	})
	require.NoError(t, err)
	require.NotNil(t, res)
	require.Equal(t, res.CodeID, uint64(3))

	creator := accs[0]
	intialTotalSupply := app.BankKeeper.GetSupply(ctx, "ustars")

	whitelistedAccounts := accs[1:51]
	require.Len(t, whitelistedAccounts, 50)

	members := make([]string, 0, 50)
	for _, a := range whitelistedAccounts {
		members = append(members, fmt.Sprintf("\"%s\"", a.Address.String()))
	}

	whitelistStartTime, err := time.Parse(time.RFC3339Nano, "2022-03-11T21:00:00Z")
	require.NoError(t, err)
	whitelistEndTime, err := time.Parse(time.RFC3339Nano, "2022-03-12T17:00:00Z")
	require.NoError(t, err)

	instantiateMsgRaw := []byte(fmt.Sprintf(instantiateWhiteListTemplate,
		strings.Join(members, ","),
		whitelistStartTime.UnixNano(),
		whitelistEndTime.UnixNano()))
	instantiateRes, err := msgServer.InstantiateContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgInstantiateContract{
		Sender: creator.Address.String(),
		Admin:  creator.Address.String(),
		CodeID: 2,
		Label:  "Whitelist",
		Msg:    instantiateMsgRaw,
		Funds:  sdk.NewCoins(sdk.NewInt64Coin("ustars", 100_000_000)),
	})
	require.NoError(t, err)
	require.NotNil(t, instantiateRes)
	whitelistAddr := instantiateRes.Address
	require.NotEmpty(t, whitelistAddr)

	// 50 STARS should have been burned
	require.Equal(t,
		intialTotalSupply.Amount.Sub(sdk.NewInt(50_000_000)).String(),
		app.BankKeeper.GetSupply(ctx, "ustars").Amount.String())

	// 50 STARS should have been transferred to community pool
	require.Equal(t,
		int64(50_000_000),
		app.DistrKeeper.GetFeePoolCommunityCoins(ctx).AmountOf("ustars").TruncateInt64(),
	)

	// Creator should have been charged 100STARS
	require.Equal(t,
		app.BankKeeper.GetBalance(ctx, creator.Address, "ustars"),
		sdk.NewInt64Coin("ustars", 1_900_000_000),
	)

	// MINTER
	//

	whiteListAddrStr := fmt.Sprintf("\"%s\"", whitelistAddr)
	instantiateMinterMsgRaw := []byte(
		fmt.Sprintf(instantiateMinterTemplate,
			creator.Address.String(),
			creator.Address.String(),
			creator.Address.String(),
			whitelistEndTime.UnixNano(),
			whiteListAddrStr,
			2, // limit public mint to 2
		),
	)
	instantiateMinterRes, err := msgServer.InstantiateContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgInstantiateContract{
		Sender: creator.Address.String(),
		Admin:  creator.Address.String(),
		CodeID: 1,
		Label:  "Minter",
		Msg:    instantiateMinterMsgRaw,
		Funds:  sdk.NewCoins(sdk.NewInt64Coin("ustars", 1_000_000_000)),
	})
	require.NoError(t, err)
	require.NotNil(t, instantiateMinterRes)
	require.NotEmpty(t, instantiateMinterRes.Address)
	minterAddress := instantiateMinterRes.Address

	// 550 STARS should have been burned so far
	require.Equal(t,
		intialTotalSupply.Amount.Sub(sdk.NewInt(550_000_000)).String(),
		app.BankKeeper.GetSupply(ctx, "ustars").Amount.String())

	// 550 STARS should have been transferred to community pool so far
	require.Equal(t,
		int64(550_000_000),
		app.DistrKeeper.GetFeePoolCommunityCoins(ctx).AmountOf("ustars").TruncateInt64(),
	)

	// Creator should have been charged another 1000STARS
	require.Equal(t,
		app.BankKeeper.GetBalance(ctx, creator.Address, "ustars"),
		sdk.NewInt64Coin("ustars", 900_000_000),
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

	// not on whitelist
	_, err = msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: minterAddress,
		Sender:   accs[100].Address.String(),
		Msg:      []byte(`{"mint":{}}`),
	})
	require.Error(t, err)
	require.Contains(t, err.Error(), "address not on whitelist")

	// mint succeeds
	_, err = msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: minterAddress,
		Sender:   accs[1].Address.String(),
		Msg:      []byte(`{"mint":{}}`),
		Funds:    sdk.NewCoins(sdk.NewInt64Coin("ustars", 50_000_000)),
	})
	require.NoError(t, err)

	// mint fails as per limit in whitelist is 1
	_, err = msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: minterAddress,
		Sender:   accs[1].Address.String(),
		Msg:      []byte(`{"mint":{}}`),
		Funds:    sdk.NewCoins(sdk.NewInt64Coin("ustars", 0)),
	})
	require.Error(t, err)
	require.Contains(t, err.Error(), "Max minting limit per address exceeded")
	// buy whitelist

	count := 0
	for i := 2; i < 51; i++ {
		count++
		_, err = msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
			Contract: minterAddress,
			Sender:   accs[i].Address.String(),
			Msg:      []byte(`{"mint":{}}`),
			Funds:    sdk.NewCoins(sdk.NewInt64Coin("ustars", 50_000_000)),
		})
		require.NoError(t, err)
		// Buyer should have still have only 50STARS less
		require.Equal(t,
			sdk.NewInt64Coin("ustars", 1_950_000_000).String(),
			app.BankKeeper.GetBalance(ctx, accs[i].Address, "ustars").String(),
		)
	}
	require.Equal(t, 49, count)

	require.NoError(t, err)
	ctx = app.BaseApp.NewContext(false, tmproto.Header{Height: 1, ChainID: "stargaze-1",
		Time: whitelistEndTime.Add(time.Second * 1)})

	// mint should succeed as per min limit in public is 2
	_, err = msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: minterAddress,
		Sender:   accs[1].Address.String(),
		Msg:      []byte(`{"mint":{}}`),
		Funds:    sdk.NewCoins(sdk.NewInt64Coin("ustars", 100_000_000)),
	})
	require.NoError(t, err)

	// Buyer should have 150STARS less
	require.Equal(t,
		sdk.NewInt64Coin("ustars", 1_850_000_000).String(),
		app.BankKeeper.GetBalance(ctx, accs[1].Address, "ustars").String(),
	)

	// already sold 51 items

	publicMintCount := 0
	for i := 51; i < 100; i++ {
		publicMintCount++
		_, err = msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
			Contract: minterAddress,
			Sender:   accs[i].Address.String(),
			Msg:      []byte(`{"mint":{}}`),
			Funds:    sdk.NewCoins(sdk.NewInt64Coin("ustars", 100_000_000)),
		})
		require.NoError(t, err)
		// Buyer should have still have only 50STARS less
		require.Equal(t,
			sdk.NewInt64Coin("ustars", 1_900_000_000).String(),
			app.BankKeeper.GetBalance(ctx, accs[i].Address, "ustars").String(),
		)
	}
	require.Equal(t, 49, publicMintCount)

	// mint fails Already sold out
	_, err = msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: minterAddress,
		Sender:   accs[120].Address.String(),
		Msg:      []byte(`{"mint":{}}`),
		Funds:    sdk.NewCoins(sdk.NewInt64Coin("ustars", 100_000_000)),
	})
	require.Error(t, err)
	require.Contains(t, err.Error(), "Sold out")
	// TODO: fake refund since test failed

	// Check Supply
	// - 50 whitelist fee
	// - 500 minter fee
	// - Whitelist Price : 50STARS * 5%  * 50 sold = 125
	// - Public Price: 100STARS * 5% * 50 sold =  250
	// should have 925STARS less
	require.Equal(t,
		intialTotalSupply.Amount.Sub(sdk.NewInt(925_000_000)).String(),
		app.BankKeeper.GetSupply(ctx, "ustars").Amount.String())

	//  should have 925STARS more
	require.Equal(t,
		int64(925_000_000),
		app.DistrKeeper.GetFeePoolCommunityCoins(ctx).AmountOf("ustars").TruncateInt64(),
	)

	// Creator should have their initial balance before withdrawal
	// 900 (balance)
	require.Equal(t,
		sdk.NewInt64Coin("ustars", 900_000_000),
		app.BankKeeper.GetBalance(ctx, creator.Address, "ustars"),
	)

	// withdraw succeeds
	_, err = msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: minterAddress,
		Sender:   creator.Address.String(),
		Msg:      []byte(`{"withdraw":{}}`),
	})
	require.NoError(t, err)

	// Creator should have earned 90% of total sales
	// 900 (balance)
	// 50STARS * 90% * 50 sold = 2,250
	// 100STARS * 90% * 50 sold = 4,500 + 100 (1 failed mint)
	// should have 7,750 STARS
	require.Equal(t,
		sdk.NewInt64Coin("ustars", 7_750_000_000),
		app.BankKeeper.GetBalance(ctx, creator.Address, "ustars"),
	)

}
