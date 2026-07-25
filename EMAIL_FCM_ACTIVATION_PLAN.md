# Активация Amazon SES email и FCM-уведомлений

Дата последней проверки: 2026-07-25

Frontend: `/Users/viktoriiakarpova/Projects/it_work/viktor_rv_front`

Backend: `/Users/viktoriiakarpova/Projects/it_work/viktor_rv_back`

Рабочая ветка обоих репозиториев: `dev`

## Цель

Обеспечить надёжную отправку:

1. email и FCM сразу после подтверждения первоначальной оплаты проверенным Stripe webhook;
2. email и FCM со ссылкой на остаточный платёж ровно за 30 дней до доставки;
3. email и FCM со ссылкой на refundable damage deposit CA$1,000 ровно за 48 часов до доставки;
4. административных уведомлений об ошибках, просроченных и критических действиях;
5. ограниченных идемпотентных повторов без дублей и без отката подтверждённой оплаты или брони.

«Сразу при booking» означает сразу после webhook-подтверждения успешной первоначальной оплаты, а не при создании временной неоплаченной брони.

## Неподвижные ограничения

- Проект только для RV.
- Stripe остаётся в `test`; live-ключи, live webhook и live routing не включать.
- Домен и DNS `vlrental.ca` не менять.
- Подтверждать платёж и бронь может только проверенный Stripe webhook.
- Ошибка SES или SNS не отменяет подтверждённый платёж и не откатывает бронь.
- Старые 26 клиентских email не переотправлять.
- Старые ошибки сохранить как историю.
- Существующие исчерпанные SES authentication/sender errors один раз пометить `retry_suppressed`, чтобы Admin retry не переслал их.
- Секреты AWS, Firebase server credentials, Stripe и Supabase остаются только на backend/GitHub Secrets.
- Публичный API отдаёт только публичную Firebase web-конфигурацию.
- Разработка, commit и push выполняются только в `dev`.
- Production backend разворачивается раньше frontend.
- Перед deployment обязательны форматирование, тесты, warning-free lint, WASM и disposable PostgreSQL checks.

## Подтверждённое состояние инфраструктуры

### Amazon SES

- Production access выдан.
- Лимит: 50,000 писем в сутки и 14 писем в секунду.
- Домен `vlrental.ca`: `Verified`.
- Easy DKIM: `Successful`.
- Отправитель backend: `no-reply@vlrental.ca`.
- Административный получатель: `Vlrental.ca@gmail.com`.

### Email в production database

- 4 новые доставки уже имеют статус `sent`:
  - 2 ссылки на остаточный платёж;
  - 2 ссылки на damage deposit.
- 26 старых доставок имеют исчерпанные попытки:
  - 20 `smtp_authentication_failed`;
  - 6 `smtp_sender_rejected`.
- Эти 26 записей не сбрасывать в очередь и не отправлять клиентам повторно.

### Firebase / Amazon SNS

- SNS platform application: `vl-rental-web`, тип FCM/GCM.
- До deployment зарегистрировано `0` SNS endpoints.
- Production API `/api/v1/push/config` до deployment отвечает `404`.
- Production frontend-ресурсы `push.js` и `firebase-messaging-sw.js` до deployment отвечают `404`.
- FCM schema migration `20260721200812_add_fcm_push_notifications.sql` уже применена в правильном Supabase project `pwhlkpwlansarstmstge`.
- В production существуют таблицы `push_devices` и `push_notification_deliveries`.

## Архитектурное решение

### Публичные интерфейсы

- `GET /api/v1/push/config`
  - без авторизации;
  - возвращает только публичную Firebase web-конфигурацию;
  - возвращает `enabled=false`, если push выключен.
- `POST /api/v1/me/push-devices`
  - требует пользовательскую сессию;
  - валидирует FCM token;
  - создаёт/обновляет SNS platform endpoint;
  - сохраняет только SHA-256 hash токена и SNS endpoint ARN.
- `DELETE /api/v1/me/push-devices`
  - требует пользовательскую сессию;
  - отключает устройство;
  - удаляет endpoint в SNS по возможности;
  - не удаляет историю доставок.

### Email и push fan-out

