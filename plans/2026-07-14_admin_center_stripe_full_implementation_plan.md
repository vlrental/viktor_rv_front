# Viktor RV — полный план Admin Center и Stripe

Дата: 2026-07-14
Статус: UI, backend и production Supabase private storage реализованы. Stripe test-mode refundable-deposit flow, автоматические тесты и реальные Checkout/full-refund/partial-refund smoke checks зелёные. Live activation остаётся отдельным запрещённым этапом.

Обновление 2026-07-15: повторный readiness-аудит закрыл SMTP diagnostics/retry, admin email alerts, browser/customer timezones, public rate limiting, согласованное истечение Checkout objects и обязательные disposable-DB/CI deployment gates. Локально зелёные unit/integration/concurrency проверки не означают разрешение production deployment или live Stripe.

Обновление 2026-07-15, security hardening pass 2: Google callback переведён на hashed five-minute one-time code и server exchange без access/refresh tokens в URL; refresh rotation сделан atomic compare-and-swap, logout отзывает expired-access session, Supabase Data API tables/sequences/default privileges закрыты явно, sensitive booking state перенесён в session storage, body limits разделены по маршрутам, uploads проверяют magic bytes/symlink paths, CSV защищён от spreadsheet formulas. На чистом PostgreSQL 17 прошли все SQL suites, 102 обычных backend tests, 7 payment DB tests, 3 auth race/session tests и 2 booking/block concurrency tests. Production migration/deployment не выполнялись.

Обновление 2026-07-15, final security pass 3: HTTP tracing больше не записывает query strings/headers, server errors логируются только безопасной категорией, sensitive API responses получили `no-store` и browser security headers, OAuth callback ограничен по размеру/формату и rate limit, auth access/refresh tokens перенесены в session storage с полной очисткой legacy local storage. Supabase backend-only perimeter теперь также отзывает schema `USAGE`, все function grants и default function execution у `PUBLIC`/`anon`/`authenticated`. Production backend/DB/live Stripe не изменялись. Финально зелёные: frontend 71 tests + strict clippy + WASM check; backend importer 8 + app 105 tests + strict clippy; PostgreSQL 17 — 5 SQL suites, 7 payment DB tests, 3 auth race/session tests и 2 concurrency tests.
Рабочая ветка: `dev`
Frontend: `/Users/viktoriiakarpova/Projects/it_work/viktor_rv_front`
Backend: `/Users/viktoriiakarpova/Projects/it_work/viktor_rv_back`

## Решение владельца от 2026-07-15 — заменяет все старые пункты про authorization hold

- Extended Authorization больше не является требованием и не используется новым flow.
- За 48 часов до delivery клиент оплачивает отдельный refundable damage deposit CA$1,000 через динамический Stripe Checkout.
- `Delivered` блокируется, пока trip balance и deposit не подтверждены webhook как оплаченные.
- После `Returned` админ либо возвращает весь CA$1,000, либо указывает документированный damage amount; backend возвращает остаток `1000 - damage amount` на исходный payment method.
- Damage retention требует reason, минимум одно private evidence photo, confirmation dialog и audit event. Full CA$1,000 retention не делает фиктивный Stripe refund; решение фиксируется транзакционно поверх уже подтверждённого deposit payment.
- При cancellation уже оплаченный damage deposit автоматически ставится в отдельный полный refund независимо от вручную выбранного trip refund.
- Stripe refund webhook/reconciliation остаётся источником истины для фактического возврата. Ошибка возврата не меняет lifecycle booking и требует admin attention.
- Stripe не возвращает VL Rental processing fee исходной deposit-транзакции; эта комиссия не вычитается из суммы возврата клиенту.
- Внутренние DB compatibility identifiers `damage_hold`, `hold_release` и `damage_capture` временно сохраняются, чтобы не ломать уже применённую production schema; customer/admin labels и бизнес-семантика используют только refundable damage deposit/refund/settlement.
- Любые более старые пункты этого документа про `manual capture`, `capture_before`, другую карту и обязательность Extended Authorization считаются историческим планом и не управляют текущей реализацией.
- Реальный Stripe test-mode smoke 2026-07-15: динамический CA$1,000 Checkout успешно создан и затем expired; отдельный CA$1,000 test charge успешно полностью refunded; отдельный CA$1,000 test charge успешно частично refunded на CA$750 с CA$250 retained. Все smoke Checkout sessions очищены.
Pencil: `/Users/viktoriiakarpova/Projects/it_work/vlrental.pen`

## 0. Фактический статус выполнения на 2026-07-14

В рамках реализации уже выполнено:

- сохранён единый `/admin` без дополнительных постоянных admin routes;
- реализованы встроенные Overview, Bookings, Payments, Calendar и Audit panels;
- реализованы desktop drawers, confirmation modals и полноэкранные mobile overlays;
- добавлены manual phone booking, двухчасовой reserve и динамический hosted Checkout;
- добавлен embedded Stripe Checkout в существующий booking overlay без отдельной callback page;
- добавлены public payment config и приватный token-based payment-status polling;
- реализованы строгая Stripe config/account validation, raw-body webhook signature verification, event idempotency и защита от out-of-order events;
- реализованы initial 30%/100%, balance Invoice, отдельное списание refundable CA$1,000 deposit, полный refund, refund остатка после документированного damage retention и webhook-backed tracking;
- финансовые действия release/capture/refund переведены на durable `payment_operations` со статусами `pending/submitted/succeeded/failed`, стабильными idempotency keys, retries и webhook/reconciliation completion;
- cancellation освобождает календарь до обращения к Stripe, а refund распределяется по нескольким PaymentIntent для сценария оплаты 30% + 70%; каждая часть отслеживается отдельно;
- письмо `damage captured` отправляется только после подтверждённого webhook/direct reconciliation результата;
- реализован DB-backed worker с `FOR UPDATE SKIP LOCKED`, восстановлением зависших claims, retries и email notification queue;
- реализован основной admin API, guarded lifecycle transitions, audit log и authenticated CSV export;
- реализована multipart-загрузка evidence с проверкой MIME/magic bytes/10 MiB, приватным local/test adapter и backend-only Supabase Storage adapter;
- просмотр evidence защищён короткоживущими backend-authorized ссылками/токенами; публичные постоянные URL запрещены, live Stripe требует Supabase storage mode;
- приватные evidence objects загружаются с `Cache-Control: 0`, чтобы CDN/browser cache не пережил срок короткоживущей signed URL;
- контакты хранятся как snapshot конкретной брони: редактирование контакта из админки больше не меняет общую customer record;
- audit log дополнен для manual booking, calendar block create/delete, resend, evidence access/upload, reconciliation и финансовых действий;
- добавлена additive migration, RLS/revokes, private Supabase bucket definition и SQL safety tests;
- синхронизированы оба `AGENTS.md`, `BOOKING_FLOW_HANDOFF.md` и customer Terms;
- frontend прошёл `cargo fmt --check`, strict `clippy`, 59 tests и WASM check, включая regression tests после browser fixes;
- backend прошёл `cargo fmt --check`, `cargo check`, strict `clippy`, 92 обычных tests (8 importer + 84 application), 6 отдельных webhook/DB tests и 2 disposable-DB concurrency tests;
- 2026-07-15 повторная изолированная проверка на чистом disposable PostgreSQL 17 подтвердила применение `sql/schema.sql`, booking schedule safety, admin/Stripe foundation safety, admin data security, все 6 ignored webhook/DB tests и оба concurrency tests; временная база удалена после прогона;
- 2026-07-15 через production `DATABASE_URL` из ignored backend `.env.prod` в правильный Supabase project `pwhlkpwlansarstmstge` применены три ожидавшие versioned migrations: admin/Stripe foundation, financial reconciliation и admin data security;
- после production migration повторно прошли все три SQL safety suites; подтверждены RLS/revokes, финансовые/admin таблицы и private bucket `damage-evidence` с лимитом 10 MiB;
- реальный private Storage E2E прошёл с временно использованным backend-only service role: upload и немедленный signed access успешны, public access и просроченная signed URL отклонены, тестовый object удалён; секрет не записывался в репозиторий или документацию;
- 2026-07-15 backend переведён на предпочтительный современный `SUPABASE_SECRET_KEY=sb_secret_...` с legacy fallback: новый секрет отправляется только через `apikey`, publishable key в secret-настройке отклоняется; повторный production Storage smoke test подтвердил upload → signed download → delete → post-delete denial, временный object удалён;
- Stripe HTTP contract tests проверяют динамические Checkout, Invoice, refundable deposit, refund и account verification; старые manual-capture fixtures сохранены только как compatibility coverage и не определяют новый flow;
- migration/bootstrap повторно применялись на временном PostgreSQL, RLS/revoke/lifecycle assertions прошли;
- RustSec audit не нашёл reachable advisories во frontend/backend; SQLx переведён на PostgreSQL-only features без default Any/MySQL/SQLite graph. `rsa` advisory остаётся только как неиспользуемая optional lockfile-ветка derive macro и отсутствует в `cargo tree --all-features --target all`; `spin 0.9.8` остаётся yanked-warning актуального Axum/multer, но не зарегистрированной security vulnerability;
- secrets, mojibake, устаревший термин `Gold`, diff formatting и live/prod mutations проверены.

