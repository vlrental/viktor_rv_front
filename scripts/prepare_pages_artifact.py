#!/usr/bin/env python3
"""Create crawlable route documents around the Dioxus GitHub Pages client."""

from __future__ import annotations

import html
import hashlib
import json
import re
import shutil
import sys
from dataclasses import dataclass
from pathlib import Path
from urllib.parse import urlparse


SITE_NAME = "VL Rental"
PRODUCTION_URL = "https://vlrental.ca"
INDEX_ROBOTS = "index,follow,max-image-preview:large,max-snippet:-1,max-video-preview:-1"
# Current published API covers, shared with the Rust static fallback. These must
# be refreshed when an administrator changes a rental cover; missing local files
# intentionally disable the route-specific optimization.
RV_COVER_PREVIEWS_PATH = Path(__file__).resolve().parents[1] / "content" / "rv_cover_previews.json"
RV_DETAIL_PRELOADS = {
    f"/rv/{slug}": filename
    for slug, filename in json.loads(RV_COVER_PREVIEWS_PATH.read_text(encoding="utf-8")).items()
}
NOINDEX_ROBOTS = "noindex,nofollow"
LOCAL_STYLESHEET = re.compile(r'<link rel="stylesheet" href="([^"]*/assets/[^"/]+\.css)" type="text/css">')
EXPECTED_STYLES = {
    "about", "checkout", "confirmed", "contact", "delivery", "main",
    "faq", "parks", "rv_detail", "rv_sales", "terms",
}
FAQ_CONTENT_PATH = Path(__file__).resolve().parents[1] / "content" / "faq.json"
TERMS_CONTENT_PATH = Path(__file__).resolve().parents[1] / "content" / "terms.json"
HERO_SMALL_FILE = "hero-bullet-overlook-768.jpg"
HERO_LARGE_FILE = "hero-bullet-overlook.webp"


def load_faq_content() -> dict[str, object]:
    document = json.loads(FAQ_CONTENT_PATH.read_text(encoding="utf-8"))
    categories = document.get("categories", [])
    items = [item for category in categories for item in category.get("items", [])]
    if len(categories) != 5 or len(items) != 50:
        raise ValueError("content/faq.json must contain five categories and 50 questions")
    return document


def load_terms_content() -> dict[str, object]:
    document = json.loads(TERMS_CONTENT_PATH.read_text(encoding="utf-8"))
    sections = document.get("sections", [])
    ids = [section.get("id", "") for section in sections]
    if len(sections) != 20 or len(set(ids)) != 20:
        raise ValueError("content/terms.json must contain 20 uniquely identified sections")
    for section in sections:
        paragraphs = section.get("paragraphs", [])
        bullets = section.get("bullets", [])
        if (
            not section.get("id", "").strip()
            or not section.get("title", "").strip()
            or not paragraphs
            or any(not text.strip() for text in paragraphs)
            or any(not text.strip() for text in bullets)
        ):
            raise ValueError("content/terms.json contains an incomplete section")
    return document


def bundle_route_stylesheets(shell: str, root: Path) -> str:
    """Keep every route's CSS available while reducing ten blocking requests to one."""
    matches = list(LOCAL_STYLESHEET.finditer(shell))
    if not matches:
        # Small unit-test shells do not contain Dioxus-generated asset links.
        return shell

    urls = [match.group(1) for match in matches]
    names = [Path(urlparse(url).path).name.split("-dxh", 1)[0] for url in urls]
    if len(names) != len(EXPECTED_STYLES) or set(names) != EXPECTED_STYLES:
        raise ValueError(f"unexpected Dioxus stylesheet set: {names}")

    styles = []
    for name, url in zip(names, urls):
        relative = urlparse(url).path.split("/assets/", 1)[1]
        source = root / "assets" / relative
        if not source.is_file():
            raise FileNotFoundError(f"missing route stylesheet: {source}")
        contents = source.read_text(encoding="utf-8")
        if "url(" in contents or "@import" in contents:
            raise ValueError(f"route stylesheet needs URL rewriting before bundling: {name}")
        styles.append(f"/* VL route CSS: {name} */\n{contents}")

    combined = "\n".join(styles) + "\n"
    digest = hashlib.sha256(combined.encode("utf-8")).hexdigest()[:16]
    filename = f"main-dxh{digest}.css"
    (root / "assets" / filename).write_text(combined, encoding="utf-8")
    first_url = urls[0]
    base = first_url.split("/assets/", 1)[0]
    replacement = f'<link rel="stylesheet" href="{base}/assets/{filename}" type="text/css">'
    count = 0

    def replace(match: re.Match[str]) -> str:
        nonlocal count
        count += 1
        return replacement if count == 1 else ""

    bundled_shell = LOCAL_STYLESHEET.sub(replace, shell)
    # Dioxus's original main stylesheet and the combined stylesheet share the
    # main-dxh name pattern. The previous Pages assets are copied in before this
    # step, so remove only unreferenced originals and retain both CSS bundles.
    for source in (root / "assets").glob("main-dxh*.css"):
        if source.name != filename and not source.read_text(encoding="utf-8").startswith("/* VL route CSS:"):
            source.unlink()
    return bundled_shell


@dataclass(frozen=True)
class SeoRoute:
    path: str
    title: str
    description: str
    heading: str
    copy: str
    image: str = "/og-image.webp"
    kind: str = "WebPage"
    robots: str = INDEX_ROBOTS