- Email остаётся первичной durable notification queue.
- Push worker идемпотентно создаёт отдельные доставки из `notification_deliveries`.
- Для одного email и одного устройства используется уникальный idempotency key.
- Поддерживаются:
  - `customer_*`;
  - `admin_*`;
  - `balance_payment_link`;
  - `damage_hold_payment_link`.
- Push, зарегистрированный после создания старого email, не получает старую доставку. Это защищает от пересылки 26 старых сообщений.

### Worker

- Работает периодически.
- Берёт только enabled devices.
- Использует `FOR UPDATE SKIP LOCKED`.
- Claim имеет timeout для восстановления после падения worker.
- Максимум 8 попыток.
- Backoff начинается с 5 минут и ограничен 6 часами.
- Успешная SNS отправка сохраняет `message_id` в `provider_reference`.
- В database сохраняются только санитизированные коды и сообщения ошибок.
- Raw provider error используется только в памяти для определения stale/disabled endpoint.
- Stale endpoint отключается, а его незавершённые доставки переводятся в `cancelled`.
- Отключение пользователем также отменяет незавершённые доставки, но сохраняет аудит.

### Frontend

- Notification toggle находится в существующей account panel; отдельная страница не создаётся.
- Service worker: `firebase-messaging-sw.js`.
- Firebase browser client: `push.js`.
- При включении:
  - запрашивается browser permission;
  - получается текущий FCM token;
  - token регистрируется через авторизованный backend endpoint;
  - предпочтение сохраняется локально.
- При следующем входе/открытии header:
  - текущий FCM token получается повторно без нового permission prompt;
  - backend registration обновляется;
  - token rotation не должна молча ломать push.
- При выключении или logout:
  - backend device отключается;
  - локальный Firebase token удаляется;
  - локальное предпочтение очищается.

## Критические дефекты, найденные повторной проверкой

Эти дефекты существовали в первоначальной локальной FCM-реализации и должны быть исправлены до deployment:

1. `balance_payment_link` и `damage_hold_payment_link` не соответствовали фильтру `customer_%`, поэтому push для обязательных сроковых платежей не создавался.
2. Disabled devices могли быть claimed, после чего доставка зависала в `sending`.
3. Удаление `push_devices` каскадно удаляло историю push deliveries.
4. Frontend показывал `enabled` только по permission/local preference и не восстанавливал регистрацию после FCM token rotation.
5. SNS `message_id` не сохранялся.
6. SNS failures не отображались отдельно в Admin.
7. Raw SNS error сохранялся в database.
8. `PushConfig` мог быть отформатирован через `Debug`, хотя содержит AWS credentials.
9. Проверка SNS ARN была слишком широкой.

## Уже выполненные локальные исправления

- Добавлен fan-out для `balance_payment_link` и `damage_hold_payment_link`.
- Claim query теперь включает только enabled devices.
- Unregister заменён на soft-disable с сохранением истории.
- Pending/sending/failed deliveries отключённого устройства отменяются атомарно.
- Frontend `status()` повторно получает текущий token и синхронизирует его с backend.
- SNS `message_id` возвращается integration layer и сохраняется как `provider_reference`.
- В Admin добавлен отдельный `push_notification_failures`.
- Push failures добавлены в `Needs attention`.
- Пользовательские unregister/stale cancellations исключены из actionable Admin failures.
- В database сохраняются санитизированные SNS errors.
- `PushConfig` больше не реализует `Debug`; секретные поля ограничены видимостью crate.
- SNS platform application ARN валидируется по точной структуре, региону, типу и имени.
- Добавлен disposable PostgreSQL regression test, проверяющий:
  - customer/admin fan-out;
  - обе обязательные payment-link доставки;
  - отсутствие доставки disabled device;
  - идемпотентность повторного fan-out;
  - сохранение истории после отключения устройства.
- Добавлена недеструктивная миграция `20260723120000_suppress_legacy_email_retries.sql`:
  - сохраняет старые email rows;
  - помечает существующие отменённые SES authentication/sender failures как `retry_suppressed`;
  - исключает их из Admin failure count, attention и ручного retry;
  - не подавляет новые будущие ошибки.
