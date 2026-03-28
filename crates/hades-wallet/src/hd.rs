use crate::error::WalletError;
use bip32::{DerivationPath, XPrv};
use bip39::{Language, Mnemonic};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use zeroize::ZeroizeOnDrop;

/// BIP-44 coin types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Chain {
    Bitcoin,
    Ethereum,
    Solana,
    Litecoin,
    Dogecoin,
    BnbSmartChain,
    Polygon,
    Arbitrum,
    Optimism,
    Avalanche,
    Base,
    Tron,
}

impl Chain {
    /// BIP-44 coin type number
    pub fn coin_type(&self) -> u32 {
        match self {
            Chain::Bitcoin => 0,
            Chain::Litecoin => 2,
            Chain::Dogecoin => 3,
            Chain::Ethereum
            | Chain::Polygon
            | Chain::Arbitrum
            | Chain::Optimism
            | Chain::Avalanche
            | Chain::Base
            | Chain::BnbSmartChain => 60,
            Chain::Solana => 501,
            Chain::Tron => 195,
        }
    }

    /// Chain ID for EVM networks
    pub fn chain_id(&self) -> Option<u64> {
        match self {
            Chain::Ethereum => Some(1),
            Chain::Polygon => Some(137),
            Chain::Arbitrum => Some(42161),
            Chain::Optimism => Some(10),
            Chain::Avalanche => Some(43114),
            Chain::Base => Some(8453),
            Chain::BnbSmartChain => Some(56),
            _ => None,
        }
    }

    pub fn is_evm(&self) -> bool {
        self.chain_id().is_some()
    }

    pub fn ticker(&self) -> &str {
        match self {
            Chain::Bitcoin => "BTC",
            Chain::Ethereum => "ETH",
            Chain::Polygon => "POL",
            Chain::Arbitrum | Chain::Optimism | Chain::Base => "ETH",
            Chain::Avalanche => "AVAX",
            Chain::BnbSmartChain => "BNB",
            Chain::Solana => "SOL",
            Chain::Litecoin => "LTC",
            Chain::Dogecoin => "DOGE",
            Chain::Tron => "TRX",
        }
    }

    pub fn decimals(&self) -> u8 {
        match self {
            Chain::Bitcoin | Chain::Litecoin | Chain::Dogecoin => 8,
            Chain::Solana => 9,
            Chain::Tron => 6,
            _ => 18,
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            Chain::Bitcoin => "Bitcoin",
            Chain::Ethereum => "Ethereum",
            Chain::Polygon => "Polygon",
            Chain::Arbitrum => "Arbitrum",
            Chain::Optimism => "Optimism",
            Chain::Avalanche => "Avalanche",
            Chain::Base => "Base",
            Chain::BnbSmartChain => "BNB Smart Chain",
            Chain::Solana => "Solana",
            Chain::Litecoin => "Litecoin",
            Chain::Dogecoin => "Dogecoin",
            Chain::Tron => "Tron",
        }
    }

    /// RPC endpoint (user can override)
    pub fn default_rpc(&self) -> &str {
        match self {
            Chain::Bitcoin => "https://blockstream.info/api",
            Chain::Ethereum => "https://eth.llamarpc.com",
            Chain::Polygon => "https://polygon-rpc.com",
            Chain::Arbitrum => "https://arb1.arbitrum.io/rpc",
            Chain::Optimism => "https://mainnet.optimism.io",
            Chain::Avalanche => "https://api.avax.network/ext/bc/C/rpc",
            Chain::Base => "https://mainnet.base.org",
            Chain::BnbSmartChain => "https://bsc-dataseed.binance.org",
            Chain::Solana => "https://api.mainnet-beta.solana.com",
            Chain::Litecoin => "https://litecoin.llamarpc.com",
            Chain::Dogecoin => "https://dogechain.info/api/v1",
            Chain::Tron => "https://api.trongrid.io",
        }
    }

    /// Explorer URL for a transaction hash
    pub fn explorer_tx_url(&self, tx_hash: &str) -> String {
        match self {
            Chain::Bitcoin => format!("https://mempool.space/tx/{}", tx_hash),
            Chain::Ethereum => format!("https://etherscan.io/tx/{}", tx_hash),
            Chain::Polygon => format!("https://polygonscan.com/tx/{}", tx_hash),
            Chain::Arbitrum => format!("https://arbiscan.io/tx/{}", tx_hash),
            Chain::Optimism => {
                format!("https://optimistic.etherscan.io/tx/{}", tx_hash)
            }
            Chain::Base => format!("https://basescan.org/tx/{}", tx_hash),
            Chain::BnbSmartChain => format!("https://bscscan.com/tx/{}", tx_hash),
            Chain::Avalanche => format!("https://snowtrace.io/tx/{}", tx_hash),
            Chain::Solana => format!("https://solscan.io/tx/{}", tx_hash),
            Chain::Litecoin => {
                format!("https://blockchair.com/litecoin/transaction/{}", tx_hash)
            }
            Chain::Dogecoin => {
                format!("https://blockchair.com/dogecoin/transaction/{}", tx_hash)
            }
            Chain::Tron => format!("https://tronscan.org/#/transaction/{}", tx_hash),
        }
    }

