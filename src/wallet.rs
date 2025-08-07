use std::str::FromStr;

use bigdecimal::BigDecimal;
use xrpl::{
    asynch::clients::{AsyncJsonRpcClient, XRPLAsyncClient}, 
    core::keypairs::sign, 
    models::{
        requests::{
            account_currencies::AccountCurrencies, account_info::AccountInfo, account_lines::AccountLines, ripple_path_find, RequestMethod
        }, 
        results::{
            account_currencies, account_info::{self, AccountInfoVersionMap}, account_lines, metadata::TransactionMetadata, submit::Submit, tx::TxVersionMap, XRPLResponse
        }, 
        transactions::{
            amm_bid::AMMBid, 
            payment::{Payment, PaymentFlag}, 
            trust_set::TrustSet, 
            CommonFields, 
            TransactionType
        }, 
        Amount, 
        Currency, 
        FlagCollection, 
        IssuedCurrency, 
        IssuedCurrencyAmount, 
        NoFlags, 
        XRPAmount, 
        XRP
    }, 
    transaction::{self, autofill_and_sign, submit, submit_and_wait}, 
    utils::{xrp_to_drops, ToBytes}, 
    wallet::Wallet
};

use crate::{ext::AmountExt, types::swap::{AssetType, SwapParams}};

pub struct WalletService {
    pub wallet: Wallet,
    pub client: AsyncJsonRpcClient
}

impl WalletService {
    pub fn new(wallet: Wallet) -> Self {
        let client = AsyncJsonRpcClient::connect("https://xrplcluster.com/".parse().unwrap());

        Self { wallet, client }
    }

    pub fn from_seed(seed: &str) -> Self {
        let client = AsyncJsonRpcClient::connect("https://xrplcluster.com/".parse().unwrap());

        Self {
            wallet: Wallet::new(seed, 0).unwrap(),
            client
        }
    }

    pub async fn get_account_info(&self, address: String) -> Result<AccountInfoVersionMap, String> {
        let account_info = AccountInfo::new(None, address.into(), None, None, None, None, None);
        let response = self.client.request(account_info.into()).await
            .map_err(|e| format!("Failed to get account info: {:?}", e))?;

        let result = response.result
            .ok_or("No result in response")?;
        let account_info: AccountInfoVersionMap = result.try_into()
            .map_err(|e| format!("Failed to parse account info: {:?}", e))?;

        Ok(account_info)
    }

    pub fn get_wallet(&self) -> &Wallet {
        &self.wallet
    }

    pub fn sign_message(&self, message: String) -> Result<String, String> {
        let private_key = self.wallet.private_key.clone();
        let signature = sign(message.as_bytes(), &private_key)
            .map_err(|e| format!("Failed to sign message: {:?}", e))?;

        Ok(signature)
    }

    pub fn verify_message(&self, message: String, signature: String) -> Result<bool, String> {
        let public_key = self.wallet.public_key.clone();
        let is_valid = xrpl::core::keypairs::is_valid_message(message.as_bytes(), &signature, &public_key);

        Ok(is_valid)
    }

    pub async fn send_native(&self, destination: String, amount_xrp: &str) -> Result<(), String> {
        let wallet = self.wallet.classic_address.clone();
        let amount_drops = xrp_to_drops(amount_xrp)
            .map_err(|e| format!("Failed to convert XRP to drops: {:?}", e))?;

        let mut payment = Payment::new(
            wallet.into(),
            None,                                         // account_txn_id
            None,                                         // transaction fee
            None,                                         // flags
            None,                                         // last_ledger_sequence
            None,                                         // memos
            None,                                         // sequence (will be auto-filled)
            None,                                         // signers
            None,                                         // source_tag
            None,                                         // ticket_sequence
            Amount::XRPAmount(XRPAmount(amount_drops.into())), // amount in drops
            destination.into(),                           // destination address
            None,                                         // deliver_min
            None,                                         // destination_tag (optional)
            None,                                         // invoice_id
            None,                                         // paths (for direct XRP transfer)
            None,                                         // send_max
        );

        autofill_and_sign(&mut payment, &self.client, &self.wallet, false)
            .map_err(|e| format!("Failed to autofill and sign: {:?}", e))?;

        let response = submit(&mut payment, &self.client)
            .map_err(|e| format!("Failed to submit: {:?}", e))?;

        println!("Response submit: {:?}", response);
        Ok(())
    }
    
    pub async fn get_info(&self, address: String) -> Result<AccountInfoVersionMap, String> {
        let account_info = AccountInfo::new(None, address.into(), None, None, None, None, None);
        let response = self.client.request(account_info.into()).await
            .map_err(|e| format!("Failed to get account info: {:?}", e))?;

        let result = response.result
            .ok_or("No result in response")?;
        let account_info: AccountInfoVersionMap = result.try_into()
            .map_err(|e| format!("Failed to parse account info: {:?}", e))?;

        Ok(account_info)
    }
    
    pub async fn get_account_currencies(&self, address: String) -> Result<Option<account_currencies::AccountCurrencies>, String> {
        let account_currencies = AccountCurrencies::new(None, address.clone().into(), None, None, None);
        
        println!("Getting account currencies for {:?}", &address);
        
        let response = self.client.request(account_currencies.into()).await
            .map_err(|e| format!("Failed to get account currencies: {:?}", e))?;
        
        println!("Response: {:?}", response);
        if let Some(result) = response.result {
            let account_currency_result: account_currencies::AccountCurrencies = result.try_into()
                .map_err(|e| format!("Failed to parse account currencies: {:?}", e))?;
            return Ok(Some(account_currency_result));
        }
        
        Ok(None)
    }
    