Ещё не выполнено и намеренно заблокировано:

- live Stripe keys, live webhook и реальные клиентские списания;
- production backend restart/deployment, frontend production deployment и переключение домена;
- финальный live-readiness прогон после отдельного прямого разрешения владельца и только с теми live-настройками, которые будут выданы для запуска.

### 0.1 Исправления после сквозного integration review

После первой реализации выполнена отдельная проверка стыков frontend/backend/Stripe. По её результатам уже исправлено:

- public Checkout получает безопасный запас сверх минимального Stripe expiry; manual reserve остаётся ровно двухчасовым;
- hosted Checkout телефонной брони возвращает клиента в существующий customer payment-return flow, а не на закрытую `/admin`;
- Stripe webhook блокирует obligation row и проверяет booking/obligation metadata, payment type, amount, currency, environment и привязку Checkout/Invoice object ID;
- terminal successful refund/payment states больше не понижаются поздним failed event;
- если initial Checkout не создаётся или не сохраняется, reservation сразу истекает, а уже созданная Stripe Session компенсирующе закрывается;
- expired damage-hold Checkout возвращает obligation в retry flow;
- каждая новая попытка damage hold использует versioned Stripe idempotency key и новую Session;
- cancellation сначала освобождает календарь, затем идемпотентно закрывает Checkout, void Invoice и отменяет damage PaymentIntent;
- закрыта гонка, когда damage hold авторизуется одновременно с cancellation;
- при выключенных test payments кнопка Phone booking во frontend недоступна;
- live mode дополнительно заблокирован отдельным `STRIPE_LIVE_ACTIVATION_APPROVED=false` gate;
- добавлены идемпотентные customer/admin operational email queues с максимум восемью попытками и backoff.

### 0.2 Оставшиеся проверки до live phase

Продуктовое решение уже принято: используется refundable damage deposit, а Extended Authorization больше не нужен. Test phase закрывает код, данные, UI и Stripe test objects. Следующий этап начинается только по отдельной прямой команде владельца и включает установку live keys вне Git, live webhook, controlled low-risk live verification, production deployment и отдельное подтверждение доменного запуска.

### 0.3 Исторический Stripe test-mode отчёт до смены deposit-модели

Stripe CLI авторизован в test mode именно для `102181797 Saskatchewan Ltd.` / `acct_1SpY7K2MR4C4rvKM`. Test API key, publishable key и CLI webhook secret записаны только в локальный ignored `.env` с правами `0600`; `.env` не входит в Git. Live key не использовался. Backend при старте вызвал Stripe `/account`, подтвердил ожидаемый account ID и публично вернул только `enabled=true`, `mode=test`, `pk_test_…` и account ID.

Фактически проверено на Stripe test objects:

- backend создал embedded Checkout `cs_test_a1QEIFwqNc9WqNK3dAg89ZIpfeHkYJ3Oe7DSuUCYXpzDH9VCagDF2vLLa5` на CA$391.63 — ровно 30% от backend quote CA$1,305.44; `livemode=false`, `ui_mode=embedded`, metadata и environment корректны;
- принудительное закрытие этого Checkout создало настоящий `checkout.session.expired`; подписанный CLI webhook получил HTTP 200, бронь стала `expired`, а даты снова стали доступны;
- direct test PaymentIntent подтвердил обычный success; отдельные тесты подтвердили decline `card_declined/generic_decline`, а реальный embedded Checkout с картой `4000 0025 0000 3155` прошёл вложенный Stripe 3DS challenge до подтверждённой брони;
- обычная CA$1,000 manual-capture авторизация `pi_3TtH5r2MR4C4rvKM12e57qUP` перешла в `requires_capture`, вернула реальный семидневный `capture_before` и `extended_authorization.status=disabled`; release перевёл её в `canceled`;
- отдельная CA$1,000 authorization `pi_3TtH642MR4C4rvKM0arG2fy9` была частично capture'нута на CA$250 и перешла в `succeeded`, неиспользованный остаток освободился;
- payment success и refund `re_3TtH6O2MR4C4rvKM1DjokKD9` на CA$23.45 прошли; завышенная сумма refund была ожидаемо отклонена Stripe;
- backend worker создал Hosted Invoice `in_1TtHJx2MR4C4rvKM2vhJ1B9L` на CA$913.81, явно привязал Invoice Item, установил due date на тот же UTC-день и сохранил Hosted Invoice URL;
- реальное `invoice.paid` для этого Invoice получило HTTP 200; balance obligation стала `succeeded`, booking payment status — `paid`;
- отдельный реальный Hosted Invoice `in_1TtKgS2MR4C4rvKMibo6k5T7` был доступен по Hosted Invoice Page, получил `generic_decline`, подписанный `invoice.payment_failed` webhook и статус `failed`; после назначения другой test-карты тот же Invoice успешно оплатился, подписанный webhook перевёл balance obligation в `succeeded`, а booking — в `paid`, без создания второго Invoice;
- manual phone booking через admin API создала hosted Checkout на backend quote, вернула URL, установила reserve ровно на 7,200 секунд и записала audit events;
- клиентский embedded Checkout реально открыт внутри unified overlay: test card с insufficient-funds decline показала ошибку без создания второй брони, затем `4242 4242 4242 4242` успешно завершила оплату;
- бронь `VL-20260714-00000007` после подписанного webhook стала `confirmed / partially_paid`: initial CA$310.99 — `succeeded`, balance CA$725.65 и damage hold CA$1,000 — отдельные `scheduled` obligations;
- бронь `VL-20260714-00000008` прошла интерактивный embedded 3DS и затем реальный balance Invoice failure/alternate-card/retry cycle; admin Payments и detail drawer показали итоговые webhook-backed состояния;
- исправлена ложная ошибка mount результата Dioxus eval (`return await`), восстановление сохранённой pending-сессии сразу на step 5 и реактивная гонка ожидания payment config; добавлены regression tests;
- роль `admin` получила HTTP 200 для dashboard/bookings/payments/audit/detail, включая `damage_claims` и новые payment IDs; отдельная роль `default` получила HTTP 403 и не увидела admin data;
- повторная доставка одного подписанного event дважды вернула HTTP 200 и сохранила только одну обработанную запись; посторонний подписанный `payment_intent.succeeded` без Viktor metadata теперь сохраняется как `ignored` и получает 2xx;
- mock и disposable-DB tests подтверждают old/new Stripe Invoice JSON, duplicate/out-of-order events, durable multi-PaymentIntent refund, release/capture reconciliation и worker recovery.

