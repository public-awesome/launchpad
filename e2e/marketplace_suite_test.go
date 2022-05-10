package e2e_test

import (
	"encoding/json"
	"fmt"
	"io"
	"io/ioutil"
	"net/http"
	"strings"
	"sync"
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

type storeCache struct {
	sync.Mutex
	contracts map[string][]byte
}

var contractsCache = storeCache{contracts: make(map[string][]byte)}

func GetContractBytes(contract string) ([]byte, error) {
	contractsCache.Lock()
	bz, found := contractsCache.contracts[contract]
	contractsCache.Unlock()
	if found {
		return bz, nil
	}
	contractsCache.Lock()
	defer contractsCache.Unlock()
	if strings.HasPrefix(contract, "https://") {
		resp, err := http.Get(contract)
		if err != nil {
			return nil, err
		}
		defer resp.Body.Close()
		bz, err = io.ReadAll(resp.Body)
		if err != nil {
			return nil, err
		}
	} else {
		var err error
		bz, err = ioutil.ReadFile(fmt.Sprintf("contracts/%s", contract))
		if err != nil {
			return nil, err
		}
	}
	contractsCache.contracts[contract] = bz
	return bz, nil
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
	admin := suite.accounts[0]
	creator := suite.accounts[1]
	creatorRoyaltyAccount := suite.accounts[2]
	seller := suite.accounts[3]
	buyer := suite.accounts[4]

	royalties := &RoyaltyInfo{
		PaymentAddress: creatorRoyaltyAccount.Address.String(),
		Share:          "0.1",
	}
	collectionAddress, err := InstantiateSG721(ctx, suite.msgServer, creator.Address, suite.sg721CodeID, royalties)
	suite.Require().NoError(err)
	suite.Require().NotEmpty(collectionAddress)

	marketplaceAddress, err := InstantiateMarketplace(ctx, suite.msgServer, admin.Address, suite.marketplaceCodeID)
	suite.Require().NoError(err)
	suite.Require().NotEmpty(marketplaceAddress)

	// mint nft
	executeMsgRaw := fmt.Sprintf(executeMintTemplate,
		1,
		seller.Address.String(),
	)
	_, err = suite.msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: collectionAddress,
		Sender:   creator.Address.String(),
		Msg:      []byte(executeMsgRaw),
	})
	suite.Require().NoError(err)

	// approve the NFT
	executeMsgRaw = fmt.Sprintf(executeApproveTemplate,
		marketplaceAddress,
		1,
	)
	_, err = suite.msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: collectionAddress,
		Sender:   seller.Address.String(),
		Msg:      []byte(executeMsgRaw),
	})
	suite.Require().NoError(err)

	// execute an ask on the marketplace
	expires := suite.startTime.Add(time.Hour * 24 * 30)
	executeMsgRaw = fmt.Sprintf(executeAskTemplate,
		"fixed_price",
		collectionAddress,
		1,
		1_000_000_000,
		expires.UnixNano(),
	)
	_, err = suite.msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: marketplaceAddress,
		Sender:   seller.Address.String(),
		Msg:      []byte(executeMsgRaw),
	})
	suite.Require().NoError(err)

	// check intial balance of buyer
	balance := suite.app.BankKeeper.GetBalance(ctx, buyer.Address, "ustars")
	suite.Require().Equal(
		"2000000000",
		balance.Amount.String(),
	)

	// check intial balance of seller
	balance = suite.app.BankKeeper.GetBalance(ctx, buyer.Address, "ustars")
	suite.Require().Equal(
		"2000000000",
		balance.Amount.String(),
	)

	// execute a bid on the marketplace
	executeMsgRaw = fmt.Sprintf(executeBidTemplate,
		collectionAddress,
		1,
		expires.UnixNano(),
	)
	_, err = suite.msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: marketplaceAddress,
		Sender:   buyer.Address.String(),
		Msg:      []byte(executeMsgRaw),
		Funds:    sdk.NewCoins(sdk.NewInt64Coin("ustars", 1_000_000_000)),
	})

	// buy should be executed without error
	suite.Require().NoError(err)

	// buyer should have 1k less
	balance = suite.app.BankKeeper.GetBalance(ctx, buyer.Address, "ustars")
	suite.Require().Equal(
		"1000000000",
		balance.Amount.String(),
	)

	// creator payment royalty account should have 10% of the sales
	balance = suite.app.BankKeeper.GetBalance(ctx, creatorRoyaltyAccount.Address, "ustars")
	// 2,000 initial + 100 (10% of the sell)
	suite.Require().Equal(
		"2100000000",
		balance.Amount.String(),
	)

	// seller should have 88% of the sale
	balance = suite.app.BankKeeper.GetBalance(ctx, seller.Address, "ustars")
	// 2,000 initial + 880 (88% of the sell)
	suite.Require().Equal(
		"2880000000",
		balance.Amount.String(),
	)

}

