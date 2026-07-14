# Viktor RV frontend — AI handoff

Last updated: 2026-07-13

This document records the current frontend booking architecture, the problems fixed during the one-page booking work, the verified behavior, and the safe continuation plan for future AI agents.

Кратко: главный результат работы — один общий оверлей бронирования на главной странице и в карточке RV. Он ведёт пользователя по цепочке даты и гости → RV → адрес доставки → допы и точная цена → вход и подтверждение. Ниже зафиксированы устройство, исправления, проверенные сценарии и ограничения, чтобы следующий AI не возвращал старые окна и не ломал зависимое состояние.

## Product direction

- Viktor RV is RV-only. Never add boats or boat bookings.
- The primary booking experience is a single overlay. A customer should not be sent through separate date, catalog, delivery, extras, and checkout screens during the normal flow.
- The customer should be able to choose dates and guests, select an RV, calculate delivery, select extras, see the live total, sign in, and confirm without leaving the overlay.
- Dates-first and RV-first are both supported without separate hidden modes. Step 2 always lists every active RV that fits the guest count. When dates exist, each card is marked either `Available for your dates` or `Choose new dates`. Selecting an available card continues to delivery; selecting a booked card keeps that model, clears the incompatible dates, and opens its live calendar.
- Completed sections collapse into summaries and can be reopened for editing.
- The home page is the primary catalog. `/catalog` is retained only as a compatibility route and redirects to the RV section on the home page.

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
- A stay must be at least three nights.
- Guests are clamped to the supported range.
- Every newly opened unified booking overlay resets the guest count to one. A guest count saved by an earlier catalog search must not silently filter the RV choices before the customer selects guests in the current overlay.
- Changing dates or guests causes the available RV resource to run again. Do not precompute the search outside the reactive resource closure; doing so previously left the RV list permanently stale.
- When an RV is selected first, the calendar loads that model's live unavailable intervals and disables date choices that cannot form a valid minimum stay. The 11:00 AM return and 2:00 PM next-delivery turnover remains selectable.
- After an RV is selected without dates, step 1 shows up to four nearest valid minimum-stay ranges from that model's live availability. Selecting a suggested range fills both dates immediately; customers can still browse later months manually.
- When a customer opens a specific RV without dates, step 1 first asks whether they prefer 3 or 7 nights. It then shows the nearest and later live openings for that exact stay length, with a separate `Choose dates manually` fallback. This guided prompt is not shown when valid dates already exist.
- Every dismissible booking overlay with a visible close control must close through the same guarded path when the user presses `Escape`.

### Step 2 — Choose an RV

- RV choices use image cards based on the home listing design, including name, capacity, summary, and nightly price.
- Step 2 uses two server catalog views: the no-date response supplies every model that fits the guest count, and the dated response marks which of those models are available for the current range.
- Step 2 is always clickable and never becomes an empty dead end merely because all RVs are booked for the selected dates. Without a valid range, selecting any model opens step 1. With dates, available cards continue to delivery, while booked cards remain clickable and open their own calendar after clearing only the incompatible date range.
- RV detail pages pass their slug as the initial selection, but the customer can choose another available RV in the same overlay.
- Each RV card has previous/next gallery controls that appear on pointer hover and remain visible on touch devices. The controls rotate through the project gallery without selecting the RV.
- Cards show the published server rating and review count. `Read comments` opens a nested, independently scrolling review overlay; its close control and `Escape` dismiss only that topmost review layer.
- The `DATES` row in an RV detail booking card is an interactive control. It always opens the unified overlay directly on step 1 so saved dates can be reviewed or changed; the main `Open booking` button continues from the next relevant step.

### Step 3 — Delivery address

- The address step includes an interactive map centred on the Kelowna base with the 150 km delivery area. After a successful calculation it also marks the selected delivery address; map wheel gestures must not scroll the page behind the overlay.
- Suggestions begin after at least three characters and are debounced.
- Suggestions first use `/api/v1/address-suggestions`; the API layer has a Canadian-address fallback.
- The customer may select a suggestion or enter an exact address and press `Calculate delivery`.
- Delivery calculation uses `/api/v1/rentals/{slug}/delivery-estimate`; the API layer has a client fallback if the route is unavailable.
- Policy: maximum 150 km one way from Kelowna; CA$150 through 50 km, then CA$3.50 for each additional one-way kilometre.
- A successful address is stored in `vl_delivery_addresses` in browser local storage. The newest unique address appears first, history is limited to five entries, clicking an entry selects it, and its adjacent remove button deletes it.
- The next steps remain unavailable until delivery is calculated and confirmed within range.

