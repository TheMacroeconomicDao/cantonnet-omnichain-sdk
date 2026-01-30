# Canton Wallet and EVM Integration Research

> **Purpose**: Экспертиза и актуальные данные для интеграции Canton Wallet и EVM в Rust SDK.  
> **Sources**: Canton Network docs, Digital Asset integrate/devnet, Alloy/ethers-rs, Ledger API auth.  
> **Last Updated**: January 2025

---

## 1. Canton Wallet (External Party) — актуальная модель

### 1.1 Концепция

- **Party** в Canton — действующая сущность; все транзакции происходят между партиями.
- **External Party (Wallet)** — партия, созданная и контролируемая владельцем ключа вне участника (validator). Участник может представлять несколько партий; одна партия может быть представлена несколькими участниками (multi-hosting).
- **Identity** в Canton — децентрализованная система без единого доверенного центра: идентификатор = случайная строка + отпечаток публичного ключа (fingerprint), предполагается глобальная уникальность.

### 1.2 Формат Party ID

- **Синтаксис**: `{partyHint}::{fingerprint}`  
  Пример: `my-wallet-1::1220e7b23ea52eb5c672fb0b1cdbc916922ffed3dd7676c223a605664315e2d43edd`
- **partyHint** — человекопонятное имя (alice, bob, my-wallet-1); должно быть уникальным для данного fingerprint.
- **fingerprint** — отпечаток публичного ключа (Ed25519 по умолчанию); генерируется из public key (см. документацию Canton security).

### 1.3 Жизненный цикл External Party (по официальной документации)

1. **Создать ключевую пару**  
   По умолчанию **Ed25519**. Поддерживается генерация из BIP-39 мнемоники (см. раздел 3).

2. **Сгенерировать fingerprint** из публичного ключа (встроенная функция Wallet SDK: `createFingerprintFromPublicKey`).

3. **Сгенерировать топологические транзакции** (три типа):
   - **PartyToParticipant** — партия соглашается быть размещённой у участника.
   - **ParticipantToParty** — участник соглашается размещать партию.
   - **KeyToParty** — ключ (публичный) привязывается к партии.

4. **Объединить хеши** этих транзакций в один multiHash и **подписать multiHash** приватным ключом.

5. **Отправить топологию** валидатору: `allocateExternalParty(signedHash, preparedParty)`.

6. **Подписание команд** (prepare + sign + execute):
   - `prepareSubmission(command)` → preparedTransactionHash;
   - `signTransactionHash(preparedTransactionHash, privateKey)`;
   - `executeSubmissionAndWait(prepareResponse, signedCommandHash, publicKey, submissionId)`.

### 1.4 Сопоставление с нашим SDK (canton-crypto, canton-ledger-api)

| Canton Wallet SDK (TypeScript) | Наш Rust SDK |
|-------------------------------|--------------|
| `createKeyPair()` (Ed25519) | `KeyStore::generate_key(KeyAlgorithm::Ed25519, KeyPurpose::Signing, …)` |
| `createFingerprintFromPublicKey(publicKey)` | `KeyFingerprint::compute(public_key, KeyAlgorithm::Ed25519)` + формат party ID |
| Party ID = `partyHint::fingerprint` | `PartyId::new(format!("{}::{}", hint, fingerprint_hex))` — валидация формата в canton-core |
| `signTransactionHash(hash, privateKey)` | `KeyStore::sign(fingerprint, hash)` |
| Topology tx (PartyToParticipant, etc.) | Отдельный слой: либо вызов Admin/Topology API, либо интеграция с User Ledger API `generateExternalParty`/`allocateExternalParty` (если доступны в gRPC) |

**Важно**: Официальный Wallet SDK (TypeScript) использует **User Ledger API** и **Topology** (Scan Proxy API). В Rust SDK нужно либо реализовать эквивалент топологии (парсинг/генерация топологических транзакций + подпись), либо документировать использование существующего участника с уже выделенной партией (allocate_party от имени участника) для сценариев без external wallet.

---

## 2. Аутентификация Ledger API (JWT / mTLS)

### 2.1 JWT

- Стандартный механизм: JWT в заголовке `Authorization: Bearer <token>`.
- Identity Provider выдаёт токен с claims: **actAs** (список партий, от имени которых разрешено действовать), **applicationId**, срок действия.
- Ledger проверяет подпись по JWKS (URL издателя), целостность и срок, а также права на запрос.

### 2.2 mTLS

- Клиент предъявляет сертификат, подписанный доверенным CA. Защита транспорта и ограничение доступа по сертификату; идентификация приложения (CN) не обязательно совпадает с `application_id` в запросе.

### 2.3 Связь с Wallet

- Для **external party** команды отправляются с `act_as = [party_id]`; право действовать от имени этой партии должно быть выдано через топологию (KeyToParty, PartyToParticipant) и/или через JWT claims. В Rust SDK: при использовании Wallet нужно передавать в командах `act_as` из идентификатора кошелька (party_id) и обеспечивать наличие JWT/mTLS с правами на эту партию.

---

## 3. Криптография и ключи (дополнение к 06)

### 3.1 Canton

- **Ed25519** — по умолчанию для внешних партий (wallet). Поддержка ECDSA P-256 в узле (см. Canton security).
- **BIP-39**: мнемоника → seed → первые 32 байта как приватный ключ Ed25519 (в документации Canton и TypeScript примерах). В Rust: `bip39` + `ed25519-dalek` (или наш KeyStore с import_key из seed).