- Regression test добавлен в production GitHub Actions workflow.

## Текущее состояние локальных проверок

После указанных исправлений:

- Backend formatting: проходит.
- Backend unit/all-target tests: 144 passed, 22 ignored database/provider tests.
- Backend clippy `-D warnings`: проходит.
- Frontend tests: 107 passed.
- Frontend clippy `-D warnings`: проходит.
- Frontend WASM check: проходит.

Ещё требуется выполнить новый ignored regression test в disposable PostgreSQL и повторить полный финальный набор проверок после всех правок.

## Следующие локальные шаги

1. Поднять disposable PostgreSQL 17.
2. Создать роли `anon` и `authenticated`.
3. Применить `sql/schema.sql`.
4. Выполнить все SQL safety contracts.
5. Запустить:
   - push regression test;
   - payment database tests;
   - auth database tests;
   - booking schedule concurrency tests.
6. Проверить frontend service worker scope и GitHub Pages base path.
7. Проверить, что публичная Firebase-конфигурация в API и service worker совпадает.
8. Проверить IAM policy:
   - только нужные SNS действия;
   - только нужный platform application/endpoints;
   - credentials не попадают в frontend, логи или database.
9. Проверить UTF-8/mojibake во всех изменённых текстовых файлах.
10. Повторно выполнить:
    - `cargo fmt --all --check`;
    - `cargo test --all-targets`;
    - `cargo clippy --all-targets -- -D warnings`;
    - frontend `cargo check --target wasm32-unknown-unknown`.

## Git-стратегия при грязных worktree

В обоих репозиториях есть сторонние незавершённые изменения пользователя, не относящиеся к FCM/email. Нельзя включать их в текущие commits.

Перед commit:

1. Просмотреть `git diff` каждого файла.
2. Выделить только email/FCM hunks.
3. Отдельно staged проверить `git diff --cached`.
4. Убедиться, что Facebook OAuth, calendar, catalog и прочие сторонние изменения не staged.
5. Создать отдельный backend commit.
6. Создать отдельный frontend commit.
7. Push только в `origin/dev`.

Не использовать destructive reset/checkout.

## Deployment

Deployment разрешён данным утверждённым планом, но выполняется только после полностью зелёных проверок.

### 1. Backend

1. Push backend `dev`.
2. Продвинуть тот же проверенный commit в `main` штатным проектным процессом.
3. Дождаться завершения GitHub Actions.
4. Проверить:
   - `GET /health` → `200`;
   - `GET /api/v1/push/config` → `200`;
   - `enabled=true`;
   - ответ не содержит private AWS/Firebase/Stripe secrets;
   - payment config всё ещё сообщает Stripe `test`;
   - live gate выключен.

### 2. Frontend

Только после здорового backend:

1. Push frontend `dev`.
2. Продвинуть проверенный commit в `main`.
3. Дождаться GitHub Pages workflow.
4. Проверить:
   - `push.js` → `200`;
   - `firebase-messaging-sw.js` → `200`;
   - service worker регистрируется с правильным scope;
   - старый frontend не остаётся в cache;
   - домен/DNS не изменялись.

## Production smoke tests

### Admin test email

1. Запустить существующее действие `Test email` в Admin.
2. Подтвердить status `sent` в database.
3. Подтвердить фактическое получение в `Vlrental.ca@gmail.com`.
4. Проверить From, DKIM и отсутствие попадания секретов/внутренних ошибок в письмо.

### Регистрация push device

1. Войти в поддерживаемом браузере.
2. Включить notifications в account panel.
3. Разрешить browser notification permission.
4. Проверить:
   - backend registration response;
   - enabled row в `push_devices`;
   - минимум один endpoint в SNS;
   - повторное открытие не создаёт дубликат database row;
   - выключение сохраняет историю и отключает endpoint/device.

### Stripe test booking более чем за 30 дней

1. Убедиться, что frontend/backend показывают Stripe `test`.
2. Создать новую отдельную тестовую бронь с доставкой более чем через 30 дней.
3. Проверить immutable quote:
   - initial payment = 30% только trip price;
   - refundable CA$1,000 deposit не входит в эти 30%;
   - balance = оставшиеся 70%;
   - balance due ровно за 30 дней;
   - deposit due ровно за 48 часов.
