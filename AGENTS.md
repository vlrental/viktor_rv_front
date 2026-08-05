# AGENTS.md - Viktor RV Frontend

## Режим выпуска `быки`

- Автовыпуск запускается только когда текущий batch реально изменил продуктовый код. Старый незатронутый WIP не считается изменением этой задачи.
- Pencil/`.pen`, Markdown, планы, `AGENTS.md`, `.agents/**`, `.codex/**`, тесты bull-оркестратора и другой rules/control-plane batch завершаются тихим no-op без gate, commit, push и deploy. Полностью отменённая до `Stop` правка кода тоже no-op; смешанный batch с реальным кодом выпускается обычно.
- Реальный выпуск выполняет отдельный worker `gpt-5.6-sol` с `reasoning_effort=low`; корневая сессия только координирует, ждёт и проверяет результат.

- Режим `быки` активен постоянно до прямой команды владельца `медведи`. Это постоянное разрешение последней задаче этого репозитория на stage, один общий commit, push разрешённой ветки и production deploy без повторного вопроса; оно является явным исключением из старых локальных запретов на эти действия.
- Перед финалом задача проверяет app-wide очередь по точному корню этого репозитория. Если других активных code writers нет, она становится интегратором всего worktree: ждёт чужие сборки, запускает полный обязательный project gate, исправляет ошибки до зелёного результата и выпускает общий батч. Перекладывать выпуск на следующую задачу запрещено.
- Если существующий корневой `make d` действительно является документированным production deploy этого проекта, использовать его. Иначе применять только фактически подтверждённый project-specific deploy/CI flow; переносить команду из другого проекта или угадывать запрещено.
- Красный gate, активный writer, изменение snapshot, неверная ветка или неизвестный deploy contract блокируют commit/push/deploy. Force-push, destructive data actions, секреты, несовместимые контракты и browser/computer-use без текущего разрешения режим не разрешает.
- `медведи` отключают автоматические stage/commit/push/deploy, сохраняя обязательные проверки. Массовая синхронизация самих файлов правил не выпускает старый продуктовый WIP; режим применяется к следующему продуктовому батчу.


## RV-only scope

- Viktor RV is an RV-only project. Do not add, seed, import, document, or expose boats or boat bookings in the new frontend, backend, database, or generated content. The legacy Simvoly site remains unchanged until a separately approved cutover.

## Local project layout

- Frontend repository: `/Users/viktoriiakarpova/Projects/it_work/viktor_rv_front`.
- Backend repository: `/Users/viktoriiakarpova/Projects/it_work/viktor_rv_back`.
- Before changing the current booking flow, read `BOOKING_FLOW_HANDOFF.md` in the frontend repository. It documents the unified overlay architecture, browser persistence, completed fixes, verified behavior, and remaining end-to-end checks.
- Treat these as the paired frontend and backend repositories for the Viktor RV project.
- Keep shared project context and cross-repository working rules synchronized in both repositories' `AGENTS.md` files. When adding or changing a rule that applies to the whole project, write it to both the frontend and backend instructions.
- The frontend is the default working directory for this workspace. When a task involves API behavior, server code, routes, database access, or backend integrations, inspect the backend repository at the path above instead of searching for it.
- The user commonly runs both projects locally. Verify the actual processes and ports when runtime state matters; do not assume that a service is currently running solely from this note.

## Cross-repository implementation authorization

- For Viktor RV tasks, the user grants standing authorization to inspect and modify both the frontend and backend repositories as needed to complete the requested work, without asking separately before crossing repository boundaries.
- This standing authorization covers repository code, configuration, documentation, tests, and the non-destructive Supabase schema migrations required by requested Viktor RV implementation work. It does not by itself authorize a production application deployment or restart, DNS/domain changes, live-payment activation, secret changes, destructive database operations, production data deletion, or broad production-data rewrites; those actions still require a direct user request.
- Execute in-scope work end to end with the available tools instead of handing the user terminal commands or routine manual steps. Ask the user to act only when authentication, permissions, a third-party approval, or a genuinely destructive/high-impact decision cannot be completed safely by the agent.

## Project design