    pub async fn get_account_lines(&self, address: String) -> Result<Option<account_lines::AccountLines>, String> {
        let account_lines = AccountLines::new(
            None,
            address.into(),
            None, None, None, None 
        );
        
        let response = self.client.request(account_lines.into()).await
            .map_err(|e| format!("Failed to get account lines: {:?}", e))?;
        
        if let Some(result) = response.result {
            let account_line_result: account_lines::AccountLines = result.try_into()
                .map_err(|e| format!("Failed to parse account lines: {:?}", e))?;
            return Ok(Some(account_line_result));
        }
        
        Ok(None)
    }
    
    pub async fn create_trust_line(&self, currency: &str, issuer: &str, limit: &str) -> Result<(), String> {
        let account = self.wallet.classic_address.clone();
        
        let common_fields = CommonFields {
            account: account.into(),
            transaction_type: TransactionType::TrustSet,
            account_txn_id: None,
            fee: None,
            flags: FlagCollection::new(vec![]),
            last_ledger_sequence: None,
            memos: None,
            signers: None,
            source_tag: None,
            ticket_sequence: None,
            network_id: None,
            sequence: None,
            signing_pub_key: None,
            txn_signature: None,
        };

        let mut trust_set = TrustSet {
            common_fields,
            limit_amount: IssuedCurrencyAmount::new(
                currency.into(),
                issuer.into(),
                limit.into(),
            ),
            quality_in: None,
            quality_out: None,
        };

        println!("Creating trust line for {}/{}", currency, issuer);
        autofill_and_sign(&mut trust_set, &self.client, &self.wallet, false)
            .map_err(|e| format!("Failed to autofill trust line: {:?}", e))?;

        let response = submit(&mut trust_set, &self.client)
            .map_err(|e| format!("Failed to submit trust line: {:?}", e))?;

        println!("Trust line created for {}/{}: {:?}", currency, issuer, response);
        Ok(())
    }
    
    pub async fn trustline_exists(&self, token_address: &str, address: &str) -> Result<bool, String> {
        let account_lines = self.get_account_lines(address.into()).await
            .map_err(|e| format!("Failed to get account lines: {}", e))?;
        
        if account_lines.is_none() {
            return Ok(false);
        }
        
        let account_lines = account_lines.unwrap();
        
        for line in account_lines.lines.clone().iter() {
            if line.account.to_string() == token_address {
                return Ok(true);
            }
        }
        Ok(false)
    }
    
    pub async fn swap_token(&self, params: SwapParams) -> Result<(), String> {
        let account = self.wallet.classic_address.clone();
        let destination_account = account.clone();
        
        match params.token_out {
            AssetType::XRP(_) => {  },
            AssetType::Token(token_val) => {
                if !self.trustline_exists(token_val.address.as_str(), token_val.address.as_str()).await? {
                    // Create trust line
                    
                    println!("Creating trust line for {}", token_val.address);
                    let account_lines = self.get_account_currencies(token_val.address.clone()).await
                        .map_err(|e| format!("Failed to get account currencies: {}", e))?;
                    
                    println!("Account lines: {:?}", account_lines);
                    if account_lines.is_none() {
                        return Err("No account lines found".into());
                    }
                    
                    let account_lines = account_lines.unwrap();
                    
                    println!("Account lines: {:?}", account_lines);
                    let currency = account_lines.receive_currencies[0].clone().to_string();
                    
                    let trustline = self.create_trust_line(
                        &currency,
                        token_val.address.as_str(),
                        &token_val.amount.to_string()
                    ).await?;
                    
                    println!("Trust line created: {:?}", trustline);
                }
            }
        };
        
        let slippage_bps = 2000; // 2%
                 
        // Create CommonFields inline to avoid lifetime issues
        let common_fields = CommonFields {
            account: account.into(),
            transaction_type: TransactionType::Payment,
            account_txn_id: None,
            fee: Some("12".into()),
            flags: FlagCollection::new(vec![PaymentFlag::TfPartialPayment]),
            last_ledger_sequence: None,
            memos: None,
            signers: None,
            source_tag: None,
            ticket_sequence: None,
            network_id: None,
            sequence: None,
            signing_pub_key: None,
            txn_signature: None,
        };
            
        let amount = params.token_out_min_amount.into();
        let send_max = Amount::XRPAmount(XRPAmount(xrp_to_drops(params.token_in_min_amount.to_string().as_str())
            .map_err(|e| format!("Failed to convert XRP: {}", e))?.into()));

        
        let mut payment = Payment {
            common_fields,
            amount,
            destination: destination_account.into(),
            send_max: Some(send_max),
            deliver_min: None,
            destination_tag: None,
            invoice_id: None,
            paths: None,
        };
        
        autofill_and_sign(&mut payment, &self.client, &self.wallet, false)
            .map_err(|e| format!("Failed to autofill and sign: {:?}", e))?;
        
        let response = submit(&mut payment, &self.client)
            .map_err(|e| format!("Failed to submit: {:?}", e))?;
        
        println!("Payment submitted: {:?}", response);
        Ok(())
    }
}