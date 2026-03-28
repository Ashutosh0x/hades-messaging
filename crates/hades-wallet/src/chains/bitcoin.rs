use crate::error::WalletError;
use bitcoin::hashes::Hash;
use bitcoin::secp256k1::{PublicKey, Secp256k1, SecretKey};
use serde::{Deserialize, Serialize};

pub fn secret_to_address(secret_bytes: &[u8], testnet: bool) -> Result<String, WalletError> {
    let secp = Secp256k1::new();
    let sk = SecretKey::from_slice(secret_bytes).map_err(|e| WalletError::DerivationFailed(e.to_string()))?;
    let pk = PublicKey::from_secret_key(&secp, &sk);
    let network = if testnet { bitcoin::Network::Testnet } else { bitcoin::Network::Bitcoin };
    let compressed = pk.serialize();
    let pubkey_hash = bitcoin::hashes::hash160::Hash::hash(&compressed);
    let wpkh = bitcoin::WitnessProgram::new(bitcoin::WitnessVersion::V0, pubkey_hash.as_ref())
        .map_err(|e| WalletError::DerivationFailed(e.to_string()))?;
    Ok(bitcoin::Address::from_witness_program(wpkh, network).to_string())
}

pub fn secret_to_ltc_address(secret_bytes: &[u8]) -> Result<String, WalletError> {
    let secp = Secp256k1::new();
    let sk = SecretKey::from_slice(secret_bytes).map_err(|e| WalletError::DerivationFailed(e.to_string()))?;
    let pk = PublicKey::from_secret_key(&secp, &sk);
    let hash = bitcoin::hashes::hash160::Hash::hash(&pk.serialize());
    let mut payload = vec![0x30];
    payload.extend_from_slice(hash.as_ref());
    Ok(bs58::encode(&payload).with_check().into_string())
}

pub fn secret_to_doge_address(secret_bytes: &[u8]) -> Result<String, WalletError> {
    let secp = Secp256k1::new();
    let sk = SecretKey::from_slice(secret_bytes).map_err(|e| WalletError::DerivationFailed(e.to_string()))?;
    let pk = PublicKey::from_secret_key(&secp, &sk);
    let hash = bitcoin::hashes::hash160::Hash::hash(&pk.serialize());
    let mut payload = vec![0x1E];
    payload.extend_from_slice(hash.as_ref());
    Ok(bs58::encode(&payload).with_check().into_string())
}

pub fn secret_to_pubkey_hex(secret_bytes: &[u8]) -> Result<String, WalletError> {
    let secp = Secp256k1::new();
    let sk = SecretKey::from_slice(secret_bytes).map_err(|e| WalletError::DerivationFailed(e.to_string()))?;
    let pk = PublicKey::from_secret_key(&secp, &sk);
    Ok(hex::encode(pk.serialize()))
}

pub fn sign_hash(secret_bytes: &[u8], hash: &[u8]) -> Result<Vec<u8>, WalletError> {
    let secp = Secp256k1::new();
    let sk = SecretKey::from_slice(secret_bytes).map_err(|e| WalletError::SigningFailed(e.to_string()))?;
    let msg = bitcoin::secp256k1::Message::from_digest_slice(hash).map_err(|e| WalletError::SigningFailed(e.to_string()))?;
    let sig = secp.sign_ecdsa(&msg, &sk);
    Ok(sig.serialize_der().to_vec())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Utxo {
    pub txid: String,
    pub vout: u32,
    pub value: u64,
    #[serde(default)]
    pub script_pubkey: String,
    #[serde(default)]
    pub status: serde_json::Value,
}

pub fn build_and_sign_btc_tx(
    secret: &[u8], utxos: &[Utxo], to_address: &str, amount_sats: u64,
    fee_rate_sat_vb: u64, network: bitcoin::Network,
) -> Result<(Vec<u8>, String), WalletError> {
    use bitcoin::{absolute::LockTime, sighash::{EcdsaSighashType, SighashCache}, transaction::Version, Amount, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid, Witness};
    use std::str::FromStr;

    let secp = Secp256k1::new();
    let sk = SecretKey::from_slice(secret).map_err(|e| WalletError::SigningFailed(e.to_string()))?;
    let pk = PublicKey::from_secret_key(&secp, &sk);
    let compressed = pk.serialize();
    let pubkey_hash = bitcoin::hashes::hash160::Hash::hash(&compressed);
    let our_wpkh = bitcoin::WitnessProgram::new(bitcoin::WitnessVersion::V0, pubkey_hash.as_ref())
        .map_err(|e| WalletError::DerivationFailed(e.to_string()))?;
    let our_addr = bitcoin::Address::from_witness_program(our_wpkh, network);
    let our_script = our_addr.script_pubkey();

    let recipient = bitcoin::Address::from_str(to_address)
        .map_err(|e| WalletError::InvalidAddress(e.to_string()))?
        .require_network(network).map_err(|e| WalletError::InvalidAddress(e.to_string()))?;

    let mut sorted = utxos.to_vec();
    sorted.sort_by(|a, b| b.value.cmp(&a.value));

    let mut inputs: Vec<TxIn> = Vec::new();
    let mut input_vals: Vec<u64> = Vec::new();
    let mut total_in: u64 = 0;
    for utxo in &sorted {
        let txid = Txid::from_str(&utxo.txid).map_err(|e| WalletError::Internal(e.to_string()))?;
        inputs.push(TxIn { previous_output: OutPoint::new(txid, utxo.vout), script_sig: ScriptBuf::new(), sequence: Sequence::ENABLE_RBF_NO_LOCKTIME, witness: Witness::default() });
        input_vals.push(utxo.value);
        total_in += utxo.value;
        let est_vb = (inputs.len() as u64) * 68 + 2 * 31 + 11;
        if total_in >= amount_sats + est_vb * fee_rate_sat_vb { break; }
    }

    let fee = ((inputs.len() as u64) * 68 + 2 * 31 + 11) * fee_rate_sat_vb;
    if total_in < amount_sats + fee {
        return Err(WalletError::InsufficientBalance { have: format!("{} sats", total_in), need: format!("{} sats", amount_sats + fee) });
    }
    let change = total_in - amount_sats - fee;

    let mut outputs = vec![TxOut { value: Amount::from_sat(amount_sats), script_pubkey: recipient.script_pubkey() }];
    if change > 546 { outputs.push(TxOut { value: Amount::from_sat(change), script_pubkey: our_script.clone() }); }

    let mut tx = Transaction { version: Version::TWO, lock_time: LockTime::ZERO, input: inputs, output: outputs };
    let n = tx.input.len();
    let mut cache = SighashCache::new(&mut tx);
    for i in 0..n {
        let sh = cache.p2wpkh_signature_hash(i, &our_script, Amount::from_sat(input_vals[i]), EcdsaSighashType::All)
            .map_err(|e| WalletError::SigningFailed(e.to_string()))?;
        let msg = bitcoin::secp256k1::Message::from_digest(sh.to_byte_array());
        let sig = secp.sign_ecdsa(&msg, &sk);
        let mut sb = sig.serialize_der().to_vec();
        sb.push(EcdsaSighashType::All as u8);
        *cache.witness_mut(i).unwrap() = Witness::from_slice(&[&sb[..], &compressed[..]]);
    }
    let stx = cache.into_transaction();
    let raw = bitcoin::consensus::encode::serialize(stx);
    let txid = stx.compute_txid().to_string();
    Ok((raw, txid))
}
