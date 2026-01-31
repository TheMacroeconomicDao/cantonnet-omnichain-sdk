// HD wallet implementation for Canton Wallet SDK

use bip32::{Mnemonic as Bip32Mnemonic, XPrv, XPub, DerivationPath, ExtendedPrivateKey, ExtendedPublicKey};
use bip39::{Mnemonic, MnemonicType, Language, Seed};
use canton_wallet_core::{KeyAlgorithm, KeyId, KeyMetadata, KeyPurpose, PublicKey, WalletError, WalletResult};
use ed25519_dalek::{Keypair, PublicKey as Ed25519PublicKey, SecretKey};
use rand::rngs::OsRng;
use rand_core::RngCore;
use std::collections::HashMap;
use zeroize::Zeroize;

/// HD wallet implementation
pub struct HDWallet {
    mnemonic: Mnemonic,
    root_key: XPrv,
    accounts: HashMap<u32, HDAccount>,
}

impl HDWallet {
    /// Create new HD wallet
    ///
    /// # Arguments
    ///
    /// * `word_count` - Number of words in mnemonic (12, 15, 18, 21, or 24)
    pub fn new(word_count: MnemonicType) -> WalletResult<Self> {
        let mnemonic = Mnemonic::new(word_count, Language::English);
        let seed = Seed::new(&mnemonic, "");
        
        let root_key = XPrv::new(seed.as_bytes())
            .map_err(|e| WalletError::DerivationError(e.to_string()))?;
        
        Ok(Self {
            mnemonic,
            root_key,
            accounts: HashMap::new(),
        })
    }

    /// Restore HD wallet from mnemonic phrase
    ///
    /// # Arguments
    ///
    /// * `mnemonic_phrase` - BIP39 mnemonic phrase
    pub fn from_mnemonic(mnemonic_phrase: &str) -> WalletResult<Self> {
        let mnemonic = Mnemonic::from_phrase(mnemonic_phrase, Language::English)
            .map_err(|e| WalletError::InvalidMnemonic(e.to_string()))?;
        
        let seed = Seed::new(&mnemonic, "");
        
        let root_key = XPrv::new(seed.as_bytes())
            .map_err(|e| WalletError::DerivationError(e.to_string()))?;
        
        Ok(Self {
            mnemonic,
            root_key,
            accounts: HashMap::new(),
        })
    }

    /// Derive account at index
    ///
    /// # Arguments
    ///
    /// * `index` - Account index
    pub fn derive_account(&mut self, index: u32) -> WalletResult<&HDAccount> {
        if self.accounts.contains_key(&index) {
            return Ok(&self.accounts[&index]);
        }
        
        // BIP44 path: m/44'/118'/account'/0/0
        let path = DerivationPath::from_str(&format!(
            "m/44'/118'/{}'/0/0",
            index
        ))
        .map_err(|e| WalletError::InvalidDerivationPath(e.to_string()))?;
        
        let account_key = self.root_key
            .derive_path(&path)
            .map_err(|e| WalletError::DerivationError(e.to_string()))?;
        
        let account = HDAccount {
            index,
            private_key: account_key,
            public_key: account_key.public_key(),
        };
        
        self.accounts.insert(index, account);
        Ok(&self.accounts[&index])
    }

    /// Get mnemonic phrase
    ///
    /// # Warning
    ///
    /// Never share or log this phrase!
    pub fn mnemonic_phrase(&self) -> &str {
        self.mnemonic.phrase()
    }

    /// Get root public key
    pub fn root_public_key(&self) -> XPub {
        self.root_key.public_key()
    }

    /// Get all derived accounts
    pub fn accounts(&self) -> &HashMap<u32, HDAccount> {
        &self.accounts
    }
}

/// HD account
#[derive(Debug, Clone)]
pub struct HDAccount {
    pub index: u32,
    pub private_key: XPrv,
    pub public_key: XPub,
}

impl HDAccount {
    /// Get account index
    pub fn index(&self) -> u32 {
        self.index
    }

    /// Get account private key
    pub fn private_key(&self) -> XPrv {
        self.private_key.clone()
    }

    /// Get account public key
    pub fn public_key(&self) -> XPub {
        self.public_key.clone()
    }

    /// Derive child key
    pub fn derive_child(&self, index: u32) -> WalletResult<HDAccount> {
        let child_key = self.private_key
            .derive_child(index.into())
            .map_err(|e| WalletError::DerivationError(e.to_string()))?;
        
        Ok(HDAccount {
            index,
            private_key: child_key,
            public_key: child_key.public_key(),
        })
    }

    /// Get Ed25519 key pair from this account
    pub fn to_ed25519_keypair(&self) -> WalletResult<Keypair> {
        let private_bytes = self.private_key.private_key();
        let secret = SecretKey::from_bytes(private_bytes)
            .map_err(|e| WalletError::DerivationError(e.to_string()))?;
        let public = Ed25519PublicKey::from(&secret);
        Ok(Keypair { secret, public })
    }

    /// Get public key as PublicKey type
    pub fn to_public_key(&self) -> WalletResult<PublicKey> {
        let keypair = self.to_ed25519_keypair()?;
        Ok(PublicKey::new(
            keypair.public.to_bytes().to_vec(),
            "ed25519",
        ))
    }
}