PUBLIC_ROUTES = (
    SeoRoute(
        "/",
        "RV Rental Kelowna — Delivered & Set Up | VL Rental",
        "Find your RV or camper rental in Kelowna. Compare guest ratings and reviews, with delivery and setup at approved destinations within 150 km.",
        "RV Rentals in Kelowna, Delivered & Set Up",
        "Rent an RV in Kelowna without towing. Compare prepared RV and camper rentals and guest reviews, select your dates and approved destination, and arrive to a trailer that has already been delivered, levelled and set up. Read selected Google Reviews from VL Rental guests.",
        kind="WebPage",
    ),
    SeoRoute(
        "/about",
        "About VL Rental | Delivered RV Rentals in Kelowna",
        "Meet VL Rental, a Kelowna RV rental service delivering and setting up prepared travel trailers across approved Okanagan destinations.",
        "About VL Rental",
        "We help families enjoy the Okanagan without towing or setting up a trailer. Our team delivers each RV, positions it at the approved site and prepares it for the stay. Read selected Google Reviews and visit our Google profile for all reviews.",
        kind="AboutPage",
    ),
    SeoRoute(
        "/faq",
        "RV Rental FAQ | Kelowna Delivery, Booking & Setup | VL Rental",
        "Answers about VL Rental RV delivery, campsite setup, booking, payments, paid extras and dry camping in Kelowna and the Okanagan.",
        "RV Rental Questions, Answered",
        "Clear answers about booking a delivered RV in Kelowna and the Okanagan, from campsite details and utility hookups to extras, payments and return-day responsibilities.",
    ),
    SeoRoute(
        "/contact",
        "Contact VL Rental | Kelowna RV Rental Help",
        "Contact VL Rental about RV availability, campground delivery, setup and bookings within 150 km of Kelowna.",
        "Contact VL Rental",
        "Ask about available RVs, delivery destinations, campsite access or an existing booking in Kelowna and the Okanagan.",
        kind="ContactPage",
    ),
    SeoRoute(
        "/delivery",
        "RV Rental Delivery & Setup in Kelowna and Okanagan | VL Rental",
        "RV delivery and setup within 150 km of Kelowna. CA$150 through 40 km, then CA$2.50 per additional kilometre each way.",
        "RV Rental Delivery & Setup in Kelowna and the Okanagan",
        "No truck or trailer hitch is needed. We deliver, position, level and set up every VL Rental RV at an approved campsite or destination. Delivery costs CA$150 through 40 km, then CA$2.50 per additional kilometre in each direction, up to 150 km one way.",
        kind="Service",
    ),
    SeoRoute(
        "/parks-in-our-range",
        "Okanagan Campgrounds in Our RV Delivery Range | VL Rental",
        "Explore Okanagan and Shuswap campgrounds. RV delivery depends on the actual road route, our 150 km limit and campsite approval.",
        "Campgrounds in Our RV Delivery Range",
        "Plan a delivered RV stay at approved destinations including Bear Creek, Fintry, Ellison, Kekuli Bay, Okanagan Lake and other campgrounds across the Okanagan and Shuswap.",
        kind="CollectionPage",
    ),
    SeoRoute(
        "/parks/bear-creek",
        "RV Delivery to Bear Creek Provincial Park | VL Rental",
        "Plan RV delivery to a reserved Bear Creek Provincial Park campsite, subject to site, trailer-fit and Westside Road access approval.",
        "RV Delivery to Bear Creek Provincial Park",
        "Bear Creek is on the west shore of Okanagan Lake via Westside Road. Summer camping and the RV are reserved separately, and delivery requires campsite and trailer-fit approval.",
        image="/assets/img/park-bear-creek.webp",
        kind="Service",
    ),
    SeoRoute(
        "/parks/fintry",
        "RV Delivery to Fintry Provincial Park | VL Rental",
        "Plan delivered RV camping at Fintry via Fintry Delta Road and Westside Road, subject to reserved-site fit, hookups and route approval.",
        "RV Delivery to Fintry Provincial Park",
        "Arrange a suitable Fintry campsite directly with the campground, then choose an available VL Rental trailer for delivery and setup.",
        image="/assets/img/park-fintry.webp",
        kind="Service",
    ),
    SeoRoute(
        "/parks/ellison",
        "RV Delivery to Ellison Provincial Park | VL Rental",
        "Plan delivered RV camping at Ellison, about 16 km from Vernon's Highway 97 junction via 25th Avenue, subject to site and access approval.",
        "RV Delivery to Ellison Provincial Park",
        "Arrange a suitable Ellison campsite directly with the campground, then choose an available VL Rental trailer for delivery and setup.",
        image="/assets/img/park-ellison.webp",
        kind="Service",
    ),
    SeoRoute(
        "/parks/kekuli-bay",
        "RV Delivery to Kekuli Bay Provincial Park | VL Rental",
        "Plan delivered RV camping at Kekuli Bay, 11 km south of Vernon with a 3 km campground approach from Highway 97, subject to site approval.",
        "RV Delivery to Kekuli Bay Provincial Park",
        "Arrange a suitable campsite above Kalamalka Lake directly with the campground, then choose an available VL Rental trailer for delivery and setup.",
        image="/assets/img/park-kekuli-bay.webp",
        kind="Service",
    ),
    SeoRoute(
        "/parks/okanagan-lake",
        "RV Delivery to Okanagan Lake Provincial Park | VL Rental",
        "Plan RV delivery to an approved North or South campground site at Okanagan Lake Provincial Park, 11 km north of Summerland.",
        "RV Delivery to Okanagan Lake Provincial Park",
        "Choose the distinct North or South campground at Okanagan Lake Provincial Park, then share the reserved loop, site, full bumper-to-bumper trailer length and hookups for delivery approval.",
        image="/assets/img/park-okanagan-lake.webp",
        kind="Service",
    ),
    SeoRoute(
        "/parks/okanagan-falls",
        "RV Delivery to sx̌ʷəx̌ʷnitkʷ Park | VL Rental",
        "Plan delivered RV camping at sx̌ʷəx̌ʷnitkʷ Park via Green Lake Road, subject to its narrow approach, reserved-site fit and access approval.",
        "RV Delivery to sx̌ʷəx̌ʷnitkʷ Provincial Park",
        "Arrange a suitable riverside campsite directly with the campground, then plan delivery of an available VL Rental RV.",
        image="/assets/img/park-okanagan-falls.webp",
        kind="Service",
    ),
    SeoRoute(
        "/parks/vaseux-lake",
        "RV Delivery to Vaseux Lake Provincial Park | VL Rental",
        "Plan delivered RV camping at Vaseux Lake beside Highway 97, about 25 km south of Penticton, subject to campsite and route approval.",
        "RV Delivery to Vaseux Lake Provincial Park",
        "Plan a quiet South Okanagan stay with your selected RV delivered directly to an approved campsite.",
        image="/assets/img/park-vaseux-lake.webp",
        kind="Service",
    ),
    SeoRoute(
        "/parks/swiws",
        "RV Delivery to sẁiẁs Park in Osoyoos | VL Rental",
        "Request RV delivery to sẁiẁs, 2 km south of Osoyoos via 32nd Avenue, subject to campsite approval and the 150 km road-route limit.",
        "RV Delivery to sẁiẁs Provincial Park",
        "Explore waterfront camping on Osoyoos Lake, then ask us to verify the road route and site before committing to an RV rental at this outer-range destination.",
        image="/assets/img/park-haynes-point.webp",
        kind="Service",
    ),
    SeoRoute(
        "/parks/shuswap-lake",
        "RV Delivery to Shuswap Lake Provincial Park | VL Rental",
        "Plan RV camping at Shuswap Lake's main arm via the 19 km paved road from Squilax, subject to campsite and 150 km route approval.",
        "RV Delivery to Shuswap Lake Provincial Park",
        "Explore a family campground at Shuswap Lake and ask us to verify the road route and campsite before making RV rental plans.",
        image="/assets/img/park-shuswap-lake.webp",
        kind="Service",
    ),
    SeoRoute(
        "/parks/herald",
        "RV Delivery to Herald Provincial Park | VL Rental",
        "Plan RV camping at Herald, 14 km east of Highway 1 at Tappen on Shuswap Lake's west shore, subject to site and 150 km route approval.",
        "RV Delivery to Herald Provincial Park",
        "Explore a Shuswap Lake campsite near Tappen and request a route and site check before committing to RV delivery.",
        image="/assets/img/park-herald.webp",
        kind="Service",
    ),
    SeoRoute(
        "/rv-sales",
        "RVs for Sale in Kelowna | VL Rental",
        "Browse RVs for sale in Kelowna, British Columbia. View asking prices, vehicle details and photos, and contact VL Rental to arrange a viewing.",
        "RVs for Sale in Kelowna",
        "Explore RVs for sale in Kelowna, British Columbia. Ask about a listing, arrange a viewing, or tell VL Rental what you are looking for.",
        kind="CollectionPage",
    ),
    SeoRoute(
        "/terms",
        "RV Rental Terms | VL Rental",
        "Read VL Rental RV terms for mandatory trip charges, payments, delivery, cancellations and customer responsibilities.",
        "RV Rental Terms and Conditions",
        "Review the terms for VL Rental bookings, delivery, payments and cancellations. Every trip includes a CA$97 RV Preparation Fee and Stationary Plus Protection at a fixed CA$150 per trip, regardless of the number of nights. Applicable taxes are shown in the itemized quote.",
    ),
    SeoRoute(
        "/privacy",
        "Privacy & Cookie Policy | VL Rental",
        "Learn how VL Rental handles personal information, browser storage, cookies and privacy choices in Canada.",
        "Privacy and Cookie Policy",
        "Learn what personal information VL Rental collects, why it is used and the choices available to you.",
    ),
    SeoRoute(
        "/rv/jayco26",
        "Jayco 26′ Fifth Wheel Rental in Kelowna | VL Rental",
        "Rent the Jayco fifth wheel in Kelowna with delivery and setup at approved Okanagan destinations. Sleeps up to four. The 26 ft model designation has a full 30 ft bumper-to-bumper length.",
        "Jayco 26′ Fifth Wheel Rental",
        "A prepared RV that sleeps up to four, delivered and set up at your approved Okanagan destination. The 26 ft model designation has a full 30 ft bumper-to-bumper length, so campsites must fit the full exterior length.",
        kind="Service",
    ),
    SeoRoute(
        "/rv/2015-keystone-bullet",
        "Keystone Bullet Family RV Rental in Kelowna | VL Rental",
        "Rent the 2015 Keystone Bullet family travel trailer in Kelowna. Sleeps up to eight, with a full 30 ft bumper-to-bumper length and Okanagan delivery and setup.",
        "2015 Keystone Bullet Family RV Rental",
        "A spacious family travel trailer that sleeps up to eight, delivered and set up at your approved Okanagan destination. Campsites must fit its full 30 ft bumper-to-bumper length.",
        kind="Service",
    ),
    SeoRoute(
        "/rv/2014-forest-river-rockwood",
        "Forest River Rockwood RV Rental in Kelowna | VL Rental",
        "Rent the 2014 Forest River Rockwood 18 ft model in Kelowna with delivery and setup. Sleeps up to four, with a full 21 ft bumper-to-bumper length for campsite fit.",
        "2014 Forest River Rockwood 18 ft Model RV Rental",
        "A comfortable travel trailer that sleeps up to four, delivered and set up at your approved destination. The 18 ft model/body designation has a full 21 ft bumper-to-bumper length, so campsites must fit the full exterior length.",
        kind="Service",
    ),
    SeoRoute(
        "/rv/2025-open-range-1",
        "2025 Open Range Family RV Rental in Kelowna | VL Rental",
        "Rent a 2025 Open Range 26BHS family travel trailer in Kelowna. It sleeps up to eight and has a full 30 ft bumper-to-bumper length.",
        "2025 Open Range Family RV Rental",
        "A modern family bunkhouse that sleeps up to eight, delivered, levelled and set up at your approved Okanagan campsite. Campsites must fit its full 30 ft bumper-to-bumper length.",
        kind="Service",
    ),
    SeoRoute(
        "/rv/2017-keystone-outback-ultra",
        "Keystone Outback Ultra RV Rental in Kelowna | VL Rental",
        "Rent the 2017 Keystone Outback Ultra travel trailer in Kelowna. Sleeps up to six, with a full 31 ft bumper-to-bumper length and Okanagan delivery and setup.",
        "2017 Keystone Outback Ultra RV Rental",
        "A family travel trailer that sleeps up to six, delivered and set up at your approved Okanagan destination. Campsites must fit its full 31 ft bumper-to-bumper length.",
        kind="Service",
    ),
    SeoRoute(
        "/rv/2025-highland-ridge-2",
        "2025 Highland Ridge RV Rental in Kelowna | VL Rental",
        "Rent a 2025 Highland Ridge Open Range 26BHS in Kelowna. It sleeps up to eight and has a full 30 ft bumper-to-bumper length.",
        "2025 Highland Ridge Family RV Rental",
        "A modern family bunkhouse that sleeps up to eight, delivered, levelled and set up at your approved Okanagan campsite. Campsites must fit its full 30 ft bumper-to-bumper length.",
        kind="Service",
    ),
)

