package e2e_test

import (
	"encoding/json"

	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

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
