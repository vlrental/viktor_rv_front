# Viktor RV — план E2E-проверки задеплоенной админки и Stripe test mode

Дата подготовки: 2026-07-15 (`America/Vancouver`)

Назначение: этот файл является самостоятельным контекстом для новой задачи Codex. В новом контексте сначала прочитать frontend/backend `AGENTS.md`, затем этот план и `BOOKING_FLOW_HANDOFF.md`. Старую переписку для понимания теста читать не требуется.

Frontend repository: `/Users/viktoriiakarpova/Projects/it_work/viktor_rv_front`

Backend repository: `/Users/viktoriiakarpova/Projects/it_work/viktor_rv_back`

Test frontend: `https://gaponovalexey.github.io/viktor_rv_front/`

Backend: `https://api.vlrental.ca`

Stripe account: `102181797 Saskatchewan Ltd.`, `acct_1SpY7K2MR4C4rvKM`

Supabase project: `pwhlkpwlansarstmstge`

## 1. Цель

Провести настоящий end-to-end тест задеплоенного приложения, но исключительно в Stripe test mode:

1. Убедиться, что задеплоены именно последние проверенные `dev` commits frontend и backend.
2. Проверить авторизацию и роль `admin` на реальном API.
3. Пройти клиентскую бронь через embedded Stripe Checkout.
4. Доказать, что только проверенный webhook подтверждает оплату и бронь.
5. Проверить админку, ручную бронь, платежи, календарь, audit log и email controls.
6. Проверить refundable CA$1,000 damage deposit, возврат и документированное удержание ущерба.
7. Проверить Supabase записи и приватные evidence files без прямого доступа браузерных ролей к Data API.
8. Удалить или корректно закрыть тестовые операционные объекты, сохранив audit/payment history.
9. Сформировать отдельный финальный test report с `PASS`, `FAIL`, доказательствами и оставшимися внешними блокерами.

Этот этап не включает live Stripe, реальные карты, реальные списания, изменение DNS, подключение `vlrental.ca` или отправку production customer traffic.

## 2. Что уже подтверждено на старте

Read-only проверка 2026-07-15 показала:

- `https://api.vlrental.ca/health` отвечает `200`, `database=true`.
- `/api/v1/payments/config` отвечает `enabled=true`, `mode=test`, account ID совпадает с `acct_1SpY7K2MR4C4rvKM`.
- Существующая браузерная admin session открывает `/admin` и получает production Supabase data.
- В старом задеплоенном admin bundle видны Overview, Bookings, Payments, Calendar и Audit; отображается `Stripe test`.
- В production данных на момент проверки было 7 confirmed/active-upcoming bookings, 0 awaiting payment, 0 payment errors и 0 overdue actions.
- Никаких тестовых booking/payment mutations во время этой сверки не выполнялось.

## 3. Обязательный blocker: задеплоена не последняя версия

Нельзя начинать транзакционные E2E-тесты до выравнивания deployment.

### Frontend mismatch

- Последний проверенный frontend `dev`: `6e99dfbcbfbcab9aeeab319a76e4c9d4a87c4f49`.
- Текущий `origin/main`: `32239ef9a19a27fbb84aa514fdf99e2ede9b5c9e`.
- Задеплоенный bundle соответствует старому интерфейсу: вкладки `RVs` нет.
- Свежий прямой запрос к `/admin` получает GitHub Pages 404 page; существующая вкладка показывает SPA только потому, что приложение уже было открыто/навигация произошла внутри SPA.
- Последний `dev` workflow уже содержит `404.html` fallback и RV administration UI.

### Backend mismatch

- Последний проверенный backend `dev`: `8a9d528a8ab497b29139b86a7de59f77aa858912`.
- Текущий `origin/main`: `49b09d6c9dcd952c03e0670a9cde09b3eb8a3216`.
- Задеплоенный `/api/v1/admin/rentals` возвращает `404`, то есть RV administration backend ещё не задеплоен.
- У `/api/v1/payments/config` пока отсутствуют новые `Cache-Control: no-store` и security headers, что также подтверждает старый backend image.

### Gate 0 — что должно произойти

До тестов необходимо:

1. Убедиться, что рабочие ветки обоих repositories — `dev` и оба worktree не содержат незакоммиченных изменений, которые забыли включить в commits.
2. Получить прямое подтверждение владельца на promotion последних `dev` commits в `main`, если этого ещё не сделал сам владелец.
3. Frontend: выполнить утверждённый `make dm` или эквивалентный promotion одного и того же проверенного commit в `main`.
4. Backend: выполнить утверждённый `make dm`, дождаться backend deploy и database migration workflow.
5. Не запускать параллельно второй deployment.
6. Дождаться зелёных GitHub Actions для Pages, backend и migrations.