PRIVATE_ROUTES = (
    ("/checkout", "Checkout | VL Rental"),
    ("/confirmed", "Booking Confirmation | VL Rental"),
    ("/login", "Sign In | VL Rental"),
    ("/register", "Create Account | VL Rental"),
    ("/auth/callback", "Signing In | VL Rental"),
    ("/account", "My Account | VL Rental"),
    ("/admin", "Administration | VL Rental"),
)

LEGACY_REDIRECTS = (
    ("/aboutus", "/about"),
    ("/trailertnc", "/terms"),
    ("/attractions", "/parks-in-our-range"),
    ("/restaurants", "/parks-in-our-range"),
    ("/home", "/"),
    ("/portfolio/jayco26", "/rv/jayco26"),
    ("/portfolio/2015KeystoneBullet", "/rv/2015-keystone-bullet"),
    ("/portfolio/2017KeystoneOutbackUltra", "/rv/2017-keystone-outback-ultra"),
    ("/portfolio/2025HighlandRidge2", "/rv/2025-highland-ridge-2"),
    ("/portfolio/2025openrange1", "/rv/2025-open-range-1"),
    ("/rv/brand-new-jayco-26ft-5th-wheel-3", "/rv/jayco26"),
    ("/rv/2015-keystone-bullet-272bhs-bunk-bed", "/rv/2015-keystone-bullet"),
    ("/rv/forest-river-rockwood-19-ft", "/rv/2014-forest-river-rockwood"),
    ("/rv/brand-new-open-range-conventional-26-bhs-bunk-bed-1", "/rv/2025-open-range-1"),
    (
        "/rv/keystone-outback-ultra-lite-family-trailer-with-kids-room",
        "/rv/2017-keystone-outback-ultra",
    ),
    (
        "/rv/brand-new-highland-ridge-open-range-26bhs-bunk-bed-2",
        "/rv/2025-highland-ridge-2",
    ),
)

