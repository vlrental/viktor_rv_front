# Viktor RV frontend — AI handoff

Last updated: 2026-08-04

This document records the current frontend booking architecture, the problems fixed during the one-page booking work, the verified behavior, and the safe continuation plan for future AI agents.

Кратко: главный результат работы — один общий оверлей бронирования на главной странице и в карточке RV. Он ведёт пользователя по цепочке даты и гости → RV → адрес доставки → допы и точная цена → вход и подтверждение. Ниже зафиксированы устройство, исправления, проверенные сценарии и ограничения, чтобы следующий AI не возвращал старые окна и не ломал зависимое состояние.

## Product direction

- Viktor RV is RV-only. Never add boats or boat bookings.
- The primary booking experience is a single overlay. A customer should not be sent through separate date, catalog, delivery, extras, and checkout screens during the normal flow.
- The customer should be able to choose dates and guests, select an RV, calculate delivery, select extras, see the live total, sign in, and confirm without leaving the overlay.
- Dates-first and RV-first are both supported without separate hidden modes. Before dates are selected, step 2 lists every active RV that fits the guest count and selecting one opens its live calendar. After a valid date range is selected, step 2 lists only RVs the server marks available for that range; if none are available, the empty state keeps the customer inside the overlay and returns them to the date calendar.
- Completed sections collapse into summaries and can be reopened for editing.
- The home page is the primary catalog. `/catalog` is retained only as a compatibility route and redirects to the RV section on the home page.

## Interac e-Transfer damage deposit (2026-08-04)

- This section supersedes the historical Stripe damage-deposit and `Pay everything now / all-in Checkout` sections below for all new and unpaid bookings. Those older sections remain only as compatibility history for already-paid Stripe deposits.
- Stripe collects only the trip price: 30% at booking when delivery is more than 30 days away, otherwise 100%, plus the scheduled 70% balance when applicable. The refundable CA$1,000 damage deposit must never be added to a new Stripe Checkout or Invoice.
- Every booking creates a separate `damage_hold` obligation with `collection_method='e_transfer'`, due exactly 48 hours before delivery. The customer sends it to `protrailercare@gmail.com` and includes the booking number.
- After the trip-price webhook confirms the booking, the embedded Stripe layer closes into a second nested deposit-instruction overlay. It shows CA$1,000, the exact due moment, the recipient, copy action, delivery gate, and account link. Desktop/mobile Pencil source-of-truth nodes are `bsONy` and `H3hktc`.
- The account booking card shows `Awaiting e-Transfer`, due time, recipient, and copy action until the authenticated admin action confirms receipt; it then shows `Paid`. Desktop/mobile Pencil nodes are `LMlZP` and `vbbDb`.
- Admin confirms receipt through `POST /api/v1/admin/bookings/{booking_id}/damage-deposit/e-transfer/confirm`. The backend atomically creates the manual payment record, marks the obligation `succeeded`, writes booking/audit events, and queues idempotent paid-deposit emails to the customer and `vlrental.ca@gmail.com`.
- `Delivered` stays blocked until all trip obligations and the e-Transfer deposit are paid. After return, the existing evidence and seven-day decision rules still apply; an admin records a full e-Transfer return or documented retained damage plus the returned remainder. Already-paid legacy Stripe deposits continue through their existing provider reconciliation path.

## Repositories and runtime safety

- Frontend: `/Users/viktoriiakarpova/Projects/it_work/viktor_rv_front`
- Backend: `/Users/viktoriiakarpova/Projects/it_work/viktor_rv_back`
- Inspect the backend repository for API behavior instead of guessing contracts.
- The user often has both applications running locally. Never stop, replace, or restart their processes unless directly requested. Check the actual processes and ports first.
- Develop only on `dev`. Do not commit, push, deploy, change production, or touch `vlrental.ca` without a direct request.

## Main implementation map

- `src/pages/home.rs`
  - Owns the applied home search state.
  - Opens the unified overlay from the date, guest, search, and listing controls.
  - A date or guest click opens step 1. The main search button opens step 2 only when a valid date range already exists; otherwise it opens step 1.
  - Closing the overlay applies and persists the current dates, guests, location, and radius, then refreshes the home listings.
- `src/pages/booking_overlay.rs`
  - Contains `UnifiedBookingOverlay`, the canonical booking UI for both the home page and RV detail pages.
  - Owns the five collapsible steps, address suggestions/history, delivery calculation, extras, quote refresh, authentication, guest details, and test-booking confirmation.
  - Locks background page scrolling while mounted and restores the previous page scroll state when closed.
- `src/pages/rv_detail.rs`
  - Uses the same `UnifiedBookingOverlay` with the current RV slug preselected.
  - The old RV-detail date/booking modal is not part of the normal booking path.
- `src/pages/catalog.rs`
  - Contains reusable search normalization, calendar, filters, listing cards, and catalog states.
  - `/catalog` redirects to the home RV section.
- `src/api.rs`
  - Contains API models, browser persistence helpers, catalog/rental/address/delivery/quote/booking calls, authentication, and public form calls.
