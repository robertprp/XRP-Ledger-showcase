use ::tracing::{error, info};
use dotenv;

pub mod tracing;
pub mod xrpl_http;

use xrpl_http::{ClientService, TransactionService};

#[tokio::main]
async fn main() {
    // Initialize tracing
    if let Err(e) = tracing::init() {
        eprintln!("Error initializing tracing: {e}");
        std::process::exit(1);
    }
    
    dotenv::dotenv().ok();

    // The secret key appears to be in base58 format (common for crypto keys)
    let secret_key_str = &std::env::var("SEED").expect("SEED not set on .env");

    // Create services
    let transaction_service = match TransactionService::from_seed(secret_key_str) {
        Ok(service) => service,
        Err(e) => {
            error!("Failed to create transaction service: {}", e);
            std::process::exit(1);
        }
    };

    info!("Account address: {}", transaction_service.address());

    // Token addresses
    let ripple_usd_address = "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De"; // USD
    let usdc_address = "rGm7WCVp9gb4jZHWTEtGUr4dd74z2XuWhE";
    let army_address = "rGG3wQ4kUzd7Jnmk1n5NWPZjjut62kCBfC";
    let token_find_address = "r9Xzi4KsSF1Xtr8WHyBmUcvfP9FzTyG5wp";
    let xrp_address = "XRP";

    let tx_hash = "C4283F49564A12BFC52933FA4B94C4E255E2D54C354264770A6C397FAF6E45A3";

    let client_service = ClientService::new();
    let details = client_service.balance_change(tx_hash).await;
    info!("Details: {:?}", details);

    // let swap_request = SwapRequest::new(
    //     token_find_address.to_string(),
    //     xrp_address.to_string(),
    //     "46.27819".to_string(),
    //     "0.8".to_string(),
    // );

    // if let Err(e) = swap_request.validate() {
    //     error!("Invalid swap request: {}", e);
    //     return;
    // }

    // info!("Execung swap request: {:?}", swap_request);

    // match transaction_service.swap(swap_request).await {
    //     Ok(response) => {
    //     }
    //     Err(e) => {
    //         error!("Failed to execute swap: {}", e);
    //     }
    // }

    info!("Application completed successfully");
}
