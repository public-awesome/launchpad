package e2e_test

import (
	"fmt"
	"io"
	"io/ioutil"
	"net/http"
	"strings"
	"testing"
	"time"

	wasmkeeper "github.com/CosmWasm/wasmd/x/wasm/keeper"
	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/public-awesome/stargaze/v5/app"
	"github.com/public-awesome/stargaze/v5/testutil/simapp"
	"github.com/stretchr/testify/suite"
	tmproto "github.com/tendermint/tendermint/proto/tendermint/types"
)

type MarketplaceTestSuite struct {
	suite.Suite
	msgServer         wasmtypes.MsgServer
	parentCtx         sdk.Context
	app               *app.App
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
	suite.parentCtx = suite.app.BaseApp.NewContext(false, tmproto.Header{Height: 1, ChainID: "stargaze-1", Time: startDateTime})

	// wasm params
	wasmParams := suite.app.WasmKeeper.GetParams(suite.parentCtx)
	wasmParams.CodeUploadAccess = wasmtypes.AllowEverybody
	wasmParams.MaxWasmCodeSize = 1000 * 1024 * 4 // 4MB
	suite.app.WasmKeeper.SetParams(suite.parentCtx, wasmParams)
	suite.msgServer = wasmkeeper.NewMsgServerImpl(wasmkeeper.NewDefaultPermissionKeeper(suite.app.WasmKeeper))

	suite.claimCodeID, err = StoreContract(suite.parentCtx, suite.msgServer, suite.accounts[0].Address.String(), "claim.wasm")
	suite.Require().NoError(err)
	suite.Require().Equal(uint64(1), suite.claimCodeID)

	suite.sg721CodeID, err = StoreContract(suite.parentCtx, suite.msgServer, suite.accounts[0].Address.String(), "sg721.wasm")
	suite.Require().NoError(err)
	suite.Require().Equal(uint64(2), suite.sg721CodeID)

	suite.minterCodeID, err = StoreContract(suite.parentCtx, suite.msgServer, suite.accounts[0].Address.String(), "minter.wasm")
	suite.Require().NoError(err)
	suite.Require().Equal(uint64(3), suite.minterCodeID)

	marketplaceURL := "https://github.com/public-awesome/marketplace/releases/latest/download/sg_marketplace.wasm"
	suite.marketplaceCodeID, err = StoreContract(suite.parentCtx, suite.msgServer, suite.accounts[0].Address.String(), marketplaceURL)
	suite.Require().NoError(err)
	suite.Require().Equal(uint64(4), suite.marketplaceCodeID)
}

func TestMarketplaceTestSuite(t *testing.T) {
	suite.Run(t, new(MarketplaceTestSuite))
}

func GetContractBytes(contract string) ([]byte, error) {
	if strings.HasPrefix(contract, "https://") {
		resp, err := http.Get(contract)
		if err != nil {
			return nil, err
		}
		defer resp.Body.Close()
		return io.ReadAll(resp.Body)
	}

	return ioutil.ReadFile(fmt.Sprintf("contracts/%s", contract))
}
func StoreContract(ctx sdk.Context, msgServer wasmtypes.MsgServer, creator string, contract string) (uint64, error) {
	b, err := GetContractBytes(contract)
	if err != nil {
		return 0, err
	}
	res, err := msgServer.StoreCode(sdk.WrapSDKContext(ctx), &wasmtypes.MsgStoreCode{
		Sender:       creator,
		WASMByteCode: b,
	})
	if err != nil {
		return 0, err
	}
	return res.CodeID, nil
}

func (suite *MarketplaceTestSuite) TestRoyalties() {
	ctx, _ := suite.parentCtx.CacheContext()
	creator := suite.accounts[0]

	sg721Address, err := InstantiateSG721(ctx, suite.msgServer, creator.Address, suite.sg721CodeID)
	suite.Require().NoError(err)
	suite.Require().NotEmpty(sg721Address)
}

func InstantiateSG721(ctx sdk.Context, msgServer wasmtypes.MsgServer, account sdk.AccAddress, codeID uint64) (string, error) {
	instantiateMsgRaw := []byte(
		fmt.Sprintf(instantiateSG721Template,
			"Collection Name",
			"COL",
			account.String(),
			account.String(),
		),
	)

	instantiateRes, err := msgServer.InstantiateContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgInstantiateContract{
		Sender: account.String(),
		Admin:  account.String(),
		CodeID: codeID,
		Label:  "SG721",
		Msg:    instantiateMsgRaw,
		Funds:  sdk.NewCoins(sdk.NewInt64Coin("ustars", 1_000_000_000)),
	})
	if err != nil {
		return "", err
	}
	return instantiateRes.Address, nil
}

type RoyaltiesInfo struct {
}

func InstantiateMarketplace(ctx sdk.Context, msgServer wasmtypes.MsgServer, account sdk.AccAddress, codeID uint64) (string, error) {
	instantiateMsgRaw := []byte(
		fmt.Sprintf(instantiateSG721Template,
			"Collection Name",
			"COL",
			account.String(),
			account.String(),
		),
	)

	instantiateRes, err := msgServer.InstantiateContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgInstantiateContract{
		Sender: account.String(),
		Admin:  account.String(),
		CodeID: codeID,
		Label:  "SG721",
		Msg:    instantiateMsgRaw,
		Funds:  sdk.NewCoins(sdk.NewInt64Coin("ustars", 1_000_000_000)),
	})
	if err != nil {
		return "", err
	}
	return instantiateRes.Address, nil
}
