# Pre-Development Checklist

Чеклист перед началом разработки и порядок действий. Связано: **DEVELOPMENT_PROMPT.md**, **docs/DEVNET_PARTICIPANT.md**.

---

## 1. Окружение

- [ ] **Rust** — `rust-toolchain.toml` (stable). Проверка: `rustc --version` (ожидается 1.75+).
- [ ] **Компоненты** — при необходимости: `rustup component add rustfmt clippy`.
- [ ] **cargo-deny** — опционально для `cargo deny check` (см. README).

---

## 2. Репозиторий и сборка

- [ ] Клонировать / обновить репозиторий.
- [ ] `cargo build --workspace` — успешная сборка всех крейтов.
- [ ] `cargo check --workspace` — без ошибок.
- [ ] `cargo test --workspace` — юнит-тесты проходят.
- [ ] `cargo clippy --workspace -- -D warnings` — без предупреждений.
- [ ] `cargo fmt -- --check` — форматирование в порядке (или `cargo fmt` для автоисправления).

---

## 3. Конфигурация

- [ ] **Пример конфига** — `config/example.yaml` (DevNet по умолчанию).
- [ ] **Локальный конфиг** — при необходимости: скопировать в `config/local.yaml` и подставить свои значения; `config/local.yaml` в .gitignore.
- [ ] **Production** — брать за основу `config/example-production.yaml` → `config/local.yaml` (TLS, таймауты, сертификаты). DevNet — без TLS, только для разработки/тестов.

Подробно по эндпоинтам и env: **docs/DEVNET_PARTICIPANT.md**.

---

## 4. Интеграционные тесты (опционально)

- [ ] При тестах против реального participant: задать env (см. docs/DEVNET_PARTICIPANT.md):
  - `CANTON_LEDGER_GRPC` — например `65.108.15.30:30501`
  - `CANTON_LEDGER_HTTP` — например `http://65.108.15.30:30757`
- [ ] Без этих переменных тесты, зависящие от ноды, могут пропускаться (по логике проекта).

---

## 5. Документация перед задачей

- [ ] **DEVELOPMENT_PROMPT.md** — фазы, критерии приёмки, блок ПРОГРЕСС (если заполнен).
- [ ] **docs/DEVNET_PARTICIPANT.md** — эндпоинты DevNet, ограничения (не production, без TLS).
- [ ] **research/** — при касании архитектуры: `04-daml-ledger-api.md`, `08-sdk-architecture-design.md`; при Wallet/EVM — `09-canton-wallet-evm-integration.md`, `10-flexible-key-derivation.md`.

---

## 6. Порядок действий (рекомендуемый)

1. Выполнить пункты 1–2 (окружение, сборка, тесты, линтеры).
2. При работе с Ledger API или интеграционными тестами — пункты 3–4 (конфиг, env).
3. Перед новой фичей/багом — пункт 5 (актуальная документация и research).
4. После изменений — снова `cargo test --workspace` и `cargo clippy --workspace -- -D warnings`.
