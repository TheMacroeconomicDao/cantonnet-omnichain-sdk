# Canton DeFi → Rust SDK: Master Implementation Prompt (2025)

**Version:** 1.1  
**Scope:** Полная реализация методов, фич и функций DeFi платформы canton-otc в Rust SDK (cantonnet-omnichain-sdk).  
**Ограничение:** Промт не менее 2000 строк. Prompt engineering 2025: chain-of-thought, structured output, context injection, prompt chains.

---

## PART 1 — META & PROMPT ENGINEERING 2025

### 1.1 Роль и контекст исполнения

Ты — опытная команда сеньор-инженеров (архитектор, Rust lead, DeFi domain expert). Задача: спроектировать и реализовать в Rust SDK полный функционал, необходимый для работы DeFi платформы canton-otc (canton-otc/defi), на основе реального кода и реальных данных репозитория.

**Обязательные принципы:**
- Основа: только реальный код и реальные данные (файлы в `canton-otc/src`, `canton-otc/cantonnet-omnichain-sdk`).
- Архитектура: слоистая, с чётким разделением Transport → Ledger API → Domain → Public API.
- Инжиниринг: best practices 2025 (Rust: async, error handling, typing; Canton: Ledger API v2, Daml contracts; DeFi: treasury, real estate, privacy, compliance, oracle).
- Никаких выдуманных эндпоинтов или типов: каждый метод/тип должен иметь соответствие в существующем коде DeFi или в официальной Canton/Daml спецификации.

["DEFI_SDK_EXPERTISE_RESEARCH.md`](../docs/DEFI_SDK_EXPERTISE_RESEARCH.md) \u0440\u0430\u0437\u0434\u0435\u043b\u044b 1.1, 1.2, 23.4*\n\n**Chain 1 \u2014 Config & Transport**  \n\u0412\u0445\u043e\u0434: \u044d\u0442\u043e\u0442 \u0434\u043e\u043a\u0443\u043c\u0435\u043d\u0442, \u0441\u0435\u043a\u0446\u0438\u0438 2\u20133, 6.1\u20136.2.  \n\u0412\u044b\u0445\u043e\u0434: LedgerApiConfig \u0438\u0437 \u043a\u043e\u043d\u0444\u0438\u0433\u0430, ChannelBuilder (TLS \u043e\u043f\u0446\u0438\u043e\u043d\u0430\u043b\u044c\u043d\u043e), \u043f\u043e\u0434\u043a\u043b\u044e\u0447\u0435\u043d\u0438\u0435 \u043a participant.  \n\u041a\u0440\u0438\u0442\u0435\u0440\u0438\u0439: `LedgerClient::connect(endpoint, ledger_id)` \u0443\u0441\u043f\u0435\u0448\u043d\u043e \u043f\u043e\u0434\u043a\u043b\u044e\u0447\u0430\u0435\u0442\u0441\u044f \u043a DevNet participant (65.108.15.30:30501).\n\n**Chain 2 \u2014 Ledger client & conversion**  \n\u0412\u0445\u043e\u0434: \u0432\u044b\u0432\u043e\u0434 Chain 1, \u0441\u0435\u043a\u0446\u0438\u0438 4 (DeFi \u2192 Ledger \u043e\u043f\u0435\u0440\u0430\u0446\u0438\u0438), 6.3, 7.  \n\u0412\u044b\u0445\u043e\u0434: \u043a\u043e\u043d\u0432\u0435\u0440\u0442\u0435\u0440\u044b `canton_core::Commands` \u2192 proto Commands; \u0432\u044b\u0441\u043e\u043a\u043e\u0443\u0440\u043e\u0432\u043d\u0435\u0432\u044b\u0435 \u043c\u0435\u0442\u043e\u0434\u044b submit_create, submit_exercise; completion handling \u0434\u043b\u044f submit_and_wait.  \n\u041a\u0440\u0438\u0442\u0435\u0440\u0438\u0439: \u0441\u043e\u0437\u0434\u0430\u043d\u0438\u0435 \u043a\u043e\u043d\u0442\u0440\u0430\u043a\u0442\u0430 InstitutionalAsset \u0438 \u0432\u044b\u043f\u043e\u043b\u043d\u0435\u043d\u0438\u0435 choice \u043f\u043e \u0441\u0443\u0449\u0435\u0441\u0442\u0432\u0443\u044e\u0449\u0438\u043c \u0448\u0430\u0431\u043b\u043e\u043d\u0430\u043c DeFi; \u043a\u043e\u0440\u0440\u0435\u043a\u0442\u043d\u0430\u044f \u043e\u0431\u0440\u0430\u0431\u043e\u0442\u043a\u0430 completions \u043f\u043e command_id."]

**Chain 3 — Domain services (Treasury)**  
Вход: вывод Chain 2, секция 4.1 (Treasury), 7.1.  
Выход: TreasuryService в SDK: create_bill, get_bills, create_purchase_request, get_holdings, get_portfolio.  
Критерий: соответствие сигнатурам и типам из `treasuryBillsService.ts` и API routes.

**Chain 4 — Domain services (Real Estate, Privacy)**  
Вход: вывод Chain 3, секции 4.2–4.3, 7.2–7.3.  
Выход: RealEstateService, PrivacyVaultService (или общий AssetService с доменными типами).  
Критерий: соответствие методам realEstateService, privacyVaultService и панелям UI.

**Chain 5 — Compliance & Oracle**  
Вход: вывод Chain 4, секции 4.4–4.5, 7.4–7.5.  
Выход: ComplianceClient (KYC/AML hooks), OracleClient (prices, treasury yields) — опционально через внешние API или заглушки с интерфейсом.  
Критерий: интерфейсы совместимы с вызовами из DeFi сервисов.

**Chain 6 — Integration & testing**  
Вход: все предыдущие выводы, секция 8.  
Выход: интеграционные тесты против DevNet participant, документация по использованию SDK для DeFi backend.  
Критерий: все acceptance criteria из секции 8 выполнены.

### 1.3 Structured output

При реализации каждой цепи возвращать:
1. **Diff/патчи** — конкретные изменения в файлах (путь, старый фрагмент, новый фрагмент).
2. **Новые типы** — полные определения (Rust или IDL), связанные с существующими типами DeFi.
3. **Список тестов** — какие тесты добавлены/обновлены и как их запускать.
4. **Риски** — что может сломаться при мерже и как откатить.

### 1.4 Context injection

При ответе всегда включать минимальный контекст:
- Текущая цепь (1–6).
- Файлы, от которых отталкиваешься (полные пути из репо).
- Ссылки на секции этого документа (например: «см. 4.1, 7.1»).

---

## PART 2 — PROJECT OVERVIEW

### 2.1 Репозиторий и пути

- **DeFi платформа:** `canton-otc/` (Next.js, TypeScript).
  - Страницы: `src/app/defi/page.tsx`, `src/app/defi/treasury/page.tsx`, `src/app/defi/realestate/page.tsx`, `src/app/defi/privacy/page.tsx`.
  - Компоненты: `src/components/defi/CantonDeFi.tsx`, `TreasuryBillsPanel.tsx`, `RealEstatePanel.tsx`, `PrivacyVaultsPanel.tsx`, `CCPurchaseWidget.tsx`, `ProductCard.tsx`, `StablecoinSelector.tsx`, `MultiPartyAuthPanel.tsx`, `MultiPartyDashboard.tsx`.
  - API routes: `src/app/api/defi/auth/*`, `src/app/api/defi/compliance/kyc/route.ts`, `src/app/api/defi/oracle/prices/route.ts`, `src/app/api/defi/oracle/treasury-yields/route.ts`, `src/app/api/defi/treasury/portfolio/route.ts`, `src/app/api/defi/treasury/bills/route.ts`, `src/app/api/defi/treasury/bills/[billId]/route.ts`, `src/app/api/defi/treasury/purchases/route.ts`.
  - Сервисы: `src/lib/canton/services/treasuryBillsService.ts`, `realEstateService.ts`, `privacyVaultService.ts`, `damlIntegrationService.ts`, `complianceService.ts`, `oracleService.ts`.
  - Хуки: `src/lib/canton/hooks/realCantonIntegration.ts`, `useTreasuryBills.ts`, `useRealEstate.ts`, `usePrivacyVaults.ts`, `useCantonBridge.ts`, `useMultiPartyWorkflowService.ts`.
  - Стор и конфиг: `src/lib/canton/store/cantonStore.ts`, `src/lib/canton/config/stablecoins.ts`, `realBridgeConfig.ts`.

- **Rust SDK:** `canton-otc/cantonnet-omnichain-sdk/`.
  - Крейты: `canton-core`, `canton-ledger-api`, `canton-crypto`, `canton-wallet`, `canton-transport`, `canton-reliability`, `canton-observability`.
  - Конфиг: `config/example.yaml`, `config/example-production.yaml`, `docs/DEVNET_PARTICIPANT.md`.
  - Исследования: `research/08-sdk-architecture-design.md`, `research/04-daml-ledger-api.md`, `DEVELOPMENT_PROMPT.md`.

- **Инфраструктура (нода):** `canton-otc/blockchain/`.
  - Participant Ledger API: HTTP `http://65.108.15.30:30757`, gRPC `65.108.15.30:30501`.
  - Документ: `blockchain/DEFI_CONNECT_DEVNET.md`.

### 2.2 Потоки данных DeFi

1. **Фронт (React)** → вызывает хуки (`useRealCantonNetwork`, `useTreasuryBills`, `useRealEstateService`, `usePrivacyVaultService`) → сервисы (TreasuryBillsService, RealEstateService, PrivacyVaultService) → DamlIntegrationService / CantonNetworkClient → Participant (HTTP JSON API 7575 или gRPC).
2. **API routes (Next.js)** → те же сервисы (DamlIntegrationService, TreasuryBillsService, OracleService, ComplianceService) → participant или внешние провайдеры (Oracle, KYC).
3. **Цель SDK:** предоставить Rust-реализацию тех же операций (Ledger API, доменные операции Treasury/RealEstate/Privacy), чтобы backend или отдельный сервис мог работать с тем же participant без дублирования логики на TypeScript.

---

## PART 3 — DEFI FEATURE INVENTORY (REAL CODE)

### 3.1 Treasury Bills (treasuryBillsService.ts, API, hooks)

**Типы (реальные):**
- `TreasuryBill`: billId, name, symbol, description, issuer, custodian, maturity, maturityDate, issueDate, totalSupply, availableSupply, pricePerToken, minimumInvestment, currentYield, expectedYield, yieldToMaturity, status (ACTIVE|SUSPENDED|MATURED|DELISTED), contractId?, createdAt, updatedAt.
- `TreasuryBillHolding`: holdingId, billId, investor, tokensOwned, averageCostBasis, currentMarketValue, unrealizedGainLoss, unrealizedGainLossPercent, purchaseDate, purchasePrice, accumulatedYield, lastYieldDistribution, status (ACTIVE|SOLD|MATURED), contractId?, createdAt, updatedAt.
- `PurchaseRequest`: requestId, billId, investor, numberOfTokens, totalAmount, paymentMethod, status (PENDING|APPROVED|REJECTED|COMPLETED|FAILED), kycLevel, complianceCheck, requestDate, expiryDate, completedAt?, contractId?, createdAt, updatedAt.
- `YieldDistribution`: distributionId, billId, totalYield, yieldPerToken, distributionDate, period { startDate, endDate }, totalTokens, totalInvestors, transactionHash?, blockNumber?, createdAt.

**Методы сервиса (реальные):**
- `createTreasuryBill(billData: Partial<TreasuryBill>): Promise<TreasuryBill>` — создаёт bill, опционально Daml createInstitutionalAsset (assetClass: FIXED_INCOME, subAssetClass: TREASURY_BILL).
- `getTreasuryBill(billId: string): TreasuryBill | undefined`
- `getAllTreasuryBills(): TreasuryBill[]`
- `getActiveTreasuryBills(): TreasuryBill[]`
- `updateTreasuryBill(billId: string, updates: Partial<TreasuryBill>): Promise<TreasuryBill>`
- `createPurchaseRequest(billId, investor, numberOfTokens, paymentData): Promise<PurchaseRequest>` — валидация min/max, availability, compliance; Daml createPurchaseRequest(assetContractId, investor, numberOfTokens, paymentData).
- `approvePurchaseRequest(requestId, approver, transactionHash, blockNumber): Promise<TreasuryBillHolding>` — обновление supply, создание holding, Daml approvePurchase при наличии contractId.
- `getInvestorPortfolioSummary(investor: string)` — агрегат по holdings (totalValue, totalInvested, yield, unrealizedGains и т.д.).
- `getUserHoldings(investor: string)` — список TreasuryBillHolding для инвестора.
- `getAvailableBills()` — для хука useTreasuryBills (по сути getActiveTreasuryBills или аналог).

**API routes:**
- GET `/api/defi/treasury/portfolio?investor=` → getInvestorPortfolioSummary(investor).
- GET `/api/defi/treasury/bills` (опционально ?status=, ?maturity=) → getAllTreasuryBills с фильтрами.
- POST `/api/defi/treasury/bills` → createTreasuryBill(body).
- GET `/api/defi/treasury/bills/[billId]` → getTreasuryBill(billId).
- PUT `/api/defi/treasury/bills/[billId]` → updateTreasuryBill(billId, body).
- DELETE `/api/defi/treasury/bills/[billId]` → updateTreasuryBill(billId, { status: 'DELISTED' }).
- GET `/api/defi/treasury/purchases` (опционально ?investor=, ?status=, ?billId=) → список purchase requests.
- POST `/api/defi/treasury/purchases` → createPurchaseRequest(body.billId, body.investor, body.numberOfTokens, body.paymentData).

**Daml контракты (damlIntegrationService.ts):**
- Шаблоны: `InstitutionalAsset:InstitutionalAsset`, `InstitutionalAsset:AssetPurchaseRequest`, `InstitutionalAsset:AssetHolding`, `InstitutionalAsset:DividendDistribution`.
- Методы: createInstitutionalAsset(assetData), createPurchaseRequest(assetContractId, investor, numberOfTokens, paymentData), approvePurchase (exercise).

### 3.2 Real Estate (realEstateService.ts, panels, API)

**Типы (реальные):**
- `PropertyInfo`: id, name, address, type, subType, totalValue (Decimal), tokenSupply, availableSupply, pricePerToken, minimumInvestment, expectedDividendYield, historicalReturns, occupancyRate, location, propertyManager, legalStructure, jurisdiction, regulatoryStatus, complianceLevel, images, documents, status, fundingProgress и др.
- `TokenPurchaseRequest`: propertyId, investorAddress, numberOfTokens, totalAmount, paymentMethod, kycLevel, accreditedInvestor, investorCountry, privacyLevel, zkProofRequired.
- `TokenPurchaseResult`: transactionId, contractId, propertyId, investorAddress, tokensAcquired, totalPaid, transactionHash, blockNumber, ownershipPercentage, votingRights, dividendRights, complianceStatus, purchaseDate.
- Governance: предложения (proposalType: MAINTENANCE|RENOVATION|SALE|REFINANCING|OTHER), голосование.

**Методы сервиса (реальные):**
- getAvailableProperties(), getUserHoldings(address), getGovernanceProposals(address).
- purchaseTokens(params: PurchaseTokensParams) — создание запроса на покупку токенов недвижимости, при успехе — создание/обновление holding.
- createPurchaseRequest(propertyId, investor, numberOfTokens, paymentData).
- voteOnProposal(proposalId, voterAddress, support).

**Панель:** RealEstatePanel использует useRealEstateService(address): availableProperties, userHoldings, governanceProposals, purchaseTokens, voteOnProposal, refreshData.

### 3.3 Privacy Vaults (privacyVaultService.ts, panels)

