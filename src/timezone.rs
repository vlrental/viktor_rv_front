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

    #[test]
    fn native_fallback_uses_business_time_without_a_zone_label() {
        let value = format_local_moment("2030-07-15T21:00:00Z");
        assert!(value.contains("2030"));
        assert!(value.contains("2:00 PM"));
        assert!(!value.contains("PDT"));
        assert!(!value.contains("-07:00"));
    }
}