### Gate 0 — критерии прохождения

- GitHub `main` обоих repositories указывает на ожидаемые `dev` commits или их явно проверенные successors.
- Новый browser session, открытый непосредственно по `/admin`, рендерит приложение, а не GitHub 404 document.
- В админке присутствует вкладка `RVs`.
- Неавторизованный `GET /api/v1/admin/rentals` возвращает `401`, а не `404`.
- `/api/v1/payments/config` имеет `Cache-Control: no-store`, `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer` и HSTS.
- `/health` возвращает `200`, `database=true` после deployment.

Если любой критерий Gate 0 не пройден, Stripe/booking mutations не начинать.

## 4. Разрешение и границы теста

Сообщение владельца разрешает тестировать задеплоенную админку и Stripe по-настоящему в test mode. Это разрешение включает:

- создание специально помеченных test bookings в production database;
- создание Stripe test-mode Checkout Sessions, Invoices, PaymentIntents, Charges и Refunds;
- прохождение Stripe test cards и 3DS test challenge;
- создание manual/phone test booking;
- отправку тестовых писем на заранее согласованные тестовые адреса;
- загрузку искусственного, не содержащего персональных данных test image как damage evidence;
- отмену и финансовое закрытие созданных тестовых броней через штатный admin UI/API.

Это разрешение не включает:

- `sk_live`, `pk_live`, live webhook или live Stripe objects;
- настоящую банковскую карту;
- использование данных реального клиента;
- изменение существующих legacy/customer bookings;
- удаление audit/payment truth напрямую из БД;
- изменение DNS, `vlrental.ca`, AWS SES production-access request или доменных записей;
- ручные SQL-изменения production business state без отдельного подтверждения.

## 5. Безопасные тестовые данные

Перед первым submit определить один тестовый набор:

- customer name должен явно содержать `E2E TEST`;
- email — согласованный адрес/plus-alias, доступный владельцу и допустимый текущими SES ограничениями;
- phone — зарезервированное вымышленное значение, не номер реального клиента;
- delivery address — публичный адрес/ориентир в пределах 150 km от Kelowna, не домашний адрес пользователя;
- notes — `AUTOMATED E2E TEST — SAFE TO CANCEL`;
- evidence photo — сгенерированное изображение с текстом `E2E TEST DAMAGE`, без людей, документов и геолокации;
- все Stripe objects должны иметь metadata booking ID, booking number, payment type и environment.

Stripe test-card данные вводить только внутрь Stripe-hosted embedded Checkout. Никогда не писать их в repository, plan, logs, Supabase или admin notes. Перед выполнением сверить актуальные test cards по официальной Stripe testing documentation.

## 6. Security cleanup перед тестированием auth

В Chrome была обнаружена старая localhost-вкладка pre-hardening версии, в URL которой остались legacy access/refresh tokens. Значения токенов не копировать и нигде не повторять.

Перед E2E auth test:

1. Закрыть старую localhost-вкладку с token-bearing URL.
2. После отдельного подтверждения владельца отозвать старые активные admin auth sessions, созданные до нового deployment, либо отозвать конкретную скомпрометированную session по hash/server-side record.
3. Выполнить новый вход через задеплоенный one-time-code OAuth flow.
4. Убедиться, что после callback URL немедленно очищен и не содержит access/refresh tokens.
5. Не читать browser cookies/local storage/session storage средствами browser automation; проверять контракт по URL, network/API behavior и server-side hashed records.

## 7. Phase A — read-only deployment preflight

Выполнить до любых форм и платежей:

1. Проверить frontend root и свежую прямую загрузку `/admin`.
2. Проверить desktop и mobile shell, отсутствие redirect на `vlrental.ca`.
3. Проверить `/health` и payment config.
4. Проверить правильный CORS для GitHub Pages origin и отсутствие CORS для постороннего origin.
5. Проверить security headers на auth, payment config и private/admin responses.
6. Проверить console errors на home и `/admin`.
7. Проверить Supabase migration list: все versioned migrations, включая admin rentals, email/timezones и auth/Data API hardening, должны быть применены ровно один раз.
8. Запустить Supabase Security Advisor и Performance Advisor read-only; сохранить remediation links для реальных предупреждений.
9. Проверить, что `anon`/`authenticated` не имеют `USAGE`/table/sequence/function access к backend-only `public` application schema.
10. Не выводить secret keys, connection strings, raw tokens или customer PII в test report.