func (suite *MarketplaceTestSuite) TestFundsRecipient() {
	ctx, _ := suite.parentCtx.CacheContext()
	admin := suite.accounts[0]
	creator := suite.accounts[1]
	creatorRoyaltyAccount := suite.accounts[2]
	seller := suite.accounts[3]
	buyer := suite.accounts[4]
	recipient := suite.accounts[5]

	royalties := &RoyaltyInfo{
		PaymentAddress: creatorRoyaltyAccount.Address.String(),
		Share:          "0.1",
	}
	collectionAddress, err := InstantiateSG721(ctx, suite.msgServer, creator.Address, suite.sg721CodeID, royalties)
	suite.Require().NoError(err)
	suite.Require().NotEmpty(collectionAddress)

	marketplaceAddress, err := InstantiateMarketplace(ctx, suite.msgServer, admin.Address, suite.marketplaceCodeID)
	suite.Require().NoError(err)
	suite.Require().NotEmpty(marketplaceAddress)

	// mint nft
	executeMsgRaw := fmt.Sprintf(executeMintTemplate,
		1,
		seller.Address.String(),
	)
	_, err = suite.msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: collectionAddress,
		Sender:   creator.Address.String(),
		Msg:      []byte(executeMsgRaw),
	})
	suite.Require().NoError(err)

	// approve the NFT
	executeMsgRaw = fmt.Sprintf(executeApproveTemplate,
		marketplaceAddress,
		1,
	)
	_, err = suite.msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: collectionAddress,
		Sender:   seller.Address.String(),
		Msg:      []byte(executeMsgRaw),
	})
	suite.Require().NoError(err)

	// execute an ask on the marketplace
	expires := suite.startTime.Add(time.Hour * 24 * 30)
	askMsg := SetAskMsg{
		SaleType:   "fixed_price",
		Collection: collectionAddress,
		TokenID:    1,
		Price: AskPrice{
			Denom:  "ustars",
			Amount: "1000000000",
		},
		Expires:        fmt.Sprintf("%d", expires.UnixNano()),
		FundsRecipient: stringPtr(recipient.Address.String()),
	}
	marketPlaceMsg := &MarketPlaceMsg{
		SetAsk: &askMsg,
	}
	askMsgRaw, err := json.Marshal(marketPlaceMsg)
	suite.Require().NoError(err)
	_, err = suite.msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: marketplaceAddress,
		Sender:   seller.Address.String(),
		Msg:      askMsgRaw,
	})
	suite.Require().NoError(err)

	// check intial balance of buyer
	balance := suite.app.BankKeeper.GetBalance(ctx, buyer.Address, "ustars")
	suite.Require().Equal(
		"2000000000",
		balance.Amount.String(),
	)

	// check intial balance of seller
	balance = suite.app.BankKeeper.GetBalance(ctx, buyer.Address, "ustars")
	suite.Require().Equal(
		"2000000000",
		balance.Amount.String(),
	)

	// execute a bid on the marketplace
	executeMsgRaw = fmt.Sprintf(executeBidTemplate,
		collectionAddress,
		1,
		expires.UnixNano(),
	)
	_, err = suite.msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: marketplaceAddress,
		Sender:   buyer.Address.String(),
		Msg:      []byte(executeMsgRaw),
		Funds:    sdk.NewCoins(sdk.NewInt64Coin("ustars", 1_000_000_000)),
	})

	// buy should be executed without error
	suite.Require().NoError(err)

	// buyer should have 1k less
	balance = suite.app.BankKeeper.GetBalance(ctx, buyer.Address, "ustars")
	suite.Require().Equal(
		"1000000000",
		balance.Amount.String(),
	)

	// creator payment royalty account should have 10% of the sales
	balance = suite.app.BankKeeper.GetBalance(ctx, creatorRoyaltyAccount.Address, "ustars")
	// 2,000 initial + 100 (10% of the sell)
	suite.Require().Equal(
		"2100000000",
		balance.Amount.String(),
	)

	// seller should have 88% of the sale sent to funds_recipient address
	balance = suite.app.BankKeeper.GetBalance(ctx, recipient.Address, "ustars")
	// 2,000 initial + 880 (88% of the sell)
	suite.Require().Equal(
		"2880000000",
		balance.Amount.String(),
	)

	// original listing address should not get anything because was sent to funds recipient address
	balance = suite.app.BankKeeper.GetBalance(ctx, seller.Address, "ustars")
	// 2,000 initial
	suite.Require().Equal(
		"2000000000",
		balance.Amount.String(),
	)

}