Исторический внешний результат, объясняющий смену модели:

- запрос `payment_method_options[card][request_extended_authorization]=if_available` отклоняется Stripe кодом `payment_intent_invalid_parameter` с сообщением, что account не eligible для requested card feature;
- backend-connected damage-hold worker безопасно сохраняет obligation как retryable `failed/provider_error`, не раскрывает provider body и не разрешает доставку;
- standard authorization даёт только около семи дней, чего недостаточно для гарантии `return + 7 days`; поэтому ослаблять gate или использовать standard hold как скрытый fallback запрещено;
- Gate E/F и live activation остаются закрыты. 2026-07-15 Stripe Priority Support ответил по case `sco_Ut5ECLsXyQP2d9`, что account не соответствует базовым требованиям Extended Authorization; критерии и путь включения не предоставлены.
- Stripe предложил SetupIntent с возможным последующим списанием либо refundable deposit. После этого владелец отдельно утвердил refundable deposit; актуальное решение и результаты его test-mode проверки зафиксированы в начале документа и заменяют этот исторический этап.

## 1. Цель

Создать внутри сайта Viktor RV полноценный административный центр для управления RV-бронированиями, платежами, refundable damage deposits, календарём доступности и журналом действий.

Административный интерфейс должен оставаться одной постоянной страницей `/admin`. Overview, Bookings, Payments, Calendar и Audit работают как встроенные вкладки. Детали, ручная бронь и чувствительные действия открываются в drawers и modals поверх текущей страницы, без лишних переходов и отдельных callback-страниц.

Платежная интеграция сначала реализуется и проверяется исключительно в Stripe test mode. Live-ключи, production-платежи, production backend, домен и live webhook не включаются без отдельного прямого разрешения владельца проекта.

## 1.1 Актуальная схема готового решения

### Клиентский flow

1. Backend считает immutable quote и отдельно показывает trip price и refundable CA$1,000 damage deposit.
2. Если до delivery больше 30 дней, embedded Checkout берёт 30% trip price; иначе — 100%.
3. Только verified Stripe webhook подтверждает initial payment и booking.
4. При 30% оплате worker ровно за 30 дней создаёт один Hosted Invoice на остаток 70% и отправляет ссылку.
5. За 48 часов worker создаёт отдельный dynamic Checkout на CA$1,000 refundable deposit и отправляет ссылку клиенту.
6. `Delivered` разрешается только при 100% trip payment и webhook-confirmed paid deposit.
7. После `Returned` админ либо возвращает весь депозит, либо фиксирует damage amount, reason и private evidence; клиенту возвращается остаток.
8. При cancellation даты освобождаются сразу, trip refund отслеживается отдельно, а уже оплаченный damage deposit автоматически получает отдельный полный refund.

### Admin Center

- Единственная постоянная страница `/admin`; Overview, Bookings, Payments, Calendar и Audit встроены как панели.
- Booking detail, manual booking и финансовые подтверждения открываются в drawers/modals; `Escape` закрывает верхний dismissible layer.
- Телефонная бронь резервирует RV на два часа и использует те же backend quote и Stripe Checkout.
- Lifecycle actions guarded: Delivered, Returned и Cancel не заменяются произвольным PATCH статуса.
- Любой refund/deposit settlement требует confirmation dialog; damage retention дополнительно требует сумму, reason и минимум одно private photo.
- Все административные и финансовые действия попадают в immutable audit log; есть CSV export.

### Backend, Stripe и данные

- `STRIPE_MODE=test` и test keys проверяются вместе с ожидаемым account `acct_1SpY7K2MR4C4rvKM`; mismatch останавливает backend.
- Secret key и webhook secret существуют только в ignored/server environment. Frontend получает только publishable key, mode и account ID.
- Stripe metadata связывает booking, obligation, payment type и environment; фиксированные Price IDs не используются.
- Webhook signature, duplicate/out-of-order events, durable operations, retries и recovery после restart обработаны backend.
- Refundable deposit внутренне использует прежний DB payment type `damage_hold`, а операции `hold_release`/`damage_capture` сохранены исключительно для совместимости применённой schema.
- Private damage evidence хранится в Supabase Storage; современный `SUPABASE_SECRET_KEY=sb_secret_...` отправляется только как server-side `apikey`, signed access короткоживущий.
- Financial/admin tables закрыты от `anon` и `authenticated`; UI получает их только через role-checked backend API.

### Проверенные сценарии

- Финальный автоматический прогон: frontend 59/59 tests, strict Clippy и WASM check; backend 8 importer + 88 application tests, strict Clippy; дополнительно 6/6 ignored webhook/financial tests и 2/2 concurrency tests на одноразовой PostgreSQL schema.
- Расчёт 30%/100%, balance и отдельного CA$1,000 deposit.
- Dynamic initial/manual/deposit Checkout и Hosted Invoice contracts.
- Success, decline, 3DS, expired Checkout, Invoice paid/failed и повторная карта в test mode.
- Full deposit refund и partial refund CA$750 при CA$250 documented retention на реальных Stripe test objects.
- Webhook idempotency, out-of-order protection, retries, cancellation before provider calls и payment after cancellation.
- Delivery gate, full refund, partial/full damage settlement, evidence requirements и seven-day overdue warning.
- Role isolation, drawers/modals/uploads/`Escape`, desktop/mobile behavior, secret scans, RLS/revokes и private Storage round-trip.
- Новый backend запущен напрямую на отдельном локальном порту с реальной test-конфигурацией: health 200, `mode=test`, `pk_test_` и ожидаемый account ID; несколько worker cycles прошли без ошибок. Production не затрагивался.

### Единственный следующий этап

После отдельной прямой команды владельца: получить live secrets вне Git, зарегистрировать live webhook, повторить account/mode/security gates, выполнить контролируемый live verification, подготовить итоговый отчёт и только затем отдельно решать production deployment и доменный cutover.

## Appendix A — исторический baseline до решения о refundable deposit

Все пункты ниже сохранены как аудит первоначального плана и выполненной миграции решения. Любые упоминания manual capture, `capture_before` или обязательного Extended Authorization ниже не являются текущими требованиями и не должны возвращаться в код/UI без нового прямого решения владельца.

## 2. Исторически зафиксированные бизнес-решения

### 2.1 Проект и доступ

- Проект обслуживает только RV. Boats, boat bookings и смешанные Stripe-продукты не используются.
- Админка доступна только пользователям с backend-ролью `admin`.
- Роли остаются `default` и `admin`; отдельная роль `owner` в первой версии не добавляется.
- Любой `admin` может выполнять финансовые действия, но destructive/financial операции требуют отдельного confirmation modal и записи в audit log.
- Админ-интерфейс выполняется на английском языке.
- Все административные API повторно проверяют серверную сессию и роль, независимо от видимости ссылки во frontend.

### 2.2 Первая оплата

- Если RV доставляется более чем через 30 дней, при бронировании оплачивается 30% trip price.
- Если RV доставляется через 30 дней или раньше, при бронировании оплачивается 100% trip price.
- CA$1,000 damage authorization hold не входит в trip price и не входит в расчёт 30%.
- Первая оплата выполняется через embedded Stripe Checkout внутри существующего booking overlay.
- Бронь создаётся в статусе `pending_payment` и блокирует выбранные даты на ограниченное время.
- Только проверенный Stripe webhook подтверждает успешную оплату и переводит бронь в `confirmed`.
- Frontend callback, success URL или наличие Checkout success state не могут самостоятельно подтвердить оплату.
- Обычный quote/checkout reservation действует 30 минут.
- При истечении срока неоплаченная бронь получает `expired`, после чего даты освобождаются.