- `assets/main.css`
  - Contains the home catalog, unified overlay, saved-address, responsive, and mobile navigation styles.

## Unified booking flow

### Step 1 — Dates and guests

- Dates are interpreted in `America/Vancouver`.
- Delivery/setup is 2:00 PM and return is 11:00 AM; the backend remains the source of truth for timestamps and availability.
- Start date must be strictly later than today. Today and past dates are disabled and stale saved values are cleared.
- Customers may select one or more nights. For a 1–2 night stay, the UI keeps and displays the selected delivery/return dates, while the backend prices billable and per-unit items at the 3-night minimum and protects availability through the full three-night window.
- Guests are clamped to the supported range.
- Every newly opened unified booking overlay resets the guest count to one. A guest count saved by an earlier catalog search must not silently filter the RV choices before the customer selects guests in the current overlay.
- Changing dates or guests causes the available RV resource to run again. Do not precompute the search outside the reactive resource closure; doing so previously left the RV list permanently stale.
- Availability UIs fail closed. A request error, mismatched rental/range, unexpected schedule metadata, or malformed blocked interval keeps calendar days disabled and exposes a retry action; it must never fall back to showing every day or RV as available.
- When an RV is selected first, the calendar loads that model's live unavailable intervals and disables date choices that cannot form a valid minimum stay. The 11:00 AM return and 2:00 PM next-delivery turnover remains selectable.
- After an RV is selected without dates, step 1 opens its custom live-availability calendar directly. Customers choose delivery and return dates manually, with unavailable days disabled.
- On mobile widths, the step 1 calendar changes months with a horizontal swipe: swipe left for the next month and right for the previous month. Short gestures and primarily vertical scrolling do not change the month, the calendar cannot swipe earlier than the current month, and the arrow buttons remain available.
- Every dismissible booking overlay with a visible close control must close through the same guarded path when the user presses `Escape`.

### Step 2 — Choose an RV

- RV choices use image cards based on the home listing design, including name, capacity, summary, and nightly price.
- Step 2 uses two server catalog views: the no-date response supplies every model that fits the guest count before a date range exists, and the dated response becomes the displayed list after valid dates are selected.
- Step 2 keeps a compact guest stepper in its upper-right toolbar so customers can see and change the active capacity filter without returning to step 1. Its fixed desktop width and mobile wrap must not shift or overflow the RV card grid.
- Step 2 is always clickable and never becomes an empty dead end merely because all RVs are booked for the selected dates. Without a valid range, selecting any model opens step 1. With dates, only available cards continue to delivery; when none are available, an inline empty state offers `Choose new dates` and reopens step 1.
- If either the dated RV list or the selected RV calendar fails to load, the overlay offers an explicit retry. It does not expose the unfiltered fleet or unlock unverified dates while that retry is pending.
- RV detail pages pass their slug as the initial selection, but the customer can choose another available RV in the same overlay.
- Each RV card has previous/next gallery controls that appear on pointer hover and remain visible on touch devices. The controls rotate through the project gallery without selecting the RV.
- Cards show the published server rating and review count. `Read comments` opens a nested, independently scrolling review overlay; its close control and `Escape` dismiss only that topmost review layer.
- The `DATES` row in an RV detail booking card is an interactive control. It always opens the unified overlay directly on step 1 so saved dates can be reviewed or changed; the main `Open booking` button continues from the next relevant step.
- Every RV detail page also shows a compact live three-month availability calendar immediately before `About this RV`. Booked or otherwise invalid days are disabled. Selecting a valid delivery and return range of at least three nights opens the existing unified booking overlay for that RV on the delivery-address step; it never creates a second booking flow or standalone calendar page. Previously saved dates are reconciled against the returned schedule and cleared with an inline notice when the server confirms that the RV is no longer available.

### Step 3 — Delivery address

- The address step includes an interactive map centred on the Kelowna base with the 150 km delivery area. After a successful calculation it also marks the selected delivery address; map wheel gestures must not scroll the page behind the overlay.
- Suggestions begin after at least three characters and are debounced.
- Suggestions first use `/api/v1/address-suggestions`; the API layer has a Canadian-address fallback.
- The customer may select a suggestion or enter an exact address and press `Calculate delivery`.
- Delivery calculation uses `/api/v1/rentals/{slug}/delivery-estimate`; the API layer has a client fallback if the route is unavailable.
- Policy: maximum 150 km one way from Kelowna; CA$150 through 40 km, then CA$2/km in each direction (CA$4 total for the two-way journey per additional one-way kilometre).
- A successful address is stored in `vl_delivery_addresses` in browser local storage. The newest unique address appears first, history is limited to five entries, clicking an entry selects it, and its adjacent remove button deletes it.
- The next steps remain unavailable until delivery is calculated and confirmed within range.

### Step 4 — Extras and trip details

- Extras are visual cards with icon, description, recommendation state, charge type, price, and clear add/remove control.
- Event attendance and movement after delivery use explicit Yes/No segmented controls.
- Selecting or removing an extra updates the quote automatically.

### Step 5 — Guest details and confirmation

