#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(inline_js = r#"
export function vlBrowserTimezone() {
  return Intl.DateTimeFormat().resolvedOptions().timeZone || 'America/Vancouver';
}
export function vlFormatLocalMoment(value) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat(undefined, {
    year: 'numeric', month: 'short', day: 'numeric',
    hour: 'numeric', minute: '2-digit', timeZoneName: 'short'
  }).format(date);
}
"#)]
extern "C" {
    #[wasm_bindgen(js_name = vlBrowserTimezone)]
    fn browser_timezone_js() -> String;
    #[wasm_bindgen(js_name = vlFormatLocalMoment)]
    fn format_local_moment_js(value: &str) -> String;
}

pub fn browser_timezone() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let value = browser_timezone_js();
        if !value.trim().is_empty() {
            return value;
        }
    }
    "America/Vancouver".into()
}

pub fn format_local_moment(value: &str) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        return format_local_moment_js(value);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        chrono::DateTime::parse_from_rfc3339(value)
            .map(|date| date.format("%b %-d, %Y · %-I:%M %p %:z").to_string())
            .unwrap_or_else(|_| value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_fallback_keeps_an_explicit_offset() {
        let value = format_local_moment("2030-07-15T21:00:00Z");
        assert!(value.contains("2030"));
        assert!(value.contains("+00:00"));
    }
}
