use libsecp256k1::{PublicKey, SecretKey};
use ripple_keypairs::Seed;
use std::str::FromStr;
use xrpl_binary_codec::sign;
use xrpl_types::Transaction;

/// Handles cryptographic operations for XRPL transactions
pub struct RippleSigner {
    pub address: String,
    pub secret_key: SecretKey,
    pub public_key: PublicKey,
}

impl RippleSigner {
    /// Create a new signer from a seed string
    pub fn from_seed(seed_str: &str) -> Result<Self, String> {
        let seed = Seed::from_str(seed_str).map_err(|e| format!("Invalid seed format: {e}"))?;

        let (private_key, public_key) = seed
            .derive_keypair()
            .map_err(|e| format!("Failed to derive keypair: {e}"))?;

        let address = public_key.derive_address();
        let secret_key_hex = private_key.to_string();

        let hex_secret = if secret_key_hex.starts_with("00") {
            &secret_key_hex[2..]
        } else {
            &secret_key_hex
        };

        let secret_bytes = hex::decode(hex_secret)
            .map_err(|e| format!("Failed to decode secret key hex: {e}"))?;

        let secret_key = SecretKey::parse_slice(&secret_bytes)
            .map_err(|e| format!("Failed to parse secret key: {e}"))?;

        let public_key = PublicKey::from_secret_key(&secret_key);

        Ok(Self {
            address,
            secret_key,
            public_key,
        })
    }

    /// Create a new signer from raw secret key bytes
    pub fn from_secret_key_bytes(secret_bytes: &[u8], address: String) -> Result<Self, String> {
        let secret_key = SecretKey::parse_slice(secret_bytes)
            .map_err(|e| format!("Failed to parse secret key: {e}"))?;

        let public_key = PublicKey::from_secret_key(&secret_key);

        Ok(Self {
            address,
            secret_key,
            public_key,
        })
    }

    /// Sign a transaction
    pub fn sign_transaction<T: Transaction>(&self, transaction: &mut T) -> Result<(), String> {
        sign::sign_transaction(transaction, &self.public_key, &self.secret_key)
            .map_err(|e| format!("Failed to sign transaction: {e}"))
    }

    /// Get the account address
    pub fn address(&self) -> &str {
        &self.address
    }

    /// Get the public key
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    /// Get a reference to the secret key (use with caution)
    pub fn secret_key(&self) -> &SecretKey {
        &self.secret_key
    }
}

impl std::fmt::Debug for RippleSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RippleSigner")
            .field("address", &self.address)
            .field("public_key", &"[REDACTED]")
            .field("secret_key", &"[REDACTED]")
            .finish()
    }
}