- An unauthenticated customer signs in or creates an account inside the overlay.
- Before Google sign-in leaves the page, the current dates, guests, selected RV, calculated delivery, extras, and any guest-form values are saved in transient browser session storage. The OAuth callback is completed before route components mount, then the same home or RV-detail booking overlay reopens on step 5 with the authenticated session. Delivery and quote data are refreshed after restoration; passwords are never stored.
- After authentication, the form collects name, email, phone, optional notes, and acceptance of rental terms.
- Confirmation uses the current server quote and calls `/api/v1/bookings` with the saved access token.
- Stripe Checkout receives the signed-in booking email as `payment_intent_data[receipt_email]` for trip-price payments. Legacy already-linked Stripe deposit objects retain their historical receipt/refund behavior, but new damage deposits never create a Stripe object.
- After an unpaid Stripe reservation is created, `Change booking details` is available both in step 5 and inside the payment overlay. The confirmation action calls the private `DELETE /api/v1/bookings/{booking_id}/pending-payment` route with the one-time booking token. The backend permits this only for `pending_payment` + `unpaid`, expires the active Stripe Checkout Session first, then expires the old booking/payment records and releases availability. Only after that succeeds does the frontend clear `vl_pending_booking_payment`, unlock all five sections, return to dates, and request a fresh server quote. A failure leaves the current reservation locked and visible; paid or advanced bookings can never enter this edit path.
- Until Stripe is explicitly enabled, the UI must continue to say this is a test booking and that no card is collected or charged.
- On webhook-confirmed trip-payment success, the booking response is saved, Stripe closes into the nested e-Transfer instruction overlay, and dismissing that overlay returns to the RV confirmation state with an account link.

## Live price behavior

- A local preview can be shown from nightly rate, nights, selected extras, delivery fee, and the separate refundable damage deposit.
- Every quote automatically includes the mandatory `RV Preparation Fee` of CA$97 once per booking.
- Every quote automatically includes mandatory `Stationary Plus Protection` at a fixed CA$150 for the first three booked calendar nights, plus CA$30 for each additional night (3 nights = CA$150, 4 = CA$180, 5 = CA$210).
- Both mandatory charges are separate server quote line items and cannot be removed as extras.
- The customer-facing trip price excludes the separate refundable CA$1,000 damage deposit. The deposit remains a compatibility quote field and a separate line item for transparency, but it must never be included in the 30% booking-payment calculation.
- The refundable CA$1,000 damage deposit is due by Interac e-Transfer to `protrailercare@gmail.com` exactly 48 hours before RV delivery. After return and inspection, an administrator records the full manual return or documented retained damage plus the returned remainder.
- For trips booked more than 30 days ahead, 30% of the trip price is due at booking and the balance is due 30 days before delivery. Trips booked within 30 days require the full trip price at booking.
- A verified `invoice.paid` event for the remaining balance queues separate idempotent customer and administrator emails. The customer email confirms that the trip price is fully paid and links to the paid Stripe Hosted Invoice Page where the invoice receipt is available.
- The authoritative total is the response from `POST /api/v1/quotes`.
- Quote `ends_at` is always the customer-visible return time. The backend-only `blocked_until_at` may extend through the 3-night minimum for a short stay and is used for overlap checks, public availability, and calendar exports.
- The exact server quote is requested only when dates, RV, and an in-range calculated delivery address are ready.
- The quote must refresh when the selected RV, dates, guests, delivery address/result, event choices, towing choice, or extras change.
- The summary sidebar shows trip-price line items, the exact CAD trip price, the separate refundable damage deposit, and payment timing when available. It must not imply that a preview is final or that the deposit is part of the trip price.

## Browser persistence

- `vl_catalog_search`: applied location, radius, dates, and guests.
- `vl_delivery_addresses`: recent calculated delivery addresses.
- `vl_saved_rvs`: saved/favorite RV slugs.
- `vl_access_token`, `vl_refresh_token`, `vl_auth_user` (session storage): authenticated session. Older local-storage token sets are migrated together once and removed; closing the browser session requires signing in again.
- `vl_trip_draft`: last booking draft used for confirmation/conflict recovery.
- `vl_active_quote`: last authoritative quote.
- `vl_last_booking`: last successfully created booking shown on the confirmation page.
- `vl_pending_booking_payment` (session storage): the current private pending booking, reusable embedded Checkout client secret, and private status token. When present, reopening the unified overlay must go directly to step 5, check webhook-backed status first, and remount the same Checkout Session without creating another booking.
- `vl_booking_auth_continuation` (session storage): transient pre-Google-auth overlay state used once to reopen the same booking window on step 5 after callback. It is consumed on return and never contains the customer's password.

## Verified rental reviews

- Published ratings and counts are returned with catalog rentals; comment bodies load only when the customer opens the review preview.
- Review comments are stored by the backend in `rental_reviews`. Browsers do not access the table directly.
- A signed-in customer may publish one review per booking only when the booking belongs to that account, has status `confirmed`, `active`, or `completed`, and has payment status `paid` or `test_paid`.
- Eligible bookings show the inline review form on the account page. Ratings are restricted to 1–5 and comments to 10–2,000 characters.
- Search selections intentionally survive a full refresh. Invalid or stale dates are normalized on load. Address history also survives a refresh but delivery distance and quote are recalculated for safety.

