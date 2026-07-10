//! Статичные данные-моки этапа «чистый фронт». Позже заменяются на API.

use dioxus::prelude::*;

pub const PHONE: &str = "+1 (250) 878 5874";

pub const IMG_JAYCO: Asset = asset!("/assets/img/jayco.webp");
pub const IMG_BULLET: Asset = asset!("/assets/img/bullet.webp");
pub const IMG_ROCKWOOD: Asset = asset!("/assets/img/rockwood.webp");
pub const IMG_OPENRANGE: Asset = asset!("/assets/img/openrange.webp");
pub const IMG_OPENRANGE2: Asset = asset!("/assets/img/openrange2.webp");
pub const IMG_OUTBACK: Asset = asset!("/assets/img/outback.webp");
pub const IMG_HERO_RV: Asset = asset!("/assets/img/hero-rv.webp");

/// Карточка каталога (RV).
#[derive(Clone, Copy, PartialEq)]
pub struct Listing {
    pub id: u32,
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
        Listing { id: 1, title: "Jayco 26' 5th Wheel", meta: "5th wheel · Sleeps 4", badge: "Pet friendly", rating: "4.9", price: "$185", per: "/ night", image: IMG_JAYCO },
        Listing { id: 2, title: "Keystone Bullet 272BHS", meta: "Travel trailer · Sleeps 10", badge: "Sleeps 10", rating: "4.8", price: "$155", per: "/ night", image: IMG_BULLET },
        Listing { id: 3, title: "Forest River Rockwood", meta: "Travel trailer · Sleeps 4", badge: "Great value", rating: "4.7", price: "$125", per: "/ night", image: IMG_ROCKWOOD },
        Listing { id: 4, title: "Open Range 26BHS", meta: "Travel trailer · Sleeps 8", badge: "New 2025", rating: "5.0", price: "$160", per: "/ night", image: IMG_OPENRANGE },
        Listing { id: 5, title: "Keystone Outback Ultra-Lite", meta: "Ultra-lite · Sleeps 8", badge: "Delivery", rating: "4.9", price: "$148", per: "/ night", image: IMG_OUTBACK },
        Listing { id: 6, title: "Open Range 26BHS — Bunk Beds", meta: "Travel trailer · Sleeps 10", badge: "Bunk beds", rating: "5.0", price: "$160", per: "/ night", image: IMG_OPENRANGE2 },
    ]
}

pub fn catalog_listings() -> Vec<Listing> {
    rv_listings()
}
