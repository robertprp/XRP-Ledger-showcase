use ::tracing::{error, info};
use dotenv;

pub mod tracing;
pub mod xrpl_http;

use xrpl_http::{ClientService, TransactionService};

#[tokio::main]
async fn main() {
    if let Err(e) = tracing::init() {
        eprintln!("Error initializing tracing: {e}");
        std::process::exit(1);
    }
    
    dotenv::dotenv().ok();

    // The seed key starts with "s".
    let seed_middle_man = &std::env::var("SEED_MIDDLE_MAN").expect("SEED not set on .env");
    let seed_solver = &std::env::var("SEED_SOLVER").expect("SEED not set on .env");
    
    info!("Middle man seed: {}", seed_middle_man);
    info!("Solver seed: {}", seed_solver);

    let solver_service = TransactionService::from_seed(seed_solver).unwrap();
    
    let solver_address = solver_service.address();
    info!("Solver address: {}", solver_address);
    
    let mm_service = TransactionService::from_seed(seed_middle_man).unwrap();
    info!("Middle man address: {}", mm_service.address());
    
    let usdc_address = "rGm7WCVp9gb4jZHWTEtGUr4dd74z2XuWhE";
    let ripple_usd_address = "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De"; // USD

    let solver_trustline = solver_service.create_trust_line(ripple_usd_address, None).await.unwrap();
    info!("Solver trustline: {:?}", solver_trustline);
    // let amount = "0.1";
    // 
    // let payment_bytes = mm_service.send_token_as_bytes(usdc_address, amount, solver_address).await.unwrap();
    // 
    // let submit_by_solver = solver_service.send_transaction_from_bytes(payment_bytes).await.unwrap();
    // 
    // info!("Submit by solver: {:?}", submit_by_solver);
    
    // Token addresses
    // let ripple_usd_address = "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De"; // USD
    // let usdc_address = "rGm7WCVp9gb4jZHWTEtGUr4dd74z2XuWhE";
    // let army_address = "rGG3wQ4kUzd7Jnmk1n5NWPZjjut62kCBfC";
    // let token_find_address = "r9Xzi4KsSF1Xtr8WHyBmUcvfP9FzTyG5wp";
    // let xrp_address = "XRP";

    // let tx_hash = "C4283F49564A12BFC52933FA4B94C4E255E2D54C354264770A6C397FAF6E45A3";

    // let client_service = ClientService::new();
    // let details = client_service.balance_change(tx_hash).await;
    // info!("Details: {:?}", details);

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
