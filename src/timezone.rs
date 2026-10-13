#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(inline_js = r#"
export function vlFormatLocalMoment(value) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat(undefined, {
    year: 'numeric', month: 'short', day: 'numeric',
    hour: 'numeric', minute: '2-digit', timeZone: 'America/Vancouver'
  }).format(date);
}
"#)]
extern "C" {
    #[wasm_bindgen(js_name = vlFormatLocalMoment)]
    fn format_local_moment_js(value: &str) -> String;
}

pub fn format_local_moment(value: &str) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        return format_local_moment_js(value);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        chrono::DateTime::parse_from_rfc3339(value)
            .map(|date| {
                date.with_timezone(&chrono_tz::America::Vancouver)
                    .format("%b %-d, %Y · %-I:%M %p")
                    .to_string()
            })
            .unwrap_or_else(|_| value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Offset, TimeZone, Utc};
    use chrono_tz::America::Vancouver;

    #[test]
    fn native_fallback_uses_business_time_without_a_zone_label() {
        let value = format_local_moment("2030-07-15T21:00:00Z");
        assert!(value.contains("2030"));
        assert!(value.contains("2:00 PM"));
        assert!(!value.contains("PDT"));
        assert!(!value.contains("-07:00"));
    }

    #[test]
    fn vancouver_timezone_keeps_winter_summer_and_dst_offsets() {
        let winter = Vancouver.with_ymd_and_hms(2030, 1, 15, 14, 0, 0).unwrap();
        let summer = Vancouver.with_ymd_and_hms(2030, 7, 15, 14, 0, 0).unwrap();
        let before_spring_transition = Vancouver.with_ymd_and_hms(2030, 3, 10, 1, 59, 0).unwrap();
        let after_spring_transition = Vancouver.with_ymd_and_hms(2030, 3, 10, 3, 0, 0).unwrap();
        let before_autumn_transition = Utc
            .with_ymd_and_hms(2030, 11, 3, 8, 59, 0)
            .unwrap()
            .with_timezone(&Vancouver);
        let after_autumn_transition = Utc
            .with_ymd_and_hms(2030, 11, 3, 9, 0, 0)
            .unwrap()
            .with_timezone(&Vancouver);

        assert_eq!(winter.offset().fix().local_minus_utc(), -8 * 60 * 60);
        assert_eq!(summer.offset().fix().local_minus_utc(), -7 * 60 * 60);
        assert_eq!(
            before_spring_transition.offset().fix().local_minus_utc(),
            -8 * 60 * 60
        );
        assert_eq!(
            after_spring_transition.offset().fix().local_minus_utc(),
            -7 * 60 * 60
        );
        assert_eq!(
            before_autumn_transition.offset().fix().local_minus_utc(),
            -7 * 60 * 60
        );
        assert_eq!(
            after_autumn_transition.offset().fix().local_minus_utc(),
            -8 * 60 * 60
        );
    }
}