func stringPtr(s string) *string {
	return &s
}
func (suite *MarketplaceTestSuite) TestFindersFeesWithRoyalties() {
	ctx, _ := suite.parentCtx.CacheContext()
	admin := suite.accounts[0]
	creator := suite.accounts[1]
	creatorRoyaltyAccount := suite.accounts[2]
	seller := suite.accounts[3]
	buyer := suite.accounts[4]
	finder := suite.accounts[5]
	findersFee := 100 // 1%

	royalties := &RoyaltyInfo{
		PaymentAddress: creatorRoyaltyAccount.Address.String(),
		Share:          "0.1",
	}
	collectionAddress, err := InstantiateSG721(ctx, suite.msgServer, creator.Address, suite.sg721CodeID, royalties)
	suite.Require().NoError(err)
	suite.Require().NotEmpty(collectionAddress)

	marketplaceAddress, err := InstantiateMarketplace(ctx, suite.msgServer, admin.Address, suite.marketplaceCodeID)
	suite.Require().NoError(err)
	suite.Require().NotEmpty(marketplaceAddress)

	// mint nft
	executeMsgRaw := fmt.Sprintf(executeMintTemplate,
		1,
		seller.Address.String(),
	)
	_, err = suite.msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: collectionAddress,
		Sender:   creator.Address.String(),
		Msg:      []byte(executeMsgRaw),
	})
	suite.Require().NoError(err)

	// approve the NFT
	executeMsgRaw = fmt.Sprintf(executeApproveTemplate,
		marketplaceAddress,
		1,
	)
	_, err = suite.msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: collectionAddress,
		Sender:   seller.Address.String(),
		Msg:      []byte(executeMsgRaw),
	})
	suite.Require().NoError(err)

	// execute an ask on the marketplace
	expires := suite.startTime.Add(time.Hour * 24 * 30)
	askMsg := SetAskMsg{
		SaleType:   "fixed_price",
		Collection: collectionAddress,
		TokenID:    1,
		Price: AskPrice{
			Denom:  "ustars",
			Amount: "1000000000",
		},
		Expires:    fmt.Sprintf("%d", expires.UnixNano()),
		FindersFee: &findersFee,
	}
	marketPlaceMsg := &MarketPlaceMsg{
		SetAsk: &askMsg,
	}
	askMsgRaw, err := json.Marshal(marketPlaceMsg)
	suite.Require().NoError(err)
	_, err = suite.msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: marketplaceAddress,
		Sender:   seller.Address.String(),
		Msg:      askMsgRaw,
	})
	suite.Require().NoError(err)

	// check intial balance of buyer
	balance := suite.app.BankKeeper.GetBalance(ctx, buyer.Address, "ustars")
	suite.Require().Equal(
		"2000000000",
		balance.Amount.String(),
	)

	// check intial balance of seller
	balance = suite.app.BankKeeper.GetBalance(ctx, buyer.Address, "ustars")
	suite.Require().Equal(
		"2000000000",
		balance.Amount.String(),
	)

	// check intial balance of finder
	balance = suite.app.BankKeeper.GetBalance(ctx, finder.Address, "ustars")
	suite.Require().Equal(
		"2000000000",
		balance.Amount.String(),
	)

	// execute a bid on the marketplace
	setBidMsg, err := json.Marshal(&MarketPlaceMsg{
		SetBid: &SetBidMsg{
			Collection: collectionAddress,
			TokenID:    1,
			Expires:    fmt.Sprintf("%d", expires.UnixNano()),
			Finder:     stringPtr(finder.Address.String()),
		},
	})
	suite.Require().NoError(err)
	_, err = suite.msgServer.ExecuteContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgExecuteContract{
		Contract: marketplaceAddress,
		Sender:   buyer.Address.String(),
		Msg:      setBidMsg,
		Funds:    sdk.NewCoins(sdk.NewInt64Coin("ustars", 1_000_000_000)),
	})

	// buy should be executed without error
	suite.Require().NoError(err)

	// buyer should have 1k less
	balance = suite.app.BankKeeper.GetBalance(ctx, buyer.Address, "ustars")
	suite.Require().Equal(
		"1000000000",
		balance.Amount.String(),
	)

	// creator payment royalty account should have 10% of the sales
	balance = suite.app.BankKeeper.GetBalance(ctx, creatorRoyaltyAccount.Address, "ustars")
	// 2,000 initial + 100 (10% of the sell)
	suite.Require().Equal(
		"2100000000",
		balance.Amount.String(),
	)

	// seller should have 87% of the sale
	balance = suite.app.BankKeeper.GetBalance(ctx, seller.Address, "ustars")
	// 2,000 initial + 870 (87% of the sell)
	suite.Require().Equal(
		"2870000000",
		balance.Amount.String(),
	)

	// finder should have 1% of the sale
	balance = suite.app.BankKeeper.GetBalance(ctx, finder.Address, "ustars")
	// 2,000 initial + 10 (1% of the sell)
	suite.Require().Equal(
		"2010000000",
		balance.Amount.String(),
	)

}