## Problems fixed in the 2026-07-13 audit

- The home date control incorrectly opened the RV selection step. It now opens the date step.
- Home search state and overlay state could diverge. Overlay close now applies and persists the current values.
- The RV resource captured a non-reactive, precomputed search. Changing dates or guests did not reload RVs. It now reads all relevant signals inside the resource closure.
- The calendar allowed today even though the backend rejects a same-day start. The frontend now requires a future start date and clears stale same-day values.
- Address search previously failed without a useful fallback. Suggestions and delivery calculation now have fallback behavior, and successful addresses are stored as reusable history.
- RV choices in the overlay were plain text rows. They now use visual listing cards consistent with the home page.
- Extras were plain rows and did not clearly communicate value or state. They now use descriptive cards and update the total automatically.
- RV detail used a separate old booking experience. The home and RV detail pages now open the same unified overlay.
- The page behind the overlay scrolled instead of the overlay. Body scroll locking and contained overlay scrolling were added.
- The separate catalog duplicated home functionality. The public catalog route now redirects to the home RV section.
- Several secondary issues were corrected: mobile navigation, contact/newsletter/sales validation, invalid RV slugs, saved RV state, sharing, account loading state, and misleading confirmation copy.
- Google sign-in previously mounted the booking route before the returned session was saved and did not preserve the open overlay. OAuth completion now happens before application mount, while a one-time continuation restores the same overlay and selections on step 5.

## Behavior verified manually

The following full path was exercised against the running local frontend without restarting the user's server:

1. Open the date control on the home page and confirm that the date calendar opens.
2. Choose future dates with at least three nights and choose the guest count.
3. Confirm that available RV image cards load for the new search.
4. Select an RV.
5. Enter and calculate a public campground address.
6. Confirm that distance and delivery fee appear and the exact server quote loads.
7. Add an extra and confirm that the exact total and extras summary update.
8. Open guest details and confirm that sign-in/account creation is shown when logged out.
9. Confirm that `document.body.style.overflow` is locked while the overlay is open.

No booking was created during this audit.

## Verification required before declaring end-to-end completion

These items were not completed with a real account during the audit and must not be described as verified until exercised:

- Email/password registration and login against the intended environment.
- Google authentication callback and return into the open booking flow.
- Successful authenticated test-booking creation.
- Availability-conflict recovery after another customer takes the selected RV.
- Confirmation state and email-delivery result displayed on the booked RV detail page after webhook-backed payment confirmation.
- Account page showing the newly created booking.
- Final responsive pass on common mobile Safari and Chrome viewport sizes.

Do not create a test booking, send email, or mutate production merely to run an audit. Obtain the user's approval and use the intended test environment.

## Compatibility surfaces that still exist

- `/checkout` remains only as an invisible compatibility redirect for saved legacy links. It restores safe date/guest search context and immediately opens the canonical Home booking overlay; it must never render or submit a second checkout form.
- The old full catalog implementation remains as reusable/dead code inside `catalog.rs`, while the public `/catalog` component redirects home. Do not restore it as a second customer-facing catalog without an explicit product decision.
- Do not delete these compatibility paths casually. First search for saved links, authentication return paths, and conflict-recovery references.

## Admin availability controls

- `/admin` is a server-authorized admin calendar page. The header shows `Close dates` beside `Book now` only when the saved session reports the `admin` role, but the backend always revalidates the live session role before reading or changing blocks.
- The existing Payments panel can resend the current Stripe payment email for an unpaid balance or refundable damage-deposit obligation. The backend permits only a `link_created` obligation on a confirmed/active booking, reuses the existing Stripe object, records an immutable audit event, and rejects paid, cancelled, expired, scheduled, failed, due-without-a-current-link, or initial-payment links. The admin action only inserts an idempotent durable outbox delivery; the notification worker claims it, sends it once per attempt, and applies bounded exponential retries so the API request cannot race the worker and send a duplicate email.
- An admin selects an RV, a closing date, a reopening date, and an optional internal reason. The backend stores a `source = 'admin'` availability block from delivery/setup at 2:00 PM through return at 11:00 AM in `America/Vancouver`.
- Admin blocks immediately participate in the same catalog, quote, booking, and concurrency checks as imported owner blocks. An attempted block that overlaps an active customer booking returns a conflict and is not created.
- The page lists future blocks created through the admin control and can reopen them. It cannot delete legacy Google blocks, external calendar blocks, or customer bookings.
- Admin access is granted only by changing `app_users.role` to `admin` after the intended account has authenticated. Never infer admin access from a frontend email comparison.

## Admin Center and Stripe test-mode handoff (2026-07-14)

