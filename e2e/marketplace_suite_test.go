package e2e_test

import (
	"encoding/json"
	"testing"
	"time"

	wasmkeeper "github.com/CosmWasm/wasmd/x/wasm/keeper"
	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/public-awesome/stargaze/v7/app"
	"github.com/public-awesome/stargaze/v7/testutil/simapp"
	"github.com/stretchr/testify/suite"
	tmproto "github.com/tendermint/tendermint/proto/tendermint/types"
)

type MarketplaceTestSuite struct {
	suite.Suite
	msgServer wasmtypes.MsgServer
	parentCtx sdk.Context
	app       *app.App
	startTime time.Time

	accounts          []Account
	claimCodeID       uint64
	sg721CodeID       uint64
	minterCodeID      uint64
	marketplaceCodeID uint64
}

func (suite *MarketplaceTestSuite) SetupSuite() {

	suite.accounts = GetAccounts()
	genAccs, balances := GetAccountsAndBalances(suite.accounts)

	suite.app = simapp.SetupWithGenesisAccounts(suite.T(), suite.T().TempDir(), genAccs, balances...)

	startDateTime, err := time.Parse(time.RFC3339Nano, "2022-03-11T20:59:00Z")
	suite.Require().NoError(err)
	suite.startTime = startDateTime
	suite.parentCtx = suite.app.BaseApp.NewContext(false, tmproto.Header{Height: 1, ChainID: "stargaze-1", Time: startDateTime})

	// wasm params
	wasmParams := suite.app.WasmKeeper.GetParams(suite.parentCtx)
	wasmParams.CodeUploadAccess = wasmtypes.AllowEverybody
	suite.app.WasmKeeper.SetParams(suite.parentCtx, wasmParams)
	suite.msgServer = wasmkeeper.NewMsgServerImpl(wasmkeeper.NewDefaultPermissionKeeper(suite.app.WasmKeeper))

	suite.claimCodeID, err = StoreContract(suite.parentCtx, suite.msgServer, suite.accounts[0].Address.String(), "claim.wasm")
	suite.Require().NoError(err)
	suite.Require().Equal(uint64(1), suite.claimCodeID)

	suite.sg721CodeID, err = StoreContract(suite.parentCtx, suite.msgServer, suite.accounts[0].Address.String(), "sg721_base.wasm")
	suite.Require().NoError(err)
	suite.Require().Equal(uint64(2), suite.sg721CodeID)

	suite.minterCodeID, err = StoreContract(suite.parentCtx, suite.msgServer, suite.accounts[0].Address.String(), "vending_minter.wasm")
	suite.Require().NoError(err)
	suite.Require().Equal(uint64(3), suite.minterCodeID)

	marketplaceURL := "https://github.com/public-awesome/marketplace/releases/download/v1.1.0/sg_marketplace.wasm"
	suite.marketplaceCodeID, err = StoreContract(suite.parentCtx, suite.msgServer, suite.accounts[0].Address.String(), marketplaceURL)
	suite.Require().NoError(err)
	suite.Require().Equal(uint64(4), suite.marketplaceCodeID)
}

func TestMarketplaceTestSuite(t *testing.T) {
	suite.Run(t, new(MarketplaceTestSuite))
}

func (suite *MarketplaceTestSuite) TestUnauthorizedSG71Instantiation() {
	ctx, _ := suite.parentCtx.CacheContext()
	creator := suite.accounts[0]
	_, err := InstantiateSG721(ctx, suite.msgServer, creator.Address, suite.sg721CodeID, nil)
	suite.Require().Error(err)
	suite.Equal(err.Error(), "Unauthorized: instantiate wasm contract failed")
}

func InstantiateSG721(ctx sdk.Context, msgServer wasmtypes.MsgServer, account sdk.AccAddress, codeID uint64, royalties *RoyaltyInfo) (string, error) {
	instantiate := SG721InstantiateMsg{
		Name:   "Collection Name",
		Symbol: "COL",
		Minter: account.String(),
		CollectionInfo: CollectionInfo{
			Creator:      account.String(),
			Description:  "Description",
			Image:        "https://example.com/image.png",
			ExternalLink: strPtr("https://github.com/public-awesome"),
			RoyaltyInfo:  royalties,
		},
	}
	instantiateMsgRaw, err := json.Marshal(&instantiate)
	if err != nil {
		return "", err
	}

	instantiateRes, err := msgServer.InstantiateContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgInstantiateContract{
		Sender: account.String(),
		Admin:  account.String(),
		CodeID: codeID,
		Label:  "SG721",
		Msg:    instantiateMsgRaw,
		Funds:  sdk.NewCoins(),
	})
	if err != nil {
		return "", err
	}
	return instantiateRes.Address, nil
}
