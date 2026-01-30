# Ledger API proto (v2)

## Источник и версия (зафиксировано в проекте)

- **Версия API**: v2 (`com.daml.ledger.api.v2`).
- **Источники** (любой на выбор):
  1. **Daml SDK** — архив protobufs из [релизов](https://github.com/digital-asset/daml/releases) (например `protobufs-2.10.x.zip`) или из репозитория `ledger-api/grpc-definitions`.
  2. **digital-asset/daml** — ветка `main`, путь: `ledger-api/grpc-definitions/src/main/protobuf/` (структура `com/daml/ledger/api/v1/` и `com/daml/ledger/api/v2/`).
  3. **Canton** (DACH-NY/canton) — если используется Canton-специфичный набор proto, см. репо Canton.

Документация API: [Ledger API Reference](https://docs.daml.com/app-dev/grpc/proto-docs.html) (v1 и v2).

## Требуемая структура в этом крейте

Файлы должны лежать так (относительно `crates/canton-ledger-api/`):

```
proto/
├── README.md                    (этот файл)
└── com/
    └── daml/
        └── ledger/
            └── api/
                └── v2/
                    ├── command_service.proto
                    ├── command_submission_service.proto
                    ├── command_completion_service.proto
                    ├── update_service.proto       # в v2 заменяет transaction_service для стримов
                    ├── state_service.proto        # в v2: GetActiveContracts, GetLedgerEnd
                    ├── party_management_service.proto  # v1/admin или v2 — см. источник
                    ├── package_service.proto
                    ├── version_service.proto
                    ├── ledger_identity_service.proto    # опционально, в v2 deprecated
                    ├── commands.proto             # и зависимости: value, event, completion, participant_offset, transaction_filter, ...
                    ├── completion.proto
                    ├── participant_offset.proto
                    ├── transaction.proto
                    ├── transaction_filter.proto
                    └── ...                        # остальные из grpc-definitions v2
```

В v2 часть сервисов переименована/объединена: активные контракты — `StateService`, стрим транзакций — `UpdateService`. Proto v2 могут импортировать типы из v1 (`com.daml.ledger.api.v1`), поэтому в том же репо нужны и `com/daml/ledger/api/v1/` (value.proto, event.proto, commands.proto и т.д.), либо копируем и их в `proto/com/daml/ledger/api/v1/`.

## Минимальный набор для build.rs (текущий)

`build.rs` компилирует только если присутствует `proto/com/daml/ledger/api/v2/command_service.proto`. Список файлов, которые он подключает:

- command_service.proto  
- command_submission_service.proto  
- command_completion_service.proto  
- transaction_service.proto *(в v2 может быть update_service)*  
- active_contracts_service.proto *(в v2 — state_service)*  
- party_management_service.proto  
- package_service.proto  
- ledger_identity_service.proto  

При переносе с v1 на v2 имена/пути сервисов могут отличаться; при необходимости `build.rs` и этот список обновляют под фактический набор из выбранного источника.

## Как положить proto

1. **Из Daml SDK (релиз)**  
   Скачать `protobufs-*.zip` из https://github.com/digital-asset/daml/releases, распаковать и скопировать `com/daml/ledger/api/v2/` (и при необходимости v1) в `proto/com/daml/ledger/api/`.

2. **Клон репо daml**  
   ```bash
   git clone --depth 1 https://github.com/digital-asset/daml.git /tmp/daml
   cp -r /tmp/daml/ledger-api/grpc-definitions/src/main/protobuf/com/daml/ledger/api/* \
         crates/canton-ledger-api/proto/com/daml/ledger/api/
   ```

3. **Git submodule** (опционально)  
   Добавить `ledger-api/grpc-definitions` как submodule и в `build.rs` указывать путь на submodule (либо копировать файлы скриптом в `proto/` при сборке).

После появления файлов `cargo build -p canton-ledger-api` сгенерирует клиентов (если `build.rs` настроен на это) и выставит `proto_compiled`. Пока proto нет, сборка крейта проходит без ошибок, но Ledger API клиент остаётся заглушкой.
