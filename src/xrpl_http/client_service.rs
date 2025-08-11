use libsecp256k1::PublicKey;
use tracing::{info, warn};
use xrpl_http_client::{
    AccountCurrenciesRequest, AccountCurrenciesResponse, AccountInfoRequest, AccountInfoResponse,
    AccountLinesRequest, AccountLinesResponse, Client,
    TxRequest, TxResponse,
};

use crate::xrpl_http::types::FulfillmentDetails;

/// Service for read-only XRPL operations that only require HTTP client interactions
pub struct ClientService {
    client: Client,
}

impl ClientService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Get the currencies that an account can receive
    pub async fn get_account_currencies(
        &self,
        address: &str,
    ) -> Result<AccountCurrenciesResponse, String> {
        let req = AccountCurrenciesRequest::new(address);

        info!("Getting account currencies for address: {}", address);
        let response = self
            .client
            .call(req)
            .await
            .map_err(|e| format!("Failed to get account currencies: {e}"))?;

        Ok(response)
    }

    /// Get account information including balance and sequence number
    pub async fn get_account_info(&self, address: &str) -> Result<AccountInfoResponse, String> {
        let req = AccountInfoRequest::new(address);

        info!("Getting account info for address: {}", address);
        let response = self
            .client
            .call(req)
            .await
            .map_err(|e| format!("Failed to get account info: {e}"))?;

        Ok(response)
    }

    /// Get account trust lines
    pub async fn get_account_lines(&self, address: &str) -> Result<AccountLinesResponse, String> {
        let req = AccountLinesRequest::new(address);

        info!("Getting account lines for address: {}", address);
        let response = self
            .client
            .call(req)
            .await
            .map_err(|e| format!("Failed to get account lines: {e}"))?;

        Ok(response)
    }

    pub async fn inspect_tx(&self, tx_hash: &str) -> Result<TxResponse, String> {
        let req = TxRequest::new(tx_hash);

        let response = self
            .client
            .call(req)
            .await
            .map_err(|e| format!("Failed to inspect transaction: {e}"))?;

        Ok(response)
    }

    pub async fn balance_change(&self, tx_hash: &str) -> Result<FulfillmentDetails, String> {
        let tx_data = self.inspect_tx(tx_hash).await.unwrap().tx;

        match tx_data {
            xrpl_http_client::Transaction::Payment(payment_tx) => {
                let amount_out = payment_tx
                    .clone()
                    .common
                    .meta
                    .unwrap()
                    .delivered_amount
                    .unwrap()
                    .size();

                let fee = payment_tx.clone().common.fee;

                let amount_in = payment_tx.clone().send_max.unwrap().size();

                let (token_in, amount_in) = match payment_tx.clone().send_max.unwrap() {
                    xrpl_http_client::Amount::Drops(_) => {
                        ("XRP".to_string(), amount_in / 1000000.0)
                    }
                    xrpl_http_client::Amount::Issued(issued) => (issued.issuer, amount_in),
                };

                let (token_out, amount_out) = match payment_tx.clone().amount {
                    xrpl_http_client::Amount::Drops(_) => {
                        ("XRP".to_string(), amount_out / 1000000.0)
                    }
                    xrpl_http_client::Amount::Issued(issued) => (issued.issuer, amount_out),
                };
                
                let xrp_first_epoch_timestamp = 946684800;
                
                let tx_timestamp = payment_tx.clone().common.date.map(|d| d as u64 + xrp_first_epoch_timestamp).unwrap_or(xrp_first_epoch_timestamp);
                
                let details = FulfillmentDetails {
                    amount_out: amount_out.to_string(),
                    token_out,
                    amount_in: amount_in.to_string(),
                    token_in,
                    fee: fee.to_string(),
                    tx_signer: payment_tx.clone().common.account,
                    tx_timestamp
                };

                Ok(details)
            }
            _ => {
                warn!("Not a payment tx");
                Err("Not a payment tx".to_string())
            }
        }
    }

    /// Check if an account exists on the ledger
    pub async fn account_exists(&self, address: &str) -> Result<bool, String> {
        match self.get_account_info(address).await {
            Ok(_) => Ok(true),
            Err(e) if e.contains("actNotFound") => Ok(false),
            Err(e) => Err(e),
        }
    }
}

impl Default for ClientService {
    fn default() -> Self {
        Self::new()
    }
}
