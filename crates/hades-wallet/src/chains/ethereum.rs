use crate::error::WalletError;
use k256::ecdsa::SigningKey;
use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Keccak};

// ─── Address derivation ─────────────────────────────────────

/// Derive Ethereum address from secp256k1 secret key bytes
pub fn secret_to_address(secret_bytes: &[u8]) -> Result<String, WalletError> {
    let pk = compressed_pubkey_to_uncompressed(secret_bytes)?;
    let mut hasher = Keccak::v256();
    let mut hash = [0u8; 32];
    hasher.update(&pk);
    hasher.finalize(&mut hash);
    Ok(format!("0x{}", eip55_checksum(&hash[12..32])))
}

pub fn secret_to_pubkey_hex(secret_bytes: &[u8]) -> Result<String, WalletError> {
    let signing_key = SigningKey::from_slice(secret_bytes)
        .map_err(|e| WalletError::DerivationFailed(e.to_string()))?;
    let verifying_key = signing_key.verifying_key();
    Ok(hex::encode(
        verifying_key.to_encoded_point(true).as_bytes(),
    ))
}

fn compressed_pubkey_to_uncompressed(
    secret_bytes: &[u8],
) -> Result<Vec<u8>, WalletError> {
    let signing_key = SigningKey::from_slice(secret_bytes)
        .map_err(|e| WalletError::DerivationFailed(e.to_string()))?;
    let point = signing_key.verifying_key().to_encoded_point(false);
    Ok(point.as_bytes()[1..].to_vec()) // strip 0x04 prefix
}

/// EIP-55 checksum encoding
fn eip55_checksum(addr_bytes: &[u8]) -> String {
    let hex_addr = hex::encode(addr_bytes);

    let mut hasher = Keccak::v256();
    let mut hash = [0u8; 32];
    hasher.update(hex_addr.as_bytes());
    hasher.finalize(&mut hash);
    let hash_hex = hex::encode(hash);

    hex_addr
        .chars()
        .enumerate()
        .map(|(i, c)| {
            if c.is_ascii_alphabetic() {
                let nibble =
                    u8::from_str_radix(&hash_hex[i..i + 1], 16).unwrap_or(0);
                if nibble >= 8 {
                    c.to_ascii_uppercase()
                } else {
                    c
                }
            } else {
                c
            }
        })
        .collect()
}

pub fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak::v256();
    let mut out = [0u8; 32];
    hasher.update(data);
    hasher.finalize(&mut out);
    out
}

/// Sign a 32-byte hash with secp256k1 (returns r || s || v, 65 bytes)
pub fn sign_hash(
    secret_bytes: &[u8],
    hash: &[u8],
) -> Result<Vec<u8>, WalletError> {
    let signing_key = SigningKey::from_slice(secret_bytes)
        .map_err(|e| WalletError::SigningFailed(e.to_string()))?;

    let (signature, recovery_id) = signing_key
        .sign_prehash_recoverable(hash)
        .map_err(|e| WalletError::SigningFailed(e.to_string()))?;

    let mut sig_bytes = signature.to_bytes().to_vec();
    sig_bytes.push(recovery_id.to_byte() + 27); // v = recovery_id + 27
    Ok(sig_bytes)
}

/// Tron uses same keys but Base58Check with 0x41 prefix
pub fn secret_to_tron_address(
    secret_bytes: &[u8],
) -> Result<String, WalletError> {
    let signing_key = SigningKey::from_slice(secret_bytes)
        .map_err(|e| WalletError::DerivationFailed(e.to_string()))?;

    let verifying_key = signing_key.verifying_key();
    let pubkey_bytes = verifying_key.to_encoded_point(false);
    let pubkey_uncompressed = &pubkey_bytes.as_bytes()[1..];

    let mut hasher = Keccak::v256();
    let mut hash = [0u8; 32];
    hasher.update(pubkey_uncompressed);
    hasher.finalize(&mut hash);

    let mut addr_with_prefix = vec![0x41]; // Tron mainnet prefix
    addr_with_prefix.extend_from_slice(&hash[12..32]);

    // Base58Check
    Ok(bs58::encode(&addr_with_prefix)
        .with_check()
        .into_string())
}

// ─── 256-bit unsigned integer ─────────────────────────────────

/// 256-bit unsigned integer for ETH values (wei)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct U256(pub [u8; 32]);

