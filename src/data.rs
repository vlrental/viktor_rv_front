//! Статичные данные-моки этапа «чистый фронт». Позже заменяются на API.

use dioxus::prelude::*;

pub const PHONE: &str = "+1 (250) 878 5874";

pub const IMG_BULLET: Asset = asset!("/assets/img/bullet.webp", AssetOptions::image().with_jpg());
pub const IMG_JAYCO: Asset = asset!("/assets/img/jayco.webp", AssetOptions::image().with_jpg());
pub const IMG_OPENRANGE: Asset = asset!(
    "/assets/img/openrange.webp",
    AssetOptions::image().with_jpg()
);
pub const IMG_OPENRANGE2: Asset = asset!(
    "/assets/img/openrange2.webp",
    AssetOptions::image().with_jpg()
);
pub const IMG_OUTBACK: Asset = asset!("/assets/img/outback.webp", AssetOptions::image().with_jpg());
pub const IMG_ROCKWOOD: Asset = asset!(
    "/assets/img/rockwood.webp",
    AssetOptions::image().with_jpg()
);
pub const IMG_BULLET_LAKESIDE: Asset = asset!(
    "/assets/img/bullet-lakeside.webp",
    AssetOptions::image().with_jpg()
);
pub const IMG_BULLET_MYRA: Asset = asset!(
    "/assets/img/bullet-myra.webp",
    AssetOptions::image().with_jpg()
);
pub const IMG_BULLET_ORCHARD: Asset = asset!(
    "/assets/img/bullet-orchard.webp",
    AssetOptions::image().with_jpg()
);
pub const IMG_BULLET_OVERLOOK: Asset = asset!(
    "/assets/img/bullet-overlook.webp",
    AssetOptions::image().with_jpg().with_preload(true)
);
pub const IMG_BULLET_PEACHLAND: Asset = asset!(
    "/assets/img/bullet-peachland.webp",
    AssetOptions::image().with_jpg()
);
pub const IMG_BULLET_VINEYARD: Asset = asset!(
    "/assets/img/bullet-vineyard.webp",
    AssetOptions::image().with_jpg()
);
macro_rules! gallery_assets {
    ($prefix:literal, $base:expr) => {{
        vec![
            $base,
            asset!(
                concat!("/assets/img/", $prefix, "-lakeside.webp"),
                AssetOptions::image().with_jpg()
            ),
            asset!(
                concat!("/assets/img/", $prefix, "-vineyard.webp"),
                AssetOptions::image().with_jpg()
            ),
            asset!(
                concat!("/assets/img/", $prefix, "-myra.webp"),
                AssetOptions::image().with_jpg()
            ),
            asset!(
                concat!("/assets/img/", $prefix, "-overlook.webp"),
                AssetOptions::image().with_jpg()
            ),
            asset!(
                concat!("/assets/img/", $prefix, "-peachland.webp"),
                AssetOptions::image().with_jpg()
            ),
            asset!(
                concat!("/assets/img/", $prefix, "-orchard.webp"),
                AssetOptions::image().with_jpg()
            ),
        ]
    }};
}

macro_rules! original_assets {
    ($prefix:literal; $($index:literal),+ $(,)?) => {
        vec![
            $(asset!(concat!("/assets/img/", $prefix, "-original-", $index, ".webp"), AssetOptions::image().with_jpg())),+
        ]
    };
}

fn with_originals(mut gallery: Vec<Asset>, originals: Vec<Asset>) -> Vec<Asset> {
    gallery.extend(originals);
    gallery
}

pub fn rv_gallery(slug: &str) -> Vec<Asset> {
    match slug {
        "jayco26" => with_originals(
            gallery_assets!("jayco", IMG_JAYCO),
            original_assets!("jayco"; "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "12", "13"),
        ),
        "2015-keystone-bullet" => with_originals(
            vec![
                IMG_BULLET,
                IMG_BULLET_LAKESIDE,
                IMG_BULLET_VINEYARD,
                IMG_BULLET_MYRA,
                IMG_BULLET_OVERLOOK,
                IMG_BULLET_PEACHLAND,
                IMG_BULLET_ORCHARD,
            ],
            original_assets!("bullet"; "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "12", "13", "14", "15"),
        ),
        "2014-forest-river-rockwood" => with_originals(
            gallery_assets!("rockwood", IMG_ROCKWOOD),
            original_assets!("rockwood"; "01", "02", "03", "04", "05", "06", "07", "08", "09", "10"),
        ),
        "2025-open-range-1" => with_originals(
            gallery_assets!("openrange", IMG_OPENRANGE),
            original_assets!("openrange"; "01", "02", "03", "04", "05", "06", "07", "08", "09"),
        ),
        "2017-keystone-outback-ultra" => with_originals(
            gallery_assets!("outback", IMG_OUTBACK),
            original_assets!("outback"; "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "20"),
        ),
        "2025-highland-ridge-2" => with_originals(
            gallery_assets!("openrange2", IMG_OPENRANGE2),
            original_assets!("openrange2"; "01", "02", "03", "04", "05", "06", "07"),
        ),
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
        Listing {
            id: 1,
            slug: "jayco26",
            title: "Jayco 26' 5th Wheel",
            meta: "5th wheel · Sleeps 4",
            badge: "Pet friendly",
            rating: "4.9",
            price: "$185",
            per: "/ night",
            image: IMG_JAYCO,
        },
        Listing {
            id: 2,
            slug: "2015-keystone-bullet",
            title: "Keystone Bullet 272BHS",
            meta: "Travel trailer · Sleeps 10",
            badge: "Sleeps 10",
            rating: "4.8",
            price: "$155",
            per: "/ night",
            image: IMG_BULLET,
        },
        Listing {
            id: 3,
            slug: "2014-forest-river-rockwood",
            title: "Forest River Rockwood",
            meta: "Travel trailer · Sleeps 4",
            badge: "Great value",
            rating: "4.7",
            price: "$125",
            per: "/ night",
            image: IMG_ROCKWOOD,
        },
        Listing {
            id: 4,
            slug: "2025-open-range-1",
            title: "Open Range 26BHS",
            meta: "Travel trailer · Sleeps 8",
            badge: "New 2025",
            rating: "5.0",
            price: "$160",
            per: "/ night",
            image: IMG_OPENRANGE,
        },
        Listing {
            id: 5,
            slug: "2017-keystone-outback-ultra",
            title: "Keystone Outback Ultra-Lite",
            meta: "Ultra-lite · Sleeps 8",
            badge: "Delivery",
            rating: "4.9",
            price: "$148",
            per: "/ night",
            image: IMG_OUTBACK,
        },
        Listing {
            id: 6,
            slug: "2025-highland-ridge-2",
            title: "Open Range 26BHS — Bunk Beds",
            meta: "Travel trailer · Sleeps 10",
            badge: "Bunk beds",
            rating: "5.0",
            price: "$160",
            per: "/ night",
            image: IMG_OPENRANGE2,
        },
    ]
}

#[cfg(test)]
mod gallery_tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn every_rv_has_at_least_seven_unique_images() {
        let expected_lengths = [20, 22, 17, 16, 27, 14];

        for (listing, expected_length) in rv_listings().into_iter().zip(expected_lengths) {
            let gallery = rv_gallery(listing.slug);
            let unique = gallery
                .iter()
                .map(ToString::to_string)
                .collect::<HashSet<_>>();
            assert_eq!(
                gallery.len(),
                expected_length,
                "{} gallery length",
                listing.slug
            );
            assert_eq!(
                unique.len(),
                gallery.len(),
                "{} gallery uniqueness",
                listing.slug
            );
        }
    }
}