/// HD wallet utilities
pub struct HDWalletUtils;

impl HDWalletUtils {
    /// Generate a new mnemonic
    ///
    /// # Arguments
    ///
    /// * `word_count` - Number of words (12, 15, 18, 21, or 24)
    pub fn generate_mnemonic(word_count: MnemonicType) -> WalletResult<String> {
        let mnemonic = Mnemonic::new(word_count, Language::English);
        Ok(mnemonic.phrase().to_string())
    }

    /// Validate a mnemonic phrase
    ///
    /// # Arguments
    ///
    /// * `mnemonic_phrase` - BIP39 mnemonic phrase to validate
    pub fn validate_mnemonic(mnemonic_phrase: &str) -> WalletResult<bool> {
        Mnemonic::from_phrase(mnemonic_phrase, Language::English)
            .map(|_| true)
            .map_err(|_| false)
            .map_err(|_| WalletError::InvalidMnemonic("Invalid mnemonic".to_string()))
    }

    /// Generate seed from mnemonic
    ///
    /// # Arguments
    ///
    /// * `mnemonic_phrase` - BIP39 mnemonic phrase
    /// * `passphrase` - Optional passphrase (empty string for none)
    pub fn generate_seed(mnemonic_phrase: &str, passphrase: &str) -> WalletResult<Vec<u8>> {
        let mnemonic = Mnemonic::from_phrase(mnemonic_phrase, Language::English)
            .map_err(|e| WalletError::InvalidMnemonic(e.to_string()))?;
        let seed = Seed::new(&mnemonic, passphrase);
        Ok(seed.as_bytes().to_vec())
    }

    /// Derive key from path
    ///
    /// # Arguments
    ///
    /// * `seed` - Seed bytes
    /// * `path` - Derivation path (e.g., "m/44'/118'/0'/0/0")
    pub fn derive_from_path(seed: &[u8], path: &str) -> WalletResult<XPrv> {
        let root_key = XPrv::new(seed)
            .map_err(|e| WalletError::DerivationError(e.to_string()))?;
        
        let derivation_path = DerivationPath::from_str(path)
            .map_err(|e| WalletError::InvalidDerivationPath(e.to_string()))?;
        
        root_key
            .derive_path(&derivation_path)
            .map_err(|e| WalletError::DerivationError(e.to_string()))
    }

    /// Generate random entropy for mnemonic
    ///
    /// # Arguments
    ///
    /// * `word_count` - Number of words (determines entropy size)
    pub fn generate_entropy(word_count: MnemonicType) -> WalletResult<Vec<u8>> {
        let entropy_size = match word_count {
            MnemonicType::Words12 => 16,
            MnemonicType::Words15 => 20,
            MnemonicType::Words18 => 24,
            MnemonicType::Words21 => 28,
            MnemonicType::Words24 => 32,
        };
        
        let mut entropy = vec![0u8; entropy_size];
        let mut rng = OsRng;
        rng.fill_bytes(&mut entropy);
        Ok(entropy)
    }
}

/// Secure mnemonic storage
#[derive(Debug, Clone)]
pub struct SecureMnemonic {
    pub mnemonic: String,
    pub passphrase: Option<String>,
}

impl SecureMnemonic {
    /// Create a new secure mnemonic
    ///
    /// # Arguments
    ///
    /// * `mnemonic` - BIP39 mnemonic phrase
    /// * `passphrase` - Optional passphrase
    pub fn new(mnemonic: impl Into<String>, passphrase: Option<impl Into<String>>) -> Self {
        Self {
            mnemonic: mnemonic.into(),
            passphrase: passphrase.map(|p| p.into()),
        }
    }

    /// Get mnemonic phrase
    ///
    /// # Warning
    ///
    /// Never share or log this!
    pub fn mnemonic(&self) -> &str {
        &self.mnemonic
    }

    /// Get passphrase
    pub fn passphrase(&self) -> Option<&str> {
        self.passphrase.as_deref()
    }

    /// Zeroize mnemonic and passphrase
    pub fn zeroize(&mut self) {
        self.mnemonic.zeroize();
        if let Some(ref mut passphrase) = self.passphrase {
            passphrase.zeroize();
        }
    }
}

