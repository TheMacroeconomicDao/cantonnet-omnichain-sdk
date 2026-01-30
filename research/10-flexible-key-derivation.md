# Flexible Key Derivation — дизайн для институционального SDK

> **Цель**: одна мнемоника → несколько сетей **или** разные мнемоники/ключи на сеть; единый гибкий API для продвинутой разработки институциональных приложений.

---

## 1. Требования

- **Unified identity**: одна seed-фраза → Canton (Ed25519) + EVM (secp256k1) + позже Cosmos/Substrate по разным BIP-44 путям.
- **Per-chain identity**: каждая сеть может иметь свой источник ключа — отдельная мнемоника, импортированный ключ или HSM/KMS (compliance, изоляция).
- **Конфигурируемые пути**: пути деривации не захардкожены — институциональный клиент может задать свои (например отдельный path на Canton для production).
- **Расширяемость**: добавление новой сети = новый `NetworkId` + опционально путь/источник без ломания API.

---

## 2. Ключевые типы

### 2.1 NetworkId

Идентификатор сети в OmniChain SDK (не путать с `ChainId` EVM).

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NetworkId {
    Canton,
    Ethereum,
    Cosmos,   // позже
    Substrate, // позже
    Custom(&'static str),
}
```

Использование: `identity_for_network(NetworkId::Canton)`, `sign_for_network(NetworkId::Ethereum, payload)`.

### 2.2 DerivationPath

BIP-32/44 путь. Строковое представление для конфигов (например `m/44'/60'/0'/0/0`), парсинг и валидация в SDK.

```rust
pub struct DerivationPath {
    path: String,  // "m/44'/60'/0'/0/0"
    // опционально: разобранные компоненты для совместимости с slip10/ed25519-bip32
}
```

Стандартные пути (по умолчанию, переопределяемые):

| NetworkId | Алгоритм   | Путь по умолчанию        | Примечание                    |
|-----------|------------|---------------------------|-------------------------------|
| Canton    | Ed25519    | custom (Canton doc)       | первые 32 байта seed или path |
| Ethereum  | secp256k1  | m/44'/60'/0'/0/0          | BIP-44 ETH                    |
| Cosmos    | secp256k1  | m/44'/118'/0'/0/0         | BIP-44 Cosmos                 |

### 2.3 IdentitySource

Откуда берётся ключ для данной сети.

```rust
pub enum IdentitySource {
    /// Одна мнемоника, путь деривации для этой сети
    Mnemonic { mnemonic_phrase: SecuredString, derivation_path: DerivationPath },
    /// Уже импортированный ключ в KeyStore по fingerprint
    KeyStore { fingerprint: KeyFingerprint },
    /// Внешний HSM/KMS (идентификатор ключа в провайдере)
    Hsm { key_id: String },
}
```

Для **unified** сценария: одна `Mnemonic { phrase, path }` на сеть, пути разные (Canton path, EVM path).  
Для **per-chain**: можно смешивать — Canton из мнемоники, EVM из KeyStore (импорт), другой chain из Hsm.

### 2.4 DerivationStrategy

Стратегия конфигурации кошелька.

```rust
pub enum DerivationStrategy {
    /// Одна мнемоника → все сети по своим путям (map NetworkId → DerivationPath)
    Unified {
        mnemonic_phrase: SecuredString,
        paths: HashMap<NetworkId, DerivationPath>,
    },
    /// Каждая сеть задаётся отдельным источником (своя мнемоника, ключ или HSM)
    PerChain {
        sources: HashMap<NetworkId, IdentitySource>,
    },
}
```

Институциональный кейс: `PerChain` с частью сетей из HSM, частью из мнемоники (dev/staging).

---

## 3. Wallet API (расширение)

- **Существующее** (обратная совместимость): `party_id_for_canton()`, `sign_for_canton(payload)` — делегируют в `identity_for_network(NetworkId::Canton)` и `sign_for_network(NetworkId::Canton, payload)`.

- **Новое**:
  - `identity_for_network(&self, network: NetworkId) -> Option<NetworkIdentity>`  
    `NetworkIdentity` — enum или trait object: Canton → `PartyId` + возможность подписи; EVM → `Address` + подпись; и т.д.
  - `sign_for_network(&self, network: NetworkId, payload: &[u8]) -> SdkResult<Signature>`
  - `available_networks(&self) -> Vec<NetworkId>` — для каких сетей настроена идентичность.

- **Builder**:
  - `WalletBuilder::unified(mnemonic, paths)` — одна мнемоника, пути по сетям.
  - `WalletBuilder::per_chain(sources)` — по сети свой источник.
  - `WalletBuilder::with_canton_from_mnemonic(phrase, path)` + `with_evm_from_mnemonic(phrase, path)` + … — пошаговое добавление (под капотом собирается PerChain или Unified).

---

## 4. Реализация (слои)

1. **canton-wallet**: типы `NetworkId`, `DerivationPath`, `IdentitySource`, `DerivationStrategy`, `WalletBuilder`; расширение `Wallet` trait. Конкретная деривация (BIP-39 → seed, BIP-32/44 → key) — в модуле `derivation` с опциональной зависимостью `bip39`/`ed25519-bip32`/`k256` (feature `derivation`).
2. **canton-crypto**: уже есть `KeyStore::import_key`; при необходимости `import_from_seed(algorithm, seed, path)` для унификации деривации в одном месте (опционально).
3. **Конфиг**: YAML/JSON — `wallet: strategy: unified | per_chain`, под ним `mnemonic` + `paths` или `sources` по сетям; пароль/passphrase для мнемоники — не хранить в конфиге, только через env/CLI/интерактив.

---

## 5. Связь с research 09

- Формат Canton Party ID и подпись (Ed25519, fingerprint) без изменений.
- EVM: secp256k1 из того же или другого источника — по `IdentitySource` для `NetworkId::Ethereum`.
- Единый кошелёк (research 09) = `DerivationStrategy::Unified` с одной мнемоникой и путями для Canton и EVM.
