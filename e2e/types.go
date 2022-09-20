package e2e_test

type CollectionInfo struct {
	Creator          string       `json:"creator"`
	Description      string       `json:"description"`
	Image            string       `json:"image"`
	ExternalLink     *string      `json:"external_link"`
	TradingStartTime *int64       `json:"trading_start_time"`
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
