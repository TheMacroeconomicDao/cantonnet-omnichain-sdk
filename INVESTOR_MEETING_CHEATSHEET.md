# Canton OmniChain SDK — памятка к митингу с инвесторами (технари Canton)

Минималистичная шпаргалка: что за продукт, что уже есть, какие вопросы могут задать, как отвечать.

---

## 1. Что за продукт (30 сек)

**Canton OmniChain SDK** — production-ready Rust-библиотека для интеграции с Canton Network и кросс-чейн сценариев (Ethereum, позже Cosmos/Substrate).

- **Canton**: полный Ledger API client (gRPC, Daml-типы, команды, стримы).
- **OmniChain**: единый API поверх Canton + EVM (Alloy); Wallet = Canton external party + EVM identity из одной мнемоники (BIP-39/44).
- **Enterprise**: type-safe Rust, async (Tokio), observability, reliability (circuit breaker, rate limit, retry), secure key store (в т.ч. HSM).

---

## 2. Текущее состояние (по факту из репо, обновлено Jan 2025)

| Компонент | Статус |
|-----------|--------|
| **canton-core** | ✅ Собирается. Типы: DamlValue, Identifier, PartyId, ContractId, Commands, Events, Transaction, Filter, Offset; SdkError (ручной Display/Error), config. Порядок модулей в types/ исправлен (identifier → value → event → …). |
| **canton-crypto** | ✅ Собирается. KeyStore (memory), KeyFingerprint, Signature, thiserror, tokio, base64. |
| **canton-wallet** | ✅ Собирается. PartyId (partyHint::fingerprint), Wallet trait + **гибкая деривация** (research/10): NetworkId, DerivationPath, IdentitySource, DerivationStrategy; WalletBuilder (unified/per_chain), MultiIdentityWallet. PerChain + KeyStore работает; Unified (одна мнемоника) — API готов, реализация bip39 — дальше. |
| **canton-ledger-api** | 🔶 build.rs есть, proto/README.md с источником и версией (v2); **proto файлы не в репо** — положить по инструкции в proto/README.md. tonic 0.13: tls-ring, tls-webpki-roots. |
| **canton-transport, reliability, observability** | 🔶 Заглушки; transport — tonic фичи обновлены. |
| **canton-omnichain, canton-sdk, canton-testing** | ⬜ Крейтов нет; **убраны из workspace members** (при создании — вернуть). |

Итого: core + crypto + wallet собираются (`cargo check -p canton-wallet`); сильная база по доменной модели, ошибкам и гибкому кошельку; до первого работающего Ledger API — добавить proto и реализовать сервисы.

---

## 3. Вопросы, которые скорее всего зададут (и короткие ответы)

### Архитектура и Canton

- **Чем отличаетесь от официального/других SDK?**  
  Нативный Rust, один стек для Canton + EVM (Alloy), одна мнемоника для Canton party и EVM-адреса, enterprise-паттерны из коробки (observability, resilience).

- **Какой Ledger API используете?**  
  Целевой — **Ledger API v2** (`com.daml.ledger.api.v2`); источник и версия зафиксированы в `canton-ledger-api/proto/README.md` (Daml SDK / digital-asset/daml или Canton).

- **Как обеспечиваете совместимость с новыми версиями Canton?**  
  Версионирование proto (submodule или зафиксированный тег), feature flags под версии API, тесты против конкретных Canton/Daml версий.

- **Participant vs Domain?**  
  SDK — клиент **Ledger API** участника (Participant Node). С Domain (Sequencer, Mediator) напрямую не разговариваем; всё через Participant.

### Wallet и идентичность

- **External party / Wallet — как у вас?**  
  Модель как в research/09: Party ID = `partyHint::fingerprint` (Ed25519); KeyToParty, PartyToParticipant, подпись multiHash топологии; allocate via Admin/Topology или User Ledger API. В SDK — Wallet trait, party_id, sign_for_canton; EVM — отдельная ветка из той же мнемоники (BIP-44).

- **JWT / mTLS?**  
  Поддержка в планах в transport layer: JWT (actAs, applicationId) и mTLS для Ledger API.

### OmniChain и EVM

- **Почему Alloy, а не ethers-rs?**  
  ethers-rs deprecated в пользу Alloy; Alloy — актуальный стек для Rust (провайдеры, подпись, ABI). Research/09 это зафиксировал.

- **Какие сценарии OmniChain в приоритете?**  
  Сначала Canton ↔ EVM (bridge/adapter): доказательства с одной цепи, верификация на другой; единый кошелёк для подписи на обеих. Cosmos/Substrate — следующий этап.

### Безопасность и production

- **Крипто?**  
  Ed25519 (Canton), secp256k1 + Keccak (EVM), хранение ключей — in-memory / file / HSM (trait-based). Никакого unwrap/panic в библиотечном коде (clippy).

- **Аудит, лицензии?**  
  cargo-deny: лицензии, дубликаты, advisories. Аудит кода — по roadmap после стабилизации API.

### Roadmap и команда

- **Ближайшие шаги?**  
  1) Фиксация источника и версии proto; 2) реализация Ledger API client (submit, stream, active contracts, party); 3) сквозной сценарий: Wallet → submit command; 4) EVM adapter (Alloy); 5) reliability/observability в production-конфиге.

- **Срок до первого “working end-to-end”?**  
  Оценить честно по своей команде; для технарей лучше назвать порядок: “месяц на Ledger API + Wallet flow”, “ещё N на EVM bridge”.

---

## 4. Что не врать и что подчеркнуть

- **Не врать:** proto ещё не подключены; canton-ledger-api пока не звонит в Canton; canton-omnichain не реализован.  
- **Подчеркнуть:** продуманная доменная модель (core), выравнивание с официальной моделью Canton (research 01, 09), чёткий план (DEVELOPMENT_PROMPT, PRE_DEVELOPMENT_CHECKLIST), выбор Alloy и Wallet/EVM дизайн по актуальной доке.

---

## 5. Полезные ссылки (если попросят углубиться)

- В репо: `DEVELOPMENT_PROMPT.md`, `research/08-sdk-architecture-design.md`, `research/09-canton-wallet-evm-integration.md`, `research/01-canton-network-architecture.md`, `research/04-daml-ledger-api.md`.
- Canton: docs.canton.network, репо DACH-NY/canton.
- Ledger API proto: github.com/digital-asset/daml (ledger-api/grpc-definitions).

---

*Кратко: ты делаешь нативный Rust SDK под Canton + OmniChain с сильным core и понятным планом; текущий разрыв — proto + реализация Ledger API и Wallet flow. Говоришь об этом прямо и фокус держишь на архитектуре и roadmap.*