FEATURED_RVS = (
    "/rv/2025-open-range-1",
    "/rv/jayco26",
    "/rv/2014-forest-river-rockwood",
)
FEATURED_PARKS = (
    "/parks/bear-creek",
    "/parks/fintry",
    "/parks/ellison",
)
PARK_PLANNING_NOTES = {
    "/parks/bear-creek": "Send the site number, full bumper-to-bumper trailer length and hookups shown on your Bear Creek reservation. Confirm the services for your booked site. We will check Westside Road access and RV fit; air conditioning requires compatible 30-amp or 50-amp service.",
    "/parks/fintry": "Fintry Delta Road leaves Westside Road about 34 km north of Kelowna and 49 km south of Vernon. Arrange the campsite separately and send the loop, site, full bumper-to-bumper trailer allowance and booked-site hookups. We confirm access and RV fit; air conditioning requires compatible 30-amp or 50-amp service.",
    "/parks/ellison": "Follow Vernon's 25th Avenue about 16 km from the Highway 97 junction. Arrange the campsite separately and send the site, full bumper-to-bumper trailer allowance and booked-site hookups. We confirm access and RV fit; air conditioning requires compatible 30-amp or 50-amp service.",
    "/parks/kekuli-bay": "The park is 11 km south of Vernon; the campground is 3 km beyond the Highway 97 turn-off, past two park-boundary signs and the campground sign. Arrange the campsite separately and send the site, full bumper-to-bumper allowance and booked-site hookups. Air conditioning requires compatible 30-amp or 50-amp service.",
    "/parks/okanagan-lake": "This guide covers the distinct North and South campgrounds, not every Okanagan Lake campground or Okanagan Mountain Provincial Park. Send North or South, the loop, site number, full bumper-to-bumper trailer length and reserved hookups.",
    "/parks/okanagan-falls": "The Green Lake Road entrance is 500 metres from Highway 97 and has a narrow approach with a blind corner. Arrange the campsite separately and send the site, full bumper-to-bumper allowance and booked-site hookups. We confirm access and fit; air conditioning requires compatible 30-amp or 50-amp service.",
    "/parks/vaseux-lake": "Vaseux Lake is beside Highway 97 about 25 km south of Penticton. Arrange the campsite separately and send the site, full bumper-to-bumper trailer allowance and booked-site hookups. We confirm access and RV fit; air conditioning requires compatible 30-amp or 50-amp service.",
    "/parks/swiws": "The park is 2 km south of Osoyoos via 32nd Avenue. Arrange the campsite separately and send the site, full bumper-to-bumper allowance and booked-site hookups. Air conditioning requires compatible 30-amp or 50-amp service. Delivery is not guaranteed until safe access, RV fit and an actual one-way road route within 150 km are confirmed.",
    "/parks/shuswap-lake": "The main-arm campground is 19 km along paved road from Highway 1 at Squilax; Copper Island is not an RV destination. Arrange the campsite separately and send the loop, site, full bumper-to-bumper allowance and booked-site hookups. Air conditioning requires compatible 30-amp or 50-amp service. Delivery is not guaranteed until safe access, fit and an actual road route within 150 km are confirmed.",
    "/parks/herald": "Herald is on Shuswap Lake's west shore near Salmon Arm, 14 km east of Highway 1 at Tappen. Arrange the campsite separately and send the loop, site, full bumper-to-bumper allowance and booked-site hookups. Air conditioning requires compatible 30-amp or 50-amp service. Delivery is not guaranteed until safe access, fit and an actual road route within 150 km are confirmed.",
}

PARK_SOURCE_URLS = {
    route.path: f"https://bcparks.ca/{slug}-park/"
    for route in PUBLIC_ROUTES
    if route.path.startswith("/parks/")
    for slug in [route.path.removeprefix("/parks/")]
}
PARK_SOURCE_URLS.update(
    {
        "/parks/okanagan-falls": "https://bcparks.ca/sxwexwnitkw-park/",
        "/parks/swiws": "https://bcparks.ca/swiws-park/",
    }
)


def route_link(site_url: str, route: SeoRoute) -> str:
    return (
        f'<a href="{html.escape(page_url(site_url, route.path), quote=True)}">'
        f"{html.escape(route.heading)}</a>"
    )


