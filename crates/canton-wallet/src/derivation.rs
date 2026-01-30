//! Гибкая деривация ключей: одна мнемоника → несколько сетей или отдельный источник на сеть.
//! See research/10-flexible-key-derivation.md.

use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use canton_crypto::keystore::KeyFingerprint;
use thiserror::Error;

// -----------------------------------------------------------------------------
// NetworkId
// -----------------------------------------------------------------------------

/// Идентификатор сети в OmniChain SDK (не путать с EVM ChainId).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NetworkId {
    Canton,
    Ethereum,
    Cosmos,
    Substrate,
    Custom(String),
}

impl fmt::Display for NetworkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkId::Canton => write!(f, "canton"),
            NetworkId::Ethereum => write!(f, "ethereum"),
            NetworkId::Cosmos => write!(f, "cosmos"),
            NetworkId::Substrate => write!(f, "substrate"),
            NetworkId::Custom(s) => write!(f, "{}", s),
        }
    }
}

impl FromStr for NetworkId {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "canton" => NetworkId::Canton,
            "ethereum" | "evm" => NetworkId::Ethereum,
            "cosmos" => NetworkId::Cosmos,
            "substrate" => NetworkId::Substrate,
            _ => NetworkId::Custom(s.to_string()),
        })
    }
}

// -----------------------------------------------------------------------------
// DerivationPath
// -----------------------------------------------------------------------------

/// BIP-32/44 путь деривации. Формат: `m/44'/60'/0'/0/0` и т.п.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DerivationPath {
    path: String,
}

impl DerivationPath {
    /// Создаёт путь из строки. Валидирует базовый формат (начинается с `m/`).
    pub fn new(path: impl Into<String>) -> Result<Self, DerivationError> {
        let path = path.into();
        if !path.starts_with("m/") && !path.starts_with("m'") {
            return Err(DerivationError::InvalidPath(
                "path must start with m/ or m'".into(),
            ));
        }
        Ok(Self { path })
    }

    /// Путь как строка.
    pub fn as_str(&self) -> &str {
        &self.path
    }

    /// Стандартный путь для Ethereum (BIP-44): m/44'/60'/0'/0/0
    pub fn ethereum_default() -> Self {
        Self {
            path: "m/44'/60'/0'/0/0".to_string(),
        }
    }

    /// Стандартный путь для Cosmos: m/44'/118'/0'/0/0
    pub fn cosmos_default() -> Self {
        Self {
            path: "m/44'/118'/0'/0/0".to_string(),
        }
    }

    /// Canton: по документации часто используют первые 32 байта seed (нет стандартного BIP-44).
    /// Этот путь — запасной для будущей стандартизации; при использовании мнемоники seed можно брать без path.
    pub fn canton_default() -> Self {
        Self {
            path: "m/44'/1022'/0'/0/0".to_string(),
        }
    }
}

impl FromStr for DerivationPath {
    type Err = DerivationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl fmt::Display for DerivationPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path)
    }
}

#[derive(Debug, Error)]
pub enum DerivationError {
    #[error("invalid derivation path: {0}")]
    InvalidPath(String),
    #[error("unsupported network for derivation: {0}")]
    UnsupportedNetwork(String),
    #[error("key derivation failed: {0}")]
    DerivationFailed(String),
}

// -----------------------------------------------------------------------------
// IdentitySource
// -----------------------------------------------------------------------------

/// Источник ключа для данной сети: мнемоника + путь, KeyStore по fingerprint или HSM.
#[derive(Debug, Clone)]
pub enum IdentitySource {
    /// Одна мнемоника, путь деривации для этой сети.
    /// В production предпочтительно оборачивать мнемонику в тип с zeroize (secrecy/zeroize).
    Mnemonic {
        mnemonic_phrase: String,
        derivation_path: DerivationPath,
    },
    /// Ключ уже в KeyStore по отпечатку (импорт или сгенерированный).
    KeyStore { fingerprint: KeyFingerprint },
    /// Внешний HSM/KMS по идентификатору ключа.
    Hsm { key_id: String },
}

impl IdentitySource {
    pub fn from_mnemonic(phrase: impl Into<String>, path: DerivationPath) -> Self {
        IdentitySource::Mnemonic {
            mnemonic_phrase: phrase.into(),
            derivation_path: path,
        }
    }

    pub fn from_keystore(fingerprint: KeyFingerprint) -> Self {
        IdentitySource::KeyStore { fingerprint }
    }

    pub fn from_hsm(key_id: impl Into<String>) -> Self {
        IdentitySource::Hsm {
            key_id: key_id.into(),
        }
    }
}

// -----------------------------------------------------------------------------
// DerivationStrategy
// -----------------------------------------------------------------------------

/// Стратегия конфигурации кошелька: одна мнемоника на все сети или отдельный источник на сеть.
#[derive(Debug, Clone)]
pub enum DerivationStrategy {
    /// Одна мнемоника → все сети по своим путям (unified identity).
    Unified {
        mnemonic_phrase: String,
        paths: HashMap<NetworkId, DerivationPath>,
    },
    /// Каждая сеть задаётся отдельным источником (своя мнемоника, KeyStore или HSM).
    PerChain {
        sources: HashMap<NetworkId, IdentitySource>,
    },
}

impl DerivationStrategy {
    /// Unified: одна мнемоника, пути по сетям.
    pub fn unified(mnemonic_phrase: impl Into<String>, paths: HashMap<NetworkId, DerivationPath>) -> Self {
        DerivationStrategy::Unified {
            mnemonic_phrase: mnemonic_phrase.into(),
            paths,
        }
    }

    /// Per-chain: свой источник на каждую сеть.
    pub fn per_chain(sources: HashMap<NetworkId, IdentitySource>) -> Self {
        DerivationStrategy::PerChain { sources }
    }

    /// Список сетей, для которых задана идентичность.
    pub fn networks(&self) -> Vec<NetworkId> {
        match self {
            DerivationStrategy::Unified { paths, .. } => paths.keys().cloned().collect(),
            DerivationStrategy::PerChain { sources } => sources.keys().cloned().collect(),
        }
    }
}