### 2.3 Остаток trip price

- Для брони с первоначальной оплатой 30% оставшиеся 70% становятся due ровно за 30 дней до доставки.
- Backend автоматически создаёт Stripe Invoice и Hosted Invoice Page.
- Клиент немедленно получает SES-письмо с прямой безопасной ссылкой.
- Администратор видит due/failed/paid status и может повторно отправить ссылку.
- Повторная отправка не создаёт второй invoice: используются idempotency и существующая payment obligation.
- Источником истины остаются Stripe webhook events.

### 2.4 Damage authorization hold

- Термин `Gold` был ошибочным. Во всех правилах, Terms и UI он заменяется на `Stripe authorization hold` или `damage authorization hold`.
- За 48 часов до доставки клиент получает ссылку для авторизации CA$1,000.
- Для hold используется PaymentIntent/Checkout с manual capture и запросом extended authorization.
- Деньги не списываются сразу; Stripe резервирует сумму на карте.
- После подтверждения сохраняются `capture_before`, статус extended authorization и сумма, доступная для capture.
- Конкретная карта может не поддержать достаточно длинный hold даже при доступной функции аккаунта.
- Если `capture_before` не покрывает аренду и inspection deadline, authorization немедленно отменяется, клиенту предлагается другая карта, а админ получает предупреждение.
- Реальное списание CA$1,000 как автоматический fallback не используется.
- Без активного и достаточно длинного hold нельзя отметить RV как `Delivered`.
- Без полностью оплаченного trip balance также нельзя отметить RV как `Delivered`.
- Если extended authorization нельзя надёжно подтвердить в Stripe test mode, live payments не включаются.

### 2.5 Delivered, Returned и решение по hold

- Администратор вручную выполняет последовательность `Delivered` → `Returned`.
- Backend хранит совместимые статусы: `active` соответствует Delivered, `completed` соответствует Returned.
- После `Returned` становятся доступны действия Release hold и Capture damage.
- Hold можно снять сразу после осмотра, если ущерба нет.
- Семь дней после Returned — крайний срок решения, а не автоматическая дата refund.
- После истечения семи дней dashboard показывает overdue warning и отправляет admin reminder.
- Автоматический refund или автоматический capture не выполняется.
- Release hold отменяет uncaptured PaymentIntent и освобождает всю авторизованную сумму.
- Damage capture может быть частичным или полным в пределах CA$1,000.
- Для damage capture обязательны сумма, текстовая причина и минимум одна фотография.
- При partial capture остаток authorization освобождается клиенту.
- Клиент получает письмо с результатом и детализацией удержания.

### 2.6 Отмена и refund

- Политика автоматического расчёта cancellation refund в первой версии не кодируется.
- Администратор вручную вводит сумму refund и обязательную причину.
- Confirmation modal показывает уже уплаченную сумму, refund amount и последствия.
- Даты RV освобождаются сразу после подтверждения отмены.
- Stripe refund отслеживается отдельно; неуспешный refund не возвращает бронь в календарь.
- Ошибка refund становится critical admin action с возможностью повторной проверки/повтора.
- Каждая отмена и refund записываются в audit log.

### 2.7 Ручная телефонная бронь

- Создаётся в drawer поверх `/admin`, без отдельной страницы.
- Использует тот же backend quote и те же pricing rules, что клиентский booking flow.
- Админ вводит клиента, RV, даты, delivery address и notes.
- После создания RV резервируется на два часа.
- Клиент получает динамическую Stripe Checkout Session URL, а не статический Payment Link или fixed Price ID.
- Если клиент не оплачивает за два часа, бронь получает `expired`, а даты освобождаются.
- В первой версии админ не отмечает cash/bank/offline payment как `paid`.

### 2.8 Редактирование брони

- Админ может изменять контактные данные клиента и внутренние admin notes.
- Даты, RV, quote и цену уже оплаченной брони нельзя редактировать напрямую.
- Для изменения финансовых или календарных параметров используется отмена и новая бронь.
- Это предотвращает рассинхронизацию quote, Stripe objects и availability constraints.

### 2.9 Уведомления

- Customer emails: booking confirmed, balance link, damage hold link, payment failure/retry link, cancellation/refund result, hold released или damage captured.
- Admin immediate emails: новая подтверждённая бронь, initial/balance/hold failure, unsupported hold, cancellation/refund failure и приближающийся `capture_before`.
- Dashboard badges показывают все текущие действия.
- Просроченные действия входят в ежедневную admin-сводку.
- Администратор может повторно отправить customer payment link, не создавая новый долг.

## 3. Stripe account и ограничения

Подключённый аккаунт:

- Display name: `102181797 Saskatchewan Ltd.`
- Account ID: `acct_1SpY7K2MR4C4rvKM`

Обнаруженные Stripe products:

- `rv + boat payment` — не использовать, потому что продукт смешивает RV и boats.
- `Damage deposit` — не использовать как fixed CA$1,000 Price для динамического hold flow.
- `item` — тестовый продукт, не использовать в production architecture.

Все суммы передаются Stripe из immutable backend quote/payment obligation через inline amount/price data. Stripe Dashboard products не являются источником цены.

## 4. Уже выполнено

### 4.1 Аудит текущей реализации

- Подтверждено наличие `/admin` во frontend.
- Подтверждена серверная роль `app_users.role` со значениями `default`/`admin`.
- Подтверждены существующие admin endpoints для списка бронирований, изменения статуса и availability blocks.
- Подтверждено, что текущий frontend admin показывает базовые bookings и closed dates, но не использует полный payment/action flow.
- Подтверждено, что backend payments integration пока является stub.
- Подтвержден `BOOKING_TEST_MODE=true` и текущая логика `confirmed`/`test_paid` без реальной карты.
- Подтверждена существующая таблица `payments`, которую нужно расширить, а не создавать несовместимую параллельную модель.
- Подтверждено отсутствие production scheduler для due payments.
- Подтверждено, что backend развёрнут одним Rust/Axum container и может запускать DB-backed worker внутри приложения.

### 4.2 Pencil-дизайн

Все состояния представляют одну страницу `/admin`, а не отдельные frontend routes.

Desktop:

- `HBpPe` — Overview tab state.
- `K8YXx` — Bookings tab с таблицей и открытым booking detail drawer.
- `p8p6P` — Payments tab с obligations, failed payment и resend/retry actions.
- `l86oa9` — Calendar tab с единым календарём bookings/owner blocks и inline close-dates panel.
- `tjohf` — Audit tab с filters, immutable timeline и event detail.

Mobile:

- `JNkTK` — Overview tab state.
- `o6Esvq` — Bookings tab state.
- `IfQL5` — Payments tab state.
- `BvlyF` — Calendar embedded state под `More`.
- `RRAEN` — Audit embedded state под `More`.
- `GBXFF` — полноэкранный booking detail drawer overlay.
- `mgKti` — полноэкранный phone booking drawer overlay.

Overlays:

- `FoWiG` — desktop phone booking drawer.
- `ujgoM` — damage capture modal с amount, reason, photo и breakdown.
- `lNyUh` — release hold, cancellation/refund, Delivered blocked и Returned confirmations.
- `ANiTF` — allowed Mark Delivered confirmation.

Для всех новых/изменённых Pencil frames выполнена layout-проверка. Clipping, overflow и collapsed layout не обнаружены.

### 4.3 Реализация после первоначального аудита