## 8. Phase B — read-only admin UI smoke

Проверить без изменения данных:

- `/admin` разрешён только роли `admin`.
- Overview загружает summary, attention, today и upcoming trips.
- Bookings: поиск, filters, booking drawer, payment schedule, timeline и notes layout.
- RVs: fleet list, published/archived filters, RV editor drawer, media/features/add-ons layout; ничего не сохранять в read-only phase.
- Payments: initial, balance, deposit, refunds, provider status и resend controls.
- Calendar: 14-day fleet schedule, return 11:00 AM, delivery 2:00 PM, same-day turnaround.
- Audit: entries и CSV download control.
- Каждый drawer/modal закрывается `×` и `Escape`; nested overlay закрывает только верхний слой.
- Mobile width 390 px: Overview, Bookings, RVs и More не обрезаются; drawers fullscreen.
- Default user не получает admin response body и не видит admin content.

Сохранить screenshots только без лишних персональных данных; при необходимости замаскировать customer details.

## 9. Phase C — auth E2E

Выполнить отдельными сценариями:

### C1. Email/password

1. Создать или использовать отдельного `default` test user.
2. Войти через inline account panel, без отдельной auth page.
3. Проверить `/auth/me`, customer-contact prefill и locked authenticated email.
4. Убедиться, что default user получает `403`/no admin data.
5. Logout должен отозвать backend session; повторный protected request со старым access token отклоняется.

### C2. Google OAuth

1. Начать вход из существующего overlay/page.
2. Проверить возврат в тот же workflow.
3. Callback содержит только краткоживущий one-time code, затем URL очищается до exchange.
4. Повторный exchange одного code отклоняется.
5. Одновременный refresh одного refresh token даёт только один success.

### C3. Admin

1. Войти согласованным admin account через новый flow.
2. Проверить роль серверным `/auth/me`.
3. Открыть `/admin` напрямую после fresh reload.
4. Не использовать standalone admin token — его не существует.

## 10. Phase D — customer booking и embedded Checkout

Нужно создать минимум две test bookings, не затрагивающие существующие customer dates.

### D1. Бронь более чем за 30 дней — 30%

1. Выбрать свободный RV и диапазон минимум 3 ночи, delivery более чем через 30 дней.
2. Выбрать разрешённый delivery address и получить backend quote.
3. Проверить отдельные line items: nightly rental, RV Preparation Fee CA$97, Stationary Plus Protection CA$50/night, delivery и add-ons.
4. Проверить, что trip price не включает CA$1,000 refundable deposit.
5. Submit создаёт `pending_payment`, 30-minute reservation и embedded Checkout Session.
6. `amount_due_now` равен округлённым 30% immutable trip price.
7. Сначала пройти decline test; booking остаётся `pending_payment`.
8. Затем оплатить успешной Stripe test card.
9. Browser completion не должен сам подтверждать booking.
10. Только verified webhook переводит booking в `confirmed` и initial obligation в success.
11. Refresh/reopen до оплаты должен использовать ту же pending booking/session, а не создавать duplicate.

### D2. Бронь за 30 дней или меньше — 100%

1. Выбрать другой безопасный свободный диапазон.
2. `amount_due_now` должен совпадать со 100% trip price.
3. Пройти Stripe 3DS test scenario.
4. Подтверждение снова происходит только после webhook.

### D3. Checkout expiry

1. Создать отдельную обычную test booking и не оплачивать.
2. Дождаться 30-minute expiry без ручного изменения БД.
3. Проверить booking `expired`, obligation/payment object consistency и освобождение availability.
4. Убедиться, что поздний/старый provider event не восстанавливает expired booking.

## 11. Phase E — admin booking workflow

Для созданной test booking проверить:

1. Появление в Bookings без reload или после контролируемого refresh.
2. Drawer: customer, trip, immutable quote, schedule, timeline, notes.
3. Редактирование test customer contact и internal notes.
4. Отсутствие UI для изменения дат/RV/цены оплаченной брони.
5. Payment history соответствует Stripe objects и webhook events.
6. Resend link создаёт notification attempt и не создаёт новую obligation.
7. Каждое admin action имеет confirmation dialog и immutable audit event.
8. CSV export открывается корректно и не исполняет spreadsheet formulas.

