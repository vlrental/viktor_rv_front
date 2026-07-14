# AGENTS.md - Viktor RV Frontend

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

## Project design

- The canonical project design is available through the connected design tool. Open and inspect it directly when design context is needed; do not ask the user to provide it again unless access fails.
- Design Node IDs: `IUHnT`, `x8t0A0`, `I6W2Es`, `raX6S`, `rmfa4`, `ODW3r`, `f1GuCf`, `ns0xG`, `LaBip`, `iKgTN`, `Oijpd`, `MEsd0`, `XgqBg`, `w19Mf`, `jr5XP`, `CdnCR`, `X9ejnB`, `g9upP`, `qaZRF`, `yTWGi`, `Al6fI`, `lsQAl`, `eb6Ck`, `cOu0u`, `YuNUS`, `TDhXo`, `M4DJcJ`, `e8z6o4`, `K7A9o`.
- Use these nodes as the visual source of truth when implementing or reviewing the corresponding frontend UI.

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
- RV delivery is limited to 150 km one way from the Kelowna base. The fee is CA$150 through 50 km, then CA$3.50 for each additional one-way kilometre (CA$1.75/km in each direction).
- RV rentals require at least three nights. Backend code is the source of truth for converting selected dates into timestamps and enforcing availability.
- Every RV quote includes a mandatory `RV Preparation Fee` of CA$97 once per booking.
- Every RV quote includes mandatory `Stationary Plus Protection` at CA$50 for each booked night, automatically calculated from the calendar date difference. Both mandatory charges must appear as separate quote line items, and backend quote code is the pricing source of truth.
- At initial booking, if RV delivery is more than 30 days away, the customer must pay 30% of the trip price immediately. Calculate the 30% only from the trip price; the separate CA$1,000 damage deposit is not part of this percentage. If delivery is 30 days away or less, the customer must pay 100% of the trip price immediately.
- For a booking made more than 30 days before delivery, the remaining trip-price balance becomes due exactly 30 days before the delivery date, bringing the booking payment to 100%.
- Every booking requires a CA$1,000 damage deposit, due 48 hours before RV delivery.
- Whenever a scheduled booking payment or damage-deposit payment becomes due, notify the customer immediately by email and include a direct payment link.
- For the Gold option, hold the CA$1,000 damage deposit for seven days after the RV is returned, then refund it without interest, subject to any valid damage charges.
- Until Stripe is explicitly enabled, bookings are test bookings stored as `confirmed` / `test_paid`; no card is collected and no real payment row is created.