- Оба `AGENTS.md`, `BOOKING_FLOW_HANDOFF.md` и Terms синхронизированы.
- Frontend/backend foundation реализован по этому документу.
- SQL migration и safety tests созданы и локально проверены повторным применением.
- Stripe CLI test credentials и webhook secret настроены только в локальном ignored `.env`; account ID подтверждён backend startup check.
- Реальные test objects и webhooks прогнаны; подробный отчёт находится в разделе 0.3.
- Production, live keys, domain и deployment не изменялись.

## 5. UX-архитектура одной страницы `/admin`

### 5.1 Desktop

- Persistent page shell: admin header, Stripe environment indicator и tabs.
- Tabs меняют встроенную панель без route navigation.
- Booking row открывает правый drawer, сохраняя список и фильтры на месте.
- Phone booking открывает правый drawer.
- Financial/lifecycle confirmations открываются modal поверх drawer.
- `Escape` закрывает только верхний dismissible layer.
- После modal success drawer остаётся открытым и обновляет данные.

### 5.2 Mobile

- Overview, Bookings и Payments доступны напрямую.
- Calendar и Audit находятся в компактном `More` с внутренними subtabs.
- Booking detail и Phone booking становятся полноэкранными drawers, а не routes.
- Confirmation modal занимает безопасную sheet/fullscreen область.
- Возврат закрывает overlay и сохраняет выбранный tab, filter и scroll position.

### 5.3 Обязательные состояния UI

Для каждой data surface реализуются:

- loading;
- empty;
- permission denied;
- request error;
- Stripe/webhook delayed;
- success confirmation;
- retrying;
- test mode;
- live mode blocked/configuration error.

## 6. Booking lifecycle и invariants

Допустимая последовательность:

```text
pending_payment
  ├─ Stripe initial payment succeeded → confirmed
  ├─ payment window expired → expired
  └─ admin cancellation → cancelled

confirmed
  ├─ requirements passed + admin confirms delivery → active (Delivered)
  └─ admin cancellation → cancelled

active (Delivered)
  └─ admin confirms return → completed (Returned)

completed (Returned)
  ├─ release hold
  └─ partial/full damage capture
```

Backend invariants:

- `pending_payment` блокирует календарь только до `payment_expires_at`.
- `expired` и `cancelled` не блокируют календарь.
- `mark_delivered` разрешён только из `confirmed`.
- `mark_delivered` требует полностью оплаченный trip price.
- `mark_delivered` требует активный CA$1,000 authorization hold с достаточным `capture_before`.
- `mark_returned` разрешён только из `active`.
- Release/capture damage разрешены только после `completed`.
- Release/capture выполняются только один раз.
- Generic arbitrary status PATCH больше не используется frontend; API принимает action-specific transitions.

## 7. План схемы данных

Все изменения выполняются через проверяемую SQL migration в backend repository.

### 7.1 Расширение `bookings`

Добавить:

- `payment_expires_at TIMESTAMPTZ` — срок pending reservation.
- `admin_notes TEXT` — внутренние заметки, не попадающие в public API.
- `delivered_at TIMESTAMPTZ`.
- `returned_at TIMESTAMPTZ`.
- `cancelled_at TIMESTAMPTZ`.
- `cancelled_by UUID` → `app_users`.
- `cancellation_reason TEXT`.

Сохранить existing `customer_notes` отдельно от `admin_notes`.

### 7.2 `payment_obligations`

Описывает ожидаемое обязательство независимо от Stripe object:

- `obligation_id UUID PRIMARY KEY`.
- `booking_id UUID NOT NULL`.
- `obligation_type TEXT`: `initial`, `balance`, `damage_hold`.
- `amount NUMERIC(12,2)`.
- `currency TEXT`.
- `due_at TIMESTAMPTZ`.
- `expires_at TIMESTAMPTZ`.
- `status TEXT`: `scheduled`, `due`, `link_created`, `pending`, `succeeded`, `failed`, `authorized`, `released`, `captured`, `cancelled`, `expired`.
- `attempt_count INTEGER`.
- `next_attempt_at TIMESTAMPTZ`.
- `last_error_code TEXT`.
- `last_error_message TEXT` с очищенными безопасными данными.
- `created_at`, `updated_at`.

Unique constraint предотвращает дубли одного `obligation_type` для booking, кроме явно версионированного retry/replacement hold.

### 7.3 Расширение `payments`

Существующая таблица становится журналом provider objects и денежных результатов:

- `obligation_id UUID`.
- `provider_object_type TEXT`: `checkout_session`, `payment_intent`, `invoice`, `refund`.
- `provider_reference TEXT UNIQUE`.
- `provider_account_id TEXT`.
- `environment TEXT`: `test`, `live`.
- `payment_type TEXT`: `initial`, `balance`, `damage_hold`, `refund`.
- `status TEXT`.
- `amount NUMERIC(12,2)`.
- `amount_authorized NUMERIC(12,2)`.
- `amount_captured NUMERIC(12,2)`.
- `amount_refunded NUMERIC(12,2)`.
- `currency TEXT`.
- `capture_before TIMESTAMPTZ`.
- `extended_authorization_status TEXT`.
- `hosted_url TEXT` только в backend-private таблице/API.
- `last_provider_event_created_at TIMESTAMPTZ` для защиты от старых событий.
- `created_at`, `updated_at`.

Никакие card number, CVC, client secret или raw payment method details не сохраняются.

### 7.4 `stripe_events`

- `stripe_event_id TEXT PRIMARY KEY`.
- `environment TEXT`.
- `event_type TEXT`.
- `provider_object_id TEXT`.
- `provider_created_at TIMESTAMPTZ`.
- `payload_hash TEXT`.
- `processing_status TEXT`: `received`, `processed`, `ignored`, `failed`.
- `processing_error TEXT`.
- `received_at`, `processed_at`.

Webhook transaction сначала регистрирует event ID. Повторный event возвращает HTTP 2xx без повторной бизнес-операции.

### 7.5 `damage_claims` и `damage_evidence`

`damage_claims`:

- booking/payment reference;
- claimed amount;
- required reason;
- status `draft`, `submitted`, `captured`, `failed`;
- actor admin;
- Stripe capture reference;
- timestamps.

`damage_evidence`:

- claim ID;
- private object key;
- original filename;
- safe MIME type;
- byte size;
- SHA-256;
- uploader admin;
- created timestamp.

Файлы хранятся в private storage bucket. В первой версии разрешаются JPEG, PNG и WEBP с ограничением размера. Публичные permanent URLs запрещены; просмотр выполняется через короткоживущую signed URL, выданную backend после admin authorization.

### 7.6 `admin_audit_events`

- `audit_event_id UUID`.
- `actor_user_id UUID` или system actor.
- `booking_id UUID NULL`.
- `action TEXT`.
- `entity_type`, `entity_id`.
- `before_data JSONB` с безопасными полями.
- `after_data JSONB` с безопасными полями.
- `reason TEXT`.
- `request_id TEXT`.
- `created_at`.

Audit events не редактируются и не удаляются через admin API.

### 7.7 `notification_deliveries`

- booking/obligation reference;
- notification type;
- recipient;
- provider reference;
- status;
- attempt count;
- last error;
- sent timestamp;
- created/updated timestamp.

### 7.8 Supabase security

Для новых финансовых/admin таблиц:

- включить RLS;
- `REVOKE ALL FROM anon, authenticated`;
- не добавлять public Data API policies;
- все операции выполнять через backend database role;
- проверить, что новые таблицы не появились в доступном клиенту GraphQL/REST API;
- evidence bucket сделать private.

## 8. Backend configuration

Добавить в `Config`:

