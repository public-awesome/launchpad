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
	"github.com/tendermint/tendermint/crypto/secp256k1"
	tmproto "github.com/tendermint/tendermint/proto/tendermint/types"
)

var (
	instantiateMarketplaceTemplate = `
	{
		"trading_fee_bps": %d,
		"ask_expiry": { "min": %d, "max": %d },
		"bid_expiry": { "min": %d, "max": %d },
		"operators": ["%s"],
		"sale_hook": null,
		"max_finders_fee_bps": %d,
		"min_price": "%d",
		"stale_bid_duration": { "time": %d },
		"bid_removal_reward_bps": %d
	}
	`

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
				"sale_type": "%s",
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
	executeBidTemplate = `
		{
			"set_bid": {
				"collection": "%s",
				"token_id": %d,
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
	executeSaleHookTemplate = `
		{
			"add_sale_hook": { 
				"hook": "%s"
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
	creator := accs[0]
	bidder := accs[1]

	// claim module setup
	app.ClaimKeeper.CreateModuleAccount(ctx, sdk.NewCoin(claimtypes.DefaultClaimDenom, sdk.NewInt(5000_000_000)))
	app.ClaimKeeper.SetParams(ctx, claimtypes.Params{
		AirdropEnabled:     true,
		AirdropStartTime:   startDateTime,
		DurationUntilDecay: claimtypes.DefaultDurationUntilDecay,
		DurationOfDecay:    claimtypes.DefaultDurationOfDecay,
		ClaimDenom:         claimtypes.DefaultClaimDenom,
	})
	claimRecords := []claimtypes.ClaimRecord{
		{
			Address:                bidder.Address.String(),
			InitialClaimableAmount: sdk.NewCoins(sdk.NewInt64Coin(claimtypes.DefaultClaimDenom, 1_000_000_000)),
			ActionCompleted:        []bool{false, false, false, false, false},
		},
	}
	err = app.ClaimKeeper.SetClaimRecords(ctx, claimRecords)
	require.NoError(t, err)

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

	// mint two NFTs
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
	executeMsgRaw = fmt.Sprintf(executeMintTemplate,
		2,
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

	instantiateMsgRawString := fmt.Sprintf(instantiateMarketplaceTemplate,
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
	)
	// instantiate marketplace
	instantiateRes, err = msgServer.InstantiateContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgInstantiateContract{
		Sender: addr1.String(),
		Admin:  addr1.String(),
		CodeID: 2,
		Label:  "Marketplace",
		Msg:    []byte(instantiateMsgRawString),
	})
	require.NoError(t, err)
	require.NotNil(t, instantiateRes)
	require.NotEmpty(t, instantiateRes.Address)
	marketplaceAddress := instantiateRes.Address
	require.NotEmpty(t, marketplaceAddress)

	// claim
	b, err = ioutil.ReadFile("contracts/claim.wasm")
	require.NoError(t, err)

	res, err = msgServer.StoreCode(sdk.WrapSDKContext(ctx), &wasmtypes.MsgStoreCode{
		Sender:       addr1.String(),
		WASMByteCode: b,
	})
	require.NoError(t, err)
	require.NotNil(t, res)
	require.Equal(t, res.CodeID, uint64(3))

	instantiateRes, err = msgServer.InstantiateContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgInstantiateContract{
		Sender: creator.Address.String(),
		Admin:  creator.Address.String(),
		CodeID: 3,
		Label:  "Claim",
		Msg:    []byte(`{"marketplace_addr":"` + marketplaceAddress + `"}`),
	})
	require.NoError(t, err)
	require.NotNil(t, instantiateRes)
	require.NotEmpty(t, instantiateRes.Address)
	claimAddress := instantiateRes.Address
	require.NotEmpty(t, claimAddress)

	// allow claim contract to call into native chain claim module
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

	// set sales finalized hook on marketplace
	executeMsgRaw = fmt.Sprintf(executeSaleHookTemplate, claimAddress)
	addr, err := sdk.AccAddressFromBech32(marketplaceAddress)
	require.NoError(t, err)
	_, err = app.WasmKeeper.Sudo(ctx, addr, []byte(executeMsgRaw))
	require.NoError(t, err)

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
	// approve the NFT
	executeMsgRaw = fmt.Sprintf(executeApproveTemplate,
		marketplaceAddress,
		2,
	)
	_, err = msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: collectionAddress,
		Sender:   creator.Address.String(),
		Msg:      []byte(executeMsgRaw),
	})
	require.NoError(t, err)

	// execute an ask on the marketplace
	expires := startDateTime.Add(time.Hour * 24 * 30)
	executeMsgRaw = fmt.Sprintf(executeAskTemplate,
		"fixed_price",
		collectionAddress,
		1,
		1_000_000_000,
		expires.UnixNano(),
	)
	_, err = msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: marketplaceAddress,
		Sender:   creator.Address.String(),
		Msg:      []byte(executeMsgRaw),
	})
	require.NoError(t, err)

	// check intial balance of buyer / airdrop claimer
	balance := app.BankKeeper.GetBalance(ctx, bidder.Address, "ustars")
	require.Equal(t,
		"2000000000",
		balance.Amount.String(),
	)

	// execute a bid on the marketplace
	executeMsgRaw = fmt.Sprintf(executeBidTemplate,
		collectionAddress,
		1,
		expires.UnixNano(),
	)
	_, err = msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: marketplaceAddress,
		Sender:   bidder.Address.String(),
		Msg:      []byte(executeMsgRaw),
		Funds:    sdk.NewCoins(sdk.NewInt64Coin("ustars", 1_000_000_000)),
	})
	// sale finalized hook should have been called
	// NFT should have been transferred to bidder
	require.NoError(t, err)

	// buyer's should lose amount of bid (1,000) and gain airdrop claim amount (1,000 / 5 = 200)
	balance = app.BankKeeper.GetBalance(ctx, bidder.Address, "ustars")
	require.Equal(t,
		"1200000000",
		balance.Amount.String(),
	)

	claim, err := app.ClaimKeeper.GetClaimRecord(ctx, bidder.Address)
	require.NoError(t, err)
	require.True(t, claim.ActionCompleted[claimtypes.ActionBidNFT])

	// add another test to make sure action cannot be claimed twice

	// execute an ask on the marketplace
	executeMsgRaw = fmt.Sprintf(executeAskTemplate,
		"fixed_price",
		collectionAddress,
		2,
		1_000_000_000,
		expires.UnixNano(),
	)
	_, err = msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: marketplaceAddress,
		Sender:   creator.Address.String(),
		Msg:      []byte(executeMsgRaw),
	})
	require.NoError(t, err)

	// execute a bid on the marketplace
	executeMsgRaw = fmt.Sprintf(executeBidTemplate,
		collectionAddress,
		2,
		expires.UnixNano(),
	)
	_, err = msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: marketplaceAddress,
		Sender:   bidder.Address.String(),
		Msg:      []byte(executeMsgRaw),
		Funds:    sdk.NewCoins(sdk.NewInt64Coin("ustars", 1_000_000_000)),
	})
	// sale finalized hook should have been called
	// NFT should have been transferred to bidder
	require.NoError(t, err)

	// buyer's should lose amount of bid (1,000)
	balance = app.BankKeeper.GetBalance(ctx, bidder.Address, "ustars")
	require.Equal(t,
		"200000000",
		balance.Amount.String(),
	)

	claim, err = app.ClaimKeeper.GetClaimRecord(ctx, bidder.Address)
	require.NoError(t, err)
	require.True(t, claim.ActionCompleted[claimtypes.ActionBidNFT])

}
