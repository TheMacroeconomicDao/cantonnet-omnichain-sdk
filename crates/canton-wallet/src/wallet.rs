//! Wallet trait and implementations: Canton external party + OmniChain identities.
//! See research/09, 10.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use canton_core::{PartyId, SdkError, SdkResult};
use canton_crypto::keystore::{KeyFingerprint, KeyStore};
use canton_crypto::Signature;

use crate::derivation::{DerivationPath, DerivationStrategy, IdentitySource, NetworkId};
use crate::party_id::canton_party_id_from_fingerprint;

// -----------------------------------------------------------------------------
// Wallet trait
// -----------------------------------------------------------------------------

/// Canton external party + опционально идентичности в других сетях (EVM и т.д.).
/// Гибко: одна мнемоника на все сети (Unified) или отдельный источник на сеть (PerChain).
/// See research/09-canton-wallet-evm-integration.md, research/10-flexible-key-derivation.md.
#[async_trait]
pub trait Wallet: Send + Sync {
    /// Party ID для Canton (формат partyHint::fingerprint).
    fn party_id_for_canton(&self) -> PartyId;

    /// Подпись для Canton (Ed25519).
    async fn sign_for_canton(&self, payload: &[u8]) -> SdkResult<Signature>;

    /// Идентичность для сети, если настроена. Canton → PartyId; EVM → позже Address; и т.д.
    fn identity_for_network(&self, network: NetworkId) -> Option<NetworkIdentity> {
        if network == NetworkId::Canton {
            Some(NetworkIdentity::Canton(self.party_id_for_canton()))
        } else {
            None
        }
    }

    /// Подпись для указанной сети (Canton = Ed25519, EVM = secp256k1, …).
    /// По умолчанию только Canton; остальные сети — Err.
    async fn sign_for_network(&self, network: NetworkId, payload: &[u8]) -> SdkResult<Signature> {
        if network == NetworkId::Canton {
            self.sign_for_canton(payload).await
        } else {
            Err(SdkError::Config(format!(
                "identity not configured for network: {}",
                network
            )))
        }
    }

    /// Сети, для которых настроена идентичность. По умолчанию только Canton.
    fn available_networks(&self) -> Vec<NetworkId> {
        vec![NetworkId::Canton]
    }
}

/// Идентичность в конкретной сети. Расширяется под Address (EVM), Cosmos, Substrate.
#[derive(Debug, Clone)]
pub enum NetworkIdentity {
    Canton(PartyId),
    Ethereum { address_hex: String }, // placeholder до интеграции Alloy
    Other { network: NetworkId, label: String },
}

// -----------------------------------------------------------------------------
// MultiIdentityWallet — PerChain с ключами из KeyStore
// -----------------------------------------------------------------------------

/// Кошелёк с отдельным ключом (KeyFingerprint) на сеть. Строится из DerivationStrategy::PerChain
/// при всех источниках KeyStore; мнемоника (Unified) — в будущем.
pub struct MultiIdentityWallet {
    keystore: Arc<dyn KeyStore>,
    /// Сеть → fingerprint ключа для подписи.
    keys_by_network: HashMap<NetworkId, KeyFingerprint>,
    /// Подсказка для Canton Party ID (partyHint в partyHint::fingerprint).
    canton_party_hint: String,
}

impl MultiIdentityWallet {
    pub fn new(
        keystore: Arc<dyn KeyStore>,
        keys_by_network: HashMap<NetworkId, KeyFingerprint>,
        canton_party_hint: impl Into<String>,
    ) -> Self {
        Self {
            keystore,
            keys_by_network,
            canton_party_hint: canton_party_hint.into(),
        }
    }

    fn canton_fingerprint(&self) -> Option<&KeyFingerprint> {
        self.keys_by_network.get(&NetworkId::Canton)
    }
}

#[async_trait]
impl Wallet for MultiIdentityWallet {
    fn party_id_for_canton(&self) -> PartyId {
        match self.canton_fingerprint() {
            Some(fp) => canton_party_id_from_fingerprint(&self.canton_party_hint, fp),
            None => {
                // Fallback: пустой party_id невалиден по смыслу; вызывающий не должен полагаться на это.
                PartyId::new_unchecked(format!("{}::", self.canton_party_hint))
            }
        }
    }