**Типы (реальные):**
- `PrivacyVault`: id, name, description, owner, custodian, authorizedViewers, trustees, privacyLevel (STANDARD|ENHANCED|MAXIMUM|QUANTUM_SAFE), encryptionStandard, zkProofProtocol, anonymitySet, complianceLevel, jurisdiction, totalValue (Decimal), assetCount, multiSigThreshold, timelock, status (INITIALIZING|ACTIVE|LOCKED|UNDER_AUDIT|MIGRATING|DEPRECATED), encryptedMetadata, metadataHash.
- `PrivateAsset`: vaultId, type (CRYPTOCURRENCY|SECURITY_TOKEN|REAL_ESTATE|…), encryptedValue, encryptedMetadata, zkProofs, commitments, nullifiers, accessLevel, complianceProofs, auditTrail, status.
- `PrivacyVaultZKProof`, `ComplianceProof`, `SelectiveDisclosure`, `AuditEvent`.

**Методы сервиса (реальные):**
- createVault(params): создание vault (опционально Daml, ZKProofService).
- getUserVaults(owner), getVaultAssets(owner), getComplianceProofs(owner).
- depositAsset(vaultId, assetType, amount, proof?), withdrawAsset(vaultId, assetId, amount, proof?).
- generateProof(params): proofType OWNERSHIP|BALANCE|COMPLIANCE.

**Панель:** PrivacyVaultsPanel — usePrivacyVaultService(address): vaults, assets, proofs, createVault, depositAsset, generateProof, refreshData.

### 3.4 Daml Integration (damlIntegrationService.ts)

**Конфиг:** participantUrl, participantId, authToken, partyId.

**Интерфейс Ledger:**
- create(templateId, payload): Promise<ContractId<T>>
- exercise(contractId, choice, argument): Promise<R>
- query(templateId, filter?): Promise<Contract<T>[]>
- streamQuery(templateId, filter?): AsyncIterable<Contract<T>[]>
- submitAndWait(commands): Promise<Transaction>

**Шаблоны и payload:**
- InstitutionalAssetPayload: assetId, name, symbol, description, issuer, custodian, transferAgent, totalSupply, availableSupply, pricePerToken, minimumInvestment, managementFee, assetClass, subAssetClass, riskRating, expectedYield, historicalReturns, volatility, sharpeRatio, complianceLevel, jurisdiction, regulatoryApproval, status, listingDate, maturityDate?, dividendFrequency, authorizedInvestors, observers, confidentialData, createdAt, updatedAt.
- AssetPurchaseRequestPayload: requestId, asset (ContractId), investor, numberOfTokens, totalAmount, paymentMethod, kycLevel, accreditedInvestor, investorCountry, sourceOfFunds, privacyLevel, zkProofRequired, requestDate, expiryDate.
- AssetHoldingPayload: holdingId, asset (ContractId), investor, tokensOwned, averageCostBasis, currentMarketValue, unrealizedGainLoss, purchaseDate, purchasePrice, transactionHash, blockNumber, votingRights, dividendRights, transferRights, complianceStatus, taxReporting, holdingPeriod, auditTrail, lastActivity.

**Методы:**
- createInstitutionalAsset(assetData)
- createPurchaseRequest(assetContractId, investor, numberOfTokens, paymentData)
- (при наличии) approvePurchase(contractId, approver) — exercise choice.

### 3.5 Compliance (complianceService.ts)

**Типы:** KYCVerification, KYCDocument, AMLCheck, AMLAlert, SanctionsCheck, ComplianceAuditEntry, ComplianceConfig (kycProvider, amlProvider, sanctionsProvider).

**Методы:** startKYCVerification(userId, personalInfo, targetLevel), validateTransaction(investor, amount, assetType, walletAddress) → { compliant, reasons }.

**API:** POST `/api/defi/compliance/kyc` — startKYCVerification; GET `/api/defi/compliance/kyc?userId=|verificationId=` — получение проверок.

### 3.6 Oracle (oracleService.ts)

**Типы:** PriceData, ExchangeRate, InterestRate, TreasuryYield (maturity: 1M|3M|6M|1Y|2Y|5Y|10Y|30Y, yield, timestamp, source), MarketIndex, PropertyValuation.

**Методы:** getPrice(symbol), getPrices(symbols), getTreasuryYield(maturity), getAllTreasuryYields(), getPropertyValuation(propertyId) (опционально).

**API:** GET `/api/defi/oracle/prices?symbol=|symbols=`, GET `/api/defi/oracle/treasury-yields?maturity=`.

### 3.7 Auth (authService.ts, API)

**API routes:** POST login, register, logout — Supabase + JWT, session/refresh cookies. Для SDK напрямую не требуется, но учёт partyId/user при подписании команд — да (act_as, read_as).

### 3.8 Canton Network Client (realCantonIntegration.ts)

