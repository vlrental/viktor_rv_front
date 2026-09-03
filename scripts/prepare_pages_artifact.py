#!/usr/bin/env python3
"""Create crawlable route documents around the Dioxus GitHub Pages client."""

from __future__ import annotations

import html
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path


SITE_NAME = "VL Rental"
PRODUCTION_URL = "https://vlrental.ca"
INDEX_ROBOTS = "index,follow,max-image-preview:large,max-snippet:-1,max-video-preview:-1"
NOINDEX_ROBOTS = "noindex,nofollow"


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
        "RV Rentals Kelowna & Okanagan — Delivered & Set Up | VL Rental",
        "Book fully equipped RV rentals in Kelowna and the Okanagan. We deliver, level and set up your trailer at approved destinations within 150 km.",
        "Delivered RV Rentals in Kelowna & the Okanagan",
        "Choose a clean, fully equipped travel trailer for your family, select your dates and destination, and arrive to an RV that has already been delivered, levelled and set up.",
        kind="WebPage",
    ),
    SeoRoute(
        "/about",
        "About VL Rental | Delivered RV Rentals in Kelowna",
        "Meet VL Rental, a Kelowna RV rental service delivering and setting up fully equipped travel trailers across approved Okanagan destinations.",
        "About VL Rental",
        "We help families enjoy the Okanagan without towing or setting up a trailer. Our team delivers each RV, positions it at the approved site and prepares it for the stay.",
        kind="AboutPage",
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
        "Explore campgrounds and provincial parks within VL Rental's 150 km RV delivery area around Kelowna and the Okanagan.",
        "Campgrounds in Our RV Delivery Range",
        "Plan a delivered RV stay at approved destinations including Bear Creek, Fintry, Ellison, Kekuli Bay, Okanagan Lake and other campgrounds across the Okanagan and Shuswap.",
        kind="CollectionPage",
    ),
    SeoRoute(
        "/rv-sales",
        "RVs for Sale in Kelowna | VL Rental",
        "View RV sales information from VL Rental in Kelowna, British Columbia.",
        "RVs for Sale in Kelowna",
        "See current RV sales information from VL Rental in Kelowna and contact us with questions.",
        kind="CollectionPage",
    ),
    SeoRoute(
        "/terms",
        "RV Rental Terms | VL Rental",
        "Read VL Rental terms for delivered RV bookings, payments, cancellations and customer responsibilities.",
        "RV Rental Terms and Conditions",
        "Review the terms that apply to VL Rental bookings, delivery, payments, cancellations and the care of our RVs.",
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
        "Rent the Jayco 26′ fifth wheel in Kelowna with delivery and setup at approved Okanagan destinations. Sleeps four with a full kitchen.",
        "Jayco 26′ Fifth Wheel Rental",
        "A fully equipped RV for couples and small families, delivered and set up at your approved Okanagan destination.",
        kind="Product",
    ),
    SeoRoute(
        "/rv/2015-keystone-bullet",
        "Keystone Bullet Family RV Rental in Kelowna | VL Rental",
        "Rent the 2015 Keystone Bullet family travel trailer in Kelowna. Sleeps up to ten with delivery and setup in the Okanagan.",
        "2015 Keystone Bullet Family RV Rental",
        "A spacious family travel trailer that sleeps up to ten, delivered and set up at your approved Okanagan destination.",
        kind="Product",
    ),
    SeoRoute(
        "/rv/2014-forest-river-rockwood",
        "Forest River Rockwood RV Rental in Kelowna | VL Rental",
        "Rent the 2014 Forest River Rockwood travel trailer in Kelowna with delivery and setup at approved Okanagan destinations.",
        "2014 Forest River Rockwood RV Rental",
        "A comfortable travel trailer for couples or small families, delivered and set up at your approved destination.",
        kind="Product",
    ),
    SeoRoute(
        "/rv/2025-open-range-1",
        "2025 Open Range Family RV Rental in Kelowna | VL Rental",
        "Rent a 2025 Open Range family travel trailer in Kelowna with delivery and setup across approved Okanagan destinations.",
        "2025 Open Range Family RV Rental",
        "A modern family bunkhouse delivered, levelled and set up at your approved Okanagan campsite.",
        kind="Product",
    ),
    SeoRoute(
        "/rv/2017-keystone-outback-ultra",
        "Keystone Outback Ultra RV Rental in Kelowna | VL Rental",
        "Rent the 2017 Keystone Outback Ultra travel trailer in Kelowna. Sleeps up to eight with Okanagan delivery and setup.",
        "2017 Keystone Outback Ultra RV Rental",
        "A family travel trailer that sleeps up to eight, delivered and set up at your approved Okanagan destination.",
        kind="Product",
    ),
    SeoRoute(
        "/rv/2025-highland-ridge-2",
        "2025 Highland Ridge RV Rental in Kelowna | VL Rental",
        "Rent a 2025 Highland Ridge family travel trailer in Kelowna with delivery and setup at approved Okanagan destinations.",
        "2025 Highland Ridge Family RV Rental",
        "A modern family bunkhouse delivered, levelled and set up at your approved Okanagan campsite.",
        kind="Product",
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


def replace_meta(document: str, selector: str, value: str) -> str:
    pattern = rf'(<meta\s+{selector}\s+content=")[^"]*("\s*/?>)'
    updated, count = re.subn(pattern, rf"\g<1>{html.escape(value, quote=True)}\g<2>", document, count=1)
    if count != 1:
        raise ValueError(f"missing metadata field: {selector}")
    return updated


def absolute_url(site_url: str, path: str) -> str:
    return f"{site_url.rstrip('/')}{path}"


def schema_for(route: SeoRoute, site_url: str) -> str:
    canonical = absolute_url(site_url, route.path)
    graph: list[dict[str, object]] = [
        {
            "@type": "Organization",
            "@id": f"{site_url}/#organization",
            "name": SITE_NAME,
            "url": f"{site_url}/",
            "image": absolute_url(site_url, route.image),
            "logo": f"{site_url}/logo-512.png",
            "telephone": "+1-250-878-5874",
            "areaServed": ["Kelowna", "Okanagan, British Columbia"],
            "priceRange": "$$",
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
        "@type": route.kind,
        "@id": f"{canonical}#page",
        "url": canonical,
        "name": route.title,
        "description": route.description,
        "inLanguage": "en-CA",
        "isPartOf": {"@id": f"{site_url}/#website"},
    }
    if route.kind == "Service":
        page["provider"] = {"@id": f"{site_url}/#organization"}
    elif route.kind == "Product":
        page["brand"] = {"@id": f"{site_url}/#organization"}
    graph.append(page)
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


def render_route(shell: str, route: SeoRoute, site_url: str) -> str:
    canonical = absolute_url(site_url, route.path)
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
    document = re.sub(
        r'<script type="application/ld\+json">.*?</script>',
        f'<script type="application/ld+json">\n{schema_for(route, site_url)}\n        </script>',
        document,
        count=1,
        flags=re.DOTALL,
    )
    snapshot = (
        '<article class="seo-prerender" data-seo-route="'
        + html.escape(route.path, quote=True)
        + '"><h1>'
        + html.escape(route.heading)
        + "</h1><p>"
        + html.escape(route.copy)
        + '</p><p><a href="'
        + html.escape(f"{site_url}/", quote=True)
        + '">Browse delivered RV rentals</a></p></article>'
    )
    document, count = re.subn(r'(<div id="main">)(</div>)', rf"\1{snapshot}\2", document, count=1)
    if count != 1:
        raise ValueError("missing Dioxus mount element")
    return document


def render_private(shell: str, path: str, title: str, site_url: str) -> str:
    route = SeoRoute(path, title, "Private VL Rental page.", title.split(" |", 1)[0], "", robots=NOINDEX_ROBOTS)
    return render_route(shell, route, site_url)


def output_path(root: Path, path: str) -> Path:
    if path == "/":
        return root / "index.html"
    return root / path.strip("/") / "index.html"


def prepare_artifact(root: Path, site_url: str) -> None:
    index = root / "index.html"
    if not index.is_file():
        raise FileNotFoundError(f"missing built shell: {index}")
    shell = index.read_text(encoding="utf-8")

    for route in PUBLIC_ROUTES:
        target = output_path(root, route.path)
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(render_route(shell, route, site_url), encoding="utf-8")

    for path, title in PRIVATE_ROUTES:
        target = output_path(root, path)
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(render_private(shell, path, title, site_url), encoding="utf-8")

    sitemap_urls = "\n".join(
        f"    <url><loc>{html.escape(absolute_url(site_url, route.path))}</loc></url>"
        for route in PUBLIC_ROUTES
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
    if len(sys.argv) not in (2, 3):
        print("usage: prepare_pages_artifact.py <artifact-root> [site-url]", file=sys.stderr)
        return 2
    root = Path(sys.argv[1]).resolve()
    site_url = (sys.argv[2] if len(sys.argv) == 3 else PRODUCTION_URL).rstrip("/")
    prepare_artifact(root, site_url)
    print(f"Prepared {len(PUBLIC_ROUTES)} public and {len(PRIVATE_ROUTES)} private route documents.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