def load_sale_routes(snapshot_path: Path | None) -> tuple[SeoRoute, ...]:
    if snapshot_path is None:
        return ()
    listings = json.loads(snapshot_path.read_text(encoding="utf-8"))
    if not isinstance(listings, list):
        raise ValueError("RV sales snapshot must be a list")
    routes = []
    seen = set()
    for listing in listings:
        sale_id = str(listing.get("sale_id", ""))
        if not re.fullmatch(r"[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}", sale_id):
            raise ValueError(f"invalid RV sale id in snapshot: {sale_id}")
        if sale_id in seen or listing.get("status") != "published":
            raise ValueError(f"invalid published RV sale snapshot row: {sale_id}")
        title = str(listing.get("title", "")).strip()
        summary = str(listing.get("summary", "")).strip()
        description = str(listing.get("description", "")).strip()
        if not title or not summary or not description:
            raise ValueError(f"RV sale snapshot row needs title, summary and description: {sale_id}")
        photos = listing.get("photos")
        if not isinstance(photos, list):
            raise ValueError(f"RV sale snapshot row needs a photo list: {sale_id}")
        image = "/og-image.webp"
        if photos:
            image = str(photos[0].get("source_url", "")).strip() or image
        location = str(listing.get("location", "Kelowna, BC")).strip() or "Kelowna, BC"
        condition = str(listing.get("condition", "used")).strip().replace("_", " ") or "used"
        rv_type = str(listing.get("rv_type", "RV")).strip().replace("_", " ") or "RV"
        seo_description = " ".join(
            f"{title} for sale in {location}. {summary} View photos and contact VL Rental to arrange a viewing.".split()
        )
        if len(seo_description) > 160:
            seo_description = seo_description[:157].rstrip() + "…"
        routes.append(SeoRoute(
            f"/rv-sales/{sale_id}",
            f"{title} for Sale | VL Rental",
            seo_description,
            title,
            f"{summary} This {condition} {rv_type} is listed in {location}. View the photos and contact VL Rental to ask about the vehicle or arrange a viewing.",
            image=image,
            kind="ItemPage",
        ))
        seen.add(sale_id)
    return tuple(routes)


def render_faq_items(*, featured_only: bool = False) -> str:
    document = load_faq_content()
    categories = document["categories"]
    if featured_only:
        items = sorted(
            (
                item
                for category in categories
                for item in category["items"]
                if item.get("featured_home") is not None
            ),
            key=lambda item: item["featured_home"],
        )
        categories = [{"id": "featured", "title": "Frequently asked questions", "items": items}]

    sections = []
    for category in categories:
        entries = []
        for item in category["items"]:
            bullets = ""
            if item.get("bullets"):
                bullets = "<ul>" + "".join(
                    f"<li>{html.escape(bullet)}</li>" for bullet in item["bullets"]
                ) + "</ul>"
            entries.append(
                f'<details id="faq-{html.escape(item["id"], quote=True)}">'
                f'<summary>{html.escape(item["question"])}</summary>'
                f'<p>{html.escape(item["answer"])}</p>{bullets}</details>'
            )
        sections.append(
            f'<section class="seo-prerender-faq-group" aria-labelledby="faq-{html.escape(category["id"], quote=True)}-title">'
            f'<h2 id="faq-{html.escape(category["id"], quote=True)}-title">{html.escape(category["title"])}</h2>'
            f'{"".join(entries)}</section>'
        )
    return '<div class="seo-prerender-faq">' + "".join(sections) + "</div>"


def render_terms_content() -> str:
    sections = []
    for section in load_terms_content()["sections"]:
        paragraphs = "".join(
            f'<p>{html.escape(paragraph)}</p>' for paragraph in section["paragraphs"]
        )
        bullets = ""
        if section["bullets"]:
            bullets = "<ul>" + "".join(
                f'<li>{html.escape(bullet)}</li>' for bullet in section["bullets"]
            ) + "</ul>"
        sections.append(
            f'<section id="terms-{html.escape(section["id"], quote=True)}">'
            f'<h2>{html.escape(section["title"])}</h2>{paragraphs}{bullets}</section>'
        )
    sections.append(
        '<section id="terms-contact"><h2>21. Contact and notices</h2>'
        '<p>VL Rental · Kelowna, British Columbia, Canada</p>'
        '<p>Email Vlrental.ca@gmail.com · Phone +1 (250) 878-5874</p></section>'
    )
    return '<div class="seo-prerender-terms">' + "".join(sections) + "</div>"