## 12. Phase F — manual/phone booking

1. Открыть Phone booking drawer/modal внутри `/admin`.
2. Создать booking с `E2E TEST` customer data и теми же backend quote rules.
3. Проверить двухчасовой availability reservation.
4. Проверить dynamic hosted Checkout link и email delivery record.
5. Resend использует существующую payment obligation/session, не создавая duplicate charge.
6. Не использовать cash/bank/manual-paid override.
7. Для expiry test дождаться штатного срока либо отменить booking через штатный flow; не менять timestamps SQL вручную без отдельного разрешения.

## 13. Phase G — balance Invoice

Автоматический balance Invoice возникает ровно за 30 дней до delivery для 30% booking.

Предпочтительный black-box способ:

1. Создать 30% booking с delivery на ближайшей доступной границе более 30 дней.
2. Не менять `due_at` вручную.
3. Дождаться реального worker boundary, если он наступает в приемлемый срок.
4. Проверить одну idempotent Invoice, Hosted Invoice Page link, email delivery и admin resend.
5. Пройти invoice failed test method, затем заменить test method и оплатить ту же Invoice.
6. Только webhook переводит balance obligation в success/full trip paid.

Если реальная граница требует слишком долгого ожидания, этот production E2E пункт пометить `DEFERRED — scheduler contract already covered by disposable DB tests`. Не подделывать production booking timestamps ради красивого отчёта.

## 14. Phase H — refundable CA$1,000 damage deposit

Используется отдельный Stripe charge/refund flow. Extended Authorization и manual-capture hold не используются.

1. Использовать только специально созданную test booking с безопасными датами.
2. Дождаться/создать штатную ситуацию, когда deposit due за 48 часов до delivery.
3. Worker создаёт ровно одну CA$1,000 test Checkout/payment link и notification.
4. Оплатить CA$1,000 Stripe test card; webhook подтверждает deposit obligation.
5. До full trip payment и paid deposit кнопка Delivered заблокирована.
6. После обоих payment gates отметить Delivered, затем Returned.

### H1. Full refund

1. После Returned выбрать полный возврат CA$1,000.
2. Подтвердить действие.
3. Проверить durable operation `pending/submitted/succeeded`.
4. Только verified refund webhook/reconciliation завершает возврат.
5. Проверить customer/admin email и audit event.

### H2. Partial retention

На отдельной test deposit или отдельной booking:

1. Указать небольшую test damage amount, например CA$100.
2. Добавить явную test reason.
3. Загрузить сгенерированную non-personal evidence photo.
4. Без фото submit должен быть заблокирован.
5. После confirmation Stripe test refund возвращает remainder, например CA$900, а CA$100 остаётся documented retention.
6. Проверить private evidence access через короткую admin-authorized URL/token, отсутствие public bucket access и audit event.

Не выполнять full-refund и partial-retention на одной и той же уже закрытой deposit operation.

## 15. Phase I — cancellation/refund

1. Создать отдельную paid test booking.
2. Cancel dialog требует refund amount и reason.
3. После подтверждения даты освобождаются немедленно.
4. Refund хранится как отдельная durable operation.
5. Ошибка Stripe test refund не возвращает booking в calendar.
6. Успешный refund подтверждается webhook/reconciliation.
7. Проверить multiple payment parts для 30% + balance booking, если Phase G доступен.

## 16. Phase J — email и timezone

1. Запустить admin test email через существующий in-page control.
2. Проверить фактическое получение письма в согласованном inbox.
3. Проверить customer и admin booking confirmation.
4. Проверить payment-link, deposit-paid, refund и damage-retention templates.
5. Customer email показывает captured customer timezone; admin/business schedule показывает `America/Vancouver`.
6. Browser timestamps показывают viewer-local timezone с явным label.
7. Email failure не отменяет webhook-confirmed booking/payment.
8. Failed delivery видна в Overview и доступна Retry.

Если SES всё ещё sandbox/unverified, не маскировать это как application failure. В report отдельно записать SES identity/DKIM/sandbox status и какие recipients разрешены.

## 17. Phase K — Supabase verification

После каждого mutation проверять только необходимые безопасные поля:

- booking status и booking number;
- obligations type/status/amount/currency/due time;
- Stripe object IDs и event processing status;
- notification status/error code без секретов;
- audit action/actor/time;
- damage claim amount/reason и evidence metadata без object secrets;
- отсутствие duplicate sessions, obligations, invoices, refunds и worker jobs.

