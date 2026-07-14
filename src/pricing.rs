use crate::api;

pub const RV_PREPARATION_FEE: f64 = 97.0;
pub const STATIONARY_PLUS_NIGHTLY_RATE: f64 = 50.0;
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
    RV_PREPARATION_FEE + STATIONARY_PLUS_NIGHTLY_RATE * nights.max(0) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mandatory_costs_follow_calendar_nights() {
        assert_eq!(mandatory_costs(3), 247.0);
        assert_eq!(mandatory_costs(7), 447.0);
    }

    #[test]
    fn cad_money_uses_thousands_separators() {
        assert_eq!(money(3_452.0), "CA$3,452.00");
        assert_eq!(money(1_000.0), "CA$1,000.00");
    }
}