def render_snapshot(
    route: SeoRoute,
    site_url: str,
    sale_routes: tuple[SeoRoute, ...] = (),
    hero_assets: tuple[str, str] | None = None,
    detail_preload: str | None = None,
) -> str:
    by_path = {item.path: item for item in PUBLIC_ROUTES + sale_routes}
    is_park = route.path.startswith("/parks/")
    is_rv = route.path.startswith("/rv/")
    confirmed_details = json.loads(
        (Path(__file__).resolve().parents[1] / "content" / "rv_confirmed_details.json").read_text(
            encoding="utf-8"
        )
    )
    if route.path == "/rv-sales":
        related_paths = tuple(item.path for item in sale_routes) or ("/contact", "/")
        related_title = "Current RVs for sale"
    elif route.path.startswith("/rv-sales/"):
        related_paths = ("/rv-sales", "/contact", "/")
        related_title = "Explore RV sales"
    elif route.path == "/parks-in-our-range":
        related_paths = tuple(item.path for item in PUBLIC_ROUTES if item.path.startswith("/parks/"))
        related_title = "Explore our park guides"
    elif route.path == "/":
        related_paths = FEATURED_RVS + FEATURED_PARKS + ("/delivery", "/parks-in-our-range", "/faq")
        related_title = "Plan your Okanagan RV stay"
    elif is_park:
        related_paths = ("/parks-in-our-range", "/delivery")
        related_title = "Choose an RV for your campsite"
    elif is_rv:
        related_paths = ("/delivery", "/parks-in-our-range") + tuple(
            path for path in FEATURED_RVS if path != route.path
        )
        related_title = "Plan delivery and compare RVs"
    elif route.path == "/delivery":
        related_paths = FEATURED_RVS + ("/parks-in-our-range",)
        related_title = "Explore RVs and destinations"
    else:
        related_paths = ("/", "/delivery", "/parks-in-our-range")
        related_title = "Continue planning your stay"

    related = "".join(
        f"<li>{route_link(site_url, by_path[path])}</li>" for path in related_paths
    )
    note = PARK_PLANNING_NOTES.get(route.path)
    planning = (
        f'<p class="seo-prerender-note"><strong>Before you book:</strong> {html.escape(note)} '
        f'<small>Source: <a href="{html.escape(PARK_SOURCE_URLS[route.path], quote=True)}">BC Parks</a> '
        f'· checked October 13, 2026</small></p>'
        if note else ""
    )
    description = f"<p>{html.escape(route.copy)}</p>" if route.copy else ""
    if route.heading == "This page has moved":
        description = (
            '<p>Continue to the current VL Rental page.</p>'
            f'<p><a href="{html.escape(page_url(site_url, route.path), quote=True)}">'
            'Continue to the current page</a></p>'
        )
        related = ""
    nav = (
        f'<a href="{html.escape(page_url(site_url, "/"), quote=True)}">Home</a>'
        f'<a href="{html.escape(page_url(site_url, "/delivery"), quote=True)}">Delivery &amp; setup</a>'
        f'<a href="{html.escape(page_url(site_url, "/parks-in-our-range"), quote=True)}">Park guides</a>'
        f'<a href="{html.escape(page_url(site_url, "/faq"), quote=True)}">FAQs</a>'
    )
    related_section = (
        f'<section class="seo-prerender-related" aria-labelledby="seo-related-title">'
        f'<h2 id="seo-related-title">{html.escape(related_title)}</h2><ul>{related}</ul></section>'
        if related else ""
    )
    campsite_planning = ""
    if is_rv:
        campsite_links = (
            ("Bear Creek delivery guide", "/parks/bear-creek"),
            ("Okanagan Lake delivery guide", "/parks/okanagan-lake"),
            ("All parks in our delivery range", "/parks-in-our-range"),
        )
        campsite_planning = (
            '<section class="seo-prerender-related" aria-labelledby="seo-campsite-title">'
            '<h2 id="seo-campsite-title">Plan your campsite</h2>'
            '<p>Reserve your campsite separately. We confirm RV fit and delivery access before delivery.</p>'
            '<ul>'
            + "".join(
                f'<li><a href="{html.escape(page_url(site_url, path), quote=True)}">'
                f'{html.escape(label)}</a></li>'
                for label, path in campsite_links
            )
            + "</ul></section>"
        )
        sleep_layout = confirmed_details["sleep_layouts"][route.path.removeprefix("/rv/")]
        electrical_note = confirmed_details["electrical_notes"][route.path.removeprefix("/rv/")]
        campsite_planning = campsite_planning.replace(
            "<ul>",
            f"<p>{html.escape(sleep_layout)}</p><p>{html.escape(electrical_note)}</p><ul>",
            1,
        )
    park_rv_comparison = ""
    if is_park:
        park_rv_comparison = (
            '<section class="seo-prerender-related" aria-labelledby="seo-park-rvs-title">'
            '<h2 id="seo-park-rvs-title">Compare RVs for your campsite</h2>'
            '<p>These are rental options, not a guarantee that they fit this park. '
            'Confirm the site’s permitted full trailer length and delivery route.</p><ul>'
            f'<li><a href="{html.escape(page_url(site_url, "/rv/2014-forest-river-rockwood"), quote=True)}">'
            'Rockwood · 21 ft · sleeps 4</a></li>'
            f'<li><a href="{html.escape(page_url(site_url, "/rv/2015-keystone-bullet"), quote=True)}">'
            'Keystone Bullet · 30 ft · sleeps 8</a></li>'
            f'<li><a href="{html.escape(page_url(site_url, "/"), quote=True)}#home-rentals">'
            'Compare all RV rentals</a></li></ul></section>'
        )
    faq_content = ""
    if route.path == "/faq":
        faq_content = (
            render_faq_items()
            + '<p class="seo-prerender-faq-help">Still need help? '
            + f'<a href="{html.escape(page_url(site_url, "/contact"), quote=True)}">Contact VL Rental</a> '
            + "or review the "
            + f'<a href="{html.escape(page_url(site_url, "/terms"), quote=True)}">rental terms</a>.</p>'
        )
    elif route.path == "/":
        faq_content = render_faq_items(featured_only=True)
    elif route.path == "/terms":
        faq_content = render_terms_content()

    hero_poster = ""
    detail_poster = ""
    if route.path == "/" and hero_assets:
        small, large = hero_assets
        hero_poster = (
            f'<img class="seo-prerender-home-poster" src="{html.escape(large, quote=True)}" '
            f'srcset="{html.escape(small, quote=True)} 768w, {html.escape(large, quote=True)} 1535w" '
            'sizes="100vw" width="1535" height="1024" alt="" aria-hidden="true" '
            'fetchpriority="high" decoding="async">'
        )
    elif detail_preload and (is_rv or is_park):
        detail_poster = (
            f'<img class="seo-prerender-detail-poster" '
            f'src="{html.escape(detail_preload, quote=True)}" '
            f'width="1140" height="456" alt="{html.escape(route.heading, quote=True)} main photo" '
            'fetchpriority="high" decoding="async">'
        )

    return (
        f'<div class="seo-prerender" data-seo-route="{html.escape(route.path, quote=True)}">'
        '<header class="seo-prerender-header">'
        f'<a class="seo-prerender-brand" href="{html.escape(page_url(site_url, "/"), quote=True)}">'
        '<img src="/logo-512.png" width="36" height="36" alt="">VL Rental</a>'
        f'<nav aria-label="Main navigation">{nav}</nav></header>'
        '<main class="seo-prerender-main">'
        f'<section class="seo-prerender-hero">{hero_poster}'
        '<p class="seo-prerender-kicker">KELOWNA · OKANAGAN RV RENTALS</p>'
        f'<h1>{html.escape(route.heading)}</h1>{description}{detail_poster}{planning}'
        '</section>'
        f'{faq_content}{campsite_planning}{park_rv_comparison}{related_section}</main>'
        '<footer class="seo-prerender-footer">'
        'Delivery-only RV rentals · Kelowna, British Columbia · '
        '<a href="tel:+12508785874">+1 (250) 878-5874</a></footer></div>'
    )


def replace_meta(document: str, selector: str, value: str) -> str:
    pattern = rf'(<meta\s+{selector}\s+content=")[^"]*("\s*/?>)'
    updated, count = re.subn(pattern, rf"\g<1>{html.escape(value, quote=True)}\g<2>", document, count=1)
    if count != 1:
        raise ValueError(f"missing metadata field: {selector}")
    return updated


def absolute_url(site_url: str, path: str) -> str:
    if urlparse(path).scheme in {"http", "https"}:
        return path
    return f"{site_url.rstrip('/')}{path}"


def page_url(site_url: str, path: str) -> str:
    return absolute_url(site_url, path if path == "/" else f"{path.rstrip('/')}/")