Проверить Storage:

- `damage-evidence` private;
- signed access короткоживущий;
- после удаления/expiry прямой доступ отклоняется;
- `rental-media` public only for published RV media;
- publishable/anon roles не получают backend-only table/function access.

Не печатать полный customer record, service/secret key, connection string, raw access token или Checkout client secret.

## 18. Stop conditions

Немедленно остановить транзакционные тесты, если:

- payment config показывает `live` или ключ не `pk_test`;
- account ID не `acct_1SpY7K2MR4C4rvKM`;
- webhook signature validation не работает;
- browser UI подтверждает оплату до webhook;
- создаётся duplicate charge/session/invoice/refund;
- admin endpoint доступен default/anonymous user;
- тест затрагивает existing non-test booking;
- email или evidence раскрывает secret/private object publicly;
- backend deployment commit нельзя однозначно связать с проверенным source;
- production DB миграции не совпадают с repository history.

При stop condition ничего не «чинить по живому» без анализа. Сначала сохранить безопасные evidence, закрыть test reservations/refunds штатным способом и описать blocker.

## 19. Cleanup

После тестов:

1. Все созданные test bookings должны быть `cancelled`, `expired` или полностью завершены по сценарию.
2. Все Stripe test deposits/refundable amounts должны быть полностью refund/settled согласно test case.
3. Открытые test Checkout Sessions и Invoices должны быть expired/voided штатным provider flow.
4. Test availability должна снова быть свободна.
5. Test evidence удалить через backend storage cleanup, если бизнес/audit контракт не требует сохранить его metadata.
6. Audit, Stripe events, payment operations и notification history не удалять — они являются доказательством теста.
7. Старые/тестовые auth sessions отозвать.
8. Убедиться, что нет `pending`, `submitted`, overdue или failed test operation без записанного решения.
9. Не удалять/редактировать legacy bookings.

## 20. Финальный отчёт

Создать новый файл в `plans/reports/` с датой выполнения. Для каждого сценария записать:

| Scenario | Result | Booking | Stripe objects | Webhook proof | Email proof | Cleanup |
|---|---|---|---|---|---|---|
| Deployment alignment | PASS/FAIL | — | — | — | — | — |
| Auth/default/admin | PASS/FAIL | test user only | — | — | optional | revoked |
| 30% Checkout | PASS/FAIL | booking number | safe IDs | event IDs/status | received/failed | cancelled/completed |
| 100% + 3DS | PASS/FAIL | booking number | safe IDs | event IDs/status | received/failed | cancelled/completed |
| Expiry | PASS/FAIL | booking number | safe IDs | event/status | optional | expired |
| Manual booking | PASS/FAIL | booking number | safe IDs | event/status | received/failed | expired/cancelled |
| Balance Invoice | PASS/FAIL/DEFERRED | booking number | safe IDs | event/status | received/failed | settled |
| Deposit full refund | PASS/FAIL | booking number | safe IDs | event/status | received/failed | refunded |
| Deposit partial retention | PASS/FAIL | booking number | safe IDs | event/status | received/failed | settled |
| Cancellation/refund | PASS/FAIL | booking number | safe IDs | event/status | received/failed | cancelled |
| Supabase/Storage | PASS/FAIL | safe references | — | — | — | test object removed |

Report должен содержать:

- точные deployed frontend/backend commits;
- время начала/окончания в UTC и `America/Vancouver`;
- desktop/mobile browser и viewport;
- список сценариев и фактический результат;
- только безопасные Stripe object/event IDs, без secrets/client secrets;
- email/SES status;
- Supabase advisor findings с remediation links;
- список оставшихся blockers;
- явный вывод: `TEST MODE READY` или `NOT READY`.

Даже `TEST MODE READY` не является разрешением на live Stripe, production customer launch или изменение домена.

## 21. Первый шаг в новом контексте

1. Прочитать оба `AGENTS.md`, этот файл и `BOOKING_FLOW_HANDOFF.md`.
2. Снова read-only проверить remote `main/dev` commits и фактические deployed behaviors.
3. Если mismatch из раздела 3 сохраняется, остановиться до promotion latest `dev` → `main`.
4. После зелёного Gate 0 начать Phase A/B.
5. Перед первым booking/payment submit ещё раз показать владельцу точный набор test data и создаваемые test-mode side effects; после подтверждения выполнить фазы C–K без перехода в live.