### 3.2 EVM (Ethereum)

- **secp256k1** (ECDSA), Keccak-256 для адресов. В нашем SDK уже: `k256`, `sha3` (06-cryptographic-requirements). Для единого кошелька Canton + EVM: одна мнемоника BIP-39/44, разные производные (Canton: путь для Ed25519, EVM: путь 44/60/0/0/0 или аналог).

---

## 4. EVM в Rust: Alloy вместо ethers-rs

### 4.1 Актуальный стек (2024–2025)

- **ethers-rs** — объявлен deprecated в пользу **Alloy** и Foundry.
- **Alloy** — преемник ethers-rs: транспорты, провайдеры, подпись, ABI, RLP, типы цепей. Версия ~1.4.x.
- Ключевые крейты: `alloy`, `alloy-provider`, `alloy-signer`, `alloy-network`, `alloy-contract`, `alloy-primitives`.

### 4.2 Подпись и провайдер (Alloy)

- **Signer** trait: подпись хешей, сообщений, typed data; реализации — `PrivateKeySigner`, аппаратные (Ledger, Trezor), KMS (AWS, GCP, Turnkey).
- **ProviderBuilder**: `ProviderBuilder::new().wallet(signer).connect_http(rpc_url)`.
- Отправка транзакции: построить `TransactionRequest`, вызвать `provider.send_transaction(tx).await`.

### 4.3 Рекомендация для canton-omnichain

- Использовать **Alloy** для EthereumAdapter (вместо условного "ethers" в doc). Feature: `ethereum = ["alloy-..."]`.
- **EthereumAdapter** в Rust: `alloy-provider` (HTTP/WS), `alloy-signer` для подписи; при необходимости единый кошелёк — наш `Wallet` (canton-wallet) держит KeyStore и отдаёт для EVM secp256k1 ключ или делегирует подпись через trait, реализованный поверх Alloy Signer.

---

## 5. Canton ↔ EVM: мосты и сценарии

### 5.1 Из исследований

- **Temple Bridge**: активы между Ethereum, Base, BNB Chain и Canton (USDC, SBC и др.); подключение EVM-кошельков (MetaMask, Phantom).
- **Gravity Bridge**: WETH, USDT, wstETH и др.
- Canton позиционируется как «сеть сетей»; фокус на RWA, соответствие регуляториям, KYC, DvP.

### 5.2 Для SDK

- **Canton as Hub**: Canton — центр, EVM — один из периферийных адаптеров (как в 02-omnichain-integration-patterns).
- **EthereumAdapter** отвечает за: RPC, подпись tx (Alloy), отправку, чтение событий; **Bridge** в SDK — оркестрация lock/release и вызов Canton команд + вызов EVM контрактов, без дублирования логики конкретного публичного моста (Temple/Gravity).

---

## 6. Архитектура Wallet в Rust SDK (сводка)

### 6.1 Слой Wallet (новый крейт или модуль в canton-crypto)

- **Wallet** trait (или единый тип):
  - `party_id_for_canton(&self) -> PartyId` — для Ledger API (формат `partyHint::fingerprint`).
  - `address_for_evm(&self) -> Option<Address>` — опционально, если тот же кошелёк используется для EVM (производная secp256k1).
  - `sign_for_canton(&self, payload: &[u8]) -> Result<Signature>` — подпись для Canton (Ed25519).
  - `sign_for_evm(&self, payload: &[u8]) -> Result<Signature>` — подпись для EVM (secp256k1), если есть производная.
- Реализация: **Wallet** держит **KeyStore** + опционально деривацию (BIP-39/44): Canton key + EVM key из одной мнемоники; party_id строится из fingerprint + hint.

### 6.2 Интеграция в CantonSdkBuilder

- `.wallet(Arc<dyn Wallet>)` или `.keystore(…)` + `.party_hint(…)` для построения default party_id.
- LedgerClient при отправке команд подставляет `act_as` из wallet.party_id_for_canton(); при необходимости prepare/sign/execute по модели external party — отдельный flow с подписью hash команды (см. п. 1.3).

### 6.3 EVM adapter (canton-omnichain)

- **EthereumAdapter**: RPC через Alloy (`alloy-provider`), подпись через `alloy-signer` или через наш Wallet (`sign_for_evm` → подписанная raw tx).
- Типы: `ChainId::Ethereum`, адреса `Address` (Alloy primitives), tx и receipt — обёртки над Alloy типами.

---

## 7. Источники и ссылки

- Canton: Create an External Party (Wallet) — https://docs.digitalasset.com/integrate/devnet/party-management/index.html  
- Canton Identity Management — https://docs.daml.com/canton/usermanual/identity_management.html  
- Canton Cryptographic keys — https://docs.digitalasset.com/overview/3.4/explanations/canton/security.html  
- Daml Ledger API auth (JWT, actAs) — https://docs.daml.com/app-dev/authorization.html  
- Alloy (Rust Ethereum) — https://alloy.rs , https://docs.rs/alloy/latest/alloy/  
- Alloy Signers vs Ethereum Wallet — https://alloy.rs/guides/signers-vs-ethereum-wallet  
- Ethers-rs deprecated in favor of Alloy — https://docs.rs/ethers  
- Canton & Ethereum (post-trade, bridge) — Temple Bridge, Gravity Bridge; Finadium/Canton whitepapers.

---

*Документ дополняет research 02, 06, 08 и DEVELOPMENT_PROMPT; ценная информация из существующих документов не заменяется, а уточняется и расширяется.*