def schema_for(route: SeoRoute, site_url: str) -> str:
    canonical = page_url(site_url, route.path)
    image = absolute_url(site_url, route.image)
    graph: list[dict[str, object]] = [
        {
            "@type": "Organization",
            "@id": f"{site_url}/#organization",
            "name": SITE_NAME,
            "url": f"{site_url}/",
            "image": f"{site_url}/og-image.webp",
            "logo": f"{site_url}/logo-512.png",
            "telephone": "+1-250-878-5874",
            "areaServed": ["Kelowna", "Okanagan, British Columbia"],
        },
        {
            "@type": "WebSite",
            "@id": f"{site_url}/#website",
            "url": f"{site_url}/",
            "name": SITE_NAME,
            "inLanguage": "en-CA",
            "publisher": {"@id": f"{site_url}/#organization"},
        },
    ]
    page: dict[str, object] = {
        "@type": "FAQPage" if route.path == "/faq" else route.kind,
        "@id": f"{canonical}#page",
        "url": canonical,
        "name": route.title,
        "description": route.description,
        "inLanguage": "en-CA",
        "isPartOf": {"@id": f"{site_url}/#website"},
        "primaryImageOfPage": {"@id": f"{canonical}#primaryimage"},
    }
    if route.kind == "Service":
        page["provider"] = {"@id": f"{site_url}/#organization"}
        page["areaServed"] = ["Kelowna", "Okanagan, British Columbia"]
        if route.path == "/delivery":
            page["serviceType"] = "RV delivery and campsite setup"
        elif route.path.startswith("/rv/"):
            page["serviceType"] = "Delivered RV rental"
        else:
            page["serviceType"] = "RV delivery and campsite setup at the named park"
    if route.path == "/faq":
        page["mainEntity"] = [
            {
                "@type": "Question",
                "name": item["question"],
                "acceptedAnswer": {
                    "@type": "Answer",
                    "text": "\n".join([item["answer"], *item.get("bullets", [])]),
                },
            }
            for category in load_faq_content()["categories"]
            for item in category["items"]
        ]
    graph.append(page)
    graph.append(
        {
            "@type": "ImageObject",
            "@id": f"{canonical}#primaryimage",
            "url": image,
            "contentUrl": image,
        }
    )
    if route.path != "/":
        graph.append(
            {
                "@type": "BreadcrumbList",
                "@id": f"{canonical}#breadcrumb",
                "itemListElement": [
                    {"@type": "ListItem", "position": 1, "name": "Home", "item": f"{site_url}/"},
                    {"@type": "ListItem", "position": 2, "name": route.heading, "item": canonical},
                ],
            }
        )
    return json.dumps({"@context": "https://schema.org", "@graph": graph}, ensure_ascii=False, indent=2)


def render_route(
    shell: str,
    route: SeoRoute,
    site_url: str,
    sale_routes: tuple[SeoRoute, ...] = (),
    hero_assets: tuple[str, str] | None = None,
    detail_preload: str | None = None,
) -> str:
    canonical = page_url(site_url, route.path)
    image = absolute_url(site_url, route.image)
    document = re.sub(r"<title>.*?</title>", f"<title>{html.escape(route.title)}</title>", shell, count=1)
    document = replace_meta(document, 'name="description"', route.description)
    document = replace_meta(document, 'name="robots"', route.robots)
    document = replace_meta(document, 'property="og:title"', route.title)
    document = replace_meta(document, 'property="og:description"', route.description)
    document = replace_meta(document, 'property="og:url"', canonical)
    document = replace_meta(document, 'property="og:image"', image)
    document = replace_meta(document, 'name="twitter:title"', route.title)
    document = replace_meta(document, 'name="twitter:description"', route.description)
    document = replace_meta(document, 'name="twitter:image"', image)
    document = re.sub(
        r'(<link\s+rel="canonical"\s+href=")[^"]*("\s*/?>)',
        rf"\g<1>{html.escape(canonical, quote=True)}\g<2>",
        document,
        count=1,
    )
    schema = f'<script type="application/ld+json">\n{schema_for(route, site_url)}\n        </script>'
    document = re.sub(
        r'<script type="application/ld\+json">.*?</script>',
        lambda _: schema,
        document,
        count=1,
        flags=re.DOTALL,
    )
    if route.path == "/" and hero_assets:
        small, large = hero_assets
        preload = (
            f'<link rel="preload" as="image" href="{html.escape(large, quote=True)}" '
            f'imagesrcset="{html.escape(small, quote=True)} 768w, {html.escape(large, quote=True)} 1535w" '
            'imagesizes="100vw" fetchpriority="high">\n    '
        )
        document = document.replace("</head>", preload + "</head>", 1)
    elif detail_preload:
        preload = (
            f'<link rel="preload" as="image" href="{html.escape(detail_preload, quote=True)}" '
            'fetchpriority="high">\n    '
        )
        document = document.replace("</head>", preload + "</head>", 1)
    snapshot = render_snapshot(route, site_url, sale_routes, hero_assets, detail_preload)
    document, count = re.subn(r'(<div id="main">)(</div>)', rf"\1{snapshot}\2", document, count=1)
    if count != 1:
        raise ValueError("missing Dioxus mount element")
    return document


def render_private(shell: str, path: str, title: str, site_url: str) -> str:
    route = SeoRoute(path, title, "Private VL Rental page.", title.split(" |", 1)[0], "", robots=NOINDEX_ROBOTS)
    return render_route(shell, route, site_url)


def render_redirect(shell: str, old_path: str, target_path: str, site_url: str) -> str:
    target = next(route for route in PUBLIC_ROUTES if route.path == target_path)
    redirect_route = SeoRoute(
        target.path,
        target.title,
        target.description,
        "This page has moved",
        "Continue to the current VL Rental page.",
        robots="noindex,follow",
    )
    document = render_route(shell, redirect_route, site_url)
    target_url = page_url(site_url, target_path)
    document = document.replace(
        "</head>",
        f'        <meta http-equiv="refresh" content="0; url={html.escape(target_url, quote=True)}">\n    </head>',
        1,
    )
    document = document.replace(
        "</body>",
        "        <script>window.location.replace("
        + json.dumps(target_url)
        + ");</script>\n    </body>",
        1,
    )
    return document.replace(
        f'data-seo-route="{target_path}"',
        f'data-seo-route="{old_path}"',
        1,
    )


