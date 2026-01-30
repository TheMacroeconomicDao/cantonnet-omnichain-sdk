# Чеклист перед началом разработки

Что уже есть и что нужно добавить перед Phase 1 (Foundation).

**Последнее обновление прогресса:** Jan 2025 — см. также блок «ПРОГРЕСС» в DEVELOPMENT_PROMPT.md.

---

## ✅ Уже готово

- **DEVELOPMENT_PROMPT.md** — полная спецификация, фазы, критерии приёмки; в начале добавлен блок **ПРОГРЕСС / СОСТОЯНИЕ ДЛЯ ПРОДОЛЖЕНИЯ**.
- **research/01–10** — архитектура Canton, OmniChain, Ledger API, gRPC/Rust, крипто, production patterns, дизайн SDK, Canton Wallet + EVM (Alloy), **10 — гибкая деривация ключей** (Unified/PerChain).
- Согласованный стек: Rust, Tonic/Prost, Canton Ledger API, Alloy для EVM, Wallet = external party + EVM identity; гибко: одна мнемоника или разные источники на сеть (research/10).
- **canton-core**: собирается; типы, ошибки, порядок модулей и реэкспорты исправлены (identifier → value → event → …).
- **canton-crypto**: собирается; KeyStore, InMemoryKeyStore, thiserror, tokio, base64; правки для tonic/ecdsa.
- **canton-wallet**: собирается; гибкая деривация (NetworkId, DerivationPath, IdentitySource, DerivationStrategy), WalletBuilder (unified/per_chain), MultiIdentityWallet (PerChain + KeyStore). Unified из мнемоники — API есть, реализация bip39 — дальше.
- **Workspace**: Cargo.toml с members без canton-sdk, canton-omnichain, canton-testing (крейтов нет; при создании — вернуть в members). canton-ledger-api, canton-transport — tonic 0.13 фичи tls-ring, tls-webpki-roots.

---

## 🔲 Что добавить перед разработкой / продолжить

### 1. Окружение и версии

- [x] **Workspace Cargo.toml** — есть; edition 2021, rust-version 1.75.
- [ ] **Rust**: при необходимости обновить до MSRV из spec (1.85 / edition 2024).
- [ ] **rust-toolchain.toml** в корне (опционально).
- [ ] Решить: edition 2024 или оставить 2021.

### 2. Proto-файлы Ledger API

- [x] **Источник и версия**: зафиксированы в `crates/canton-ledger-api/proto/README.md` — v2 (`com.daml.ledger.api.v2`), источник: Daml SDK / digital-asset/daml `ledger-api/grpc-definitions` или Canton.
- [ ] **Скопировать или подмодуль**: положить нужные `.proto` в `crates/canton-ledger-api/proto/com/daml/ledger/api/v2/` (и при необходимости v1) по инструкции в proto/README.md.
- [ ] Список proto (см. proto/README.md): command_service, command_submission_service, command_completion_service; в v2 — update_service, state_service; party_management_service, package_service, version_service, ledger_identity_service; commands, completion, transaction, transaction_filter, participant_offset + зависимости (value, event и т.д.).

### 3. Скелет workspace (до первой реализации)

- [x] **Корень**: Cargo.toml (workspace), deny.toml, clippy.toml, rustfmt.toml (если есть).
- [x] **Крейты**: canton-core, canton-ledger-api, canton-crypto, canton-wallet, canton-transport, canton-reliability, canton-observability — есть (заглушки или реализация).
- [ ] **Крейты-заглушки** (опционально): canton-sdk, canton-omnichain, canton-testing — при создании добавить в workspace members.
- [x] **canton-ledger-api**: директория `proto/` есть (proto/README.md с инструкцией); `build.rs` готов — при отсутствии proto сборка не падает, при наличии proto — компиляция и `proto_compiled`.

### 4. Конфигурация качества и безопасности

- [x] **cargo-deny**: `deny.toml` — запрет дубликатов, проверка лицензий, уязвимостей (advisories).
- [x] **clippy**: `clippy.toml` или атрибуты в корневом Cargo — согласовать с research/03 (no unwrap, no panic в библиотеке).
- [x] **rustfmt**: `rustfmt.toml` — единый стиль.

### 5. CI/CD (минимальный скелет для Phase 1)

- [x] **GitHub Actions** (или аналог): job `check` — `cargo check --workspace`; job `test` — `cargo test --workspace`; job `clippy` — `cargo clippy -- -D warnings`; опционально `cargo deny check`.
- [x] Триггеры: push в main/master, PR.

### 6. Документация и конфиг-примеры

- [x] **README.md** в корне: описание проекта, как собрать, как запустить тесты, ссылки на DEVELOPMENT_PROMPT и research/09 (Wallet + EVM).
- [x] **Пример конфига**: `config/example.yaml` (или `docs/example.yaml`) — скопировать блок из DEVELOPMENT_PROMPT (canton, reliability, observability, omnichain), чтобы не вводить значения вручную.

### 7. Опционально, но полезно

- [ ] **CONTRIBUTING.md** — порядок коммитов, запуск тестов/линтеров, ссылка на PRE_DEVELOPMENT_CHECKLIST и Phase 1.
- [x] **.gitignore**: `target/`, `**/*.rs.bk`, конфиги с секретами (например `config/local.yaml`), IDE.
- [ ] **dependabot** (или Renovate): обновление зависимостей Rust (Cargo.toml).

---

## Порядок действий (рекомендуемый для продолжения)

1. **Ledger API**: источник и версия зафиксированы в `canton-ledger-api/proto/README.md` (v2). Положить proto по инструкции → реализовать сервисы (command, update/state, party, completion).
2. Опционально: создать крейты canton-sdk, canton-omnichain, canton-testing и вернуть их в workspace members.
3. ~~Добавить deny.toml, clippy.toml, CI (check, test, clippy), README, example config~~ — сделано.
4. Деривация из мнемоники (bip39) в canton-wallet для Unified и build_with_keystore(Unified).
5. Phase 1 дальше: полная реализация canton-ledger-api client, интеграция Wallet → submit command.

---

## Ссылки

- DEVELOPMENT_PROMPT.md — фазы, критерии, **блок ПРОГРЕСС в начале**
- research/08-sdk-architecture-design.md — структура крейтов
- research/05-grpc-protobuf-rust.md — proto layout, build.rs
- research/09-canton-wallet-evm-integration.md — Wallet + EVM
- research/10-flexible-key-derivation.md — гибкая деривация (Unified/PerChain)