- Viktor RV UI design must always be created, updated, and verified in Pencil. Do not use or suggest Figma for this project.
- The canonical project design is available through the connected design tool. Open and inspect it directly when design context is needed; do not ask the user to provide it again unless access fails.
- Design Node IDs: `IUHnT`, `x8t0A0`, `I6W2Es`, `raX6S`, `rmfa4`, `ODW3r`, `f1GuCf`, `ns0xG`, `LaBip`, `iKgTN`, `Oijpd`, `MEsd0`, `XgqBg`, `w19Mf`, `jr5XP`, `CdnCR`, `X9ejnB`, `g9upP`, `qaZRF`, `yTWGi`, `Al6fI`, `lsQAl`, `eb6Ck`, `cOu0u`, `YuNUS`, `TDhXo`, `M4DJcJ`, `e8z6o4`, `K7A9o`.
- Use these nodes as the visual source of truth when implementing or reviewing the corresponding frontend UI.

## Supabase access and diagnostics

- The correct Supabase project ref is `pwhlkpwlansarstmstge`. Never substitute the separately visible project `oysipecbuubmjgdiqrku` merely because a connector can access it.
- Before claiming that the correct Supabase project is inaccessible, inspect the backend repository's ignored `.env.prod` safely. It contains the production IPv4 Session Pooler `DATABASE_URL` for `pwhlkpwlansarstmstge` and can be used for scoped read-only SQL diagnostics even when the Supabase connector returns permission denied or the local Supabase CLI is unavailable.
- Never print, paste, log, or return the full `.env.prod`, `DATABASE_URL`, database password, service-role key, or other secrets. Check only exact project-ref matches, variable presence, safe parsed host/user metadata, or query results that do not expose credentials.
- Distinguish access surfaces: a working `DATABASE_URL` proves PostgreSQL access only; Supabase MCP/Management API access is separate; Storage upload and signed-URL E2E require backend-only `SUPABASE_URL` plus the preferred `SUPABASE_SECRET_KEY=sb_secret_...` (or legacy `SUPABASE_SERVICE_ROLE_KEY`). New secret keys are sent only as the `apikey` header, never as a Bearer JWT. A connector permission error does not prove that database access is unavailable.
- The backend local `.env` normally targets local PostgreSQL and must not be mistaken for production Supabase. Use `.env.prod` only for read-only production inspection unless the user directly authorizes a production database write or migration.
- For every requested Supabase schema, function, trigger, index, RLS, grant, or migration change, create and test the versioned SQL locally first, then apply the same non-destructive change to the correct production project `pwhlkpwlansarstmstge` and verify the remote result before reporting completion. Do not stop after creating local tables or migration files unless the user explicitly asks for local-only work.
- The user has given standing direct authorization for these non-destructive Supabase migrations as part of requested implementation work. Destructive DDL, production data deletion, irreversible backfills, broad production data rewrites, project deletion, secret rotation, and billing or organization changes still require separate task-specific confirmation.
- Run Supabase and database commands yourself whenever credentials and tools are available; do not respond with commands for the user to copy. If the preferred connector is unavailable, use the backend-only production `DATABASE_URL` through a safe PostgreSQL client, keep secrets out of output, and verify the expected schema plus relevant security checks afterward.

## Page architecture

- Prefer completing actions inside the user's current page, panel, dialog, drawer, or established workflow.
- Every overlay, dialog, or drawer with a visible close (`×`) control must also close when the user presses `Escape`. Use the same guarded close path as the visible control; when overlays are nested, `Escape` closes only the topmost dismissible layer.
- Do not create a dedicated page for authentication, callbacks, transient success/error states, validation, or a single small action when the existing interface can contain it clearly.
- Compatibility routes may exist only as invisible immediate redirects; they must not render a standalone user-facing screen.
- Add a new page only when it represents a distinct, persistent destination in the product's information architecture and materially improves navigation.

## Application roles

- Application roles are stored in the backend `app_users.role` column and are limited to `default` and `admin`.
- New email/password and Google users receive the `default` role. Administrative access is granted only by changing the database role to `admin`; there is no standalone admin API token.

## Booking notifications

