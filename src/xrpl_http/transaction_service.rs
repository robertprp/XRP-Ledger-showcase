use std::str::FromStr;
use bigdecimal::{FromPrimitive, ToPrimitive, BigDecimal};
use tracing::info;
use xrpl_binary_codec::serialize;
use xrpl_http_client::{Client, SubmitRequest, SubmitResponse};
use xrpl_types::{
    AccountId, Amount, CurrencyCode, DropsAmount, IssuedAmount, IssuedValue, PaymentFlags, PaymentTransaction, Transaction, TrustSetTransaction
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
    
    pub async fn send_token_as_bytes(&self, token_address: &str, amount: &str, destination_address: &str) -> Result<Vec<u8>, String> {
        let account_id = AccountId::from_address(self.signer.address())
            .map_err(|e| format!("Invalid account address: {e}"))?;
        
        let destination = AccountId::from_address(destination_address).unwrap();
        
        let currencies = self.client_service
            .get_account_currencies(token_address)
            .await?;

        if currencies.receive_currencies.is_empty() {
            return Err(format!("No currencies found for token: {}", token_address));
        }

        let currency_code = &currencies.receive_currencies[0];
        let currency = CurrencyCode::from_str(currency_code)
            .map_err(|e| format!("Invalid currency code: {e}"))?;
        
        let value = BigDecimal::from_str(amount)
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

        let issued_value = IssuedValue::from_mantissa_exponent(mantissa, exponent)
            .map_err(|e| format!("Failed to create issued value: {e}"))?;
        
        let issuer = AccountId::from_address(token_address).unwrap();
        let amount = Amount::Issued(
            IssuedAmount::from_issued_value(issued_value, currency, issuer).unwrap()
        );
        
        let payment = PaymentTransaction::new(account_id, amount, destination);
        
        self.prepare_transaction(payment).await
    }

    
    pub async fn send_transaction_from_bytes(&self, tx_bytes: Vec<u8>) -> Result<SubmitResponse, String> {
        let req = SubmitRequest::new(hex::encode(&tx_bytes));
        let response = self
            .client
            .call(req)
            .await
            .map_err(|e| format!("Failed to submit transaction: {e}"))?;

        Ok(response)
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
        let currencies = self
            .client_service
            .get_account_currencies(token_address)
            .await?;

        if currencies.receive_currencies.is_empty() {
            return Err("No currencies found for the given address".to_string());
        }

        let currency_code = &currencies.receive_currencies[0];
        
        info!("Signer address: {}", self.signer.address);
        info!("Signer address v2: {}", self.signer.address());
        
        let account_id = AccountId::from_address(&self.signer.address.clone())
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

        let mut tx = TrustSetTransaction::new(account_id, issued_amount);

        let address = self.signer.address.clone();
        let resp = self.client_service.get_account_info(&address).await?;
        
        let common_mut = tx.common_mut();
        common_mut.sequence = Some(resp.account_data.sequence);
        
        self.client
            .prepare_transaction(common_mut)
            .await
            .map_err(|e| format!("Failed to prepare transaction: {e}"))
            .unwrap();
        
        self.signer.sign_transaction(&mut tx)?;
        
        let tx_bytes = serialize::serialize(&tx)
            .map_err(|e| format!("Failed to serialize transaction: {e}"))?;
        
        let req = SubmitRequest::new(hex::encode(&tx_bytes));
        let response = self
            .client
            .call(req)
            .await
            .map_err(|e| format!("Failed to submit transaction: {e}"))?;

        Ok(response)
    }
    
    
    pub async fn prepare_transaction<T>(&self, mut transaction: T) -> Result<Vec<u8>, String> 
    where
        T: Transaction + Clone + std::fmt::Debug,
    {
        let address = self.signer.address.clone();
        let resp = self.client_service.get_account_info(&address).await?;
        
        let common_mut = transaction.common_mut();
        common_mut.sequence = Some(resp.account_data.sequence);
        
        self.client
            .prepare_transaction(common_mut)
            .await
            .map_err(|e| format!("Failed to prepare transaction: {e}"))
            .unwrap();
        
        info!("Transaction before signing: {:?}", transaction);
        
        self.signer.sign_transaction(&mut transaction)?;
        
        info!("Transaction after signing: {:?}", transaction);
        let tx_bytes = serialize::serialize(&transaction)
            .map_err(|e| format!("Failed to serialize transaction: {e}"))?;
        
        Ok(tx_bytes)
    }

    /// Prepare, sign, and submit a transaction
    async fn prepare_and_submit_transaction<T>(
        &self,
        transaction: T,
    ) -> Result<SubmitResponse, String>
    where
        T: Transaction + Clone + std::fmt::Debug,
    {
        let tx_blob = self.prepare_transaction(transaction.clone()).await?;

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
