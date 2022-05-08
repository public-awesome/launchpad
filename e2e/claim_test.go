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
	"github.com/public-awesome/stargaze/v5/testutil/simapp"
	claimtypes "github.com/public-awesome/stargaze/v5/x/claim/types"
	"github.com/stretchr/testify/require"
	tmproto "github.com/tendermint/tendermint/proto/tendermint/types"
)

func TestClaim(t *testing.T) {
	accs := GetAccounts()

	genAccs, balances := GetAccountsAndBalances(accs)

	app := simapp.SetupWithGenesisAccounts(t, t.TempDir(), genAccs, balances...)

	startDateTime, err := time.Parse(time.RFC3339Nano, "2022-03-11T20:59:00Z")
	require.NoError(t, err)
	ctx := app.BaseApp.NewContext(false, tmproto.Header{Height: 1, ChainID: "stargaze-1", Time: startDateTime})

	app.ClaimKeeper.CreateModuleAccount(ctx, sdk.NewCoin(claimtypes.DefaultClaimDenom, sdk.NewInt(5000_000_000)))

	app.ClaimKeeper.SetParams(ctx, claimtypes.Params{
		AirdropEnabled:     true,
		AirdropStartTime:   startDateTime,
		DurationUntilDecay: claimtypes.DefaultDurationUntilDecay,
		DurationOfDecay:    claimtypes.DefaultDurationOfDecay,
		ClaimDenom:         claimtypes.DefaultClaimDenom,
	})

	// wasm params
	wasmParams := app.WasmKeeper.GetParams(ctx)
	wasmParams.CodeUploadAccess = wasmtypes.AllowEverybody
	wasmParams.MaxWasmCodeSize = 1000 * 1024 * 4 // 4MB
	app.WasmKeeper.SetParams(ctx, wasmParams)

	addr1 := accs[1].Address

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

	// claim
	b, err = ioutil.ReadFile("contracts/claim.wasm")
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
	require.Equal(t, res.CodeID, uint64(4))

	creator := accs[0]

	instantiateMsgRaw := []byte(
		fmt.Sprintf(instantiateMarketplaceTemplate,
			200,
			86400,
			15552000,
			86400,
			15552000,
			creator.Address.String(),
			500,
			5000000,
			15552000,
			500,
		),
	)

	// instantiate marketplace
	instantiateRes, err := msgServer.InstantiateContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgInstantiateContract{
		Sender: creator.Address.String(),
		Admin:  creator.Address.String(),
		CodeID: 4,
		Label:  "Marketplace",
		Msg:    instantiateMsgRaw,
	})
	require.NoError(t, err)
	require.NotNil(t, instantiateRes)
	require.NotEmpty(t, instantiateRes.Address)
	marketplaceAddress := instantiateRes.Address
	require.NotEmpty(t, marketplaceAddress)

	// minter
	afterGenesisMint, err := time.Parse(time.RFC3339Nano, "2022-03-11T21:00:01Z")
	require.NoError(t, err)
	genesisMintDateTime, err := time.Parse(time.RFC3339Nano, "2022-03-11T21:00:00Z")
	require.NoError(t, err)

	instantiateMsgRaw = []byte(
		fmt.Sprintf(instantiateMinterTemplate,
			creator.Address.String(),
			creator.Address.String(),
			creator.Address.String(),
			genesisMintDateTime.UnixNano(),
			"null",
			1, // limit 1
		),
	)
	instantiateRes, err = msgServer.InstantiateContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgInstantiateContract{
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
	ctx = app.BaseApp.NewContext(false, tmproto.Header{Height: 1, ChainID: "stargaze-1", Time: afterGenesisMint})

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
		sdk.NewInt64Coin("ustars", 1_900_000_000).String(),
		app.BankKeeper.GetBalance(ctx, accs[1].Address, "ustars").String(),
	)

	claimRecords := []claimtypes.ClaimRecord{
		{
			Address:                addr1.String(),
			InitialClaimableAmount: sdk.NewCoins(sdk.NewInt64Coin(claimtypes.DefaultClaimDenom, 1_000_000_000)),
			ActionCompleted:        []bool{false, false, false, false, false},
		},
	}
	err = app.ClaimKeeper.SetClaimRecords(ctx, claimRecords)
	require.NoError(t, err)

	instantiateRes, err = msgServer.InstantiateContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgInstantiateContract{
		Sender: creator.Address.String(),
		Admin:  creator.Address.String(),
		CodeID: 2,
		Label:  "Claim",
		Msg:    []byte(`{"marketplace_addr":"` + marketplaceAddress + `"}`),
	})
	require.NoError(t, err)
	require.NotNil(t, instantiateRes)
	require.NotEmpty(t, instantiateRes.Address)
	claimAddress := instantiateRes.Address
	require.NotEmpty(t, claimAddress)

	// claim airdrop
	claimMsgTemplate := `
		{
		"claim_mint_nft" : {
				"minter_address":"%s"
			}
		}
	`
	claimMsgRaw := []byte(fmt.Sprintf(claimMsgTemplate, minterAddress))
	_, err = msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: claimAddress,
		Sender:   accs[1].Address.String(),
		Msg:      claimMsgRaw,
	})
	require.Error(t, err)
	require.Contains(t, err.Error(), "address is not allowed to claim")

	// allow contract

	app.ClaimKeeper.SetParams(ctx, claimtypes.Params{
		AirdropEnabled:     true,
		AirdropStartTime:   startDateTime,
		DurationUntilDecay: claimtypes.DefaultDurationUntilDecay,
		DurationOfDecay:    claimtypes.DefaultDurationOfDecay,
		ClaimDenom:         claimtypes.DefaultClaimDenom,
		AllowedClaimers: []claimtypes.ClaimAuthorization{
			{
				ContractAddress: claimAddress,
				Action:          claimtypes.ActionBidNFT,
			},
		},
	})

	claimMsgRaw = []byte(fmt.Sprintf(claimMsgTemplate, minterAddress))
	_, err = msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: claimAddress,
		Sender:   accs[1].Address.String(),
		Msg:      claimMsgRaw,
	})
	require.Error(t, err)
	require.Contains(t, err.Error(), "address is not allowed to claim")
	claim, err := app.ClaimKeeper.GetClaimRecord(ctx, addr1)
	require.NoError(t, err)
	require.False(t, claim.ActionCompleted[claimtypes.ActionMintNFT])

	app.ClaimKeeper.SetParams(ctx, claimtypes.Params{
		AirdropEnabled:     true,
		AirdropStartTime:   startDateTime,
		DurationUntilDecay: claimtypes.DefaultDurationUntilDecay,
		DurationOfDecay:    claimtypes.DefaultDurationOfDecay,
		ClaimDenom:         claimtypes.DefaultClaimDenom,
		AllowedClaimers: []claimtypes.ClaimAuthorization{
			{
				ContractAddress: claimAddress,
				Action:          claimtypes.ActionMintNFT,
			},
		},
	})

	balance := app.BankKeeper.GetBalance(ctx, accs[1].Address, "ustars")
	require.Equal(t,
		"1900000000",
		balance.Amount.String(),
	)

	claimMsgRaw = []byte(fmt.Sprintf(claimMsgTemplate, minterAddress))
	_, err = msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: claimAddress,
		Sender:   accs[1].Address.String(),
		Msg:      claimMsgRaw,
	})
	require.NoError(t, err)
	balance = app.BankKeeper.GetBalance(ctx, accs[1].Address, "ustars")
	perAction := claimRecords[0].InitialClaimableAmount.AmountOf(claimtypes.DefaultClaimDenom).Quo(sdk.NewInt(int64(len(claimRecords[0].ActionCompleted))))

	require.Equal(t, perAction.Int64(), int64(200_000_000))
	expectedBalance := perAction.Add(sdk.NewInt(1_900_000_000)) // user already had 19000
	require.Equal(t,
		expectedBalance.String(),
		balance.Amount.String(),
	)

	claim, err = app.ClaimKeeper.GetClaimRecord(ctx, addr1)
	require.NoError(t, err)
	require.True(t, claim.ActionCompleted[claimtypes.ActionMintNFT])

	claimMsgRaw = []byte(fmt.Sprintf(claimMsgTemplate, minterAddress))
	_, err = msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: claimAddress,
		Sender:   accs[1].Address.String(),
		Msg:      claimMsgRaw,
	})
	require.NoError(t, err)

	balance = app.BankKeeper.GetBalance(ctx, accs[1].Address, "ustars")
	require.Equal(t,
		expectedBalance.String(),
		balance.Amount.String(),
		"balance should stay the same",
	)

}
