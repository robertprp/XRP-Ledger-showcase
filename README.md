# XRP Ledger Rust Tests

A Rust library for interacting with the XRP Ledger (XRPL) with a clean, modular architecture.

## Architecture

This project has been refactored to provide clear separation of concerns between read-only operations and transaction operations that require signing.

### Services

#### `ClientService`
Handles read-only XRPL operations that only require HTTP client interactions:
- `get_account_info()` - Retrieve account information and balance
- `get_account_currencies()` - Get currencies an account can send/receive
- `get_account_lines()` - Get trust lines for an account
- `account_exists()` - Check if an account exists on the ledger

#### `TransactionService`
Handles transaction operations that require signing and submission:
- `swap()` - Execute token swaps using Payment transactions
- `create_trust_line()` - Create trust lines for tokens
- Built-in access to `ClientService` methods for convenience
- Automatic transaction preparation, signing, and submission

#### `RippleSigner`
Handles cryptographic operations:
- Key derivation from seed phrases
- Transaction signing
- Secure key management

### Types

#### `SwapRequest`
Structure for token swap operations:
```rust
pub struct SwapRequest {
    pub token_in: String,      // Token to send ("XRP" for native XRP)
    pub token_out: String,     // Token to receive
    pub amount_in: String,     // Amount to send
    pub amount_out_min: String,// Minimum amount to receive
}
```

#### `TrustLineRequest`
Structure for trust line creation:
```rust
pub struct TrustLineRequest {
    pub token_address: String, // Token address to trust
    pub limit: Option<String>, // Optional trust limit
}
```

#### Error Types
- `SwapError` - Comprehensive error handling for swap operations
- `OperationResponse` - Standardized response format

## Usage

### Basic Setup

```rust
use shogun_xrp::xrpl_http::{ClientService, TransactionService, SwapRequest};

// For read-only operations
let client_service = ClientService::new();

// For transactions (requires seed)
let transaction_service = TransactionService::from_seed("your_seed_here")?;
```

### Read-Only Operations

```rust
// Get account information
let account_info = client_service.get_account_info("rAccount...").await?;

// Check account currencies
let currencies = client_service.get_account_currencies("rAccount...").await?;

// Get trust lines
let lines = client_service.get_account_lines("rAccount...").await?;
```

### Transaction Operations

```rust
// Create a swap request
let swap_request = SwapRequest::new(
    "XRP".to_string(),                    // Send XRP
    "rTokenAddress...".to_string(),       // Receive token
    "1".to_string(),                      // Send 1 XRP
    "100".to_string(),                    // Minimum 100 tokens
);

// Validate and execute swap
swap_request.validate()?;
let result = transaction_service.swap(swap_request).await?;

// Create trust line
let response = transaction_service.create_trust_line("rTokenAddress...", Some("1000000")).await?;
```

### Accessing Both Services

The `TransactionService` includes convenience methods that use the internal `ClientService`:

```rust
// Get your own account info
let account_info = transaction_service.get_account_info(None).await?;

// Get another account's info
let other_info = transaction_service.get_account_info(Some("rOther...")).await?;

// Direct access to client service
let client = transaction_service.client_service();
let currencies = client.get_account_currencies("rAccount...").await?;
```

## Key Features

- **Separation of Concerns**: Read-only and transaction operations are clearly separated
- **Type Safety**: Comprehensive type system with validation
- **Error Handling**: Detailed error types with meaningful messages
- **Security**: Secure key management with the `RippleSigner`
- **Flexibility**: Support for XRP-to-token, token-to-XRP, and token-to-token swaps
- **Validation**: Built-in request validation before submission
- **Logging**: Comprehensive tracing/logging throughout

## Dependencies

Key dependencies include:
- `xrpl_http_client` - XRPL HTTP client
- `xrpl_binary_codec` - Transaction serialization
- `xrpl_types` - XRPL type definitions
- `ripple-keypairs` - Key generation and management
- `libsecp256k1` - Cryptographic operations
- `tokio` - Async runtime
- `tracing` - Logging and observability

## Example

See `src/main.rs` for a complete example demonstrating both services and various operations.

## License

This project is licensed under the terms specified in `Cargo.toml`.