use std::str::FromStr;
use tracing::info;
use xrpl_binary_codec::serialize;
use xrpl_http_client::{Client, SubmitRequest, SubmitResponse};
use xrpl_types::{
    AccountId, CurrencyCode, DropsAmount, IssuedAmount, IssuedValue, PaymentFlags,
    PaymentTransaction, Transaction, TrustSetTransaction,
};

use super::{client_service::ClientService, signer::RippleSigner, types::SwapRequest};

/// Service for transaction operations that require signing and submission
pub struct TransactionService {
    client: Client,
    client_service: ClientService,
    signer: RippleSigner,
}

impl TransactionService {
    /// Create a new transaction service from a seed string
    pub fn from_seed(seed_str: &str) -> Result<Self, String> {
        let signer = RippleSigner::from_seed(seed_str)?;
        let client = Client::new();
        let client_service = ClientService::new();

        Ok(Self {
            client,
            client_service,
            signer,
        })
    }

    /// Create a new transaction service with an existing signer
    pub fn new(signer: RippleSigner) -> Self {
        Self {
            client: Client::new(),
            client_service: ClientService::new(),
            signer,
        }
    }

    /// Get the account address
    pub fn address(&self) -> &str {
        self.signer.address()
    }

    /// Get a reference to the client service for read-only operations
    pub fn client_service(&self) -> &ClientService {
        &self.client_service
    }

    /// Execute a swap transaction
    pub async fn swap(&self, request: SwapRequest) -> Result<SubmitResponse, String> {
        let account_id = AccountId::from_address(self.signer.address())
            .map_err(|e| format!("Invalid account address: {e}"))?;

        let amount = request.get_max_amount_out().await.unwrap();

        // Create payment transaction
        let destination = account_id; // Self-payment for swaps
        let mut payment = PaymentTransaction::new(account_id, amount, destination);

        let deliver_min = request.get_receive_min().await.unwrap();
        let send_max = request.get_send_max().await.unwrap();
        payment.deliver_min = Some(deliver_min);
        payment.send_max = Some(send_max);
        payment.common.fee = Some(DropsAmount::from_drops(12).unwrap());
        payment.flags = PaymentFlags::PartialPayment.into();

        self.prepare_and_submit_transaction(payment).await
    }

    /// Create a trust line for a token
    pub async fn create_trust_line(
        &self,
        token_address: &str,
        limit: Option<&str>,
    ) -> Result<SubmitResponse, String> {
        info!("Creating trust line for token: {}", token_address);

        let currencies = self
            .client_service
            .get_account_currencies(token_address)
            .await?;

        if currencies.receive_currencies.is_empty() {
            return Err("No currencies found for the given address".to_string());
        }

        let currency_code = &currencies.receive_currencies[0];
        let account_id = AccountId::from_address(self.signer.address())
            .map_err(|e| format!("Invalid account address: {e}"))?;

        let limit_value = limit.unwrap_or("10000000");
        let limit_value = limit_value.parse::<i64>().unwrap();
        let issued_value = IssuedValue::from_mantissa_exponent(limit_value, 0).unwrap();

        let currency = CurrencyCode::from_str(currency_code)
            .map_err(|e| format!("Invalid currency code: {e}"))?;

        let issuer = AccountId::from_address(token_address)
            .map_err(|e| format!("Invalid token address: {e}"))?;

        let issued_amount = IssuedAmount::from_issued_value(issued_value, currency, issuer)
            .map_err(|e| format!("Failed to create issued amount: {e}"))?;

        let tx = TrustSetTransaction::new(account_id, issued_amount);

        self.prepare_and_submit_transaction(tx).await
    }

    /// Prepare, sign, and submit a transaction
    async fn prepare_and_submit_transaction<T>(
        &self,
        mut transaction: T,
    ) -> Result<SubmitResponse, String>
    where
        T: Transaction + Clone,
    {
        let address = self.signer.address.clone();
        let resp = self.client_service.get_account_info(&address).await?;

        let common_mut = transaction.common_mut();

        common_mut.sequence = Some(resp.account_data.sequence);

        self.client
            .prepare_transaction(common_mut)
            .await
            .map_err(|e| format!("Failed to prepare transaction: {e}"))?;

        self.signer.sign_transaction(&mut transaction)?;

        let tx_blob = serialize::serialize(&transaction)
            .map_err(|e| format!("Failed to serialize transaction: {e}"))?;

        let req = SubmitRequest::new(hex::encode(&tx_blob));
        let response = self
            .client
            .call(req)
            .await
            .map_err(|e| format!("Failed to submit transaction: {e}"))?;

        Ok(response)
    }

    /// Get account info using the internal client service
    pub async fn get_account_info(
        &self,
        address: Option<&str>,
    ) -> Result<xrpl_http_client::AccountInfoResponse, String> {
        let addr = address.unwrap_or(self.signer.address());
        self.client_service.get_account_info(addr).await
    }

    /// Get account currencies using the internal client service
    pub async fn get_account_currencies(
        &self,
        address: Option<&str>,
    ) -> Result<xrpl_http_client::AccountCurrenciesResponse, String> {
        let addr = address.unwrap_or(self.signer.address());
        self.client_service.get_account_currencies(addr).await
    }

    /// Get account lines using the internal client service
    pub async fn get_account_lines(
        &self,
        address: Option<&str>,
    ) -> Result<xrpl_http_client::AccountLinesResponse, String> {
        let addr = address.unwrap_or(self.signer.address());
        self.client_service.get_account_lines(addr).await
    }
}

impl std::fmt::Debug for TransactionService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransactionService")
            .field("address", &self.signer.address())
            .field("signer", &self.signer)
            .finish()
    }
}
