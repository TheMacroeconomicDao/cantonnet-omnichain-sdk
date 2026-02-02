# Анализ экспертизы и план интеграции в DEFI_SDK_MASTER_PROMPT

**Дата:** 2025-02-02  
**Версия:** 1.0  
**Статус:** Черновик для утверждения

---

## Исполнительное резюме

Проведён сравнительный анализ [`DEFI_SDK_MASTER_PROMPT.md`](../prompts/DEFI_SDK_MASTER_PROMPT.md) (~2000 строк) и [`DEFI_SDK_EXPERTISE_RESEARCH.md`](../docs/DEFI_SDK_EXPERTISE_RESEARCH.md) (~1445 строк). Определены критические пробелы в мастер-промте, которые необходимо устранить для улучшения SDL (Software Development Lifecycle) при реализации DeFi SDK на Rust.

**Ключевые выводы:**
- Мастер-промт хорошо структурирован и содержит детальные цепи реализации
- Отсутствуют критические аспекты: асинхронная модель Ledger API, дедупликация команд, фильтры Canton 2.8+, observability, безопасность
- Экспертиза содержит актуальные best practices 2024-2025, которые необходимо интегрировать

---

## 1. Критические пробелы (внедрить немедленно)

### Пробел 1: Асинхронная архитектура Ledger API

**Текущее состояние:**
- Part 6.2 упоминает: "стандартный Ledger API v2 — асинхронный submit; для submit_and_wait нужен CommandCompletionService"
- Нет детального описания модели submit vs completion
- Нет объяснения, почему submit не возвращает результат исполнения

**Что добавить (из экспертизы, разделы 1.1, 1.2, 23.4):**
- Явное описание двух потоков данных: команды → ledger, транзакции/события ← ledger
- Разделение Command Submission Service и Command Completion Service
- Change ID = (submitting parties, application_id, command_id) как ключ дедупликации
- Submit возвращает сразу после парсинга, без гарантии исполнения
- Completion stream для корреляции результата по command_id
- submit_and_wait паттерн: submit + SubscribeCompletion + ожидание по command_id

**Где в мастер-промте:**
- Part 1.2 (Prompt chains) — добавить примечание об асинхронной модели
- Part 4.2 (Разрывы) — добавить "Асинхронная модель и completion handling"
- Part 6.2 (LedgerClient) — расширить описание submit_domain_commands
- Part 7 (Chain 2) — добавить подшаг для completion handling

**Почему критично:**
- Непонимание асинхронной модели приведёт к ошибкам в архитектуре приложения
- Без правильной обработки completions невозможно реализовать надёжный submit_and_wait
- Дедупликация команд критична для идемпотентности при retry

---

### Пробел 2: Дедупликация команд и Change ID

**Текущее состояние:**
- Part 14.1 (Commands) упоминает command_id, но без контекста дедупликации
- Нет описания application_id, submission_id, deduplication_period
- Нет объяснения поведения при retry

**Что добавить (из экспертизы, разделы 1.3, 12.8, 34):**
- Change ID = (submitting parties, application_id, command_id)
- Дедупликация в течение deduplication period (по умолчанию из конфига участника)
- Повторная отправка с тем же change ID → отклонение или идемпотентный возврат
- submission_id — для корреляции в completion, не переиспользовать
- При retry после таймаута использовать тот же command_id
- При новой бизнес-операции — новый command_id (UUID v4)
- application_id — константа для приложения (например "canton-defi-sdk")

**Где в мастер-промте:**
- Part 6.2 (LedgerClient) — добавить поля Commands с описанием
- Part 12.4 (InstitutionalAssetPayload) — добавить пример с command_id
- Part 14.1 (Minimal CreateCommand) — добавить application_id, command_id, deduplication_period
- Part 37.2 (Scenario: Conversion roundtrip) — добавить проверку command_id

**Почему критично:**
- Без правильной дедупликации возможны дубликаты транзакций при retry
- Неправильный retry может привести к двойному исполнению команд
- command_id критичен для корреляции в completion stream

---

### Пробел 3: Фильтры Canton 2.8+ (template_filters vs template_ids)

**Текущее состояние:**
- Part 5.3 упоминает template_ids в конфиге
- Нет предупреждения о deprecated template_ids в Canton 2.8+
- Нет описания template_filters и interface_filters