impl U256 {
    pub fn zero() -> Self {
        Self([0u8; 32])
    }

    /// Parse from decimal string like "1500000000000000000" (1.5 ETH in wei)
    pub fn from_decimal_str(s: &str) -> Result<Self, WalletError> {
        if s.is_empty() || s == "0" {
            return Ok(Self::zero());
        }

        // Validate input
        if !s.chars().all(|c| c.is_ascii_digit()) {
            return Err(WalletError::Internal(
                "Invalid decimal string".into(),
            ));
        }

        // Multiply-and-add approach in big-endian bytes
        let mut result = [0u8; 32];

        for ch in s.chars() {
            let digit = (ch as u8) - b'0';

            // result = result * 10 + digit
            let mut carry: u16 = digit as u16;
            for byte in result.iter_mut().rev() {
                let val = (*byte as u16) * 10 + carry;
                *byte = (val & 0xFF) as u8;
                carry = val >> 8;
            }
            if carry > 0 {
                return Err(WalletError::Internal(
                    "Value too large for U256".into(),
                ));
            }
        }

        Ok(Self(result))
    }

    /// From human-readable amount + decimals: "1.5" with 18 decimals → wei
    pub fn from_human(
        amount: &str,
        decimals: u8,
    ) -> Result<Self, WalletError> {
        let parts: Vec<&str> = amount.split('.').collect();
        let whole = parts[0];
        let frac = if parts.len() > 1 { parts[1] } else { "" };

        // Pad or truncate fractional part to `decimals` digits
        let frac_padded = if frac.len() >= decimals as usize {
            frac[..decimals as usize].to_string()
        } else {
            format!("{:0<width$}", frac, width = decimals as usize)
        };

        let raw_str = format!("{}{}", whole, frac_padded);
        // Remove leading zeros but handle all-zeros case
        let raw_str = raw_str.trim_start_matches('0');
        if raw_str.is_empty() {
            return Ok(Self::zero());
        }
        Self::from_decimal_str(raw_str)
    }

    /// Returns the minimal big-endian byte representation (no leading zeros)
    pub fn to_minimal_bytes(&self) -> Vec<u8> {
        let start = self.0.iter().position(|&b| b != 0).unwrap_or(31);
        self.0[start..].to_vec()
    }

    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }
}

// ─── EIP-1559 Transaction ─────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmTxParams {
    pub chain_id: u64,
    pub nonce: u64,
    pub to: [u8; 20],
    pub value: U256,
    pub data: Vec<u8>,
    pub max_fee_per_gas: u64,
    pub max_priority_fee_per_gas: u64,
    pub gas_limit: u64,
}

/// Build, sign, and serialize an EIP-1559 transaction.
/// Returns `(raw_signed_bytes, tx_hash_hex)`.
pub fn sign_eip1559_tx(
    secret: &[u8],
    params: &EvmTxParams,
) -> Result<(Vec<u8>, String), WalletError> {
    // 1. RLP-encode the unsigned transaction fields
    let unsigned_fields: Vec<RlpItem> = vec![
        RlpItem::Uint(params.chain_id),
        RlpItem::Uint(params.nonce),
        RlpItem::Uint(params.max_priority_fee_per_gas),
        RlpItem::Uint(params.max_fee_per_gas),
        RlpItem::Uint(params.gas_limit),
        RlpItem::Bytes(params.to.to_vec()),
        RlpItem::Bytes(params.value.to_minimal_bytes()),
        RlpItem::Bytes(params.data.clone()),
        RlpItem::List(vec![]), // access_list
    ];

    let unsigned_rlp = rlp_encode_list(&unsigned_fields);

    // 2. Signing payload = keccak256(0x02 || unsigned_rlp)
    let mut preimage = vec![0x02u8];
    preimage.extend_from_slice(&unsigned_rlp);
    let hash = keccak256(&preimage);

    // 3. Sign with secp256k1
    let signing_key = SigningKey::from_slice(secret)
        .map_err(|e| WalletError::SigningFailed(e.to_string()))?;

    let (signature, recid) = signing_key
        .sign_prehash_recoverable(&hash)
        .map_err(|e| WalletError::SigningFailed(e.to_string()))?;

    let sig_bytes = signature.to_bytes();
    let r = &sig_bytes[..32];
    let s = &sig_bytes[32..64];
    let v = recid.to_byte(); // 0 or 1 for EIP-1559

    // 4. RLP-encode the signed transaction
    let mut signed_fields = unsigned_fields;
    signed_fields.push(RlpItem::Uint(v as u64));
    signed_fields.push(RlpItem::Bytes(strip_leading_zeros(r)));
    signed_fields.push(RlpItem::Bytes(strip_leading_zeros(s)));

    let signed_rlp = rlp_encode_list(&signed_fields);

    // 5. Prepend type byte 0x02
    let mut raw_tx = vec![0x02u8];
    raw_tx.extend_from_slice(&signed_rlp);

    // 6. Compute tx hash = keccak256(raw_tx)
    let tx_hash = keccak256(&raw_tx);
    let tx_hash_hex = format!("0x{}", hex::encode(tx_hash));

    Ok((raw_tx, tx_hash_hex))
}

