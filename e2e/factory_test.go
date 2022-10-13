package e2e_test

import (
	"encoding/json"
	"fmt"
	"os"
	"time"

	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

const CREATION_FEE = 1_000_000_000
const MINT_PRICE = 100_000_000

func (suite *MarketplaceTestSuite) TestInstantiateFactory() {
	ctx, _ := suite.parentCtx.CacheContext()
	creator := suite.accounts[0]
	_, err := InstantiateFactory(ctx, suite.msgServer, creator.Address, suite.factoryCodeID, suite.minterCodeID)
	suite.Require().NoError(err)
}

func CreateMinterMessage(creator sdk.AccAddress, sg721CodeID uint64, start time.Time, numTokens, limit int, startTradingTime *time.Time) *CreateMinterMsg {
	var startTradingTimeStr *string
	if startTradingTime != nil {
		startTradingTimeStr = strPtr(fmt.Sprintf("%d", startTradingTime.UnixNano()))
	}
	return &CreateMinterMsg{
		InitMsg: VendingMinterInitMsgExtension{
			BaseTokenURI:    "ipfs://...",
			PaymentAddress:  strPtr(creator.String()),
			StartTime:       fmt.Sprintf("%d", start.UnixNano()),
			NumTokens:       numTokens,
			MintPrice:       coin(MINT_PRICE),
			PerAddressLimit: limit,
		},
		CollectionParams: CollectionParams{
			CodeID: sg721CodeID,
			Name:   "Collection",
			Symbol: "SYM",

			Info: CollectionInfo{
				Creator:          creator.String(),
				Description:      "Description",
				Image:            "https://example.com/image.png",
				StartTradingTime: startTradingTimeStr,
			},
		},
	}
}
func (suite *MarketplaceTestSuite) TestCreateMinter() {
	if os.Getenv("10K_ENABLED") == "" {
		suite.T().Skip()
	}

	ctx, _ := suite.parentCtx.CacheContext()
	creator := suite.accounts[0]
	factoryAddress, err := InstantiateFactory(ctx, suite.msgServer, creator.Address, suite.factoryCodeID, suite.minterCodeID)
	suite.Require().NoError(err)
	suite.Require().NotEmpty(factoryAddress)

	createMinterMsg := FactoryMessages{
		CreateMinter: CreateMinterMessage(creator.Address, suite.sg721CodeID, suite.startTime, 10_000, 50, nil),
	}
	bz, err := json.Marshal(&createMinterMsg)
	suite.Require().NoError(err)

	// requires fee
	_, err = suite.msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: factoryAddress,
		Sender:   creator.Address.String(),
		Msg:      bz,
	})
	suite.Require().Error(err)

	// requires exact fee
	_, err = suite.msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: factoryAddress,
		Sender:   creator.Address.String(),
		Msg:      bz,
		Funds:    sdk.NewCoins(sdk.NewInt64Coin("ustars", 50_000_000)),
	})
	suite.Require().Error(err)

	_, err = suite.msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: factoryAddress,
		Sender:   creator.Address.String(),
		Msg:      bz,
		Funds:    sdk.NewCoins(sdk.NewInt64Coin("ustars", CREATION_FEE)),
	})

	suite.Require().NoError(err)

	events := FindEventsByType(ctx.EventManager().Events(), "instantiate")
	suite.Require().NotEmpty(events)

	minter, ok := FindAttributeByKey(events[1], "_contract_address")
	suite.Require().True(ok)

	sg721, ok := FindAttributeByKey(events[2], "_contract_address")
	suite.Require().True(ok)
	suite.Require().NotEmpty(sg721)
	suite.Require().NoError(err)
	totalMints := 0
	mints := make(map[string]bool)
	for i := 0; i < 10_000; i++ {
		totalMints += 1
		evtManager := sdk.NewEventManager()
		newCtx := ctx.WithEventManager(evtManager)
		_, err = suite.msgServer.ExecuteContract(sdk.WrapSDKContext(newCtx), &wasmtypes.MsgExecuteContract{
			Contract: minter,
			Sender:   suite.accounts[(i%275)+1].Address.String(),
			Msg:      []byte(`{"mint":{}}`),
			Funds:    sdkCoins(MINT_PRICE),
		})
		evts := FindEventsByType(evtManager.Events(), "wasm")
		tokenId, _ := FindAttributeByKey(evts[0], "token_id")
		_, minted := mints[tokenId]
		suite.Require().False(minted)
		mints[tokenId] = true
		suite.Require().NoError(err)
	}
	for i := 1; i <= 10_000; i++ {
		_, minted := mints[fmt.Sprintf("%d", i)]
		suite.Require().True(minted)
	}
	suite.Require().Equal(totalMints, 10_000)
	// mint nft
	_, err = suite.msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: minter,
		Sender:   suite.accounts[280].Address.String(),
		Msg:      []byte(`{"mint":{}}`),
		Funds:    sdkCoins(MINT_PRICE),
	})

	suite.Require().Error(err)
	suite.Require().Contains(err.Error(), "Sold out")

}

