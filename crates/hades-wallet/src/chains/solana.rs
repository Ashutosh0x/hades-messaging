use crate::error::WalletError;
use ed25519_dalek::{Signer, SigningKey};

pub fn secret_to_address(secret_bytes: &[u8]) -> Result<String, WalletError> {
    let key = to_signing_key(secret_bytes)?;
    Ok(bs58::encode(key.verifying_key().as_bytes()).into_string())
}

pub fn secret_to_pubkey_hex(secret_bytes: &[u8]) -> Result<String, WalletError> {
    let key = to_signing_key(secret_bytes)?;
    Ok(hex::encode(key.verifying_key().as_bytes()))
}

pub fn sign_message(secret_bytes: &[u8], message: &[u8]) -> Result<Vec<u8>, WalletError> {
    let key = to_signing_key(secret_bytes)?;
    let signature = key.sign(message);
    Ok(signature.to_bytes().to_vec())
}

fn to_signing_key(secret: &[u8]) -> Result<SigningKey, WalletError> {
    if secret.len() < 32 {
        return Err(WalletError::DerivationFailed("Key too short for Solana".into()));
    }
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&secret[..32]);
    Ok(SigningKey::from_bytes(&bytes))
}

/// Solana system program transfer instruction
pub fn build_transfer_instruction(from_idx: u8, to_idx: u8, lamports: u64) -> CompiledInstruction {
    // System program Transfer = discriminator 2 (u32 LE)
    let mut data = vec![2, 0, 0, 0];
    data.extend_from_slice(&lamports.to_le_bytes());
    CompiledInstruction { program_id_index: 2, accounts: vec![from_idx, to_idx], data }
}

pub struct CompiledInstruction {
    pub program_id_index: u8,
    pub accounts: Vec<u8>,
    pub data: Vec<u8>,
}

/// Build, sign, and serialize a Solana SOL transfer transaction.
/// Returns `(raw_tx_bytes, signature_base58)`.
pub fn build_and_sign_sol_tx(
    secret: &[u8], to_address: &str, lamports: u64, recent_blockhash: &str,
) -> Result<(Vec<u8>, String), WalletError> {
    let signing_key = to_signing_key(secret)?;
    let from_pubkey = signing_key.verifying_key().to_bytes();

    let to_bytes = bs58::decode(to_address).into_vec()
        .map_err(|e| WalletError::InvalidAddress(e.to_string()))?;
    if to_bytes.len() != 32 {
        return Err(WalletError::InvalidAddress("Solana address must be 32 bytes".into()));
    }
    let mut to_pubkey = [0u8; 32];
    to_pubkey.copy_from_slice(&to_bytes);

    let bh_bytes = bs58::decode(recent_blockhash).into_vec()
        .map_err(|e| WalletError::Internal(format!("Invalid blockhash: {}", e)))?;
    if bh_bytes.len() != 32 {
        return Err(WalletError::Internal("Blockhash must be 32 bytes".into()));
    }
    let mut blockhash = [0u8; 32];
    blockhash.copy_from_slice(&bh_bytes);

    let system_program = [0u8; 32];
    let ix = build_transfer_instruction(0, 1, lamports);

    // Serialize the message
    let message = serialize_message(
        &[from_pubkey, to_pubkey, system_program], &blockhash, &[ix], 1, 0, 1,
    );

    let signature = signing_key.sign(&message);
    let sig_bytes = signature.to_bytes();

    // Serialize full transaction: compact-u16(1) + sig(64) + message
    let mut tx = Vec::new();
    tx.push(1); // one signature
    tx.extend_from_slice(&sig_bytes);
    tx.extend_from_slice(&message);

    let tx_sig = bs58::encode(&sig_bytes).into_string();
    Ok((tx, tx_sig))
}

fn serialize_message(
    account_keys: &[[u8; 32]], recent_blockhash: &[u8; 32],
    instructions: &[CompiledInstruction], num_required_sigs: u8,
    num_readonly_signed: u8, num_readonly_unsigned: u8,
) -> Vec<u8> {
    let mut msg = Vec::new();
    msg.push(num_required_sigs);
    msg.push(num_readonly_signed);
    msg.push(num_readonly_unsigned);
    compact_u16(&mut msg, account_keys.len() as u16);
    for key in account_keys { msg.extend_from_slice(key); }
    msg.extend_from_slice(recent_blockhash);
    compact_u16(&mut msg, instructions.len() as u16);
    for ix in instructions {
        msg.push(ix.program_id_index);
        compact_u16(&mut msg, ix.accounts.len() as u16);
        msg.extend_from_slice(&ix.accounts);
        compact_u16(&mut msg, ix.data.len() as u16);
        msg.extend_from_slice(&ix.data);
    }
    msg
}

fn compact_u16(buf: &mut Vec<u8>, val: u16) {
    if val < 0x80 {
        buf.push(val as u8);
    } else if val < 0x4000 {
        buf.push(((val & 0x7F) | 0x80) as u8);
        buf.push((val >> 7) as u8);
    } else {
        buf.push(((val & 0x7F) | 0x80) as u8);
        buf.push((((val >> 7) & 0x7F) | 0x80) as u8);
        buf.push((val >> 14) as u8);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solana_address() {
        let sk = SigningKey::generate(&mut rand::rngs::OsRng);
        let addr = secret_to_address(&sk.to_bytes()).unwrap();
        assert!(addr.len() >= 32 && addr.len() <= 44);
        let decoded = bs58::decode(&addr).into_vec().unwrap();
        assert_eq!(decoded.len(), 32);
    }

    #[test]
    fn test_solana_sign_message() {
        let sk = SigningKey::generate(&mut rand::rngs::OsRng);
        let sig = sign_message(&sk.to_bytes(), b"hello").unwrap();
        assert_eq!(sig.len(), 64);
    }
}