4. Оплатить initial 30% тестовой картой.
5. Дождаться verified webhook.
6. Подтвердить:
   - booking стала confirmed только после webhook;
   - ровно один customer email;
   - ровно один admin email;
   - ровно один customer push на каждое зарегистрированное customer device;
   - ровно один admin push на каждое зарегистрированное admin device;
   - повторный webhook не создаёт дублей.

### Balance и damage deposit deadlines

В изолированной тестовой database:

1. Принудительно приблизить balance due.
2. Запустить worker.
3. Проверить одну Stripe test payment link, email и FCM.
4. Повторить worker и убедиться в отсутствии дублей.
5. Подтвердить, что повторная/старая Stripe event не может повторно оплатить обязательство.
6. Повторить для damage deposit due за 48 часов.
7. Проверить точную сумму CA$1,000 и отдельность от trip price.

### SES/SNS failures

Только в изолированной тестовой среде или безопасным контролируемым способом:

1. Смоделировать SES failure.
2. Смоделировать SNS failure.
3. Подтвердить:
   - платёж и booking остаются сохранёнными;
   - email failure виден в Admin и доступен для ручного retry;
   - push failure виден отдельно в Admin;
   - автоматический retry использует ограниченный backoff;
   - после 8 попыток delivery становится `cancelled`;
   - secrets и raw provider payload не сохраняются.

## Production database safety

- Правильный Supabase project ref: `pwhlkpwlansarstmstge`.
- Frontend не обращается к Supabase Data API напрямую.
- `push_devices` и `push_notification_deliveries`:
  - RLS enabled;
  - privileges отозваны у `PUBLIC`, `anon`, `authenticated`;
  - backend работает через private PostgreSQL connection.
- Production database writes допустимы только для утверждённых non-destructive migrations и smoke-test данных в рамках этого плана.
- Не выполнять destructive DDL, broad rewrites или удаление реальных customer data.

## Финальные критерии приёмки

Работа считается завершённой только если одновременно выполнено всё:

- Backend API push endpoints возвращают ожидаемые `200`.
- Frontend push resources возвращают `200`.
- Stripe остаётся в `test`.
- Домен и live-payment routing не изменены.
- Минимум одно устройство зарегистрировано в `push_devices` и SNS.
- Новая тестовая бронь подтверждается только verified webhook.
- Initial booking email/FCM приходят без дублей.
- Balance email/FCM создаются ровно за 30 дней и содержат правильную test payment link.
- Damage deposit email/FCM создаются ровно за 48 часов на CA$1,000.
- Новые успешные доставки имеют `sent`.
- SNS successful delivery хранит provider `message_id`.
- Ошибки видны в Admin и повторяются с ограниченным backoff.
- 26 старых email не были переотправлены.
- Все локальные и CI checks зелёные.
- В commits нет секретов и посторонних пользовательских изменений.

## Статус

### Выполнено 2026-07-23

- Production migration `20260723120000_suppress_legacy_email_retries.sql`
  применена и проверена:
  - 26 из 26 старых SES authentication/sender failures имеют
    `retry_suppressed=true`;
  - старые строки сохранены;
  - активных старых ошибок для retry: 0.
- Backend production SHA:
  `344ba9df59338a0a663e8ade7605d3741a9b34ea`.
- Frontend production SHA:
  `0aa5c8ea40ad90d42ed67a546c488ed772923f28`.
- Backend DB workflow:
  `https://github.com/GaponovAlexey/viktor_rv_back/actions/runs/30067269665`
  — `success`.
- Backend test/build/deploy workflow:
  `https://github.com/GaponovAlexey/viktor_rv_back/actions/runs/30067270271`
  — `success`.
- Frontend test/build/Pages workflow:
  `https://github.com/GaponovAlexey/viktor_rv_front/actions/runs/30067736434`
  — `success`.
- Backend CI подтвердил:
  - formatting;
  - disposable PostgreSQL schema;
  - SQL safety contracts;
  - 145 backend tests;
  - payment/auth/push/concurrency database tests;
  - warning-free clippy.