```text
STRIPE_MODE=test|live
STRIPE_SECRET_KEY=sk_test_...|sk_live_...
STRIPE_PUBLISHABLE_KEY=pk_test_...|pk_live_...
STRIPE_WEBHOOK_SECRET=whsec_...
STRIPE_EXPECTED_ACCOUNT_ID=acct_1SpY7K2MR4C4rvKM
PAYMENTS_ENABLED=true|false
STRIPE_EXTENDED_AUTH_REQUIRED=true
PAYMENT_WORKER_INTERVAL_SECONDS=60
```

Правила startup validation:

- `STRIPE_MODE=test` принимает только `sk_test_` и `pk_test_`.
- `STRIPE_MODE=live` принимает только `sk_live_` и `pk_live_`.
- `PAYMENTS_ENABLED=true` требует все Stripe values.
- Полученный через Stripe Account API account ID должен совпадать с `STRIPE_EXPECTED_ACCOUNT_ID`.
- Несовпадение environment/account останавливает payments subsystem или весь backend до безопасного исправления.
- Secret values не печатаются в logs и error responses.
- Frontend получает только publishable key, mode и account ID через public config endpoint.

## 9. Public API

### 9.1 Payment config

`GET /api/v1/payments/config`

Response:

```json
{
  "enabled": true,
  "mode": "test",
  "publishable_key": "pk_test_...",
  "account_id": "acct_1SpY7K2MR4C4rvKM"
}
```

Endpoint никогда не возвращает secret key или webhook secret.

### 9.2 Создание клиентской брони

`POST /api/v1/bookings`

- Проверяет quote expiry и availability в транзакции.
- Создаёт `pending_payment` booking и initial obligation.
- Создаёт embedded Checkout Session с idempotency key.
- Stripe metadata: booking ID, booking number, obligation ID, payment type и environment.
- Возвращает booking token, Checkout `client_secret` и `payment_expires_at`.

Frontend не передаёт Stripe amount; backend использует сохранённый quote.

### 9.3 Booking/payment status

`GET /api/v1/bookings/{booking_id}/payment-status`

- Требует private booking access token.
- Возвращает только безопасный booking/payment summary.
- Используется overlay polling после Stripe onComplete, пока webhook не подтвердит результат.

### 9.4 Stripe webhook

`POST /api/v1/stripe/webhook`

- Принимает raw body.
- Проверяет `Stripe-Signature` через test/live webhook secret.
- Регистрирует event ID до изменения бизнес-состояния.
- Обрабатывает события идемпотентно и транзакционно.
- Возвращает 2xx для уже обработанного event.
- Не доверяет metadata без проверки booking/obligation в БД.

Минимальный набор событий определяется по используемым Stripe APIs, включая Checkout completion/expiration, PaymentIntent success/failure/cancel, invoice paid/payment_failed/void и refund updates.

## 10. Admin API

Все endpoints используют `require_admin`.

### 10.1 Dashboard и lists

- `GET /api/v1/admin/dashboard` — metrics, attention queue, today schedule.
- `GET /api/v1/admin/bookings` — filters/search/pagination.
- `GET /api/v1/admin/bookings/{booking_id}` — полный drawer detail.
- `GET /api/v1/admin/payments` — obligations/provider states.
- `GET /api/v1/admin/audit-events` — filters/pagination.
- `GET /api/v1/admin/audit-events.csv` — CSV export.

### 10.2 Manual booking

- `POST /api/v1/admin/bookings/manual`.
- Использует backend quote и two-hour reservation.
- Создаёт hosted Checkout Session URL.
- Отправляет SES email.
- Возвращает booking detail и notification status.

### 10.3 Customer и notes

- `PATCH /api/v1/admin/bookings/{booking_id}/customer` — first/last name, email, phone.
- `PATCH /api/v1/admin/bookings/{booking_id}/notes` — admin notes.
- Quote, dates, RV и price в этих endpoints отсутствуют.

### 10.4 Guarded lifecycle actions

Вместо generic arbitrary status update:

- `POST /api/v1/admin/bookings/{booking_id}/mark-delivered`.
- `POST /api/v1/admin/bookings/{booking_id}/mark-returned`.
- `POST /api/v1/admin/bookings/{booking_id}/cancel` с refund amount/reason.

Backend валидирует current state и invariants в транзакции.

### 10.5 Payment actions

- `POST /api/v1/admin/payment-obligations/{id}/resend-link`.
- `POST /api/v1/admin/payments/{id}/refresh-status` — read/reconcile, не ручное `mark paid`.
- `POST /api/v1/admin/bookings/{id}/damage-hold/release`.
- `POST /api/v1/admin/bookings/{id}/damage-evidence` — multipart upload.
- `POST /api/v1/admin/bookings/{id}/damage-hold/capture` — amount, reason, evidence IDs.

Каждый action endpoint поддерживает idempotency/request ID и создаёт audit event.

## 11. Stripe flows

### 11.1 Embedded initial Checkout

1. Backend получает действующий quote.
2. Транзакционно создаёт pending booking и initial obligation.
3. Создаёт Checkout Session `ui_mode=embedded` с inline amount.
4. Frontend монтирует Stripe Checkout внутри booking overlay.
5. Stripe UI сообщает frontend о completion, но booking остаётся pending.
6. Webhook подтверждает payment и переводит booking в confirmed.
7. Frontend polling получает confirmed state и показывает существующее booking confirmation state.

Новая standalone Stripe callback page не создаётся.

### 11.2 Scheduled balance invoice

1. Worker находит due balance obligation.
2. DB transaction claim предотвращает параллельное создание.
3. Создаётся Stripe Invoice/Invoice Item с booking metadata и idempotency key.
4. Invoice finalizes, получается Hosted Invoice Page URL.
5. SES отправляет customer email.
6. Webhooks обновляют paid/failed status.
7. Admin может resend существующую URL.

### 11.3 Damage authorization

1. За 48 часов worker создаёт customer authorization flow.
2. PaymentIntent использует manual capture и extended authorization request.
3. Клиент подтверждает карту.
4. Webhook сохраняет `requires_capture`, `capture_before` и extended status.
5. Backend проверяет, покрывает ли срок конец аренды и семидневный maximum review window.
6. Если нет — отменяет hold и требует другую карту.
7. Если да — obligation получает `authorized`.
8. `mark-delivered` требует `authorized` и достаточный срок.

### 11.4 Release

1. Admin подтверждает Release modal.
2. Backend повторно проверяет completed booking и uncaptured hold.
3. Stripe PaymentIntent отменяется.
4. Webhook/API response обновляет payment на `released`.
5. Customer/admin emails и audit event фиксируют результат.

### 11.5 Damage capture

1. Админ загружает минимум одно private evidence photo.
2. Вводит amount `> 0` и `<= amount_capturable` и reason.
3. Confirmation modal показывает captured/released breakdown.
4. Backend создаёт damage claim и делает partial/full capture.
5. Остаток hold освобождается Stripe.
6. Claim/payment/audit/notification обновляются транзакционно настолько, насколько позволяет внешний provider; provider failures остаются retryable.

### 11.6 Cancellation refund

1. Бронь транзакционно становится `cancelled`, даты освобождаются, audit event создаётся.
2. Backend создаёт Stripe refund на указанную сумму.
3. Refund object сохраняется отдельно.
4. Webhooks обновляют refund status.
5. Failure создаёт critical attention item, но booking остаётся cancelled.

## 12. Background worker

Запускается внутри backend process, потому что production сейчас использует один Rust/Axum container.

Worker responsibilities:

- expire 30-minute customer reservations;
- expire 2-hour manual reservations;
- create balance invoices at due time;
- create/send damage hold links за 48 часов;
- retry safe failed Stripe/SES operations;
- обнаруживать approaching `capture_before`;
- создавать overdue hold decision reminders;
- отправлять daily admin digest;
- после рестарта обрабатывать пропущенные due tasks.

