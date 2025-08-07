use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenValue {
    pub address: String,
    pub amount: BigDecimal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AssetType {
    XRP(BigDecimal),
    Token(TokenValue),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwapParams {
    pub token_in: AssetType,
    pub token_out: AssetType,
    pub token_in_min_amount: BigDecimal,
    pub token_out_min_amount: BigDecimal,
}