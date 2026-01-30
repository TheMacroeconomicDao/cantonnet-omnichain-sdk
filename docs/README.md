# Документация Canton OmniChain SDK

Краткий индекс: где что лежит и в каком порядке использовать.

## Перед разработкой

1. **PRE_DEVELOPMENT_CHECKLIST.md** (корень) — чеклист: окружение, proto, workspace, CI, конфиг. Порядок действий перед Phase 1.
2. **DEVELOPMENT_PROMPT.md** (корень) — полная спецификация, фазы, критерии приёмки. Блок ПРОГРЕСС в начале.

## Окружение и участник для разработки

3. **DEVNET_PARTICIPANT.md** (эта папка) — **единый источник по DevNet participant**: эндпоинты (gRPC 65.108.15.30:30501, HTTP 30757), конфиг `ledger_api` в `config/example.yaml`, переменные окружения для интеграционных тестов, ограничения (без TLS, только dev). Использовать для локальной разработки и опциональных E2E-тестов.

## Конфигурация

- **config/example.yaml** — пример для разработки (DevNet). Секция `ledger_api` по умолчанию — DevNet (см. DEVNET_PARTICIPANT.md).
- **config/example-production.yaml** — шаблон для production: `ledger_api` с `grpc_host`, `grpc_port`, `tls: true`, `tls_certs` (ca, client cert/key). Копировать в `config/local.yaml` и подставить свои значения. Секреты и локальные переопределения — только в `config/local.yaml` (в .gitignore).

## Архитектура и исследование

- **research/** — архитектура Canton, OmniChain, Ledger API, gRPC/Protobuf в Rust, крипто, production-паттерны, дизайн SDK (08), Canton Wallet + EVM (09), гибкая деривация (10).

## Прочее

- **PROMPT_CLEANUP_MACOS.md** — промпт для очистки места на macOS (вне основного флоу разработки SDK).

---

Флоу: **PRE_DEVELOPMENT_CHECKLIST** → **DEVELOPMENT_PROMPT** (фазы) → при разработке клиента Ledger API и тестов — **DEVNET_PARTICIPANT** + **config/example.yaml**.