- `/admin` remains the only persistent admin route. Overview, Bookings, Payments, Calendar, and Audit are embedded tab states; booking detail and phone booking use drawers, while financial/lifecycle decisions use nested confirmation modals. Mobile drawers are full-screen. No standalone admin action pages are added.
- The approved Pencil source states are desktop `HBpPe`, `K8YXx`, `p8p6P`, `l86oa9`, and `tjohf`; mobile `JNkTK`, `o6Esvq`, `IfQL5`, `BvlyF`, `RRAEN`, `GBXFF`, and `mgKti`; overlays `FoWiG`, `ujgoM`, `lNyUh`, and `ANiTF`.
- The customer payment step is a nested top-layer dialog owned by the existing `UnifiedBookingOverlay`, not content appended inside step 5 and not a standalone page. Closing it preserves the same pending booking and Checkout Session. Embedded Checkout completion only starts a webhook-backed confirming state; it must not navigate to a callback page or mark a booking paid in the browser.
- With Stripe test payments enabled, booking creation produces `pending_payment` plus a 30-minute payment reservation. A successful verified webhook moves it to `confirmed`; expiry moves it to `expired` and releases availability. A manual phone booking uses the same quote rules but reserves availability for two hours.
- The immutable backend quote supplies every Stripe amount. Initial payment is 30% more than 30 days before delivery and 100% at 30 days or less. A remaining balance uses one backend-created invoice due exactly 30 days before delivery. Fixed Stripe Price IDs are not part of this architecture.
- The CA$1,000 damage amount is a refundable Stripe charge requested 48 hours before delivery, not a `Gold` option and not an authorization hold. Extended Authorization is not used. Stripe's original processing fees are not returned to VL Rental, but they are never deducted from the customer's approved refund.
- `Delivered` maps to booking status `active` and is blocked without full trip payment and a paid deposit. `Returned` maps to `completed`; only then may an admin refund the deposit or retain documented damage and refund the remainder. Retention requires amount, reason, at least one private photo, confirmation, and audit logging. Seven days after return is the decision deadline.
- Cancellation immediately changes calendar availability before any Stripe request. Each refund part is recorded as a durable financial operation; a booking paid through separate 30% and 70% PaymentIntents is refunded across those PaymentIntents, and a provider failure never restores the cancelled booking to the calendar.
- Release, damage capture, and refund use durable `pending/submitted/succeeded/failed` operations. Final success is established by a verified webhook or authenticated reconciliation rather than a browser response. Damage-captured email is queued only after that final success.
- Admin contact edits update a booking-scoped snapshot and never silently change another booking's shared customer record. Manual booking, calendar changes, resend, evidence access/upload, reconciliation, and all financial actions are audited.
- The database foundation adds payment obligations, provider event reconciliation, durable financial operations, immutable admin audit events, notification delivery attempts, damage claims/evidence, worker claim fields, and private evidence storage. These tables have RLS enabled and all `anon`/`authenticated` privileges revoked; browsers never query them directly.
- Evidence supports an ignored local/test adapter and a backend-only Supabase private-storage adapter. The preferred production credential is `SUPABASE_SECRET_KEY=sb_secret_...`; the legacy service-role JWT remains a compatibility fallback. Modern secret keys are sent only in the `apikey` header. Uploads validate file signatures and size and set `Cache-Control: 0`, so a browser/CDN cache cannot outlive the short signed-access window. Access is short-lived and admin-authorized. Live Stripe startup is blocked unless durable Supabase evidence storage is configured.
- Stripe remains test-only. No live key, live webhook, production backend deployment, production payment routing, or `vlrental.ca` change is authorized by this handoff. The deposit model no longer depends on Extended Authorization.
- Backend Stripe requests are pinned to API version `2026-06-24.dahlia`; all Checkout Session creations include the stable integration identifier `viktor_rv_qpmxkzrt`. Configure and test the Dashboard webhook endpoint against the same API contract before any separately approved deployment or live activation. Prefer a least-privilege restricted test key (`rk_test_...`); standard test secret keys remain supported for compatibility.
- Stripe CLI test credentials and webhook forwarding were configured locally on 2026-07-14 for `acct_1SpY7K2MR4C4rvKM`; they remain ignored and server-only. Real test-mode Checkout creation/expiry, Invoice creation/payment, signed webhook handling, decline, 3DS, standard manual authorization, release, partial capture, and refund were exercised successfully.
- After the refundable-deposit decision, a new real test-mode smoke created and expired the dynamic CA$1,000 deposit Checkout, completed a full CA$1,000 charge/refund, and completed a CA$750 partial refund while retaining CA$250 from a separate CA$1,000 test charge. No live object was created.
- A complete customer embedded Checkout was exercised inside the unified overlay with an insufficient-funds test-card decline followed by a successful `4242` test payment. The verified webhook moved the local booking to `confirmed / partially_paid`; initial, balance, and CA$1,000 refundable-deposit obligations remained separate. Browser refresh/reopen reused the saved pending Session and did not create another booking.
- A second customer booking completed the real nested Stripe 3DS challenge inside embedded Checkout. Its real Hosted Invoice then produced `invoice.payment_failed` with a declining test method, stayed on the same Invoice while the card was replaced, and completed through a verified paid webhook. Admin Payments/detail showed the final webhook-backed result.
- On 2026-07-15, the full disposable-database verification was repeated against a clean PostgreSQL 17 instance: schema bootstrap, all three SQL safety suites, six ignored webhook/database tests, and both booking-schedule concurrency tests passed. The temporary database was removed after the run.
- The browser pass found and fixed two Checkout regressions: the evaluated async mount script must `return await` its result, and payment configuration must be read reactively so a saved pending Session remounts after config arrives. Regression tests cover the initial step and mount-script return contract.
- The one-page admin UI was exercised on desktop and 390 px mobile widths. Overview, Bookings, Payments, Calendar, and Audit stay embedded in `/admin`; booking and phone-booking overlays close with `Escape`. The mobile `More` selector no longer clips or pushes tabs outside the viewport.
- On 2026-07-15, customer Checkout was moved from the bottom of step 5 into a separate nested payment overlay above the booking. `Escape` and the close control dismiss only that top layer, the pending booking stays persisted, and reopening reuses the same Checkout Session. Navigation to the booked RV detail page and its confirmation banner occurs only after the backend reports webhook-confirmed status. The admin Calendar became a 14-day per-RV fleet scheduler with explicit 11:00 AM return and 2:00 PM delivery markers; mobile uses the schedule agenda instead of compressing the grid.
- On 2026-07-15, the global header `Book now` action was changed from a root-relative link into an in-app request for the canonical booking overlay. It stays on Home, routes every other page back through the SPA, and opens step 2 only for a complete saved date range. The lower mobile CTA no longer has decorative image layers intercepting taps, and `Escape` now closes the mobile navigation or account panel. The GitHub Pages release build disables DWARF debug symbols so `wasm-opt` can complete instead of silently falling back to an unoptimized bundle.
- While a pending Checkout Session exists, the customer UI must display the immutable `booking.total` and `booking.amount_due_now`, disable changes to dates/RV/delivery/extras, and never mix a new draft quote with the existing Stripe Session. The frontend blocks Checkout when `amount_due_now` is neither the full trip total nor the rounded 30% payment. The backend still creates Stripe from the immutable payment obligation and rejects webhook amounts or currencies that differ from that obligation.
- Auth sessions now expose the latest authenticated customer contact from the protected `/auth/me` response. Step 5 pre-fills that name and phone and collapses complete contact details into an editable summary. A new account with no saved customer contact still receives the minimal missing-contact fields because booking confirmation requires a name and phone; the signed-in email is locked to the authenticated account.
- Stripe Priority Support answered case `sco_Ut5ECLsXyQP2d9` on 2026-07-15 and declined Extended Authorization eligibility. The owner subsequently selected the refundable CA$1,000 charge/refund model. The backend, admin UI, customer copy, Terms and tests must use this selected model; live activation remains separately prohibited until the complete new test report is green and directly approved.
- On 2026-07-15, production access was recovered through the ignored backend `.env.prod`, which points to the correct Supabase project `pwhlkpwlansarstmstge`; the unrelated visible project `oysipecbuubmjgdiqrku` was not used. The three pending versioned admin/Stripe/security migrations were applied, all three production SQL safety suites passed, and the private `damage-evidence` bucket was confirmed with a 10 MiB limit.
- On 2026-07-15, the email/Stripe readiness audit added sanitized SMTP failure codes, admin overview attention items, in-page test/retry controls, and preserved webhook-confirmed bookings when email fails. Customer browser IANA timezones are captured on booking; emails show customer-local times plus Kelowna `America/Vancouver` schedule when they differ, while browser timestamps show the viewer-local timezone.
- Public authentication/booking/quote/form/review/address/delivery-estimate routes now have bounded per-client rate limits; the signed Stripe webhook is intentionally excluded. Checkout expiry now updates the booking, obligation, and stored Checkout Session payment row together.
- Backend production deployment is gated by formatting, warning-free lint, normal tests, disposable PostgreSQL webhook/refund tests and both booking/block concurrency tests. Frontend Pages deployment is gated by formatting, tests, warning-free lint and the WASM target check.
- Real private Storage E2E first passed with an ephemeral legacy service role and was repeated successfully with the configured modern backend-only `sb_secret_...` key: upload, short-lived signed download, deletion, and post-delete denial all passed. The backend rejects publishable keys in the secret setting and never sends a modern secret as a Bearer JWT. No secret value is stored in tracked source, plans, or handoff files.
- A read-only SES audit on 2026-07-15 confirmed that `vlrental.ca` is still `Unverified` in `ca-central-1`, Easy DKIM is `Not configured`, and the account is still in the SES sandbox (200 messages/day, 1 message/second). Three AWS-provided Easy DKIM CNAME records must be published in the authoritative DNS and verified before requesting production access. No DNS or AWS account change was made during the audit.
- Google OAuth now redirects with a five-minute one-time code stored only as a SHA-256 hash. The frontend removes that code from history before exchanging it; access/refresh tokens are returned only in the exchange response and never appear in a URL. A legacy token-bearing callback is sanitized and rejected instead of persisted.
- Facebook OAuth uses the same in-page authentication and five-minute one-time-code exchange. OAuth identities are stored separately by provider/subject so a verified email account can safely use both Google and Facebook without replacing either identity. Facebook requires backend-only `FACEBOOK_APP_ID` and `FACEBOOK_APP_SECRET`; its valid redirect URI is `{PUBLIC_BASE_URL}/api/v1/auth/facebook/callback` and the Meta app must grant the `email` permission.
- Refresh rotation is compare-and-swap: simultaneous reuse of the same refresh token allows exactly one success. Both customer sign-out controls call the backend before clearing browser state, and the backend revokes the matching session even if the access token has already expired.
- Auth tokens, private booking tokens, and pending Checkout client secrets use session storage, with complete one-time migration/removal of older local-storage token sets. Normal JSON bodies are limited to 256 KiB, Stripe webhook bodies to 1 MiB, and authenticated image routes to 11 MiB; image files are checked by MIME and magic bytes.
- The complete Supabase `public` application schema is backend-only: RLS remains enabled; schema `USAGE` and all table/sequence/function privileges are explicitly revoked from `PUBLIC`, `anon`, and `authenticated`; matching default privileges are locked down; and CI runs every SQL safety contract with real Supabase-style roles present.
- HTTP request spans contain only method, path and protocol version, never OAuth query parameters or headers. Sensitive auth/booking/payment/admin/owner/RPC responses use `no-store`, `no-referrer`, anti-sniffing, frame denial, permissions restrictions and HSTS headers; server errors are logged only by sanitized category.
- Admin audit CSV text cells are neutralized against spreadsheet formulas. Managed local media rejects symlinked path components, and the local public media endpoint serves only photos attached to active RVs.
- The final 2026-07-15 pass removed query strings and headers from HTTP spans, sanitized internal logs to stable categories, added `no-store` and browser security headers to private API responses, rate-limited and bounded the Google callback, moved the complete auth token set to session storage, and revoked Supabase `public` schema/function access from Data API roles. The final clean verification passed 71 frontend tests, 113 normal backend/importer tests, 12 disposable-DB/concurrency tests, five PostgreSQL 17 SQL suites, strict clippy and the frontend WASM check. No production mutation or deployment was performed.

