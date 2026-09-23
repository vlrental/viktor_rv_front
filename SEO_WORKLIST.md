# VL Rental SEO worklist

Updated: 2026-09-23. This is a working checklist, not a claim that Google has indexed or ranked every URL.

## 1. Indexing and technical diagnosis — in progress

- Baseline from the last available Search Console report: 7 of 24 submitted sitemap URLs indexed; report data lagged the September 16 sitemap resubmission and URL inspection requests.
- Confirmed in the signed-in Search Console on September 16: the all-known-pages report still says "Last update: 9/13/26", with 12 indexed and 41 not indexed. This is an older reporting snapshot, not a fresh crawl result.
- The submitted `/sitemap.xml` showed **Success**, 24 discovered pages and a September 16 last-read date. It does not guarantee that all 24 pages are indexed.
- Fresh URL Inspection on September 16: the homepage and canonical `/delivery` are indexed. Open Range (`/rv/2025-open-range-1/`), Jayco (`/rv/jayco26/`), the park hub (`/parks-in-our-range/`) and Bear Creek (`/parks/bear-creek/`) were not yet indexed, but their live tests said they were available to Google and indexable. The tested Bear Creek HTML had its own title, description and canonical URL.
- Google confirmed indexing requests for the homepage, Open Range, park hub and Bear Creek. A request for Jayco on September 16 was **not accepted** because Search Console reported its daily quota exceeded. On September 17, URL Inspection showed Jayco was already indexed, so no repeat request was needed.
- On September 17, individual URL Inspection also showed Open Range, the park hub and Bear Creek as **indexed**, each with HTTPS and one valid breadcrumb item. No repeat indexing requests were made. The aggregate Pages report still showed the older 12/41 snapshot, so do not infer a new total from it.
- After the WebP release on September 17, the aggregate Pages report still said **Last update 9/13/26**, 12 indexed and 41 not indexed. Individual inspection now showed `/rv/2017-keystone-outback-ultra/` and `/parks/herald/` **indexed**, despite their presence in the older excluded-page examples. `/rv/2025-highland-ridge-2/` was still reported as discovered/not indexed, but a fresh live test said it was available to Google and indexable with one valid breadcrumb item. Do not use the stale aggregate counts as the current status of individual URLs.
- `/delivery/` was reported as a page with redirect, while its destination `/delivery` is indexed. Treat the slash variant as an intentional redirect, not a separate page requiring indexing.
- September 23 Search Console update: all 24 submitted sitemap URLs are indexed. The broader all-known-pages report contains 28 indexed and 37 excluded URLs because it also includes historical and legacy URLs outside the sitemap.
- Seven slashless RV and park URL variants remained under "Redirect error", and Google's validation attempt failed on September 21. Fresh URL Inspection for the slashless Bear Creek URL passed its live test on September 23 and reported the page available to Google with one valid breadcrumb; its aggregate error is therefore historical. The RV router still removed the canonical trailing slash after hydration, so the public router and checked-in sitemap are being aligned with the canonical trailing-slash URLs before another validation request.
- Triage from that snapshot: prioritize the 10 discovered-not-indexed and 4 crawled-not-indexed URLs that are current RV/park pages. Review the 4 redirect errors after Google's validation refresh. The 13 404 URLs, 5 noindex URLs, 4 pages with redirect and 1 alternate canonical require URL-by-URL classification; legacy or intentional exclusions must not be treated as blanket defects.
- Previously inspected live examples for Jayco, Open Range, Rockwood, Herald and Kekuli Bay were indexable; indexing itself remains Google's decision.
- Recheck Search Console on or after September 19, 2026: submitted sitemap count, indexed count, and affected URL examples for 404, redirect error, discovered-not-indexed and crawled-not-indexed. Distinguish intentional redirects/noindex from errors.
- For each important RV and park page still excluded, record Google's chosen canonical, last crawl, referring page, rendered HTML and HTTP result before changing code.
- Keep legacy boat-related pages out of the new site; do not restore those URLs as product content.

## 2. Initial HTML and internal links — open

- Commit `a15e3d7` on `dev` replaces the hidden summary with visible, route-specific initial HTML and relevant internal links for homepage, RVs, delivery and all ten park guides. The initial view follows the inspected desktop/mobile Pencil frames; a mount observer removes it when Dioxus renders, while preserving it if JavaScript fails.
- The fresh release artifact and verifier passed for all 43 generated route documents. The verifier now checks visible route-specific content, one initial `h1`, relevant links and referenced assets.
- Published to production on September 16 via commit `a15e3d7`; the Pages workflow passed and live HTML on the homepage, About page and Open Range page shows the new route-specific snapshots. Still open: browser-level visual/handoff check after hydration and Search Console follow-up. Do not equate successful publication with Google indexing.

## 3. Distinctive local content — open

- Existing ten park guides contain individual location and access notes. Add checked, practical details such as compatible RV lengths, campsite information required for approval, route limitations, and links to relevant RVs. Avoid unverified campground claims or mass-produced near-duplicates.
- Strengthen RV detail pages with verified layouts, capacities, equipment, photos and booking expectations rather than generic copy.
- September 23: the owner-approved English FAQ is implemented as 50 crawlable answers in five groups on `/faq/`, with six matching questions on the homepage. Terms, metadata, the sitemap and prerendered search content use the same confirmed policies; unverified model capacities and placeholder prices were not published. A per-selected-bed bedding quantity control remains a separate booking-flow follow-up.