- Frontend CI подтвердил:
  - formatting;
  - 107 tests;
  - Pages artifact verification;
  - warning-free clippy;
  - WASM/browser target.
- Production smoke:
  - `/health` → `200`;
  - `/api/v1/push/config` → `200`, `enabled=true`;
  - unauthenticated `POST /api/v1/me/push-devices` → `401`;
  - `/api/v1/payments/config` → `mode=test`;
  - `push.js`, `firebase-messaging-sw.js`, `site.webmanifest` → `200`;
  - browser marker `data-vl-push-client=ready`;
  - домен, DNS и live Stripe не менялись.
- Admin `Test email` успешно отправлен и фактически получен в
  `Vlrental.ca@gmail.com`.
- Production database подтверждает:
  - 2 `balance_payment_link` → `sent`;
  - 2 `damage_hold_payment_link` → `sent`;
  - старые 26 ошибок не переотправлены.
- Проведён отдельный Stripe test E2E:
  - booking `VL-20260723-00000011`;
  - trip price CA$1,055.74;
  - initial 30% CA$316.72;
  - balance CA$739.02, due 2027-03-06 13:00 PST;
  - refundable damage deposit CA$1,000, due 2027-04-03 14:00 PDT;
  - verified webhook перевёл booking в `confirmed/partially_paid`;
  - customer/admin confirmation email созданы ровно по одному и имеют
    `sent`;
  - оба письма фактически получены в Gmail;
  - Stripe `checkout_session` и `payment_intent` — разные provider objects,
    обязательство зачтено один раз;
  - после проверки booking отменена, полный test refund CA$316.72 имеет
    `succeeded`, future obligations отменены, даты освобождены;
  - refund/cancellation customer/admin emails имеют `sent`.

### FCM production smoke 2026-07-25

- Chrome notification permission для
  `https://gaponovalexey.github.io` выдан.
- Первый backend registration вызвал:
  - успешный `CreatePlatformEndpoint`;
  - `AccessDenied` на `SNS:SetEndpointAttributes`;
  - отсутствие database row, хотя orphan endpoint появился в SNS.
- CloudTrail подтвердил точную причину: IAM user `vl-rental-push` не мог
  выполнить `SNS:SetEndpointAttributes` для platform application ARN.
- Inline policy `VlRentalPushSns` исправлена по принципу least privilege:
  - широкие `sns:*` и `Resource: "*"` не добавлялись;
  - существующий список действий сохранён;
  - к endpoint ARN добавлен только ARN приложения
    `arn:aws:sns:ca-central-1:812607972157:app/GCM/vl-rental-web`.
- После исправления повторная регистрация через production frontend/backend
  успешна:
  - account panel показывает `Notifications on`;
  - `push_devices`: 1 row, 1 enabled, 1 user;
  - SNS `vl-rental-web`: 1 enabled endpoint;
  - повторно используется тот же endpoint, дубликат не создан.
- Два контрольных FCM v1 payload приняты SNS с `Message ID`, последний:
  `afc14fe7-db7d-5196-a76a-2d38d1347b04`.
- Endpoint после отправок остаётся `Enabled`.

### Единственный незавершённый FCM-критерий

- Визуальное получение push в Chrome/macOS пока не подтверждено.
- У SNS platform application не настроены:
  - IAM role для successful deliveries;
  - IAM role для failed deliveries;
  - successful delivery sample rate.
- Поэтому успешный ответ `Publish` доказывает принятие сообщения SNS, но не
  позволяет отличить доставку FCM от асинхронного отказа FCM.
- Для окончательной диагностики необходимо:
  1. включить CloudWatch delivery status logging для `vl-rental-web`;
  2. отправить ещё один контрольный FCM v1 payload;
  3. проверить FCM provider response в CloudWatch;
  4. при успешной доставке проверить локальные настройки Chrome/macOS;
  5. при provider failure исправить только конкретную FCM credential/config
     причину и повторить тест.

Текущий этап: email, Stripe webhook и регистрация FCM device полностью
активированы и проверены. Осталось подтвердить последний участок
`SNS → FCM → Chrome notification`.