## Historical compatibility: Pay everything now / all-in Checkout (2026-07-16)

- Historical behavior only: an older customer with a legacy Stripe damage-deposit obligation could choose `Pay everything now` inside the nested payment dialog. New and migrated uncollected bookings use e-Transfer and cannot enter this path. The remaining bullets document legacy reconciliation compatibility, not the current booking offer.
- The private switch endpoint is `POST /api/v1/bookings/{booking_id}/payment-option/all-in`. It first reserves a durable `all_in` payment bundle and its planned trip/deposit allocations, creates or reuses one idempotent replacement Checkout, binds its identifiers, and only then expires the previous unpaid Session. A Stripe creation or replacement failure keeps the original schedule and Checkout recoverable.
- The all-in choice is reversible while its Checkout remains unpaid. `POST /api/v1/bookings/{booking_id}/payment-option/scheduled` creates an idempotent embedded Checkout for the immutable original `booking.amount_due_now`, expires the unpaid all-in Session, restores the initial obligation to 30%, restores the 70% balance and separate damage-deposit obligations, and cancels the unused all-in bundle. Provider payment/complete state blocks either replacement so switching cannot race a successful charge. Repeated scheduled → all-in → scheduled switches use the concrete replaced Session ID in each idempotency key instead of reusing an expired Checkout.
- `payment_bundles` and `payment_bundle_allocations` store the planned all-in transaction. `payments.bundle_id` links the provider payment, while `payment_allocations` records webhook-confirmed trip/deposit allocation and per-allocation refunds. These backend-only tables use RLS and have no browser/Data API privileges.
- Verified Stripe webhooks are still the only source of payment truth. The all-in webhook must match the bound Session, PaymentIntent metadata, exact CAD total and exact allocation sum before the booking becomes `confirmed / paid`. Duplicate delivery is idempotent. A late success from a replaced Session cannot overwrite the active payment path and creates an administrator-attention state for manual reconciliation.
- All-in cancellation and post-return deposit handling operate against the same provider transaction without double-refunding: cancellation refunds only the trip allocation through cancellation operations, while the refundable-deposit allocation remains under the existing release/retain-damage decision and audit workflow.
- Customer confirmation, account booking data, administrator payment/detail data and notification emails expose the all-in option, one-transaction total and refundable-deposit state. A successful all-in payment suppresses separate balance and deposit-due notifications because those obligations are already allocated as paid.
- The updated customer payment design is in Pencil only: desktop node `tQDxd` and mobile node `xajNB`, integrated into the existing payment frames `PVL9a` and `Tz1rT` with the forest/gold premium treatment. It is not a standalone route or page.
- The reversible selected all-in desktop state is Pencil node `iLaCd`. Its gold action says `Return to 30% payment`, explains the later 70% balance and separate refundable deposit, and remains inside the same payment dialog rather than adding a route.
- Browser session persistence stores the replacement Checkout secret and the same private booking token in `vl_pending_booking_payment`. Refreshing Home or an RV detail page automatically reopens step 5 and the nested all-in payment dialog, checks webhook-backed status, and remounts the same replacement Session; it never creates another booking or Checkout.
- The 2026-07-16 local browser pass exercised a real Stripe test-mode replacement at desktop and 390 px mobile widths. It verified the exact CA$770.14 trip + CA$1,000 deposit = CA$1,770.14 presentation, selected-state copy, nested `Escape`, and full refresh restoration. Database/webhook tests cover duplicate events, mismatched amount/currency, replacement failure preservation, late success, allocation-aware cancellation and deposit separation.

