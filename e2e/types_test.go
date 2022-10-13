package e2e_test

type CollectionInfo struct {
	Creator          string       `json:"creator"`
	Description      string       `json:"description"`
	Image            string       `json:"image"`
	ExternalLink     *string      `json:"external_link"`
	ExplicitContent  *bool        `json:"explicit_content"`
	StartTradingTime *string      `json:"start_trading_time,omitempty"`
	RoyaltyInfo      *RoyaltyInfo `json:"royalty_info,omitempty"`
}
type SG721InstantiateMsg struct {
	Name           string         `json:"name"`
	Symbol         string         `json:"symbol"`
	Minter         string         `json:"minter"`
	CollectionInfo CollectionInfo `json:"collection_info"`
}

type RoyaltyInfo struct {
	PaymentAddress string `json:"payment_address"`
	Share          string `json:"share"`
}

type Coin struct {
	Amount string `json:"amount"`
	Denom  string `json:"denom"`
}

type FactoryExtension struct {
	MaxTokenLimit      int  `json:"max_token_limit"`
	MaxPerAddressLimit int  `json:"max_per_address_limit"`
	AirdropMintPrice   Coin `json:"airdrop_mint_price"`
	AirdropMintFeeBPS  int  `json:"airdrop_mint_fee_bps"`
	ShuffleFee         Coin `json:"shuffle_fee"`
}
type FactoryParams struct {
	CodeID               uint64           `json:"code_id"`
	CreationFee          Coin             `json:"creation_fee"`
	MinMintPrice         Coin             `json:"min_mint_price"`
	MinFeeBPS            int              `json:"mint_fee_bps"`
	MaxTradingOffsetSecs int              `json:"max_trading_offset_secs"`
	Extension            FactoryExtension `json:"extension"`
}
type InstantiateFactoryMsg struct {
	Params FactoryParams `json:"params"`
}

type VendingMinterInitMsgExtension struct {
	BaseTokenURI    string  `json:"base_token_uri"`
	PaymentAddress  *string `json:"payment_address,omitempty"`
	StartTime       string  `json:"start_time"`
	NumTokens       int     `json:"num_tokens"`
	MintPrice       Coin    `json:"mint_price"`
	PerAddressLimit int     `json:"per_address_limit"`
	Whitelist       *string `json:"whitelist,omitempty"`
}

type CreateMinterMsg struct {
	InitMsg          VendingMinterInitMsgExtension `json:"init_msg"`
	CollectionParams CollectionParams              `json:"collection_params"`
}
type FactoryMessages struct {
	CreateMinter *CreateMinterMsg `json:"create_minter"`
}
type CollectionParams struct {
	CodeID uint64         `json:"code_id"`
	Name   string         `json:"name"`
	Symbol string         `json:"symbol"`
	Info   CollectionInfo `json:"info"`
}
