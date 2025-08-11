use bigdecimal::{BigDecimal, ToPrimitive};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
use tracing::info;
use xrpl_types::{AccountId, Amount, CurrencyCode, DropsAmount, IssuedAmount, IssuedValue};

use crate::xrpl_http::ClientService;

/// Request structure for token swaps on XRPL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapRequest {
    /// Token to send (use "XRP" for native XRP)
    pub token_in: String,
    /// Token to receive (use "XRP" for native XRP)
    pub token_out: String,
    /// Amount to send (as string to preserve precision)
    pub amount_in: String,
    /// Minimum amount to receive (as string to preserve precision)
    pub amount_out_min: String,
}

impl SwapRequest {
    /// Create a new swap request
    pub fn new(
        token_in: String,
        token_out: String,
        amount_in: String,
        amount_out_min: String,
    ) -> Self {
        Self {
            token_in,
            token_out,
            amount_in,
            amount_out_min,
        }
    }

    /// Check if this is an XRP-to-token swap
    pub fn is_xrp_to_token(&self) -> bool {
        self.token_in == "XRP" && self.token_out != "XRP"
    }

    /// Check if this is a token-to-XRP swap
    pub fn is_token_to_xrp(&self) -> bool {
        self.token_in != "XRP" && self.token_out == "XRP"
    }

    /// Check if this is a token-to-token swap
    pub fn is_token_to_token(&self) -> bool {
        self.token_in != "XRP" && self.token_out != "XRP"
    }

    /// Check if this is an XRP-to-XRP swap (which doesn't make sense)
    pub fn is_xrp_to_xrp(&self) -> bool {
        self.token_in == "XRP" && self.token_out == "XRP"
    }

    fn parse_issued_value(&self, amount_str: &str) -> Result<IssuedValue, String> {
        let value = BigDecimal::from_str(amount_str)
            .map_err(|e| format!("Invalid amount format: {e}"))?;

        let (value_big_int, scale) = value.into_bigint_and_scale();

        let mantissa = value_big_int
            .to_i64()
            .ok_or("Amount too large for mantissa conversion")?;

        let exponent = -(scale as i8);

        info!(
            "Parsed amount - mantissa: {}, exponent: {}",
            mantissa, exponent
        );

        IssuedValue::from_mantissa_exponent(mantissa, exponent)
            .map_err(|e| format!("Failed to create issued value: {e}"))
    }

    pub async fn get_max_amount_out(&self) -> Result<Amount, String> {
        let client_service = ClientService::new();
        let base_amount_out = "1000000000";
        if self.token_out == "XRP" {
            let xrp_amount = BigDecimal::from_str(base_amount_out)
                .map_err(|e| format!("Invalid XRP amount: {e}"))?;

            let drops = xrp_amount
                * BigDecimal::from_str("1000000")
                    .map_err(|e| format!("Failed to calculate drops: {e}"))?;

            let drops_u64 = drops
                .to_u64()
                .ok_or("Amount too large for drops conversion")?;

            let drops_amount = DropsAmount::from_drops(drops_u64)
                .map_err(|e| format!("Invalid drops amount: {e}"))?;

            Ok(Amount::Drops(drops_amount))
        } else {
            let currencies = client_service
                .get_account_currencies(&self.token_out)
                .await?;

            if currencies.receive_currencies.is_empty() {
                return Err(format!("No currencies found for token: {}", self.token_out));
            }

            let currency_code = &currencies.receive_currencies[0];
            let currency = CurrencyCode::from_str(currency_code)
                .map_err(|e| format!("Invalid currency code: {e}"))?;

            let token = &self.token_out.clone();
            let issued_value = self.parse_issued_value(base_amount_out)?;

            let token_id = AccountId::from_address(token)
                .map_err(|e| format!("Invalid token address: {e}"))?;

            let issued_amount = IssuedAmount::from_issued_value(issued_value, currency, token_id)
                .map_err(|e| format!("Failed to create issued amount: {e}"))?;

            Ok(Amount::Issued(issued_amount))
        }
    }

    pub async fn get_send_max(&self) -> Result<Amount, String> {
        let client_service = ClientService::new();
        if self.token_in == "XRP" {
            let xrp_amount = BigDecimal::from_str(&self.amount_in)
                .map_err(|e| format!("Invalid XRP amount: {e}"))?;

            let drops = xrp_amount
                * BigDecimal::from_str("1000000")
                    .map_err(|e| format!("Failed to calculate drops: {e}"))?;

            let drops_u64 = drops
                .to_u64()
                .ok_or("Amount too large for drops conversion")?;

            let drops_amount = DropsAmount::from_drops(drops_u64)
                .map_err(|e| format!("Invalid drops amount: {e}"))?;

            Ok(Amount::Drops(drops_amount))
        } else {
            let currencies = client_service
                .get_account_currencies(&self.token_in)
                .await?;

            if currencies.receive_currencies.is_empty() {
                return Err(format!("No currencies found for token: {}", self.token_in));
            }

            let currency_code = &currencies.receive_currencies[0];
            let currency = CurrencyCode::from_str(currency_code)
                .map_err(|e| format!("Invalid currency code: {e}"))?;

            let issued_value = self.parse_issued_value(&self.amount_in)?;

            let token_id = AccountId::from_address(&self.token_in.to_string())
                .map_err(|e| format!("Invalid token address: {e}"))?;

            let issued_amount = IssuedAmount::from_issued_value(issued_value, currency, token_id)
                .map_err(|e| format!("Failed to create issued amount: {e}"))?;

            Ok(Amount::Issued(issued_amount))
        }
    }

