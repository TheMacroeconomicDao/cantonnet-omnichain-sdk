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

Пример конфига: `config/example.yaml`. Локальные/секреты — в `config/local.yaml` (в .gitignore).

## Документация

- **DEVELOPMENT_PROMPT.md** — спецификация, фазы, критерии приёмки, блок **ПРОГРЕСС** в начале.
- **PRE_DEVELOPMENT_CHECKLIST.md** — чеклист перед разработкой и порядок действий.
- **research/** — архитектура Canton, OmniChain, Ledger API, gRPC/Rust, крипто, Wallet + EVM (research/09), гибкая деривация (research/10).

## Лицензия

Apache-2.0 OR MIT