**Что добавить (из экспертизы, разделы 1.5, 18, 33):**
- В Canton 2.8+ template_ids в InclusiveFilter deprecated
- Рекомендуется использовать template_filters (Map Identifier → Filter.Template)
- interface_filters (Map Identifier → Filter.Interface) для интерфейсов
- TransactionFilter с filtersByParty и inclusive
- Пример фильтра для партии Alice и шаблона InstitutionalAsset
- Минимальная версия Canton 2.8 в требованиях

**Где в мастер-промте:**
- Part 5.3 (Идентификаторы шаблонов) — добавить предупреждение о deprecated
- Part 6.2 (LedgerClient) — добавить описание TransactionFilter
- Part 31.3 (Конфиг Ledger API) — добавить template_filters
- Part 42 (Canton Ledger API V2 Services) — добавить примечание о фильтрах

**Почему критично:**
- Использование deprecated template_ids может привести к пустым результатам в новых участниках
- Несовместимость с Canton 2.8+ при обновлении инфраструктуры

---

### Пробел 4: Offsets и восстановление после сбоев

**Текущее состояние:**
- Part 4.1 упоминает StateService и get_ledger_end()
- Нет описания offset и его использования для восстановления
- Нет сценария failover

**Что добавить (из экспертизы, разделы 1.7, 23.1-23.3):**
- Offset — непрозрачная строка байт, выдаётся для каждой транзакции
- Последнее сообщение Active Contracts Service содержит offset
- Подписка на Transaction Service с этого offset даёт согласованное продолжение
- При перезапуске: загрузить сохранённый offset, подписаться с него
- Failover: приложение не может прозрачно переключиться на другого участника (разные offset)
- При смене участника нужно заново выстраивать состояние через Active Contracts

**Где в мастер-промте:**
- Part 4.2 (Разрывы) — добавить "Offsets и восстановление состояния"
- Part 6.2 (LedgerClient) — добавить методы для работы с offset
- Part 8.2 (Тесты) — добавить тест восстановления после перезапуска
- Part 16 (Risks & Mitigation) — добавить риск "Разные offset на участниках"

**Почему критично:**
- Без сохранения offset невозможно корректно восстановить состояние после перезапуска
- Потеря событий при неправильном восстановлении

---

### Пробел 5: Ошибки SDK (thiserror, контекст)

**Текущее состояние:**
- Part 4.1 упоминает SdkError, SdkResult
- Нет детального списка вариантов ошибок
- Нет рекомендаций по thiserror/anyhow

