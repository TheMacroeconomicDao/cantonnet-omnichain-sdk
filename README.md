# Canton OmniChain SDK

Rust SDK для интеграции с Canton Network и OmniChain (Canton ↔ EVM/Cosmos/Substrate).

## Требования

- Rust 1.75+ (stable)
- См. `rust-toolchain.toml` и `Cargo.toml` (workspace.package.rust-version)

## Сборка

```bash
cargo build --workspace
```

## Проверка

```bash
cargo check --workspace
```

## Тесты

```bash
cargo test --workspace
```

## Линтеры

```bash
cargo clippy --workspace -- -D warnings
cargo fmt -- --check
cargo deny check   # при установленном cargo-deny
```

## Структура крейтов

| Крейт | Назначение |
|-------|------------|
| `canton-core` | Типы, ошибки, трейты (identifier, value, event, command, transaction, filter, offset) |
| `canton-ledger-api` | gRPC‑клиент Ledger API v2 (источник/версия proto: `crates/canton-ledger-api/proto/README.md`) |
| `canton-crypto` | KeyStore, подписи (Ed25519, P-256, secp256k1) |
| `canton-wallet` | Wallet, гибкая деривация (Unified/PerChain), PartyId, MultiIdentityWallet |
| `canton-transport` | gRPC transport (tonic) |
| `canton-reliability` | Заглушка |
| `canton-observability` | Заглушка |

## Конфигурация

Пример конфига: `config/example.yaml` (разработка, DevNet). Для production: `config/example-production.yaml` → скопировать в `config/local.yaml` и подставить свои значения. Локальные/секреты — в `config/local.yaml` (в .gitignore).

Секция `ledger_api`: по умолчанию в примере — **participant DevNet** (gRPC `65.108.15.30:30501`, HTTP `http://65.108.15.30:30757`). Production — переопределить в `config/local.yaml` (хост, порт, `tls: true`, пути к сертификатам). Подробно: **docs/DEVNET_PARTICIPANT.md**.

## Участник DevNet (разработка)

Для локальной разработки и интеграционных тестов используется participant Canton DevNet (validator, sync.global). Эндпоинты:

| Назначение            | Адрес |
|-----------------------|--------|
| gRPC Ledger API       | `65.108.15.30:30501` |
| HTTP Ledger API (JSON)| `http://65.108.15.30:30757` |

Конфиг: `config/example.yaml` → `ledger_api`. Полное описание, env-переменные и ограничения: **docs/DEVNET_PARTICIPANT.md**.

## Документация

- **docs/README.md** — индекс документации (флоу: с чего начать, где что искать).
- **docs/DEVNET_PARTICIPANT.md** — участник DevNet для разработки: эндпоинты, конфиг, env, ограничения.
- **DEVELOPMENT_PROMPT.md** — спецификация, фазы, критерии приёмки, блок **ПРОГРЕСС** в начале.
- **PRE_DEVELOPMENT_CHECKLIST.md** — чеклист перед разработкой и порядок действий.
- **research/** — архитектура Canton, OmniChain, Ledger API, gRPC/Rust, крипто, Wallet + EVM (research/09), гибкая деривация (research/10).

## Лицензия

Apache-2.0 OR MIT
