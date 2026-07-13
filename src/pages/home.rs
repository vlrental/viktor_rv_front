use dioxus::prelude::*;

use crate::components::{Icon, ListingCard};
use crate::data::{rv_listings, IMG_BULLET_VINEYARD, IMG_HERO_RV, IMG_OUTBACK};
use crate::Route;

#[component]
pub fn Home() -> Element {
    rsx! {
        Hero {}
        PopularRvs {}
        HowItWorks {}
        MoreServices {}
        CtaBand {}
    }
}

#[component]
fn Hero() -> Element {
    use_effect(|| {
        document::eval(
            r#"
                const iframe = document.getElementById('hero-youtube-player');
                if (!iframe || iframe.dataset.initialized === 'true') return;
                iframe.dataset.initialized = 'true';

                const createPlayer = () => {
                    const playerIframe = document.getElementById('hero-youtube-player');
                    if (!playerIframe || !window.YT?.Player) return;

                    new window.YT.Player('hero-youtube-player', {
                        events: {
                            onReady: (event) => {
                                event.target.mute();
                                event.target.loadVideoById({
                                    videoId: 'HEgbRefLY_A',
                                    startSeconds: 0
                                });
                            },
                            onStateChange: (event) => {
                                if (event.data === window.YT.PlayerState.PLAYING) {
                                    event.target.getIframe().classList.add('is-playing');
                                }
                                if (event.data === window.YT.PlayerState.ENDED) {
                                    event.target.seekTo(0);
                                    event.target.playVideo();
                                }
                            }
                        }
                    });
                };

                if (window.YT?.Player) {
                    createPlayer();
                } else {
                    const previousReady = window.onYouTubeIframeAPIReady;
                    window.onYouTubeIframeAPIReady = () => {
                        if (typeof previousReady === 'function') previousReady();
                        createPlayer();
                    };

                    if (!document.querySelector('script[src="https://www.youtube.com/iframe_api"]')) {
                        const script = document.createElement('script');
                        script.src = 'https://www.youtube.com/iframe_api';
                        document.head.appendChild(script);
                    }
                }
            "#,
        );
    });

    rsx! {
        section { class: "hero",
            div { class: "hero-media", style: "background-image: url('{IMG_HERO_RV}');",
                iframe {
                    id: "hero-youtube-player",
                    class: "hero-video",
                    src: "https://www.youtube.com/embed/?autoplay=0&controls=0&disablekb=1&enablejsapi=1&iv_load_policy=3&modestbranding=1&mute=1&playsinline=1&rel=0",
                    title: "Kelowna, British Columbia - Drone 4K",
                    allow: "autoplay; encrypted-media; picture-in-picture",
                    tabindex: "-1",
                    "aria-hidden": "true",
                }
            }
            div { class: "hero-overlay" }
            div { class: "hero-copy",
                div { class: "eyebrow-pill",
                    Icon { name: "map-pin", size: 15, color: "var(--vl-accent)" }
                    span { "Kelowna & the Okanagan" }
                }
                h1 { class: "hero-title", "Explore nature on the open road" }
                p { class: "hero-sub",
                    "Book fully-equipped RVs in minutes. Choose your wheels, chase your adventure — we handle the logistics."
                }
            }
            div { class: "searchbar",
                Link { class: "search-field", to: Route::Delivery {},
                    div { class: "search-label", "DELIVERY RADIUS" }
                    div { class: "search-value",
                        Icon { name: "map-pin", size: 17, color: "var(--vl-forest)" }
                        span { "Up to 250 km" }
                    }
                }
                div { class: "search-divider" }
                Link { class: "search-field", to: Route::Catalog {},
                    div { class: "search-label", "WHAT" }
                    div { class: "search-value",
                        Icon { name: "compass", size: 17, color: "var(--vl-forest)" }
                        span { "RVs" }
                    }
                }
                div { class: "search-divider" }
                Link { class: "search-field", to: Route::Catalog {},
                    div { class: "search-label", "DATES" }
                    div { class: "search-value",
                        Icon { name: "calendar", size: 17, color: "var(--vl-forest)" }
                        span { "Add dates" }
                    }
                }
                div { class: "search-divider" }
                Link { class: "search-field", to: Route::Catalog {},
                    div { class: "search-label", "GUESTS" }
                    div { class: "search-value",
                        Icon { name: "users", size: 17, color: "var(--vl-forest)" }
                        span { "2 guests" }
                    }
                }
                Link { class: "search-btn", to: Route::Catalog {},
                    Icon { name: "search", size: 19, color: "var(--vl-white)" }
                    span { "Search" }
                }
            }
        }
    }
}

#[component]
fn PopularRvs() -> Element {
    let rvs = rv_listings();
    rsx! {
        section { class: "section bg-white",
            div { class: "sec-header",
                div {
                    div { class: "eyebrow", "CHOOSE YOUR WHEELS" }
                    h2 { class: "sec-title", "Popular RV rentals" }
                }
                Link { class: "see-all", to: Route::Catalog {},
                    span { "See all 6 RVs" }
                    Icon { name: "arrow-right", size: 17, color: "var(--vl-forest)" }
                }
            }
            div { class: "card-row", style: "margin-bottom: 26px;",
                for rv in rvs.iter().take(4).copied() {
                    ListingCard { key: "{rv.id}", listing: rv }
                }
            }
            FeaturedRv {}
        }
    }
}