**Что добавить (из экспертизы, разделы 2.6, 22, 31.4, 38):**
- Полный список вариантов SdkError: ConfigLoadFailed, ConnectFailed, ConversionError, SubmitFailed, ContractNotFound, ValidationFailed, BusinessRuleViolation, Timeout
- Использование thiserror для библиотек (derive(Error, Display), #[source])
- anyhow для приложений (контекст через .context())
- Не логировать секреты и полные payload в ошибках
- Таблица ошибок с контекстом (раздел 31.4 экспертизы)

**Где в мастер-промте:**
- Part 4.1 (Что есть в SDK) — расширить описание error.rs
- Part 38 (ERROR CODES & MESSAGES) — уже есть, но нужно расширить
- Part 8.3 (Риски и откат) — добавить риск "Неправильная обработка ошибок"

**Почему критично:**
- Без структурированных ошибок невозможно корректно обрабатывать сбои
- Утечка секретов в логах — уязвимость безопасности

---

### Пробел 6: Безопасность (секреты, логирование, TLS)

**Текущее состояние:**
- Part 6.1 упоминает tls: false в конфиге
- Нет детальных рекомендаций по безопасности
- Нет предупреждений о логировании секретов

**Что добавить (из экспертизы, разделы 7, 28, 25):**
- Секреты (auth_token) только из env или секрет-менеджера, не в файле в репо
- Не логировать auth_token, полные payload команд
- TLS обязателен в продакшене
- Минимальные права: только нужные партии в act_as/read_as
- JWT/JWKS конфигурация Canton (issuer, audience, jwksUrl, leeway)
- User Management Service для централизованного управления правами

**Где в мастер-промте:**
- Part 6.1 (Config) — добавить секцию безопасности
- Part 8.3 (Риски и откат) — добавить риск "Утечка секретов"
- Part 16 (Risks & Mitigation) — добавить риск "Отсутствие TLS в проде"
- Part 27 (API REQUEST/RESPONSE SCHEMAS & ENV) — добавить предупреждения о секретах

**Почему критично:**
- Утечка секретов — критическая уязвимость
- Отсутствие TLS — перехват трафика
- Избыточные права — нарушение принципа наименьших привилегий

---

## 2. Важные улучшения (внедрить в ближайшее время)

### Улучшение 1: Observability (OpenTelemetry, tracing)

**Текущее состояние:**
- canton-observability — минимальная реализация
- Нет описания спанов и метрик

**Что добавить (из экспертизы, разделы 4, 21):**
- Feature-gate "observability" для опциональных зависимостей
- Спаны: ledger.submit, ledger.submit_and_wait, ledger.create_contract, ledger.exercise
- Метрики: ledger_commands_submitted_total, ledger_commands_failed_total, ledger_submit_duration_seconds, ledger_active_contracts_fetch_duration_seconds
- tracing + tracing-opentelemetry мост
- OTLP exporter в Collector (feature "grpc-tonic")

**Где в мастер-промте:**
- Part 5.2 (Размещение по крейтам) — добавить canton-observability
- Part 7 (Chain 5) — добавить подшаг для observability
- Part 39 (DEPENDENCY VERSIONS) — добавить opentelemetry зависимости

**Почему важно:**
- Без observability невозможно отлаживать продакшен
- Метрики критичны для мониторинга производительности

---

### Улучшение 2: Transport (Tower retry/timeout, connection pool)

**Текущее состояние:**
- canton-transport — минимальная реализация
- Нет описания retry, timeout, connection pool

**Что добавить (из экспертизы, разделы 3, 40):**
- Tower retry layer: политика для transient errors (Unavailable, ResourceExhausted)
- Tower timeout layer: request_timeout на уровне вызова
- Опциональный connection pool (soda-pool, mobc-tonic) для высокой нагрузки
- Keep-Alive настройки HTTP/2
- TLS конфигурация на клиенте

**Где в мастер-промте:**
- Part 5.2 (Размещение по крейтам) — расширить canton-transport
- Part 7 (Chain 1) — добавить подшаг для transport
- Part 39 (DEPENDENCY VERSIONS) — добавить tower зависимости

**Почему важно:**
- Без retry временные сбои сети приведут к ошибкам
- Без timeout возможны зависания запросов

---

### Улучшение 3: Traits и dependency injection (тестируемость)

**Текущее состояние:**
- Part 6.6 упоминает ComplianceProvider и OracleProvider как интерфейсы
- Нет детального описания трейтов и DI

**Что добавить (из экспертизы, разделы 2.4, 2.5):**
- Трейты как интерфейсы; функции принимают обобщённые параметры по трейту
- Разделение: создание объектов снаружи, использование внутри
- Инжекция зависимостей через конструктор
- Статическая диспетчеризация (generics) — нулевые накладные расходы
- Динамическая (dyn Trait) — для моков в тестах
- Builder pattern (derive_builder, typed_builder) для конфигов

**Где в мастер-промте:**
- Part 5.1 (Слои) — добавить описание DI
- Part 6.6 (Compliance & Oracle) — расширить описание трейтов
- Part 7 (Chain 5) — добавить подшаг для DI

**Почему важно:**
- Без DI невозможно тестировать доменные сервисы без участника
- Моки критичны для юнит-тестов

---

### Улучшение 4: Тестирование (стратегии, mocktail, feature gates)

**Текущее состояние:**
- Part 8.2 упоминает тесты, но без детальной стратегии
- Нет описания mocktail, feature gates для интеграционных тестов

**Что добавить (из экспертизы, разделы 6, 27):**
- Юнит-тесты: все зависимости через трейты, моки реализуют трейты
- Интеграционные тесты: feature "integration" или env CANTON_LEDGER_GRPC
- mocktail — мок HTTP и gRPC серверов, в т.ч. стриминг
- Изоляция: уникальные command_id (UUID) в тестах
- Очистка данных или изолированный участник/песочница

**Где в мастер-промте:**
- Part 8.2 (Тесты) — расширить описание стратегии
- Part 15 (TEST MATRIX) — добавить тесты с mocktail
- Part 39 (DEPENDENCY VERSIONS) — добавить mocktail в dev-dependencies

**Почему важно:**
- Без чёткой стратегии тестирования невозможно обеспечить качество
- Интеграционные тесты без feature gates будут падать в CI

---

### Улучшение 5: Rust API Guidelines (naming, common traits, SemVer)

**Текущее состояние:**
- Нет явного следования Rust API Guidelines
- Нет описания naming conventions

**Что добавить (из экспертизы, разделы 2.1, 12.4, 26):**
- Naming: UpperCamelCase для типов, snake_case для функций/методов, SCREAMING_SNAKE_CASE для констант
- Конверсии: as_ (дешёвые), to_ (дорогие), into_ (consuming)
- Common traits: Clone, PartialEq, Eq, Hash, Debug, Default, Serialize, Deserialize где применимо
- #[non_exhaustive] для enum и struct с возможностью расширения
- SemVer: MAJOR — несовместимые изменения, MINOR — новая функциональность, PATCH — исправления
- Допустимые изменения в MINOR: новые методы, новые варианты enum с #[non_exhaustive]

**Где в мастер-промте:**
- Part 5.1 (Слои) — добавить раздел "Rust API Guidelines"
- Part 12 (DETAILED TYPE DEFINITIONS) — добавить #[non_exhaustive] к enum
- Part 36 (RUST TYPE SKELETONS) — добавить #[non_exhaustive] к статусам

**Почему важно:**
- Следование API Guidelines улучшает DX (developer experience)
- SemVer критичен для обратной совместимости

---

### Улучшение 6: Конфигурация (12-factor, config-rs, env override)

**Текущее состояние:**
- Part 6.1 описывает LedgerApiConfig, но без слоёв
- Нет описания config-rs и приоритета источников

**Что добавить (из экспертизы, разделы 5, 16, 35):**
- 12-factor: конфигурация в окружении, секреты не в репо
- config-rs: ConfigBuilder с add_source(File) + add_source(Environment)
- Порядок приоритета: дефолты → файл → env
- #[serde(default)] и #[serde(default = "path")] для опциональных полей
- Секреты (auth_token) только из env, не из файла
- Пример полной загрузки конфига (раздел 37.1 экспертизы)

**Где в мастер-промте:**
- Part 6.1 (Config) — расширить описание config-rs
- Part 22 (CONFIG YAML FULL EXAMPLE) — добавить пример с env override
- Part 46 (ENV & CONFIG QUICK REF) — добавить таблицу переменных окружения

**Почему важно:**
- Без 12-factor невозможно различать среды (dev/stage/prod)
- Без env override невозможно гибко конфигурировать в проде

---

## 3. Полезные дополнения (внедрить по возможности)

### Дополнение 1: Completion status и обработка ошибок

**Что добавить (из экспертизы, раздел 34):**
- Completion status: Success, Failed, AbortedDueToShutdown, MaxRetriesReached
- При Failed извлекать reason и маппить в SdkError::SubmitFailed
- Retry: при таймауте повторять с тем же command_id, при постоянной ошибке не повторять

**Где в мастер-промте:**
- Part 6.2 (LedgerClient) — добавить описание обработки completion
- Part 38 (ERROR CODES & MESSAGES) — добавить SubmitFailed с reason

---

### Дополнение 2: Daml Decimal и Numeric в proto

**Что добавить (из экспертизы, раздел 41):**
- Decimal в Daml имеет precision и scale
- В proto передаётся как строка или Numeric (зависит от версии)
- Парсинг в rust_decimal::Decimal на стороне SDK
- Единообразие точности: scale 18 для денежных сумм

**Где в мастер-промте:**
- Part 7 (Chain 2) — добавить подшаг для конвертации Decimal
- Part 12 (DETAILED TYPE DEFINITIONS) — добавить примечание о precision

---

### Дополнение 3: Party и ContractId в proto

**Что добавить (из экспертизы, раздел 42):**
- Party — строка (party id), непрозрачный идентификатор
- ContractId — строка (contract id), непрозрачный идентификатор
- В SDK: PartyId и ContractId как type-обёртки или type alias

**Где в мастер-промте:**
- Part 12 (DETAILED TYPE DEFINITIONS) — добавить описание PartyId, ContractId

---

### Дополнение 4: Identifier и record_id в proto

**Что добавить (из экспертизы, раздел 43):**
- Identifier: package_id, module_name, entity_name
- record_id в Record: Identifier типа записи
- При конвертации DamlRecord → proto Record заполнять record_id

**Где в мастер-промте:**
- Part 7 (Chain 2) — добавить подшаг для identifier_to_proto

---

### Дополнение 5: RWA/compliance (регуляторика, KYC/AML, settlement)

**Что добавить (из экспертизы, разделы 8, 19, 29):**
- Регуляторика 2024: EU MiCA, Travel Rule, FATF
- KYC/AML через verifiable credentials (VC) и zero-knowledge proofs (ZKP)
- Токенизация RWA: ERC-3643, оракулы для офчейн-фактов
- Settlement: US T+1 с мая 2024, T+0 — редкий
- OracleProvider: get_price, get_treasury_yield, get_property_valuation

**Где в мастер-промте:**
- Part 6.6 (Compliance & Oracle) — расширить описание RWA/compliance
- Part 26 (ADDITIONAL DEFI SERVICES) — добавить раздел RWA/compliance

---

### Дополнение 6: Canton конфигурация сервера

**Что добавить (из экспертизы, разделы 20, 25):**
- Типичные порты: Ledger API gRPC (5001 или 30501), Admin API (5002), HTTP JSON API (7575)
- Конфиг участника: address, port, tls, maxInboundMessageSize, keepAlive
- JWT/JWKS: issuer, audience, jwksUrl, leeway
- Для клиента SDK: grpc_endpoint, tls, auth_token, max_inbound_message_size, keep_alive

**Где в мастер-промте:**
- Part 6.1 (Config) — добавить секцию "Canton server config reference"
- Part 46 (ENV & CONFIG QUICK REF) — добавить таблицу портов

---

### Дополнение 7: Async runtime (Tokio vs async-std)

**Что добавить (из экспертизы, раздел 15, 12.1):**
- Tokio — мультипоточный work-stealing, большая экосистема
- async-std — проще, меньше зависимостей
- Для SDK с tonic де-факто стандарт — Tokio
- Return Type Notation (RTN) для Send bounds в async traits

**Где в мастер-промте:**
- Part 5.1 (Слои) — добавить раздел "Async runtime"
- Part 39 (DEPENDENCY VERSIONS) — добавить tokio

---

### Дополнение 8: Streams (стримы Ledger API)

**Что добавить (из экспертизы, раздел 17, 12.2):**
- Transaction Service и Active Contracts Service возвращают gRPC streams
- В Rust: futures::Stream / StreamExt (next(), then())
- Обработка: while let Some(msg) = stream.message().await
- Учёт offset в последнем сообщении ACS

**Где в мастер-промте:**
- Part 6.2 (LedgerClient) — добавить описание streaming
- Part 7 (Chain 2) — добавить подшаг для streaming

---

### Дополнение 9: Документация и doctests

**Что добавить (из экспертизы, разделы 2.8, 12.10):**
- Примеры в /// с тройными бэктиками компилируются и запускаются как тесты
- Скрытие служебного кода: строки с # не показываются в доке
- Для каждого публичного модуля/крейта: краткое описание назначения
- Для основных типов и методов: /// с описанием и примером

**Где в мастер-промте:**
- Part 8 (ACCEPTANCE CRITERIA & TESTING) — добавить критерий "Doctests для основных типов"
- Part 6 (API SURFACE SPECIFICATION) — добавить требование документации

---

### Дополнение 10: Feature flags и опциональные зависимости

**Что добавить (из экспертизы, разделы 2.9, 12.3):**
- [features] + optional = true в зависимостях
- dep:crate-name — явная привязка фичи к опциональной зависимости
- default = ["rustls-tls"], "native-tls", "observability", "serde"
- Минимальный default для встраивания

**Где в мастер-промте:**
- Part 39 (DEPENDENCY VERSIONS) — добавить таблицу features
- Part 5.2 (Размещение по крейтам) — добавить описание features

---

## 4. План внедрения

### Фаза 1: Критические улучшения (неделя 1)

**Цель:** Устранить пробелы, влияющие на архитектуру, безопасность и производительность.

- [ ] **1.1 Асинхронная архитектура Ledger API**
  - Обновить Part 1.2: добавить описание submit vs completion
  - Обновить Part 4.2: добавить раздел "Асинхронная модель и completion handling"
  - Обновить Part 6.2: расширить описание submit_domain_commands
  - Обновить Part 7 (Chain 2): добавить подшаг для completion handling
  - Добавить пример submit_and_wait с SubscribeCompletion

- [ ] **1.2 Дедупликация команд и Change ID**
  - Обновить Part 6.2: добавить поля Commands (application_id, command_id, submission_id, deduplication_period)
  - Обновить Part 12.4: добавить пример с command_id (UUID v4)
  - Обновить Part 14.1: добавить application_id, command_id, deduplication_period в Minimal CreateCommand
  - Обновить Part 37.2: добавить проверку command_id в тестах

- [ ] **1.3 Фильтры Canton 2.8+**
  - Обновить Part 5.3: добавить предупреждение о deprecated template_ids
  - Обновить Part 6.2: добавить описание TransactionFilter с template_filters
  - Обновить Part 31.3: добавить template_filters в конфиг
  - Обновить Part 42: добавить примечание о фильтрах Canton 2.8+

- [ ] **1.4 Offsets и восстановление после сбоев**
  - Обновить Part 4.2: добавить раздел "Offsets и восстановление состояния"
  - Обновить Part 6.2: добавить методы для работы с offset
  - Обновить Part 8.2: добавить тест восстановления после перезапуска
  - Обновить Part 16: добавить риск "Разные offset на участниках"

- [ ] **1.5 Ошибки SDK (thiserror, контекст)**
  - Обновить Part 4.1: расширить описание error.rs
  - Обновить Part 38: расширить таблицу ошибок (добавить все варианты из раздела 31.4)
  - Добавить рекомендации по thiserror/anyhow
  - Добавить предупреждения о логировании секретов

- [ ] **1.6 Безопасность (секреты, логирование, TLS)**
  - Обновить Part 6.1: добавить секцию безопасности
  - Обновить Part 8.3: добавить риск "Утечка секретов"
  - Обновить Part 16: добавить риск "Отсутствие TLS в проде"
  - Обновить Part 27: добавить предупреждения о секретах
  - Добавить рекомендации по JWT/JWKS и User Management Service

**Критерий завершения:** Все 6 критических пробелов устранены, мастер-промт обновлён.

---

### Фаза 2: Важные улучшения (неделя 2-3)

**Цель:** Улучшить качество кода, тестируемость, observability.

- [ ] **2.1 Observability (OpenTelemetry, tracing)**
  - Обновить Part 5.2: добавить canton-observability
  - Обновить Part 7 (Chain 5): добавить подшаг для observability
  - Обновить Part 39: добавить opentelemetry зависимости
  - Добавить описание спанов и метрик

- [ ] **2.2 Transport (Tower retry/timeout, connection pool)**
  - Обновить Part 5.2: расширить canton-transport
  - Обновить Part 7 (Chain 1): добавить подшаг для transport
  - Обновить Part 39: добавить tower зависимости
  - Добавить описание retry/timeout/connection pool

- [ ] **2.3 Traits и dependency injection (тестируемость)**
  - Обновить Part 5.1: добавить описание DI
  - Обновить Part 6.6: расширить описание трейтов
  - Обновить Part 7 (Chain 5): добавить подшаг для DI
  - Добавить описание builder pattern

- [ ] **2.4 Тестирование (стратегии, mocktail, feature gates)**
  - Обновить Part 8.2: расширить описание стратегии
  - Обновить Part 15: добавить тесты с mocktail
  - Обновить Part 39: добавить mocktail в dev-dependencies
  - Добавить описание feature "integration"

- [ ] **2.5 Rust API Guidelines (naming, common traits, SemVer)**
  - Обновить Part 5.1: добавить раздел "Rust API Guidelines"
  - Обновить Part 12: добавить #[non_exhaustive] к enum
  - Обновить Part 36: добавить #[non_exhaustive] к статусам
  - Добавить описание SemVer

- [ ] **2.6 Конфигурация (12-factor, config-rs, env override)**
  - Обновить Part 6.1: расширить описание config-rs
  - Обновить Part 22: добавить пример с env override
  - Обновить Part 46: добавить таблицу переменных окружения
  - Добавить описание 12-factor

**Критерий завершения:** Все 6 важных улучшений внедрены, мастер-промт обновлён.

---

### Фаза 3: Полезные дополнения (неделя 4+)

**Цель:** Дополнить документацию, примеры, справочные материалы.

- [ ] **3.1 Completion status и обработка ошибок**
  - Обновить Part 6.2: добавить описание обработки completion
  - Обновить Part 38: добавить SubmitFailed с reason

- [ ] **3.2 Daml Decimal и Numeric в proto**
  - Обновить Part 7 (Chain 2): добавить подшаг для конвертации Decimal
  - Обновить Part 12: добавить примечание о precision

- [ ] **3.3 Party и ContractId в proto**
  - Обновить Part 12: добавить описание PartyId, ContractId

- [ ] **3.4 Identifier и record_id в proto**
  - Обновить Part 7 (Chain 2): добавить подшаг для identifier_to_proto

- [ ] **3.5 RWA/compliance (регуляторика, KYC/AML, settlement)**
  - Обновить Part 6.6: расширить описание RWA/compliance
  - Обновить Part 26: добавить раздел RWA/compliance

- [ ] **3.6 Canton конфигурация сервера**
  - Обновить Part 6.1: добавить секцию "Canton server config reference"
  - Обновить Part 46: добавить таблицу портов

- [ ] **3.7 Async runtime (Tokio vs async-std)**
  - Обновить Part 5.1: добавить раздел "Async runtime"
  - Обновить Part 39: добавить tokio

- [ ] **3.8 Streams (стримы Ledger API)**
  - Обновить Part 6.2: добавить описание streaming
  - Обновить Part 7 (Chain 2): добавить подшаг для streaming

- [ ] **3.9 Документация и doctests**
  - Обновить Part 8: добавить критерий "Doctests для основных типов"
  - Обновить Part 6: добавить требование документации

- [ ] **3.10 Feature flags и опциональные зависимости**
  - Обновить Part 39: добавить таблицу features
  - Обновить Part 5.2: добавить описание features

**Критерий завершения:** Все 10 полезных дополнений внедрены, мастер-промт обновлён.

---

## 5. Рекомендации

### Рекомендация 1: Приоритет критических улучшений

Начните с Фазы 1 (критические улучшения) — они влияют на архитектуру, безопасность и производительность. Без них невозможно создать надёжный SDK.

### Рекомендация 2: Инкрементальное внедрение

Внедряйте улучшения по одной теме за раз, тестируйте изменения после каждого обновления. Это позволит быстро обнаружить проблемы и откатиться при необходимости.

### Рекомендация 3: Сохранение обратной совместимости

При обновлении мастер-промта сохраняйте существующую структуру и нумерацию частей. Добавляйте новый контекст, а не заменяйте существующий.

### Рекомендация 4: Использование примеров из экспертизы

Экспертиза содержит готовые примеры кода (разделы 37, 38 экспертизы). Используйте их как основу для обновления мастер-промта.

### Рекомендация 5: Обновление чеклистов

После внедрения улучшений обновите чеклисты в Part 8, Part 18, Part 25 мастер-промта, чтобы они отражали новые требования.

### Рекомендация 6: Документация изменений

Ведите CHANGELOG.md для мастер-промта, фиксируя:
- Версию (например 1.2)
- Дату изменения
- Список добавленных/изменённых разделов
- Причины изменений

### Рекомендация 7: Peer review

После завершения каждой фазы запросите review от:
- Canton/Daml эксперта (для проверки Ledger API аспектов)
- Rust эксперта (для проверки API Guidelines и best practices)
- Security эксперта (для проверки аспектов безопасности)

### Рекомендация 8: Тестирование обновлённого промта

После внедрения улучшений протестируйте мастер-промт:
- Запустите Chain 1–6 с обновлённым промтом
- Проверьте, что все acceptance criteria из Part 8 выполнены
- Убедитесь, что новые требования (например completion handling) отражены в коде

### Рекомендация 9: Обучение команды

Проведите сессию обучения для команды разработчиков:
- Объясните асинхронную модель Ledger API
- Покажите примеры дедупликации команд
- Демонстрируйте работу с offsets и completion stream
- Объясните best practices по безопасности и observability

### Рекомендация 10: Периодический пересмотр экспертизы

Экспертиза актуальна на 2024-2025. Планируйте пересмотр через 6-12 месяцев для учёта новых изменений в Canton/Daml и Rust ecosystem.

---

## 6. Риски и митигация

| Риск | Влияние | Митигация |
|------|----------|-----------|
| **Риск 1:** Неправильное понимание асинхронной модели | Высокое | Добавить детальные примеры submit_and_wait с completion stream; провести обучение команды |
| **Риск 2:** Утечка секретов в логах | Критическое | Добавить явные предупреждения о логировании; использовать thiserror для исключения секретов из Display |
| **Риск 3:** Несовместимость с Canton 2.8+ (deprecated template_ids) | Высокое | Добавить предупреждение о deprecated; указать минимальную версию Canton 2.8 |
| **Риск 4:** Потеря событий при неправильном восстановлении offset | Высокое | Добавить детальное описание работы с offset; добавить тест восстановления после перезапуска |
| **Риск 5:** Дубликаты транзакций при неправильном retry | Высокое | Добавить описание дедупликации по Change ID; использовать UUID v4 для command_id |
| **Риск 6:** Отсутствие TLS в продакшене | Критическое | Добавить требование TLS в проде; добавить риск в Part 16 |
| **Риск 7:** Сложность внедрения всех улучшений | Среднее | Разбить на фазы; внедрять инкрементально; тестировать после каждого изменения |
| **Риск 8:** Регрессия существующего функционала | Среднее | Запускать cargo test --workspace после каждого изменения; сохранять обратную совместимость |
| **Риск 9:** Устаревание экспертизы | Среднее | Планировать пересмотр через 6-12 месяцев; следить за обновлениями Canton/Daml |
| **Риск 10:** Недостаточное тестирование | Среднее | Добавить feature "integration" для интеграционных тестов; использовать mocktail для моков |

---

## 7. Приложение: Сводная таблица улучшений

| № | Улучшение | Критичность | Фаза | Разделы мастер-промта для обновления | Разделы экспертизы |
|----|------------|--------------|-------|-----------------------------------|-------------------|
| 1 | Асинхронная архитектура Ledger API | Критично | 1 | 1.2, 4.2, 6.2, 7 | 1.1, 1.2, 23.4 |
| 2 | Дедупликация команд и Change ID | Критично | 1 | 6.2, 12.4, 14.1, 37.2 | 1.3, 12.8, 34 |
| 3 | Фильтры Canton 2.8+ | Критично | 1 | 5.3, 6.2, 31.3, 42 | 1.5, 18, 33 |
| 4 | Offsets и восстановление после сбоев | Критично | 1 | 4.2, 6.2, 8.2, 16 | 1.7, 23.1-23.3 |
| 5 | Ошибки SDK (thiserror, контекст) | Критично | 1 | 4.1, 38, 8.3 | 2.6, 22, 31.4, 38 |
| 6 | Безопасность (секреты, логирование, TLS) | Критично | 1 | 6.1, 8.3, 16, 27 | 7, 28, 25 |
| 7 | Observability (OpenTelemetry, tracing) | Важно | 2 | 5.2, 7, 39 | 4, 21 |
| 8 | Transport (Tower retry/timeout, connection pool) | Важно | 2 | 5.2, 7, 39 | 3, 40 |
| 9 | Traits и dependency injection (тестируемость) | Важно | 2 | 5.1, 6.6, 7 | 2.4, 2.5 |
| 10 | Тестирование (стратегии, mocktail, feature gates) | Важно | 2 | 8.2, 15, 39 | 6, 27 |
| 11 | Rust API Guidelines (naming, common traits, SemVer) | Важно | 2 | 5.1, 12, 36 | 2.1, 12.4, 26 |
| 12 | Конфигурация (12-factor, config-rs, env override) | Важно | 2 | 6.1, 22, 46 | 5, 16, 35 |
| 13 | Completion status и обработка ошибок | Полезно | 3 | 6.2, 38 | 34 |
| 14 | Daml Decimal и Numeric в proto | Полезно | 3 | 7, 12 | 41 |
| 15 | Party и ContractId в proto | Полезно | 3 | 12 | 42 |
| 16 | Identifier и record_id в proto | Полезно | 3 | 7 | 43 |
| 17 | RWA/compliance (регуляторика, KYC/AML, settlement) | Полезно | 3 | 6.6, 26 | 8, 19, 29 |
| 18 | Canton конфигурация сервера | Полезно | 3 | 6.1, 46 | 20, 25 |
| 19 | Async runtime (Tokio vs async-std) | Полезно | 3 | 5.1, 39 | 15, 12.1 |
| 20 | Streams (стримы Ledger API) | Полезно | 3 | 6.2, 7 | 17, 12.2 |
| 21 | Документация и doctests | Полезно | 3 | 8, 6 | 2.8, 12.10 |
| 22 | Feature flags и опциональные зависимости | Полезно | 3 | 39, 5.2 | 2.9, 12.3 |

---

## Заключение

Проведённый анализ выявил 6 критических пробелов, 6 важных улучшений и 10 полезных дополнений в [`DEFI_SDK_MASTER_PROMPT.md`](../prompts/DEFI_SDK_MASTER_PROMPT.md). Интеграция экспертизы из [`DEFI_SDK_EXPERTISE_RESEARCH.md`](../docs/DEFI_SDK_EXPERTISE_RESEARCH.md) значительно улучшит SDL при реализации DeFi SDK на Rust.

**Рекомендуемый порядок действий:**
1. Утвердить этот план интеграции
2. Начать с Фазы 1 (критические улучшения)
3. После завершения Фазы 1 перейти к Фазе 2
4. По возможности внедрить Фазу 3
5. Провести peer review и тестирование обновлённого промта

**Ожидаемый результат:**
- Мастер-промт будет содержать актуальные best practices 2024-2025
- Улучшится качество кода, безопасность и observability
- Сократится время на разработку и отладку
- Уменьшится риск ошибок в продакшене

---

**Конец документа.**