- Booking confirmation email is sent by the backend through Amazon SES SMTP in `ca-central-1` from `no-reply@vlrental.ca`; the administrator recipient is `Vlrental.ca@gmail.com`.
- A successful booking remains valid if email delivery fails; the confirmation page must report the email result returned by the backend.
- Store timestamps in UTC. The RV delivery/return business schedule remains `America/Vancouver`; customer emails also show the validated browser timezone captured with the booking, and browser UI timestamps use the viewer's local timezone with an explicit timezone label.
- Failed customer/admin email deliveries must retain a sanitized error code/message, appear in the existing admin overview, and be retryable from that page. Email failure never rolls back a webhook-confirmed payment or booking.
- Public authentication, quote, booking, contact, newsletter, review, address and delivery-estimate endpoints must use bounded per-client rate limits. Stripe webhook delivery is protected by signature verification and is not placed behind the public client rate limiter.
- Frontend and backend production workflows must run formatting, tests, warning-free lint checks, and the relevant browser/database target checks before any deployment job can begin.
- Google OAuth callbacks may return only a short-lived one-time login code. Access and refresh tokens are created by the backend exchange endpoint, never placed in callback URLs, and refresh rotation must be atomic so the same refresh token cannot succeed twice.
- Logout must revoke the matching backend session even when its access token has expired; clearing browser storage alone is not a logout.
- The frontend never queries Supabase Data API directly. The `public` application schema is backend-only: revoke schema `USAGE` plus all table, sequence, and function privileges from `PUBLIC`/`anon`/`authenticated`, lock down matching default privileges, and keep RLS enabled; backend-only one-time login codes are stored only as SHA-256 hashes.
- Auth access/refresh tokens, private booking access tokens, and pending Checkout client secrets may persist only in browser session storage, with one-time migration/removal of complete legacy local-storage token sets.
- HTTP tracing may log only the request method and URL path, never query strings or headers. Authentication, booking, payment-status, owner, admin, and legacy RPC responses must use `Cache-Control: no-store` and security headers.
- Ordinary JSON/form bodies are capped at 256 KiB; only Stripe webhook (1 MiB) and authenticated image uploads (11 MiB request, 10 MiB image) receive larger route-specific limits. Uploaded JPEG/PNG/WebP content must match its declared file signature.

## Git branches and deployment

- Develop only on the `dev` branch. Push ordinary work to `origin/dev`; do not develop directly on `main`.
- `main` is the production branch. Promote `dev` to `main` only through `make dm` after relevant checks pass and the user directly requests deployment.
- `make d` pushes `dev` without deploying production. `make dm` pushes `dev`, promotes the same commit to `main`, and triggers the production GitHub Pages workflow.
- Do not connect, repoint, publish, or otherwise modify `vlrental.ca` until the complete booking flow, availability calendar, authentication, email delivery, and deployment have been configured and verified end to end, and the user explicitly approves the domain launch.
- Before that approval, preserve the current `vlrental.ca` DNS and live site so existing users are not disrupted. Do not change Wix DNS, SES domain records, the production custom-domain configuration, or the live domain routing.
- The maximum permitted deployment before domain-launch approval is a test frontend deployment to the repository's GitHub Pages URL. A GitHub Pages test deployment must not attach or redirect `vlrental.ca` and still requires a direct user request.
- Do not deploy or restart the production backend, change production API routing, or promote a test build onto the live domain as part of a GitHub Pages-only request.

## SSH access

- Connect to the VL Rental server with `ssh vlrental`.
- SSH target: `root@159.203.47.33`, port `22`.
- The local SSH alias must use `~/.ssh/vlrental` with `IdentitiesOnly yes`.
- If the alias is unavailable, use `ssh -i ~/.ssh/vlrental root@159.203.47.33`.
- Never copy the private key into this repository, commits, logs, task descriptions, or chat output.
- Do not deploy, restart services, change production files, or execute production database writes over SSH without a direct user request.

## Safety

- Do not commit, push, deploy, or modify production without a direct user request.
- Check every changed text file for UTF-8/mojibake before completion.

## Booking schedule and test payments