### Step 4 — Extras and trip details

- Extras are visual cards with icon, description, recommendation state, charge type, price, and clear add/remove control.
- Event attendance and movement after delivery use explicit Yes/No segmented controls.
- Selecting or removing an extra updates the quote automatically.

### Step 5 — Guest details and confirmation

- An unauthenticated customer signs in or creates an account inside the overlay.
- After authentication, the form collects name, email, phone, optional notes, and acceptance of rental terms.
- Confirmation uses the current server quote and calls `/api/v1/bookings` with the saved access token.
- Until Stripe is explicitly enabled, the UI must continue to say this is a test booking and that no card is collected or charged.
- On success, the booking response is saved and the app navigates to `/confirmed`.

## Live price behavior

- A local preview can be shown from nightly rate, nights, selected extras, delivery fee, and refundable deposit.
- Every quote automatically includes the mandatory `RV Preparation Fee` of CA$97 once per booking.
- Every quote automatically includes mandatory `Stationary Plus Protection` at CA$50 multiplied by the number of booked calendar nights.
- Both mandatory charges are separate server quote line items and cannot be removed as extras.
- The customer-facing trip price excludes the separate CA$1,000 refundable damage deposit. The deposit remains a quote field and deposit line item for transparency, but it must never be included in the 30% booking-payment calculation.
- The CA$1,000 refundable damage deposit is due separately 48 hours before RV delivery.
- For trips booked more than 30 days ahead, 30% of the trip price is due at booking and the balance is due 30 days before delivery. Trips booked within 30 days require the full trip price at booking.
- The authoritative total is the response from `POST /api/v1/quotes`.
- The exact server quote is requested only when dates, RV, and an in-range calculated delivery address are ready.
- The quote must refresh when the selected RV, dates, guests, delivery address/result, event choices, towing choice, or extras change.
- The summary sidebar shows trip-price line items, the exact CAD trip price, the separate damage deposit, and payment timing when available. It must not imply that a preview is final or that the damage deposit is part of the trip price.

## Browser persistence

- `vl_catalog_search`: applied location, radius, dates, and guests.
- `vl_delivery_addresses`: recent calculated delivery addresses.
- `vl_saved_rvs`: saved/favorite RV slugs.
- `vl_access_token`, `vl_refresh_token`, `vl_auth_user`: authenticated session.
- `vl_trip_draft`: last booking draft used for confirmation/conflict recovery.
- `vl_active_quote`: last authoritative quote.
- `vl_last_booking`: last successfully created booking shown on the confirmation page.

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
- Confirmation email result displayed on `/confirmed`.
- Account page showing the newly created booking.
- Final responsive pass on common mobile Safari and Chrome viewport sizes.

Do not create a test booking, send email, or mutate production merely to run an audit. Obtain the user's approval and use the intended test environment.

## Compatibility surfaces that still exist

- `/checkout` and its component remain for saved legacy drafts and direct compatibility. The normal home and RV-detail flows must not navigate there.
- The old full catalog implementation remains as reusable/dead code inside `catalog.rs`, while the public `/catalog` component redirects home. Do not restore it as a second customer-facing catalog without an explicit product decision.
- Do not delete these compatibility paths casually. First search for saved links, authentication return paths, and conflict-recovery references.

## Admin availability controls

- `/admin` is a server-authorized admin calendar page. The header shows `Close dates` beside `Book now` only when the saved session reports the `admin` role, but the backend always revalidates the live session role before reading or changing blocks.
- An admin selects an RV, a closing date, a reopening date, and an optional internal reason. The backend stores a `source = 'admin'` availability block from delivery/setup at 2:00 PM through return at 11:00 AM in `America/Vancouver`.
- Admin blocks immediately participate in the same catalog, quote, booking, and concurrency checks as imported owner blocks. An attempted block that overlaps an active customer booking returns a conflict and is not created.
- The page lists future blocks created through the admin control and can reopen them. It cannot delete legacy Google blocks, external calendar blocks, or customer bookings.
- Admin access is granted only by changing `app_users.role` to `admin` after the intended account has authenticated. Never infer admin access from a frontend email comparison.

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