    async fn sign_for_canton(&self, payload: &[u8]) -> SdkResult<Signature> {
        self.sign_for_network(NetworkId::Canton, payload).await
    }

    fn identity_for_network(&self, network: NetworkId) -> Option<NetworkIdentity> {
        if !self.keys_by_network.contains_key(&network) {
            return None;
        }
        match network {
            NetworkId::Canton => Some(NetworkIdentity::Canton(self.party_id_for_canton())),
            NetworkId::Ethereum => Some(NetworkIdentity::Ethereum {
                address_hex: String::new(), // TODO: derive from pubkey when EVM adapter is in place
            }),
            _ => Some(NetworkIdentity::Other {
                network: network.clone(),
                label: format!("{}", network),
            }),
        }
    }

    async fn sign_for_network(&self, network: NetworkId, payload: &[u8]) -> SdkResult<Signature> {
        let fp = self
            .keys_by_network
            .get(&network)
            .ok_or_else(|| SdkError::Config(format!("identity not configured for network: {}", network)))?;
        self.keystore
            .sign(fp, payload)
            .await
            .map_err(|e| SdkError::Crypto(e.to_string()))
    }

    fn available_networks(&self) -> Vec<NetworkId> {
        self.keys_by_network.keys().cloned().collect()
    }
}

// -----------------------------------------------------------------------------
// WalletBuilder — гибкая сборка из одной мнемоники или разных источников
// -----------------------------------------------------------------------------

/// Сборка кошелька: Unified (одна мнемоника → пути по сетям) или PerChain (свой источник на сеть).
pub struct WalletBuilder {
    strategy: DerivationStrategy,
    canton_party_hint: Option<String>,
}

impl WalletBuilder {
    /// Одна мнемоника, пути деривации по сетям (unified identity).
    pub fn unified(mnemonic_phrase: impl Into<String>, paths: HashMap<NetworkId, DerivationPath>) -> Self {
        Self {
            strategy: DerivationStrategy::unified(mnemonic_phrase, paths),
            canton_party_hint: None,
        }
    }

    /// Отдельный источник на каждую сеть (разные мнемоники, KeyStore или HSM).
    pub fn per_chain(sources: HashMap<NetworkId, IdentitySource>) -> Self {
        Self {
            strategy: DerivationStrategy::per_chain(sources),
            canton_party_hint: None,
        }
    }

    /// Подсказка для Canton Party ID (обязательна для Canton при build с KeyStore).
    pub fn canton_party_hint(mut self, hint: impl Into<String>) -> Self {
        self.canton_party_hint = Some(hint.into());
        self
    }

    /// Собрать кошелёк, используя переданный KeyStore. Работает только для PerChain,
    /// когда все источники — KeyStore (mnemonic/hsm пока не реализованы).
    pub fn build_with_keystore(self, keystore: Arc<dyn KeyStore>) -> SdkResult<MultiIdentityWallet> {
        let canton_party_hint = self
            .canton_party_hint
            .unwrap_or_else(|| "wallet".to_string());

        match &self.strategy {
            DerivationStrategy::Unified { .. } => {
                return Err(SdkError::Config(
                    "unified mnemonic derivation not yet implemented; use per_chain with KeyStore sources".into(),
                ));
            }
            DerivationStrategy::PerChain { sources } => {
                let mut keys_by_network = HashMap::new();
                for (network, source) in sources {
                    match source {
                        IdentitySource::KeyStore { fingerprint } => {
                            keys_by_network.insert(network.clone(), fingerprint.clone());
                        }
                        IdentitySource::Mnemonic { .. } | IdentitySource::Hsm { .. } => {
                            return Err(SdkError::Config(format!(
                                "mnemonic/hsm source for network {} not yet implemented; use KeyStore source",
                                network
                            )));
                        }
                    }
                }
                Ok(MultiIdentityWallet::new(
                    keystore,
                    keys_by_network,
                    canton_party_hint,
                ))
            }
        }
    }
}