/// Build ERC-20 `transfer(address, uint256)` calldata
pub fn erc20_transfer_data(to: &[u8; 20], amount: &U256) -> Vec<u8> {
    // function selector: keccak256("transfer(address,uint256)")[..4]
    let selector = &keccak256(b"transfer(address,uint256)")[..4];
    let mut data = Vec::with_capacity(68);
    data.extend_from_slice(selector);
    // address padded to 32 bytes
    data.extend_from_slice(&[0u8; 12]);
    data.extend_from_slice(to);
    // uint256
    data.extend_from_slice(&amount.0);
    data
}

/// Build and sign an EVM transaction using legacy parameters (backward compat)
pub fn build_and_sign_tx(
    secret_bytes: &[u8],
    chain_id: u64,
    nonce: u64,
    to: &str,
    value_wei: &[u8],
    data: &[u8],
    max_fee_per_gas: u64,
    max_priority_fee: u64,
    gas_limit: u64,
) -> Result<Vec<u8>, WalletError> {
    let to_bytes = hex::decode(to.trim_start_matches("0x"))
        .map_err(|_| WalletError::InvalidAddress(to.to_string()))?;
    if to_bytes.len() != 20 {
        return Err(WalletError::InvalidAddress(
            "Address must be 20 bytes".into(),
        ));
    }
    let mut to_arr = [0u8; 20];
    to_arr.copy_from_slice(&to_bytes);

    // Pad value_wei to 32 bytes
    let mut value = U256::zero();
    if !value_wei.is_empty() {
        let start = 32usize.saturating_sub(value_wei.len());
        value.0[start..].copy_from_slice(value_wei);
    }

    let params = EvmTxParams {
        chain_id,
        nonce,
        to: to_arr,
        value,
        data: data.to_vec(),
        max_fee_per_gas,
        max_priority_fee_per_gas: max_priority_fee,
        gas_limit,
    };

    let (raw_tx, _hash) = sign_eip1559_tx(secret_bytes, &params)?;
    Ok(raw_tx)
}

fn strip_leading_zeros(bytes: &[u8]) -> Vec<u8> {
    let start = bytes
        .iter()
        .position(|&b| b != 0)
        .unwrap_or(bytes.len().saturating_sub(1));
    bytes[start..].to_vec()
}

// ─── RLP Encoding ─────────────────────────────────────────────

#[derive(Clone)]
enum RlpItem {
    Bytes(Vec<u8>),
    Uint(u64),
    List(Vec<RlpItem>),
}

fn rlp_encode_item(item: &RlpItem) -> Vec<u8> {
    match item {
        RlpItem::Uint(0) => vec![0x80], // empty string for zero
        RlpItem::Uint(val) => {
            let bytes = val.to_be_bytes();
            let start = bytes.iter().position(|&b| b != 0).unwrap_or(7);
            let trimmed = &bytes[start..];
            rlp_encode_item(&RlpItem::Bytes(trimmed.to_vec()))
        }
        RlpItem::Bytes(b) if b.is_empty() => vec![0x80],
        RlpItem::Bytes(b) if b.len() == 1 && b[0] < 0x80 => vec![b[0]],
        RlpItem::Bytes(b) if b.len() <= 55 => {
            let mut out = vec![0x80 + b.len() as u8];
            out.extend_from_slice(b);
            out
        }
        RlpItem::Bytes(b) => {
            let len_bytes = encode_length(b.len());
            let mut out = vec![0xb7 + len_bytes.len() as u8];
            out.extend_from_slice(&len_bytes);
            out.extend_from_slice(b);
            out
        }
        RlpItem::List(items) => {
            let mut payload = Vec::new();
            for item in items {
                payload.extend_from_slice(&rlp_encode_item(item));
            }
            rlp_encode_list_payload(&payload)
        }
    }
}