def render_sale_legacy_redirect(shell: str, site_url: str) -> str:
    document = render_private(shell, "/rv-sales/listing", "RV Sale Listing Moved | VL Rental", site_url)
    sales_url = page_url(site_url, "/rv-sales")
    script = (
        "<script>(function(){var id=new URLSearchParams(window.location.search).get('sale_id')||'';"
        "var valid=/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(id);"
        f"window.location.replace(valid?{json.dumps(site_url + '/rv-sales/')}+id+'/'"
        f":{json.dumps(sales_url)});}})();</script>"
    )
    return document.replace("</body>", script + "</body>", 1)


def output_path(root: Path, path: str) -> Path:
    if path == "/":
        return root / "index.html"
    return root / path.strip("/") / "index.html"


def detail_preload_for(root: Path, route: SeoRoute, base_path: str) -> str | None:
    if preview := RV_DETAIL_PRELOADS.get(route.path):
        path = root / "rv-previews" / preview
        return f"{base_path}/rv-previews/{preview}" if path.is_file() else None
    if route.path.startswith("/parks/") and route.image.startswith("/assets/img/"):
        stem = Path(route.image).stem
        matches = sorted((root / "assets").glob(f"{stem}-dxh*.jpg"))
        if len(matches) != 1:
            return None
        return f"{base_path}/assets/{matches[0].name}"
    return None


def prepare_artifact(root: Path, site_url: str, sales_snapshot: Path | None = None) -> None:
    index = root / "index.html"
    if not index.is_file():
        raise FileNotFoundError(f"missing built shell: {index}")
    shell = bundle_route_stylesheets(index.read_text(encoding="utf-8"), root)

    source_root = Path(__file__).resolve().parents[1]
    base_path = urlparse(site_url).path.rstrip("/")
    hero_assets = (
        f"{base_path}/{HERO_SMALL_FILE}",
        f"{base_path}/{HERO_LARGE_FILE}",
    )
    for hero_file in (HERO_SMALL_FILE, HERO_LARGE_FILE):
        source = source_root / "public" / hero_file
        if not source.is_file():
            raise FileNotFoundError(f"missing hero source: {source}")
        # Like the RV previews below, these files are already compressed for
        # delivery; restore their exact bytes after Dioxus re-encodes them.
        shutil.copyfile(source, root / hero_file)

    # Dioxus fingerprints and converts images used by the app. SEO metadata uses
    # stable source URLs, so publish those files alongside the transformed assets.
    # Dioxus re-encodes WebP assets losslessly, making these already compressed
    # previews much larger. Restore their exact source bytes after the build.
    preview_source = source_root / "public" / "rv-previews"
    preview_target = root / "rv-previews"
    manifest = json.loads((source_root / "scripts" / "rv_preview_sources.json").read_text(encoding="utf-8"))
    expected_previews = {
        f"{Path(entry['url']).stem}-{suffix}.webp"
        for entry in manifest
        for suffix in (("thumb", "preview") if entry["kind"] == "cover" else ("thumb",))
    }
    actual_previews = {source.name for source in preview_source.glob("*.webp")}
    if actual_previews != expected_previews:
        raise ValueError("RV preview files do not match the source manifest")
    preview_target.mkdir(parents=True, exist_ok=True)
    for name in expected_previews:
        shutil.copyfile(preview_source / name, preview_target / name)

    for image in {route.image for route in PUBLIC_ROUTES if route.image.startswith("/assets/")}:
        relative = image.lstrip("/")
        source = source_root / relative
        if not source.is_file():
            raise FileNotFoundError(f"missing SEO image: {source}")
        target = root / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, target)

    sale_routes = load_sale_routes(sales_snapshot)
    public_routes = PUBLIC_ROUTES + sale_routes
    detail_preloads = {
        route.path: preload
        for route in public_routes
        if (preload := detail_preload_for(root, route, base_path)) is not None
    }

    sales_root = root / "rv-sales"
    if sales_root.is_dir():
        for child in sales_root.iterdir():
            if child.is_dir() and re.fullmatch(
                r"[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}",
                child.name,
            ):
                shutil.rmtree(child)

    for route in public_routes:
        target = output_path(root, route.path)
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(
            render_route(
                shell,
                route,
                site_url,
                sale_routes,
                hero_assets,
                detail_preloads.get(route.path),
            ),
            encoding="utf-8",
        )

    for path, title in PRIVATE_ROUTES:
        target = output_path(root, path)
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(render_private(shell, path, title, site_url), encoding="utf-8")

    for old_path, target_path in LEGACY_REDIRECTS:
        target = output_path(root, old_path)
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(
            render_redirect(shell, old_path, target_path, site_url),
            encoding="utf-8",
        )

    legacy_sale = output_path(root, "/rv-sales/listing")
    legacy_sale.parent.mkdir(parents=True, exist_ok=True)
    legacy_sale.write_text(render_sale_legacy_redirect(shell, site_url), encoding="utf-8")

    sitemap_urls = "\n".join(
        f"    <url><loc>{html.escape(page_url(site_url, route.path))}</loc></url>"
        for route in public_routes
    )
    (root / "sitemap.xml").write_text(
        '<?xml version="1.0" encoding="UTF-8"?>\n'
        '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n'
        f"{sitemap_urls}\n"
        "</urlset>\n",
        encoding="utf-8",
    )

    not_found = SeoRoute(
        "/404",
        "Page Not Found | VL Rental",
        "The requested VL Rental page could not be found.",
        "Page not found",
        "Return to VL Rental to browse available RVs.",
        robots=NOINDEX_ROBOTS,
    )
    (root / "404.html").write_text(render_route(shell, not_found, site_url), encoding="utf-8")


def main() -> int:
    if len(sys.argv) not in (2, 3, 4):
        print("usage: prepare_pages_artifact.py <artifact-root> [site-url] [sales-json]", file=sys.stderr)
        return 2
    root = Path(sys.argv[1]).resolve()
    site_url = (sys.argv[2] if len(sys.argv) >= 3 else PRODUCTION_URL).rstrip("/")
    sales_snapshot = Path(sys.argv[3]).resolve() if len(sys.argv) == 4 else None
    prepare_artifact(root, site_url, sales_snapshot)
    print(
        f"Prepared {len(PUBLIC_ROUTES) + len(load_sale_routes(sales_snapshot))} public, {len(PRIVATE_ROUTES)} private "
        f"and {len(LEGACY_REDIRECTS)} legacy route documents."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