impl Drop for SecureMnemonic {
    fn drop(&mut self) {
        self.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hd_wallet_new() {
        let wallet = HDWallet::new(MnemonicType::Words12).unwrap();
        let phrase = wallet.mnemonic_phrase();
        assert!(!phrase.is_empty());
        assert_eq!(phrase.split_whitespace().count(), 12);
    }

    #[test]
    fn test_hd_wallet_from_mnemonic() {
        let wallet1 = HDWallet::new(MnemonicType::Words12).unwrap();
        let phrase = wallet1.mnemonic_phrase();
        
        let wallet2 = HDWallet::from_mnemonic(phrase).unwrap();
        assert_eq!(wallet2.mnemonic_phrase(), phrase);
    }

    #[test]
    fn test_hd_wallet_derive_account() {
        let mut wallet = HDWallet::new(MnemonicType::Words12).unwrap();
        let account = wallet.derive_account(0).unwrap();
        assert_eq!(account.index(), 0);
    }

    #[test]
    fn test_hd_wallet_derive_multiple_accounts() {
        let mut wallet = HDWallet::new(MnemonicType::Words12).unwrap();
        let account0 = wallet.derive_account(0).unwrap();
        let account1 = wallet.derive_account(1).unwrap();
        let account2 = wallet.derive_account(2).unwrap();
        
        assert_eq!(account0.index(), 0);
        assert_eq!(account1.index(), 1);
        assert_eq!(account2.index(), 2);
        
        assert_ne!(account0.private_key(), account1.private_key());
        assert_ne!(account1.private_key(), account2.private_key());
    }

    #[test]
    fn test_hd_wallet_same_account() {
        let mut wallet = HDWallet::new(MnemonicType::Words12).unwrap();
        let account1 = wallet.derive_account(0).unwrap();
        let account2 = wallet.derive_account(0).unwrap();
        
        assert_eq!(account1.private_key(), account2.private_key());
    }

    #[test]
    fn test_hd_wallet_root_public_key() {
        let wallet = HDWallet::new(MnemonicType::Words12).unwrap();
        let public_key = wallet.root_public_key();
        assert!(!public_key.public_key().is_empty());
    }

    #[test]
    fn test_hd_account_to_ed25519_keypair() {
        let mut wallet = HDWallet::new(MnemonicType::Words12).unwrap();
        let account = wallet.derive_account(0).unwrap();
        let keypair = account.to_ed25519_keypair().unwrap();
        
        assert_eq!(keypair.secret.as_bytes().len(), 32);
        assert_eq!(keypair.public.to_bytes().len(), 32);
    }

    #[test]
    fn test_hd_account_to_public_key() {
        let mut wallet = HDWallet::new(MnemonicType::Words12).unwrap();
        let account = wallet.derive_account(0).unwrap();
        let public_key = account.to_public_key().unwrap();
        
        assert_eq!(public_key.algorithm, "ed25519");
        assert_eq!(public_key.as_bytes().len(), 32);
    }

    #[test]
    fn test_hd_wallet_utils_generate_mnemonic() {
        let mnemonic = HDWalletUtils::generate_mnemonic(MnemonicType::Words12).unwrap();
        assert_eq!(mnemonic.split_whitespace().count(), 12);
    }

    #[test]
    fn test_hd_wallet_utils_validate_mnemonic() {
        let mnemonic = HDWalletUtils::generate_mnemonic(MnemonicType::Words12).unwrap();
        let valid = HDWalletUtils::validate_mnemonic(&mnemonic).unwrap();
        assert!(valid);
        
        let invalid = HDWalletUtils::validate_mnemonic("invalid mnemonic phrase").unwrap();
        assert!(!invalid);
    }

    #[test]
    fn test_hd_wallet_utils_generate_seed() {
        let mnemonic = HDWalletUtils::generate_mnemonic(MnemonicType::Words12).unwrap();
        let seed = HDWalletUtils::generate_seed(&mnemonic, "").unwrap();
        assert_eq!(seed.len(), 64);
    }

    #[test]
    fn test_hd_wallet_utils_derive_from_path() {
        let mnemonic = HDWalletUtils::generate_mnemonic(MnemonicType::Words12).unwrap();
        let seed = HDWalletUtils::generate_seed(&mnemonic, "").unwrap();
        let key = HDWalletUtils::derive_from_path(&seed, "m/44'/118'/0'/0/0").unwrap();
        assert!(!key.private_key().is_empty());
    }

    #[test]
    fn test_hd_wallet_utils_generate_entropy() {
        let entropy1 = HDWalletUtils::generate_entropy(MnemonicType::Words12).unwrap();
        let entropy2 = HDWalletUtils::generate_entropy(MnemonicType::Words12).unwrap();
        
        assert_eq!(entropy1.len(), 16);
        assert_eq!(entropy2.len(), 16);
        assert_ne!(entropy1, entropy2);
    }

    #[test]
    fn test_secure_mnemonic() {
        let mnemonic = HDWalletUtils::generate_mnemonic(MnemonicType::Words12).unwrap();
        let secure = SecureMnemonic::new(mnemonic, None::<String>);
        
        assert_eq!(secure.mnemonic(), mnemonic);
        assert!(secure.passphrase().is_none());
    }

    #[test]
    fn test_secure_mnemonic_with_passphrase() {
        let mnemonic = HDWalletUtils::generate_mnemonic(MnemonicType::Words12).unwrap();
        let secure = SecureMnemonic::new(mnemonic, Some("test-passphrase"));
        
        assert_eq!(secure.passphrase(), Some("test-passphrase"));
    }

    #[test]
    fn test_secure_mnemonic_zeroize() {
        let mnemonic = HDWalletUtils::generate_mnemonic(MnemonicType::Words12).unwrap();
        let mut secure = SecureMnemonic::new(mnemonic.clone(), None::<String>);
        
        secure.zeroize();
        assert_ne!(secure.mnemonic(), mnemonic);
    }
}