func InstantiateFactory(ctx sdk.Context, msgServer wasmtypes.MsgServer,
	account sdk.AccAddress, codeID, vendingMinterCodeID uint64) (string, error) {
	instantiate := InstantiateFactoryMsg{
		Params: FactoryParams{
			CodeID:               vendingMinterCodeID,
			CreationFee:          coin(CREATION_FEE),
			MinMintPrice:         coin(50),
			MinFeeBPS:            1000, // 10%
			MaxTradingOffsetSecs: (60 * 60) * 24,
			Extension: FactoryExtension{
				MaxTokenLimit:      10_000,
				MaxPerAddressLimit: 50,
				AirdropMintPrice:   coin(0),
				AirdropMintFeeBPS:  0,
				ShuffleFee:         coin(500_000_000),
			},
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
		Label:  "Vending Factory",
		Msg:    instantiateMsgRaw,
		Funds:  sdk.NewCoins(),
	})
	if err != nil {
		return "", err
	}
	return instantiateRes.Address, nil
}

func (suite *MarketplaceTestSuite) TestStartTradingTime() {

	ctx, _ := suite.parentCtx.CacheContext()
	creator := suite.accounts[0]
	factoryAddress, err := InstantiateFactory(ctx, suite.msgServer, creator.Address, suite.factoryCodeID, suite.minterCodeID)
	suite.Require().NoError(err)
	suite.Require().NotEmpty(factoryAddress)

	invalidTime := suite.startTime.Add(time.Hour * 1)
	createMinterMsg := FactoryMessages{
		CreateMinter: CreateMinterMessage(creator.Address, suite.sg721CodeID, suite.startTime, 1000, 10, &invalidTime),
	}
	bz, err := json.Marshal(&createMinterMsg)
	fmt.Println(string(bz))
	suite.Require().NoError(err)

	_, err = suite.msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: factoryAddress,
		Sender:   creator.Address.String(),
		Msg:      bz,
		Funds:    sdk.NewCoins(sdk.NewInt64Coin("ustars", CREATION_FEE)),
	})

	suite.Require().NoError(err)

	events := FindEventsByType(ctx.EventManager().Events(), "instantiate")
	suite.Require().NotEmpty(events)

	minter, ok := FindAttributeByKey(events[1], "_contract_address")
	suite.Require().True(ok)

	sg721, ok := FindAttributeByKey(events[2], "_contract_address")
	suite.Require().True(ok)
	suite.Require().NotEmpty(sg721)
	suite.Require().NoError(err)

	for i := 0; i < 100; i++ {
		_, err = suite.msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
			Contract: minter,
			Sender:   suite.accounts[(i%275)+1].Address.String(),
			Msg:      []byte(`{"mint":{}}`),
			Funds:    sdkCoins(MINT_PRICE),
		})
		suite.Require().NoError(err)
	}

}

func (suite *MarketplaceTestSuite) TestInvalidstartTradingTime() {
	ctx, _ := suite.parentCtx.CacheContext()
	creator := suite.accounts[0]
	factoryAddress, err := InstantiateFactory(ctx, suite.msgServer, creator.Address, suite.factoryCodeID, suite.minterCodeID)
	suite.Require().NoError(err)
	suite.Require().NotEmpty(factoryAddress)
	invalidTime := suite.startTime.Add(time.Hour * 24 * 365)
	createMinterMsg := FactoryMessages{
		CreateMinter: CreateMinterMessage(creator.Address, suite.sg721CodeID, suite.startTime, 1000, 10, &invalidTime),
	}
	bz, err := json.Marshal(&createMinterMsg)
	suite.Require().NoError(err)

	_, err = suite.msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: factoryAddress,
		Sender:   creator.Address.String(),
		Msg:      bz,
		Funds:    sdk.NewCoins(sdk.NewInt64Coin("ustars", CREATION_FEE)),
	})
	suite.Require().Error(err)
}