func InstantiateSG721(ctx sdk.Context, msgServer wasmtypes.MsgServer, account sdk.AccAddress, codeID uint64, royalties *RoyaltyInfo) (string, error) {
	instantiate := SG721InstantiateMsg{
		Name:   "Collection Name",
		Symbol: "COL",
		Minter: account.String(),
		CollectionInfo: CollectionInfo{
			Creator:     account.String(),
			Description: "Description",
			Image:       "https://example.com/image.png",
			RoyaltyInfo: royalties,
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
		Funds:  sdk.NewCoins(sdk.NewInt64Coin("ustars", 1_000_000_000)),
	})
	if err != nil {
		return "", err
	}
	return instantiateRes.Address, nil
}

func InstantiateMarketplace(ctx sdk.Context, msgServer wasmtypes.MsgServer, account sdk.AccAddress, codeID uint64) (string, error) {
	instantiateMsgRawString := fmt.Sprintf(instantiateMarketplaceTemplate,
		200,
		86400,
		15552000,
		86400,
		15552000,
		account.String(),
		500,
		5000000,
		15552000,
		500,
	)
	// instantiate marketplace
	instantiateRes, err := msgServer.InstantiateContract(sdk.WrapSDKContext(ctx), &wasmtypes.MsgInstantiateContract{
		Sender: account.String(),
		Admin:  account.String(),
		CodeID: codeID,
		Label:  "Marketplace",
		Msg:    []byte(instantiateMsgRawString),
	})
	if err != nil {
		return "", err
	}
	return instantiateRes.Address, nil
}

type RoyaltyInfo struct {
	PaymentAddress string `json:"payment_address"`
	Share          string `json:"share"`
}

type CollectionInfo struct {
	Creator     string       `json:"creator"`
	Description string       `json:"description"`
	Image       string       `json:"image"`
	RoyaltyInfo *RoyaltyInfo `json:"royalty_info,omitempty"`
}
type SG721InstantiateMsg struct {
	Name           string         `json:"name"`
	Symbol         string         `json:"symbol"`
	Minter         string         `json:"minter"`
	CollectionInfo CollectionInfo `json:"collection_info"`
}

type AskPrice struct {
	Amount string `json:"amount"`
	Denom  string `json:"denom"`
}
type SetAskMsg struct {
	SaleType       string   `json:"sale_type"`
	Collection     string   `json:"collection"`
	TokenID        int      `json:"token_id"`
	Price          AskPrice `json:"price"`
	Expires        string   `json:"expires"`
	FindersFee     *int     `json:"finders_fee_bps,omitempty"`
	Finder         *string  `json:"finder"`
	FundsRecipient *string  `json:"funds_recipient"`
}

type SetBidMsg struct {
	Collection string  `json:"collection"`
	TokenID    int     `json:"token_id"`
	Expires    string  `json:"expires"`
	FindersFee *string `json:"finders_fee_bps,omitempty"`
	Finder     *string `json:"finder"`
}
type MarketPlaceMsg struct {
	SetAsk *SetAskMsg `json:"set_ask,omitempty"`
	SetBid *SetBidMsg `json:"set_bid,omitempty"`
}