Concurrency safety:

- `FOR UPDATE SKIP LOCKED` или эквивалентный transactional claim;
- unique constraints;
- Stripe idempotency keys;
- bounded exponential retry;
- permanent failures не зацикливаются;
- worker не изменяет paid/authorized state без Stripe proof.

## 13. Frontend implementation

### 13.1 Booking overlay

- Сохранить unified overlay architecture из `BOOKING_FLOW_HANDOFF.md`.
- Добавить payment step без standalone route.
- Загружать public Stripe config с backend.
- Монтировать Stripe embedded Checkout.
- Не хранить card data.
- После onComplete показывать `Confirming payment…` и polling webhook-backed status.
- Поддержать close/reopen overlay без повторного создания obligation/session.
- При expired session предложить безопасно пересоздать booking/payment attempt после проверки availability.

### 13.2 `/admin`

Одна route и один page component:

- top-level active tab state;
- query/search/filter state;
- selected booking drawer state;
- nested modal state;
- сохранение текущего списка при открытии drawer;
- responsive desktop/mobile shell;
- `Escape` закрывает только topmost overlay;
- destructive actions имеют disabled/busy/error/success states;
- после action выполняется targeted refresh, а не полная потеря context.

### 13.3 Admin tabs

- Overview: metrics, attention queue, today schedule.
- Bookings: fleet filter, search, statuses, table/cards, detail drawer.
- Payments: obligations, due/failed/authorized/refund filters, detail/actions.
- Calendar: bookings и owner blocks в одной timeline; inline close/reopen panel.
- Audit: immutable events, actor/type/date filters и CSV export.

## 14. Project rules и документация

После утверждения дизайна первым code/documentation change:

1. Синхронно обновить frontend и backend `AGENTS.md`.
2. Заменить `Gold option` на `Stripe authorization hold`.
3. Добавить test-first/live-after-approval policy.
4. Зафиксировать webhook source of truth.
5. Зафиксировать initial/balance/hold flows.
6. Зафиксировать Delivered/Returned gates и 7-day deadline.
7. Зафиксировать reason + photo для damage capture.
8. Зафиксировать two-hour manual booking.
9. Зафиксировать confirmation/audit requirements.
10. Зафиксировать запрет live Stripe/production/domain без прямого разрешения.
11. Обновить `BOOKING_FLOW_HANDOFF.md` после реализации.
12. Обновить customer-facing Terms, чтобы удалить `Gold` и точно описать authorization hold.

## 15. Порядок реализации

### Phase 0 — Design approval

- [x] Проанализировать текущую админку и backend.
- [x] Зафиксировать бизнес-решения.
- [x] Дорисовать desktop/mobile tab states.
- [x] Дорисовать drawers/modals.
- [x] Проверить Pencil layout.
- [x] Получить подтверждение владельца на дизайн и направление единой страницы.

### Phase 1 — Rules and contracts

- [x] Обновить оба `AGENTS.md`.
- [x] Обновить Terms terminology.
- [x] Описать API request/response Rust types.
- [x] Зафиксировать lifecycle transition matrix в backend tests.

### Phase 2 — Database

- [x] Создать additive migration.
- [x] Расширить bookings/payments.
- [x] Создать obligations/events/audit/notifications/damage tables.
- [x] Настроить RLS/revokes/private storage definition.
- [x] Прогнать migration на локальной/test DB.
- [x] Проверить re-run/idempotency migration; destructive rollback не выполнялся.

### Phase 3 — Stripe test configuration

- [x] Выбрать test mode подключённого аккаунта через Stripe CLI authorization.
- [x] Получить test API/publishable credentials без публикации в chat/repository.
- [x] Добавить local ignored `.env` с правами `0600`.
- [x] Создать Stripe CLI test webhook forwarding для локальной разработки.
- [x] Получить и локально настроить test `whsec_...`.
- [x] Реализовать startup environment/account validation.
- [x] Не создавать и не подключать live keys.

### Phase 4 — Backend payment foundation

- [x] Stripe HTTP client/service.
- [x] Idempotency keys и metadata helper.
- [x] Public payment config.
- [x] Webhook signature/idempotency/order handling.
- [x] Initial embedded Checkout flow.
- [x] Payment status polling endpoint.
- [x] Initial-session compensation при Stripe create/DB record failure.

### Phase 5 — Scheduled payments and hold

- [x] Payment obligation service.
- [x] Balance invoice creation/Hosted Invoice Page.
- [x] Damage hold authorization flow.
- [x] Extended authorization/capture_before validation.
- [x] Alternate card retry state/link flow.
- [x] Завершить release/partial/full capture через pending-operation + webhook reconciliation.
- [x] Поддержать durable multi-PaymentIntent refund для оплаты 30% + 70%.
- [x] Background worker, notification queue и retries.

### Phase 6 — Admin backend

- [x] Dashboard summary.
- [x] Booking detail/customer/notes.
- [x] Manual booking + 2-hour expiry.
- [x] Guarded Delivered/Returned/cancel actions.
- [x] Payment actions.
- [x] Evidence upload.
- [x] Дополнить audit для manual booking/calendar/resend/reconcile/evidence/financial actions.
- [x] Хранить admin-edited контакты как booking-scoped snapshot, не изменяя общую customer record.
- [x] Добавить private local/test и Supabase evidence adapters с короткоживущим backend-authorized доступом.

### Phase 7 — Frontend

- [x] Embedded Checkout in unified booking overlay.
- [x] Webhook-backed confirming/status UX.
- [x] Implement one-page `/admin` tabs.
- [x] Booking/payment/calendar/audit panels.
- [x] Desktop drawers and modals.
- [x] Mobile fullscreen drawers и compact tab behavior.
- [x] Escape/topmost overlay behavior.
- [x] Loading/empty/error/success/permission states.

### Phase 8 — Verification

- [x] Backend unit/integration tests, SQL safety tests и disposable-DB concurrency tests без внешнего Stripe E2E.
- [x] Все доступные Stripe test-mode scenarios пройдены: API/webhook/invoice/manual-capture/refund, interactive embedded Checkout success/decline/3DS, Session expiry и Hosted Invoice failure/alternate-card/retry.
- [ ] Damage-risk E2E остаётся закрытым после отказа Stripe в Extended Authorization и требует отдельного утверждённого продуктового решения; это не локальная недоделка кода.
- [x] Frontend compile/unit/WASM и desktop/mobile browser visual/interaction checks; mobile tab clipping и Checkout recovery regressions исправлены.
- [x] Security/RLS/secrets audit.
- [x] UTF-8/mojibake check.
- [x] Update `BOOKING_FLOW_HANDOFF.md` с фактическим результатом.
- [x] Добавить текущий test report и внешние blockers в раздел 0 этого документа.

### Phase 9 — Live readiness, но не включение

- [x] Подготовить список production env variables без секретных значений.
- [x] Подготовить live webhook checklist (раздел 16.9); это не разрешение на включение.
- [x] Проверить extended authorization eligibility: Stripe подтвердил, что account сейчас не eligible; live остаётся заблокирован.
- [ ] Получить отдельное прямое разрешение владельца.
- [ ] Только после этого планировать live activation/deployment отдельной задачей.

## 16. Test matrix

### 16.1 Pricing и booking

- >30 дней: initial obligation ровно 30% trip price.
- <=30 дней: initial obligation ровно 100% trip price.
- Damage hold отсутствует в trip-price percentage.
- Mandatory preparation/protection остаются отдельными quote lines.
- Expired quote не создаёт Stripe Session.
- Concurrent booking conflict отклоняет второй booking.
- 30-minute и 2-hour expiry освобождают availability.