    pub async fn get_receive_min(&self) -> Result<Amount, String> {
        let client_service = ClientService::new();
        if self.token_out == "XRP" {
            let xrp_amount = BigDecimal::from_str(&self.amount_out_min)
                .map_err(|e| format!("Invalid XRP amount: {e}"))?;

            let drops = xrp_amount
                * BigDecimal::from_str("1000000")
                    .map_err(|e| format!("Failed to calculate drops: {e}"))?;

            let drops_u64 = drops
                .to_u64()
                .ok_or("Amount too large for drops conversion")?;

            let drops_amount = DropsAmount::from_drops(drops_u64)
                .map_err(|e| format!("Invalid drops amount: {e}"))?;

            Ok(Amount::Drops(drops_amount))
        } else {
            let currencies = client_service
                .get_account_currencies(&self.token_out)
                .await?;

            if currencies.receive_currencies.is_empty() {
                return Err(format!("No currencies found for token: {}", self.token_out));
            }

            let currency_code = &currencies.receive_currencies[0];
            let currency = CurrencyCode::from_str(currency_code)
                .map_err(|e| format!("Invalid currency code: {e}"))?;

            let issued_value = self.parse_issued_value(&self.amount_out_min)?;

            let token_id = AccountId::from_address(&self.token_out.to_string())
                .map_err(|e| format!("Invalid token address: {e}"))?;

            let issued_amount = IssuedAmount::from_issued_value(issued_value, currency, token_id)
                .map_err(|e| format!("Failed to create issued amount: {e}"))?;

            Ok(Amount::Issued(issued_amount))
        }
    }

    /// Validate the swap request
    pub fn validate(&self) -> Result<(), SwapError> {
        if self.is_xrp_to_xrp() {
            return Err(SwapError::InvalidSwap("Cannot swap XRP to XRP".to_string()));
        }

        if self.token_in.is_empty() {
            return Err(SwapError::InvalidToken(
                "token_in cannot be empty".to_string(),
            ));
        }

        if self.token_out.is_empty() {
            return Err(SwapError::InvalidToken(
                "token_out cannot be empty".to_string(),
            ));
        }

        if self.amount_in.is_empty() {
            return Err(SwapError::InvalidAmount(
                "amount_in cannot be empty".to_string(),
            ));
        }

        if self.amount_out_min.is_empty() {
            return Err(SwapError::InvalidAmount(
                "amount_out_min cannot be empty".to_string(),
            ));
        }

        // Try to parse amounts to validate they're numeric
        self.amount_in.parse::<f64>().map_err(|_| {
            SwapError::InvalidAmount("amount_in must be a valid number".to_string())
        })?;

        self.amount_out_min.parse::<f64>().map_err(|_| {
            SwapError::InvalidAmount("amount_out_min must be a valid number".to_string())
        })?;

        Ok(())
    }
}

/// Request structure for creating trust lines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustLineRequest {
    /// Token address to create trust line for
    pub token_address: String,
    /// Optional limit for the trust line (defaults to a reasonable amount)
    pub limit: Option<String>,
}

impl TrustLineRequest {
    /// Create a new trust line request
    pub fn new(token_address: String, limit: Option<String>) -> Self {
        Self {
            token_address,
            limit,
        }
    }

    /// Create a new trust line request with default limit
    pub fn with_default_limit(token_address: String) -> Self {
        Self::new(token_address, None)
    }
}

/// Errors that can occur during swap operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SwapError {
    InvalidSwap(String),
    InvalidToken(String),
    InvalidAmount(String),
    NetworkError(String),
    TransactionError(String),
}

impl fmt::Display for SwapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SwapError::InvalidSwap(msg) => write!(f, "Invalid swap: {msg}"),
            SwapError::InvalidToken(msg) => write!(f, "Invalid token: {msg}"),
            SwapError::InvalidAmount(msg) => write!(f, "Invalid amount: {msg}"),
            SwapError::NetworkError(msg) => write!(f, "Network error: {msg}"),
            SwapError::TransactionError(msg) => write!(f, "Transaction error: {msg}"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FulfillmentDetails {
    pub amount_out: String,
    pub token_out: String,
    pub amount_in: String,
    pub token_in: String,
    pub fee: String,
    pub tx_signer: String,
    pub tx_timestamp: u64,
}
