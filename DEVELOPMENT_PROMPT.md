# Canton OmniChain SDK — Development Prompt

Полная спецификация, фазы и критерии приёмки. Перед разработкой: **PRE_DEVELOPMENT_CHECKLIST.md**. Участник DevNet: **docs/DEVNET_PARTICIPANT.md**.

---

## ПРОГРЕСС

| Фаза | Статус | Заметки |
|------|--------|--------|
| Phase 1: Ledger API client | В работе | LedgerClient: connect, get_ledger_end, submit(proto Commands). CommandSubmissionService подключён. |
| Phase 2: Config + Version/Health | Не начата | LedgerApiConfig (ledger_api из example.yaml), VersionService, опционально Health. |
| Phase 3: Transport + Reliability | Не начата | canton-transport: ChannelBuilder, TLS; canton-reliability: retry, circuit_breaker. |
| Phase 4: Conversion + high-level API | Не начата | canton_core::Commands → proto Commands; CantonClient facade. |

Обновлять этот блок по мере выполнения.

---

## Phase 1: Ledger API client (текущая)

**Цель:** клиент Ledger API v2: подключение к participant, StateService (get_ledger_end), CommandSubmissionService (submit).

**Критерии приёмки:**
- [x] LedgerClient::connect(endpoint, ledger_id) — gRPC channel, возврат клиента.
- [x] get_ledger_end() — возврат LedgerOffset.
- [x] submit(commands) — приём proto Commands, вызов CommandSubmissionService.Submit.
- [ ] (опционально) Загрузка endpoint из конфига (LedgerApiConfig, см. config/example.yaml ledger_api).

**Референс:** research/04-daml-ledger-api.md, research/08-sdk-architecture-design.md, crates/canton-ledger-api.

---

## Phase 2: Config и вспомогательные сервисы

**Цель:** конфиг `ledger_api` (grpc_host, grpc_port, http_url, tls) в canton-core или отдельном типе; VersionService (узнать ledger_id); опционально Health.

**Критерии приёмки:**
- LedgerApiConfig в конфиге (совместимость с config/example.yaml).
- LedgerClient::connect из LedgerApiConfig (endpoint = grpc_host:grpc_port, схема http(s)).
- VersionService или аналог для получения ledger_id при необходимости.
- Документация: использование config/example.yaml и config/local.yaml.

**Референс:** docs/DEVNET_PARTICIPANT.md, config/example.yaml, config/example-production.yaml.

---

## Phase 3: Transport и Reliability

**Цель:** canton-transport — ChannelBuilder, TLS (опционально), таймауты; canton-reliability — retry, circuit_breaker (заглушки заменить реализацией по config).

**Критерии приёмки:**
- LedgerClient использует transport для построения channel (или явный выбор: raw tonic vs transport).
- Reliability: retry/circuit_breaker по конфигу (reliability.*).
- research/05-grpc-protobuf-rust.md, research/07-production-ready-patterns.md.

---

## Phase 4: Conversion и high-level API

**Цель:** конвертация canton_core::types::Commands → proto Commands; высокоуровневый CantonClient (или аналог из research/08), переиспользующий LedgerClient.

**Критерии приёмки:**
- Конвертеры domain → proto для Commands и при необходимости Value/Identifier.
- Публичный API: submit domain commands, get_ledger_end, опционально стримы (Completion, Update).
- research/08 (Public API Layer, CantonClient).

---

## Конфигурация

- **Разработка:** config/example.yaml (ledger_api → DevNet, docs/DEVNET_PARTICIPANT.md).
- **Production:** config/example-production.yaml → config/local.yaml (TLS, секреты в local; config/local.yaml в .gitignore).

---

## Порядок действий

1. PRE_DEVELOPMENT_CHECKLIST.md — окружение, сборка, тесты, clippy.
2. Выбрать фазу (по умолчанию Phase 1 → 2 → 3 → 4).
3. После изменений: `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`.
4. Обновлять блок ПРОГРЕСС в начале этого файла.
