package e2e_test

import (
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

	suite.sg721CodeID, err = StoreContract(suite.parentCtx, suite.msgServer, suite.accounts[0].Address.String(), "sg721_base.wasm")
	suite.Require().NoError(err)
	suite.Require().Equal(uint64(1), suite.sg721CodeID)

	suite.minterCodeID, err = StoreContract(suite.parentCtx, suite.msgServer, suite.accounts[0].Address.String(), "vending_minter.wasm")
	suite.Require().NoError(err)
	suite.Require().Equal(uint64(2), suite.minterCodeID)

	marketplaceURL := "https://github.com/public-awesome/marketplace/releases/download/v1.1.0/sg_marketplace.wasm"
	suite.marketplaceCodeID, err = StoreContract(suite.parentCtx, suite.msgServer, suite.accounts[0].Address.String(), marketplaceURL)
	suite.Require().NoError(err)
	suite.Require().Equal(uint64(3), suite.marketplaceCodeID)
}

func TestMarketplaceTestSuite(t *testing.T) {
	suite.Run(t, new(MarketplaceTestSuite))
}
