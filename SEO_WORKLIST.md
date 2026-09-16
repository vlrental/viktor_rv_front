# VL Rental SEO worklist

Updated: 2026-09-16. This is a working checklist, not a claim that Google has indexed or ranked a URL.

## 1. Indexing and technical diagnosis — in progress

- Baseline from the last available Search Console report: 7 of 24 submitted sitemap URLs indexed; report data lagged the September 16 sitemap resubmission and URL inspection requests.
- Confirmed in the signed-in Search Console on September 16: the all-known-pages report still says "Last update: 9/13/26", with 12 indexed and 41 not indexed. This is an older reporting snapshot, not a fresh crawl result.
- Triage from that snapshot: prioritize the 10 discovered-not-indexed and 4 crawled-not-indexed URLs that are current RV/park pages. Review the 4 redirect errors after Google's validation refresh. The 13 404 URLs, 5 noindex URLs, 4 pages with redirect and 1 alternate canonical require URL-by-URL classification; legacy or intentional exclusions must not be treated as blanket defects.
- Previously inspected live examples for Jayco, Open Range, Rockwood, Herald and Kekuli Bay were indexable; indexing itself remains Google's decision.
- Recheck Search Console on or after September 19, 2026: submitted sitemap count, indexed count, and affected URL examples for 404, redirect error, discovered-not-indexed and crawled-not-indexed. Distinguish intentional redirects/noindex from errors.
- For each important RV and park page still excluded, record Google's chosen canonical, last crawl, referring page, rendered HTML and HTTP result before changing code.
- Keep legacy boat-related pages out of the new site; do not restore those URLs as product content.

## 2. Initial HTML and internal links — open

- Build output gives each public route a title, description, canonical, robots and structured data, but the initial body currently contains only a short hidden route summary and a homepage link (`scripts/prepare_pages_artifact.py`).
- Plan visible, useful, route-specific HTML and crawlable links for homepage, RVs, delivery and park guides; ensure it matches the client app and does not flash or duplicate headings after hydration.
- The project's Pencil design must be inspected and the result verified there before changing the visible UI. Do not replace this with hidden keyword text.
- On September 16, Pencil was not available through the connected design tools or local app, so the visible prerender/UI change remains open. Do not mark this item complete based on metadata-only changes.
- Extend artifact verification to check route-specific text and relevant internal links, not only files and metadata.

## 3. Distinctive local content — open

- Existing ten park guides contain individual location and access notes. Add checked, practical details such as compatible RV lengths, campsite information required for approval, route limitations, and links to relevant RVs. Avoid unverified campground claims or mass-produced near-duplicates.
- Strengthen RV detail pages with verified layouts, capacities, equipment, photos and booking expectations rather than generic copy.
- The customer will supply and approve the FAQ later; do not publish answers before that review.

## 4. Accuracy and local trust — open

- About-page claims such as "no service fees" and "no fees" conflicted with the mandatory RV Preparation Fee and Stationary Plus Protection. Copy was corrected locally on September 16 to describe direct booking and a full quote; verify its release before marking this sub-item complete.
- Keep the Google Business Profile accurate for delivery-only RV rentals, service area, hours, services and photos. Ask real customers for honest reviews without incentives; respond to reviews.
- Seek relevant local campground/tourism mentions or partnerships; do not purchase links or create duplicate business profiles.

## 5. Measurement — open

- Use Search Console monthly for non-brand queries, target URLs, clicks, impressions and indexing; avoid reading too much into the small initial sample.
- Define enquiry, telephone and booking conversion events before choosing any analytics tool. Review consent and the privacy policy before introducing cookies or tracking.

## 6. Mobile experience — open

- Obtain actual mobile field/lab measurements for homepage, an RV detail page and a park guide. Review LCP, INP and CLS, then optimize measured bottlenecks rather than assuming a speed problem.

## Completion rule

Mark a work item complete only after the change is verified on the deployed site (when deployment is authorized) and the relevant Search Console or user-facing result has been rechecked. Ranking positions and AI mentions cannot be guaranteed.
