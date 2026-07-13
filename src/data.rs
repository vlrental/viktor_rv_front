//! Статичные данные-моки этапа «чистый фронт». Позже заменяются на API.

use dioxus::prelude::*;

pub const PHONE: &str = "+1 (250) 878 5874";

pub const IMG_BULLET: Asset = asset!("/assets/img/bullet.webp");
pub const IMG_JAYCO: Asset = asset!("/assets/img/jayco.webp");
pub const IMG_OPENRANGE: Asset = asset!("/assets/img/openrange.webp");
pub const IMG_OPENRANGE2: Asset = asset!("/assets/img/openrange2.webp");
pub const IMG_OUTBACK: Asset = asset!("/assets/img/outback.webp");
pub const IMG_ROCKWOOD: Asset = asset!("/assets/img/rockwood.webp");
pub const IMG_BULLET_LAKESIDE: Asset = asset!("/assets/img/bullet-lakeside.webp");
pub const IMG_BULLET_MYRA: Asset = asset!("/assets/img/bullet-myra.webp");
pub const IMG_BULLET_ORCHARD: Asset = asset!("/assets/img/bullet-orchard.webp");
pub const IMG_BULLET_OVERLOOK: Asset = asset!("/assets/img/bullet-overlook.webp");
pub const IMG_BULLET_PEACHLAND: Asset = asset!("/assets/img/bullet-peachland.webp");
pub const IMG_BULLET_VINEYARD: Asset = asset!("/assets/img/bullet-vineyard.webp");
macro_rules! gallery_assets {
    ($prefix:literal, $base:expr) => {{
        vec![
            $base,
            asset!(concat!("/assets/img/", $prefix, "-lakeside.webp")),
            asset!(concat!("/assets/img/", $prefix, "-vineyard.webp")),
            asset!(concat!("/assets/img/", $prefix, "-myra.webp")),
            asset!(concat!("/assets/img/", $prefix, "-overlook.webp")),
            asset!(concat!("/assets/img/", $prefix, "-peachland.webp")),
            asset!(concat!("/assets/img/", $prefix, "-orchard.webp")),
        ]
    }};
}

pub fn rv_gallery(slug: &str) -> Vec<Asset> {
    match slug {
        "jayco26" => gallery_assets!("jayco", IMG_JAYCO),
        "2015-keystone-bullet" => vec![
            IMG_BULLET,
            IMG_BULLET_LAKESIDE,
            IMG_BULLET_VINEYARD,
            IMG_BULLET_MYRA,
            IMG_BULLET_OVERLOOK,
            IMG_BULLET_PEACHLAND,
            IMG_BULLET_ORCHARD,
        ],
        "2014-forest-river-rockwood" => gallery_assets!("rockwood", IMG_ROCKWOOD),
        "2025-open-range-1" => gallery_assets!("openrange", IMG_OPENRANGE),
        "2017-keystone-outback-ultra" => gallery_assets!("outback", IMG_OUTBACK),
        "2025-highland-ridge-2" => gallery_assets!("openrange2", IMG_OPENRANGE2),
        _ => Vec::new(),
    }
}
pub const IMG_HERO_RV: Asset = IMG_BULLET_OVERLOOK;

/// Карточка каталога (RV).
#[derive(Clone, Copy, PartialEq)]
pub struct Listing {
    pub id: u32,
    pub slug: &'static str,
    pub title: &'static str,
    pub meta: &'static str,
    pub badge: &'static str,
    pub rating: &'static str,
    pub price: &'static str,
    pub per: &'static str,
    pub image: Asset,
}

pub fn rv_listings() -> Vec<Listing> {
    vec![
        Listing { id: 1, slug: "jayco26", title: "Jayco 26' 5th Wheel", meta: "5th wheel · Sleeps 4", badge: "Pet friendly", rating: "4.9", price: "$185", per: "/ night", image: IMG_JAYCO },
        Listing { id: 2, slug: "2015-keystone-bullet", title: "Keystone Bullet 272BHS", meta: "Travel trailer · Sleeps 10", badge: "Sleeps 10", rating: "4.8", price: "$155", per: "/ night", image: IMG_BULLET },
        Listing { id: 3, slug: "2014-forest-river-rockwood", title: "Forest River Rockwood", meta: "Travel trailer · Sleeps 4", badge: "Great value", rating: "4.7", price: "$125", per: "/ night", image: IMG_ROCKWOOD },
        Listing { id: 4, slug: "2025-open-range-1", title: "Open Range 26BHS", meta: "Travel trailer · Sleeps 8", badge: "New 2025", rating: "5.0", price: "$160", per: "/ night", image: IMG_OPENRANGE },
        Listing { id: 5, slug: "2017-keystone-outback-ultra", title: "Keystone Outback Ultra-Lite", meta: "Ultra-lite · Sleeps 8", badge: "Delivery", rating: "4.9", price: "$148", per: "/ night", image: IMG_OUTBACK },
        Listing { id: 6, slug: "2025-highland-ridge-2", title: "Open Range 26BHS — Bunk Beds", meta: "Travel trailer · Sleeps 10", badge: "Bunk beds", rating: "5.0", price: "$160", per: "/ night", image: IMG_OPENRANGE2 },
    ]
}

#[cfg(test)]
mod gallery_tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn every_rv_has_seven_unique_images() {
        for listing in rv_listings() {
            let gallery = rv_gallery(listing.slug);
            let unique = gallery.iter().map(ToString::to_string).collect::<HashSet<_>>();
            assert_eq!(gallery.len(), 7, "{} gallery length", listing.slug);
            assert_eq!(unique.len(), 7, "{} gallery uniqueness", listing.slug);
        }
    }
}