fn rlp_encode_list(items: &[RlpItem]) -> Vec<u8> {
    let mut payload = Vec::new();
    for item in items {
        payload.extend_from_slice(&rlp_encode_item(item));
    }
    rlp_encode_list_payload(&payload)
}

fn rlp_encode_list_payload(payload: &[u8]) -> Vec<u8> {
    if payload.len() <= 55 {
        let mut out = vec![0xc0 + payload.len() as u8];
        out.extend_from_slice(payload);
        out
    } else {
        let len_bytes = encode_length(payload.len());
        let mut out = vec![0xf7 + len_bytes.len() as u8];
        out.extend_from_slice(&len_bytes);
        out.extend_from_slice(payload);
        out
    }
}

fn encode_length(len: usize) -> Vec<u8> {
    let bytes = (len as u64).to_be_bytes();
    let start = bytes.iter().position(|&b| b != 0).unwrap_or(7);
    bytes[start..].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_derivation() {
        let secret = hex::decode(
            "4c0883a69102937d6231471b5dbb6204fe512961708279f1d4e9c1d0e80a0b2e",
        )
        .unwrap();
        let addr = secret_to_address(&secret).unwrap();
        assert!(addr.starts_with("0x"));
        assert_eq!(addr.len(), 42);
    }

    #[test]
    fn test_sign_roundtrip() {
        let signing_key = SigningKey::random(&mut rand::thread_rng());
        let hash = [0x42u8; 32];
        let sig =
            sign_hash(&signing_key.to_bytes().to_vec(), &hash).unwrap();
        assert_eq!(sig.len(), 65); // r(32) + s(32) + v(1)
    }

    #[test]
    fn test_u256_from_human() {
        let val = U256::from_human("1.5", 18).unwrap();
        assert!(!val.is_zero());

        let val2 = U256::from_human("0.0", 18).unwrap();
        assert!(val2.is_zero());

        // 1 ETH = 10^18 wei
        let one_eth = U256::from_human("1.0", 18).unwrap();
        assert!(!one_eth.is_zero());
    }

    #[test]
    fn test_u256_from_decimal_str() {
        let val = U256::from_decimal_str("1000000000000000000").unwrap();
        assert!(!val.is_zero());

        let zero = U256::from_decimal_str("0").unwrap();
        assert!(zero.is_zero());
    }

    #[test]
    fn test_sign_eip1559_produces_valid_tx() {
        let sk = SigningKey::random(&mut rand::thread_rng());
        let secret = sk.to_bytes().to_vec();

        let mut to = [0u8; 20];
        to[19] = 0x01;

        let params = EvmTxParams {
            chain_id: 1,
            nonce: 0,
            to,
            value: U256::from_human("0.01", 18).unwrap(),
            data: vec![],
            max_fee_per_gas: 30_000_000_000,
            max_priority_fee_per_gas: 2_000_000_000,
            gas_limit: 21_000,
        };

        let (raw, hash) = sign_eip1559_tx(&secret, &params).unwrap();
        assert_eq!(raw[0], 0x02); // EIP-1559 type
        assert!(hash.starts_with("0x"));
        assert_eq!(hash.len(), 66);
    }

    #[test]
    fn test_erc20_transfer_data() {
        let mut to = [0u8; 20];
        to[19] = 0x42;
        let amount = U256::from_human("100.0", 18).unwrap();
        let data = erc20_transfer_data(&to, &amount);
        assert_eq!(data.len(), 68); // 4 selector + 32 addr + 32 amount
        assert_eq!(
            &data[..4],
            &keccak256(b"transfer(address,uint256)")[..4]
        );
    }

    #[test]
    fn test_backward_compat_build_and_sign() {
        let sk = SigningKey::random(&mut rand::thread_rng());
        let secret = sk.to_bytes().to_vec();
        let to = "0x0000000000000000000000000000000000000001";
        let value = vec![0x01]; // 1 wei

        let result = build_and_sign_tx(
            &secret,
            1,
            0,
            to,
            &value,
            &[],
            30_000_000_000,
            2_000_000_000,
            21_000,
        );
        assert!(result.is_ok());
        let tx = result.unwrap();
        assert_eq!(tx[0], 0x02);
    }
}