- RVs are delivery-only; customer pickup is not supported. Delivery/setup is at 2:00 PM and return is at 11:00 AM in `America/Vancouver`.
- A following customer may receive the same RV at 2:00 PM on the previous customer's return date; the 11:00 AM–2:00 PM gap is reserved for cleaning and transport.
- RV delivery is limited to 150 km one way from the Kelowna base. The fee is CA$150 through 40 km, then CA$2/km in each direction (CA$4 total for the two-way journey per additional one-way kilometre).
- Customers may book an RV for one or more nights. A 1–2 night booking keeps the customer-selected delivery and return dates everywhere, but is priced at the 3-night minimum and blocks availability through the full 3-night minimum window. Backend code is the source of truth for billable nights, blocked-until timestamps, date-to-time conversion, and availability.
- Every RV quote includes a mandatory `RV Preparation Fee` of CA$97 once per booking.
- Every RV quote includes mandatory `Stationary Plus Protection` at a fixed CA$150 for the first three booked nights, plus CA$30 for each additional night, automatically calculated from the calendar date difference. Both mandatory charges must appear as separate quote line items, and backend quote code is the pricing source of truth.
- At initial booking, if RV delivery is more than 30 days away, the customer must pay 30% of the trip price immediately. Calculate the 30% only from the trip price; the separate refundable CA$1,000 damage deposit is not part of this percentage. If delivery is 30 days away or less, the customer must pay 100% of the trip price immediately.
- For a booking made more than 30 days before delivery, the remaining trip-price balance becomes due exactly 30 days before the delivery date, bringing the booking payment to 100%.
- Every booking requires a separate refundable CA$1,000 damage deposit by Interac e-Transfer to `protrailercare@gmail.com`, due exactly 48 hours before RV delivery. Never include this deposit in Stripe Checkout or a Stripe invoice. Delivery remains blocked until an administrator verifies receipt.
- Whenever a scheduled trip payment or refundable damage deposit becomes due, notify the customer immediately by email. Stripe trip-payment notices include the existing secure link; deposit notices include the e-Transfer recipient and booking number. Critical failures and overdue actions also notify the administrator and appear in the admin dashboard.
- Stripe is test-first. `PAYMENTS_ENABLED=true` is allowed only with test keys and `STRIPE_MODE=test` until the complete test report is green and the user separately gives direct approval for live activation. Never add live keys, a live webhook, production payment routing, production deployment, or domain changes without that approval.
- The backend immutable quote is the only source of Stripe amounts. Do not bind fixed Stripe Price IDs. The frontend receives only the publishable key, mode, and expected account ID; secret and webhook keys never enter frontend code, the database, logs, commits, or chat output.
- A verified Stripe webhook is the only source of trip-payment truth. Browser completion and callback state never confirm a booking, invoice, or Stripe refund. The e-Transfer damage deposit becomes paid only through the authenticated admin confirmation action, which creates an audit event and emails both the customer and `vlrental.ca@gmail.com`.
- Initial payment is 30% of trip price when delivery is more than 30 days away and 100% at 30 days or less. A 30% booking receives one automatic balance invoice exactly 30 days before delivery.
- The CA$1,000 damage deposit never uses Stripe. After return and inspection, an admin records a full e-Transfer return or retains documented damage and records the returned remainder; legacy already-paid Stripe deposits retain their existing reconciliation path.
- `Delivered` (`active`) is blocked until the trip price and refundable CA$1,000 damage deposit are fully paid. `Returned` (`completed`) enables an admin to refund the full deposit or retain documented damage and refund the remainder. Seven days after return is the admin decision deadline.
- Retaining any deposit amount requires a positive amount within the available deposit, a non-empty reason, at least one private photo, a confirmation dialog, and an audit event. Evidence is backend-only/private and is viewed only with short-lived admin-authorized signed URLs.
- Phone/manual bookings use the same quote rules and a dynamic Stripe Checkout Session, reserve the RV for two hours, and expire without manual cash/bank `paid` overrides.
- Only the backend `admin` role may access `/admin` data or actions. Financial and lifecycle actions require a confirmation dialog and immutable audit log; arbitrary generic status changes are not used by the UI.
- When `PAYMENTS_ENABLED=false`, preserve the legacy no-card `confirmed` / `test_paid` test-booking behavior and do not create real payment rows.