## RV administration and dynamic public details (2026-07-15)

- `/admin` now includes the `RVs` tab between Bookings and Payments. Desktop keeps Overview, Bookings, RVs, Payments, Calendar and Audit; mobile keeps Overview, Bookings, RVs and More.
- RV creation and editing stay in the right drawer. A new RV is archived/draft until explicitly published; permanent deletion is not exposed. Dirty close and nested confirmations share the guarded `Escape` behavior.
- The editor manages structured RV fields, public photos, highlight/amenity rows and per-RV add-ons. Add-ons can be disabled without changing immutable quotes or bookings that already exist.
- Public RV detail pages now load arbitrary active slugs and render API media, structured overview data, features, amenities, descriptions, prices and add-ons. Catalog type filters use `rv_type` instead of parsing customer copy.
- Booking extras use the API-provided Lucide icon and description. When an authentication continuation restores disabled or removed add-ons, the overlay removes them and shows an availability notice before requesting another quote.
- Managed public photos live in Supabase Storage bucket `rental-media` as `rentals/{rental_id}/{uuid}.{ext}`. Local development uses `viktor_rv_back/data/public/rental-media` and the public backend media endpoint. The private `damage-evidence` bucket is unrelated and remains private.
- The backend supplies admin-only CRUD/publish/archive/media/feature/add-on endpoints, writes audit events, creates a Standard rate plan and Kelowna delivery rule with each draft RV, and synchronizes the active rate plan when nightly price changes.
- The schema source of truth and migration `20260715230001_admin_rental_management.sql` add structured fields, managed media metadata, icon/description fields, fixed RV constraints and the `rental-media` Storage policy. Production application and upload of the six existing galleries remain a separately authorized production operation.
- Design source is Pencil only. Current RV administration frames are desktop `FUyYz`, mobile fleet `A7GYL`, and mobile editor `s6aCt`; Figma is not used for Viktor RV.