**Конфиг:** participantHost, participantPort (7575), participantId, ledgerApiUrl (http://host:port/api/v1), ledgerWsUrl (ws://host:port/ws), authToken, domainId, requestTimeout, maxRetries.

**Методы:** connect(), getInstitutionalAssets(), getUserPortfolio(investorId), investInAsset(assetId, amount, investorId), subscribeToUpdates(callback).  
Реальные вызовы: fetch(ledgerApiUrl/assets/institutional), fetch(ledgerApiUrl/portfolios/{investorId}). investInAsset в коде пока симуляция (TODO: real Daml contract execution).

---

## PART 4 — SDK CURRENT STATE & GAPS

### 4.1 Что есть в SDK (cantonnet-omnichain-sdk)

- **canton-core:** types (Commands, CreateCommand, ExerciseCommand, DamlValue, DamlRecord, Identifier, ContractId, PartyId, LedgerOffset), error (SdkError, SdkResult), config (базовый).
- **canton-ledger-api:** LedgerClient::connect(endpoint, ledger_id), get_ledger_end(), submit(proto Commands). Proto: CommandSubmissionService, StateService (GetLedgerEnd). Нет: VersionService, IdentityService, ActiveContractsService, CommandCompletionService, EventQueryService (стримы).
- **canton-wallet:** Wallet trait, party_id_for_canton(), sign_for_canton(), canton_party_id_from_fingerprint, KeyStore.
- **canton-crypto:** KeyStore, KeyFingerprint, keys.
- **canton-transport, canton-reliability, canton-observability:** заглушки или минимальная реализация.

**Конфиг (example.yaml):** ledger_api: grpc_host, grpc_port, http_url, tls. Нет загрузки в LedgerClient из конфига (endpoint собирается вручную).

### 4.2 Разрывы (что нужно для DeFi)

1. **Конфиг:** LedgerApiConfig из YAML → построение endpoint для LedgerClient; опционально VersionService для ledger_id.
2. **Конвертация:** canton_core::Commands (и вложенные CreateCommand, ExerciseCommand с DamlValue) → proto Commands (com.daml.ledger.api.v2); идентификаторы шаблонов (package_id, module_name, entity_name).
3. **Высокоуровневый API:** create_contract(template_id, create_arguments), exercise(contract_id, choice, choice_argument), submit_and_wait — поверх LedgerClient.
4. **Доменные сервисы:** Treasury (create_bill, list_bills, create_purchase_request, get_holdings, get_portfolio_summary); RealEstate (list_properties, create_purchase_request, get_holdings, governance); Privacy (create_vault, get_vaults, deposit/withdraw, proofs) — с типами, совместимыми с DeFi (или явный маппинг).
5. **Compliance/Oracle:** интерфейсы для validate_transaction и get_prices/get_treasury_yields (внешние вызовы или заглушки).
6. **Стримы (опционально):** ActiveContractsService, CompletionStream, UpdateStream — для real-time UI; можно отложить на вторую итерацию.

---

## PART 5 — ARCHITECTURE DESIGN

### 5.1 Слои (сверху вниз)

1. **Public API (CantonClient / DeFiClient)**  
   Единая точка входа: connect_from_config(), treasury(), real_estate(), privacy(), ledger(). Методы доменных сервисов возвращают типы, совместимые с контрактами и фронтом.

2. **Domain layer**  
   - TreasuryService: create_bill, get_bill, list_bills, create_purchase_request, approve_purchase_request, get_holdings, get_portfolio_summary.  
   - RealEstateService: list_properties, get_property, create_purchase_request, get_holdings, get_governance_proposals, vote_on_proposal.  
   - PrivacyVaultService: create_vault, list_vaults, get_vault_assets, deposit_asset, withdraw_asset, generate_proof.  
   Все используют LedgerClient + конвертеры (domain types ↔ Daml records).

3. **Ledger API layer**  
   - LedgerClient (существующий): connect, get_ledger_end, submit(proto).  
   - Добавить: create_contract(template_id, record), exercise(contract_id, choice, argument) — с конвертацией canton_core::types в proto.  
   - Опционально: get_active_contracts(filter), stream_completions, stream_updates.

4. **Conversion layer**  
   - Domain → DamlRecord (Rust structs → DamlValue/Record для полей InstitutionalAssetPayload, AssetPurchaseRequestPayload, AssetHoldingPayload).  
   - DamlRecord/ContractId → Domain (парсинг из GetActiveContractsResponse или событий).

5. **Transport & config**  
   - LedgerApiConfig (grpc_host, grpc_port, http_url, tls, timeouts).  
   - ChannelBuilder с TLS при tls: true.  
   - Загрузка из config/example.yaml или config/local.yaml.

### 5.2 Размещение по крейтам

- **canton-core:** LedgerApiConfig, доменные типы (TreasuryBill, TreasuryBillHolding, PurchaseRequest, PropertyInfo, PrivacyVault, …) — без зависимостей от ledger-api.
- **canton-ledger-api:** LedgerClient, conversion (canton_core::Commands → proto), высокоуровневые create_contract, exercise; опционально VersionService, ActiveContractsService.
- **canton-defi** (новый крейт) или **canton-sdk:** TreasuryService, RealEstateService, PrivacyVaultService — зависят от canton-ledger-api и canton-core. Либо всё в canton-ledger-api как модули (services/treasury.rs и т.д.).
- **canton-transport:** ChannelBuilder, TLS, таймауты.  
- **canton-reliability:** retry, circuit_breaker — используются внутри LedgerClient или доменных сервисов.

### 5.3 Идентификаторы шаблонов

Из DeFi используются полные имена шаблонов:
- `InstitutionalAsset:InstitutionalAsset` → package_id, module_name = "InstitutionalAsset", entity_name = "InstitutionalAsset".
- `InstitutionalAsset:AssetPurchaseRequest`, `InstitutionalAsset:AssetHolding`, `InstitutionalAsset:DividendDistribution`.

Package_id берётся из participant (PackageService) или из конфига (если известен для DevNet). В конфиге задать: template_ids.institutional_asset, template_ids.asset_purchase_request, template_ids.asset_holding.

---

## PART 6 — API SURFACE SPECIFICATION (SDK)

### 6.1 Config

```yaml
# config/example.yaml (расширение)
ledger_api:
  grpc_host: "65.108.15.30"
  grpc_port: 30501
  http_url: "http://65.108.15.30:30757"
  tls: false
  connect_timeout: "10s"
  request_timeout: "30s"

template_ids:
  institutional_asset: "InstitutionalAsset:InstitutionalAsset"
  asset_purchase_request: "InstitutionalAsset:AssetPurchaseRequest"
  asset_holding: "InstitutionalAsset:AssetHolding"
```

Rust: struct LedgerApiConfig { grpc_host, grpc_port, http_url, tls, connect_timeout, request_timeout }. struct TemplateIds { institutional_asset, asset_purchase_request, asset_holding }.

### 6.2 LedgerClient (расширение)

- `connect_from_config(config: &LedgerApiConfig) -> SdkResult<Self>` — строит endpoint = format!("http://{}:{}", config.grpc_host, config.grpc_port), вызывает connect(endpoint, ledger_id). ledger_id — из конфига или VersionService::get_ledger_identity().
- `submit_domain_commands(&mut self, commands: &canton_core::Commands) -> SdkResult<()>` — конвертирует Commands в proto, вызывает submit.
- `create_contract(&mut self, template_id: &str, create_arguments: DamlRecord) -> SdkResult<ContractId>` — формирует CreateCommand, submit, возвращает contract_id из результата (при наличии CompletionStream или синхронного ответа; иначе нужен get_ledger_end + poll или submit_and_wait).
- `exercise(&mut self, contract_id: &str, choice: &str, choice_argument: DamlValue) -> SdkResult<()>` — ExerciseCommand, submit.

Примечание: стандартный Ledger API v2 — асинхронный submit; для submit_and_wait нужен CommandCompletionService (SubscribeCompletion) или эквивалент. В первом приближении: submit + возврат command_id; клиент может опционально ждать по CompletionStream.

### 6.3 TreasuryService (Rust)

Типы (в canton-core или canton-defi):
- TreasuryBill, TreasuryBillHolding, PurchaseRequest (поля как в 3.1).
- CreateBillInput, CreatePurchaseRequestInput (минимальный набор полей).

Методы:
- `create_bill(&mut self, input: CreateBillInput) -> SdkResult<TreasuryBill>` — строит InstitutionalAssetPayload (asset_class = FIXED_INCOME, sub_asset_class = TREASURY_BILL), create_contract, маппит в TreasuryBill.
- `get_bill(&self, bill_id: &str) -> SdkResult<Option<TreasuryBill>>` — через get_active_contracts по template_id + фильтр по assetId (или ключу).
- `list_bills(&self, status_filter: Option<BillStatus>) -> SdkResult<Vec<TreasuryBill>>`.
- `create_purchase_request(&mut self, bill_id: &str, investor: &str, number_of_tokens: u64, payment_data: PaymentData) -> SdkResult<PurchaseRequest>` — валидация (min/max, availability), создание контракта AssetPurchaseRequest.
- `approve_purchase_request(&mut self, request_id: &str, approver: &str) -> SdkResult<TreasuryBillHolding>` — exercise choice Approve на контракте запроса, создание/обновление AssetHolding.
- `get_holdings(&self, investor: &str) -> SdkResult<Vec<TreasuryBillHolding>>` — get_active_contracts AssetHolding по investor.
- `get_portfolio_summary(&self, investor: &str) -> SdkResult<PortfolioSummary>` — агрегат по get_holdings (total_value, total_invested, yield_earned, unrealized_gains).

### 6.4 RealEstateService (Rust)

- list_properties(filter?), get_property(property_id), create_purchase_request(property_id, investor, number_of_tokens, payment_data), get_holdings(investor), get_governance_proposals(property_id?), vote_on_proposal(proposal_id, voter, support).  
Типы: PropertyInfo (упрощённый для SDK), TokenPurchaseRequest, TokenPurchaseResult, GovernanceProposal.  
Реализация через те же шаблоны InstitutionalAsset (asset_class = REAL_ESTATE) и при необходимости отдельные шаблоны для предложений/голосов, если они есть в Daml.

### 6.5 PrivacyVaultService (Rust)

- create_vault(params), list_vaults(owner), get_vault_assets(vault_id), deposit_asset, withdraw_asset, generate_proof(params).  
Типы: PrivacyVault, PrivateAsset, CreateVaultParams, ComplianceProof.  
Реализация может быть частично off-ledger (шифрование, ZK) с привязкой к контрактам на Canton при наличии соответствующих шаблонов в Daml.

### 6.6 Compliance & Oracle (интерфейсы)

- ComplianceProvider: validate_transaction(investor, amount, asset_type, wallet_address) -> ValidationResult.
- OracleProvider: get_price(symbol), get_prices(symbols), get_treasury_yield(maturity), get_all_treasury_yields().  
Реализации: HTTP-клиенты к внешним API (Sumsub, Pyth, и т.д.) или заглушки для тестов. В DeFi эти вызовы уже есть в TypeScript; в SDK достаточно трейтов и одной-двух реализаций (mock + optional real).

---

## PART 7 — IMPLEMENTATION PROMPT CHAINS (DETAILED)

### Chain 1 — Config & Transport

**Input:** Part 2, 3 (paths), Part 5.1–5.2, 6.1.

**Tasks:**
1. В canton-core добавить LedgerApiConfig (grpc_host, grpc_port, http_url, tls, connect_timeout, request_timeout). Десериализация из YAML (serde).
2. В canton-ledger-api добавить fn connect_from_config(config: &LedgerApiConfig) -> SdkResult<LedgerClient>. Endpoint = format!("http://{}:{}", config.grpc_host, config.grpc_port). ledger_id пока из конфига (например "participant" или из env).
3. Обновить config/example.yaml: добавить connect_timeout, request_timeout при необходимости.
4. (Опционально) canton-transport: ChannelBuilder с timeout из config; использовать в LedgerClient::connect.

**Output:** Патчи для canton-core/config.rs, canton-ledger-api/client.rs, config/example.yaml. Тест: connect_from_config к 65.108.15.30:30501, get_ledger_end() возвращает offset.

**Acceptance:** cargo test, connect to DevNet participant from CI or local.

---

### Chain 2 — Ledger client & conversion

**Input:** Output Chain 1, Part 4.2, 5.1, 6.2, canton_core::types::command, value, identifier.

**Tasks:**
1. Реализовать conversion: canton_core::Commands → proto com.daml.ledger.api.v2.Commands. Для каждого CreateCommand: template_id (Identifier → proto), create_arguments (DamlRecord → proto Record). Для ExerciseCommand: contract_id, choice, choice_argument (DamlValue → proto Value).
2. Реализовать Identifier → proto (package_id, module_name, entity_name). Для шаблона "InstitutionalAsset:InstitutionalAsset" разбор строки или константы.
3. Реализовать DamlValue → proto Value (рекурсивно: Record, Variant, List, Optional, Text, Int64, Numeric, Party, ContractId, Timestamp, Date и т.д.). Использовать generated код proto.
4. Добавить в LedgerClient: submit_domain_commands(&mut self, commands: &canton_core::Commands). Внутри: to_proto_commands(commands)?, submit(proto_commands).await.
5. Добавить хелперы: build_create_command(template_id, record) -> Commands с одним Create; build_exercise_command(contract_id, choice, argument) -> Commands с одним Exercise. submit_domain_commands после построения.

**Output:** Новый модуль conversion.rs в canton-ledger-api, расширение client.rs. Юнит-тесты на конвертацию (roundtrip или сравнение с эталонным proto JSON).

**Acceptance:** submit CreateCommand для InstitutionalAsset с минимальным набором полей к DevNet; контракт появляется в ledger (проверка через get_active_contracts или UI).

---

### Chain 3 — Domain services (Treasury)

**Input:** Output Chain 2, Part 3.1, 6.3.

**Tasks:**
1. В canton-core (или canton-defi) определить типы: TreasuryBill, TreasuryBillHolding, PurchaseRequest, CreateBillInput, PaymentData, PortfolioSummary. Поля как в 3.1 (Rust-эквиваленты: String, Decimal, Option<ContractId>).
2. Реализовать маппинг TreasuryBill ↔ DamlRecord (InstitutionalAsset payload). Функции to_daml_record(bill) -> DamlRecord, from_contract_payload(record) -> TreasuryBill.
3. Создать TreasuryService { ledger_client, template_ids, config }. Методы: create_bill, get_bill (через get_active_contracts по template + filter), list_bills, create_purchase_request (build AssetPurchaseRequest payload, create_contract), approve_purchase_request (exercise), get_holdings, get_portfolio_summary.
4. get_active_contracts: если в LedgerClient ещё нет — добавить вызов ActiveContractsService.GetActiveContracts (по TransactionFilter по template_id и party). Парсинг ответа в Vec<TreasuryBill> или Vec<Contract>.
5. Интеграционный тест: create_bill → create_purchase_request → approve_purchase_request → get_holdings; сравнить с ожидаемыми значениями.

**Output:** Файлы treasury.rs, types/treasury.rs, конвертеры, тесты.

**Acceptance:** Полный цикл Treasury на DevNet (или sandbox) совпадает с поведением DeFi TypeScript сервиса.

---

### Chain 4 — Domain services (Real Estate, Privacy)

**Input:** Output Chain 3, Part 3.2, 3.3, 6.4, 6.5.

**Tasks:**
1. RealEstateService: типы PropertyInfo (упрощённый), TokenPurchaseRequest, PropertyHolding, GovernanceProposal. Методы: list_properties (InstitutionalAsset с asset_class REAL_ESTATE), create_purchase_request, get_holdings. vote_on_proposal — если в Daml есть шаблон голосования, exercise; иначе заглушка.
2. PrivacyVaultService: типы PrivacyVault, CreateVaultParams. create_vault — при наличии Daml-шаблона vault: create_contract; иначе — сохранение состояния локально или через другой бэкенд (в DeFi часть логики в TypeScript + ZKProofService). list_vaults(owner), get_vault_assets — через контракты или заглушки.
3. Общий фасад DeFiClient { treasury(), real_estate(), privacy(), ledger() } возвращающий соответствующие сервисы.

**Output:** real_estate.rs, privacy_vault.rs, defi_client.rs, тесты.

**Acceptance:** Сигнатуры и поведение соответствуют useRealEstateService, usePrivacyVaultService и API routes.

---

### Chain 5 — Compliance & Oracle

**Input:** Output Chain 4, Part 3.5, 3.6, 6.6.

**Tasks:**
1. Трейт ComplianceProvider с методом validate_transaction. Реализация MockComplianceProvider (всегда compliant) для тестов. Опционально HttpComplianceProvider (вызов к внутреннему API или Sumsub).
2. Трейт OracleProvider: get_price, get_prices, get_treasury_yield, get_all_treasury_yields. MockOracleProvider с фиксированными ценами и yields. Опционально Pyth/Chainlink HTTP клиент.
3. В TreasuryService (и при необходимости RealEstate) инжектить ComplianceProvider и OracleProvider; перед create_purchase_request вызывать validate_transaction; при create_bill брать yield из oracle.

**Output:** compliance/mod.rs, oracle/mod.rs, интеграция в TreasuryService/RealEstateService.

**Acceptance:** В тестах с mock провайдерами создание purchase request и bill проходят; с failing compliance — ошибка.

---

### Chain 6 — Integration & documentation

**Input:** Все предыдущие выводы, Part 8.

**Tasks:**
1. Интеграционный тест end-to-end: config из example.yaml → connect → create_bill → create_purchase_request → approve → get_holdings → get_portfolio_summary. Сравнение с ожидаемыми числами.
2. Документ docs/DEFI_SDK_USAGE.md: как подключиться к DevNet, как вызвать treasury().create_bill(...), пример кода на Rust. Ссылка на DEFI_CONNECT_DEVNET.md для эндпоинтов.
3. README cantonnet-omnichain-sdk: раздел "DeFi platform support" со ссылками на DEFI_SDK_USAGE.md и этот промт.
4. Проверка acceptance criteria из Part 8; обновление DEVELOPMENT_PROMPT.md (прогресс фаз).

**Output:** Тесты, документация, обновлённый DEVELOPMENT_PROMPT.md.

**Acceptance:** Все пункты из Part 8 выполнены; новый разработчик может по документации подключиться к participant и выполнить сценарий Treasury.

---

## PART 8 — ACCEPTANCE CRITERIA & TESTING

### 8.1 Критерии приёмки (общие)

- [ ] LedgerClient подключается к DevNet participant (65.108.15.30:30501) по конфигу из YAML.
- [ ] Конвертация canton_core::Commands в proto Commands корректна для Create и Exercise; submit успешен.
- [ ] TreasuryService: create_bill, list_bills, create_purchase_request, get_holdings, get_portfolio_summary работают против реального participant (или sandbox) с теми же шаблонами, что и DeFi.
- [ ] Типы TreasuryBill, TreasuryBillHolding, PurchaseRequest в SDK соответствуют по полям типам из treasuryBillsService.ts (с учётом Rust-эквивалентов).
- [ ] Реализованы интерфейсы ComplianceProvider и OracleProvider; интеграция в TreasuryService (и при необходимости RealEstate) проверена тестами.
- [ ] Документация (DEFI_SDK_USAGE.md) описывает подключение и пример вызова Treasury.
- [ ] cargo test --workspace и cargo clippy --workspace -- -D warnings проходят.
- [ ] Нет регрессий в существующих тестах canton-ledger-api и canton-core.

### 8.2 Тесты (минимальный набор)

- Юнит: conversion Commands → proto (несколько шаблонов полей).
- Юнит: DamlValue → proto Value (Record, List, Optional, Text, Numeric).
- Юнит: TreasuryService create_bill/build_payload → ожидаемый DamlRecord.
- Интеграция (feature = "integration" или env CANTON_LEDGER_GRPC): connect, get_ledger_end, submit Create InstitutionalAsset, GetActiveContracts по фильтру.
- Интеграция: полный цикл Treasury (create_bill → create_purchase_request → exercise Approve → get_holdings).

### 8.3 Риски и откат

- Изменение формата конфига: обратная совместимость (старые ключи без connect_timeout остаются валидными).
- Новая зависимость (serde_yaml): добавить в Cargo.toml canton-core. При откате — оставить только ручной LedgerClient::connect(endpoint, ledger_id).
- Конвертеры proto: при обновлении proto-файлов Canton перепроверить имена полей и типы.

---

## PART 9 — APPENDIX

### 9.1 File paths (quick reference)

| Area | Path |
|------|------|
| DeFi pages | src/app/defi/*.tsx |
| DeFi components | src/components/defi/*.tsx, treasury/*.tsx, realestate/*.tsx, privacy/*.tsx |
| DeFi API | src/app/api/defi/**/route.ts |
| Treasury service | src/lib/canton/services/treasuryBillsService.ts |
| RealEstate service | src/lib/canton/services/realEstateService.ts |
| Privacy service | src/lib/canton/services/privacyVaultService.ts |
| Daml integration | src/lib/canton/services/damlIntegrationService.ts |
| Compliance | src/lib/canton/services/complianceService.ts |
| Oracle | src/lib/canton/services/oracleService.ts |
| Canton hooks | src/lib/canton/hooks/realCantonIntegration.ts, useTreasuryBills.ts, useRealEstate.ts, usePrivacyVaults.ts |
| SDK core | cantonnet-omnichain-sdk/crates/canton-core/src/*.rs |
| SDK ledger | cantonnet-omnichain-sdk/crates/canton-ledger-api/src/client.rs |
| SDK config | cantonnet-omnichain-sdk/config/example.yaml |
| DevNet doc | canton-otc/blockchain/DEFI_CONNECT_DEVNET.md, cantonnet-omnichain-sdk/docs/DEVNET_PARTICIPANT.md |

### 9.2 Template IDs (Daml)

- InstitutionalAsset: module "InstitutionalAsset", entity "InstitutionalAsset". Full: package_id:module_name:entity_name (package_id из participant).
- AssetPurchaseRequest: "InstitutionalAsset:AssetPurchaseRequest".
- AssetHolding: "InstitutionalAsset:AssetHolding".

### 9.3 Environment variables (DeFi → SDK)

Для SDK при запуске тестов или примеров:
- CANTON_LEDGER_GRPC=65.108.15.30:30501
- CANTON_LEDGER_HTTP=http://65.108.15.30:30757
- Или config/local.yaml с ledger_api.grpc_host, grpc_port.

### 9.4 Prompt chain checklist (for executor)

- [ ] Chain 1 done; LedgerClient::connect_from_config works.
- [ ] Chain 2 done; submit_domain_commands and create_contract/exercise work.
- [ ] Chain 3 done; TreasuryService implemented and tested.
- [ ] Chain 4 done; RealEstateService, PrivacyVaultService (or stubs) implemented.
- [ ] Chain 5 done; Compliance and Oracle interfaces integrated.
- [ ] Chain 6 done; e2e test and docs updated.

---

## PART 10 — EXPANDED REFERENCE (DeFi methods → SDK mapping)

### 10.1 treasuryBillsService.ts → TreasuryService (Rust)

| TypeScript method | Rust method | Notes |
|-------------------|-------------|--------|
| createTreasuryBill(billData) | create_bill(input: CreateBillInput) | Daml Create InstitutionalAsset (FIXED_INCOME, TREASURY_BILL) |
| getTreasuryBill(billId) | get_bill(bill_id: &str) -> Option<TreasuryBill> | GetActiveContracts by key/assetId |
| getAllTreasuryBills() | list_bills(status_filter?) | GetActiveContracts template InstitutionalAsset |
| getActiveTreasuryBills() | list_bills(Some(ACTIVE)) | filter status |
| updateTreasuryBill(billId, updates) | update_bill(bill_id, updates) | Exercise Update or re-create; зависит от Daml модели |
| createPurchaseRequest(billId, investor, num, payment) | create_purchase_request(bill_id, investor, num, payment_data) | Create AssetPurchaseRequest |
| approvePurchaseRequest(requestId, approver, txHash, block) | approve_purchase_request(request_id, approver) | Exercise Approve on request contract |
| getInvestorPortfolioSummary(investor) | get_portfolio_summary(investor) | Aggregate holdings |
| getUserHoldings(investor) | get_holdings(investor) | GetActiveContracts AssetHolding by investor |
| getAvailableBills() | list_bills(Some(ACTIVE)) | Same as getActiveTreasuryBills |

### 10.2 realEstateService.ts → RealEstateService (Rust)

| TypeScript | Rust | Notes |
|------------|------|--------|
| getAvailableProperties() | list_properties() | InstitutionalAsset REAL_ESTATE |
| getUserHoldings(address) | get_holdings(investor) | AssetHolding for investor |
| getGovernanceProposals(address) | get_governance_proposals(property_id?) | If Daml has governance template |
| purchaseTokens(params) | create_purchase_request + approve flow | Same as treasury flow |
| createPurchaseRequest(propertyId, investor, num, payment) | create_purchase_request(property_id, investor, num, payment) | |
| voteOnProposal(proposalId, voter, support) | vote_on_proposal(proposal_id, voter, support) | Exercise Vote choice if exists |

### 10.3 privacyVaultService.ts → PrivacyVaultService (Rust)

| TypeScript | Rust | Notes |
|------------|------|--------|
| createVault(params) | create_vault(params: CreateVaultParams) | Daml or off-ledger |
| getUserVaults(owner) | list_vaults(owner) | |
| getVaultAssets(owner) | get_vault_assets(vault_id) | |
| getComplianceProofs(owner) | get_compliance_proofs(owner) | |
| depositAsset(vaultId, type, amount, proof?) | deposit_asset(vault_id, asset_type, amount) | |
| generateProof(params) | generate_proof(params) | ZK layer; может вызывать внешний сервис |

### 10.4 damlIntegrationService.ts → LedgerClient + conversion

| TypeScript | Rust | Notes |
|------------|------|--------|
| create(templateId, payload) | create_contract(template_id, create_arguments: DamlRecord) | submit CreateCommand |
| exercise(contractId, choice, argument) | exercise(contract_id, choice, choice_argument: DamlValue) | submit ExerciseCommand |
| query(templateId, filter?) | get_active_contracts(filter) | ActiveContractsService |
| submitAndWait(commands) | submit_domain_commands + optional wait on CompletionStream | |

### 10.5 API routes → SDK usage (backend)

- GET /api/defi/treasury/portfolio?investor= → treasury_service.get_portfolio_summary(investor).
- GET /api/defi/treasury/bills → treasury_service.list_bills(None).
- POST /api/defi/treasury/bills → treasury_service.create_bill(body).
- GET /api/defi/treasury/bills/:id → treasury_service.get_bill(id).
- PUT /api/defi/treasury/bills/:id → treasury_service.update_bill(id, body).
- POST /api/defi/treasury/purchases → treasury_service.create_purchase_request(bill_id, investor, num, payment_data).
- Аналогично для realestate и privacy при наличии соответствующих API.

---

## PART 11 — PROMPT ENGINEERING NOTES (2025)

### 11.1 Chain-of-thought

При выполнении каждой цепи сначала кратко выведи план (1–2–3 шага), затем код/патчи, затем проверку. Это уменьшает ошибки и даёт возможность прервать на шаге.

### 11.2 Structured output

Для кода придерживайся формата: [FILE path] → [CHANGE description] → [code block]. Для тестов: [TEST name] → [criterion] → [code block].

### 11.3 Context window

Если контекст обрезается, приоритет: Part 3 (DeFi inventory), Part 6 (API surface), Part 7 (Chain для текущей фазы). Остальное — по ссылкам на секции.

### 11.4 Iteration

После каждой цепи — запуск тестов и линтера. При падении — исправление в той же цепи перед переходом к следующей.

---

---

## PART 12 — DETAILED TYPE DEFINITIONS (TypeScript → Rust)

### 12.1 TreasuryBill (full field mapping)

| TypeScript (treasuryBillsService.ts) | Rust (proposed) | Proto / Daml |
|--------------------------------------|-----------------|--------------|
| billId: string | bill_id: String | assetId in payload |
| name: string | name: String | name |
| symbol: string | symbol: String | symbol |
| description: string | description: String | description |
| issuer: string | issuer: String | issuer |
| custodian: string | custodian: String | custodian |
| maturity: '1M'\|'3M'\|... | maturity: TreasuryMaturity (enum) | - |
| maturityDate: string (ISO) | maturity_date: DateTime<Utc> | maturityDate |
| issueDate: string | issue_date: DateTime<Utc> | issueDate |
| totalSupply: string | total_supply: Decimal | totalSupply |
| availableSupply: string | available_supply: Decimal | availableSupply |
| pricePerToken: string | price_per_token: Decimal | pricePerToken |
| minimumInvestment: string | minimum_investment: Decimal | minimumInvestment |
| currentYield: string | current_yield: Decimal | - |
| expectedYield: string | expected_yield: Decimal | expectedYield |
| yieldToMaturity: string | yield_to_maturity: Decimal | - |
| status: ACTIVE\|... | status: BillStatus (enum) | status |
| contractId?: ContractId | contract_id: Option<ContractId> | - |
| createdAt, updatedAt | created_at, updated_at: DateTime<Utc> | createdAt, updatedAt |

### 12.2 TreasuryBillHolding (full field mapping)

| TypeScript | Rust | Notes |
|------------|------|--------|
| holdingId | holding_id: String | |
| billId | bill_id: String | |
| investor | investor: String (PartyId) | |
| tokensOwned | tokens_owned: Decimal | |
| averageCostBasis | average_cost_basis: Decimal | |
| currentMarketValue | current_market_value: Decimal | |
| unrealizedGainLoss | unrealized_gain_loss: Decimal | |
| unrealizedGainLossPercent | unrealized_gain_loss_percent: Decimal | |
| purchaseDate | purchase_date: DateTime<Utc> | |
| purchasePrice | purchase_price: Decimal | |
| accumulatedYield | accumulated_yield: Decimal | |
| lastYieldDistribution | last_yield_distribution: DateTime<Utc> | |
| status: ACTIVE\|SOLD\|MATURED | status: HoldingStatus (enum) | |
| contractId? | contract_id: Option<ContractId> | |

### 12.3 PurchaseRequest (full field mapping)

| TypeScript | Rust | Notes |
|------------|------|--------|
| requestId | request_id: String | |
| billId | bill_id: String | |
| investor | investor: String | |
| numberOfTokens | number_of_tokens: u64 | |
| totalAmount | total_amount: Decimal | |
| paymentMethod | payment_method: String | |
| status | status: RequestStatus (enum) | |
| kycLevel | kyc_level: String | |
| complianceCheck | compliance_check: ComplianceResult | |
| requestDate, expiryDate | request_date, expiry_date: DateTime<Utc> | |
| completedAt? | completed_at: Option<DateTime<Utc>> | |
| contractId? | contract_id: Option<ContractId> | |

### 12.4 InstitutionalAssetPayload (Daml) — fields for create_contract

Все поля из damlIntegrationService.ts InstitutionalAssetPayload должны быть представлены в Rust структуре для сериализации в DamlRecord: assetId, name, symbol, description, issuer, custodian, transferAgent, totalSupply, availableSupply, pricePerToken, minimumInvestment, managementFee, assetClass (enum), subAssetClass, riskRating, expectedYield, historicalReturns (Vec<String>), volatility, sharpeRatio, complianceLevel, jurisdiction (Vec<String>), regulatoryApproval, reportingRequirements, status, listingDate, maturityDate?, dividendFrequency, nextDividendDate?, authorizedInvestors, observers, confidentialData, createdAt, updatedAt.

### 12.5 AssetPurchaseRequestPayload (Daml)

requestId, asset (ContractId<InstitutionalAssetPayload>), investor, numberOfTokens, totalAmount, paymentMethod, kycLevel, accreditedInvestor, investorCountry, sourceOfFunds, privacyLevel, zkProofRequired, requestDate, expiryDate.

### 12.6 AssetHoldingPayload (Daml)

holdingId, asset (ContractId), investor, tokensOwned, averageCostBasis, currentMarketValue, unrealizedGainLoss, purchaseDate, purchasePrice, transactionHash, blockNumber, votingRights, dividendRights, transferRights, complianceStatus, taxReporting, holdingPeriod, auditTrail, lastActivity.

---

## PART 13 — PROMPT CHAIN SUB-STEPS (granular)

### Chain 1 — Sub-steps

1.1. Добавить в canton-core Cargo.toml: serde = { version = "...", features = ["derive"] }, serde_yaml.  
1.2. Создать config.rs: #[derive(Deserialize)] struct LedgerApiConfig { grpc_host: String, grpc_port: u16, http_url: Option<String>, tls: Option<bool>, connect_timeout_secs: Option<u64>, request_timeout_secs: Option<u64> }.  
1.3. Функция load_ledger_api_config(path: &Path) -> SdkResult<LedgerApiConfig>: открыть файл, serde_yaml::from_reader.  
1.4. В canton-ledger-api добавить pub fn connect_from_config(config: &LedgerApiConfig) -> SdkResult<LedgerClient>. Endpoint = format!("http://{}:{}", config.grpc_host, config.grpc_port). Ledger_id: из конфига (поле ledger_id: Option<String>) или "participant".  
1.5. Тест: создать config/example.yaml с grpc_host: "65.108.15.30", grpc_port: 30501; load_ledger_api_config; connect_from_config; get_ledger_end(); assert offset.

### Chain 2 — Sub-steps

2.1. В canton-ledger-api создать conversion/mod.rs.  
2.2. Реализовать fn identifier_to_proto(id: &Identifier) -> proto::Identifier. Identifier в canton_core содержит package_id, module_name, entity_name (или одну строку "Module:Entity"; парсить).  
2.3. Реализовать fn daml_value_to_proto(v: &DamlValue) -> SdkResult<proto::Value>. Обработать Unit, Bool, Int64, Numeric, Text, Timestamp, Date, Party, ContractId, List, Optional, Record, Variant, Enum.  
2.4. Реализовать fn daml_record_to_proto_record(r: &DamlRecord) -> SdkResult<proto::Record>.  
2.5. Реализовать fn create_command_to_proto(c: &CreateCommand) -> SdkResult<proto::Command>.  
2.6. Реализовать fn exercise_command_to_proto(e: &ExerciseCommand) -> SdkResult<proto::Command>.  
2.7. Реализовать fn commands_to_proto(c: &Commands) -> SdkResult<proto::Commands>. Для каждого command в c.commands маппить в proto Command (create/exercise).  
2.8. В LedgerClient добавить pub async fn submit_domain_commands(&mut self, commands: &canton_core::Commands) -> SdkResult<()>. let proto_commands = commands_to_proto(commands)?; self.submit(proto_commands).await.  
2.9. Юнит-тест: build Commands с одним CreateCommand (template_id = "InstitutionalAsset:InstitutionalAsset", create_arguments = minimal record); commands_to_proto; проверить что proto содержит create_arguments.  
2.10. Интеграционный тест: connect to DevNet; build minimal CreateCommand for InstitutionalAsset; submit_domain_commands; (опционально) get_active_contracts и проверить появление контракта.

### Chain 3 — Sub-steps

3.1. В canton-core (или canton-defi) types/treasury.rs: struct TreasuryBill { ... } с полями из 12.1.  
3.2. struct TreasuryBillHolding { ... }, struct PurchaseRequest { ... }, struct CreateBillInput { ... }, struct PaymentData { ... }, struct PortfolioSummary { ... }.  
3.3. Маппинг: fn institutional_asset_payload_to_treasury_bill(record: &DamlRecord) -> SdkResult<TreasuryBill>. Извлечь поля по именам (name, symbol, totalSupply, ...).  
3.4. Маппинг: fn treasury_bill_to_create_arguments(bill: &CreateBillInput) -> DamlRecord.  
3.5. TreasuryService::new(ledger_client, template_ids) где template_ids содержит полные имена шаблонов.  
3.6. create_bill: build CreateBillInput -> DamlRecord (InstitutionalAsset payload), build CreateCommand, submit_domain_commands. Получить contract_id: либо из ответа (если есть submit_and_wait с возвратом), либо через get_active_contracts по workflow_id/application_id.  
3.7. get_bill: GetActiveContracts с filter по template_id и по party (read_as или act_as). По контракту извлечь payload -> TreasuryBill.  
3.8. list_bills: GetActiveContracts по template InstitutionalAsset, фильтр по asset_class = FIXED_INCOME (или по sub_asset_class TREASURY_BILL).  
3.9. create_purchase_request: получить контракт bill по bill_id; build AssetPurchaseRequest payload; CreateCommand; submit.  
3.10. approve_purchase_request: ExerciseCommand по контракту запроса, choice "Approve", argument (approver, tx_hash, block_number или минимальный).  
3.11. get_holdings: GetActiveContracts по template AssetHolding, filter по investor.  
3.12. get_portfolio_summary: get_holdings; агрегировать total_value, total_invested, yield_earned, unrealized_gains.  
3.13. Интеграционный тест: create_bill -> list_bills -> create_purchase_request -> approve_purchase_request -> get_holdings -> get_portfolio_summary.

### Chain 4 — Sub-steps

4.1. RealEstateService: types PropertyInfo (упрощённый), TokenPurchaseRequest, PropertyHolding.  
4.2. list_properties: GetActiveContracts InstitutionalAsset с asset_class = REAL_ESTATE.  
4.3. create_purchase_request: аналогично Treasury с asset = contract_id недвижимости.  
4.4. get_holdings: GetActiveContracts AssetHolding по investor; фильтр по asset (property).  
4.5. vote_on_proposal: если в Daml есть шаблон Proposal и choice Vote(proposal_id, voter, support) — exercise; иначе unimplemented! или заглушка.  
4.6. PrivacyVaultService: create_vault — если есть Daml Vault template — create_contract; иначе возвращать локальный id и хранить в памяти/БД для тестов.  
4.7. list_vaults(owner): GetActiveContracts по Vault template и owner.  
4.8. get_vault_assets: по vault_id получить контракты активов (если есть шаблон VaultAsset).  
4.9. DeFiClient struct { ledger: LedgerClient, treasury: TreasuryService, real_estate: RealEstateService, privacy: PrivacyVaultService }. Методы treasury(), real_estate(), privacy() возвращают ссылки.

### Chain 5 — Sub-steps

5.1. Трейт ComplianceProvider: fn validate_transaction(&self, investor: &str, amount: &Decimal, asset_type: &str, wallet: Option<&str>) -> SdkResult<ValidationResult>. ValidationResult { compliant: bool, reasons: Vec<String> }.  
5.2. MockComplianceProvider: всегда ValidationResult { compliant: true, reasons: vec![] }.  
5.3. TreasuryService::new(..., compliance: Arc<dyn ComplianceProvider>, oracle: Arc<dyn OracleProvider>). В create_purchase_request вызвать compliance.validate_transaction; при !compliant вернуть SdkError.  
5.4. Трейт OracleProvider: get_price(symbol: &str), get_prices(symbols: &[String]), get_treasury_yield(maturity: &str), get_all_treasury_yields().  
5.5. MockOracleProvider: get_treasury_yield возвращает фиксированный yield по maturity (например 4.5 для "1Y").  
5.6. В create_bill использовать oracle.get_treasury_yield(maturity) для current_yield, expected_yield.

### Chain 6 — Sub-steps

6.1. Интеграционный тест e2e: загрузка config, connect_from_config, создание TreasuryService с mock compliance и oracle, create_bill, create_purchase_request, approve_purchase_request, get_holdings, get_portfolio_summary; assert portfolio total_value > 0.  
6.2. docs/DEFI_SDK_USAGE.md: заголовок "Using Canton SDK for DeFi Backend"; пример на Rust: use canton_ledger_api::LedgerClient; use canton_defi::TreasuryService; let config = load_config("config/example.yaml"); let mut ledger = LedgerClient::connect_from_config(&config.ledger_api)?; let treasury = TreasuryService::new(&mut ledger, &config.template_ids, compliance, oracle); let bill = treasury.create_bill(CreateBillInput { ... })?;  
6.3. В README.md cantonnet-omnichain-sdk добавить секцию "DeFi platform (canton-otc)": ссылка на DEFI_SDK_USAGE.md и на этот промт (blockchain/prompts/DEFI_SDK_MASTER_PROMPT.md).  
6.4. DEVELOPMENT_PROMPT.md: обновить таблицу прогресса — Phase 1–4 и добавить Phase 5 "DeFi domain services", Phase 6 "Compliance & Oracle".

---

## PART 14 — CODE SNIPPETS (reference)

### 14.1 Minimal CreateCommand (Rust)

```rust
use canton_core::types::{Commands, Command, CreateCommand, DamlRecord, Identifier};

fn minimal_create_institutional_asset(bill_id: &str, name: &str, symbol: &str) -> Commands {
    let template_id = Identifier::new(
        None,
        "InstitutionalAsset",
        "InstitutionalAsset"
    );
    let create_args = DamlRecord {
        record_id: None,
        fields: vec![
            RecordField::new("assetId", DamlValue::text(bill_id)),
            RecordField::new("name", DamlValue::text(name)),
            RecordField::new("symbol", DamlValue::text(symbol)),
            RecordField::new("totalSupply", DamlValue::text("1000000")),
            RecordField::new("availableSupply", DamlValue::text("1000000")),
            RecordField::new("pricePerToken", DamlValue::text("100")),
            RecordField::new("minimumInvestment", DamlValue::text("100")),
            // ... остальные обязательные поля
        ],
    };
    let create = CreateCommand { template_id, create_arguments: create_args };
    Commands {
        ledger_id: Some("participant".into()),
        workflow_id: format!("treasury-bill-{}", bill_id),
        application_id: "canton-defi-sdk".into(),
        command_id: uuid::Uuid::new_v4().to_string(),
        act_as: vec!["party1".into()],
        read_as: vec![],
        commands: vec![Command::Create(create)],
        min_ledger_time_abs: None,
        min_ledger_time_rel: None,
        deduplication_period: None,
        submission_id: None,
    }
}
```

### 14.2 Load config and connect (Rust)

```rust
use std::path::Path;
use canton_ledger_api::LedgerClient;

fn connect_devnet() -> SdkResult<LedgerClient> {
    let config_path = Path::new("config/example.yaml");
    let config = load_ledger_api_config(config_path)?;
    LedgerClient::connect_from_config(&config)
}
```

### 14.3 TreasuryService create_bill (pseudo)

```rust
pub async fn create_bill(&mut self, input: CreateBillInput) -> SdkResult<TreasuryBill> {
    let yield_rate = self.oracle.get_treasury_yield(&input.maturity).await?;
    let payload = self.build_institutional_asset_payload(&input, &yield_rate)?;
    let create_cmd = self.build_create_command(self.template_ids.institutional_asset.clone(), payload)?;
    self.ledger.submit_domain_commands(&create_cmd).await?;
    // Optionally fetch contract_id via get_active_contracts
    let bill = self.payload_to_treasury_bill(&input, None)?;
    Ok(bill)
}
```

---

## PART 15 — TEST MATRIX

| Test | Chain | Description | Pass criterion |
|------|-------|-------------|----------------|
| config_load | 1 | load_ledger_api_config("config/example.yaml") | Ok(LedgerApiConfig) |
| connect_from_config | 1 | LedgerClient::connect_from_config(&config) | Ok(LedgerClient) |
| get_ledger_end | 1 | client.get_ledger_end().await | Ok(LedgerOffset) |
| conversion_create_command | 2 | commands_to_proto(minimal_create_commands()) | Ok(proto::Commands) |
| conversion_exercise_command | 2 | commands_to_proto(exercise_commands()) | Ok(proto::Commands) |
| submit_create_institutional_asset | 2 | submit_domain_commands(create_commands) | Ok(()) |
| treasury_create_bill | 3 | treasury.create_bill(input) | Ok(TreasuryBill) |
| treasury_list_bills | 3 | treasury.list_bills(None) | Ok(Vec) with len >= 1 |
| treasury_create_purchase_request | 3 | treasury.create_purchase_request(...) | Ok(PurchaseRequest) |
| treasury_approve_and_holdings | 3 | approve; get_holdings(investor) | Ok(Vec) with new holding |
| treasury_portfolio_summary | 3 | get_portfolio_summary(investor) | Ok(PortfolioSummary), total_value > 0 |
| real_estate_list_properties | 4 | real_estate.list_properties() | Ok(Vec) or empty |
| privacy_create_vault | 4 | privacy.create_vault(params) | Ok(PrivacyVault) or stub |
| compliance_validate | 5 | MockComplianceProvider validate | compliant true |
| compliance_reject | 5 | FailingComplianceProvider validate | compliant false, error |
| oracle_get_yield | 5 | MockOracleProvider get_treasury_yield("1Y") | Ok(TreasuryYield) |
| e2e_treasury_flow | 6 | Full flow create_bill -> ... -> portfolio_summary | All Ok, portfolio non-empty |

---

## PART 16 — RISKS & MITIGATION

| Risk | Impact | Mitigation |
|------|--------|------------|
| Proto API change (Canton upgrade) | Conversion breaks | Pin proto versions; integration tests against fixed participant version |
| DevNet participant reset | Integration tests fail | Use feature gate "integration"; skip if CANTON_LEDGER_GRPC not set |
| Template IDs differ per env | Wrong template in submit | Load template_ids from config per environment |
| Daml model has no Approve choice | approve_purchase_request fails | Document required Daml choices; or implement two-step flow (create holding contract manually) |
| GetActiveContracts filter syntax | No contracts returned | Align filter with Canton Ledger API v2 TransactionFilter; test with grpcurl |
| Decimal precision (Rust vs TypeScript) | Rounding mismatch | Use rust_decimal::Decimal; same precision as JS Decimal (e.g. 18 digits) |

---

## PART 17 — GLOSSARY

- **Ledger API v2:** Canton/Daml gRPC API (StateService, CommandSubmissionService, ActiveContractsService, CommandCompletionService, etc.).
- **Participant:** Canton node exposing Ledger API (HTTP JSON on 7575, gRPC on 5001; у нас NodePort 30757, 30501).
- **Template:** Daml contract template (e.g. InstitutionalAsset:InstitutionalAsset).
- **ContractId:** Unique id of a contract instance on the ledger.
- **Choice:** Daml choice on a template (e.g. Approve on AssetPurchaseRequest).
- **act_as, read_as:** Parties under which commands are submitted (Ledger API Commands).
- **GetActiveContracts:** Ledger API call to stream active contracts matching a filter.
- **DeFi (canton-otc):** Next.js frontend + API routes + TypeScript services (treasury, realestate, privacy, daml, compliance, oracle).

---

## PART 18 — REPEATED CHECKLIST (for copy-paste)

- [ ] Chain 1: Config & Transport — LedgerApiConfig, connect_from_config, test connect
- [ ] Chain 2: Conversion — commands_to_proto, submit_domain_commands, create_contract/exercise helpers
- [ ] Chain 3: TreasuryService — create_bill, list_bills, create_purchase_request, approve, get_holdings, get_portfolio_summary
- [ ] Chain 4: RealEstateService, PrivacyVaultService, DeFiClient facade
- [ ] Chain 5: ComplianceProvider, OracleProvider; integration in TreasuryService
- [ ] Chain 6: E2E test, DEFI_SDK_USAGE.md, README update, DEVELOPMENT_PROMPT update
- [ ] cargo test --workspace; cargo clippy --workspace -- -D warnings
- [ ] No regression in existing ledger-api and core tests

---

## PART 19 — FULL DEFI TYPE REFERENCE (TypeScript excerpts)

Ниже приведены выдержки из реальных TypeScript-интерфейсов DeFi для точного маппинга в Rust. Источник: canton-otc/src/lib/canton/services/*.ts.

### 19.1 TreasuryBill (treasuryBillsService.ts)

```ts
export interface TreasuryBill {
  billId: string;
  name: string;
  symbol: string;
  description: string;
  issuer: string;
  custodian: string;
  maturity: TreasuryYield['maturity'];  // '1M'|'3M'|'6M'|'1Y'|'2Y'|'5Y'|'10Y'|'30Y'
  maturityDate: string;
  issueDate: string;
  totalSupply: string;
  availableSupply: string;
  pricePerToken: string;
  minimumInvestment: string;
  currentYield: string;
  expectedYield: string;
  yieldToMaturity: string;
  status: 'ACTIVE' | 'SUSPENDED' | 'MATURED' | 'DELISTED';
  contractId?: ContractId<any>;
  createdAt: string;
  updatedAt: string;
}
```

Rust: struct TreasuryBill { bill_id, name, symbol, description, issuer, custodian, maturity (enum), maturity_date (DateTime<Utc>), issue_date, total_supply (Decimal), available_supply, price_per_token, minimum_investment, current_yield, expected_yield, yield_to_maturity, status (enum), contract_id (Option<ContractId>), created_at, updated_at }.

### 19.2 TreasuryBillHolding (treasuryBillsService.ts)

```ts
export interface TreasuryBillHolding {
  holdingId: string;
  billId: string;
  investor: string;
  tokensOwned: string;
  averageCostBasis: string;
  currentMarketValue: string;
  unrealizedGainLoss: string;
  unrealizedGainLossPercent: string;
  purchaseDate: string;
  purchasePrice: string;
  accumulatedYield: string;
  lastYieldDistribution: string;
  status: 'ACTIVE' | 'SOLD' | 'MATURED';
  contractId?: ContractId<any>;
  createdAt: string;
  updatedAt: string;
}
```

### 19.3 PurchaseRequest (treasuryBillsService.ts)

```ts
export interface PurchaseRequest {
  requestId: string;
  billId: string;
  investor: string;
  numberOfTokens: string;
  totalAmount: string;
  paymentMethod: string;
  status: 'PENDING' | 'APPROVED' | 'REJECTED' | 'COMPLETED' | 'FAILED';
  kycLevel: string;
  complianceCheck: { passed: boolean; reasons: string[] };
  requestDate: string;
  expiryDate: string;
  completedAt?: string;
  contractId?: ContractId<any>;
  createdAt: string;
  updatedAt: string;
}
```

### 19.4 InstitutionalAssetPayload (damlIntegrationService.ts)

```ts
export interface InstitutionalAssetPayload {
  assetId: string;
  name: string;
  symbol: string;
  description: string;
  issuer: string;
  custodian: string;
  transferAgent: string;
  totalSupply: string;
  availableSupply: string;
  pricePerToken: string;
  minimumInvestment: string;
  managementFee: string;
  assetClass: 'EQUITY' | 'FIXED_INCOME' | 'REAL_ESTATE' | 'COMMODITIES' | 'ALTERNATIVES' | 'DERIVATIVES';
  subAssetClass: string;
  riskRating: 'AAA' | 'AA' | 'A' | 'BBB' | 'BB' | 'B' | 'CCC';
  expectedYield: string;
  historicalReturns: string[];
  volatility: string;
  sharpeRatio: string;
  complianceLevel: 'RETAIL' | 'ACCREDITED' | 'INSTITUTIONAL' | 'ULTRA_HNW';
  jurisdiction: string[];
  regulatoryApproval: string[];
  reportingRequirements: string[];
  status: 'ACTIVE' | 'SUSPENDED' | 'DELISTED' | 'UNDER_REVIEW';
  listingDate: string;
  maturityDate?: string;
  dividendFrequency: string;
  nextDividendDate?: string;
  authorizedInvestors: string[];
  observers: string[];
  confidentialData: string;
  createdAt: string;
  updatedAt: string;
}
```

### 19.5 AssetPurchaseRequestPayload (damlIntegrationService.ts)

```ts
export interface AssetPurchaseRequestPayload {
  requestId: string;
  asset: ContractId<InstitutionalAssetPayload>;
  investor: string;
  numberOfTokens: string;
  totalAmount: string;
  paymentMethod: string;
  kycLevel: 'RETAIL' | 'ACCREDITED' | 'INSTITUTIONAL' | 'ULTRA_HNW';
  accreditedInvestor: boolean;
  investorCountry: string;
  sourceOfFunds: string;
  privacyLevel: string;
  zkProofRequired: boolean;
  requestDate: string;
  expiryDate: string;
}
```

### 19.6 AssetHoldingPayload (damlIntegrationService.ts)

```ts
export interface AssetHoldingPayload {
  holdingId: string;
  asset: ContractId<InstitutionalAssetPayload>;
  investor: string;
  tokensOwned: string;
  averageCostBasis: string;
  currentMarketValue: string;
  unrealizedGainLoss: string;
  purchaseDate: string;
  purchasePrice: string;
  transactionHash: string;
  blockNumber: number;
  votingRights: boolean;
  dividendRights: boolean;
  transferRights: boolean;
  complianceStatus: string;
  taxReporting: boolean;
  holdingPeriod: number;
  auditTrail: string[];
  lastActivity: string;
}
```

### 19.7 PropertyInfo (realEstateService.ts) — key fields

id, name, address, type (PropertyType), subType (PropertySubType), totalValue (Decimal), tokenSupply, availableSupply, pricePerToken, minimumInvestment, expectedDividendYield, historicalReturns, occupancyRate, location (PropertyLocation), propertyManager (PropertyManager), legalStructure, jurisdiction, regulatoryStatus, complianceLevel, images (PropertyImage[]), documents (PropertyDocument[]), status (PropertyStatus), fundingProgress.

### 19.8 TokenPurchaseRequest (realEstateService.ts)

propertyId, investorAddress, numberOfTokens, totalAmount (Decimal), paymentMethod (PaymentMethod), kycLevel ('BASIC'|'ENHANCED'|'INSTITUTIONAL'), accreditedInvestor, investorCountry, privacyLevel ('STANDARD'|'ENHANCED'|'MAXIMUM'), zkProofRequired.

### 19.9 PrivacyVault (privacyVaultService.ts) — key fields

id, name, description, owner, custodian, authorizedViewers, trustees, privacyLevel ('STANDARD'|'ENHANCED'|'MAXIMUM'|'QUANTUM_SAFE'), encryptionStandard, zkProofProtocol, anonymitySet, complianceLevel, jurisdiction, totalValue (Decimal), assetCount, multiSigThreshold, timelock, status ('INITIALIZING'|'ACTIVE'|'LOCKED'|'UNDER_AUDIT'|'MIGRATING'|'DEPRECATED'), encryptedMetadata, metadataHash.

### 19.10 Compliance validateTransaction (complianceService.ts)

validateTransaction(investor: string, amount: string, assetType: string, walletAddress?: string): Promise<{ compliant: boolean; reasons: string[] }>.

### 19.11 Oracle getTreasuryYield (oracleService.ts)

getTreasuryYield(maturity: '1M'|'3M'|'6M'|'1Y'|'2Y'|'5Y'|'10Y'|'30Y'): Promise<TreasuryYield>. TreasuryYield { maturity, yield: string, timestamp, source }.

---

## PART 20 — PROMPT CHAIN EXECUTION TEMPLATE

При выполнении каждой цепи используй следующий шаблон вывода.

### Output template (Chain N)

**Chain N — [Name]**

**Inputs used:** [list section numbers and file paths]

**Changes made:**
1. [FILE] [path] — [description]
   ```[lang]
   [code snippet]
   ```
2. ...

**New types/structs:**
- [TypeName]: [brief description]

**Tests added:**
- [test name]: [criterion]

**Verification:** [command run, e.g. cargo test --package canton-ledger-api]

**Risks/Notes:** [any breaking change or follow-up]

**Ready for Chain N+1:** [Yes/No — if No, what is missing]

---

## PART 21 — REPEATED MAPPING TABLES (quick lookup)

### Treasury API routes → SDK

| Route | Method | SDK call |
|-------|--------|----------|
| GET /api/defi/treasury/portfolio?investor= | GET | treasury.get_portfolio_summary(investor) |
| GET /api/defi/treasury/bills | GET | treasury.list_bills(None) |
| GET /api/defi/treasury/bills?status=ACTIVE | GET | treasury.list_bills(Some(BillStatus::Active)) |
| POST /api/defi/treasury/bills | POST | treasury.create_bill(body) |
| GET /api/defi/treasury/bills/:billId | GET | treasury.get_bill(bill_id) |
| PUT /api/defi/treasury/bills/:billId | PUT | treasury.update_bill(bill_id, body) |
| DELETE /api/defi/treasury/bills/:billId | DELETE | treasury.update_bill(bill_id, { status: DELISTED }) |
| GET /api/defi/treasury/purchases | GET | treasury.get_purchase_requests(filter?) |
| POST /api/defi/treasury/purchases | POST | treasury.create_purchase_request(bill_id, investor, num, payment) |

### Hooks → SDK

| Hook | Returns | SDK equivalent |
|------|---------|-----------------|
| useTreasuryBills(address) | availableBills, userHoldings, purchaseTokens, refreshData | treasury.list_bills(), treasury.get_holdings(investor), treasury.create_purchase_request(), then refresh |
| useRealEstateService(address) | availableProperties, userHoldings, governanceProposals, purchaseTokens, voteOnProposal | real_estate.list_properties(), get_holdings(), get_governance_proposals(), create_purchase_request(), vote_on_proposal() |
| usePrivacyVaultService(address) | vaults, assets, proofs, createVault, depositAsset, generateProof | privacy.list_vaults(owner), get_vault_assets(), get_compliance_proofs(), create_vault(), deposit_asset(), generate_proof() |
| useRealCantonNetwork() | availableAssets, userPortfolio, investInAsset | ledger get_active_contracts (InstitutionalAsset), aggregate holdings, ledger submit create/exercise |

### Daml templates → Rust template_id

| Template | Full name | Rust constant |
|----------|-----------|---------------|
| InstitutionalAsset | InstitutionalAsset:InstitutionalAsset | TEMPLATE_INSTITUTIONAL_ASSET |
| AssetPurchaseRequest | InstitutionalAsset:AssetPurchaseRequest | TEMPLATE_ASSET_PURCHASE_REQUEST |
| AssetHolding | InstitutionalAsset:AssetHolding | TEMPLATE_ASSET_HOLDING |
| DividendDistribution | InstitutionalAsset:DividendDistribution | TEMPLATE_DIVIDEND_DISTRIBUTION |

---

## PART 22 — CONFIG YAML FULL EXAMPLE

```yaml
# config/example.yaml (full)
ledger_api:
  grpc_host: "65.108.15.30"
  grpc_port: 30501
  http_url: "http://65.108.15.30:30757"
  tls: false
  connect_timeout_secs: 10
  request_timeout_secs: 30
  ledger_id: "participant"

template_ids:
  institutional_asset: "InstitutionalAsset:InstitutionalAsset"
  asset_purchase_request: "InstitutionalAsset:AssetPurchaseRequest"
  asset_holding: "InstitutionalAsset:AssetHolding"
  dividend_distribution: "InstitutionalAsset:DividendDistribution"

# Optional: compliance and oracle (for domain services)
compliance:
  provider: "mock"  # or "http", "sumsub"
  strict_mode: false

oracle:
  provider: "mock"  # or "pyth", "chainlink"
  cache_ttl_secs: 60
```

---

## PART 23 — RUST CRATE LAYOUT (proposed)

```
cantonnet-omnichain-sdk/
├── crates/
│   ├── canton-core/
│   │   └── src/
│   │       ├── config.rs          # LedgerApiConfig, TemplateIds
│   │       ├── types/
│   │       │   ├── treasury.rs    # TreasuryBill, Holding, PurchaseRequest, CreateBillInput
│   │       │   ├── real_estate.rs  # PropertyInfo, TokenPurchaseRequest
│   │       │   ├── privacy.rs      # PrivacyVault, CreateVaultParams
│   │       │   └── ...
│   │       └── ...
│   ├── canton-ledger-api/
│   │   └── src/
│   │       ├── client.rs          # LedgerClient, connect_from_config, submit_domain_commands
│   │       ├── conversion/
│   │       │   ├── mod.rs
│   │       │   ├── commands.rs    # commands_to_proto
│   │       │   ├── value.rs       # daml_value_to_proto
│   │       │   └── identifier.rs  # identifier_to_proto
│   │       ├── services/
│   │       │   ├── mod.rs
│   │       │   ├── treasury.rs    # TreasuryService
│   │       │   ├── real_estate.rs # RealEstateService
│   │       │   └── privacy.rs     # PrivacyVaultService
│   │       └── defi_client.rs     # DeFiClient facade
│   └── ...
├── config/
│   ├── example.yaml
│   └── example-production.yaml
└── docs/
    ├── DEVNET_PARTICIPANT.md
    └── DEFI_SDK_USAGE.md
```

---

## PART 24 — ACCEPTANCE CRITERIA (repeated, detailed)

1. **Config:** LedgerApiConfig загружается из YAML; LedgerClient::connect_from_config(&config) подключается к 65.108.15.30:30501 и get_ledger_end() возвращает offset.
2. **Conversion:** canton_core::Commands с одним CreateCommand (InstitutionalAsset) конвертируется в proto Commands без ошибки; submit(proto_commands) к participant успешен (код Ok).
3. **Treasury:** TreasuryService создаёт bill (create_bill), список bills (list_bills), создаёт purchase request (create_purchase_request), после approve — get_holdings возвращает новую holding; get_portfolio_summary возвращает агрегат с total_value > 0.
4. **Types:** Все поля TreasuryBill, TreasuryBillHolding, PurchaseRequest из TypeScript присутствуют в Rust-структурах (или явно отмечены как optional для первой версии).
5. **Compliance/Oracle:** TreasuryService принимает Arc<dyn ComplianceProvider> и Arc<dyn OracleProvider>; при mock провайдерах сценарий create_bill + create_purchase_request проходит; при compliance.compliant = false create_purchase_request возвращает ошибку.
6. **Docs:** DEFI_SDK_USAGE.md содержит пример: load config, connect, create TreasuryService, create_bill, list_bills. README cantonnet-omnichain-sdk содержит ссылку на DEFI_SDK_USAGE.md и на этот промт.
7. **Tests:** cargo test --workspace проходит; cargo clippy --workspace -- -D warnings без ошибок. Существующие тесты canton-ledger-api и canton-core не сломаны.
8. **E2E:** Интеграционный тест (feature = "integration" или env CANTON_LEDGER_GRPC) выполняет полный цикл Treasury на DevNet и проверяет portfolio total_value.

---

## PART 25 — FINAL CHECKLIST (copy-paste for executor)

- [ ] Part 1–2 прочитаны; роль и контекст понятны.
- [ ] Chain 1 выполнен: LedgerApiConfig, connect_from_config, тест подключения.
- [ ] Chain 2 выполнен: conversion Commands→proto, submit_domain_commands, тест submit Create.
- [ ] Chain 3 выполнен: TreasuryService, create_bill, list_bills, create_purchase_request, approve, get_holdings, get_portfolio_summary; тесты.
- [ ] Chain 4 выполнен: RealEstateService, PrivacyVaultService (или заглушки), DeFiClient.
- [ ] Chain 5 выполнен: ComplianceProvider, OracleProvider, интеграция в TreasuryService.
- [ ] Chain 6 выполнен: e2e тест, DEFI_SDK_USAGE.md, обновление README и DEVELOPMENT_PROMPT.
- [ ] Part 8 acceptance criteria все отмечены.
- [ ] Part 16 risks учтены; Part 24 acceptance criteria выполнены.
- [ ] Документ использован по цепям 1→6 с структурированным выводом (Part 20 template).

---

## PART 26 — ADDITIONAL DEFI SERVICES (ZK, VALUATION, MULTI-PARTY)

### 26.1 ZK Proof Service (zkProofService.ts)

**Путь:** `src/lib/canton/services/zkProofService.ts`.

**Типы (реальные):**
- `ZKProofType`: BALANCE_PROOF | RANGE_PROOF | MEMBERSHIP_PROOF | OWNERSHIP_PROOF | COMPLIANCE_PROOF | IDENTITY_PROOF | SPENDING_PROOF | AUDIT_PROOF.
- `ZKCircuit`: id, name, description, circuitType, wasmPath, zkeyPath, verificationKeyPath, constraints, publicSignals, privateSignals, provingTime, verificationTime, proofSize, trustedSetup, setupPhase, securityLevel, isReady, createdAt.
- `ZKProof`: id, circuitId, proofType, proofData, publicSignals, verificationResult, createdAt.
- `CircomCircuit`, `CircuitInput`, `CircuitOutput`, `HomomorphicCiphertext`, `DisclosureRule`, `SelectiveDisclosure`.

**SDK relevance:** PrivacyVaultService.generate_proof() в DeFi вызывает ZK-сервис для proofType OWNERSHIP | BALANCE | COMPLIANCE. В SDK: интерфейс ProofProvider с generate_proof(proof_type, params) → ProofResult; реализация MockProofProvider или вызов внешнего HTTP API (snarkjs/circom).

### 26.2 Property Valuation API (propertyValuationAPI.ts)

**Путь:** `src/lib/canton/services/propertyValuationAPI.ts`.

**Типы (реальные):**
- `PropertyValuationRequest`: propertyId, address, city, state, zipCode, propertyType, squareFeet, bedrooms, bathrooms, lotSize, buildingAge, condition, currentListPrice, lastSalePrice, annualPropertyTax, currentRent, capRate, valuationPurpose, urgencyLevel, requestedBy, clientType.
- `PropertyValuationResult`: valuationId, propertyId, estimatedValue (Decimal), confidenceLevel, valuationRange { low, high }, comparativeSales, incomeApproach.

**SDK relevance:** OracleProvider расширение: get_property_valuation(property_id) → PropertyValuationResult. В oracleService.ts уже есть getPropertyValuation(propertyId); в SDK OracleProvider добавить метод get_property_valuation для RealEstateService (оценка при листинге).

### 26.3 Multi-Party Workflow Service (multiPartyWorkflowService.ts)

**Путь:** `src/lib/canton/services/multiPartyWorkflowService.ts`.

**Типы (реальные):**
- `TransactionType`: ASSET_PURCHASE | ASSET_SALE | DIVIDEND_DISTRIBUTION | BRIDGE_TRANSFER | PRIVACY_VAULT_ACCESS | COMPLIANCE_REPORTING | CONTRACT_UPGRADE.
- `TransactionStatus`: PENDING_SIGNATURES | UNDER_REVIEW | APPROVED | REJECTED | EXPIRED | EXECUTED.
- `PartyType`: INVESTOR | CUSTODIAN | ISSUER | REGULATOR | AUDITOR | COMPLIANCE_OFFICER | TRANSFER_AGENT | VALIDATOR.
- `SignatureType`: APPROVAL | WITNESS | COMPLIANCE_SIGN_OFF | REGULATORY_APPROVAL | AUDIT_CONFIRMATION | RISK_ACKNOWLEDGMENT.
- `AuthorizationLevel`: BASIC (up to $10K) | ENHANCED ($100K) | INSTITUTIONAL ($1M) | EXECUTIVE ($10M) | BOARD_LEVEL (unlimited).
- `MultiPartyTransaction`: id, contractId, templateType, title, description, amount, assetId, transactionType, requiredSignatures (PartyRequirement[]), collectedSignatures (PartySignature[]), minimumApprovals, privacyLevel, complianceCheck, regulatoryApproval, status, createdAt, expiresAt, completedAt, initiator, priority, tags.

**Методы:** createTransaction, getTransaction, submitSignature, getPendingTransactions(partyId), executeTransaction (после сбора подписей). Использует DamlIntegrationService для контрактов.

**SDK relevance:** Опционально WorkflowService в SDK: create_multi_party_transaction(params), submit_signature(transaction_id, party, signature), get_pending_for_party(party_id). Реализация через те же Daml шаблоны (если есть workflow template) или заглушка для первой версии.

### 26.4 Mapping table (additional services → SDK)

| DeFi Service | SDK Trait/Module | Methods to implement |
|--------------|------------------|----------------------|
| zkProofService | ProofProvider (optional) | generate_proof(proof_type, params) |
| propertyValuationAPI | OracleProvider | get_property_valuation(property_id) |
| multiPartyWorkflowService | WorkflowService (optional) | create_transaction, submit_signature, get_pending |

---

## PART 27 — API REQUEST/RESPONSE SCHEMAS & ENV

### 27.1 Treasury API request bodies (from route handlers)

**POST /api/defi/treasury/bills**
- Required: `name` (string), `symbol` (string).
- Optional: все остальные поля TreasuryBill (description, issuer, custodian, maturity, maturityDate, issueDate, totalSupply, availableSupply, pricePerToken, minimumInvestment, currentYield, expectedYield, yieldToMaturity, status).
- Validation: 400 если !body.name || !body.symbol.
- Response 201: `{ success: true, data: TreasuryBill, message: "Treasury bill created successfully" }`.

**POST /api/defi/treasury/purchases**
- Required: `billId` (string), `investor` (string), `numberOfTokens` (number).
- Optional: `paymentData` (object).
- Validation: 400 если отсутствует required; 400 если numberOfTokens <= 0.
- Response 201: `{ success: true, data: PurchaseRequest, message: "Purchase request created successfully" }`.

### 27.2 Environment variables (from API route init)

Используются при инициализации DamlIntegrationService и TreasuryBillsService в API routes:
- `CANTON_PARTICIPANT_URL` — default 'http://localhost:5011'.
- `CANTON_PARTICIPANT_ID` — default 'participant1'.
- `CANTON_AUTH_TOKEN` — default ''.
- `CANTON_PARTY_ID` — default 'party1'.

Для SDK: те же переменные или конфиг YAML (ledger_api.grpc_host, grpc_port, ledger_id; опционально auth_token, party_id).

### 27.3 TreasuryBillConfig (runtime config from treasuryBillsService)

Из кода API route getTreasuryService():
- `enabled`: boolean.
- `minInvestment`: string (e.g. '100').
- `maxInvestment`: string (e.g. '10000000').
- `settlementType`: 'T0' | 'T1' | 'T2'.
- `yieldDistributionFrequency`: 'DAILY' | 'WEEKLY' | 'MONTHLY' | 'QUARTERLY'.
- `autoReinvest`: boolean.
- `secondaryMarketEnabled`: boolean.

SDK: при create_purchase_request проверять amount >= min_investment и <= max_investment (конфиг передаётся в TreasuryService::new или через ComplianceProvider/OracleProvider).

---

## PART 28 — E2E & INTEGRATION TEST PATHS

### 28.1 E2E test files (Playwright)

- `e2e/treasury/view-bills.spec.ts` — просмотр списка облигаций.
- `e2e/treasury/purchase-bill.spec.ts` — покупка облигации.
- `e2e/treasury/portfolio.spec.ts` — портфолио.
- `e2e/realestate/view-properties.spec.ts` — просмотр недвижимости.
- `e2e/realestate/purchase-tokens.spec.ts` — покупка токенов недвижимости.
- `e2e/realestate/governance-vote.spec.ts` — голосование по предложениям.
- `e2e/privacy/create-vault.spec.ts` — создание vault.
- `e2e/privacy/deposit-assets.spec.ts` — депозит в vault.
- `e2e/privacy/generate-proof.spec.ts` — генерация ZK proof.
- `e2e/auth/logout.spec.ts` — выход (косвенно для DeFi session).

### 28.2 Integration test files (Vitest)

- `src/lib/canton/services/__tests__/integration/api-routes/treasury.integration.test.ts`.
- `src/lib/canton/services/__tests__/integration/api-routes/realestate.integration.test.ts`.
- `src/lib/canton/services/__tests__/integration/api-routes/privacy.integration.test.ts`.
- `src/lib/canton/services/__tests__/integration/api-routes/oracle.integration.test.ts`.
- `src/lib/canton/services/__tests__/integration/api-routes/compliance.integration.test.ts`.
- `src/lib/canton/services/__tests__/integration/services-canton/daml-canton.integration.test.ts`.
- `src/lib/canton/services/__tests__/integration/services-supabase/treasury-supabase.integration.test.ts`.
- `src/lib/canton/services/__tests__/integration/ui-components/treasury-panel.integration.test.ts`.
- `src/lib/canton/services/__tests__/integration/user-flows/purchase-treasury.integration.test.ts`.
- `src/lib/canton/services/__tests__/integration/user-flows/purchase-realestate.integration.test.ts`.
- `src/lib/canton/services/__tests__/integration/user-flows/create-vault.integration.test.ts`.
- `src/lib/canton/services/__tests__/integration/user-flows/registration-login.integration.test.ts`.

### 28.3 E2E test data (e2e/fixtures/test-data.ts)

- `testTreasuryBills`: bill1 (TB001, US Treasury 3-Month, yield 5.25, price 1000), bill2 (TB002, 6-Month), bill3 (TB003, 1-Year).
- `testVaults`: vault1 (STANDARD, GROTH16), vault2 (ENHANCED, PLONK), vault3 (MAXIMUM, GROTH16).
- `testRealEstate`: property1 (RE001, COMMERCIAL, 50000), property2 (RESIDENTIAL, 100000), property3 (INDUSTRIAL, 75000).
- `testGovernanceProposals`: proposal1 (RENOVATION), proposal2.

Использовать при написании интеграционных тестов SDK: те же id/типы для сравнения с ожидаемыми значениями.

---

## PART 29 — PROMPT ENGINEERING 2025 (EXTENDED)

### 29.1 RAG-style context injection

При ответе по цепи подтягивать только релевантные секции:
- Chain 1: Part 2.1, 5.1–5.2, 6.1, 9.1 (paths).
- Chain 2: Part 4.2, 5.1, 6.2, 7 (Chain 2), 9.2 (template IDs), 12.4–12.6 (Daml payloads).
- Chain 3: Part 3.1, 6.3, 7 (Chain 3), 12.1–12.3, 19.1–19.3.
- Chain 4: Part 3.2, 3.3, 6.4, 6.5, 7 (Chain 4), 19.7–19.9.
- Chain 5: Part 3.5, 3.6, 6.6, 19.10–19.11.
- Chain 6: Part 8, 9.1, 15 (test matrix).

Указывать в ответе: «Использованы секции: X, Y, Z».

### 29.2 ReAct-style step disclosure

Для каждой подзадачи цепи выводить:
1. **Thought:** что делаем и зачем.
2. **Action:** какой файл/функция/тип.
3. **Observation:** результат (код, тест, вывод).
4. **Next:** следующий шаг или «Chain N complete».

### 29.3 Few-shot для конвертеров

При реализации DamlValue → proto Value использовать 1–2 примера из Part 14 (minimal CreateCommand) как образец формата полей (RecordField::new("name", DamlValue::text(...))). При добавлении нового типа (например TreasuryBill → DamlRecord) привести один полный пример маппинга из Part 12.

### 29.4 Tool-use / function-calling alignment

Если исполнитель — агент с инструментами: описать «инструменты» как вызовы — read_file(path), edit_file(path, old, new), run_terminal(command). Цепи 1–6 разбить на атомарные вызовы (например Chain 2.2 = read_file(canton-ledger-api/src/conversion/identifier.rs), edit_file(..., add identifier_to_proto)).

### 29.5 Output format enforcement

Требовать JSON или markdown-блоки для структурированного вывода:
- **Files changed:** список путей.
- **Types added:** список имён и модулей.
- **Tests:** список имён тестов и команд запуска.
- **Acceptance:** Yes/No для критерия цепи.

---

## PART 30 — DAML TEMPLATE CONSTANTS (EXACT FROM CODE)

Из damlIntegrationService.ts (строки 204–207):

```ts
private readonly INSTITUTIONAL_ASSET_TEMPLATE = 'InstitutionalAsset:InstitutionalAsset';
private readonly PURCHASE_REQUEST_TEMPLATE = 'InstitutionalAsset:AssetPurchaseRequest';
private readonly ASSET_HOLDING_TEMPLATE = 'InstitutionalAsset:AssetHolding';
private readonly DIVIDEND_DISTRIBUTION_TEMPLATE = 'InstitutionalAsset:DividendDistribution';
```

В SDK config (YAML или Rust const):
- `institutional_asset`: "InstitutionalAsset:InstitutionalAsset"
- `asset_purchase_request`: "InstitutionalAsset:AssetPurchaseRequest"
- `asset_holding`: "InstitutionalAsset:AssetHolding"
- `dividend_distribution`: "InstitutionalAsset:DividendDistribution"

Package_id не указан в коде — берётся с participant (PackageService) или оставляется пустым/дефолтным при разборе "Module:Entity" (module_name = "InstitutionalAsset", entity_name = "InstitutionalAsset").

---

## PART 31 — SDK CRATE LAYOUT (ACTUAL)

Текущая структура cantonnet-omnichain-sdk/crates:

- **canton-core:** src/config.rs, error.rs, lib.rs, traits/mod.rs, traits/proto.rs, types/command.rs, event.rs, filter.rs, identifier.rs, mod.rs, offset.rs, transaction.rs, value.rs.
- **canton-ledger-api:** src/client.rs, lib.rs; proto/ com/daml/ledger/api/v2/*.proto (command_submission_service, state_service, commands, value, transaction_filter, etc.).
- **canton-crypto:** src/keys.rs, keystore/memory.rs, mod.rs, lib.rs.
- **canton-wallet:** src/derivation.rs, lib.rs, party_id.rs, wallet.rs.
- **canton-transport:** lib.rs (минимальный).
- **canton-reliability:** lib.rs (минимальный).
- **canton-observability:** lib.rs (минимальный).

Для реализации по цепям:
- Chain 1: canton-core/config.rs (LedgerApiConfig), canton-ledger-api/client.rs (connect_from_config).
- Chain 2: новый модуль canton-ledger-api/conversion/ (или conversion.rs) — commands_to_proto, value_to_proto, identifier_to_proto; client.rs — submit_domain_commands.
- Chain 3: canton-core/types/treasury.rs (TreasuryBill, Holding, PurchaseRequest, CreateBillInput, PortfolioSummary); canton-ledger-api/services/treasury.rs (TreasuryService) или отдельный крейт canton-defi.
- Chain 4: canton-core/types/real_estate.rs, privacy.rs; canton-ledger-api/services/real_estate.rs, privacy.rs (или canton-defi).
- Chain 5: canton-ledger-api/compliance.rs, oracle.rs (traits + mock impl).
- Chain 6: тесты в соответствующих крейтах, docs/DEFI_SDK_USAGE.md.

---

## PART 32 — YIELDDISTRIBUTION & DIVIDEND FLOW

### 32.1 YieldDistribution (treasuryBillsService.ts)

Тип уже описан в Part 3.1. Поля: distributionId, billId, totalYield, yieldPerToken, distributionDate, period { startDate, endDate }, totalTokens, totalInvestors, transactionHash?, blockNumber?, createdAt.

В Daml используется шаблон InstitutionalAsset:DividendDistribution. SDK: опционально TreasuryService.distribute_yield(bill_id, period, yield_per_token) → exercise на DividendDistribution или create контракт; и get_yield_distributions(bill_id) — GetActiveContracts по шаблону DividendDistribution.

### 32.2 Dividend flow (Daml → SDK)

1. Issuer/Custodian создаёт DividendDistribution контракт (или вызывает choice на InstitutionalAsset).
2. Инвесторы получают выплаты по holding (автоматически или через choice).
3. SDK: list_dividend_distributions(bill_id), get_distribution(distribution_id) — для отображения в UI и отчётах.

---

## PART 33 — PROMPT CHAIN EXECUTION ORDER (STRICT)

1. **Chain 1** — без LedgerClient и conversion нельзя подключаться к participant. Выход: LedgerApiConfig, connect_from_config, тест connect.
2. **Chain 2** — без submit_domain_commands и create_contract/exercise нельзя создавать контракты. Выход: conversion.rs, submit_domain_commands, тест submit Create.
3. **Chain 3** — зависит от Chain 2 (create_contract для InstitutionalAsset, exercise для Approve). Выход: TreasuryService, все методы, тесты.
4. **Chain 4** — зависит от Chain 3 (тот же LedgerClient и шаблоны). Выход: RealEstateService, PrivacyVaultService, DeFiClient.
5. **Chain 5** — зависит от Chain 4 (инжекция в TreasuryService/RealEstate). Выход: ComplianceProvider, OracleProvider, интеграция.
6. **Chain 6** — зависит от всех предыдущих. Выход: e2e тест, документация.

Не переходить к Chain N+1 пока критерий приёмки Chain N не выполнен. При падении теста — вернуться в ту же цепь и исправить.

---

## PART 34 — QUICK REFERENCE: DEFI FILES BY DOMAIN

| Domain | Services | API Routes | Hooks | Components |
|--------|----------|------------|-------|------------|
| Treasury | treasuryBillsService.ts | treasury/bills, bills/[billId], portfolio, purchases | useTreasuryBills | TreasuryBillsPanel, CCPurchaseWidget |
| Real Estate | realEstateService.ts, propertyValuationAPI.ts | (через общий defi или будущие routes) | useRealEstate | RealEstatePanel, ProductCard |
| Privacy | privacyVaultService.ts, zkProofService.ts | (через общий defi) | usePrivacyVaults | PrivacyVaultsPanel |
| Ledger | damlIntegrationService.ts | — | realCantonIntegration, useCantonBridge | MultiPartyDashboard |
| Compliance | complianceService.ts | compliance/kyc | — | MultiPartyAuthPanel |
| Oracle | oracleService.ts | oracle/prices, oracle/treasury-yields | — | StablecoinSelector, price displays |
| Auth | authService.ts | auth/login, logout, register | — | — |

---

## PART 35 — VERSION & CHANGELOG PLACEHOLDER

**Version:** 1.1  
**Changes from 1.0:** Добавлены Part 26–35: zkProofService, propertyValuationAPI, multiPartyWorkflowService; API request/response schemas и env; E2E/integration test paths и test data; расширенный prompt engineering 2025 (RAG, ReAct, few-shot, tool-use); Daml template constants из кода; актуальная структура SDK крейтов; YieldDistribution/Dividend flow; строгий порядок выполнения цепей; quick reference по файлам DeFi.

---

## PART 36 — RUST TYPE SKELETONS (REFERENCE)

Ниже — скелеты структур для canton-core/types (без полной реализации; для согласованности с Part 12 и 19).

### 36.1 Enums (canton-core)

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BillStatus { ACTIVE, SUSPENDED, MATURED, DELISTED }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HoldingStatus { ACTIVE, SOLD, MATURED }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RequestStatus { PENDING, APPROVED, REJECTED, COMPLETED, FAILED }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TreasuryMaturity { M1, M3, M6, Y1, Y2, Y5, Y10, Y30 }  // 1M, 3M, 6M, 1Y, ...

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AssetClass { EQUITY, FIXED_INCOME, REAL_ESTATE, COMMODITIES, ALTERNATIVES, DERIVATIVES }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RiskRating { AAA, AA, A, BBB, BB, B, CCC }
```

### 36.2 TreasuryBill (canton-core/types/treasury.rs)

```rust
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasuryBill {
    pub bill_id: String,
    pub name: String,
    pub symbol: String,
    pub description: String,
    pub issuer: String,
    pub custodian: String,
    pub maturity: TreasuryMaturity,
    pub maturity_date: DateTime<Utc>,
    pub issue_date: DateTime<Utc>,
    pub total_supply: Decimal,
    pub available_supply: Decimal,
    pub price_per_token: Decimal,
    pub minimum_investment: Decimal,
    pub current_yield: Decimal,
    pub expected_yield: Decimal,
    pub yield_to_maturity: Decimal,
    pub status: BillStatus,
    pub contract_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### 36.3 CreateBillInput (minimal for POST /api/defi/treasury/bills)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBillInput {
    pub name: String,
    pub symbol: String,
    pub description: Option<String>,
    pub issuer: Option<String>,
    pub custodian: Option<String>,
    pub maturity: Option<TreasuryMaturity>,
    pub total_supply: Option<Decimal>,
    pub price_per_token: Option<Decimal>,
    pub minimum_investment: Option<Decimal>,
    pub expected_yield: Option<Decimal>,
}
```

### 36.4 PaymentData (for create_purchase_request)

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PaymentData {
    pub payment_method: Option<String>,
    pub wallet_address: Option<String>,
    pub transaction_hash: Option<String>,
    pub block_number: Option<u64>,
}
```

### 36.5 PortfolioSummary (aggregate from get_holdings)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioSummary {
    pub total_value: Decimal,
    pub total_invested: Decimal,
    pub unrealized_gains: Decimal,
    pub yield_earned: Decimal,
    pub holding_count: usize,
}
```

### 36.6 LedgerApiConfig (canton-core/config.rs)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerApiConfig {
    pub grpc_host: String,
    pub grpc_port: u16,
    pub http_url: Option<String>,
    pub tls: Option<bool>,
    #[serde(default)]
    pub connect_timeout_secs: Option<u64>,
    #[serde(default)]
    pub request_timeout_secs: Option<u64>,
    pub ledger_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateIds {
    pub institutional_asset: String,
    pub asset_purchase_request: String,
    pub asset_holding: String,
    pub dividend_distribution: Option<String>,
}
```

---

## PART 37 — INTEGRATION TEST SCENARIOS (STEP-BY-STEP)

### 37.1 Scenario: Full Treasury flow (Chain 3 acceptance)

1. Load config from config/example.yaml (or env CANTON_LEDGER_GRPC).
2. LedgerClient::connect_from_config(&config).expect("connect").
3. TreasuryService::new(&mut ledger, &template_ids, compliance, oracle).
4. create_bill(CreateBillInput { name: "US Treasury 3M".into(), symbol: "TB001".into(), ... }).expect("create_bill").
5. list_bills(None).expect("list_bills"); assert!(!bills.is_empty()); assert_eq!(bills[0].symbol, "TB001").
6. create_purchase_request("TB001", "investor-party-id", 10, PaymentData::default()).expect("create_purchase_request").
7. approve_purchase_request(request_id, "custodian-party-id").expect("approve").
8. get_holdings("investor-party-id").expect("holdings"); assert!(!holdings.is_empty()).
9. get_portfolio_summary("investor-party-id").expect("portfolio"); assert!(summary.total_value > Decimal::ZERO).

### 37.2 Scenario: Conversion roundtrip (Chain 2)

1. Build Commands with one CreateCommand (template_id = "InstitutionalAsset:InstitutionalAsset", create_arguments = minimal DamlRecord with assetId, name, symbol, totalSupply, availableSupply, pricePerToken, minimumInvestment).
2. commands_to_proto(&commands).expect("to_proto").
3. Assert proto.commands.len() == 1; assert Create with template_id matches; assert create_arguments has required fields.
4. (Optional) proto_to_commands(proto).expect("from_proto"); assert roundtrip equality for key fields.

### 37.3 Scenario: Compliance reject (Chain 5)

1. FailingComplianceProvider: validate_transaction always returns ValidationResult { compliant: false, reasons: vec!["test reject".into()] }.
2. TreasuryService::new(..., Arc::new(FailingComplianceProvider), oracle).
3. create_purchase_request(...).expect_err("must fail"); assert error message contains "compliant" or "reject".

### 37.4 Scenario: Oracle yield in create_bill (Chain 5)

1. MockOracleProvider: get_treasury_yield("1Y") returns TreasuryYield { maturity: Y1, yield: "5.75", timestamp, source: "mock" }.
2. create_bill(CreateBillInput { maturity: Some(TreasuryMaturity::Y1), ... }).
3. Assert returned TreasuryBill.current_yield == 5.75 (or string "5.75"); assert expected_yield populated from oracle.

---

## PART 38 — ERROR CODES & MESSAGES (SDK)

Рекомендуемые варианты ошибок для единообразия:

- **ConfigLoadFailed** — не удалось загрузить YAML или парсить LedgerApiConfig.
- **ConnectFailed** — LedgerClient::connect не удался (таймаут, unreachable).
- **ConversionError** — DamlValue → proto или Commands → proto не удался (неподдерживаемый тип, отсутствующее поле).
- **SubmitFailed** — Ledger API submit вернул ошибку (дубликат command_id, неверный шаблон, права).
- **ContractNotFound** — get_bill / get_holdings не нашли контракт по id.
- **ValidationFailed** — compliance validate_transaction вернул compliant: false.
- **BusinessRuleViolation** — amount < minimum_investment или amount > available_supply.

В canton-core/error.rs добавить варианты SdkError с контекстом (например SdkError::SubmitFailed { reason: String, command_id: String }).

---

## PART 39 — DEPENDENCY VERSIONS (SDK Cargo.toml)

Рекомендуемые версии для совместимости с Canton Ledger API v2:

- **tonic** = "0.11" — gRPC client.
- **prost** = "0.12" — proto (generated code).
- **serde** = { version = "1", features = ["derive"] }.
- **serde_json** = "1".
- **serde_yaml** = "0.9".
- **chrono** = { version = "0.4", features = ["serde"] }.
- **rust_decimal** = { version = "1", features = ["serde"] }.
- **uuid** = { version = "1", features = ["v4", "serde"] }.
- **thiserror** = "1", **anyhow** = "1" — error handling.

canton-ledger-api: зависимость от canton-core; build.rs — компиляция proto из proto/com/daml/ledger/api/v2/*.proto.

---

## PART 40 — DEFI FRONTEND → SDK DATA FLOW (SUMMARY)

1. **User** выбирает bill на TreasuryBillsPanel → GET /api/defi/treasury/bills → TreasuryBillsService.getAllTreasuryBills() → (in-memory или DamlIntegrationService.getInstitutionalAssets()) → Response bills.
2. **User** нажимает "Purchase" → POST /api/defi/treasury/purchases { billId, investor, numberOfTokens, paymentData } → TreasuryBillsService.createPurchaseRequest() → ComplianceService.validateTransaction() → DamlIntegrationService.createPurchaseRequest() → Response purchaseRequest.
3. **Custodian** подтверждает → (в UI или отдельный flow) → TreasuryBillsService.approvePurchaseRequest() → DamlIntegrationService.exercise(AssetPurchaseRequest, Approve) → создаётся AssetHolding → Response holding.
4. **User** открывает Portfolio → GET /api/defi/treasury/portfolio?investor= → getInvestorPortfolioSummary(investor) → aggregate holdings → Response { totalValue, totalInvested, ... }.

SDK должен поддерживать те же операции из Rust: LedgerClient + TreasuryService заменяют вызовы к Next.js API, когда backend написан на Rust или нужен прямой доступ к participant без прохождения через Next.js.

---

## PART 41 — API REQUEST/RESPONSE JSON EXAMPLES

### 41.1 POST /api/defi/treasury/bills — request body (minimal)

```json
{
  "name": "US Treasury 3-Month",
  "symbol": "TB001"
}
```

### 41.2 POST /api/defi/treasury/bills — request body (full)

```json
{
  "name": "US Treasury 3-Month",
  "symbol": "TB001",
  "description": "Short-term government security",
  "issuer": "US Treasury",
  "custodian": "CustodianParty1",
  "maturity": "3M",
  "maturityDate": "2025-04-30T00:00:00.000Z",
  "issueDate": "2025-01-30T00:00:00.000Z",
  "totalSupply": "1000000",
  "availableSupply": "1000000",
  "pricePerToken": "1000",
  "minimumInvestment": "100",
  "currentYield": "5.25",
  "expectedYield": "5.25",
  "yieldToMaturity": "5.25",
  "status": "ACTIVE"
}
```

### 41.3 POST /api/defi/treasury/bills — response 201

```json
{
  "success": true,
  "data": {
    "billId": "TB001",
    "name": "US Treasury 3-Month",
    "symbol": "TB001",
    "description": "Short-term government security",
    "issuer": "US Treasury",
    "custodian": "CustodianParty1",
    "maturity": "3M",
    "maturityDate": "2025-04-30T00:00:00.000Z",
    "issueDate": "2025-01-30T00:00:00.000Z",
    "totalSupply": "1000000",
    "availableSupply": "1000000",
    "pricePerToken": "1000",
    "minimumInvestment": "100",
    "currentYield": "5.25",
    "expectedYield": "5.25",
    "yieldToMaturity": "5.25",
    "status": "ACTIVE",
    "contractId": null,
    "createdAt": "2025-01-30T12:00:00.000Z",
    "updatedAt": "2025-01-30T12:00:00.000Z"
  },
  "message": "Treasury bill created successfully"
}
```

### 41.4 POST /api/defi/treasury/purchases — request body

```json
{
  "billId": "TB001",
  "investor": "investor-party-id",
  "numberOfTokens": 10,
  "paymentData": {
    "paymentMethod": "STABLECOIN",
    "walletAddress": "0x..."
  }
}
```

### 41.5 POST /api/defi/treasury/purchases — response 201

```json
{
  "success": true,
  "data": {
    "requestId": "req-uuid-1",
    "billId": "TB001",
    "investor": "investor-party-id",
    "numberOfTokens": "10",
    "totalAmount": "10000",
    "paymentMethod": "STABLECOIN",
    "status": "PENDING",
    "kycLevel": "BASIC",
    "complianceCheck": { "passed": true, "reasons": [] },
    "requestDate": "2025-01-30T12:00:00.000Z",
    "expiryDate": "2025-02-06T12:00:00.000Z",
    "contractId": null,
    "createdAt": "2025-01-30T12:00:00.000Z",
    "updatedAt": "2025-01-30T12:00:00.000Z"
  },
  "message": "Purchase request created successfully"
}
```

### 41.6 GET /api/defi/treasury/portfolio?investor=party1 — response 200

```json
{
  "success": true,
  "data": {
    "totalValue": "105000",
    "totalInvested": "100000",
    "unrealizedGains": "5000",
    "yieldEarned": "1200",
    "holdingCount": 2
  }
}
```

SDK типы (TreasuryBill, PurchaseRequest, PortfolioSummary) должны сериализоваться в JSON, совместимый с этими формами для замены API при Rust backend.

---

## PART 42 — CANTON LEDGER API V2 SERVICES (PROTO)

Какие gRPC сервисы используются для DeFi и SDK:

- **CommandSubmissionService** — Submit (commands) — обязательно для create_contract, exercise, submit_domain_commands.
- **StateService** — GetLedgerEnd — обязательно для получения текущего offset (и опционально для submit_and_wait по completion).
- **VersionService** — GetLedgerIdentity — опционально для получения ledger_id при connect.
- **ActiveContractsService** — GetActiveContracts — необходимо для get_bill, list_bills, get_holdings (query по template_id и party filter).
- **CommandCompletionService** — SubscribeCompletion — опционально для submit_and_wait (ожидание completion по command_id).
- **EventQueryService** — GetUpdates — опционально для стриминга событий (real-time UI); можно отложить.

В Chain 1 минимум: StateService (get_ledger_end). В Chain 2: CommandSubmissionService (submit). В Chain 3: ActiveContractsService (get_active_contracts) для чтения контрактов. При реализации LedgerClient проверить наличие клиентов именно этих сервисов в canton-ledger-api.

---

## PART 43 — DEFI COMPONENT → SDK METHOD MAPPING

| UI Component | User action | API / Service call | SDK method |
|--------------|-------------|--------------------|------------|
| TreasuryBillsPanel | Load bills | GET bills → getAllTreasuryBills | treasury.list_bills(None) |
| TreasuryBillsPanel | Filter by status | GET bills?status=ACTIVE | treasury.list_bills(Some(BillStatus::Active)) |
| CCPurchaseWidget | Submit purchase | POST purchases → createPurchaseRequest | treasury.create_purchase_request(bill_id, investor, num, payment_data) |
| Portfolio view | Load portfolio | GET portfolio?investor= | treasury.get_portfolio_summary(investor) |
| RealEstatePanel | Load properties | getAvailableProperties() | real_estate.list_properties() |
| RealEstatePanel | Purchase tokens | purchaseTokens(params) | real_estate.create_purchase_request(property_id, investor, num, payment_data) |
| RealEstatePanel | Vote | voteOnProposal(proposalId, voter, support) | real_estate.vote_on_proposal(proposal_id, voter, support) |
| PrivacyVaultsPanel | Create vault | createVault(params) | privacy.create_vault(params) |
| PrivacyVaultsPanel | Deposit | depositAsset(vaultId, type, amount) | privacy.deposit_asset(vault_id, asset_type, amount) |
| PrivacyVaultsPanel | Generate proof | generateProof(params) | privacy.generate_proof(params) |
| MultiPartyDashboard | List pending | getPendingTransactions(party) | workflow.get_pending_for_party(party) (optional) |
| MultiPartyAuthPanel | Submit signature | submitSignature(txId, party, sig) | workflow.submit_signature(tx_id, party, sig) (optional) |

---

## PART 44 — RECOMMENDED IMPLEMENTATION ORDER (SUMMARY)

1. **Week 1 (Chain 1–2):** Config load, connect_from_config, conversion Commands→proto, submit_domain_commands. Тесты: connect, submit minimal Create.
2. **Week 2 (Chain 3):** Treasury types, TreasuryService (create_bill, list_bills, create_purchase_request, approve_purchase_request, get_holdings, get_portfolio_summary). Тесты: full Treasury flow.
3. **Week 3 (Chain 4):** RealEstateService, PrivacyVaultService (или заглушки), DeFiClient facade. Тесты: list_properties, create_vault.
4. **Week 4 (Chain 5):** ComplianceProvider, OracleProvider (mock + optional HTTP), интеграция в TreasuryService. Тесты: compliance reject, oracle yield.
5. **Week 5 (Chain 6):** E2E интеграционный тест против DevNet, DEFI_SDK_USAGE.md, обновление README и DEVELOPMENT_PROMPT.

При сжатых сроках: приоритет Chain 1 → 2 → 3; Chain 4–5–6 можно сократить до заглушек и документации.

---

## PART 45 — INDEX OF ALL PARTS (NAVIGATION)

| Part | Title | Use when |
|------|-------|----------|
| 1 | META & PROMPT ENGINEERING 2025 | Start; role, chains, structured output |
| 2 | PROJECT OVERVIEW | Repo paths, data flows |
| 3 | DEFI FEATURE INVENTORY | Real types/methods from TS services |
| 4 | SDK CURRENT STATE & GAPS | What exists vs what to implement |
| 5 | ARCHITECTURE DESIGN | Layers, crates, template IDs |
| 6 | API SURFACE SPECIFICATION | Config, LedgerClient, Treasury/RealEstate/Privacy, Compliance/Oracle |
| 7 | IMPLEMENTATION PROMPT CHAINS | Detailed tasks per Chain 1–6 |
| 8 | ACCEPTANCE CRITERIA & TESTING | General criteria, tests, risks |
| 9 | APPENDIX (paths, template IDs, env, checklist) | Quick reference |
| 10 | EXPANDED REFERENCE (TS→SDK mapping) | Method mapping tables |
| 11 | PROMPT ENGINEERING NOTES | Chain-of-thought, structured output |
| 12 | DETAILED TYPE DEFINITIONS (TS→Rust) | Field mapping tables |
| 13 | PROMPT CHAIN SUB-STEPS | Granular sub-steps per chain |
| 14 | CODE SNIPPETS | Minimal CreateCommand, connect, create_bill pseudo |
| 15 | TEST MATRIX | Test name, chain, criterion |
| 16 | RISKS & MITIGATION | Risk table |
| 17 | GLOSSARY | Ledger API, Participant, Template, etc. |
| 18 | REPEATED CHECKLIST | Copy-paste checklist |
| 19 | FULL DEFI TYPE REFERENCE | TS interface excerpts |
| 20 | PROMPT CHAIN EXECUTION TEMPLATE | Output template per chain |
| 21 | REPEATED MAPPING TABLES | Routes, hooks, Daml templates |
| 22 | CONFIG YAML FULL EXAMPLE | example.yaml |
| 23 | RUST CRATE LAYOUT (proposed) | Directory structure |
| 24 | ACCEPTANCE CRITERIA (repeated, detailed) | 8 criteria |
| 25 | FINAL CHECKLIST | Copy-paste for executor |
| 26 | ADDITIONAL DEFI SERVICES | ZK, Valuation, Multi-Party |
| 27 | API REQUEST/RESPONSE SCHEMAS & ENV | POST body, env vars, TreasuryBillConfig |
| 28 | E2E & INTEGRATION TEST PATHS | Playwright, Vitest, test data |
| 29 | PROMPT ENGINEERING 2025 (EXTENDED) | RAG, ReAct, few-shot, tool-use |
| 30 | DAML TEMPLATE CONSTANTS | Exact strings from code |
| 31 | SDK CRATE LAYOUT (ACTUAL) | canton-core, canton-ledger-api file list |
| 32 | YIELDDISTRIBUTION & DIVIDEND FLOW | DividendDistribution template |
| 33 | PROMPT CHAIN EXECUTION ORDER | Strict order, no skip |
| 34 | QUICK REFERENCE: DEFI FILES BY DOMAIN | Services, routes, hooks, components |
| 35 | VERSION & CHANGELOG | 1.1 changes |
| 36 | RUST TYPE SKELETONS | Enums, TreasuryBill, CreateBillInput, etc. |
| 37 | INTEGRATION TEST SCENARIOS | Step-by-step scenarios |
| 38 | ERROR CODES & MESSAGES | SdkError variants |
| 39 | DEPENDENCY VERSIONS | Cargo.toml |
| 40 | DEFI FRONTEND → SDK DATA FLOW | Summary flow |
| 41 | API REQUEST/RESPONSE JSON EXAMPLES | Full request/response bodies |
| 42 | CANTON LEDGER API V2 SERVICES | Proto services list |
| 43 | DEFI COMPONENT → SDK METHOD MAPPING | UI action → SDK method |
| 44 | RECOMMENDED IMPLEMENTATION ORDER | Week-by-week plan |
| 45 | INDEX OF ALL PARTS | This table |
| 46 | ENV & CONFIG QUICK REF | Env vars and YAML keys |

---

## PART 46 — ENV & CONFIG QUICK REF

### Environment variables (DeFi API routes → SDK)

| Variable | DeFi default | SDK usage |
|----------|-------------|-----------|
| CANTON_PARTICIPANT_URL | http://localhost:5011 | Map to grpc_host:port or http_url |
| CANTON_PARTICIPANT_ID | participant1 | ledger_id or participant identifier |
| CANTON_AUTH_TOKEN | '' | Optional auth for Ledger API |
| CANTON_PARTY_ID | party1 | act_as party for commands |
| NEXT_PUBLIC_DAML_USE_REAL_LEDGER | (optional) | DeFi only; SDK always uses real |
| CANTON_LEDGER_GRPC | (e.g. 65.108.15.30:30501) | Integration tests: skip if unset |

### YAML config keys (config/example.yaml)

| Key | Type | Required | Example |
|-----|------|----------|---------|
| ledger_api.grpc_host | string | yes | "65.108.15.30" |
| ledger_api.grpc_port | number | yes | 30501 |
| ledger_api.http_url | string | no | "http://65.108.15.30:30757" |
| ledger_api.tls | bool | no | false |
| ledger_api.connect_timeout_secs | number | no | 10 |
| ledger_api.request_timeout_secs | number | no | 30 |
| ledger_api.ledger_id | string | no | "participant" |
| template_ids.institutional_asset | string | yes | "InstitutionalAsset:InstitutionalAsset" |
| template_ids.asset_purchase_request | string | yes | "InstitutionalAsset:AssetPurchaseRequest" |
| template_ids.asset_holding | string | yes | "InstitutionalAsset:AssetHolding" |
| template_ids.dividend_distribution | string | no | "InstitutionalAsset:DividendDistribution" |
| compliance.provider | string | no | "mock" |
| oracle.provider | string | no | "mock" |

---

## PART 47 — EXECUTION REMINDERS

- **Перед Chain 1:** Убедиться, что в репо есть cantonnet-omnichain-sdk с крейтами canton-core, canton-ledger-api; config/example.yaml существует или создаётся.
- **Перед Chain 2:** LedgerClient должен успешно подключаться к participant; proto-файлы Canton Ledger API v2 должны быть в canton-ledger-api/proto/.
- **Перед Chain 3:** submit_domain_commands и create_contract (или эквивалент) должны работать; шаблон InstitutionalAsset:InstitutionalAsset должен быть задеплоен на participant (или использовать sandbox).
- **Перед Chain 4:** TreasuryService полностью реализован и протестирован; GetActiveContracts доступен в LedgerClient.
- **Перед Chain 5:** RealEstateService и PrivacyVaultService (или заглушки) готовы; TreasuryService принимает зависимости через конструктор или builder.
- **Перед Chain 6:** Все цепи 1–5 выполнены; acceptance criteria из Part 8 отмечены; документация обновляется в последнюю очередь.
- **После каждой цепи:** Запустить `cargo test --workspace` и `cargo clippy --workspace -- -D warnings`; при падении — исправить в той же цепи.
- **Контекст при ответе:** Всегда указывать текущую цепь (1–6), использованные секции документа (например Part 3.1, 6.3, 7) и изменённые файлы (полные пути).
- **Длина документа:** Промт не менее 2000 строк; при исполнении можно ссылаться на секции по номерам (Part N) для экономии контекста.
- **Источники:** Все типы, методы и пути взяты из реального кода canton-otc (src/lib/canton/services/*.ts, src/app/api/defi/**/route.ts) и cantonnet-omnichain-sdk (crates/*/src/*.rs). Не добавлять выдуманные эндпоинты или шаблоны.

---

## PART 48 — SOURCES & FILE MANIFEST (KEY FILES)

**DeFi services (canton-otc):** treasuryBillsService.ts, realEstateService.ts, privacyVaultService.ts, damlIntegrationService.ts, complianceService.ts, oracleService.ts, zkProofService.ts, propertyValuationAPI.ts, multiPartyWorkflowService.ts.

**DeFi API routes:** src/app/api/defi/treasury/bills/route.ts, bills/[billId]/route.ts, portfolio/route.ts, purchases/route.ts; oracle/prices/route.ts, oracle/treasury-yields/route.ts; compliance/kyc/route.ts; auth/login, logout, register.

**DeFi hooks:** useTreasuryBills.ts, useRealEstate.ts, usePrivacyVaults.ts, realCantonIntegration.ts, useCantonBridge.ts, useMultiPartyWorkflowService.ts.

**SDK (cantonnet-omnichain-sdk):** crates/canton-core/src/config.rs, types/*.rs; crates/canton-ledger-api/src/client.rs, proto/; crates/canton-crypto, canton-wallet, canton-transport, canton-reliability, canton-observability.

**Config & docs:** config/example.yaml, docs/DEVNET_PARTICIPANT.md, blockchain/DEFI_CONNECT_DEVNET.md, blockchain/prompts/DEFI_SDK_MASTER_PROMPT.md.

---

**END OF MASTER PROMPT**

Итого: документ покрывает роль, prompt chains (1–6), полный инвентарь DeFi по реальному коду, состояние SDK, архитектуру, спецификацию API, пошаговые цепи реализации с подшагами, детальные маппинги типов TypeScript→Rust, полные выдержки TypeScript-интерфейсов (Part 19), критерии приёмки, тесты, риски, глоссарий, чеклисты, шаблон вывода при выполнении цепей, повторные таблицы маршрутов/хуков/шаблонов, пример конфига YAML, предложенная структура крейтов и финальный чеклист; дополнены секции 26–48: дополнительные сервисы (ZK, Valuation, Multi-Party), API body/env, E2E/integration пути, prompt engineering 2025 (RAG, ReAct, few-shot, tool-use), константы Daml-шаблонов, актуальный layout SDK, YieldDistribution, порядок выполнения цепей, quick reference, Rust type skeletons (Part 36), интеграционные сценарии (Part 37), коды ошибок SDK (Part 38), версии зависимостей (Part 39), сводка потока данных DeFi→SDK (Part 40), API request/response JSON (Part 41), Canton Ledger API proto services (Part 42), UI→SDK method mapping (Part 43), recommended implementation order (Part 44), index of all parts (Part 45), env & config quick ref (Part 46), execution reminders (Part 47), sources & file manifest (Part 48). Исполняй по цепям 1→6, с структурированным выводом и проверкой после каждой цепи. Длина документа: 2000 строк.
