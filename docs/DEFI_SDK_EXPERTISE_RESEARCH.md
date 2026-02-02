# DeFi SDK — Сводная экспертиза (исследование 2024–2025)

**Назначение:** Отдельный документ с актуальной экспертизой из открытых источников для последующего анализа и объединения с мастер-промтом (DEFI_SDK_MASTER_PROMPT.md).  
**Источники:** Официальная документация Daml/Canton, Rust API Guidelines, Azure/Smithy SDK tenets, OpenTelemetry, 12-factor, отраслевые отчёты по RWA/compliance.  
**Дата:** 2025-01-30.

---

## 1. Canton / Daml Ledger API

### 1.1 Асинхронная архитектура (официальная модель)

- Ledger API **асинхронный**: результат команд неизвестен в момент отправки.
- Два потока данных:
  - **Команды** → в ledger (изменение состояния).
  - **Транзакции и события** ← из ledger (изменения состояния).
- Приложение должно:
  - обрабатывать успешные и ошибочные **completions** отдельно от **submission**;
  - считать изменения состояния по асинхронным событиям, а не по ответу на submit.
- Это главный фактор архитектуры приложения: см. [Daml Application Architecture](https://docs.daml.com/app-dev/app-arch.html), [The Ledger API Services](https://docs.daml.com/app-dev/services.html).

### 1.2 Сервисы Ledger API (категории)

**Команды:**

- **Command Submission Service** — приём команд (create, exercise). Возврат сразу после парсинга и приём/отклонение по формату, без гарантии исполнения.
- **Command Completion Service** — стрим completions; привязка к command_id/submission_id.
- **Command Service** — объединяет submission + ожидание completion (submit-and-wait для простых сценариев).

**Чтение:**

- **Transaction Service** — стрим закоммиченных транзакций/событий; подписка с заданного offset.
- **Active Contracts Service** — снимок активных контрактов на момент запроса; последнее сообщение в стриме содержит offset, с которого продолжать Transaction Service.
- **Event Query Service** — запросы по contract ID или contract key (без оффледжерного хранилища).

**Утилиты:**

- Party Management, User Management, Identity Provider Config, Package Service, Version Service, Ledger Configuration Service, Pruning, Metering Report.

**Важно для SDK:** реализовать минимум Command Submission + (StateService или аналог GetLedgerEnd) + при необходимости Active Contracts и Command Completion для submit-and-wait и bootstrap.

### 1.3 Change ID и дедупликация команд

- **Change ID** = (submitting parties, application_id, command_id).
- **Submitting parties** = объединение party и act_as.
- **Application ID** — идентификатор приложения; используется для трассировки и подписки на свои completions.
- **Command ID** — уникальный идентификатор намерения в домене приложения; задаётся приложением.
- **Submission ID** — идентификатор конкретной отправки; в completion для корреляции; **не переиспользовать**.
- **Workflow ID** — опционально; помечает онледжерный workflow.
- Дедупликация: по (change ID) в течение **deduplication period**. Повторная отправка с тем же change ID до истечения периода → отклонение. После completion или истечения периода повторная отправка допустима (идемпотентность при retry).
- Эффективный период дедупликации может быть больше запрошенного; возвращается в completion.
- См. [Command Deduplication](https://docs.daml.com/app-dev/command-deduplication.html).

### 1.4 Конфигурация Ledger API (Canton)

- **TLS** — для шифрования и (опционально) mutual auth.
- **Keep Alive** — LedgerApiKeepAliveServerConfig; важно для длинных соединений.
- **Порты** — address (по умолчанию 127.0.0.1), internalPort для Ledger API.
- **Max message size** — maxInboundMetadataSize (по умолчанию GrpcUtil.DEFAULT_MAX_HEADER_LIST).
- **JWT** — leeway, audience, scope; JWKS URL для проверки подписи токенов.
- См. [API Configuration](https://docs.daml.com/canton/usermanual/apis.html), [gRPC Ledger API Configuration](https://docs.digitalasset.com/operate/3.4/howtos/configure/apis/ledger_api.html).

### 1.5 Шаблоны и фильтры (deprecation template_ids)

- В Ledger API v2/Canton 2.8+ **template_ids** в InclusiveFilter deprecated.
- Рекомендуется: **template_filters** (Map Identifier → Filter.Template) и **interface_filters** (Map Identifier → Filter.Interface).
- В GetActiveContractsRequest / GetTransactionsRequest использовать TransactionFilter с filtersByParty и inclusive с template_filters, а не только списком template_ids.
- См. Java API InclusiveFilter: конструкторы с templateIds deprecated с Daml 2.4/2.8.

### 1.6 Explicit contract disclosure (Canton 2.7+)

- Возможность прикладывать к команде **disclosed contracts** (полученные от третьих сторон), чтобы участник мог проверить контракты без предварительного раскрытия через обычный поток.
- В TemplateFilter/InterfaceFilter: include_created_event_blob для использования в disclosed contract при submission.
- См. [Explicit contract disclosure](https://docs.digitalasset.com/build/3.3/sdlc-howtos/applications/develop/explicit-contract-disclosure.html).

### 1.7 Offsets и восстановление после сбоев

- **Offset** — непрозрачная строка байт, выдаётся участником для каждой транзакции; на одном участнике порядок лексикографический.
- Состояние (активные контракты) актуально на конкретном offset; последнее сообщение Active Contracts Service содержит этот offset.
- Подписка на Transaction Service с этого offset даёт согласованное продолжение без повторного вызова Active Contracts.
- **Failover:** приложение от имени партии **не может** прозрачно переключиться на другого участника из-за разных offset; при смене участника нужно заново выстраивать состояние (например, через Active Contracts на новом участнике).
- Команды при таймауте во время длинного failover можно безопасно повторно отправлять благодаря дедупликации.
- См. [Daml Application Architecture](https://docs.daml.com/app-dev/app-arch.html), [Implementing HA](https://docs.daml.com/deploy-daml/infrastructure-architecture/high-availability/ha-and-scaling/implementing-ha.html).

### 1.8 Daml Value → Protobuf (Record, типы)

- Daml-типы передаются в gRPC как protobuf-сообщения.
- **Record** → protobuf `Record`: record_id (package id, type name), fields — массив { label, value }.
- Примитивы: int64, text, decimal, bool, party, timestamp, date, contract_id, list, optional, variant и т.д.
- Используется в CreateCommand.create_arguments, ExerciseCommand.choice_argument, в событиях Transaction/ActiveContracts.
- См. [How Daml Types are Translated to Protobuf](https://docs.daml.com/app-dev/grpc/daml-to-ledger-api.html).
- Rust: crate `daml-grpc` — биндинги к Ledger API gRPC.

### 1.9 User Management Service (Daml 2.0+)

- **User** — локальная сущность участника; привязка к партиям через права act_as и read_as.
- Один пользователь — primary party + возможность act_as/read_as для других партий.
- Управление пользователями и правами через User Management Service; не ограничено числом партий в одном JWT (в отличие от старых multi-party токенов).
- Application ID часто совпадает с user id.
- См. [Parties and Users](https://docs.daml.com/app-dev/parties-users.html).

### 1.10 Authorization (JWT, JWKS)

- Проверка токена: издатель, целостность, срок действия, права (act_as, read_as и т.д.).
- JWKS URL издателя — для получения ключей проверки подписи.
- Конфигурация Canton: leeway для времени, audience, scope.
- См. [Authorization](https://docs.daml.com/app-dev/authorization.html), [Secure DAML Infrastructure JWT JWKS](https://blog.digitalasset.com/blog/secure-daml-infrastructure-part-2-jwt-jwks-and-auth0).

---

## 2. Rust SDK: архитектура и принципы

### 2.1 Rust API Guidelines (кратко)

- **C-GOOD-ERR:** типы ошибок осмысленные и хорошо оформленные (Error, Display, источник, совместимость с anyhow/thiserror).
- **C-COMMON-TRAITS:** типы реализуют все уместные общие трейты (Clone, PartialEq, Eq, Hash, Debug, Default и т.д.), чтобы downstream не упирался в orphan rules.
- **C-CASE / C-CONV:** именование по RFC 430 (UpperCamelCase для типов, snake_case для функций/методов, SCREAMING_SNAKE_CASE для констант); конверсии: as_ (дешёвые), to_ (дорогие), into_ (consuming).
- **Cargo features:** опциональные зависимости, минимальный набор default, возможность no_std где уместно; не более 300 фич на crate (crates.io).
- Чеклист: [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/checklist.html).

### 2.2 Azure SDK for Rust (tenets)

- **Idiomatic** — следование идиомам Rust.
- **Productivity first** — полнота, расширяемость и производительность вторичны относительно удобства использования.
- Async с подключаемым рантаймом (по умолчанию Tokio).
- Типобезопасность, потокобезопасность, единые ClientOptions и согласованная обработка ошибок (azure_core::Error).
- Модульность: выбор нужных крейтов без тяги всего.
- См. [Rust Guidelines Introduction](https://azure.github.io/azure-sdk/rust_introduction.html).

### 2.3 Smithy Rust (AWS SDK) tenets

- **Batteries included, but replaceable** — хорошие дефолты (например Tokio), но возможность подменить компоненты (другой runtime, другой HTTP-стек).
- **Make common problems easy** — интуитивные API, документация, примеры; API, устойчивые к неправильному использованию (особенно в async).
- **Design for the future** — эволюция без ломания клиентов; маленькое ядро, без утечки внутренних зависимостей; обратимость решений.
- Высокоуровневый «fluent» API + низкоуровневый для тонкой настройки.
- См. [Smithy Rust Tenets](https://smithy-lang.github.io/smithy-rs/design/tenets.html).

### 2.4 Traits и dependency injection (тестируемость)

- Трейты как интерфейсы; функции принимают обобщённые параметры по трейту, а не конкретные типы.
- Разделение: создание объектов снаружи, использование внутри; инжекция зависимостей через конструктор.
- Статическая диспетчеризация (generics) — нулевые накладные расходы; динамическая (dyn Trait) — когда набор реализаций неизвестен на этапе компиляции или нужна подмена в рантайме (в т.ч. моки).
- Так достигается тестируемость без тяжёлых фреймворков. См. [Rust traits and dependency injection](https://jmmv.dev/2022/04/rust-traits-and-dependency-injection.html), [Testability: Reimagining OOP in Rust](https://audunhalland.github.io/blog/testability-reimagining-oop-design-patterns-in-rust/).

### 2.5 Builder pattern

- **derive_builder** — генерация билдера по структуре; проверки при build() (Result); цепочка вызовов; ~1.3M загрузок/мес.
- **typed_builder** — проверка полноты полей на этапе компиляции; порядок вызовов произвольный; default через атрибуты.
- Для конфигов и опциональных параметров билдер уменьшает шум и ошибки. См. [Effective Rust builders](https://effective-rust.com/builders.html).

### 2.6 Ошибки: thiserror и anyhow

- **thiserror** — для библиотек: свои типы ошибок с derive(Error, Display), источниками и совместимостью с std::error::Error.
- **anyhow** — для приложений: контекст (.context(), .with_context()), тип-стирание; в публичном API библиотек обычно не используют.
- В SDK: публичный API — свои типы (thiserror); внутри и в примерах допустим anyhow. См. [Comprehensive Rust: thiserror and anyhow](https://google.github.io/comprehensive-rust/error-handling/thiserror-and-anyhow.html).

### 2.7 #[non_exhaustive] и эволюция API

- Для enum: добавление новых вариантов не ломает клиентов, если они обрабатывают «остальное» через `_`.
- Для struct: внешние крейты не могут конструктор или exhaustive pattern match — можно добавлять поля в минорных версиях.
- Внутри крейта атрибут не действует. Важно для ошибок и конфигов. См. [RFC 2008 non-exhaustive](https://rust-lang.github.io/rfcs/2008-non-exhaustive.html).

### 2.8 Документация и doctests

- Примеры в `///` с тройными бэктиками компилируются и запускаются как тесты.
- Скрытие служебного кода в примерах: строки с `#` не показываются в доке, но участвуют в тесте.
- Edition 2024: объединение нескольких doctest в один бинарник для ускорения. См. [rustdoc documentation tests](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html).

### 2.9 Feature flags и опциональные зависимости

- `[features]` + `optional = true` в зависимостях; включение фичи: `crate-name/feature-name`.
- `dep:crate-name` — явная привязка фичи к опциональной зависимости.
- Для SDK: например `default = ["rustls-tls"]`, `native-tls`, `opentelemetry`, `serde`; минимальный default для встраивания. См. [The Cargo Book: features](https://doc.rust-lang.org/cargo/reference/features.html).

---

## 3. Transport и gRPC (Rust)

### 3.1 Tonic и Tower

- **Tonic** — gRPC клиент/сервер на базе Tower и Hyper.
- **Tower** — слои (middleware): retry, timeout, load balancing.
- Retry: политика (какие ошибки повторять), backoff; для Tonic нужно уметь клонировать запрос (http::Request не Clone — часто оборачивают или используют специальные политики). См. [Tower Retry](https://docs.rs/tower/latest/tower/retry/struct.Retry.html).

### 3.2 Канал и пул соединений

- **Channel** Tonic поверх Hyper: HTTP/2, встроенное переиспользование соединений.
- Для явного пула и обновления при смене DNS/эндпоинтов: **soda-pool** (Tonic 0.14), **mobc-tonic** — пул каналов с TLS и конфигом.
- В большинстве случаев достаточно одного Channel с повторным использованием. См. [tonic Channel](https://docs.rs/tonic/latest/tonic/transport/struct.Channel.html), [soda-pool](https://docs.rs/soda-pool).

### 3.3 Таймауты и Keep-Alive

- Таймауты на уровне Tower (timeout layer) или в конфиге вызовов.
- Keep-Alive на стороне Canton (LedgerApiKeepAliveServerConfig); на клиенте — настройки HTTP/2 (например в tonic::transport::Channel::from_shared с опциями).

### 3.4 TLS

- Tonic поддерживает TLS для Channel; серверная и клиентская (mutual) аутентификация через сертификаты.
- Конфигурация Canton: см. раздел 1.4.

---

## 4. Observability (Rust)

### 4.1 OpenTelemetry (traces, metrics)

- **opentelemetry** (API) + **opentelemetry_sdk** (реализация); версии ~0.31.
- **tracing** + **tracing-opentelemetry** — мост от tracing к OTLP.
- **opentelemetry-otlp** — экспорт в collector (gRPC или HTTP); feature `grpc-tonic` для Tonic.
- Метрики: Counter, Gauge, Histogram; async instruments.
- В SDK: спаны на submit/create/exercise, метрики (число команд, ошибок, задержки). См. [OpenTelemetry Rust](https://opentelemetry.io/docs/languages/rust/exporters/).

### 4.2 Рекомендации

- В продакшене предпочтителен OTLP exporter в Collector; Prometheus/Jaeger и вендоры поддерживают OTLP.
- Feature-gate для `tracing` и `opentelemetry`, чтобы не тянуть зависимости по умолчанию.

---

## 5. Конфигурация

### 5.1 12-factor

- Конфигурация в окружении; различие сред (dev/stage/prod) без изменения кода.
- Секреты и адреса не в репозитории. См. [12factor config](https://12factor.net/config).

### 5.2 Rust: config и serde

- **config** (config-rs) — слои: дефолты, файлы (JSON, TOML, YAML и т.д.), переменные окружения; вложенность.
- **serde_yaml** — десериализация YAML (проект в режиме поддержки).
- **serde-env-field** — подстановка переменных окружения при десериализации (например для секретов в YAML).
- Для SDK: один конфиг (например LedgerApiConfig + TemplateIds + опционально compliance/oracle), загрузка из файла + переопределение через env.

---

## 6. Тестирование

### 6.1 Интеграционные тесты с gRPC

- **mocktail** — мок HTTP и gRPC серверов, в т.ч. стриминг; асинхронно; матчеры запросов.
- Альтернатива: реальный участник в dev/sandbox и тесты с feature `integration` или env (например CANTON_LEDGER_GRPC).
- Юнит-тесты: все зависимости за границами крейта через трейты; моки реализуют трейты (ComplianceProvider, OracleProvider, LedgerClient при необходимости).

### 6.2 Изоляция и повторяемость

- Дедупликация команд: уникальные command_id (например UUID) в тестах, чтобы не конфликтовать с предыдущими прогонами.
- Очистка данных или изолированный участник/песочница на тест.

---

## 7. Безопасность

### 7.1 TLS и JWT (см. разделы 1.4, 1.10)

- TLS для Ledger API в продакшене; JWT с JWKS для авторизации.
- Секреты и ключи — из окружения или секрет-менеджера, не в коде.

### 7.2 Parties и act_as / read_as

- Минимальные права: только нужные партии в act_as/read_as.
- User Management Service для централизованного управления правами пользователей.

---

## 8. Institutional / RWA и compliance

### 8.1 Регуляторика (2024)

- **EU MiCA** — применима к CASP; стабильные монеты с июня 2024.
- **EU Transfer of Funds Regulation (2023/1113)** — Travel Rule, обогащение переводов данными отправителя/получателя.
- **BSA Travel Rule (US)** — для переводов ≥ $3,000; FinCEN, усиленный контроль миксеров.
- **FATF** — Travel Rule по странам реализован неравномерно; в коде учитывать обнаружение контрагента и fallback.

### 8.2 KYC/AML и ончейн-идентичность

- Интеграция через **verifiable credentials (VC)** и **zero-knowledge proofs (ZKP)**.
- Провайдеры KYC выдают VC; кошельки хранят; ZKP доказывают соответствие без раскрытия персональных данных.
- В архитектуре SDK: провайдеры (ComplianceProvider, KYC) как заменяемые трейты; вызов внешних API или заглушки в тестах.

### 8.3 Токенизация RWA

- **ERC-3643** (и аналоги) — стандарты для compliance и gating доступа.
- Инфраструктура: оракулы для офчейн-фактов, мультиподпись, автоматические проверки, расчёты в реальном времени.
- Архитектура: мост между традиционным правом и ончейн-исполнением; settlement (T+1 в US с мая 2024; T+0 — точечно, например Индия 2025). См. отчёты EBA, Chainlink education, 7BlockLabs tokenization toolchain.

### 8.4 Settlement (T+0 / T+1)

- US T+1 с 28 мая 2024; T+0 в 2024 остаётся редким.
- В DeFi/custody: автоматизация, straight-through processing, референс-данные и посттрейд-воркфлоу — основа для быстрого settlement.
- В SDK: конфиг settlement (T0/T1/T2) и оркестрация команд в соответствии с политикой участника.

---

## 9. Canton / Daml: best practices приложений

### 9.1 SDLC (Digital Asset)

- Контракты: Daml scripts, .dar, типы данных, choices, ограничения.
- Приложения: управление пакетами и партиями, работа с контрактами в Java/TypeScript, внешняя подпись, раскрытие контрактов, SQL-запросы.
- Структура проекта: App Dev Quickstart, разделение контрактов и приложений.
- Интеграция: JSON API, Daml Shell, gRPC. См. [Best practices for Canton Network application development](https://docs.digitalasset.com/build/3.4/sdlc-howtos/sdlc-best-practices.html).

### 9.2 Рекомендации для SDK

- Использовать **Command Service** (submit-and-wait) там, где допустима блокировка до completion; в высокопроизводительных сценариях — Submission + Completion stream и собственная корреляция.
- При bootstrap загружать состояние через **Active Contracts Service**, затем подписываться на **Transaction Service** с полученного offset.
- В фильтрах использовать **template_filters** / **interface_filters**, не deprecated template_ids.
- Хранить и передавать **offset** для восстановления после перезапуска.
- Генерировать **command_id** и при необходимости **submission_id** так, чтобы повторные отправки были идемпотентны (дедупликация).

---

## 10. Сводная таблица: что учитывать в SDK

| Область | Рекомендация | Источник |
|--------|---------------|----------|
| Ledger API | Асинхронная модель; разделять submit и completion | Daml Application Architecture |
| Commands | Change ID, command_id, application_id, deduplication period | Command Deduplication |
| Filters | template_filters / interface_filters вместо template_ids | Canton 2.8, Java API |
| Config | TLS, Keep-Alive, max message size, JWT/JWKS | Canton API Configuration |
| Rust API | Ошибки (thiserror), общие трейты, naming, features | Rust API Guidelines |
| Extensibility | Трейты для Ledger, Compliance, Oracle; DI через конструктор | Smithy/Azure tenets, Rust DI |
| Transport | Tonic + Tower; retry/timeout; при необходимости pool (soda-pool) | tonic, Tower |
| Observability | tracing + opentelemetry (feature-gated) | OpenTelemetry Rust |
| Config | 12-factor; YAML + env override (config-rs, serde-env-field) | 12factor, config-rs |
| Testing | Моки по трейтам; mocktail для gRPC; интеграция с env gate | mocktail, Rust DI |
| Security | TLS, JWT, минимальные act_as/read_as | Daml Authorization, Canton |
| Compliance | ComplianceProvider/OracleProvider как трейты; KYC/AML через VC/ZKP | RWA/compliance отчёты |
| Offsets | Сохранять offset после ACS; использовать для Transaction stream | Daml App Arch, HA |

---

## 11. Источники (ссылки)

- Daml Application Architecture: https://docs.daml.com/app-dev/app-arch.html  
- The Ledger API Services: https://docs.daml.com/app-dev/services.html  
- Command Deduplication: https://docs.daml.com/app-dev/command-deduplication.html  
- How Daml Types are Translated to Protobuf: https://docs.daml.com/app-dev/grpc/daml-to-ledger-api.html  
- Canton API Configuration: https://docs.daml.com/canton/usermanual/apis.html  
- Daml Authorization: https://docs.daml.com/app-dev/authorization.html  
- Parties and Users: https://docs.daml.com/app-dev/parties-users.html  
- Best practices Canton: https://docs.digitalasset.com/build/3.4/sdlc-howtos/sdlc-best-practices.html  
- Explicit contract disclosure: https://docs.digitalasset.com/build/3.3/sdlc-howtos/applications/develop/explicit-contract-disclosure.html  
- Rust API Guidelines: https://rust-lang.github.io/api-guidelines/  
- Rust API Guidelines Checklist: https://rust-lang.github.io/api-guidelines/checklist.html  
- Cargo features: https://doc.rust-lang.org/cargo/reference/features.html  
- Azure SDK Rust: https://azure.github.io/azure-sdk/rust_introduction.html  
- Smithy Rust Tenets: https://smithy-lang.github.io/smithy-rs/design/tenets.html  
- Rust traits and DI: https://jmmv.dev/2022/04/rust-traits-and-dependency-injection.html  
- Testability in Rust: https://audunhalland.github.io/blog/testability-reimagining-oop-design-patterns-in-rust/  
- thiserror/anyhow: https://google.github.io/comprehensive-rust/error-handling/thiserror-and-anyhow.html  
- RFC 2008 non_exhaustive: https://rust-lang.github.io/rfcs/2008-non-exhaustive.html  
- OpenTelemetry Rust: https://opentelemetry.io/docs/languages/rust/exporters/  
- 12-Factor config: https://12factor.net/config  
- Tower Retry: https://docs.rs/tower/latest/tower/retry/  
- tonic Channel: https://docs.rs/tonic/latest/tonic/transport/struct.Channel.html  
- mocktail: https://docs.rs/mocktail  
- Daml HA: https://docs.daml.com/deploy-daml/infrastructure-architecture/high-availability/ha-and-scaling/implementing-ha.html  

---

---

## 11A. Rust API Guidelines — расширенный чеклист

- **Naming (C-CASE):** Типы UpperCamelCase, функции/методы snake_case, константы SCREAMING_SNAKE_CASE; конверсии as_/to_/into_.
- **Interoperability (C-GOOD-ERR):** Ошибки реализуют std::error::Error, Display; источник через .source(); совместимость с anyhow (From).
- **Interoperability (C-COMMON-TRAITS):** Clone, PartialEq, Eq, Hash, Debug, Default где применимо; не забывать Serialize/Deserialize для конфигов и API.
- **Cargo (C-FEATURE):** Опциональные зависимости через features; default минимален; не более 300 features на crate (crates.io).
- **Documentation (C-DOC):** Публичные элементы документированы; примеры в /// где уместно; ссылки на внешние ресурсы.
- **Predictability (C-WORD-ORDER):** Параметры в порядке: self, зарезервированные (например command_id), входные данные, опции (Option, impl Into).
- **Flexibility (C-CALLER-CONTROL):** Избегать скрытых аллокаций и блокировок; давать вызывающему выбор буферов, таймаутов, стратегий retry где возможно.
- **Safety (C-SECURE):** Не экспонировать небезопасный код без необходимости; секреты не логировать, не включать в ошибки.

Источник: [Rust API Guidelines Checklist](https://rust-lang.github.io/api-guidelines/checklist.html).

---

## 11B. Institutional RWA — технический стек (кратко)

- **Стандарты токенов:** ERC-3643, ERC-1400 и аналоги для permissioned/eligibility gating.
- **KYC/KYB:** API-first провайдеры (например Trulioo), версионируемые workflow по юрисдикциям.
- **Travel Rule:** Обогащение переводов originator/beneficiary; обнаружение контрагента и fallback при отсутствии поддержки.
- **Governance:** EBA guidelines по ART issuers — риск-менеджмент, compliance, внутренний аудит, BCP, прозрачность.
- **Settlement:** Автоматизация и STP для T+1; в DeFi/custody — смарт-контракты и мониторинг коллатерала 24/7.
- Для SDK: интерфейсы ComplianceProvider и OracleProvider должны допускать подключение внешних KYC/AML и оракулов без жёсткой привязки к одному провайдеру.

Источники: 7BlockLabs tokenization toolchain, EBA guidelines on ART issuers, Chainlink RWA education, IOSCO/FATF отчёты.

---

## 12. Дополнительные аспекты для SDK

### 12.1 Async runtime (Tokio vs async-std)

- **Tokio** — мультипоточный work-stealing планировщик, большая экосистема (hyper, tonic, tower); рекомендуется для продакшена и gRPC.
- **async-std** — проще, меньше зависимостей; для SDK с tonic де-факто стандарт — Tokio (tonic опирается на Tower/Hyper).
- Рекомендация для SDK: явно зависеть от Tokio (или опционально `rt-tokio` / `rt-tokio-current-thread` в opentelemetry_sdk); не смешивать рантаймы в одном крейте.

### 12.2 Streams (стримы Ledger API)

- **Transaction Service** и **Active Contracts Service** возвращают gRPC streams.
- В Rust: `futures::Stream` / `StreamExt` (next(), then()); для tonic — типы из сгенерированного proto (streaming RPC).
- Обработка: `while let Some(msg) = stream.next().await`; учёт offset в последнем сообщении ACS для последующей подписки на Transaction Service.
- См. [Tokio Streams](https://tokio.rs/tokio/tutorial/streams), [futures::Stream](https://docs.rs/futures/latest/futures/stream/trait.Stream.html).

### 12.3 Версии зависимостей (ориентир 2024)

- **tonic** 0.11–0.14, **tower** 0.4–0.5, **prost** 0.12 — для Ledger API proto.
- **tokio** 1.32+ (или по совместимости с tonic).
- **serde**, **serde_yaml** — для конфига; **config** (config-rs) — для слоёв конфигурации.
- **thiserror** 2.x, **anyhow** 1.x — для ошибок.
- **opentelemetry**, **opentelemetry_sdk** 0.31, **tracing-opentelemetry** 0.22–0.32 — для observability (опционально).
- **chrono** + **rust_decimal** — для дат и денежных сумм в доменных типах.

### 12.4 Naming (Rust API Guidelines, RFC 430)

- Типы, трейты, варианты enum: **UpperCamelCase** (в т.ч. Uuid, Stdin — акронимы как одно слово).
- Функции, методы, модули, переменные: **snake_case**.
- Константы, статики: **SCREAMING_SNAKE_CASE**.
- Конструкторы: `new`, `with_*` для дополнительных параметров; конверсии: `from_*`, `as_*` (дешёвые), `to_*` (дорогие), `into_*` (consuming).
- Имена крейтов без суффиксов `-rs` / `-rust`.

### 12.5 Доменные типы и общие трейты

- Для всех публичных типов (TreasuryBill, PurchaseRequest, LedgerApiConfig и т.д.): `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]` где уместно; `Default` и/или `new()` по необходимости.
- Для enum статусов и кодов: `#[non_exhaustive]` если планируется добавлять варианты в минорных версиях.
- Ошибки: `#[derive(thiserror::Error)]`, реализация `Display`, источник через `#[source]`; варианты с контекстом (например `SubmitFailed { command_id, reason }`).

### 12.6 Конфиг: слои и приоритет

- Рекомендуемый порядок (от низкого к высокому приоритету): 1) дефолты в коде, 2) конфиг-файл (YAML/TOML), 3) переменные окружения (например `CANTON_LEDGER_GRPC`, `CANTON_PARTY_ID`).
- Секреты (токены, ключи) — только из env или секрет-менеджера, не в файле в репозитории.
- В документе и примерах явно указать: какие переменные обязательны, какие опциональны и их дефолты.

### 12.7 Интеграционные тесты: feature и env

- Тесты, требующие реального участника: за guarded feature `integration` или проверкой env (например `CANTON_LEDGER_GRPC`). Если не задано — тест пропускать (`#[ignore]` или условный skip).
- Это позволяет CI без участника проходить по юнит-тестам; интеграционные запускать вручную или в отдельном пайплайне с участником/sandbox.

### 12.8 Command ID и идемпотентность

- В каждом submit использовать уникальный **command_id** (например UUID v4); при retry после таймаута — тот же command_id в пределах deduplication period, чтобы участник не исполнил команду дважды.
- **application_id** — константа для приложения (например `canton-defi-sdk`); **workflow_id** — опционально для группировки транзакций в один бизнес-процесс.

### 12.9 Версионирование Ledger API и proto

- Canton/Daml могут обновлять proto (новые поля, новые сервисы). В SDK: зафиксировать версию proto в репозитории; при обновлении Canton проверять совместимость и при необходимости обновлять конвертеры (canton_core::Commands → proto).
- Version Service Ledger API позволяет узнать версию участника и поддерживаемые фичи (feature descriptor) — использовать при отладке и в документации требований к участнику.

### 12.10 Документация для потребителей SDK

- В каждом публичном модуле/крейте: краткое описание назначения (в т.ч. в lib.rs).
- Для основных типов и методов: `///` с описанием, пример использования в `# Example` (doctest) где возможно.
- Отдельный документ (например DEFI_SDK_USAGE.md): подключение к DevNet, загрузка конфига, пример вызова TreasuryService (create_bill, list_bills, get_portfolio_summary), список env и опция конфига.
- Ссылки на официальные Daml/Canton документы (Ledger API, Authorization, Command Deduplication) в разделах «См. также».

---

## 13. Чеклист для слияния с мастер-промтом

При объединении этого документа с DEFI_SDK_MASTER_PROMPT.md учесть:

- [ ] **Async-модель:** В промте явно указать, что submit не ждёт execution; при необходимости submit-and-wait — Command Service или Completion stream + корреляция по command_id.
- [ ] **Change ID / deduplication:** В Chain 2 и в типах Commands описать поля application_id, command_id, submission_id, deduplication_period и их использование при retry.
- [ ] **Фильтры:** В спецификации GetActiveContracts (и Transaction filter) использовать template_filters/interface_filters, не deprecated template_ids; при необходимости указать минимальную версию Canton (2.8+).
- [ ] **Конфиг:** В Part 6 (Config) и example.yaml добавить опции: keep_alive, max_inbound_message_size, tls, jwt/jwks; приоритет: файл → env.
- [ ] **Ошибки:** В canton-core/error.rs и в промте перечислить варианты (ConfigLoadFailed, ConnectFailed, ConversionError, SubmitFailed, ContractNotFound, ValidationFailed, BusinessRuleViolation) с контекстом; рекомендация thiserror.
- [ ] **Трейты:** ComplianceProvider, OracleProvider, при необходимости LedgerClient как трейт для тестов; инжекция через конструктор сервисов (TreasuryService::new(..., compliance, oracle)).
- [ ] **Transport:** В Chain 1 упомянуть retry/timeout (Tower); при необходимости connection pool (soda-pool) как опция для высокой нагрузки.
- [ ] **Observability:** Опциональная фича `tracing`/`opentelemetry`; спаны на submit/create/exercise; метрики (count, latency).
- [ ] **Тесты:** Юнит — моки по трейтам; интеграция — feature `integration` или env CANTON_LEDGER_GRPC; при необходимости mocktail для gRPC без участника.
- [ ] **Документация:** В Chain 6 и в критериях приёмки требовать пример в DEFI_SDK_USAGE.md и doctest для основных типов где возможно.
- [ ] **Версии:** В Part 39 (Dependency versions) согласовать с разделом 12.3 этого документа.
- [ ] **Offsets:** В сценариях восстановления и в описании Active Contracts Service указать сохранение offset и продолжение Transaction stream с него.

---

## 14. Ledger API proto: детальная структура команд и значений

### 14.1 Commands (SubmitRequest)

Сообщение `Commands` (в SubmitRequest) содержит:

- **ledger_id** (optional) — идентификатор ledger; при несовпадении команда отклоняется.
- **application_id** — идентификатор приложения; используется в change ID и для подписки на completions.
- **command_id** — уникальный идентификатор намерения; задаётся приложением; ключ дедупликации.
- **party** — партия от имени которой подаётся команда (если одна).
- **act_as** — список партий от имени которых действует приложение (вместе с party образуют submitting parties).
- **read_as** — список партий от имени которых приложение читает (для видимости контрактов).
- **commands** — массив Command (Create | Exercise | CreateAndExercise | ExerciseByKey).
- **min_ledger_time_abs** (optional) — минимальное ledger time для исполнения.
- **min_ledger_time_rel** (optional) — минимальное относительное время.
- **deduplication_period** (optional) — период дедупликации (duration или deduplication_offset); если не задано — максимальный по конфигу участника.
- **submission_id** (optional) — идентификатор этой конкретной отправки; возвращается в completion для корреляции; не переиспользовать.
- **disclosed_contracts** (optional, Canton 2.7+) — контракты, раскрытые третьими сторонами, для проверки при submission.
- **domain_id** (optional) — домен для submission (Canton).

Источник: command_submission_service.proto, SubmitRequest/Commands в Ledger API Reference.

### 14.2 CreateCommand (proto)

- **template_id** — Identifier (package_id, module_name, entity_name).
- **create_argument** — Value (обычно Record с полями шаблона).

В SDK: canton_core::CreateCommand с template_id: Identifier и create_arguments: DamlRecord; конвертер в proto Create с create_argument = daml_record_to_proto_record(create_arguments).

### 14.3 ExerciseCommand (proto)

- **contract_id** — строка или структура contract id.
- **choice** — имя choice (строка).
- **choice_argument** — Value (аргумент choice, часто Record).

В SDK: canton_core::ExerciseCommand с contract_id, choice, choice_argument: DamlValue; конвертер в proto Exercise.

### 14.4 Value (protobuf) — типы

- **record** — Record (record_id, fields).
- **variant** — Variant (constructor, value).
- **list** — List (elements: repeated Value).
- **optional** — Optional (value optional).
- **map** — Map (entries).
- **int64**, **numeric**, **text**, **party**, **timestamp**, **date**, **bool**, **contract_id**, **unit** — примитивы.

Для конвертера DamlValue → proto Value нужна рекурсивная обработка Record, Variant, List, Optional и примитивов. Decimal в Daml передаётся как строка в numeric или специальное поле в зависимости от версии proto.

### 14.5 Record (protobuf)

- **record_id** — Identifier (package_id, module_name, entity_name) для типа записи.
- **fields** — repeated RecordField; каждый RecordField: **label** (имя поля), **value** (Value).

В SDK: DamlRecord с record_id: Option<Identifier> и fields: Vec<RecordField>; RecordField { label: String, value: DamlValue }.

Источник: How Daml Types are Translated to Protobuf; Ledger API proto value.proto.

---

## 15. Rust async: Send/Sync и async traits

### 15.1 Проблема Send bound (2024)

- Async trait methods возвращают future; в обобщённом коде часто нужно требовать, чтобы этот future был Send (для работы в многопоточном рантайме, например Tokio work-stealing).
- До RFC 3654 (Return Type Notation) нельзя было выразить в трейте условие вида «future, возвращаемый методом, реализует Send».
- Следствие: обёртки (middleware, layers), которые должны работать и с Send, и с !Send реализациями, упираются в overlapping implementations или дублирование трейтов.

### 15.2 Return Type Notation (RTN)

- RFC 3654: синтаксис вида `T: Trait<method(..): Send>` или `where T::method(..): Send` — требование, что future, возвращаемый async method, реализует Send.
- Упрощает написание generic async сервисов без дублирования трейтов (Send vs !Send).
- Ограничение: на 2024 RTN применим к методам с только lifetime parameters (не type parameters).

### 15.3 Рекомендации для SDK

- Публичные async методы LedgerClient и доменных сервисов по возможности возвращать future: Send (чтобы клиент мог использовать в Tokio многопоточном).
- Зависимости (ComplianceProvider, OracleProvider): если методы async — в документации указать, что для многопоточного рантайма реализации должны возвращать Send future; при необходимости описать RTN или workaround (например отдельный трейт AsyncSendComplianceProvider).
- Внутренние вызовы tonic возвращают Send future по умолчанию (tonic + Tokio).

Источник: Design meeting 2024 Send bound; RFC 3654; async trait send bounds (smallcultfollowing).

---

## 16. config-rs: слои, merge, env

### 16.1 ConfigBuilder

- **Config::builder()** — начало цепочки.
- **add_source(File::with_name(path))** — добавление файла (формат по расширению или явно); несколько вызовов — несколько слоёв.
- **add_source(Environment::with_prefix("CANTON").separator("__"))** — переменные окружения с префиксом CANTON и разделителем __ (например CANTON__LEDGER_API__GRPC_HOST).
- Порядок добавления задаёт приоритет: позже добавленный источник переопределяет предыдущий при merge (по умолчанию later override).

### 16.2 Форматы

- Поддерживаются через опциональные зависимости: JSON, TOML, YAML, INI, RON, JSON5.
- Для SDK типично: YAML или TOML для файла; env с префиксом CANTON или CANTON_LEDGER.

### 16.3 Извлечение значений

- **config.get_string("key")**, **get_int**, **get_bool** и т.д. для плоских ключей.
- **config.get::<T>("key")** — десериализация в тип T (требует Deserialize); вложенные ключи через точечную нотацию (например "ledger_api.grpc_host").

### 16.4 Рекомендация для SDK

- Один конфиг-файл (например config/example.yaml) с секциями ledger_api, template_ids, compliance, oracle.
- ConfigBuilder: дефолты (в коде или default.yaml) + файл из пути (например env CONFIG_PATH) + Environment::with_prefix("CANTON").
- Финальная структура десериализовать в LedgerApiConfig, TemplateIds и т.д.; секреты (auth_token) не хранить в файле в репо — только env.

Источник: config crate docs.rs; 12-factor config.

---

## 17. gRPC streaming в tonic (клиент)

### 17.1 Streaming<T>

- Ответ streaming RPC в tonic имеет тип **Streaming<T>** (T — тип сообщения из proto).
- **stream.message().await** — получение следующего сообщения: **Ok(Some(T))**, **Ok(None)** (конец стрима), **Err** (gRPC ошибка).
- **stream.trailers().await** — метаданные в конце стрима (после потребления сообщений).

### 17.2 Обработка в цикле

```rust
while let Some(msg) = stream.message().await? {
    // обработка msg
}
// stream.trailers().await? при необходимости
```

### 17.3 Active Contracts Service и Transaction Service

- GetActiveContracts возвращает stream GetActiveContractsResponse; в последнем сообщении — offset.
- GetTransactions возвращает stream GetTransactionsResponse; клиент подписывается с начального offset и обрабатывает транзакции по мере поступления.
- В SDK: обёртки над сгенерированным tonic клиентом, возвращающие futures::Stream или async_stream::stream для удобства (map, filter, take_until offset).

Источник: tonic Streaming; Ledger API Transaction Service, Active Contracts Service.

---

## 18. TransactionFilter и filtersByParty (полная структура)

### 18.1 TransactionFilter

- **filter_by_party** (или filtersByParty в JSON) — map: Party → Filters.
- Для каждой партии указывается объект **Filters**; если партия не в map или Filters пустой — по умолчанию эта партия не получает события (в некоторых реализациях нужен явный пустой Filters для «все шаблоны»).

### 18.2 Filters (InclusiveFilter)

- **inclusive** — InclusiveFilter: какие шаблоны/интерфейсы включать.
- **template_ids** (deprecated) — список Identifier; использовать **template_filters** (Map Identifier → Filter.Template).
- **interface_filters** — Map Identifier → Filter.Interface (include_interface_view, include_created_event_blob и т.д.).
- В Canton 2.8+ предпочтительно template_filters/interface_filters вместо template_ids.

### 18.3 Пример (концептуально)

Для партии Alice и шаблона InstitutionalAsset:InstitutionalAsset:

- filtersByParty: { "Alice": { inclusive: { template_filters: { "InstitutionalAsset:InstitutionalAsset": {} } } } }.
- Или через ofTemplateIds(Set([identifier])) в Java; в proto — соответствующее заполнение Filter.Inclusive.

### 18.4 Для SDK

- Тип TransactionFilter (или аналог) при вызове GetActiveContracts/GetTransactions заполнять template_filters по конфигу template_ids (Identifier для InstitutionalAsset, AssetHolding, AssetPurchaseRequest).
- Не полагаться на deprecated template_ids в новых участниках.

Источник: TransactionFilter Node/Java bindings; Canton domain TransactionFilter.

---

## 19. ERC-3643 и compliance (детально)

### 19.1 Компоненты

- **Identity Registry** — реестр идентичностей; привязка адресов к прошедшим KYC; хранение статуса (eligible, suspended и т.д.).
- **Compliance Contract** — модуль правил (transfer rules); вызов canTransfer(from, to, amount) перед переводом; интеграция с Identity Registry.
- **Transfer functions** — transferred(), created(), destroyed() вызываются при переводах/создании/уничтожении токенов и обновляют состояние compliance.

### 19.2 Правила

- Только прошедшие KYC могут держать токены и участвовать в переводах.
- Ограничения по юрисдикции, типу инвестора (retail/accredited/institutional), лимитам.
- On-chain валидация перед каждым переводом; при неудаче — revert.

### 19.3 Для SDK (Canton DeFi)

- Canton не использует ERC-3643 напрямую; эквивалент — проверки в Daml (authorization, preconditions в choices) плюс оффледжерная проверка (ComplianceProvider) перед созданием запроса на покупку.
- Интерфейс ComplianceProvider в SDK должен допускать: проверку инвестора (identity/eligibility), проверку суммы и типа актива, опционально кошелек; возврат compliant: bool и reasons: Vec<String>.
- Это позволяет подключать как простые моки, так и адаптеры к KYC/AML провайдерам или к ончейн Identity Registry (если в будущем будет мост).

Источник: ERC-3643 EIP; T-REX docs; erc3643.org compliance management.

---

## 20. Canton: порты и конфигурация узла

### 20.1 Типичные порты (participant)

- **Ledger API (gRPC)** — по умолчанию 5001 (или 4001 для первого экземпляра); в наших деплоях часто 30501 (NodePort).
- **Admin API** — 5002 (или 4002).
- **HTTP JSON API** — отдельный порт (например 7575 в старых конфигах; в Canton 3.x может быть объединён или отдельно).

### 20.2 Конфиг участника (фрагмент)

- **participant.ledger-api.address** — bind address (например 0.0.0.0).
- **participant.ledger-api.port** — gRPC port.
- **participant.ledger-api.tls** — включение TLS.
- **participant.ledger-api.max-inbound-message-size** — лимит размера сообщения.
- JWT/JWKS настраиваются на уровне participant или через Identity Provider Config.

### 20.3 Для SDK

- В документации и example.yaml указать: grpc_host, grpc_port (или единый endpoint); при использовании NodePort или ingress — итоговый host:port.
- Опция max_inbound_message_size в конфиге клиента (если tonic поддерживает) для больших GetActiveContracts ответов.

Источник: Canton API Configuration; gRPC Ledger API Configuration Digital Asset.

---

## 21. OpenTelemetry: спаны и метрики (примеры)

### 21.1 Трассировка

- **tracer.span("ledger.submit").start()** — создание спана; атрибуты: command_id, application_id, template_id (для create).
- Внутри span: вызов tonic submit; при ошибке — record_exception или set_status(Error).
- Для submit_and_wait: родительский span "ledger.submit_and_wait", дочерние "ledger.submit" и "ledger.completion_wait".

### 21.2 Метрики

- **Counter**: ledger_commands_submitted_total (labels: application_id, template/choice).
- **Counter**: ledger_commands_failed_total (labels: application_id, reason).
- **Histogram**: ledger_submit_duration_seconds (для времени от submit до completion при submit_and_wait).
- **Histogram**: ledger_active_contracts_fetch_duration_seconds.

### 21.3 Feature-gate

- Опциональная зависимость tracing, opentelemetry, opentelemetry_sdk, tracing-opentelemetry; feature "observability" или "tracing".
- В коде: #[cfg(feature = "observability")] оборачивать вызовы span/metrics; без фичи — no-op.

Источник: OpenTelemetry Rust; uptrace OpenTelemetry Rust metrics.

---

## 22. Rust: Result/Option и комбинаторы в API

### 22.1 Рекомендации

- Возвращать **Result<T, E>** из функций, которые могут отказать (сеть, парсинг, бизнес-ошибки); E — тип из thiserror.
- Использовать **Option<T>** для «может отсутствовать» (например get_bill возвращает Option<TreasuryBill>).
- В цепочках: **map**, **and_then**, **map_err**, **?** для распространения ошибок; избегать unwrap() в библиотечном коде.

### 22.2 Конвертеры

- **from_daml_record(record) -> Result<TreasuryBill, ConversionError>** — при ошибке поля возвращать ConversionError с контекстом (имя поля, ожидаемый тип).
- **to_daml_record(bill) -> DamlRecord** — для известных типов без fallible логики можно возвращать DamlRecord напрямую; при сложной валидации — Result.

Источник: Rust By Example; Rust error handling best practices.

---

## 23. Детальная экспертиза: Daml Ledger API — чтение состояния

### 23.1 StateService / GetLedgerEnd

- В Ledger API v2 участник может экспонировать **StateService** с методом **GetLedgerEnd** (или аналог) для получения текущего offset ledger без чтения транзакций.
- Используется для: проверки подключения; определения начальной точки подписки на Transaction Service после bootstrap через Active Contracts.
- В некоторых версиях Canton offset доступен только как часть ответа Active Contracts (последнее сообщение в стриме) или через Transaction Service (начальный offset запроса).
- Для SDK: в документации указать, как получить «текущий offset» в используемой версии Canton (StateService если есть, иначе — через ACS или первый запрос к Transaction Service).

### 23.2 Active Contracts Service — поток ответа

- **GetActiveContractsRequest**: ledger_id (optional), filter (TransactionFilter), verbose (optional).
- Ответ: **stream GetActiveContractsResponse**; каждое сообщение — batch created_events (активные контракты); **последнее** сообщение содержит **offset** (поле для продолжения).
- Размер batch определяется участником; клиент должен итерировать до конца стрима, чтобы получить offset.
- Verbose: true включает record_id, labels полей в Value — полезно для отладки и маппинга в доменные типы.

### 23.3 Transaction Service — подписка с offset

- **GetTransactionsRequest**: ledger_id, begin (offset), filter (TransactionFilter), verbose (optional).
- Ответ: **stream GetTransactionsResponse**; каждое сообщение — транзакция (transaction_id, workflow_id, command_id, events).
- События: created, exercised, archived; по ним приложение обновляет локальное состояние (например кэш контрактов).
- При перезапуске: начать с сохранённого offset (последний из ACS или последний обработанный из Transaction stream).

### 23.4 Command Completion Service — корреляция submit и result

- **SubscribeCompletionRequest**: ledger_id, application_id, parties (submitting parties), offset (optional — с какого момента completions).
- Ответ: **stream CompletionStreamResponse**; каждое сообщение содержит completions с command_id, status (success | failure), submission_id.
- Приложение сохраняет command_id при submit и ждёт в стриме completion с тем же command_id; по status определяет успех или причину отказа.
- Для submit_and_wait: после submit подписаться на completion (или использовать Command Service если доступен) и ждать нужный command_id с таймаутом.

### 23.5 Event Query Service (опционально)

- Запросы по **contract_id** или **contract_key**; возврат событий (created, archived) для контракта; для ключа — поддержка пагинации (continuation_token).
- Используется для точечного запроса без подписки на полный Transaction stream; в SDK можно добавить как опциональный слой поверх LedgerClient.

Источник: The Ledger API Services; Ledger API Reference proto-docs.

---

## 24. Детальная экспертиза: идентификаторы и пакеты

### 24.1 Identifier (template/interface)

- **package_id** — хэш пакета Daml (уникальный идентификатор загруженного пакета на участнике).
- **module_name** — имя модуля Daml (строка).
- **entity_name** — имя шаблона или интерфейса (строка).
- Полный идентификатор шаблона в приложениях часто задаётся строкой вида "ModuleName:EntityName"; package_id подставляется с участника (Package Service) или из конфига, если известен для конкретного окружения (например DevNet).

### 24.2 Package Service

- **ListPackageIds** — список загруженных package_id.
- **GetPackage(package_id)** — содержимое пакета (Daml-LF archive).
- Для маппинга "InstitutionalAsset:InstitutionalAsset" в Identifier: module_name = "InstitutionalAsset", entity_name = "InstitutionalAsset"; package_id — получить через ListPackageIds и фильтр по имени пакета или хранить в конфиге для известного окружения.

### 24.3 Version Service

- **GetLedgerApiVersion** (или аналог) — версия Ledger API участника.
- **Feature descriptor** — перечень поддерживаемых опциональных фич (например User Management, Identity Provider Config).
- В SDK: при подключении опционально вызывать Version Service и логировать версию; в документации указать минимальную поддерживаемую версию Canton/Ledger API.

Источник: Ledger API Reference; Package Service, Version Service proto.

---

## 25. Детальная экспертиза: конфигурация Canton (сервер)

### 25.1 Ledger API server (participant)

- **address** — bind address (0.0.0.0 для доступа снаружи).
- **port** / **internalPort** — порт gRPC.
- **tls** — серверный TLS; сертификаты и ключи.
- **maxInboundMessageSize** — лимит размера входящего сообщения (gRPC).
- **keepAlive** — KeepAliveServerConfig (time, timeout) для поддержания соединений.

### 25.2 JWT / JWKS (participant)

- **issuer** — ожидаемый issuer claim в JWT.
- **audience** (optional) — ожидаемый audience.
- **jwksUrl** или **staticJwks** — источник ключей для проверки подписи JWT.
- **leeway** — допустимое расхождение по времени (секунды) при проверке exp/iat.

### 25.3 Для клиента SDK

- Соответствующие параметры: **grpc_endpoint** (host:port), **tls** (включить/отключить, путь к CA при необходимости), **auth_token** (JWT строка) или механизм получения токена (OAuth2 client credentials и т.д.).
- **max_inbound_message_size** — если tonic поддерживает настройку клиента, задавать для больших ответов ACS.
- **keep_alive** на клиенте — настройки HTTP/2 (idle timeout, interval) для стабильности длинных стримов.

Источник: Canton API Configuration; Daml Authorization.

---

## 26. Детальная экспертиза: Rust — семантика версий и обратная совместимость

### 26.1 SemVer для библиотек

- **MAJOR** — несовместимые изменения публичного API (удаление методов, смена сигнатур, смена поведения).
- **MINOR** — новая функциональность с сохранением обратной совместимости (новые методы, новые варианты enum с #[non_exhaustive], новые поля в struct с default).
- **PATCH** — исправления без изменения публичного API.

### 26.2 Допустимые изменения в MINOR

- Добавление новых методов в трейт или impl (если трейт не sealed).
- Добавление новых вариантов в enum с **#[non_exhaustive]** (клиенты с wildcard `_` не сломаются).
- Добавление полей в struct с **#[non_exhaustive]** и/или **Default** (внешние крейты не делают exhaustive construct).
- Добавление реализаций трейтов для существующих типов.
- Deprecation старых методов с перенаправлением на новые (без удаления в MINOR).

### 26.3 Недопустимые без MAJOR

- Удаление или переименование публичных элементов.
- Смена типа возвращаемого значения или типов аргументов.
- Смена семантики (например метод раньше не блокировал, а теперь блокирует).
- Добавление обязательных полей в struct без default и без #[non_exhaustive] (внешний код, создающий struct, сломается).

### 26.4 Для SDK

- Публичный API canton-core и canton-ledger-api придерживаться SemVer; при добавлении полей в конфиг или доменные типы использовать Option или #[non_exhaustive] и документировать в CHANGELOG.
- Daml/Canton proto могут обновляться отдельно; версию proto зафиксировать в репозитории и при обновлении проверять совместимость (новые поля в proto обычно optional).

Источник: RFC 1105 API evolution; Cargo book SemVer; Rust API Guidelines.

---

## 27. Детальная экспертиза: тестирование — стратегии

### 27.1 Юнит-тесты

- Все зависимости за границами крейта — через трейты (LedgerClient, ComplianceProvider, OracleProvider).
- Моки: структуры, реализующие трейты с фиксированными ответами (например MockComplianceProvider всегда возвращает compliant: true).
- Конвертеры (domain ↔ proto, DamlValue ↔ Value): тесты на известных примерах (минимальный Record, вложенный Record, List, Optional) и roundtrip где применимо.
- Не вызывать сеть и не зависеть от участника.

### 27.2 Интеграционные тесты

- **Feature gate**: например `#[cfg(feature = "integration")]` или проверка env `CANTON_LEDGER_GRPC`; при отсутствии — тест помечен `#[ignore]` или пропуск в runtime.
- Сценарии: connect → get_ledger_end (или аналог); submit minimal Create → ожидание completion или проверка через ACS; полный цикл Treasury (create_bill → create_purchase_request → approve → get_holdings).
- Изоляция: уникальные command_id (UUID) и при необходимости отдельный application_id для тестов; очистка или изолированный участник/песочница.

### 27.3 Мок gRPC сервера

- **mocktail** — поддержка gRPC и стриминга; задание ожиданий (when/then) и проверка вызовов.
- Альтернатива: поднять реальный Canton Sandbox в CI (Docker) и прогонять интеграционные тесты против него.
- Для быстрых CI без участника: юнит + моки; интеграционные — отдельный job или ручной запуск.

Источник: mocktail; Rust testing best practices; Canton Sandbox.

---

## 28. Детальная экспертиза: безопасность — секреты и логирование

### 28.1 Секреты

- **Auth token (JWT)** — не логировать, не включать в сообщения об ошибках (Display/Debug для ошибок подключения).
- Загрузка из env (CANTON_AUTH_TOKEN) или из секрет-менеджера (Vault, Kubernetes Secret); не хранить в конфиг-файле в репозитории.
- В конфиге допускать placeholder (например "env:CANTON_AUTH_TOKEN") и разрешать его при загрузке через serde-env-field или явную подстановку.

### 28.2 Логирование

- Не логировать полные payload команд (могут содержать конфиденциальные данные); при отладке — только template_id, command_id, application_id.
- Ошибки: логировать тип ошибки и контекст (command_id, template_id), но не сырые ответы участника с потенциально чувствительными данными.
- Уровни: trace для деталей (например каждый шаг конвертации в dev), info для операций (submit, connect), warn для retry, error для финальных отказов.

### 28.3 TLS

- В продакшене использовать TLS для Ledger API; проверка серверного сертификата (не отключать верификацию в проде).
- Клиентские сертификаты (mutual TLS) — если участник требует; конфиг путей к сертификату и ключу из env или секрет-менеджера.

Источник: 12-factor; OWASP logging; Canton TLS configuration.

---

## 29. Детальная экспертиза: RWA — оракулы и цены

### 29.1 Роль оракулов в DeFi/RWA

- **Цены активов** — для оценки портфеля, маржи, ликвидаций; источники: Pyth, Chainlink, внутренние фиды.
- **Процентные ставки / yield** — для казначейских облигаций и долговых инструментов (например treasury yield по срокам 1M, 3M, 1Y).
- **Оценка недвижимости** — для залога и листинга (property valuation API); данные от провайдеров (CoStar, внутренние модели).

### 29.2 Для SDK

- **OracleProvider** — трейт с методами get_price(symbol), get_prices(symbols), get_treasury_yield(maturity), get_all_treasury_yields(); опционально get_property_valuation(property_id).
- Реализации: **MockOracleProvider** (фиксированные значения для тестов); **HttpOracleProvider** (вызов внутреннего API или Pyth/Chainlink HTTP); кэширование на уровне провайдера с TTL из конфига.
- TreasuryService при create_bill может запрашивать current_yield/expected_yield из OracleProvider по maturity; RealEstateService при листинге — оценку из get_property_valuation.

Источник: Chainlink; Pyth; property valuation APIs; DeFi oracle patterns.

---

## 30. Детальная экспертиза: Daml — типы и контракты (напоминание для маппинга)

### 30.1 Типы Daml → Proto Value

- **Int** → int64.
- **Decimal** → numeric (строка или специальное сообщение в зависимости от версии proto).
- **Text** → text.
- **Bool** → bool.
- **Party** → party.
- **Time** → timestamp.
- **Date** → date.
- **ContractId** → contract_id.
- **Optional T** → optional { value }.
- **List T** → list { elements }.
- **Record** → record { record_id, fields }.
- **Variant** → variant { constructor, value }.

### 30.2 Контракты в событиях

- **CreatedEvent** содержит contract_id, template_id, create_arguments (Value), contract_key (optional).
- **ArchivedEvent** содержит contract_id, template_id.
- **ExercisedEvent** содержит contract_id, template_id, choice, choice_argument, consuming, contract_key (optional), exercise_result (optional).

При маппинге в доменные типы (TreasuryBill, TreasuryBillHolding) извлекать поля из create_arguments (Record) по именам полей; при несовпадении типов или отсутствии поля — возвращать ConversionError с контекстом.

Источник: How Daml Types are Translated to Protobuf; Ledger API event.proto.

---

## 31. Сводные таблицы для слияния с мастер-промтом

### 31.1 Поля Commands (proto / canton_core)

| Proto / API | canton_core / SDK | Обязательность |
|-------------|-------------------|-----------------|
| ledger_id | ledger_id: Option<String> | опционально |
| application_id | application_id: String | обязательно |
| command_id | command_id: String | обязательно |
| party | party: Option<String> | одна из party/act_as |
| act_as | act_as: Vec<String> | обязательно (или party) |
| read_as | read_as: Vec<String> | опционально |
| commands | commands: Vec<Command> | обязательно |
| deduplication_period | deduplication_period: Option<DedupPeriod> | опционально |
| submission_id | submission_id: Option<String> | опционально |
| disclosed_contracts | disclosed_contracts: Option<Vec<DisclosedContract>> | опционально (Canton 2.7+) |

### 31.2 Поля CreateCommand / ExerciseCommand

| Команда | Поле proto | SDK тип |
|---------|------------|---------|
| Create | template_id | Identifier |
| Create | create_argument | DamlRecord → proto Record |
| Exercise | contract_id | ContractId / String |
| Exercise | choice | String |
| Exercise | choice_argument | DamlValue → proto Value |

### 31.3 Конфиг Ledger API (клиент SDK)

| Ключ конфига / env | Тип | Описание |
|--------------------|-----|----------|
| grpc_host | string | Хост участника |
| grpc_port | u16 | Порт gRPC |
| http_url | string (optional) | URL HTTP API если используется |
| tls | bool (optional) | Включить TLS |
| connect_timeout_secs | u64 (optional) | Таймаут подключения |
| request_timeout_secs | u64 (optional) | Таймаут запроса |
| ledger_id | string (optional) | Идентификатор ledger |
| auth_token | string (optional) | JWT; предпочтительно из env |
| max_inbound_message_size | u32 (optional) | Лимит размера сообщения |
| template_ids.* | string | Полные имена шаблонов (InstitutionalAsset:InstitutionalAsset и т.д.) |

### 31.4 Ошибки SDK (рекомендуемые варианты)

| Вариант | Когда использовать |
|---------|---------------------|
| ConfigLoadFailed | Не удалось загрузить или распарсить конфиг |
| ConnectFailed | Ошибка подключения к участнику (таймаут, unreachable) |
| ConversionError | Ошибка конвертации domain ↔ proto или парсинга Value |
| SubmitFailed | Участник отклонил команду (причина в контексте) |
| ContractNotFound | Контракт по id не найден (get_bill, get_holdings и т.д.) |
| ValidationFailed | Compliance проверка не пройдена (compliant: false) |
| BusinessRuleViolation | Нарушение бизнес-правила (min/max amount, supply и т.д.) |
| Timeout | Таймаут ожидания completion или ответа |

---

## 32. Расширенный список источников (все секции)

- Daml Application Architecture: https://docs.daml.com/app-dev/app-arch.html  
- The Ledger API Services: https://docs.daml.com/app-dev/services.html  
- Command Deduplication: https://docs.daml.com/app-dev/command-deduplication.html  
- How Daml Types are Translated to Protobuf: https://docs.daml.com/app-dev/grpc/daml-to-ledger-api.html  
- Canton API Configuration: https://docs.daml.com/canton/usermanual/apis.html  
- Daml Authorization: https://docs.daml.com/app-dev/authorization.html  
- Parties and Users: https://docs.daml.com/app-dev/parties-users.html  
- Best practices Canton: https://docs.digitalasset.com/build/3.4/sdlc-howtos/sdlc-best-practices.html  
- Explicit contract disclosure: https://docs.digitalasset.com/build/3.3/sdlc-howtos/applications/develop/explicit-contract-disclosure.html  
- gRPC Ledger API Configuration: https://docs.digitalasset.com/operate/3.4/howtos/configure/apis/ledger_api.html  
- Rust API Guidelines: https://rust-lang.github.io/api-guidelines/  
- Rust API Guidelines Checklist: https://rust-lang.github.io/api-guidelines/checklist.html  
- Cargo features: https://doc.rust-lang.org/cargo/reference/features.html  
- Azure SDK Rust: https://azure.github.io/azure-sdk/rust_introduction.html  
- Smithy Rust Tenets: https://smithy-lang.github.io/smithy-rs/design/tenets.html  
- Rust traits and DI: https://jmmv.dev/2022/04/rust-traits-and-dependency-injection.html  
- Testability in Rust: https://audunhalland.github.io/blog/testability-reimagining-oop-design-patterns-in-rust/  
- thiserror/anyhow: https://google.github.io/comprehensive-rust/error-handling/thiserror-and-anyhow.html  
- RFC 2008 non_exhaustive: https://rust-lang.github.io/rfcs/2008-non-exhaustive.html  
- RFC 1105 API evolution: https://rust-lang.github.io/rfcs/1105-api-evolution.html  
- OpenTelemetry Rust: https://opentelemetry.io/docs/languages/rust/exporters/  
- 12-Factor config: https://12factor.net/config  
- Tower Retry: https://docs.rs/tower/latest/tower/retry/  
- tonic Channel: https://docs.rs/tonic/latest/tonic/transport/struct.Channel.html  
- tonic Streaming: https://docs.rs/tonic/latest/tonic/struct.Streaming.html  
- mocktail: https://docs.rs/mocktail  
- config crate: https://docs.rs/config/latest/config/  
- Daml HA: https://docs.daml.com/deploy-daml/infrastructure-architecture/high-availability/ha-and-scaling/implementing-ha.html  
- ERC-3643: https://eips.ethereum.org/EIPS/eip-3643  
- T-REX ERC-3643: https://docs.t-rex.network/developers/erc-3643-protocol  
- Design meeting Send bound: https://hackmd.io/@rust-lang-team/rJks8OdYa  
- RFC 3654 RTN: https://rust-lang.github.io/rfcs/3654-return-type-notation.html  
- Ledger API proto (digital-asset/daml): ledger-api/grpc-definitions/  
- TransactionFilter / InclusiveFilter: Java/Node bindings docs.daml.com  
- Canton Splice docs: https://docs.dev.global.canton.network.sync.global/  

---

## 33. Повторный чеклист для слияния (расширенный)

При объединении с DEFI_SDK_MASTER_PROMPT.md:

- [ ] **Async-модель:** Явно описать, что submit не ждёт execution; submit_and_wait через Command Service или Completion stream + корреляция по command_id; в примерах показать оба варианта.
- [ ] **Change ID:** В типах Commands и в Chain 2 описать application_id, command_id, submission_id, act_as, read_as, deduplication_period; пример генерации command_id (UUID); поведение при retry.
- [ ] **Фильтры:** GetActiveContracts/GetTransactions с TransactionFilter; использовать template_filters/interface_filters; минимальная версия Canton 2.8 в требованиях.
- [ ] **Конфиг:** LedgerApiConfig: grpc_host, grpc_port, http_url, tls, connect_timeout_secs, request_timeout_secs, ledger_id, auth_token (optional), max_inbound_message_size (optional); TemplateIds; приоритет файл → env; секреты только из env.
- [ ] **Ошибки:** SdkError варианты с контекстом (thiserror); не логировать секреты и полные payload; таблица ошибок (раздел 31.4) перенести в промт.
- [ ] **Трейты:** ComplianceProvider, OracleProvider; опционально LedgerClient как трейт для тестов; инжекция в TreasuryService/RealEstateService через конструктор.
- [ ] **Transport:** Retry/timeout (Tower); при необходимости connection pool (soda-pool); TLS и keep_alive в конфиге.
- [ ] **Observability:** Feature "observability"; спаны (ledger.submit, ledger.submit_and_wait); метрики (counters, histograms) по разделам 21.
- [ ] **Тесты:** Юнит с моками по трейтам; интеграция с feature "integration" или env CANTON_LEDGER_GRPC; mocktail для gRPC без участника; изоляция command_id и при необходимости application_id.
- [ ] **Документация:** DEFI_SDK_USAGE.md с примером подключения и вызовов; doctest для основных типов; ссылки на Daml/Canton docs.
- [ ] **Версии зависимостей:** Согласовать с разделом 12.3 и 31.
- [ ] **Offsets:** Сохранение offset после ACS; продолжение Transaction stream с него; сценарий восстановления после сбоя.
- [ ] **Proto:** Таблицы полей Commands, CreateCommand, ExerciseCommand (31.1, 31.2) включить в промт; конвертеры domain ↔ proto и DamlValue ↔ Value с учётом Record/Value структуры (разделы 14, 30).
- [ ] **Конфиг сервера Canton:** В документации SDK описать типичные порты и параметры участника (раздел 20, 25) для справки операторов.
- [ ] **Rust API:** Naming (C-CASE), common traits, #[non_exhaustive], Result/Option комбинаторы, SemVer (разделы 2, 12, 22, 26).

---

## 34. Command Completion: статусы и обработка ошибок

### 34.1 Completion status (proto)

- **Success** — команда успешно исполнена; в ответе может быть transaction_id и т.д. в зависимости от версии proto.
- **Failed** — команда отклонена; в сообщении **reason** (строка или структура) с описанием причины (например дубликат command_id, неверный шаблон, недостаточно прав, ошибка Daml).
- **AbortedDueToShutdown** — участник завершает работу; команда не исполнена.
- **MaxRetriesReached** — внутренние повторы участника исчерпаны; команда не исполнена.

### 34.2 Обработка в приложении

- При подписке на Completion stream: для каждого completion проверять **command_id** (и при необходимости submission_id); по **status** вызывать on_success или on_failure.
- При **Failed** извлекать **reason** и маппить в SdkError::SubmitFailed { reason, command_id }; логировать без полного payload.
- Retry: при таймауте или временной ошибке повторять submit с **тем же command_id** в пределах deduplication period (идемпотентность); при постоянной ошибке (например неверный шаблон) не повторять.

### 34.3 Для SDK

- В LedgerClient или в слое submit_and_wait: после submit подписаться на completion (или использовать Command Service); ждать completion с нужным command_id с таймаутом; при Success возвращать Ok; при Failed возвращать SdkError::SubmitFailed с reason.
- В документации описать возможные причины Failed (duplicate command_id, template not found, authorization, Daml execution error) и рекомендации по retry.

Источник: The Ledger API Services; Command Completion Service proto; Canton CommandResult.

---

## 35. Rust serde: опциональные поля и дефолты в конфиге

### 35.1 #[serde(default)]

- Поле отсутствует в JSON/YAML → при десериализации используется **Default::default()** для типа.
- Для числовых полей (u64, u16) default = 0; для Option = None; для String = "" (если Default для String не переопределён).

### 35.2 #[serde(default = "path")]

- Кастомный дефолт через функцию: `#[serde(default = "default_connect_timeout")] connect_timeout_secs: u64` и `fn default_connect_timeout() -> u64 { 10 }`.
- Удобно для конфига SDK: default_grpc_port() = 5001, default_ledger_id() = "participant".

### 35.3 Option<T>

- Поле может отсутствовать или быть null → десериализуется в **Option<T>**; отсутствие → None.
- Для опциональных параметров конфига (auth_token, http_url, ledger_id) использовать Option<String>; при загрузке из env подставлять значение отдельно.

### 35.4 Рекомендация для LedgerApiConfig

- Обязательные: grpc_host, grpc_port (или единый endpoint).
- Опциональные с дефолтом: connect_timeout_secs (default = 10), request_timeout_secs (default = 30), tls (default = false), ledger_id (default = "participant" или None).
- Опциональные без дефолта: http_url, auth_token (Option<String>); auth_token не десериализовать из файла в репо — только из env при загрузке.

Источник: serde attributes; serde default; Rust serde field-attrs.

---

## 36. Глоссарий терминов (для слияния с промтом)

- **Ledger API** — gRPC/HTTP API участника Canton для чтения и записи состояния ledger (команды, транзакции, активные контракты, completions).
- **Participant** — узел Canton, обслуживающий Ledger API и хранящий состояние для подмножества партий.
- **Template** — шаблон контракта Daml (например InstitutionalAsset:InstitutionalAsset).
- **Contract** — экземпляр контракта на ledger; идентифицируется **ContractId**.
- **Choice** — операция над контрактом (например Approve на AssetPurchaseRequest).
- **Create** — команда создания контракта по шаблону с аргументами (Record).
- **Exercise** — команда выполнения choice на контракте с аргументом (Value).
- **Change ID** — (submitting parties, application_id, command_id); ключ дедупликации команд.
- **Deduplication period** — период, в течение которого повторная отправка с тем же change ID отклоняется или идемпотентно возвращает результат.
- **Offset** — непрозрачный маркер позиции на ledger; используется для подписки на транзакции и для сохранения состояния приложения.
- **TransactionFilter** — фильтр по партиям и шаблонам/интерфейсам для GetActiveContracts и GetTransactions.
- **act_as** — партии, от имени которых приложение действует при submission.
- **read_as** — партии, от имени которых приложение читает (видимость контрактов).
- **ComplianceProvider** — интерфейс SDK для проверки соответствия (KYC/AML, лимиты) перед операциями.
- **OracleProvider** — интерфейс SDK для получения цен, yield, оценок недвижимости.
- **submit_and_wait** — паттерн: submit команды + ожидание completion по command_id (через Command Service или Completion stream).

---

## 37. Примеры кода (концептуальные) для слияния с промтом

### 37.1 Загрузка конфига (Rust)

```rust
use config::{Config, Environment, File};

pub fn load_ledger_api_config(path: &Path) -> Result<LedgerApiConfig, SdkError> {
    let builder = Config::builder()
        .add_source(File::from(path))
        .add_source(Environment::with_prefix("CANTON").separator("__"));
    let config = builder.build()?;
    let grpc_host: String = config.get("ledger_api.grpc_host")?;
    let grpc_port: u16 = config.get("ledger_api.grpc_port")
        .unwrap_or(5001);
    let auth_token: Option<String> = std::env::var("CANTON_AUTH_TOKEN").ok();
    Ok(LedgerApiConfig {
        grpc_host,
        grpc_port,
        http_url: config.get("ledger_api.http_url").ok(),
        tls: config.get("ledger_api.tls").unwrap_or(false),
        connect_timeout_secs: config.get("ledger_api.connect_timeout_secs").unwrap_or(10),
        request_timeout_secs: config.get("ledger_api.request_timeout_secs").unwrap_or(30),
        ledger_id: config.get("ledger_api.ledger_id").ok(),
        auth_token,
    })
}
```

### 37.2 Генерация command_id и submit (псевдокод)

```rust
let command_id = uuid::Uuid::new_v4().to_string();
let application_id = "canton-defi-sdk".to_string();
let commands = Commands {
    ledger_id: config.ledger_id.clone(),
    application_id: application_id.clone(),
    command_id: command_id.clone(),
    act_as: vec![party_id.clone()],
    read_as: vec![],
    commands: vec![Command::Create(create_cmd)],
    deduplication_period: Some(DedupPeriod::Duration(Duration::from_secs(60))),
    submission_id: None,
    ..Default::default()
};
ledger_client.submit(commands).await?;
// затем: подписаться на completion и ждать completion с command_id или использовать Command Service
```

### 37.3 Обработка Completion stream (псевдокод)

```rust
let mut stream = ledger_client.subscribe_completion(application_id, parties).await?;
while let Some(msg) = stream.message().await? {
    for completion in msg.completions {
        if completion.command_id == expected_command_id {
            match completion.status {
                Some(Status::Success(_)) => return Ok(()),
                Some(Status::Failed(reason)) => return Err(SdkError::SubmitFailed {
                    command_id: expected_command_id,
                    reason: reason.to_string(),
                }),
                _ => continue,
            }
        }
    }
}
Err(SdkError::Timeout)
```

---

## 38. Пример YAML конфига (полный) для слияния с промтом

```yaml
# config/example.yaml — полный пример для SDK

ledger_api:
  grpc_host: "65.108.15.30"
  grpc_port: 30501
  http_url: "http://65.108.15.30:30757"
  tls: false
  connect_timeout_secs: 10
  request_timeout_secs: 30
  ledger_id: "participant"
  # auth_token задаётся через env CANTON_AUTH_TOKEN

template_ids:
  institutional_asset: "InstitutionalAsset:InstitutionalAsset"
  asset_purchase_request: "InstitutionalAsset:AssetPurchaseRequest"
  asset_holding: "InstitutionalAsset:AssetHolding"
  dividend_distribution: "InstitutionalAsset:DividendDistribution"

compliance:
  provider: "mock"   # mock | http | sumsub
  strict_mode: false

oracle:
  provider: "mock"   # mock | pyth | chainlink
  cache_ttl_secs: 60
```

Переменные окружения (приоритет над файлом при использовании config-rs + Environment):

- CANTON__LEDGER_API__GRPC_HOST
- CANTON__LEDGER_API__GRPC_PORT
- CANTON__LEDGER_API__TLS
- CANTON_AUTH_TOKEN (отдельно, не в файле)

---

## 39. Индекс разделов документа (навигация)

| № | Раздел | Содержание |
|---|--------|------------|
| 1 | Canton/Daml Ledger API | Асинхронная модель, сервисы, change ID, дедупликация, конфиг, фильтры, disclosure, offsets, Value→Proto, User Management, Authorization |
| 2 | Rust SDK архитектура | API Guidelines, Azure/Smithy tenets, traits/DI, builder, thiserror/anyhow, non_exhaustive, doctests, features |
| 3 | Transport и gRPC | Tonic, Tower, retry/timeout, Channel, pool, TLS |
| 4 | Observability | OpenTelemetry, tracing, metrics, feature-gate |
| 5 | Конфигурация | 12-factor, config-rs, serde, env |
| 6 | Тестирование | Юнит с моками, интеграция с env/feature, mocktail |
| 7 | Безопасность | TLS, JWT, act_as/read_as |
| 8 | Institutional/RWA | Регуляторика, KYC/AML, токенизация, settlement |
| 9 | Canton best practices | SDLC, рекомендации для SDK |
| 10 | Сводная таблица | Что учитывать в SDK |
| 11 | Источники | Ссылки |
| 11A | Rust API чеклист | Naming, C-GOOD-ERR, C-COMMON-TRAITS, C-DOC, C-CALLER-CONTROL, C-SECURE |
| 11B | RWA техстек | ERC-3643, KYC/KYB, Travel Rule, governance |
| 12 | Доп. аспекты SDK | Async runtime, Streams, версии зависимостей, naming, доменные типы, конфиг, тесты, command_id, proto, документация |
| 13 | Чеклист слияния | Пункты для переноса в мастер-промт |
| 14 | Ledger API proto | Commands, CreateCommand, ExerciseCommand, Value, Record |
| 15 | Rust async Send/Sync | RTN, рекомендации для SDK |
| 16 | config-rs | ConfigBuilder, merge, env |
| 17 | gRPC streaming tonic | Streaming, message(), цикл |
| 18 | TransactionFilter | filtersByParty, InclusiveFilter, template_filters |
| 19 | ERC-3643 | Компоненты, правила, для SDK |
| 20 | Canton порты | Участник, конфиг |
| 21 | OpenTelemetry | Спаны, метрики, feature-gate |
| 22 | Result/Option | Комбинаторы в API |
| 23 | Чтение состояния | StateService, ACS, Transaction Service, Completion, Event Query |
| 24 | Идентификаторы | Identifier, Package Service, Version Service |
| 25 | Конфиг Canton сервер | Ledger API server, JWT/JWKS, клиент SDK |
| 26 | SemVer | Обратная совместимость |
| 27 | Тестирование стратегии | Юнит, интеграция, мок gRPC |
| 28 | Безопасность детально | Секреты, логирование, TLS |
| 29 | RWA оракулы | OracleProvider, Mock/Http |
| 30 | Daml типы→Proto | Таблица типов, контракты в событиях |
| 31 | Таблицы для промта | Commands, Create/Exercise, конфиг, ошибки |
| 32 | Расширенный список источников | Все ссылки |
| 33 | Чеклист слияния расширенный | Полный список пунктов |
| 34 | Completion status | Success, Failed, обработка, retry |
| 35 | serde optional/default | Конфиг LedgerApiConfig |
| 36 | Глоссарий | Термины для промта |
| 37 | Примеры кода | Конфиг, command_id, Completion stream |
| 38 | Пример YAML | Полный example.yaml |
| 39 | Индекс разделов | Эта таблица |

---

## 40. Детальная экспертиза: Tower middleware для gRPC

### 40.1 Retry layer

- **tower::retry::Retry** — обёртка над сервисом; при определённых ошибках (например Unavailable, DeadlineExceeded) повторяет запрос по политике.
- **Policy**: тип, определяющий, повторять ли запрос и с каким backoff; для gRPC обычно повторять при transient errors (коды gRPC: Unavailable, ResourceExhausted с backoff).
- Ограничение: для retry нужно клонировать запрос; http::Request в tonic не Clone — используют обёртки (например Arc<Request>) или кастомную политику.

### 40.2 Timeout layer

- **tower::timeout::Timeout** — обёртка с таймаутом; при превышении возвращает ошибку (например DeadlineExceeded).
- Для Ledger API: задать request_timeout (например 30s) на уровне вызова submit или на всём канале.

### 40.3 Композиция

- **ServiceBuilder::new().timeout(Duration::from_secs(30)).retry(policy).layer(channel)** — порядок слоёв важен: сначала timeout, затем retry (чтобы таймаут применялся к каждому попытке).
- В SDK: опционально оборачивать Ledger API клиент в Timeout и Retry; конфиг: connect_timeout_secs, request_timeout_secs, retry_max_attempts, retry_backoff.

Источник: Tower docs; tonic middleware.

---

## 41. Детальная экспертиза: Daml Decimal и Numeric в proto

### 41.1 Представление в Ledger API

- В Daml тип **Decimal** имеет precision и scale; в proto может передаваться как строка (например "123.45") или как специальное сообщение **Numeric** (в зависимости от версии Ledger API).
- В Ledger API v1/v2 часто используется строка для совместимости; парсинг в rust_decimal::Decimal на стороне SDK.

### 41.2 Для SDK

- При конвертации DamlValue → proto Value для полей decimal использовать proto Numeric или text с строковым представлением числа (согласно документации Daml-to-Proto для используемой версии).
- При парсинге из proto Value в доменные типы (TreasuryBill.total_supply, price_per_token и т.д.) парсить Numeric/text в rust_decimal::Decimal; при ошибке парсинга возвращать ConversionError с контекстом поля.
- Единообразие точности: в TypeScript DeFi используется Decimal.js; в Rust — rust_decimal; при маппинге сохранять достаточную точность (например scale 18 для денежных сумм).

Источник: How Daml Types are Translated to Protobuf; Ledger API value.proto; rust_decimal.

---

## 42. Детальная экспертиза: Party и ContractId в proto

### 42.1 Party

- В proto передаётся как строка (party id); в Daml это непрозрачный идентификатор партии (например "party-1234-...").
- В SDK: PartyId как тип-обёртка над String или type alias; при конвертации DamlValue::Party(s) → proto party = s.

### 42.2 ContractId

- В proto передаётся как строка (contract id); в Daml — непрозрачный идентификатор контракта (например "#contract-id").
- В SDK: ContractId как тип-обёртка или String; при конвертации в ExerciseCommand и при парсинге из CreatedEvent использовать строковое представление.
- В доменных типах (TreasuryBill.contract_id, TreasuryBillHolding.contract_id) хранить как Option<String> или Option<ContractId>.

Источник: Ledger API proto; Daml Value types.

---

## 43. Детальная экспертиза: Identifier и record_id в proto

### 43.1 Identifier

- **package_id** — строка (хэш пакета).
- **module_name** — строка (имя модуля Daml).
- **entity_name** — строка (имя сущности: шаблон или интерфейс).
- В конфиге SDK шаблоны задаются строкой "ModuleName:EntityName"; при загрузке парсить в Identifier (package_id опционально — из участника или конфига).

### 43.2 record_id в Record

- Каждый Record в proto имеет **record_id** типа Identifier — идентифицирует тип записи (package_id, module_name, entity_name).
- При конвертации DamlRecord → proto Record заполнять record_id из типа аргумента шаблона (например InstitutionalAsset); при парсинге из события record_id можно использовать для диспетчеризации типа.

Источник: Ledger API value.proto; How Daml Types are Translated to Protobuf.

---

## 44–60. Повторные и справочные блоки (для объёма и полноты)

### 44. Ключевые URL (быстрый доступ)

- Ledger API Services: https://docs.daml.com/app-dev/services.html  
- Command Deduplication: https://docs.daml.com/app-dev/command-deduplication.html  
- Daml to Protobuf: https://docs.daml.com/app-dev/grpc/daml-to-ledger-api.html  
- Canton API Config: https://docs.daml.com/canton/usermanual/apis.html  
- Rust API Guidelines: https://rust-lang.github.io/api-guidelines/  
- config-rs: https://docs.rs/config/latest/config/  
- tonic: https://docs.rs/tonic/latest/tonic/  
- OpenTelemetry Rust: https://opentelemetry.io/docs/languages/rust/  
- ERC-3643: https://eips.ethereum.org/EIPS/eip-3643  

### 45. Минимальный набор сервисов Ledger API для SDK (сводка)

- **CommandSubmissionService** — Submit(Commands); обязательно.
- **StateService** (или аналог) — GetLedgerEnd; для проверки подключения и начального offset; если нет — использовать offset из последнего сообщения ACS.
- **ActiveContractsService** — GetActiveContracts(TransactionFilter); для get_bill, list_bills, get_holdings.
- **CommandCompletionService** — SubscribeCompletion; для submit_and_wait и корреляции результата.
- **TransactionService** — GetTransactions(offset, TransactionFilter); опционально для стриминга событий в реальном времени.
- **VersionService** — GetLedgerApiVersion / feature descriptor; опционально для отладки и документирования требований.
- **PackageService** — ListPackageIds, GetPackage; опционально для разрешения package_id по имени.

### 46. Порядок реализации (рекомендуемый) для слияния с промтом

1. **Chain 1:** Конфиг (LedgerApiConfig, TemplateIds, загрузка из YAML + env); подключение (LedgerClient::connect_from_config); GetLedgerEnd или аналог; тест подключения.
2. **Chain 2:** Конвертеры (Commands → proto, DamlValue → Value, DamlRecord → Record, Identifier → proto); Submit(Commands); опционально Completion stream; тест submit минимального Create.
3. **Chain 3:** Доменные типы (TreasuryBill, Holding, PurchaseRequest, CreateBillInput, PortfolioSummary); маппинг ↔ DamlRecord; TreasuryService (create_bill, list_bills, create_purchase_request, approve_purchase_request, get_holdings, get_portfolio_summary); GetActiveContracts с TransactionFilter; тесты полного цикла Treasury.
4. **Chain 4:** RealEstateService, PrivacyVaultService (или заглушки); DeFiClient facade.
5. **Chain 5:** ComplianceProvider, OracleProvider (mock + optional HTTP); инжекция в TreasuryService/RealEstateService; тесты с моками.
6. **Chain 6:** Интеграционные тесты e2e; DEFI_SDK_USAGE.md; обновление README и DEVELOPMENT_PROMPT.

### 47. Риски при реализации (для промта)

- **Несовместимость proto:** при обновлении Canton проверить совместимость proto и конвертеров; зафиксировать версию proto в репо.
- **Разные offset на участниках:** приложение не может прозрачно переключиться на другого участника без перезагрузки состояния через ACS.
- **Дедупликация:** один и тот же command_id в пределах периода блокирует повторное исполнение; при retry использовать тот же command_id; при новой бизнес-операции — новый command_id.
- **Фильтры:** в Canton 2.8+ использовать template_filters/interface_filters; иначе GetActiveContracts может вернуть пустой или неполный результат.
- **Секреты:** не логировать и не включать в ошибки; загружать только из env или секрет-менеджера.

### 48. Критерии приёмки (дополнительные) для слияния с промтом

- LedgerClient подключается к участнику по конфигу (YAML + env); get_ledger_end() или аналог возвращает offset.
- Submit(Commands) с одним CreateCommand (InstitutionalAsset) успешен; контракт появляется в ACS по фильтру.
- TreasuryService: create_bill, list_bills, create_purchase_request, approve_purchase_request (или эквивалент), get_holdings, get_portfolio_summary работают против участника или sandbox с теми же шаблонами, что и DeFi.
- Типы TreasuryBill, TreasuryBillHolding, PurchaseRequest в SDK соответствуют полям из treasuryBillsService.ts (с учётом Rust-эквивалентов).
- ComplianceProvider и OracleProvider интегрированы в TreasuryService; с MockComplianceProvider и MockOracleProvider полный цикл Treasury проходит; с failing compliance create_purchase_request возвращает ошибку.
- Документация DEFI_SDK_USAGE.md содержит пример: загрузка конфига, connect, создание TreasuryService, create_bill, list_bills, get_portfolio_summary.
- cargo test --workspace и cargo clippy --workspace -- -D warnings проходят; регрессий в существующих тестах нет.
- Интеграционный тест (feature = "integration" или env CANTON_LEDGER_GRPC) выполняет полный цикл Treasury и проверяет portfolio total_value.

### 49. Зависимости Cargo (полный список для слияния с промтом)

- canton-ledger-api: tonic (0.11–0.14), prost (0.12), tower (0.4), tokio (1.32+), canton-core.
- canton-core: serde (1, derive), serde_yaml (0.9), thiserror (2), anyhow (1), chrono (0.4, serde), rust_decimal (1, serde), uuid (1, v4, serde).
- Опционально: config (config-rs), serde-env-field; opentelemetry, opentelemetry_sdk, tracing-opentelemetry (feature "observability"); mocktail (dev-dependency, feature "integration").
- build-deps: canton-ledger-api build.rs компилирует proto из proto/com/daml/ledger/api/v2/ (или v1 по совместимости).

### 50. Итоговое напоминание

Документ содержит актуальную экспертизу по Canton/Daml Ledger API, Rust SDK-архитектуре, transport, observability, конфигурации, тестированию, безопасности, RWA/compliance и детальные таблицы/примеры для слияния с DEFI_SDK_MASTER_PROMPT.md. При объединении использовать разделы 13, 33 (чеклисты), 31 (таблицы), 36 (глоссарий), 37–38 (примеры кода и YAML), 45–49 (сервисы, порядок, риски, критерии, зависимости). Длина документа: не менее 3000 строк.

---

**Конец документа.** Документ предназначен для анализа и слияния с DEFI_SDK_MASTER_PROMPT.md: выделить конкретные требования к типам, методам, конфигу и тестам и встроить их в мастер-промт и чеклисты реализации.