    pub fn all() -> Vec<Chain> {
        vec![
            Chain::Bitcoin,
            Chain::Ethereum,
            Chain::Solana,
            Chain::Polygon,
            Chain::Arbitrum,
            Chain::Optimism,
            Chain::Avalanche,
            Chain::Base,
            Chain::BnbSmartChain,
            Chain::Litecoin,
            Chain::Dogecoin,
            Chain::Tron,
        ]
    }
}

/// Secret key material — zeroized on drop
#[derive(ZeroizeOnDrop)]
pub struct DerivedSecret {
    #[zeroize(skip)]
    _guard: (),
    pub bytes: Vec<u8>,
}

/// HD Wallet: single seed → all chains
#[derive(ZeroizeOnDrop)]
pub struct HdWallet {
    #[zeroize(skip)]
    mnemonic_phrase: String,
    seed: [u8; 64],
}

/// Account address for a specific chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletAccount {
    pub chain: Chain,
    pub address: String,
    pub derivation_path: String,
    pub public_key_hex: String,
}

impl HdWallet {
    /// Create a new HD wallet with a fresh 24-word mnemonic
    pub fn generate() -> Result<Self, WalletError> {
        let mnemonic = Mnemonic::generate_in(Language::English, 24)
            .map_err(|e| WalletError::DerivationFailed(e.to_string()))?;

        let seed = mnemonic.to_seed("");
        let mut seed_arr = [0u8; 64];
        seed_arr.copy_from_slice(&seed[..64]);

        Ok(Self {
            mnemonic_phrase: mnemonic.to_string(),
            seed: seed_arr,
        })
    }

    /// Restore from existing mnemonic (from Hades recovery phrase or import)
    pub fn from_mnemonic(phrase: &str) -> Result<Self, WalletError> {
        let mnemonic = Mnemonic::parse_in(Language::English, phrase)
            .map_err(|e| WalletError::InvalidMnemonic(e.to_string()))?;

        let seed = mnemonic.to_seed("");
        let mut seed_arr = [0u8; 64];
        seed_arr.copy_from_slice(&seed[..64]);

        Ok(Self {
            mnemonic_phrase: mnemonic.to_string(),
            seed: seed_arr,
        })
    }

    /// Derive from same seed as Hades identity
    pub fn from_seed(seed: &[u8; 64]) -> Self {
        Self {
            mnemonic_phrase: String::new(),
            seed: *seed,
        }
    }

    pub fn mnemonic(&self) -> &str {
        &self.mnemonic_phrase
    }

    /// Derive raw 32-byte secret key for a chain + account index.
    /// Returns `DerivedSecret` which is zeroized on drop.
    pub fn derive_secret(
        &self,
        chain: Chain,
        account: u32,
    ) -> Result<DerivedSecret, WalletError> {
        let path_str = format!("m/44'/{}'/{}'/0/0", chain.coin_type(), account);
        let path = DerivationPath::from_str(&path_str)
            .map_err(|e| WalletError::DerivationFailed(e.to_string()))?;

        let child_key = XPrv::derive_from_path(&self.seed, &path)
            .map_err(|e| WalletError::DerivationFailed(e.to_string()))?;

        Ok(DerivedSecret {
            _guard: (),
            bytes: child_key.to_bytes().to_vec(),
        })
    }

    /// Derive a private key for a specific chain and account index
    /// Path: m/44'/{coin_type}'/{account}'/0/0
    /// Kept for backward compatibility — prefer `derive_secret()`.
    pub fn derive_secret_key(
        &self,
        chain: Chain,
        account: u32,
    ) -> Result<Vec<u8>, WalletError> {
        let secret = self.derive_secret(chain, account)?;
        Ok(secret.bytes.clone())
    }