## 4. Accuracy and local trust — open

- About-page claims such as "no service fees" and "no fees" conflicted with the mandatory RV Preparation Fee and Stationary Plus Protection. The September 17 live `/about` hydration check confirmed the corrected direct-booking/full-quote copy and required-charges wording are published. Its remaining "no hidden fees" wording describes upfront disclosure, not an absence of mandatory charges.
- Keep the Google Business Profile accurate for delivery-only RV rentals, service area, hours, services and photos. Ask real customers for honest reviews without incentives; respond to reviews.
- Seek relevant local campground/tourism mentions or partnerships; do not purchase links or create duplicate business profiles.

## 5. Measurement — open

- Use Search Console monthly for non-brand queries, target URLs, clicks, impressions and indexing; avoid reading too much into the small initial sample.
- September 17 baseline: the 3-month Web report (data through September 15) showed 21 clicks, 881 impressions, 2.4% CTR and average position 28.2. The largest page-level impression sources were `/` (554) and legacy `/restaurants` (312); relevant Kelowna RV query variants each had roughly 28–32 impressions and no clicks. This period predates the September 16 SEO release, so compare against later data rather than judging the new pages by this baseline.
- Define enquiry, telephone and booking conversion events before choosing any analytics tool. Review consent and the privacy policy before introducing cookies or tracking.
- Proposed measurement events, not yet implemented: `contact_submit_success` (successful inquiry), `phone_click` (tap/click on a `tel:` link), and `booking_confirmed` (backend-verified booking only, never a browser-only payment return). Count events without names, emails, phone numbers or booking IDs in analytics payloads.

## 6. Mobile experience — open

- Obtain actual mobile field/lab measurements for homepage, an RV detail page and a park guide. Review LCP, INP and CLS, then optimize measured bottlenecks rather than assuming a speed problem.
- September 17 PageSpeed Insights mobile lab baseline for `/`: Performance 55, SEO 100, FCP 11.5 s, LCP 13.6 s, TBT 110 ms, CLS 0; no field data yet. The largest image opportunity is one Supabase-hosted PNG used by a catalog card (2,731 KiB with about 2,475 KiB estimated savings). Other flagged items: render-blocking requests (estimated 3,940 ms), total network payload 6,240 KiB, and image delivery (2,715 KiB estimated savings). Diagnose and resize/convert the specific image before changing layout or claiming an improvement; a single lab run is a baseline, not a field-vitals verdict.
- Mobile lab baseline for `/rv/2025-open-range-1/`: Performance 57, SEO 100, FCP 4.7 s, LCP 68.5 s, TBT 30 ms, CLS 0, 21,880 KiB network payload; no field data. Six gallery thumbnails downloaded original Supabase JPGs of roughly 2.5–4.2 MiB each while displaying at roughly 252 px width. Prioritize appropriately sized/compressed thumbnail delivery while preserving full-size photos when opened. The correct Supabase project's organization was checked on September 17 and is on the **Free** plan; Supabase's on-the-fly transformations require Pro or above, so use generated thumbnails without a plan upgrade.
- Mobile lab baseline for `/parks/bear-creek/`: Performance 60, SEO 100, FCP 4.7 s, LCP 11.9 s, TBT 0 ms, CLS 0; no field data. The report flagged render-blocking requests (estimated 3,600 ms) and a smaller image-delivery opportunity (251 KiB). Across the three sampled routes, defer performance claims until fixes are deployed and rerun under comparable conditions.
- After the WebP production release, a fresh September 17 mobile PageSpeed lab run for `/rv/2025-open-range-1/` scored Performance 59, SEO 100, FCP 4.7 s, LCP 12.4 s, TBT 0 ms and CLS 0; the earlier single run had Performance 57 and LCP 68.5 s. [New RV report](https://pagespeed.web.dev/analysis/https-vlrental-ca-rv-2025-open-range-1/vwmri3hdqz?form_factor=mobile). This is a strong directional LCP improvement, not a field-data guarantee.
- The fresh homepage mobile run scored Performance 60, SEO 100, FCP 4.7 s, LCP 13.6 s, TBT 130 ms, CLS 0 and total payload 3,488 KiB; the prior single run had Performance 55, FCP 11.5 s and total payload 6,240 KiB. [New homepage report](https://pagespeed.web.dev/analysis/https-vlrental-ca/8ng5bi28iu?form_factor=mobile). The remaining largest lab opportunity is render-blocking CSS (estimated 3,700 ms), including route styles loaded on the homepage, the external Lucide font stylesheet, and Google Fonts; image-delivery savings are now estimated at 178 KiB. Diagnose the CSS loading path before changing it, and remeasure after any change.

## Completion rule

- September 23 redirect follow-up: production commit `a84b0a2` passed Pages run `35917330661`. All seven affected slashless URLs returned one redirect and HTTP 200 at their slash targets. Browser checks confirmed Rockwood and Open Range retain the slash after app startup. Google's fresh Rockwood live test at 13:44 reported available/indexable with one valid breadcrumb. Restarted the Redirect error validation; Search Console confirmed **Validation Started 9/23/26**, seven pending, zero failed. This is pending Google validation, not a completed clearance of the error report.

Mark a work item complete only after the change is verified on the deployed site (when deployment is authorized) and the relevant Search Console or user-facing result has been rechecked. Ranking positions and AI mentions cannot be guaranteed.