### 16.2 Webhook

- Valid signature.
- Invalid signature.
- Duplicate event.
- Out-of-order event.
- Unknown object/booking metadata.

- Test event при live config и наоборот.
- Provider event после локальной cancellation.
- DB failure с безопасным Stripe retry response.

### 16.3 Initial Checkout

- Success.
- Declined card.
- 3DS/authentication.
- Customer closes overlay.
- Session expires.
- Webhook delayed after Checkout completion.
- Browser refresh/reopen without duplicate charge.

### 16.4 Balance

- Worker creates one invoice.
- Duplicate worker pass creates no duplicate.
- Hosted link email success/failure.
- Invoice paid.
- Invoice payment failed.
- Resend existing link.
- Delivered blocked while balance unpaid.

### 16.5 Damage hold

- Extended authorization enabled and valid.
- Extended authorization disabled.
- `capture_before` too early.
- Customer retries with another card.
- Hold missing at delivery.
- Delivered blocked.
- Full release.
- Partial capture.
- Full capture.
- Missing reason rejected.
- Missing photo rejected.
- Amount > capturable rejected.
- Duplicate release/capture rejected idempotently.
- Seven-day overdue reminder.

### 16.6 Cancellation/refund

- Refund zero/partial/full within paid amount.
- Amount > refundable rejected.
- Required reason.
- Dates released immediately.
- Stripe refund succeeds.
- Stripe refund fails while booking stays cancelled.
- Duplicate confirmation does not duplicate refund.

### 16.7 Admin security

- `default` role gets 403 from every admin endpoint.
- Logged-out user gets 401.
- Hidden admin nav does not substitute backend checks.
- Evidence signed URL requires admin.
- CSV export requires admin.
- Audit events cannot be mutated via API.
- Customer-facing endpoints never expose admin notes, evidence or hosted URLs belonging to other bookings.

### 16.8 Frontend behavior

- Tab switching does not navigate away from `/admin`.
- Drawer preserves filters/list state.
- Modal closes before drawer on Escape.
- Mobile uses fullscreen drawer.
- Destructive button disables during request.
- Stripe test mode is visible.
- Permission/loading/error/empty/success states render correctly.
- No secret key is present in JS/WASM bundle or HTML.

### 16.9 Live webhook checklist (только подготовка)

- Получить отдельное прямое разрешение владельца на live-этап; до него не создавать live endpoint и не менять production env.
- До live завершить полный test-mode отчёт для выбранной модели refundable CA$1,000 charge/refund; Extended Authorization после отказа Stripe больше не является частью продукта.
- Создать отдельный live webhook endpoint именно на ожидаемом Stripe account; test и live signing secrets не переиспользовать.
- Хранить `sk_live_…` и `whsec_…` только в production server environment; никогда не передавать их frontend, БД, логам или Git.
- Подписать endpoint только на реально обрабатываемые события: `checkout.session.completed`, `checkout.session.async_payment_succeeded`, `checkout.session.expired`, `payment_intent.amount_capturable_updated`, `payment_intent.succeeded`, `payment_intent.payment_failed`, `payment_intent.canceled`, `invoice.paid`, `invoice.payment_failed`, `refund.created`, `refund.updated`, `refund.failed`. Перед включением ещё раз сверить список с backend dispatcher.
- Сохранить raw-body signature verification, account/environment/metadata/amount/currency checks, idempotency и защиту от duplicate/out-of-order delivery.
- Выполнить безопасный replay test подписанных live-webhook fixtures без реального списания и проверить retries/alerts/worker reconciliation.
- Проверить startup fail-closed при несовпадении live/test key, ожидаемого account ID, activation gate или storage mode.
- После отдельного разрешения выполнить контролируемый минимальный live smoke payment и refund, проверить audit/email/monitoring и только затем разрешать обычные live bookings.

## 17. Acceptance criteria

Реализация test phase считается завершённой, когда:

- одна `/admin` реализует все утверждённые tabs и overlays;
- default users не получают admin data;
- initial Stripe test payment подтверждает booking только webhook;
- 30%/100% schedule корректен;
- balance invoice создаётся автоматически и идемпотентно;
- отдельный CA$1,000 refundable damage deposit списывается за 48 часов до доставки;
- Delivered блокируется без full trip payment и подтверждённого CA$1,000 deposit payment;
- Returned открывает release/capture actions;
- partial damage capture требует reason + photo;
- cancellation/refund работает с немедленным release availability;
- manual phone booking резервирует RV на два часа;
- audit log фиксирует все финансовые/admin actions;
- все critical emails и retry states видимы;
- backend/frontend tests зелёные;
- secrets/RLS/security audit пройден;
- ни один live key или production payment не использован.

## 18. Rollout gates

### Gate A — Design

Владелец подтверждает Pencil состояния и one-page interaction model.

### Gate B — Local/test backend

Migration, APIs, webhook и worker проходят automated tests.

### Gate C — Stripe test E2E

Проверены success/failure/3DS/invoice/hold/release/capture/refund scenarios.

### Gate D — Test frontend

Проверен полный booking/admin flow desktop и mobile.

### Gate E — Live readiness

Есть документированный зелёный отчёт по refundable deposit charge/refund, production SES готов и подтверждена server-only production configuration. Даже после этого live не включается автоматически.

### Gate F — Separate owner approval

Только отдельное прямое разрешение позволяет подключить live keys, live webhook или production deployment.

## 19. Риски и защита

- Refundable deposit создаёт обычную Stripe processing fee, которую Stripe обычно не возвращает бизнесу при refund. Защита: показывать deposit отдельно, возвращать клиенту утверждённую сумму полностью и учитывать fee как расход бизнеса.
- Duplicate/out-of-order webhooks. Защита: event table, unique provider IDs, provider timestamps и транзакции.
- Backend restart в due time. Защита: catch-up worker по due obligations.
- Двойной worker. Защита: row locks, unique constraints и Stripe idempotency.
- Stripe succeeded, DB update failed. Защита: webhook retry/reconciliation endpoint, provider state не заменяется ручным `paid`.
- Refund failure после cancellation. Защита: даты освобождены по бизнес-решению, refund остаётся critical action.
- Evidence privacy. Защита: private bucket, signed URL и admin authorization.
- Test/live mix. Защита: prefix/account/mode validation при startup и в webhook.
- Existing dirty frontend worktree. Защита: не перезаписывать пользовательские изменения, перед каждым patch проверять diff пересекающихся файлов.

## 20. Явно вне первой реализации

- Boats и boat bookings.
- Live Stripe activation.
- Production deploy/restart.
- Изменение `vlrental.ca`, DNS или GitHub Pages custom domain.
- Offline cash/bank payment marking.
- Автоматическая cancellation policy calculation.
- Новые admin/owner роли или double approval.
- Прямое изменение dates/RV/price оплаченной брони.
- Хранение card data.
- Автоматический damage refund/capture без admin action.

## 21. Следующий разрешённый шаг

Текущий следующий этап:

1. Выбранная владельцем модель — отдельный refundable CA$1,000 charge за 48 часов до доставки с admin refund/retention после возврата. Extended Authorization не используется.
2. Завершить внешнюю готовность SES: опубликовать три Easy DKIM CNAME в DNS, дождаться статуса `Verified`, затем запросить выход SES `ca-central-1` из sandbox. DNS и AWS submission требуют отдельного подтверждения владельца непосредственно перед действием.
3. После полностью зелёного обновлённого test report запросить отдельное разрешение владельца на live Stripe и production deployment. Подготовленный checklist сам по себе не разрешает activation.

Live Stripe, deployment и production остаются заблокированными до отдельного запроса.