## Web push notifications (2026-07-21)

- Signed-in customers and administrators can enable or disable notifications inside the existing account panel. Browser permission is requested only from that user action; sign-out unregisters the current browser before the backend session is revoked.
- Firebase project `vl-rental` supplies Web Push/FCM tokens. Amazon SNS application `vl-rental-web` in `ca-central-1` owns the provider endpoints and publishes FCM HTTP v1 payloads. The backend stores only SHA-256 token hashes and SNS endpoint ARNs, never raw FCM tokens or Firebase private keys.
- Email notification rows are the durable source for matching customer/admin push fan-out. A separate retry queue uses bounded attempts and disables stale endpoints without rolling back bookings or payments. Confirmed no-card test bookings also enqueue matching customer/admin push updates directly.
- Production migration `20260721200812_add_fcm_push_notifications.sql` was applied to Supabase project `pwhlkpwlansarstmstge` and verified with RLS enabled, no `PUBLIC`/`anon`/`authenticated` privileges, and all worker indexes present.
- Firebase was connected to SNS and least-privilege backend AWS credentials were stored as repository secrets. No frontend/backend deployment or production service restart was performed; the checked-in production workflow will enable push only when a separately authorized `main` deployment runs.

## Safe regression checklist

Before handing off changes:

1. Confirm the current branch is `dev` and inspect the existing dirty worktree before editing.
2. Do not overwrite unrelated user changes.
3. If the local server is already running, use it read-only. Do not launch another process on its port.
4. Manually walk the critical flow: home date control → dates/guests → RV → address → quote → extras → guest details.
5. Check both home entry and RV-detail entry use `UnifiedBookingOverlay`.
6. Check that background scroll is locked and the overlay itself scrolls on desktop and mobile.
7. Refresh the page and confirm search and address history persistence.
8. Confirm stale/past dates are cleared rather than sent to the API.
9. Run:
   - `cargo check`
   - `cargo test`
   - `git diff --check`
   - inspect every changed text file for broken UTF-8/mojibake before completion
10. Do not commit, push, deploy, restart production, or change DNS unless directly requested.

## Design and maintenance rules

- Keep one canonical booking overlay; do not create another date modal or checkout wizard.
- Reuse the home RV card language and visual hierarchy in all RV selectors.
- Treat backend availability, quote, delivery limit, and booking responses as authoritative.
- Never silently preserve an invalid date, unavailable RV, out-of-range address, or stale quote.
- When an upstream selection changes, clear or refresh every dependent value rather than displaying a plausible but stale total.
- User-facing errors should state what the customer can do next. Do not expose raw request failures when a clearer message is available.
- Preserve the existing `vlrental.ca` site and production environment until explicit launch approval.
