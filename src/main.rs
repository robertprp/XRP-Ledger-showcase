use std::str::FromStr;

use bigdecimal::BigDecimal;

use crate::{types::swap::{AssetType, SwapParams}, wallet::WalletService};
pub mod wallet;
pub mod ext;
pub mod types;
pub mod xrpl_http;

#[tokio::main]
async fn main() {
    let seed = "spugUpafEpEthEPNLwbg52GUumFqM";
    // Create wallet from seed
    let wallet_service = WalletService::from_seed(seed);

    let info = wallet_service.get_account_info("rrpuHcXfpBh68V1kGxWYX9X3Qvfkfcwcy9".to_string()).await.unwrap();
    println!("Address: {:?}", info);

    let message = "Hello, world!".to_string();
    let signature = wallet_service.sign_message(message.clone()).unwrap();
    println!("Signature: {}", signature);

    let is_valid = wallet_service.verify_message(message, signature).unwrap();
    println!("Is valid: {}", is_valid);

    let token_in = AssetType::XRP(BigDecimal::from_str("0.5").unwrap());
    let token_out = AssetType::Token(crate::types::swap::TokenValue {
        address: "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De".to_string(),
        amount: BigDecimal::from_str("1.8").unwrap(),
    });
    let result = wallet_service.swap_token(SwapParams {
        token_in: token_in.clone(),
        token_out: token_out.clone(),
        token_in_min_amount: BigDecimal::from_str("0.5").unwrap(),
        token_out_min_amount: BigDecimal::from_str("1.75").unwrap(),
    }).await.unwrap();

    println!("Result: {:?}", result);
}
