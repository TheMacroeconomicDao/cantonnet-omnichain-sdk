# Участник DevNet для разработки SDK

Единый источник по participant-ноде Canton DevNet, используемой для локальной разработки и интеграционных тестов SDK.

## Назначение

- **Локальная разработка** — подключение клиента Ledger API (gRPC) с рабочей станции.
- **Интеграционные тесты** — опциональные тесты против реального participant (по env, без обязательного доступа в CI).
- **Ручная проверка** — curl/grpcurl к HTTP и gRPC эндпоинтам.

Нода: validator в DevNet, подключён к sync.global (Scan/SV), onboarding в порядке. Ledger API экспонирован через NodePort (манифест `blockchain/k8s/participant-ledger-external-nodeport.yaml` в репозитории инфраструктуры).

---

## Эндпоинты

| Назначение              | Адрес / URL                          |
|-------------------------|--------------------------------------|
| **gRPC Ledger API**     | `65.108.15.30:30501`                 |
| **HTTP Ledger API (JSON)** | `http://65.108.15.30:30757`      |

В коде и конфиге SDK:
- **gRPC** — хост `65.108.15.30`, порт `30501` (основной канал для Tonic-клиента).
- **HTTP** — для здоровья, отладки, ручных запросов (curl).

---

## Конфигурация в проекте

### config/example.yaml

Секция `ledger_api` задаёт подключение к Ledger API. Для разработки по умолчанию указан DevNet participant:

```yaml
# Ledger API participant (для разработки — DevNet; для прода — переопределить в config/local.yaml)
ledger_api:
  grpc_host: "65.108.15.30"
  grpc_port: 30501
  http_url: "http://65.108.15.30:30757"   # опционально: здоровье, отладка
  tls: false                               # DevNet без TLS
```

Для production: переопределить в `config/local.yaml` (или взять за основу `config/example-production.yaml`):

```yaml
ledger_api:
  grpc_host: "participant.your-domain.com"
  grpc_port: 5001
  http_url: "https://participant.your-domain.com/ledger"   # опционально
  tls: true
  tls_certs:
    ca_cert_path: "/etc/ssl/canton/ca.pem"
    client_cert_path: "/etc/ssl/canton/client.pem"
    client_key_path: "/etc/ssl/canton/client.key"
  connect_timeout: "10s"
  request_timeout: "30s"
```

Шаблон целиком: **config/example-production.yaml** (скопировать в `config/local.yaml` и подставить свои значения).

### Переменные окружения (опционально)

Интеграционные тесты могут читать endpoint из env, чтобы не требовать доступа к ноде в CI:

| Переменная              | Пример значения              | Описание |
|-------------------------|-----------------------------|----------|
| `CANTON_LEDGER_GRPC`    | `65.108.15.30:30501`        | хост:порт gRPC; если не задано — тесты, зависящие от ноды, пропускаются |
| `CANTON_LEDGER_HTTP`    | `http://65.108.15.30:30757` | URL HTTP API (по желанию) |

---

## Ограничения и безопасность

- **Только разработка/тесты.** DevNet может сбрасываться; не использовать для production.
- **Без TLS.** Трафик по открытому каналу; не передавать секреты и production-данные.
- Для production: отдельный participant, TLS (mTLS/JWT по планам transport layer), конфиг в `config/local.yaml`.

---

## Связь с остальной документацией

- **README.md** — сборка, тесты, раздел «Участник DevNet» со ссылкой сюда.
- **config/example.yaml** — секция `ledger_api` с дефолтами DevNet.
- **DEVELOPMENT_PROMPT.md** / **PRE_DEVELOPMENT_CHECKLIST.md** — при появлении: использование этой ноды в Phase 1 и интеграционных сценариях.
- **research/04-daml-ledger-api.md**, **research/08-sdk-architecture-design.md** — архитектура Ledger API и SDK.