    /// Get the address for a chain
    pub fn derive_account(
        &self,
        chain: Chain,
        account: u32,
    ) -> Result<WalletAccount, WalletError> {
        let secret_bytes = self.derive_secret_key(chain, account)?;

        match chain {
            Chain::Bitcoin => {
                let address =
                    crate::chains::bitcoin::secret_to_address(&secret_bytes, false)?;
                let pubkey_hex =
                    crate::chains::bitcoin::secret_to_pubkey_hex(&secret_bytes)?;
                Ok(WalletAccount {
                    chain: Chain::Bitcoin,
                    address,
                    derivation_path: format!("m/44'/0'/{}'/0/0", account),
                    public_key_hex: pubkey_hex,
                })
            }
            Chain::Solana => {
                let address =
                    crate::chains::solana::secret_to_address(&secret_bytes)?;
                let pubkey_hex =
                    crate::chains::solana::secret_to_pubkey_hex(&secret_bytes)?;
                Ok(WalletAccount {
                    chain: Chain::Solana,
                    address,
                    derivation_path: format!("m/44'/501'/{}'/0/0", account),
                    public_key_hex: pubkey_hex,
                })
            }
            Chain::Litecoin => {
                let address =
                    crate::chains::bitcoin::secret_to_ltc_address(&secret_bytes)?;
                let pubkey_hex =
                    crate::chains::bitcoin::secret_to_pubkey_hex(&secret_bytes)?;
                Ok(WalletAccount {
                    chain: Chain::Litecoin,
                    address,
                    derivation_path: format!("m/44'/2'/{}'/0/0", account),
                    public_key_hex: pubkey_hex,
                })
            }
            Chain::Dogecoin => {
                let address =
                    crate::chains::bitcoin::secret_to_doge_address(&secret_bytes)?;
                let pubkey_hex =
                    crate::chains::bitcoin::secret_to_pubkey_hex(&secret_bytes)?;
                Ok(WalletAccount {
                    chain: Chain::Dogecoin,
                    address,
                    derivation_path: format!("m/44'/3'/{}'/0/0", account),
                    public_key_hex: pubkey_hex,
                })
            }
            Chain::Tron => {
                let address =
                    crate::chains::ethereum::secret_to_tron_address(&secret_bytes)?;
                let pubkey_hex =
                    crate::chains::ethereum::secret_to_pubkey_hex(&secret_bytes)?;
                Ok(WalletAccount {
                    chain: Chain::Tron,
                    address,
                    derivation_path: format!("m/44'/195'/{}'/0/0", account),
                    public_key_hex: pubkey_hex,
                })
            }
            // All EVM chains
            _ => {
                let address =
                    crate::chains::ethereum::secret_to_address(&secret_bytes)?;
                let pubkey_hex =
                    crate::chains::ethereum::secret_to_pubkey_hex(&secret_bytes)?;
                Ok(WalletAccount {
                    chain,
                    address,
                    derivation_path: format!("m/44'/60'/{}'/0/0", account),
                    public_key_hex: pubkey_hex,
                })
            }
        }
    }

    /// Derive all default accounts (one per chain)
    pub fn derive_all_accounts(&self) -> Vec<WalletAccount> {
        Chain::all()
            .into_iter()
            .filter_map(|chain| self.derive_account(chain, 0).ok())
            .collect()
    }

    /// Sign raw data for a specific chain
    pub fn sign(
        &self,
        chain: Chain,
        account: u32,
        data: &[u8],
    ) -> Result<Vec<u8>, WalletError> {
        let secret = self.derive_secret_key(chain, account)?;
        match chain {
            Chain::Bitcoin | Chain::Litecoin | Chain::Dogecoin => {
                crate::chains::bitcoin::sign_hash(&secret, data)
            }
            Chain::Solana => crate::chains::solana::sign_message(&secret, data),
            // All EVM chains + Tron use secp256k1 signing
            _ => crate::chains::ethereum::sign_hash(&secret, data),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_derive_all() {
        let wallet = HdWallet::generate().unwrap();
        let accounts = wallet.derive_all_accounts();
        assert!(accounts.len() >= 10);

        for acc in &accounts {
            assert!(!acc.address.is_empty());
            println!("{}: {}", acc.chain.display_name(), acc.address);
        }
    }

    #[test]
    fn test_restore_from_mnemonic() {
        let wallet1 = HdWallet::generate().unwrap();
        let phrase = wallet1.mnemonic().to_string();

        let wallet2 = HdWallet::from_mnemonic(&phrase).unwrap();

        let addr1 = wallet1
            .derive_account(Chain::Ethereum, 0)
            .unwrap()
            .address;
        let addr2 = wallet2
            .derive_account(Chain::Ethereum, 0)
            .unwrap()
            .address;
        assert_eq!(addr1, addr2);
    }

    #[test]
    fn test_different_chains_different_addresses() {
        let wallet = HdWallet::generate().unwrap();
        let eth = wallet.derive_account(Chain::Ethereum, 0).unwrap();
        let btc = wallet.derive_account(Chain::Bitcoin, 0).unwrap();
        let sol = wallet.derive_account(Chain::Solana, 0).unwrap();

        assert_ne!(eth.address, btc.address);
        assert_ne!(eth.address, sol.address);
        assert_ne!(btc.address, sol.address);
    }

    #[test]
    fn test_derive_secret_zeroize() {
        let wallet = HdWallet::generate().unwrap();
        let secret = wallet.derive_secret(Chain::Ethereum, 0).unwrap();
        assert_eq!(secret.bytes.len(), 32);
    }

    #[test]
    fn test_chain_decimals() {
        assert_eq!(Chain::Bitcoin.decimals(), 8);
        assert_eq!(Chain::Ethereum.decimals(), 18);
        assert_eq!(Chain::Solana.decimals(), 9);
        assert_eq!(Chain::Tron.decimals(), 6);
    }
}