#[component]
fn FeaturedRv() -> Element {
    rsx! {
        div { class: "featured-card",
            div { class: "fc-image", style: "background-image: url('{IMG_OUTBACK}');",
                div { class: "lc-badge",
                    Icon { name: "truck", size: 14, color: "var(--vl-forest)" }
                    span { "Delivery available" }
                }
            }
            div { class: "fc-body",
                div { class: "fc-tag",
                    Icon { name: "sparkles", size: 14, color: "var(--vl-forest)" }
                    span { "Complete the fleet" }
                }
                div { class: "fc-title-row",
                    div { class: "fc-title", "Keystone Outback Ultra-Lite" }
                    div { class: "fc-rating",
                        Icon { name: "star", size: 15, color: "var(--vl-accent)" }
                        span { "4.9" }
                    }
                }
                p { class: "fc-desc",
                    "Light, easy-to-tow and sleeps 8 — the flexible all-rounder for family trips around the Okanagan. We can deliver, level and set it up for you."
                }
                div { class: "fc-footer",
                    div { class: "lc-price-row",
                        span { class: "fc-price", "$148" }
                        span { class: "lc-per", "/ night" }
                    }
                    Link { class: "btn-forest", to: Route::RvDetail { slug: "2017-keystone-outback-ultra".to_string() },
                        span { "Reserve" }
                        Icon { name: "arrow-right", size: 16, color: "var(--vl-white)" }
                    }
                }
            }
        }
    }
}

#[component]
fn HowItWorks() -> Element {
    let steps = [
        ("search", "01", "Search & compare", "Browse fully-equipped RVs with transparent, upfront pricing — no hidden fees."),
        ("calendar-check", "02", "Book online", "Reserve in minutes with fair policies and helpful guidance from our local Okanagan team."),
        ("tent-tree", "03", "Hit the road", "We deliver, level and set up your rig. You focus on the fun, we handle the logistics."),
    ];
    rsx! {
        section { class: "section bg-white", style: "padding-top: 74px; padding-bottom: 74px;",
            div { class: "hiw-header",
                div { class: "eyebrow", "HOW IT WORKS" }
                h2 { class: "hiw-title", "Book in minutes, adventure in hours" }
            }
            div { class: "card-row",
                for (icon, num, title, desc) in steps {
                    div { key: "{num}", class: "step-card",
                        div { class: "step-top",
                            div { class: "step-icon",
                                Icon { name: icon, size: 24, color: "var(--vl-white)" }
                            }
                            div { class: "step-num", "{num}" }
                        }
                        div { class: "step-title", "{title}" }
                        p { class: "step-desc", "{desc}" }
                    }
                }
            }
        }
    }
}

#[component]
fn MoreServices() -> Element {
    let services: [(&'static str, &'static str, &'static str, Route); 5] = [
        ("snowflake", "Cooler Trailers", "Keep food & drinks cold on any trip", Route::CoolerTrailers {}),
        ("truck", "Delivery Services", "We drop off, level and set up for you", Route::Delivery {}),
        ("badge-dollar-sign", "RV Sales", "Ready to own? Browse RVs for sale", Route::RvSales {}),
        ("mountain", "Attractions", "Local picks for the best of Kelowna", Route::Attractions {}),
        ("utensils", "Restaurants", "Where to eat around the lake", Route::Restaurants {}),
    ];
    rsx! {
        section { class: "section bg-forest", style: "padding-top: 64px; padding-bottom: 64px;",
            div { class: "sec-header", style: "margin-bottom: 34px;",
                div {
                    div { class: "eyebrow gold", "BEYOND RENTALS" }
                    h2 { class: "sec-title on-dark", style: "font-size: 38px;", "More ways to explore with VL" }
                }
                p { class: "services-sub",
                    "One local team for your whole trip — from gear to guidance around the Okanagan."
                }
            }
            div { class: "service-row",
                for (icon, title, desc, to) in services {
                    Link { key: "{title}", class: "service-cell", to,
                        div { class: "service-icon",
                            Icon { name: icon, size: 22, color: "var(--vl-accent)" }
                        }
                        div { class: "service-title", "{title}" }
                        div { class: "service-desc", "{desc}" }
                    }
                }
            }
        }
    }
}

#[component]
fn CtaBand() -> Element {
    rsx! {
        section { class: "cta-band",
            div { class: "cta-img", style: "background-image: url('{IMG_BULLET_VINEYARD}');" }
            div { class: "cta-overlay" }
            div { class: "cta-copy",
                div { class: "eyebrow gold", "TRAVEL WITH US" }
                h2 { class: "cta-title", "Loved by guests, remembered for the fun" }
                p { class: "cta-sub",
                    "The best memories are made outdoors — tracing scenic Okanagan highways and settling into lakeside campsites. Let's get you out there."
                }
                div { class: "cta-buttons",
                    Link { class: "btn-gold", to: Route::Catalog {},
                        Icon { name: "tent-tree", size: 18, color: "var(--vl-forest-2)" }
                        span { "Book an RV" }
                    }
                }
            }
        }
    }
}
