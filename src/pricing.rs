use crate::api;

pub const RV_PREPARATION_FEE: f64 = 97.0;
pub const STATIONARY_PLUS_BASE_NIGHTS: i64 = 3;
pub const STATIONARY_PLUS_BASE_PRICE: f64 = 150.0;
pub const STATIONARY_PLUS_EXTRA_NIGHT_RATE: f64 = 30.0;
pub const DAMAGE_DEPOSIT: f64 = 1_000.0;
pub const BOOKING_DEPOSIT_PERCENT: i32 = 30;
pub const BALANCE_DUE_DAYS: i64 = 30;
pub const DAMAGE_DEPOSIT_DUE_HOURS: i64 = 48;

pub fn money(value: f64) -> String {
    let formatted = format!("{value:.2}");
    let (whole, cents) = formatted.split_once('.').unwrap_or((&formatted, "00"));
    let (sign, digits) = whole
        .strip_prefix('-')
        .map_or(("", whole), |digits| ("-", digits));
    let grouped = digits
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(|chunk| std::str::from_utf8(chunk).unwrap_or_default())
        .collect::<Vec<_>>()
        .join(",");
    format!("CA${sign}{grouped}.{cents}")
}

pub fn night_count_label(nights: i64) -> String {
    let unit = if nights == 1 { "night" } else { "nights" };
    format!("{nights} {unit}")
}

pub fn amount(value: &str) -> f64 {
    value.parse::<f64>().unwrap_or(0.0)
}

pub fn quote_trip_price(quote: &api::QuoteResponse) -> f64 {
    quote
        .items
        .iter()
        .filter(|item| item.item_type != "deposit")
        .map(|item| amount(&item.amount))
        .sum()
}

pub fn mandatory_costs(nights: i64) -> f64 {
    RV_PREPARATION_FEE + stationary_plus_amount(nights)
}

pub fn stationary_plus_amount(nights: i64) -> f64 {
    if nights <= 0 {
        return 0.0;
    }

    STATIONARY_PLUS_BASE_PRICE
        + (nights - STATIONARY_PLUS_BASE_NIGHTS).max(0) as f64 * STATIONARY_PLUS_EXTRA_NIGHT_RATE
}

pub fn stationary_plus_detail(nights: i64) -> String {
    let extra_nights = (nights - STATIONARY_PLUS_BASE_NIGHTS).max(0);
    if extra_nights == 0 {
        return format!(
            "{}-night base · {} fixed",
            STATIONARY_PLUS_BASE_NIGHTS,
            money(STATIONARY_PLUS_BASE_PRICE)
        );
    }

    let night_label = if extra_nights == 1 { "night" } else { "nights" };
    format!(
        "{}-night base {} + {} extra {} × {}",
        STATIONARY_PLUS_BASE_NIGHTS,
        money(STATIONARY_PLUS_BASE_PRICE),
        extra_nights,
        night_label,
        money(STATIONARY_PLUS_EXTRA_NIGHT_RATE)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stationary_plus_uses_a_three_night_base_then_thirty_per_extra_night() {
        assert_eq!(stationary_plus_amount(0), 0.0);
        assert_eq!(stationary_plus_amount(3), 150.0);
        assert_eq!(stationary_plus_amount(4), 180.0);
        assert_eq!(stationary_plus_amount(5), 210.0);
        assert_eq!(stationary_plus_amount(7), 270.0);
        assert_eq!(mandatory_costs(3), 247.0);
        assert_eq!(mandatory_costs(7), 367.0);
    }

    #[test]
    fn cad_money_uses_thousands_separators() {
        assert_eq!(money(3_452.0), "CA$3,452.00");
        assert_eq!(money(1_000.0), "CA$1,000.00");
    }

    #[test]
    fn night_count_uses_singular_only_for_one() {
        assert_eq!(night_count_label(1), "1 night");
        assert_eq!(night_count_label(2), "2 nights");
    }
}
