use chrono::{DateTime, Duration, NaiveDate, Utc};
use chrono_tz::America::Vancouver;
use dioxus::prelude::*;

use crate::{api, components::Icon, Route};

fn display_date(timestamp: &str) -> String {
    DateTime::parse_from_rfc3339(timestamp)
        .map(|value| {
            value
                .with_timezone(&Vancouver)
                .format("%b %-d, %Y")
                .to_string()
        })
        .unwrap_or_else(|_| timestamp.get(0..10).unwrap_or(timestamp).to_string())
}

fn display_moment(timestamp: &str) -> String {
    crate::timezone::format_local_moment(timestamp)
}

fn display_money(currency: &str, amount: &str) -> String {
    let value = amount.parse::<f64>().unwrap_or_default();
    format!("{currency} ${value:.2}")
}

fn display_file_size(bytes: i64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.0} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes.max(0))
    }
}

fn open_evidence_preview_window(target: &str) -> bool {
    web_sys::window()
        .and_then(|window| {
            window
                .open_with_url_and_target("about:blank", target)
                .ok()
                .flatten()
        })
        .is_some()
}

async fn navigate_evidence_preview(
    access: &api::AdminDamageEvidenceAccess,
    target: &str,
) -> Result<(), String> {
    let url = serde_json::to_string(&access.url).map_err(|error| error.to_string())?;
    let token = serde_json::to_string(&access.access_token).map_err(|error| error.to_string())?;
    let target = serde_json::to_string(target).map_err(|error| error.to_string())?;
    let script = format!(
        r#"
const target = {target};
const popup = window.open('', target);
if (!popup || popup.closed) throw new Error('The evidence preview window was closed');
popup.opener = null;
const url = {url};
const token = {token};
if (token) {{
  const response = await fetch(url, {{
    credentials: 'omit',
    cache: 'no-store',
    headers: {{ 'x-evidence-access-token': token }}
  }});
  if (!response.ok) throw new Error('Private evidence could not be loaded');
  const blobUrl = URL.createObjectURL(await response.blob());
  popup.location.replace(blobUrl);
  setTimeout(() => URL.revokeObjectURL(blobUrl), 300000);
}} else {{
  popup.location.replace(url);
}}
"#
    );
    document::eval(&script)
        .await
        .map(|_| ())
        .map_err(|_| "The private evidence preview could not be opened".into())
}

fn close_evidence_preview(target: &str) {
    let target = serde_json::to_string(target).unwrap_or_else(|_| "\"\"".into());
    document::eval(&format!(
        "const popup = window.open('', {target}); if (popup) popup.close();"
    ));
}

async fn copy_calendar_url(value: &str) -> Result<(), String> {
    let value = serde_json::to_string(value).map_err(|error| error.to_string())?;
    document::eval(&format!(
        "if (!navigator.clipboard) throw new Error('Clipboard is unavailable'); await navigator.clipboard.writeText({value});"
    ))
    .await
    .map(|_| ())
    .map_err(|_| "The calendar URL could not be copied".into())
}

fn calendar_provider_label(provider: &str) -> &'static str {
    match provider {
        "rvezy" => "RVezy",
        "outdoorsy" => "Outdoorsy",
        _ => "External calendar",
    }
}

fn calendar_connection_status(status: &str) -> &'static str {
    match status {
        "connected" => "Connected",
        "error" => "Error",
        "conflict" => "Conflict",
        "disabled" => "Disabled",
        _ => "Needs setup",
    }
}

fn calendar_block_provider(block: &api::AdminAvailabilityBlock) -> Option<&str> {
    block.provider.as_deref().or(match block.source.as_str() {
        "rvezy" => Some("rvezy"),
        "outdoorsy" => Some("outdoorsy"),
        _ => None,
    })
}

fn calendar_block_class(block: &api::AdminAvailabilityBlock) -> String {
    if block.has_conflict {
        "conflict".into()
    } else if let Some(provider) = calendar_block_provider(block) {
        provider.to_string()
    } else {
        "block".into()
    }
}

fn is_external_calendar_block(block: &api::AdminAvailabilityBlock) -> bool {
    block.external_calendar_id.is_some() || calendar_block_provider(block).is_some()
}

#[derive(Clone, Debug, PartialEq)]
struct AdminCalendarExternalGroup {
    key: String,
    label: String,
    css_class: String,
    reason: String,
    count: usize,
}

fn aggregate_external_calendar_blocks<'a>(
    blocks: impl Iterator<Item = &'a api::AdminAvailabilityBlock>,
) -> Vec<AdminCalendarExternalGroup> {
    let mut groups = Vec::<AdminCalendarExternalGroup>::new();
    for block in blocks.filter(|block| is_external_calendar_block(block)) {
        let (key, label, css_class) = if block.has_conflict {
            ("conflict", "Conflict", "conflict")
        } else if let Some(provider) = calendar_block_provider(block) {
            (provider, calendar_provider_label(provider), provider)
        } else {
            ("external", "External", "block")
        };
        if let Some(group) = groups.iter_mut().find(|group| group.key == key) {
            group.count += 1;
        } else {
            groups.push(AdminCalendarExternalGroup {
                key: key.into(),
                label: label.into(),
                css_class: css_class.into(),
                reason: block.reason.clone(),
                count: 1,
            });
        }
    }
    groups.sort_by_key(|group| match group.key.as_str() {
        "rvezy" => 0,
        "outdoorsy" => 1,
        "conflict" => 2,
        _ => 3,
    });
    groups
}

fn status_label(status: &str) -> &'static str {
    match status {
        "pending_payment" => "Pending payment",
        "confirmed" => "Confirmed",
        "active" => "Delivered",
        "completed" => "Returned",
        "cancelled" => "Cancelled",
        "expired" => "Expired",
        _ => "Other",
    }
}

fn payment_label(status: &str) -> String {
    match status {
        "damage_hold" => return "Refundable damage deposit".into(),
        "hold_release" | "release" => return "Deposit refund".into(),
        "damage_capture" | "capture" => return "Damage settlement".into(),
        "captured" => return "Damage settled".into(),
        "released" => return "Deposit refunded".into(),
        _ => {}
    }
    status
        .replace('_', " ")
        .split_whitespace()
        .enumerate()
        .map(|(index, value)| {
            if index == 0 {
                let mut chars = value.chars();
                chars
                    .next()
                    .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
                    .unwrap_or_default()
            } else {
                value.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn payment_booking_context(
    payment: &api::AdminPaymentObligation,
    bookings: &[api::AdminBooking],
) -> (String, String) {
    let booking = bookings
        .iter()
        .find(|booking| booking.booking_id == payment.booking_id);
    let number = if payment.booking_number.is_empty() {
        booking
            .map(|value| value.booking_number.clone())
            .unwrap_or_else(|| payment.booking_id.clone())
    } else {
        payment.booking_number.clone()
    };
    let customer = if payment.customer_name.is_empty() {
        booking
            .map(|value| format!("{} {}", value.first_name, value.last_name))
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "Booking payment".into())
    } else {
        payment.customer_name.clone()
    };
    (number, customer)
}

fn can_resend_payment_link(payment: &api::AdminPaymentObligation) -> bool {
    !payment.financial_operation
        && payment.hosted_url.is_some()
        && matches!(payment.payment_type.as_str(), "balance" | "damage_hold")
        && payment.status == "link_created"
}

fn is_future_booking(booking: &api::AdminBooking, now: DateTime<Utc>) -> bool {
    matches!(
        booking.status.as_str(),
        "pending_payment" | "confirmed" | "active"
    ) && DateTime::parse_from_rfc3339(&booking.ends_at)
        .map(|value| value.with_timezone(&Utc) > now)
        .unwrap_or(false)
}

fn admin_calendar_date(timestamp: &str) -> Option<NaiveDate> {
    DateTime::parse_from_rfc3339(timestamp)
        .ok()
        .map(|value| value.with_timezone(&Vancouver).date_naive())
        .or_else(|| NaiveDate::parse_from_str(timestamp.get(0..10)?, "%Y-%m-%d").ok())
}

fn admin_calendar_short_date(day: NaiveDate) -> String {
    day.format("%b %-d").to_string()
}

fn admin_calendar_weekday(day: NaiveDate) -> String {
    day.format("%a").to_string()
}

fn admin_calendar_day_number(day: NaiveDate) -> String {
    day.format("%-d").to_string()
}

fn booking_occupies_day(booking: &api::AdminBooking, day: NaiveDate) -> bool {
    if matches!(booking.status.as_str(), "cancelled" | "expired") {
        return false;
    }
    admin_calendar_date(&booking.starts_at)
        .zip(admin_calendar_date(&booking.ends_at))
        .is_some_and(|(start, end)| start <= day && day <= end)
}

fn block_occupies_day(block: &api::AdminAvailabilityBlock, day: NaiveDate) -> bool {
    admin_calendar_date(&block.starts_at)
        .zip(admin_calendar_date(&block.ends_at))
        .is_some_and(|(start, end)| start <= day && day <= end)
}

fn calendar_interval_overlaps_window(
    starts_at: &str,
    ends_at: &str,
    window_start: NaiveDate,
    window_end: NaiveDate,
) -> bool {
    admin_calendar_date(starts_at)
        .zip(admin_calendar_date(ends_at))
        .is_some_and(|(start, end)| start <= window_end && end >= window_start)
}

fn is_admin_role(role: &str) -> bool {
    role == "admin"
}

fn valid_admin_amount(value: &str, allow_zero: bool) -> bool {
    value
        .trim()
        .parse::<f64>()
        .is_ok_and(|amount| amount.is_finite() && (amount > 0.0 || (allow_zero && amount == 0.0)))
}

fn admin_amount(value: &str) -> f64 {
    value.trim().parse::<f64>().unwrap_or_default()
}

fn action_confirmation_ready(
    action: &str,
    amount: &str,
    reason: &str,
    evidence_count: usize,
    uploads_pending: usize,
    maximum_amount: f64,
) -> bool {
    let amount_value = admin_amount(amount);
    match action {
        "cancel" => {
            valid_admin_amount(amount, true)
                && amount_value <= maximum_amount + f64::EPSILON
                && !reason.trim().is_empty()
        }
        "capture" => {
            valid_admin_amount(amount, false)
                && amount_value <= maximum_amount + f64::EPSILON
                && !reason.trim().is_empty()
                && evidence_count > 0
                && uploads_pending == 0
        }
        "delivered" | "returned" | "release" | "confirm_deposit" => true,
        _ => false,
    }
}

fn active_damage_hold(detail: &api::AdminBookingDetail) -> Option<&api::AdminPaymentObligation> {
    detail.obligations.iter().find(|obligation| {
        obligation.payment_type == "damage_hold" && obligation.status == "succeeded"
    })
}

fn etransfer_damage_hold(detail: &api::AdminBookingDetail) -> Option<&api::AdminPaymentObligation> {
    detail.obligations.iter().find(|obligation| {
        obligation.payment_type == "damage_hold" && obligation.collection_method == "e_transfer"
    })
}

fn awaiting_etransfer_damage_hold(detail: &api::AdminBookingDetail) -> bool {
    etransfer_damage_hold(detail).is_some_and(|obligation| {
        !matches!(
            obligation.status.as_str(),
            "succeeded" | "released" | "captured"
        )
    })
}

fn hold_action_in_progress(detail: &api::AdminBookingDetail) -> bool {
    detail.financial_operations.iter().any(|operation| {
        matches!(
            operation.operation_type.as_str(),
            "hold_release" | "damage_capture"
        ) && matches!(
            operation.status.as_str(),
            "pending" | "submitted" | "failed"
        )
    })
}

fn refundable_amount(detail: &api::AdminBookingDetail) -> f64 {
    let paid = detail
        .obligations
        .iter()
        .filter(|obligation| {
            matches!(obligation.payment_type.as_str(), "initial" | "balance")
                && obligation.status == "succeeded"
        })
        .map(|obligation| {
            (admin_amount(&obligation.amount) - admin_amount(&obligation.amount_refunded)).max(0.0)
        })
        .sum::<f64>();
    let reserved = detail
        .financial_operations
        .iter()
        .filter(|operation| {
            operation.operation_type == "refund"
                && matches!(operation.status.as_str(), "pending" | "submitted")
        })
        .map(|operation| admin_amount(&operation.amount))
        .sum::<f64>();
    (paid - reserved).max(0.0)
}

fn capturable_damage_amount(detail: &api::AdminBookingDetail) -> f64 {
    active_damage_hold(detail)
        .map(|obligation| {
            (admin_amount(&obligation.amount) - admin_amount(&obligation.amount_refunded)).max(0.0)
        })
        .unwrap_or_default()
}

fn delivery_requirements_ready(detail: &api::AdminBookingDetail) -> bool {
    let trip_obligations = detail
        .obligations
        .iter()
        .filter(|obligation| matches!(obligation.payment_type.as_str(), "initial" | "balance"))
        .collect::<Vec<_>>();
    !trip_obligations.is_empty()
        && trip_obligations
            .iter()
            .all(|obligation| obligation.status == "succeeded")
        && active_damage_hold(detail).is_some()
}

#[allow(clippy::too_many_arguments)]
fn manual_booking_error(
    today: NaiveDate,
    rental_slug: &str,
    starts_on: &str,
    ends_on: &str,
    guests: i32,
    first_name: &str,
    last_name: &str,
    email: &str,
    phone: &str,
    address: &str,
) -> Option<&'static str> {
    let (Ok(start), Ok(end)) = (
        NaiveDate::parse_from_str(starts_on, "%Y-%m-%d"),
        NaiveDate::parse_from_str(ends_on, "%Y-%m-%d"),
    ) else {
        return Some("Choose valid delivery and return dates.");
    };
    if rental_slug.is_empty()
        || start <= today
        || (end - start).num_days() < 1
        || !(1..=10).contains(&guests)
    {
        return Some("Choose an available RV and a future trip of at least one night.");
    }
    if first_name.trim().len() < 2
        || last_name.trim().len() < 2
        || !email.contains('@')
        || phone.trim().len() < 7
        || address.trim().len() < 5
    {
        return Some("Complete the customer and delivery details.");
    }
    None
}

#[component]
pub fn Admin() -> Element {
    let navigator = use_navigator();
    let mut authorized = use_signal(|| None::<bool>);
    let mut active_tab = use_signal(|| "overview".to_string());
    let mut rentals = use_signal(Vec::<api::Rental>::new);
    let mut admin_rentals = use_signal(Vec::<api::AdminRentalSummary>::new);
    let mut addon_templates = use_signal(Vec::<api::RentalAddon>::new);
    let mut admin_rentals_loading = use_signal(|| true);
    let mut bookings = use_signal(Vec::<api::AdminBooking>::new);
    let mut blocks = use_signal(Vec::<api::AdminAvailabilityBlock>::new);
    let mut dashboard = use_signal(api::AdminDashboard::default);
    let mut payments = use_signal(Vec::<api::AdminPaymentObligation>::new);
    let mut audit = use_signal(Vec::<api::AdminAuditEvent>::new);
    let mut payment_config = use_signal(|| None::<api::PaymentConfig>);
    let mut payment_config_error = use_signal(String::new);
    let mut payment_config_retry = use_signal(|| 0_u32);
    let mut email_action_busy = use_signal(|| false);
    let mut loading = use_signal(|| true);
    let mut admin_data_initialized = use_signal(|| false);
    let mut admin_data_failed = use_signal(|| false);
    let mut notice = use_signal(String::new);
    let mut manual_result = use_signal(|| None::<api::CreatedBooking>);
    let mut selected_booking = use_signal(|| None::<api::AdminBookingDetail>);
    let mut drawer_loading = use_signal(|| false);
    let mut admin_load_version = use_signal(|| 0_u64);
    let mut booking_detail_version = use_signal(|| 0_u64);
    let mut rental_detail_version = use_signal(|| 0_u64);
    let mut manual_open = use_signal(|| false);
    let mut rv_editor_open = use_signal(|| false);
    let mut rv_editor_new = use_signal(|| false);
    let mut rv_editor_dirty = use_signal(|| false);
    let mut rv_editor_aux_dirty = use_signal(|| false);
    let mut rv_editor_busy = use_signal(|| false);
    let mut calendar_editor_dirty = use_signal(|| false);
    let calendar_editor_busy = use_signal(|| false);
    let mut selected_rental = use_signal(|| None::<api::AdminRentalDetail>);

    let load_admin_data = move || async move {
        let request_version = admin_load_version().wrapping_add(1);
        admin_load_version.set(request_version);
        loading.set(true);
        admin_rentals_loading.set(true);
        notice.set(String::new());
        let bookings_result = api::admin_bookings().await;
        if *admin_load_version.peek() != request_version {
            return;
        }
        if matches!(
            &bookings_result,
            Err(error) if matches!(error.status, 401 | 403)
        ) {
            authorized.set(Some(false));
            bookings.set(Vec::new());
            dashboard.set(api::AdminDashboard::default());
            payments.set(Vec::new());
            audit.set(Vec::new());
            blocks.set(Vec::new());
            admin_rentals.set(Vec::new());
            addon_templates.set(Vec::new());
            admin_rentals_loading.set(false);
            admin_data_initialized.set(true);
            admin_data_failed.set(false);
            loading.set(false);
            return;
        }
        let dashboard_result = api::admin_dashboard().await;
        if *admin_load_version.peek() != request_version {
            return;
        }
        let payments_result = api::admin_payments().await;
        if *admin_load_version.peek() != request_version {
            return;
        }
        let audit_result = api::admin_audit_events().await;
        if *admin_load_version.peek() != request_version {
            return;
        }
        let blocks_result = api::admin_availability_blocks().await;
        if *admin_load_version.peek() != request_version {
            return;
        }
        let rentals_result = api::admin_rentals().await;
        if *admin_load_version.peek() != request_version {
            return;
        }

        let mut failed_sections = Vec::new();
        if dashboard_result.is_err() {
            failed_sections.push("overview");
        }
        if payments_result.is_err() {
            failed_sections.push("payments");
        }
        if audit_result.is_err() {
            failed_sections.push("audit");
        }
        if blocks_result.is_err() {
            failed_sections.push("calendar");
        }
        if rentals_result.is_err() {
            failed_sections.push("RVs");
        }
        let bookings_loaded = bookings_result.is_ok();
        match bookings_result {
            Ok(values) => bookings.set(values),
            Err(error) => {
                bookings.set(Vec::new());
                notice.set(error.message);
            }
        }
        match dashboard_result {
            Ok(value) => dashboard.set(value),
            Err(_) => dashboard.set(api::AdminDashboard::default()),
        }
        match payments_result {
            Ok(values) => payments.set(values),
            Err(_) => payments.set(Vec::new()),
        }
        match audit_result {
            Ok(values) => audit.set(values),
            Err(_) => audit.set(Vec::new()),
        }
        match blocks_result {
            Ok(values) => blocks.set(values),
            Err(_) => blocks.set(Vec::new()),
        }
        match rentals_result {
            Ok(value) => {
                admin_rentals.set(value.rentals);
                addon_templates.set(value.addon_templates);
            }
            Err(_) => {
                admin_rentals.set(Vec::new());
                addon_templates.set(Vec::new());
            }
        }
        admin_rentals_loading.set(false);
        if bookings_loaded && !failed_sections.is_empty() {
            notice.set(format!(
                "Some admin sections could not refresh: {}. Those sections were locked to an empty state; retry before taking action.",
                failed_sections.join(", ")
            ));
        }
        let data_failed = !bookings_loaded || !failed_sections.is_empty();
        admin_data_initialized.set(true);
        admin_data_failed.set(data_failed);
        loading.set(data_failed);
    };

    use_effect(move || {
        let _retry = *payment_config_retry.read();
        spawn(async move {
            match api::auth_me().await {
                Ok(user) if is_admin_role(&user.role) => {
                    let _ = api::save_auth_user(&user);
                    authorized.set(Some(true));
                    let search = api::CatalogSearchDraft {
                        location: "Kelowna, BC".into(),
                        radius_km: 150,
                        starts_on: None,
                        ends_on: None,
                        guests: 1,
                    };
                    if let Ok(values) = api::catalog(&search).await {
                        rentals.set(values);
                    }
                    payment_config.set(None);
                    payment_config_error.set(String::new());
                    match api::payment_config().await {
                        Ok(value) => payment_config.set(Some(value)),
                        Err(error) => payment_config_error.set(error.message),
                    }
                    load_admin_data().await;
                }
                Ok(_) | Err(_) => {
                    loading.set(false);
                    admin_data_initialized.set(true);
                    admin_data_failed.set(false);
                    authorized.set(Some(false));
                }
            }
        });
    });

    let mut open_booking = move |booking_id: String| {
        let request_version = booking_detail_version().wrapping_add(1);
        booking_detail_version.set(request_version);
        drawer_loading.set(true);
        let fallback = bookings
            .peek()
            .iter()
            .find(|booking| booking.booking_id == booking_id)
            .cloned();
        if let Some(booking) = fallback {
            selected_booking.set(Some(api::AdminBookingDetail {
                booking,
                ..api::AdminBookingDetail::default()
            }));
        }
        spawn(async move {
            let result = api::admin_booking(&booking_id).await;
            if *booking_detail_version.peek() != request_version {
                return;
            }
            match result {
                Ok(value) => selected_booking.set(Some(value)),
                Err(error) => {
                    notice.set(error.message);
                    selected_booking.set(None);
                }
            }
            drawer_loading.set(false);
        });
    };

    let open_rental = move |rental_id: String| {
        let request_version = rental_detail_version().wrapping_add(1);
        rental_detail_version.set(request_version);
        rv_editor_open.set(true);
        rv_editor_new.set(false);
        rv_editor_dirty.set(false);
        rv_editor_aux_dirty.set(false);
        rv_editor_busy.set(false);
        selected_rental.set(None);
        spawn(async move {
            let result = api::admin_rental(&rental_id).await;
            if *rental_detail_version.peek() != request_version {
                return;
            }
            match result {
                Ok(value) => selected_rental.set(Some(value)),
                Err(error) => {
                    notice.set(error.message);
                    rv_editor_open.set(false);
                }
            }
        });
    };

    let now = Utc::now();
    let active_count = bookings
        .read()
        .iter()
        .filter(|booking| is_future_booking(booking, now))
        .count() as i64;
    let pending_count = bookings
        .read()
        .iter()
        .filter(|booking| booking.status == "pending_payment")
        .count() as i64;
    let payment_failures = payments
        .read()
        .iter()
        .filter(|payment| matches!(payment.status.as_str(), "failed" | "past_due"))
        .count() as i64;
    let dashboard_value = dashboard.read().clone();
    let metric_active = dashboard_value.confirmed.max(active_count);
    let metric_pending = dashboard_value.pending_payments.max(pending_count);
    let metric_failures = dashboard_value.payment_errors.max(payment_failures)
        + dashboard_value.notification_failures
        + dashboard_value.push_notification_failures
        + dashboard_value.calendar_sync_failures
        + dashboard_value.calendar_conflicts;
    let payment_availability = api::payment_availability(
        payment_config.read().as_ref(),
        !payment_config_error.read().is_empty(),
    );
    let payments_ready = payment_availability == api::PaymentAvailability::Ready;
    let stripe_mode_label = match payment_config
        .read()
        .as_ref()
        .map(|config| config.mode.as_str())
    {
        Some("live") => "Stripe live",
        Some("test") => "Stripe test",
        _ => "Stripe",
    };
    let rv_editor_key = if rv_editor_new() {
        "new".to_string()
    } else {
        selected_rental
            .read()
            .as_ref()
            .map(|value| value.rental.rental_id.clone())
            .unwrap_or_else(|| "loading".into())
    };
    let mut change_admin_tab = move |next: String| {
        if next == active_tab() {
            return;
        }
        if active_tab() == "rvs" && rv_editor_open() {
            if rv_editor_busy() {
                notice.set("Wait for the RV operation to finish before changing sections.".into());
                return;
            }
            if (rv_editor_dirty() || rv_editor_aux_dirty())
                && !web_sys::window()
                    .and_then(|window| {
                        window
                            .confirm_with_message("Discard unsaved RV changes?")
                            .ok()
                    })
                    .unwrap_or(false)
            {
                return;
            }
            rv_editor_open.set(false);
            rv_editor_new.set(false);
            rv_editor_dirty.set(false);
            rv_editor_aux_dirty.set(false);
            selected_rental.set(None);
            rental_detail_version.set(rental_detail_version().wrapping_add(1));
        }
        if active_tab() == "calendar" {
            if calendar_editor_busy() {
                notice.set(
                    "Wait for the calendar operation to finish before changing sections.".into(),
                );
                return;
            }
            if calendar_editor_dirty()
                && !web_sys::window()
                    .and_then(|window| {
                        window
                            .confirm_with_message("Discard unsaved calendar changes?")
                            .ok()
                    })
                    .unwrap_or(false)
            {
                return;
            }
            calendar_editor_dirty.set(false);
        }
        active_tab.set(next);
    };

    rsx! {
        section {
            class: "admin-page admin-center",
            tabindex: "-1",
            onkeydown: move |event| if event.key() == Key::Escape
                && (manual_open() || rv_editor_open() || selected_booking.read().is_some()) {
                event.stop_propagation();
            },
            if authorized.read().is_none() {
                div { class: "admin-state", h1 { "Checking admin access…" } }
            } else if !authorized.read().unwrap_or(false) {
                div { class: "admin-state",
                    Icon { name: "shield-alert", size: 34, color: "var(--vl-coral)" }
                    h1 { "Admin access required" }
                    p { "Sign in with an administrator account to open the private admin center." }
                    button { class: "btn-forest", onclick: move |_| { navigator.push(Route::Login {}); }, "Sign in" }
                }
            } else {
                div { class: "admin-shell",
                    header { class: "admin-hero admin-center-hero",
                        div {
                            p { class: "admin-kicker", "VL RENTAL · ADMIN CENTER" }
                            h1 { "Operations at a glance" }
                            p { "Bookings, payments, delivery readiness and refundable damage deposits stay together on this page." }
                        }
                        div { class: "admin-hero-actions",
                            span { class: if payments_ready { "admin-mode-badge enabled" } else { "admin-mode-badge" },
                                Icon { name: "shield-check", size: 15, color: "currentColor" }
                                match payment_availability {
                                    api::PaymentAvailability::Loading => "Checking payments…",
                                    api::PaymentAvailability::Disabled => "Payments disabled",
                                    api::PaymentAvailability::Ready => stripe_mode_label,
                                    api::PaymentAvailability::Blocked => "Payments unavailable",
                                }
                            }
                            button { class: "admin-book-link", r#type: "button", disabled: !payments_ready, title: if payments_ready { "Create a phone booking" } else { "Enable verified Stripe payments before creating a phone booking" }, onclick: move |_| if payments_ready { manual_open.set(true); },
                                Icon { name: "phone-call", size: 17, color: "currentColor" }
                                "Phone booking"
                            }
                            button { class: "admin-book-link", r#type: "button", disabled: email_action_busy(), onclick: move |_| { async move { email_action_busy.set(true); match api::admin_test_email().await { Ok(result) => notice.set(result.message), Err(error) => notice.set(error.message) } email_action_busy.set(false); } },
                                Icon { name: "mail-check", size: 17, color: "currentColor" }
                                "Test email"
                            }
                            if dashboard_value.notification_failures > 0 {
                                button { class: "admin-book-link", r#type: "button", disabled: email_action_busy(), onclick: move |_| { async move { email_action_busy.set(true); match api::admin_retry_failed_emails().await { Ok(result) => { notice.set(result.message); load_admin_data().await; }, Err(error) => notice.set(error.message) } email_action_busy.set(false); } },
                                    Icon { name: "refresh-cw", size: 17, color: "currentColor" }
                                    "Retry {dashboard_value.notification_failures} emails"
                                }
                            }
                        }
                    }

                    div { class: "admin-metrics admin-center-metrics",
                        article { span { "CONFIRMED" } strong { "{metric_active}" } small { "active and upcoming" } }
                        article { span { "AWAITING PAYMENT" } strong { "{metric_pending}" } small { "reserved, not confirmed" } }
                        article { span { "PAYMENT / EMAIL / PUSH / CALENDAR" } strong { "{metric_failures}" } small { "need attention" } }
                        article { span { "OVERDUE ACTIONS" } strong { "{dashboard_value.overdue_actions}" } small { "deposit or return decisions" } }
                    }

                    nav { class: "admin-tabs admin-center-tabs", role: "tablist", aria_label: "Admin center sections",
                        for (tab, label, icon) in [
                            ("overview", "Overview", "layout-dashboard"),
                            ("bookings", "Bookings", "notebook-tabs"),
                            ("rvs", "RVs", "caravan"),
                            ("payments", "Payments", "credit-card"),
                            ("calendar", "Calendar", "calendar-days"),
                            ("reviews", "Reviews", "message-square-heart"),
                            ("audit", "Audit", "scroll-text"),
                        ] {
                            button { key: "{tab}", class: match (active_tab() == tab, matches!(tab, "payments" | "calendar" | "reviews" | "audit")) { (true, true) => "active admin-tab-secondary", (false, true) => "admin-tab-secondary", (true, false) => "active", (false, false) => "" }, r#type: "button", role: "tab", aria_selected: active_tab() == tab, onclick: move |_| change_admin_tab(tab.into()),
                                Icon { name: icon, size: 16, color: "currentColor" }
                                "{label}"
                            }
                        }
                        select { class: "admin-mobile-more", aria_label: "More admin sections", value: if matches!(active_tab().as_str(), "payments" | "calendar" | "reviews" | "audit") { active_tab() } else { "more".into() }, onchange: move |event| { let value = event.value(); if matches!(value.as_str(), "payments" | "calendar" | "reviews" | "audit") { change_admin_tab(value); } },
                            option { value: "more", disabled: true, "More" }
                            option { value: "payments", "Payments" }
                            option { value: "calendar", "Calendar" }
                            option { value: "reviews", "Reviews" }
                            option { value: "audit", "Audit log" }
                        }
                    }

                    if !notice.read().is_empty() {
                        div { class: "admin-error admin-page-notice", role: "alert",
                            span { "{notice}" }
                            if loading() {
                                button { r#type: "button", onclick: move |_| { spawn(async move { load_admin_data().await; }); }, "Retry admin data" }
                            }
                        }
                    }
                    if payment_availability == api::PaymentAvailability::Blocked {
                        div { class: "admin-error admin-page-notice", role: "alert",
                            span { if payment_config_error.read().is_empty() { "Stripe configuration could not be verified. Payment actions are blocked." } else { "Payment configuration could not be verified: {payment_config_error}" } }
                            button { r#type: "button", onclick: move |_| { let next = payment_config_retry().wrapping_add(1); payment_config_retry.set(next); }, "Retry" }
                        }
                    }
                    if let Some(created) = manual_result.read().as_ref() {
                        div { class: if created.notification_email_sent { "admin-success admin-page-notice" } else { "admin-error admin-page-notice" }, role: "status",
                            if created.notification_email_sent {
                                span { "Phone booking {created.booking.booking_number} was reserved for two hours and its payment link was emailed." }
                            } else {
                                span { "Phone booking {created.booking.booking_number} was reserved, but email delivery was not confirmed." }
                                if let Some(url) = created.checkout_url.as_ref() { a { href: "{url}", target: "_blank", rel: "noopener noreferrer", "Open the existing Checkout link" } }
                            }
                            button { r#type: "button", aria_label: "Dismiss phone booking result", onclick: move |_| manual_result.set(None), Icon { name: "x", size: 15, color: "currentColor" } }
                        }
                    }

                    main { class: "admin-tab-content",
                        match active_tab().as_str() {
                            "overview" => rsx! { OverviewTab { dashboard: dashboard_value, bookings: bookings.read().clone(), loading: loading(), on_open_booking: open_booking } },
                            "bookings" => rsx! { BookingsTab { bookings: bookings.read().clone(), rentals: rentals.read().clone(), loading: loading(), on_open_booking: open_booking } },
                            "rvs" => if rv_editor_open() {
                                rsx! { RvEditorPanel {
                                    key: "{rv_editor_key}",
                                    detail: selected_rental.read().clone(),
                                    addon_templates: addon_templates.read().clone(),
                                    is_new: rv_editor_new(),
                                    dirty: rv_editor_dirty,
                                    aux_dirty: rv_editor_aux_dirty,
                                    busy: rv_editor_busy,
                                    on_close: move |_| {
                                        rental_detail_version.set(rental_detail_version().wrapping_add(1));
                                        rv_editor_open.set(false);
                                        rv_editor_new.set(false);
                                        rv_editor_dirty.set(false);
                                        rv_editor_aux_dirty.set(false);
                                        selected_rental.set(None);
                                    },
                                    on_changed: move |value: api::AdminRentalDetail| {
                                        rv_editor_new.set(false);
                                        selected_rental.set(Some(value));
                                        spawn(async move { load_admin_data().await; });
                                    }
                                } }
                            } else {
                                rsx! { RvsTab { rentals: admin_rentals.read().clone(), loading: admin_rentals_loading(), on_open_rental: open_rental, on_add_rental: move |_| {
                                    rental_detail_version.set(rental_detail_version().wrapping_add(1));
                                    selected_rental.set(None);
                                    rv_editor_new.set(true);
                                    rv_editor_dirty.set(false);
                                    rv_editor_aux_dirty.set(false);
                                    rv_editor_busy.set(false);
                                    rv_editor_open.set(true);
                                } } }
                            },
                            "payments" => rsx! { PaymentsTab { payments: payments.read().clone(), bookings: bookings.read().clone(), loading: loading(), on_open_booking: open_booking, on_refresh: move |_| { spawn(async move { load_admin_data().await; }); } } },
                            "calendar" => if !admin_data_initialized() || admin_data_failed() {
                                rsx! { section { class: "admin-panel admin-full-panel", AdminLoading {} } }
                            } else {
                                rsx! { CalendarTab { bookings: bookings.read().clone(), blocks: blocks.read().clone(), rentals: rentals.read().clone(), editor_dirty: calendar_editor_dirty, busy: calendar_editor_busy, on_open_booking: open_booking, on_refresh: move |_| { spawn(async move { load_admin_data().await; }); } } }
                            },
                            "reviews" => rsx! { ReviewsTab { rentals: admin_rentals.read().clone() } },
                            "audit" => rsx! { AuditTab { events: audit.read().clone(), loading: loading() } },
                            _ => rsx! {},
                        }
                    }
                }

                if let Some(detail) = selected_booking.read().clone() {
                    BookingDrawer {
                        key: "{detail.booking.booking_id}-{drawer_loading()}",
                        detail,
                        loading: drawer_loading(),
                        on_close: move |_| {
                            booking_detail_version.set(booking_detail_version().wrapping_add(1));
                            drawer_loading.set(false);
                            selected_booking.set(None);
                        },
                        on_changed: move |value| {
                            selected_booking.set(Some(value));
                            spawn(async move { load_admin_data().await; });
                        }
                    }
                }

                if manual_open() {
                    ManualBookingModal {
                        rentals: rentals.read().clone(),
                        on_close: move |_| manual_open.set(false),
                        on_created: move |created: api::CreatedBooking| {
                            manual_open.set(false);
                            let booking_id = created.booking.booking_id.clone();
                            manual_result.set(Some(created));
                            open_booking(booking_id);
                            spawn(async move { load_admin_data().await; });
                        }
                    }
                }

            }
        }
    }
}

#[component]
fn OverviewTab(
    dashboard: api::AdminDashboard,
    bookings: Vec<api::AdminBooking>,
    loading: bool,
    on_open_booking: EventHandler<String>,
) -> Element {
    let upcoming = bookings
        .iter()
        .filter(|booking| matches!(booking.status.as_str(), "confirmed" | "active"))
        .take(6)
        .cloned()
        .collect::<Vec<_>>();
    rsx! {
        div { class: "admin-overview-grid",
            section { class: "admin-panel admin-attention-panel",
                div { class: "admin-panel-head", div { h2 { "Needs attention" } p { "Payment, delivery, notification and calendar sync issues that need an admin decision." } } span { "{dashboard.attention.len() + dashboard.calendar_attention.len()} items" } }
                if loading { AdminLoading {} }
                else if dashboard.attention.is_empty() && dashboard.calendar_attention.is_empty() {
                    div { class: "admin-empty admin-empty-positive", Icon { name: "circle-check-big", size: 24, color: "var(--vl-forest)" } "No urgent actions right now." }
                } else {
                    div { class: "admin-attention-list",
                        for item in dashboard.attention.iter() {
                            button { class: "admin-attention-item", r#type: "button", disabled: item.booking_id.is_none(), onclick: { let id = item.booking_id.clone(); move |_| if let Some(id) = id.clone() { on_open_booking.call(id); } },
                                span { class: "admin-attention-icon", if item.severity == "critical" { Icon { name: "triangle-alert", size: 17, color: "currentColor" } } else { Icon { name: "clock-3", size: 17, color: "currentColor" } } }
                                span { strong { if item.title.is_empty() { "{payment_label(&item.item_type)} · {item.booking_number}" } else { "{item.title}" } } small { "{item.detail}" if let Some(due) = item.due_at.as_ref() { " · due {display_moment(due)}" } } }
                                Icon { name: "chevron-right", size: 16, color: "var(--vl-muted)" }
                            }
                        }
                        for item in dashboard.calendar_attention.iter() {
                            article { class: "admin-attention-item",
                                span { class: "admin-attention-icon", Icon { name: "calendar-warning", size: 17, color: "currentColor" } }
                                span { strong { "{calendar_provider_label(&item.provider)} · {item.rental_name}" } small { "{item.message}" if let Some(attempted) = item.last_attempted_at.as_ref() { " · checked {display_moment(attempted)}" } } }
                                span { class: if item.kind == "calendar_conflict" { "admin-status admin-status-failed" } else { "admin-status admin-status-pending" }, if item.kind == "calendar_conflict" { "Conflict" } else { "Sync error" } }
                            }
                        }
                    }
                }
            }
            section { class: "admin-panel admin-today-panel",
                div { class: "admin-panel-head", div { h2 { "Today" } p { "Deliveries, returns and payment deadlines." } } span { "{dashboard.today.len()} events" } }
                if dashboard.today.is_empty() {
                    div { class: "admin-empty", "Nothing is scheduled for today." }
                } else {
                    div { class: "admin-today-list",
                        for item in dashboard.today.iter() {
                            button { r#type: "button", onclick: { let id = item.booking_id.clone(); move |_| on_open_booking.call(id.clone()) },
                                time { if item.scheduled_at.is_empty() { "{display_moment(&item.starts_at)}" } else { "{display_moment(&item.scheduled_at)}" } }
                                strong { if item.action.is_empty() { "{status_label(&item.status)}" } else { "{item.action}" } }
                                span { if item.customer_name.is_empty() { "{item.first_name} {item.last_name}" } else { "{item.customer_name}" } " · {item.rental_name}" }
                            }
                        }
                    }
                }
            }
            section { class: "admin-panel admin-upcoming-panel",
                div { class: "admin-panel-head", div { h2 { "Upcoming trips" } p { "Open a booking without leaving the dashboard." } } }
                if upcoming.is_empty() { div { class: "admin-empty", "No confirmed upcoming trips." } }
                else { div { class: "admin-compact-bookings",
                    for booking in upcoming {
                        button { key: "{booking.booking_id}", r#type: "button", onclick: { let id = booking.booking_id.clone(); move |_| on_open_booking.call(id.clone()) },
                            div { strong { "{booking.first_name} {booking.last_name}" } small { "{booking.booking_number}" } }
                            span { "{booking.rental_name}" }
                            time { "{display_date(&booking.starts_at)}" }
                            span { class: "admin-status admin-status-{booking.status}", "{status_label(&booking.status)}" }
                        }
                    }
                } }
            }
        }
    }
}

#[component]
fn BookingsTab(
    bookings: Vec<api::AdminBooking>,
    rentals: Vec<api::Rental>,
    loading: bool,
    on_open_booking: EventHandler<String>,
) -> Element {
    let mut search = use_signal(String::new);
    let mut status = use_signal(|| "all".to_string());
    let mut rental = use_signal(|| "all".to_string());
    let query = search().trim().to_lowercase();
    let filtered = bookings
        .iter()
        .filter(|booking| status() == "all" || booking.status == status())
        .filter(|booking| rental() == "all" || booking.rental_slug == rental())
        .filter(|booking| {
            query.is_empty()
                || format!(
                    "{} {} {} {} {} {}",
                    booking.first_name,
                    booking.last_name,
                    booking.email,
                    booking.phone,
                    booking.booking_number,
                    booking.rental_name
                )
                .to_lowercase()
                .contains(&query)
        })
        .cloned()
        .collect::<Vec<_>>();
    rsx! {
        section { class: "admin-panel admin-bookings-panel admin-full-panel",
            div { class: "admin-panel-head admin-list-head",
                div { h2 { "Bookings" } p { "{filtered.len()} matching reservations" } }
                div { class: "admin-inline-filters",
                    label { class: "admin-search", Icon { name: "search", size: 16, color: "var(--vl-muted)" } input { r#type: "search", placeholder: "Guest, email or booking #", value: "{search}", oninput: move |event| search.set(event.value()) } }
                    select { aria_label: "Filter bookings by RV", value: "{rental}", onchange: move |event| rental.set(event.value()), option { value: "all", "All RVs" } for item in rentals.iter() { option { value: "{item.slug}", "{item.name}" } } }
                    select { aria_label: "Filter bookings by status", value: "{status}", onchange: move |event| status.set(event.value()),
                        option { value: "all", "All statuses" }
                        option { value: "pending_payment", "Pending payment" }
                        option { value: "confirmed", "Confirmed" }
                        option { value: "active", "Delivered" }
                        option { value: "completed", "Returned" }
                        option { value: "cancelled", "Cancelled" }
                        option { value: "expired", "Expired" }
                    }
                }
            }
            if loading { AdminLoading {} }
            else if filtered.is_empty() { div { class: "admin-empty", "No bookings match these filters." } }
            else {
                div { class: "admin-table-wrap",
                    table { class: "admin-bookings-table",
                        thead { tr { th { "Guest" } th { "RV" } th { "Trip" } th { "Booking" } th { "Payment" } th { "Total" } th { "" } } }
                        tbody { for booking in filtered.iter() { tr { key: "{booking.booking_id}",
                            td { div { class: "admin-guest", strong { "{booking.first_name} {booking.last_name}" } a { href: "mailto:{booking.email}", "{booking.email}" } small { "{booking.booking_number}" } } }
                            td { strong { class: "admin-rv-name", "{booking.rental_name}" } small { "{booking.guests} guests" } }
                            td { span { class: "admin-date-range", "{display_date(&booking.starts_at)} → {display_date(&booking.ends_at)}" } }
                            td { span { class: "admin-status admin-status-{booking.status}", "{status_label(&booking.status)}" } }
                            td { span { class: "admin-payment", "{payment_label(&booking.payment_status)}" } }
                            td { strong { "{display_money(&booking.currency, &booking.total)}" } }
                            td { button { class: "admin-row-open", r#type: "button", onclick: { let id = booking.booking_id.clone(); move |_| on_open_booking.call(id.clone()) }, "Open" } }
                        } } }
                    }
                }
                div { class: "admin-booking-cards", for booking in filtered.iter() {
                    button { class: "admin-booking-card admin-card-button", key: "mobile-{booking.booking_id}", r#type: "button", onclick: { let id = booking.booking_id.clone(); move |_| on_open_booking.call(id.clone()) },
                        div { class: "admin-booking-card-head", div { strong { "{booking.first_name} {booking.last_name}" } small { "{booking.booking_number}" } } span { class: "admin-status admin-status-{booking.status}", "{status_label(&booking.status)}" } }
                        dl { div { dt { "RV" } dd { "{booking.rental_name}" } } div { dt { "Trip" } dd { "{display_date(&booking.starts_at)} → {display_date(&booking.ends_at)}" } } div { dt { "Payment" } dd { "{payment_label(&booking.payment_status)}" } } div { dt { "Total" } dd { "{display_money(&booking.currency, &booking.total)}" } } }
                    }
                } }
            }
        }
    }
}

#[component]
fn RvsTab(
    rentals: Vec<api::AdminRentalSummary>,
    loading: bool,
    on_open_rental: EventHandler<String>,
    on_add_rental: EventHandler<()>,
) -> Element {
    let mut search = use_signal(String::new);
    // New RVs are created as drafts. Showing every status by default keeps the
    // newly saved RV visible when the editor returns to this list.
    let mut status = use_signal(|| "all".to_string());
    let query = search().trim().to_lowercase();
    let filtered = rentals
        .iter()
        .filter(|rental| match status().as_str() {
            "published" => rental.is_active,
            "archived" => !rental.is_active,
            _ => true,
        })
        .filter(|rental| {
            query.is_empty()
                || format!(
                    "{} {} {} {}",
                    rental.name, rental.manufacturer, rental.model, rental.slug
                )
                .to_lowercase()
                .contains(&query)
        })
        .cloned()
        .collect::<Vec<_>>();
    rsx! {
        section { class: "admin-panel admin-full-panel admin-rvs-panel",
            div { class: "admin-panel-head admin-list-head",
                div { h2 { "RVs" } p { "Trailers, photos, features, pricing, and RV-specific add-ons." } }
                div { class: "admin-inline-filters",
                    label { class: "admin-search", Icon { name: "search", size: 16, color: "var(--vl-muted)" } input { r#type: "search", placeholder: "Name, model, or slug", value: "{search}", oninput: move |event| search.set(event.value()) } }
                    select { value: "{status}", onchange: move |event| status.set(event.value()), option { value: "published", "Published" } option { value: "archived", "Archived" } option { value: "all", "All RVs" } }
                    button { class: "admin-primary-small", r#type: "button", onclick: move |_| on_add_rental.call(()), Icon { name: "plus", size: 15, color: "currentColor" } "Add RV" }
                }
            }
            if loading { AdminLoading {} }
            else if filtered.is_empty() { div { class: "admin-empty", "No RVs match these filters." } }
            else { div { class: "admin-rv-grid",
                for rental in filtered {
                    button { key: "{rental.rental_id}", class: "admin-rv-card", r#type: "button", onclick: { let id=rental.rental_id.clone(); move |_| on_open_rental.call(id.clone()) },
                        if let Some(image) = rental.hero_image_url.as_ref() { img { src: "{image}", alt: "{rental.name}" } } else { span { class: "admin-rv-placeholder", Icon { name: "image-plus", size: 28, color: "currentColor" } } }
                        div { class: "admin-rv-card-copy",
                            span { class: if rental.is_active { "admin-status admin-status-confirmed" } else { "admin-status admin-status-cancelled" }, if rental.is_active { "Published" } else { "Archived" } }
                            h3 { "{rental.name}" }
                            p { "{rental.model_year.map(|v|v.to_string()).unwrap_or_default()} {rental.manufacturer} {rental.model}" }
                            dl { div { dt { "Photos" } dd { "{rental.media_count}" } } div { dt { "Add-ons" } dd { "{rental.addon_count}" } } div { dt { "Sleeps" } dd { "{rental.capacity}" } } div { dt { "Nightly" } dd { "{display_money(&rental.currency, &rental.base_rate)}" } } }
                        }
                    }
                }
            } }
        }
    }
}

async fn refresh_rental_editor_after_change(
    rental_id: &str,
    mut message: Signal<String>,
    on_changed: EventHandler<api::AdminRentalDetail>,
) {
    match api::admin_rental(rental_id).await {
        Ok(next) => on_changed.call(next),
        Err(error) => message.set(format!(
            "The change was saved, but RV details could not refresh: {}",
            error.message
        )),
    }
}

#[component]
fn RvEditorPanel(
    detail: Option<api::AdminRentalDetail>,
    addon_templates: Vec<api::RentalAddon>,
    is_new: bool,
    mut dirty: Signal<bool>,
    mut aux_dirty: Signal<bool>,
    mut busy: Signal<bool>,
    on_close: EventHandler<()>,
    on_changed: EventHandler<api::AdminRentalDetail>,
) -> Element {
    if !is_new && detail.is_none() {
        return rsx! {
            section {
                class: "admin-panel admin-rv-editor-panel",
                aria_label: "Loading RV editor",
                tabindex: "-1",
                autofocus: true,
                onkeydown: move |event| if event.key() == Key::Escape {
                    event.stop_propagation();
                    on_close.call(());
                },
                header { class: "admin-drawer-head",
                    div { p { "RV DETAILS" } h2 { "Loading RV…" } }
                    button { r#type: "button", aria_label: "Back to RV list", onclick: move |_| on_close.call(()),
                        Icon { name: "arrow-left", size: 18, color: "currentColor" }
                        span { "Back to RV list" }
                    }
                }
                AdminLoading {}
            }
        };
    }
    let initial = detail.clone();
    let mut name = use_signal(|| {
        initial
            .as_ref()
            .map(|v| v.rental.name.clone())
            .unwrap_or_default()
    });
    let mut year = use_signal(|| {
        initial
            .as_ref()
            .and_then(|v| v.rental.model_year)
            .map(|v| v.to_string())
            .unwrap_or_default()
    });
    let mut manufacturer = use_signal(|| {
        initial
            .as_ref()
            .map(|v| v.rental.manufacturer.clone())
            .unwrap_or_default()
    });
    let mut model = use_signal(|| {
        initial
            .as_ref()
            .map(|v| v.rental.model.clone())
            .unwrap_or_default()
    });
    let mut rv_type = use_signal(|| {
        initial
            .as_ref()
            .map(|v| v.rental.rv_type.clone())
            .unwrap_or_else(|| "travel_trailer".into())
    });
    let mut summary = use_signal(|| {
        initial
            .as_ref()
            .map(|v| v.rental.summary.clone())
            .unwrap_or_default()
    });
    let mut description = use_signal(|| {
        initial
            .as_ref()
            .map(|v| v.rental.description.clone())
            .unwrap_or_default()
    });
    let mut capacity = use_signal(|| {
        initial
            .as_ref()
            .map(|v| v.rental.capacity.to_string())
            .unwrap_or_else(|| "1".into())
    });
    let mut length = use_signal(|| {
        initial
            .as_ref()
            .and_then(|v| v.rental.length_ft.clone())
            .unwrap_or_default()
    });
    let mut slide_outs = use_signal(|| {
        initial
            .as_ref()
            .map(|v| v.rental.slide_outs.to_string())
            .unwrap_or_else(|| "0".into())
    });
    let mut pet_friendly = use_signal(|| initial.as_ref().is_some_and(|v| v.rental.pet_friendly));
    let mut nightly = use_signal(|| {
        initial
            .as_ref()
            .map(|v| v.rental.base_rate.clone())
            .unwrap_or_else(|| "0".into())
    });
    let mut cleaning = use_signal(|| {
        initial
            .as_ref()
            .map(|v| v.cleaning_fee.clone())
            .unwrap_or_else(|| "0".into())
    });
    let mut sort_order = use_signal(|| {
        initial
            .as_ref()
            .map(|v| v.rental.sort_order.to_string())
            .unwrap_or_else(|| "0".into())
    });
    let mut message = use_signal(String::new);
    let mut photo_file = use_signal(|| None::<web_sys::File>);
    let mut photo_alt = use_signal(String::new);
    let mut photo_cover = use_signal(|| initial.as_ref().is_none_or(|v| v.media.is_empty()));
    let mut dragged_media_id = use_signal(String::new);
    let mut feature_id = use_signal(String::new);
    let mut feature_group = use_signal(|| "highlight".to_string());
    let mut feature_icon = use_signal(|| "circle-check".to_string());
    let mut feature_label = use_signal(String::new);
    let mut feature_description = use_signal(String::new);
    let mut feature_sort_order = use_signal(|| {
        initial
            .as_ref()
            .map(|value| value.features.len() as i32)
            .unwrap_or(0)
    });
    let mut addon_id = use_signal(String::new);
    let mut addon_label = use_signal(String::new);
    let mut addon_description = use_signal(String::new);
    let mut addon_icon = use_signal(|| "sparkles".to_string());
    let mut addon_price = use_signal(|| "50".to_string());
    let mut addon_charge = use_signal(|| "per_booking".to_string());
    let mut addon_recommended = use_signal(|| false);
    let mut addon_active = use_signal(|| true);
    let mut photo_draft_dirty = use_signal(|| false);
    let mut feature_draft_dirty = use_signal(|| false);
    let mut addon_draft_dirty = use_signal(|| false);
    let mut addon_sort_order = use_signal(|| {
        initial
            .as_ref()
            .map(|value| value.addons.len() as i32)
            .unwrap_or(0)
    });
    let mut selected_addon_templates = use_signal(std::collections::HashSet::<String>::new);
    let rental_id = initial
        .as_ref()
        .map(|v| v.rental.rental_id.clone())
        .unwrap_or_default();
    let next_media_order = initial.as_ref().map(|value| value.media.len()).unwrap_or(0) as i32;
    let next_feature_order = initial
        .as_ref()
        .map(|value| value.features.len())
        .unwrap_or(0) as i32;
    let next_addon_order = initial
        .as_ref()
        .map(|value| value.addons.len())
        .unwrap_or(0) as i32;
    let published = initial.as_ref().is_some_and(|v| v.rental.is_active);
    let slug = initial
        .as_ref()
        .map(|v| v.rental.slug.clone())
        .unwrap_or_else(|| "Created automatically".into());
    let media_rows_for_reorder = detail
        .as_ref()
        .map(|value| value.media.clone())
        .unwrap_or_default();
    let request_close = move || {
        if busy() {
            return;
        }
        let may_close = (!dirty() && !aux_dirty())
            || web_sys::window()
                .and_then(|w| w.confirm_with_message("Discard unsaved RV changes?").ok())
                .unwrap_or(false);
        if may_close {
            on_close.call(());
        }
    };
    let save_rental_id = rental_id.clone();
    let mut save = move || {
        let creating = is_new;
        let payload = api::AdminRentalPayload {
            name: name(),
            model_year: year().trim().parse().ok(),
            manufacturer: manufacturer(),
            model: model(),
            rv_type: rv_type(),
            summary: summary(),
            description: description(),
            capacity: capacity().trim().parse().unwrap_or(0),
            length_ft: if length().trim().is_empty() {
                None
            } else {
                Some(length())
            },
            slide_outs: slide_outs().trim().parse().unwrap_or(-1),
            pet_friendly: pet_friendly(),
            nightly_rate: nightly(),
            cleaning_fee: cleaning(),
            sort_order: sort_order().trim().parse().unwrap_or(0),
            addon_template_keys: if creating {
                selected_addon_templates.read().iter().cloned().collect()
            } else {
                Vec::new()
            },
        };
        let id = save_rental_id.clone();
        busy.set(true);
        message.set(String::new());
        spawn(async move {
            let result = if creating {
                api::create_admin_rental(&payload).await
            } else {
                api::update_admin_rental(&id, &payload).await
            };
            match result {
                Ok(value) => {
                    dirty.set(false);
                    message.set("RV saved.".into());
                    on_changed.call(value)
                }
                Err(error) => message.set(error.message),
            }
            busy.set(false);
        });
    };
    rsx! {
        section { class: "admin-panel admin-rv-editor-panel", role: "region", aria_label: if is_new { "Add RV" } else { "Edit RV" }, tabindex: "-1", autofocus: true, onkeydown: move |event| { event.stop_propagation(); if event.key()==Key::Escape { request_close(); } },
                header { class: "admin-drawer-head", div { p { if published { "PUBLISHED RV" } else if is_new { "NEW DRAFT RV" } else { "ARCHIVED / DRAFT RV" } } h2 { if is_new { "Add RV" } else { "{name}" } } span { "Slug: {slug}" } } button { r#type:"button", disabled:busy(), aria_label:"Back to RV list", onclick:move |_|request_close(), Icon{name:"arrow-left",size:18,color:"currentColor"} span { "Back to RV list" } } }
                div { class: "admin-drawer-scroll admin-rv-editor-scroll",
                    if !message.read().is_empty() { p { class: if message().contains("saved") { "admin-success" } else { "admin-error" }, "{message}" } }
                    section { class:"admin-drawer-section", h3 { "RV details" }
                        div { class:"admin-rv-form-grid",
                            label { "Customer-facing name" input { value:"{name}", disabled:busy(), oninput:move|e|{name.set(e.value());dirty.set(true)} } }
                            label { "Slug" input { value:"{slug}", disabled:true } }
                            label { "Year" input { r#type:"number", min:"1950", max:"2100", value:"{year}", disabled:busy(), oninput:move|e|{year.set(e.value());dirty.set(true)} } }
                            label { "Manufacturer" input { value:"{manufacturer}", disabled:busy(), oninput:move|e|{manufacturer.set(e.value());dirty.set(true)} } }
                            label { "Model" input { value:"{model}", disabled:busy(), oninput:move|e|{model.set(e.value());dirty.set(true)} } }
                            label { "RV type" select { value:"{rv_type}", disabled:busy(), onchange:move|e|{rv_type.set(e.value());dirty.set(true)}, option{value:"travel_trailer","Travel trailer"} option{value:"fifth_wheel","Fifth wheel"} option{value:"toy_hauler","Toy hauler"} } }
                            label { class:"admin-field-wide", "Short summary" textarea { value:"{summary}", disabled:busy(), oninput:move|e|{summary.set(e.value());dirty.set(true)} } }
                            label { class:"admin-field-wide", "Full description" textarea { rows:"5", value:"{description}", disabled:busy(), oninput:move|e|{description.set(e.value());dirty.set(true)} } }
                            label { "Sleeps" input { r#type:"number", min:"1", max:"10", value:"{capacity}", disabled:busy(), oninput:move|e|{capacity.set(e.value());dirty.set(true)} } }
                            label { "Length (ft)" input { r#type:"number", min:"1", step:"0.1", value:"{length}", disabled:busy(), oninput:move|e|{length.set(e.value());dirty.set(true)} } }
                            label { "Slide-outs" input { r#type:"number", min:"0", max:"10", value:"{slide_outs}", disabled:busy(), oninput:move|e|{slide_outs.set(e.value());dirty.set(true)} } }
                            label { "Nightly rate (CAD)" input { r#type:"number", min:"0", step:"0.01", value:"{nightly}", disabled:busy(), oninput:move|e|{nightly.set(e.value());dirty.set(true)} } }
                            label { "Cleaning fee (CAD)" input { r#type:"number", min:"0", step:"0.01", value:"{cleaning}", disabled:busy(), oninput:move|e|{cleaning.set(e.value());dirty.set(true)} } }
                            label { "Catalog order" input { r#type:"number", value:"{sort_order}", disabled:busy(), oninput:move|e|{sort_order.set(e.value());dirty.set(true)} } }
                            label { class:"admin-check-field", input { r#type:"checkbox", checked:pet_friendly(), disabled:busy(), onchange:move|e|{pet_friendly.set(e.checked());dirty.set(true)} } "Pet friendly" }
                        }
                        p { class:"admin-system-rules", "Fixed: RV · CAD · 1+ night stay with 3-night minimum pricing · CA$97 prep · protection CA$150 for 3 nights, then CA$30/extra night · CA$1,000 refundable deposit · delivery from Kelowna up to 150 km." }
                    }
                    if is_new && !addon_templates.is_empty() {
                        section { class:"admin-drawer-section admin-addon-template-section",
                            h3 { "Add-on templates" }
                            p { class:"admin-system-rules", "Select the add-ons to copy into this RV. You can edit every copied item after the RV is saved." }
                            div { class:"admin-addon-template-actions",
                                button { r#type:"button", disabled:busy(), onclick:{let keys=addon_templates.iter().map(|item|item.addon_key.clone()).collect::<std::collections::HashSet<_>>();move |_|{selected_addon_templates.set(keys.clone());dirty.set(true)}}, "Select all" }
                                button { r#type:"button", disabled:busy()||selected_addon_templates.read().is_empty(), onclick:move |_|{selected_addon_templates.set(std::collections::HashSet::new());dirty.set(true)}, "Clear" }
                            }
                            div { class:"admin-addon-template-grid",
                                for template in addon_templates.iter() {
                                    label { key:"template-{template.addon_key}", class:"admin-addon-template-choice",
                                        input { r#type:"checkbox", checked:selected_addon_templates.read().contains(&template.addon_key), disabled:busy(), onchange:{let key=template.addon_key.clone();move|event|{let mut next=selected_addon_templates.read().clone();if event.checked(){next.insert(key.clone());}else{next.remove(&key);}selected_addon_templates.set(next);dirty.set(true)}} }
                                        span { strong { "{template.label}" } small { "CA${template.price} · " if template.charge_type=="per_unit" { "per night" } else { "per booking" } } }
                                    }
                                }
                            }
                        }
                    }
                    if let Some(value)=detail {
                        section { class:"admin-drawer-section", h3 { "Photos ({value.media.len()}/40)" }
                            div { class:"admin-rv-photo-upload", label { "Image file" input { r#type:"file", disabled:busy()||value.media.len()>=40, accept:"image/jpeg,image/png,image/webp", oninput:move|event|{for file in event.files(){if file.size()>10*1024*1024{message.set(format!("{} is larger than 10 MB.",file.name()));continue}photo_file.set(file.inner().downcast_ref::<web_sys::File>().cloned());photo_draft_dirty.set(true);aux_dirty.set(true);break}} } } label { "Alt text" input { value:"{photo_alt}", disabled:busy(), oninput:move|e|{photo_alt.set(e.value());photo_draft_dirty.set(true);aux_dirty.set(true)} } } label { class:"admin-check-field", input { r#type:"checkbox", checked:photo_cover(), disabled:busy(), onchange:move|e|{photo_cover.set(e.checked());photo_draft_dirty.set(true);aux_dirty.set(true)} } "Cover photo" } button { r#type:"button", disabled:busy()||value.media.len()>=40||photo_file.read().is_none()||photo_alt().trim().is_empty(), onclick:{let id=rental_id.clone();move |_|{let Some(file)=photo_file.read().clone()else{return};let alt=photo_alt();let cover=photo_cover();let id=id.clone();busy.set(true);let _=spawn(async move{match api::upload_admin_rental_media(&id,&file,&alt,cover,next_media_order).await{Ok(_)=>{photo_file.set(None);photo_alt.set(String::new());photo_draft_dirty.set(false);aux_dirty.set(feature_draft_dirty()||addon_draft_dirty());refresh_rental_editor_after_change(&id,message,on_changed).await},Err(error)=>message.set(error.message)}busy.set(false)});}}, "Upload photo" } }
                            div { class:"admin-rv-media-grid", for media in value.media.iter(){ article { key:"{media.media_id}", class:"admin-rv-media-card", draggable:!busy(), title:"Drag to change photo order", ondragstart:{let media_id=media.media_id.clone();move |_|if !busy(){dragged_media_id.set(media_id.clone())}}, ondragover:move|event|event.prevent_default(), ondrop:{let id=rental_id.clone();let target_id=media.media_id.clone();let media_rows=media_rows_for_reorder.clone();move|event|{event.prevent_default();if busy(){return}let source_id=dragged_media_id();if source_id.is_empty()||source_id==target_id{return}let mut media_ids=media_rows.iter().map(|item|item.media_id.clone()).collect::<Vec<_>>();let Some(source_index)=media_ids.iter().position(|item|item==&source_id)else{return};let Some(target_index)=media_ids.iter().position(|item|item==&target_id)else{return};let moved=media_ids.remove(source_index);media_ids.insert(target_index,moved);let id=id.clone();dragged_media_id.set(String::new());busy.set(true);let _=spawn(async move{match api::reorder_admin_rental_media(&id,&media_ids).await{Ok(_)=>refresh_rental_editor_after_change(&id,message,on_changed).await,Err(error)=>message.set(error.message)}busy.set(false)});}}, span { class:"admin-rv-drag-handle", "⋮⋮ Drag" } img { src:"{media.source_url}", alt:"{media.alt_text}" } label { "Alt text" input { value:"{media.alt_text}", disabled:busy(), onchange:{let id=rental_id.clone();let media_id=media.media_id.clone();let order=media.sort_order;let cover=media.is_cover;move|e|{let alt=e.value();let id=id.clone();let media_id=media_id.clone();busy.set(true);let _=spawn(async move{match api::update_admin_rental_media(&id,&media_id,&alt,order,cover).await{Ok(_)=>refresh_rental_editor_after_change(&id,message,on_changed).await,Err(error)=>message.set(error.message)}busy.set(false)});}} } } div { if media.is_cover { span { "Cover" } } else { button { r#type:"button", disabled:busy(), onclick:{let id=rental_id.clone();let media_id=media.media_id.clone();let alt=media.alt_text.clone();let order=media.sort_order;move |_|{let id=id.clone();let media_id=media_id.clone();let alt=alt.clone();busy.set(true);let _=spawn(async move{match api::update_admin_rental_media(&id,&media_id,&alt,order,true).await{Ok(_)=>refresh_rental_editor_after_change(&id,message,on_changed).await,Err(error)=>message.set(error.message)}busy.set(false)});}}, "Make cover" } } input { aria_label:"Photo order", r#type:"number", value:"{media.sort_order}", disabled:busy(), onchange:{let id=rental_id.clone();let media_id=media.media_id.clone();let alt=media.alt_text.clone();let cover=media.is_cover;move|e|{let order=e.value().parse().unwrap_or(0);let id=id.clone();let media_id=media_id.clone();let alt=alt.clone();busy.set(true);let _=spawn(async move{match api::update_admin_rental_media(&id,&media_id,&alt,order,cover).await{Ok(_)=>refresh_rental_editor_after_change(&id,message,on_changed).await,Err(error)=>message.set(error.message)}busy.set(false)});}} } button { class:"danger", r#type:"button", disabled:busy(), onclick:{let id=rental_id.clone();let media_id=media.media_id.clone();move |_|{if web_sys::window().and_then(|w|w.confirm_with_message("Remove this RV photo?").ok()).unwrap_or(false){let id=id.clone();let media_id=media_id.clone();busy.set(true);let _=spawn(async move{match api::delete_admin_rental_media(&id,&media_id).await{Ok(())=>refresh_rental_editor_after_change(&id,message,on_changed).await,Err(error)=>message.set(error.message)}busy.set(false)});}}}, "Remove" } } } } }
                        }
                        section { class:"admin-drawer-section", h3 { "Features & amenities" }
                            div { class:"admin-subentity-list", for feature in value.features.clone(){ article { key:"{feature.feature_id}", Icon{name:"circle-check",size:16,color:"currentColor"} div { strong { "{feature.label}" } small { "{feature.group_name} · {feature.description}" } } button { r#type:"button", disabled:busy(), onclick:{let feature=feature.clone();move |_|{if feature_draft_dirty()&&!web_sys::window().and_then(|w|w.confirm_with_message("Discard unsaved feature changes?").ok()).unwrap_or(false){return}feature_id.set(feature.feature_id.clone());feature_group.set(feature.group_name.clone());feature_icon.set(feature.icon_name.clone());feature_label.set(feature.label.clone());feature_description.set(feature.description.clone());feature_sort_order.set(feature.sort_order);feature_draft_dirty.set(false);aux_dirty.set(photo_draft_dirty()||addon_draft_dirty())}}, "Edit" } button { class:"danger", r#type:"button", disabled:busy(), onclick:{let id=rental_id.clone();let target_feature_id=feature.feature_id.clone();move |_|{if !web_sys::window().and_then(|w|w.confirm_with_message("Remove this feature? This cannot be undone.").ok()).unwrap_or(false){return}let id=id.clone();let target_feature_id=target_feature_id.clone();busy.set(true);let _=spawn(async move{match api::delete_admin_rental_feature(&id,&target_feature_id).await{Ok(())=>{if feature_id()==target_feature_id{feature_id.set(String::new());feature_label.set(String::new());feature_description.set(String::new());feature_draft_dirty.set(false);aux_dirty.set(photo_draft_dirty()||addon_draft_dirty())}refresh_rental_editor_after_change(&id,message,on_changed).await},Err(error)=>message.set(error.message)}busy.set(false)});}}, "Remove" } } } }
                            div { class:"admin-subentity-form", label { "Group" select { value:"{feature_group}", disabled:busy(), onchange:move|e|{feature_group.set(e.value());feature_draft_dirty.set(true);aux_dirty.set(true)}, option{value:"highlight","Highlight"} option{value:"amenity","Amenity"} } } label { "Icon" input { value:"{feature_icon}", disabled:busy(), oninput:move|e|{feature_icon.set(e.value());feature_draft_dirty.set(true);aux_dirty.set(true)} } } label { "Label" input { value:"{feature_label}", disabled:busy(), oninput:move|e|{feature_label.set(e.value());feature_draft_dirty.set(true);aux_dirty.set(true)} } } label { "Description" input { value:"{feature_description}", disabled:busy(), oninput:move|e|{feature_description.set(e.value());feature_draft_dirty.set(true);aux_dirty.set(true)} } } button { r#type:"button", disabled:busy()||feature_label().trim().is_empty(), onclick:{let id=rental_id.clone();move |_|{let edit=feature_id();let order=if edit.is_empty(){next_feature_order}else{feature_sort_order()};let payload=api::AdminFeaturePayload{group_name:feature_group(),icon_name:feature_icon(),label:feature_label(),description:feature_description(),sort_order:order};let id=id.clone();busy.set(true);let _=spawn(async move{let result=if edit.is_empty(){api::create_admin_rental_feature(&id,&payload).await}else{api::update_admin_rental_feature(&id,&edit,&payload).await};match result{Ok(_)=>{feature_id.set(String::new());feature_label.set(String::new());feature_description.set(String::new());feature_sort_order.set(next_feature_order+1);feature_draft_dirty.set(false);aux_dirty.set(photo_draft_dirty()||addon_draft_dirty());refresh_rental_editor_after_change(&id,message,on_changed).await},Err(error)=>message.set(error.message)}busy.set(false)});}}, if feature_id().is_empty(){"Add feature"}else{"Save feature"} } }
                        }
                        section { class:"admin-drawer-section", h3 { "RV-specific add-ons" }
                            div { class:"admin-subentity-list", for addon in value.addons.clone(){ article { key:"{addon.addon_id}", Icon{name:"sparkles",size:16,color:"currentColor"} div { strong { "{addon.label}" } small { "CA${addon.price} · " if addon.charge_type=="per_unit" { "per night" } else { "per booking" } " · " if addon.is_active { "Active" } else { "Disabled" } } } button { r#type:"button", disabled:busy(), onclick:{let addon=addon.clone();move |_|{if addon_draft_dirty()&&!web_sys::window().and_then(|w|w.confirm_with_message("Discard unsaved add-on changes?").ok()).unwrap_or(false){return}addon_id.set(addon.addon_id.clone());addon_label.set(addon.label.clone());addon_description.set(addon.description.clone());addon_icon.set(addon.icon_name.clone());addon_price.set(addon.price.clone());addon_charge.set(addon.charge_type.clone());addon_recommended.set(addon.is_recommended);addon_active.set(addon.is_active);addon_sort_order.set(addon.sort_order);addon_draft_dirty.set(false);aux_dirty.set(photo_draft_dirty()||feature_draft_dirty())}}, "Edit" } } } }
                            div { class:"admin-subentity-form", label { "Name" input { value:"{addon_label}", disabled:busy(), oninput:move|e|{addon_label.set(e.value());addon_draft_dirty.set(true);aux_dirty.set(true)} } } label { "Description" input { value:"{addon_description}", disabled:busy(), oninput:move|e|{addon_description.set(e.value());addon_draft_dirty.set(true);aux_dirty.set(true)} } } label { "Icon" input { value:"{addon_icon}", disabled:busy(), oninput:move|e|{addon_icon.set(e.value());addon_draft_dirty.set(true);aux_dirty.set(true)} } } label { "Price (CAD)" input { r#type:"number", min:"0.01", step:"0.01", value:"{addon_price}", disabled:busy(), oninput:move|e|{addon_price.set(e.value());addon_draft_dirty.set(true);aux_dirty.set(true)} } } label { "Charge" select { value:"{addon_charge}", disabled:busy(), onchange:move|e|{addon_charge.set(e.value());addon_draft_dirty.set(true);aux_dirty.set(true)}, option{value:"per_booking","Per booking"} option{value:"per_unit","Per night"} } } label { class:"admin-check-field", input { r#type:"checkbox", checked:addon_recommended(), disabled:busy(), onchange:move|e|{addon_recommended.set(e.checked());addon_draft_dirty.set(true);aux_dirty.set(true)} } "Recommended" } label { class:"admin-check-field", input { r#type:"checkbox", checked:addon_active(), disabled:busy(), onchange:move|e|{addon_active.set(e.checked());addon_draft_dirty.set(true);aux_dirty.set(true)} } "Active" } button { r#type:"button", disabled:busy()||addon_label().trim().is_empty(), onclick:{let id=rental_id.clone();move |_|{let edit=addon_id();let order=if edit.is_empty(){next_addon_order}else{addon_sort_order()};let payload=api::AdminAddonPayload{label:addon_label(),description:addon_description(),icon_name:addon_icon(),price:addon_price(),charge_type:addon_charge(),is_recommended:addon_recommended(),is_active:addon_active(),sort_order:order};let id=id.clone();busy.set(true);let _=spawn(async move{let result=if edit.is_empty(){api::create_admin_rental_addon(&id,&payload).await}else{api::update_admin_rental_addon(&id,&edit,&payload).await};match result{Ok(_)=>{addon_id.set(String::new());addon_label.set(String::new());addon_description.set(String::new());addon_icon.set("sparkles".into());addon_price.set("50".into());addon_charge.set("per_booking".into());addon_recommended.set(false);addon_active.set(true);addon_sort_order.set(next_addon_order+1);addon_draft_dirty.set(false);aux_dirty.set(photo_draft_dirty()||feature_draft_dirty());refresh_rental_editor_after_change(&id,message,on_changed).await},Err(error)=>message.set(error.message)}busy.set(false)});}}, if addon_id().is_empty(){"Add add-on"}else{"Save add-on"} } }
                        }
                    }
                }
                footer { class:"admin-drawer-actions", button { r#type:"button", disabled:busy()||!dirty(), onclick:move |_|save(), if busy(){"Saving…"}else{"Save changes"} }
                    if !is_new { button { r#type:"button", class:if published{"danger"}else{""}, disabled:busy()||dirty()||aux_dirty(), title:if dirty()||aux_dirty(){"Save or discard draft changes first"}else{""}, onclick:{let id=rental_id.clone();move |_|{let action=if published{"archive"}else{"publish"};let prompt=if published{"Archive this RV? Existing bookings remain unchanged."}else{"Publish this RV? It will immediately appear in booking."};if !web_sys::window().and_then(|w|w.confirm_with_message(prompt).ok()).unwrap_or(false){return}let id=id.clone();busy.set(true);let _=spawn(async move{match api::admin_rental_publication_action(&id,action).await{Ok(value)=>on_changed.call(value),Err(error)=>message.set(error.message)}busy.set(false)});}}, if published{"Archive RV"}else{"Publish RV"} } }
                }
        }
    }
}

#[component]
fn PaymentsTab(
    payments: Vec<api::AdminPaymentObligation>,
    bookings: Vec<api::AdminBooking>,
    loading: bool,
    on_open_booking: EventHandler<String>,
    on_refresh: EventHandler<()>,
) -> Element {
    let mut filter = use_signal(|| "all".to_string());
    let mut busy_id = use_signal(String::new);
    let mut refreshing_id = use_signal(String::new);
    let mut message = use_signal(String::new);
    let mut resend_confirmation = use_signal(|| None::<api::AdminPaymentObligation>);
    let filtered = payments
        .iter()
        .filter(|payment| {
            filter() == "all"
                || payment.status == filter()
                || (filter() == "attention"
                    && matches!(payment.status.as_str(), "failed" | "past_due" | "expired"))
        })
        .cloned()
        .collect::<Vec<_>>();
    let due_count = payments
        .iter()
        .filter(|payment| matches!(payment.status.as_str(), "due" | "past_due"))
        .count();
    let scheduled_count = payments
        .iter()
        .filter(|payment| payment.status == "scheduled")
        .count();
    let failed_count = payments
        .iter()
        .filter(|payment| matches!(payment.status.as_str(), "failed" | "expired"))
        .count();
    let paid_count = payments
        .iter()
        .filter(|payment| matches!(payment.status.as_str(), "succeeded" | "paid" | "released"))
        .count();
    rsx! {
        section { class: "admin-panel admin-full-panel",
            div { class: "admin-panel-head admin-list-head",
                div { h2 { "Payments" } p { "Initial payment, balance, refundable damage deposit and refunds." } }
                select { class: "admin-compact-filter", value: "{filter}", onchange: move |event| filter.set(event.value()),
                    option { value: "all", "All payment states" }
                    option { value: "attention", "Needs attention" }
                    option { value: "scheduled", "Scheduled" }
                    option { value: "link_created", "Link created" }
                    option { value: "due", "Due" }
                    option { value: "pending", "Operation pending" }
                    option { value: "submitted", "Sent to Stripe" }
                    option { value: "succeeded", "Paid" }
                    option { value: "failed", "Failed" }
                    option { value: "authorized", "Deposit authorized (legacy)" }
                    option { value: "released", "Released" }
                    option { value: "captured", "Damage captured" }
                    option { value: "cancelled", "Cancelled" }
                    option { value: "expired", "Expired" }
                }
            }
            div { class: "admin-payment-metrics",
                article { span { "DUE NOW" } strong { "{due_count}" } }
                article { span { "SCHEDULED" } strong { "{scheduled_count}" } }
                article { class: if failed_count > 0 { "needs-attention" } else { "" }, span { "NEEDS ATTENTION" } strong { "{failed_count}" } }
                article { span { "PAID / REFUNDED" } strong { "{paid_count}" } }
            }
            if !message.read().is_empty() { p { class: "admin-success admin-inline-message", "{message}" } }
            if loading { AdminLoading {} }
            else if filtered.is_empty() { div { class: "admin-empty", "No payment obligations match this filter." } }
            else { div { class: "admin-payment-list",
                for payment in filtered.iter() {
                    article { key: "{payment.payment_obligation_id}", class: "admin-payment-row",
                        button { class: "admin-payment-main", r#type: "button", onclick: { let id = payment.booking_id.clone(); move |_| on_open_booking.call(id.clone()) },
                            span { class: "admin-payment-type", if payment.payment_type == "damage_hold" { Icon { name: "shield", size: 17, color: "currentColor" } } else { Icon { name: "credit-card", size: 17, color: "currentColor" } } "{payment_label(&payment.payment_type)}" if payment.collection_method == "e_transfer" { small { "e-Transfer" } } }
                            span { strong { "{payment_booking_context(payment, &bookings).1}" } small { "{payment_booking_context(payment, &bookings).0}" } }
                            span { strong { "{display_money(&payment.currency, &payment.amount)}" } if let Some(due) = payment.due_at.as_ref() { small { "Due {display_moment(due)}" } } }
                            span { class: "admin-status admin-pay-{payment.status}", "{payment_label(&payment.status)}" }
                        }
                        div { class: "admin-payment-actions",
                            if can_resend_payment_link(payment) {
                                button { r#type: "button", disabled: busy_id() == payment.payment_obligation_id, onclick: { let payment = payment.clone(); move |_| { message.set(String::new()); resend_confirmation.set(Some(payment.clone())); } }, "Resend payment email" }
                            }
                            if payment.collection_method != "e_transfer" && payment.payment_id.is_some() && (!payment.financial_operation || payment.payment_type != "refund") {
                                button { r#type: "button", disabled: !refreshing_id().is_empty(), onclick: { let payment_id = payment.payment_id.clone().unwrap_or_default(); move |_| { let payment_id = payment_id.clone(); async move { refreshing_id.set(payment_id.clone()); message.set(String::new()); match api::refresh_admin_payment_status(&payment_id).await { Ok(_) => { message.set("Stripe status refreshed. Webhook and authenticated reconciliation remain the payment source of truth.".into()); on_refresh.call(()); }, Err(error) => message.set(error.message) } refreshing_id.set(String::new()); } } }, if refreshing_id() == payment.payment_id.clone().unwrap_or_default() { "Refreshing…" } else { "Refresh status" } }
                            }
                            button { r#type: "button", onclick: { let id = payment.booking_id.clone(); move |_| on_open_booking.call(id.clone()) }, "Details" }
                        }
                        if let Some(error) = payment.last_error_message.as_ref().or(payment.last_error.as_ref()) { p { class: "admin-payment-error", "{error}" } }
                    }
                }
            } }
        }
        if let Some(payment) = resend_confirmation.read().clone() {
            div { class: "admin-confirm-layer", onclick: move |_| if busy_id().is_empty() { resend_confirmation.set(None); },
                section { class: "admin-confirm-modal", role: "alertdialog", aria_modal: "true", aria_label: "Confirm payment-link resend", tabindex: "-1", autofocus: true, onclick: move |event| event.stop_propagation(), onkeydown: move |event| if event.key() == Key::Escape { event.stop_propagation(); if busy_id().is_empty() { resend_confirmation.set(None); } },
                    header { h3 { "Resend payment link?" } button { r#type: "button", disabled: !busy_id().is_empty(), aria_label: "Close resend confirmation", onclick: move |_| resend_confirmation.set(None), Icon { name: "x", size: 19, color: "currentColor" } } }
                    p { "The existing secure Stripe link for the {payment_label(&payment.payment_type)} of {display_money(&payment.currency, &payment.amount)} will be queued for email delivery. No new charge, obligation or Stripe object is created." }
                    if !message.read().is_empty() { p { class: "admin-error", role: "alert", "{message}" } }
                    footer { button { r#type: "button", disabled: !busy_id().is_empty(), onclick: move |_| resend_confirmation.set(None), "Go back" } button { class: "primary", r#type: "button", disabled: !busy_id().is_empty(), onclick: { let id = payment.payment_obligation_id.clone(); move |_| { let id = id.clone(); async move { busy_id.set(id.clone()); message.set(String::new()); match api::resend_admin_payment_link(&id).await { Ok(()) => { message.set("The payment email was queued for delivery.".into()); resend_confirmation.set(None); on_refresh.call(()); }, Err(error) => message.set(error.message) } busy_id.set(String::new()); } } }, if busy_id().is_empty() { "Queue resend" } else { "Queueing…" } } }
                }
            }
        }
    }
}

#[component]
fn CalendarSyncDrawer(
    rentals: Vec<api::Rental>,
    mut section_dirty: Signal<bool>,
    mut section_busy: Signal<bool>,
    on_close: EventHandler<()>,
    on_calendar_changed: EventHandler<()>,
) -> Element {
    let mut connections = use_signal(Vec::<api::AdminCalendarConnection>::new);
    let mut loading = use_signal(|| true);
    let mut message = use_signal(String::new);
    let mut reload_nonce = use_signal(|| 0_u32);
    let pending_actions = use_signal(|| 0_usize);
    let dirty_rows = use_signal(|| 0_usize);
    use_effect(move || section_busy.set(pending_actions() > 0));
    use_effect(move || section_dirty.set(dirty_rows() > 0));

    use_effect(move || {
        let reload = reload_nonce();
        loading.set(true);
        message.set(String::new());
        spawn(async move {
            let result = api::admin_calendar_connections().await;
            if *reload_nonce.peek() != reload {
                return;
            }
            match result {
                Ok(values) => connections.set(values),
                Err(error) => {
                    connections.set(Vec::new());
                    message.set(error.message);
                }
            }
            loading.set(false);
        });
    });
    let mut request_close = move || {
        if pending_actions() > 0 {
            return;
        }
        let may_close = dirty_rows() == 0
            || web_sys::window()
                .and_then(|window| {
                    window
                        .confirm_with_message("Discard unsaved calendar URL changes?")
                        .ok()
                })
                .unwrap_or(false);
        if may_close {
            section_dirty.set(false);
            on_close.call(());
        }
    };

    rsx! {
        div { class: "admin-overlay admin-drawer-backdrop", onclick: move |_| request_close(),
            aside {
                class: "admin-booking-drawer admin-calendar-sync-drawer",
                role: "dialog",
                aria_modal: "true",
                aria_label: "Calendar sync",
                tabindex: "-1",
                autofocus: true,
                onclick: move |event| event.stop_propagation(),
                onkeydown: move |event| if event.key() == Key::Escape { event.stop_propagation(); request_close(); },
                header { class: "admin-drawer-head",
                    div { h2 { "Calendar sync" } p { "RVezy and Outdoorsy · secure incoming feeds refresh every 15 minutes" } }
                    button { r#type: "button", disabled: pending_actions() > 0, aria_label: "Close calendar sync", onclick: move |_| request_close(), Icon { name: "x", size: 19, color: "currentColor" } }
                }
                div { class: "admin-drawer-scroll admin-calendar-sync-scroll",
                    section { class: "admin-calendar-sync-note",
                        Icon { name: "info", size: 18, color: "currentColor" }
                        p { "VL Rental securely imports only occupied dates. Customer names, phone numbers, addresses and prices never enter a calendar feed. Refresh VL now updates incoming feeds in VL; each provider refreshes the VL export on its own schedule." }
                    }
                    if !message.read().is_empty() { p { class: "admin-error", role: "alert", "{message}" } }
                    if loading() { AdminLoading {} }
                    else if !message.read().is_empty() {
                        div { class: "admin-empty",
                            p { "Calendar connections are unavailable, so sync actions are locked." }
                            button { r#type: "button", onclick: move |_| reload_nonce.set(reload_nonce().wrapping_add(1)), "Retry" }
                        }
                    }
                    else {
                        for rental in rentals.iter() {
                            section { key: "sync-{rental.slug}", class: "admin-calendar-sync-rental",
                                header { h3 { "{rental.name}" } small { "{rental.slug}" } }
                                details { class: "admin-calendar-sync-details",
                                    summary {
                                        span { "Manage calendar links" }
                                        small { "iCal" }
                                    }
                                    CalendarExportRow {
                                        key: "export-{rental.slug}",
                                        rental_slug: rental.slug.clone(),
                                        connection: connections.read().iter().find(|connection| connection.rental_slug == rental.slug && connection.provider == "vl").cloned(),
                                        pending_actions,
                                        on_changed: move |_| {
                                            reload_nonce.set(reload_nonce().wrapping_add(1));
                                        }
                                    }
                                    for provider in ["rvezy", "outdoorsy"] {
                                        CalendarSyncRow {
                                            key: "{rental.slug}-{provider}",
                                            rental_slug: rental.slug.clone(),
                                            rental_name: rental.name.clone(),
                                            provider: provider.to_string(),
                                            connection: connections.read().iter().find(|connection| connection.rental_slug == rental.slug && connection.provider == provider).cloned(),
                                            pending_actions,
                                            dirty_rows,
                                            on_changed: move |_| {
                                                reload_nonce.set(reload_nonce().wrapping_add(1));
                                                on_calendar_changed.call(());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn CalendarExportRow(
    rental_slug: String,
    connection: Option<api::AdminCalendarConnection>,
    mut pending_actions: Signal<usize>,
    on_changed: EventHandler<()>,
) -> Element {
    let mut busy = use_signal(|| false);
    let mut result_message = use_signal(String::new);
    let mut result_error = use_signal(|| false);
    let export_url = connection
        .as_ref()
        .and_then(|value| value.export_url.clone());
    let connection_id = connection
        .as_ref()
        .map(|value| value.external_calendar_id.clone())
        .unwrap_or_default();

    rsx! {
        article { class: "admin-calendar-sync-row admin-calendar-export-row",
            div { class: "admin-calendar-sync-row-head",
                div { span { class: "admin-calendar-provider-dot" } strong { "VL iCal link" } }
                span { class: "admin-calendar-connection-status status-connected", if export_url.is_some() { "Ready" } else { "Not created" } }
            }
            p { class: "admin-calendar-export-description", "Use this unique link on any other booking site to show this RV’s booked and blocked dates." }
            if let Some(url) = export_url.as_ref() {
                label { class: "admin-calendar-url-field",
                    span { "Private iCal URL" }
                    input { readonly: true, value: "{url}", aria_label: "VL iCal URL for this RV" }
                }
                div { class: "admin-calendar-sync-actions",
                    button {
                        class: "primary",
                        r#type: "button",
                        disabled: busy(),
                        onclick: {
                            let url = url.clone();
                            move |_| {
                                let url = url.clone();
                                async move {
                                    busy.set(true);
                                    result_message.set(String::new());
                                    match copy_calendar_url(&url).await {
                                        Ok(()) => { result_error.set(false); result_message.set("VL iCal link copied.".into()); }
                                        Err(error) => { result_error.set(true); result_message.set(error); }
                                    }
                                    busy.set(false);
                                }
                            }
                        },
                        "Copy link"
                    }
                    button {
                        r#type: "button",
                        disabled: busy(),
                        onclick: {
                            let id = connection_id.clone();
                            move |_| {
                                let id = id.clone();
                                async move {
                                    let confirmed = web_sys::window()
                                        .and_then(|window| {
                                            window
                                                .confirm_with_message(
                                                    "Create a new VL iCal link? The current link will stop working immediately.",
                                                )
                                                .ok()
                                        })
                                        .unwrap_or(false);
                                    if !confirmed {
                                        return;
                                    }
                                    busy.set(true);
                                    pending_actions.set(pending_actions().saturating_add(1));
                                    result_message.set(String::new());
                                    match api::rotate_admin_calendar_export_token(&id).await {
                                        Ok(_) => {
                                            result_error.set(false);
                                            result_message.set("New VL iCal link created. The old link no longer works.".into());
                                            on_changed.call(());
                                        }
                                        Err(error) => { result_error.set(true); result_message.set(error.message); }
                                    }
                                    pending_actions.set(pending_actions().saturating_sub(1));
                                    busy.set(false);
                                }
                            }
                        },
                        if busy() { "Creating…" } else { "Create new link" }
                    }
                }
            } else {
                div { class: "admin-calendar-sync-actions",
                    button {
                        class: "primary",
                        r#type: "button",
                        disabled: busy(),
                        onclick: {
                            let slug = rental_slug.clone();
                            move |_| {
                                let slug = slug.clone();
                                async move {
                                    busy.set(true);
                                    pending_actions.set(pending_actions().saturating_add(1));
                                    result_message.set(String::new());
                                    match api::create_admin_calendar_export(&slug).await {
                                        Ok(_) => {
                                            result_error.set(false);
                                            result_message.set("VL iCal link created. Copy it below after this section refreshes.".into());
                                            on_changed.call(());
                                        }
                                        Err(error) => { result_error.set(true); result_message.set(error.message); }
                                    }
                                    pending_actions.set(pending_actions().saturating_sub(1));
                                    busy.set(false);
                                }
                            }
                        },
                        if busy() { "Creating…" } else { "Create iCal link" }
                    }
                }
            }
            if !result_message.read().is_empty() { p { class: if result_error() { "admin-calendar-sync-result is-error" } else { "admin-calendar-sync-result" }, role: if result_error() { "alert" } else { "status" }, "{result_message}" } }
        }
    }
}

#[component]
fn CalendarSyncRow(
    rental_slug: String,
    rental_name: String,
    provider: String,
    connection: Option<api::AdminCalendarConnection>,
    mut pending_actions: Signal<usize>,
    mut dirty_rows: Signal<usize>,
    on_changed: EventHandler<()>,
) -> Element {
    let mut calendar_url = use_signal(String::new);
    let mut busy_action = use_signal(String::new);
    let mut result_message = use_signal(String::new);
    let mut result_error = use_signal(|| false);
    let mut disconnect_open = use_signal(|| false);
    let mut row_dirty = use_signal(|| false);
    let connection_id = connection
        .as_ref()
        .map(|value| value.external_calendar_id.clone())
        .unwrap_or_default();
    let status = connection
        .as_ref()
        .map(|value| value.status.as_str())
        .unwrap_or("needs_setup");
    let provider_label = calendar_provider_label(&provider);
    let setup_name = format!("VL Rental — {rental_name}");

    rsx! {
        article { class: "admin-calendar-sync-row provider-{provider}",
            div { class: "admin-calendar-sync-row-head",
                div { span { class: "admin-calendar-provider-dot" } strong { "{provider_label}" } }
                span { class: "admin-calendar-connection-status status-{status}", "{calendar_connection_status(status)}" }
            }
            if let Some(value) = connection.as_ref() {
                div { class: "admin-calendar-sync-meta",
                    span { "Last successful sync" strong { if let Some(last_sync) = value.last_synced_at.as_ref() { "{display_moment(last_sync)}" } else { "Not yet synced" } } }
                    span { "Imported periods" strong { "{value.imported_event_count}" } }
                    if value.conflict_count > 0 { span { class: "is-conflict", "Conflicts" strong { "{value.conflict_count}" } } }
                }
                if let Some(error) = value.last_error.as_ref() { p { class: "admin-calendar-sync-error", role: "alert", "{error}" } }
            }
            label { class: "admin-calendar-url-field",
                span { "{provider_label} Calendar URL" }
                input {
                    r#type: "url",
                    autocomplete: "off",
                    spellcheck: "false",
                    placeholder: if connection.as_ref().is_some_and(|value| value.has_external_url) { "Saved securely · paste a URL to replace it" } else { "Paste the provider iCal URL" },
                    value: "{calendar_url}",
                    disabled: !busy_action().is_empty(),
                    oninput: move |event| {
                        calendar_url.set(event.value());
                        result_message.set(String::new());
                        if !row_dirty() {
                            row_dirty.set(true);
                            dirty_rows.set(dirty_rows().saturating_add(1));
                        }
                    }
                }
            }
            div { class: "admin-calendar-sync-actions",
                button {
                    r#type: "button",
                    disabled: !busy_action().is_empty() || calendar_url().trim().is_empty(),
                    onclick: {
                        let slug = rental_slug.clone();
                        let provider = provider.clone();
                        move |_| {
                            let slug = slug.clone();
                            let provider = provider.clone();
                            let url = calendar_url();
                            async move {
                                pending_actions.set(pending_actions().saturating_add(1));
                                busy_action.set("test".into());
                                result_message.set(String::new());
                                match api::test_admin_calendar(&slug, &provider, url.trim()).await {
                                    Ok(result) => {
                                        result_error.set(result.conservative_fallbacks > 0);
                                        result_message.set(if result.conservative_fallbacks > 0 {
                                            format!("Calendar is valid, but {} event(s) could not be matched to one RV. VL will block those dates for every RV until the listing name is corrected.", result.conservative_fallbacks)
                                        } else {
                                            format!("Calendar is valid · {} current periods found", result.imported_periods)
                                        });
                                    }
                                    Err(error) => { result_error.set(true); result_message.set(error.message); }
                                }
                                busy_action.set(String::new());
                                pending_actions.set(pending_actions().saturating_sub(1));
                            }
                        }
                    },
                    if busy_action() == "test" { "Testing…" } else { "Test calendar" }
                }
                button {
                    class: "primary",
                    r#type: "button",
                    disabled: !busy_action().is_empty() || calendar_url().trim().is_empty(),
                    onclick: {
                        let slug = rental_slug.clone();
                        let provider = provider.clone();
                        let id = connection_id.clone();
                        move |_| {
                            let slug = slug.clone();
                            let provider = provider.clone();
                            let id = id.clone();
                            let url = calendar_url();
                            async move {
                                pending_actions.set(pending_actions().saturating_add(1));
                                busy_action.set("save".into());
                                result_message.set(String::new());
                                let saved = if id.is_empty() {
                                    api::save_admin_calendar(&slug, &provider, url.trim()).await.map(|connection| connection.status)
                                } else {
                                    match api::update_admin_calendar(&id, Some(url.trim()), Some(true), Some(true)).await {
                                        Ok(_) => api::sync_admin_calendar(&id).await.map(|result| result.status),
                                        Err(error) => Err(error),
                                    }
                                };
                                match saved {
                                    Ok(status) => {
                                        let needs_attention = matches!(status.as_str(), "error" | "conflict" | "suspicious_empty");
                                        result_error.set(needs_attention);
                                        result_message.set(if needs_attention { "Saved, but the first VL sync needs attention.".into() } else { "Saved and synced with VL Rental.".into() });
                                        calendar_url.set(String::new());
                                        if row_dirty() {
                                            row_dirty.set(false);
                                            dirty_rows.set(dirty_rows().saturating_sub(1));
                                        }
                                        on_changed.call(());
                                    }
                                    Err(error) => { result_error.set(true); result_message.set(error.message); }
                                }
                                busy_action.set(String::new());
                                pending_actions.set(pending_actions().saturating_sub(1));
                            }
                        }
                    },
                    if busy_action() == "save" { "Saving…" } else { "Save & sync" }
                }
                if !connection_id.is_empty() {
                    button {
                        r#type: "button",
                        disabled: !busy_action().is_empty(),
                        onclick: {
                            let id = connection_id.clone();
                            move |_| {
                                let id = id.clone();
                                async move {
                                    pending_actions.set(pending_actions().saturating_add(1));
                                    busy_action.set("sync".into());
                                    result_message.set(String::new());
                                    match api::sync_admin_calendar(&id).await {
                                        Ok(result) => {
                                            let needs_attention = result.conflict_count > 0
                                                || matches!(result.status.as_str(), "error" | "suspicious_empty");
                                            result_error.set(needs_attention);
                                            result_message.set(if needs_attention {
                                                format!("VL refreshed, but needs attention · {} periods · {} conflicts", result.imported_blocks, result.conflict_count)
                                            } else {
                                                format!("VL refreshed · {} periods · {} conflicts", result.imported_blocks, result.conflict_count)
                                            });
                                            on_changed.call(());
                                        }
                                        Err(error) => { result_error.set(true); result_message.set(error.message); }
                                    }
                                    busy_action.set(String::new());
                                    pending_actions.set(pending_actions().saturating_sub(1));
                                }
                            }
                        },
                        if busy_action() == "sync" { "Refreshing…" } else { "Refresh VL now" }
                    }
                }
                if let Some(export_url) = connection.as_ref().and_then(|value| value.export_url.clone()) {
                    button {
                        r#type: "button",
                        disabled: !busy_action().is_empty(),
                        onclick: move |_| {
                            let url = export_url.clone();
                            async move {
                                match copy_calendar_url(&url).await {
                                    Ok(()) => { result_error.set(false); result_message.set("VL Calendar URL copied.".into()); }
                                    Err(error) => { result_error.set(true); result_message.set(error); }
                                }
                            }
                        },
                        "Copy VL URL"
                    }
                }
            }
            if !result_message.read().is_empty() { p { class: if result_error() { "admin-calendar-sync-result is-error" } else { "admin-calendar-sync-result" }, role: if result_error() { "alert" } else { "status" }, "{result_message}" } }
            div { class: "admin-calendar-setup-steps",
                p { strong { "1" } span { "In {provider_label}, copy this RV’s calendar URL and paste it above." } }
                p { strong { "2" } span { "Copy the VL URL, open {provider_label} → Add Calendar, paste it, and name it " code { "{setup_name}" } "." } }
                small {
                    if provider == "outdoorsy" {
                        "Outdoorsy bookings reach VL through VL’s secure calendar relay, without passing through RVezy. VL checks the Outdoorsy feed every 15 minutes; Outdoorsy refreshes the VL export on its own schedule."
                    } else {
                        "RVezy bookings are imported directly into VL. VL checks the RVezy feed every 15 minutes; RVezy refreshes the VL export on its own schedule."
                    }
                }
            }
            if !connection_id.is_empty() {
                button { class: "admin-calendar-disconnect", r#type: "button", disabled: !busy_action().is_empty(), onclick: move |_| { result_message.set(String::new()); result_error.set(false); disconnect_open.set(true); }, "Disconnect calendar" }
            }
        }
        if disconnect_open() {
            div { class: "admin-confirm-layer", onclick: move |_| if busy_action().is_empty() { disconnect_open.set(false); },
                section { class: "admin-confirm-modal", role: "alertdialog", aria_modal: "true", aria_label: "Disconnect calendar", tabindex: "-1", autofocus: true, onclick: move |event| event.stop_propagation(), onkeydown: move |event| if event.key() == Key::Escape { event.stop_propagation(); if busy_action().is_empty() { disconnect_open.set(false); } },
                    header { h3 { "Disconnect {provider_label}?" } button { r#type: "button", disabled: !busy_action().is_empty(), aria_label: "Close disconnect confirmation", onclick: move |_| disconnect_open.set(false), Icon { name: "x", size: 19, color: "currentColor" } } }
                    p { "Imported {provider_label} periods for {rental_name} will be removed and its VL export URL will be revoked. This does not change bookings on {provider_label}." }
                    if !result_message.read().is_empty() { p { class: "admin-error", role: "alert", "{result_message}" } }
                    footer {
                        button { r#type: "button", disabled: !busy_action().is_empty(), onclick: move |_| disconnect_open.set(false), "Go back" }
                        button { class: "danger", r#type: "button", disabled: !busy_action().is_empty(), onclick: { let id = connection_id.clone(); move |_| { let id = id.clone(); async move { pending_actions.set(pending_actions().saturating_add(1)); busy_action.set("disconnect".into()); match api::disconnect_admin_calendar(&id).await { Ok(()) => { calendar_url.set(String::new()); if row_dirty() { row_dirty.set(false); dirty_rows.set(dirty_rows().saturating_sub(1)); } disconnect_open.set(false); on_changed.call(()); }, Err(error) => { result_error.set(true); result_message.set(error.message); } } busy_action.set(String::new()); pending_actions.set(pending_actions().saturating_sub(1)); } } }, if busy_action() == "disconnect" { "Disconnecting…" } else { "Disconnect" } }
                    }
                }
            }
        }
    }
}

#[component]
fn CalendarTab(
    bookings: Vec<api::AdminBooking>,
    blocks: Vec<api::AdminAvailabilityBlock>,
    rentals: Vec<api::Rental>,
    mut editor_dirty: Signal<bool>,
    mut busy: Signal<bool>,
    on_open_booking: EventHandler<String>,
    on_refresh: EventHandler<()>,
) -> Element {
    let today = Utc::now().with_timezone(&Vancouver).date_naive();
    let mut window_start = use_signal(|| today);
    let mut fleet_filter = use_signal(|| "all".to_string());
    let mut panel_open = use_signal(|| false);
    let mut sync_open = use_signal(|| false);
    let mut selected_block = use_signal(|| None::<api::AdminAvailabilityBlock>);
    let mut rental_slug = use_signal(|| {
        rentals
            .first()
            .map(|rental| rental.slug.clone())
            .unwrap_or_default()
    });
    let mut starts_on = use_signal(|| (today + Duration::days(1)).to_string());
    let mut ends_on = use_signal(|| (today + Duration::days(4)).to_string());
    let mut reason = use_signal(|| "Owner use".to_string());
    let mut message = use_signal(String::new);
    let can_leave_editor = move || {
        !busy()
            && (!editor_dirty()
                || web_sys::window()
                    .and_then(|window| {
                        window
                            .confirm_with_message("Discard unsaved calendar changes?")
                            .ok()
                    })
                    .unwrap_or(false))
    };
    let mut close_editor = move || {
        if can_leave_editor() {
            panel_open.set(false);
            editor_dirty.set(false);
        }
    };
    let days = (0..14)
        .map(|offset| *window_start.read() + Duration::days(offset))
        .collect::<Vec<_>>();
    let visible_rentals = rentals
        .iter()
        .filter(|rental| fleet_filter() == "all" || rental.slug == fleet_filter())
        .cloned()
        .collect::<Vec<_>>();
    let window_end = days.last().copied().unwrap_or(*window_start.read());
    let mut schedule_bookings = bookings
        .iter()
        .filter(|booking| {
            !matches!(booking.status.as_str(), "cancelled" | "expired")
                && (fleet_filter() == "all" || booking.rental_slug == fleet_filter())
                && calendar_interval_overlaps_window(
                    &booking.starts_at,
                    &booking.ends_at,
                    *window_start.read(),
                    window_end,
                )
        })
        .cloned()
        .collect::<Vec<_>>();
    schedule_bookings.sort_by(|left, right| left.starts_at.cmp(&right.starts_at));
    let mut schedule_blocks = blocks
        .iter()
        .filter(|block| {
            (fleet_filter() == "all" || block.rental_slug == fleet_filter())
                && calendar_interval_overlaps_window(
                    &block.starts_at,
                    &block.ends_at,
                    *window_start.read(),
                    window_end,
                )
        })
        .cloned()
        .collect::<Vec<_>>();
    schedule_blocks.sort_by(|left, right| left.starts_at.cmp(&right.starts_at));
    rsx! {
        div { class: "admin-calendar-layout",
            section { class: "admin-panel admin-calendar-panel",
                div { class: "admin-panel-head admin-calendar-head", div { h2 { "Fleet calendar" } p { "Return at 11:00 AM and the next delivery at 2:00 PM can share one date." } } div { class: "admin-calendar-actions",
                    select { class: "admin-compact-filter", aria_label: "Filter calendar by RV", value: "{fleet_filter}", onchange: move |event| fleet_filter.set(event.value()), option { value: "all", "All RVs" } for rental in rentals.iter() { option { value: "{rental.slug}", "{rental.name}" } } }
                    button { r#type: "button", aria_label: "Previous two weeks", onclick: move |_| { let current = *window_start.read(); window_start.set(current - Duration::days(14)); }, Icon { name: "chevron-left", size: 16, color: "currentColor" } }
                    button { r#type: "button", onclick: move |_| window_start.set(today), "Today" }
                    button { r#type: "button", aria_label: "Next two weeks", onclick: move |_| { let current = *window_start.read(); window_start.set(current + Duration::days(14)); }, Icon { name: "chevron-right", size: 16, color: "currentColor" } }
                    button { class: "admin-calendar-sync-open", r#type: "button", disabled: busy(), onclick: move |_| { if panel_open() && !can_leave_editor() { return; } selected_block.set(None); panel_open.set(false); editor_dirty.set(false); sync_open.set(true); }, Icon { name: "refresh-cw", size: 15, color: "currentColor" } "Calendar sync" }
                    button { class: "admin-primary-small", r#type: "button", disabled: busy(), onclick: move |_| { if panel_open() { return; } selected_block.set(None); editor_dirty.set(false); message.set(String::new()); panel_open.set(true); }, Icon { name: "calendar-off", size: 15, color: "currentColor" } "Close dates" }
                } }
                div { class: "admin-calendar-legend", span { class: "booking", "VL booking" } span { class: "block", "Closed by admin" } span { class: "rvezy", "RVezy" } span { class: "outdoorsy", "Outdoorsy" } span { class: "conflict", "Conflict" } small { "Click an item for details · + closes an open day" } }
                div { class: "admin-fleet-scroll",
                    div { class: "admin-fleet-calendar", style: "--admin-calendar-days: {days.len()}",
                        div { class: "admin-fleet-corner", strong { "RV / DATE" } small { "{admin_calendar_short_date(*window_start.read())} – {admin_calendar_short_date(window_end)}" } }
                        for day in days.iter() { div { key: "head-{day}", class: if *day == today { "admin-fleet-day is-today" } else { "admin-fleet-day" }, span { "{admin_calendar_weekday(*day)}" } strong { "{admin_calendar_day_number(*day)}" } } }
                        for rental in visible_rentals.iter() {
                            button { key: "rental-{rental.slug}", class: "admin-fleet-rental", r#type: "button", aria_label: "Show only {rental.name}", title: "Show only this RV", onclick: { let slug = rental.slug.clone(); move |_| fleet_filter.set(slug.clone()) }, strong { "{rental.name}" } small { "Delivery only · click to focus" } }
                            for day in days.iter() { div { key: "cell-{rental.slug}-{day}", class: if *day == today { "admin-fleet-cell is-today" } else { "admin-fleet-cell" },
                                for booking in bookings.iter().filter(|booking| booking.rental_slug == rental.slug && booking_occupies_day(booking, *day)) {
                                    button { class: "admin-calendar-chip booking", r#type: "button", title: "{booking.first_name} {booking.last_name} · {booking.booking_number}", onclick: { let id = booking.booking_id.clone(); move |_| on_open_booking.call(id.clone()) },
                                        if admin_calendar_date(&booking.ends_at) == Some(*day) { small { "11 AM" } }
                                        if admin_calendar_date(&booking.starts_at) == Some(*day) { small { "2 PM" } }
                                        span { "{booking.last_name}" }
                                    }
                                }
                                for group in aggregate_external_calendar_blocks(blocks.iter().filter(|block| block.rental_slug == rental.slug && block_occupies_day(block, *day))) {
                                    button { class: "admin-calendar-chip {group.css_class}", r#type: "button", disabled: busy(), title: "{group.reason} · {group.count} external periods · open calendar sync", aria_label: if group.key == "conflict" { format!("Review {} conflicts for {}", group.count, rental.name) } else { format!("Manage {} calendar for {}", group.label, rental.name) }, onclick: move |_| { if panel_open() && !can_leave_editor() { return; } selected_block.set(None); panel_open.set(false); editor_dirty.set(false); sync_open.set(true); }, small { "{group.label}" } span { "{group.reason}" } if group.count > 1 { small { class: "admin-calendar-chip-count", "+{group.count - 1} periods" } } }
                                }
                                for block in blocks.iter().filter(|block| block.rental_slug == rental.slug && block_occupies_day(block, *day) && !is_external_calendar_block(block)) {
                                    button { class: "admin-calendar-chip {calendar_block_class(block)}", r#type: "button", disabled: busy(), title: "{block.reason} · view details", aria_label: "View closed dates for {rental.name}", onclick: { let block = block.clone(); move |_| { if panel_open() && !can_leave_editor() { return; } panel_open.set(false); editor_dirty.set(false); message.set(String::new()); selected_block.set(Some(block.clone())); } }, small { "CLOSED" } span { "{block.reason}" } }
                                }
                                if *day >= today && !bookings.iter().any(|booking| booking.rental_slug == rental.slug && booking_occupies_day(booking, *day)) && !blocks.iter().any(|block| block.rental_slug == rental.slug && block_occupies_day(block, *day)) {
                                    button { class: "admin-calendar-cell-close", r#type: "button", disabled: busy(), title: "Close dates starting here", aria_label: "Close {rental.name} starting {day}", onclick: { let slug = rental.slug.clone(); let day = *day; move |_| { if panel_open() && !can_leave_editor() { return; } selected_block.set(None); rental_slug.set(slug.clone()); starts_on.set(day.to_string()); ends_on.set((day + Duration::days(3)).to_string()); reason.set("Owner use".into()); editor_dirty.set(false); message.set(String::new()); panel_open.set(true); } }, Icon { name: "plus", size: 12, color: "currentColor" } span { "Close" } }
                                }
                            } }
                        }
                    }
                }
                div { class: "admin-calendar-agenda",
                    h3 { "Schedule · {admin_calendar_short_date(*window_start.read())}–{admin_calendar_short_date(window_end)}" }
                    if schedule_bookings.is_empty() && schedule_blocks.is_empty() { div { class: "admin-empty", "No bookings or closed dates in this two-week window." } }
                    for booking in schedule_bookings.iter() { button { class: "booking", r#type: "button", onclick: { let id = booking.booking_id.clone(); move |_| on_open_booking.call(id.clone()) }, div { time { "{display_date(&booking.starts_at)} 2 PM → {display_date(&booking.ends_at)} 11 AM" } strong { "{booking.rental_name}" } span { "{booking.first_name} {booking.last_name} · {booking.booking_number}" } } span { class: "admin-status admin-status-{booking.status}", "{status_label(&booking.status)}" } } }
                    for block in schedule_blocks.iter() { article { class: "{calendar_block_class(block)}", div { time { "{display_date(&block.starts_at)} 2 PM → {display_date(&block.ends_at)} 11 AM" } strong { "{block.rental_name}" } span { "{block.reason}" if let Some(provider) = calendar_block_provider(block) { " · {calendar_provider_label(provider)}" } } } if is_external_calendar_block(block) { button { class: "manage-external", r#type: "button", disabled: busy(), onclick: move |_| { if panel_open() && !can_leave_editor() { return; } selected_block.set(None); panel_open.set(false); editor_dirty.set(false); sync_open.set(true); }, if block.has_conflict { "Review conflict" } else { "Manage sync" } } } else { button { r#type: "button", disabled: busy(), onclick: { let id = block.availability_block_id.clone(); move |_| { let id = id.clone(); async move { busy.set(true); message.set(String::new()); match api::delete_admin_availability_block(&id).await { Ok(()) => { selected_block.set(None); message.set("Dates reopened for customers.".into()); on_refresh.call(()); }, Err(error) => message.set(error.message) } busy.set(false); } } }, "Reopen" } } } }
                }
                if !message.read().is_empty() { p { class: "admin-inline-message", "{message}" } }
            }
            if let Some(block) = selected_block.read().clone() {
                aside { class: "admin-panel admin-calendar-editor admin-calendar-block-detail", tabindex: "-1", autofocus: true, onkeydown: move |event| if event.key() == Key::Escape { event.stop_propagation(); if !busy() { selected_block.set(None); } },
                    header { div { h3 { "Closed dates" } p { "Local admin block · customers cannot book this RV." } } button { r#type: "button", disabled: busy(), aria_label: "Close block details", onclick: move |_| if !busy() { selected_block.set(None); }, Icon { name: "x", size: 19, color: "currentColor" } } }
                    dl {
                        div { dt { "RV" } dd { "{block.rental_name}" } }
                        div { dt { "Closed from" } dd { "{display_date(&block.starts_at)} · 2:00 PM" } }
                        div { dt { "Open again" } dd { "{display_date(&block.ends_at)} · 11:00 AM" } }
                        div { dt { "Reason" } dd { "{block.reason}" } }
                    }
                    p { class: "admin-calendar-block-note", "Reopening removes only this local block. External dates must be changed on their original platform." }
                    if !message.read().is_empty() { p { class: "admin-error", role: "alert", "{message}" } }
                    button { class: "admin-submit danger", r#type: "button", disabled: busy(), onclick: { let id = block.availability_block_id.clone(); move |_| { let id = id.clone(); async move { busy.set(true); message.set(String::new()); match api::delete_admin_availability_block(&id).await { Ok(()) => { selected_block.set(None); message.set("Dates reopened for customers.".into()); on_refresh.call(()); }, Err(error) => message.set(error.message) } busy.set(false); } } }, if busy() { "Reopening…" } else { "Reopen these dates" } }
                }
            } else if panel_open() {
                aside { class: "admin-panel admin-calendar-editor", tabindex: "-1", autofocus: true, onkeydown: move |event| if event.key() == Key::Escape { event.stop_propagation(); close_editor(); },
                    header { div { h3 { "Close dates" } p { "The RV becomes unavailable in customer search." } } button { r#type: "button", disabled: busy(), aria_label: "Close date editor", onclick: move |_| close_editor(), Icon { name: "x", size: 19, color: "currentColor" } } }
                    label { "RV" select { value: "{rental_slug}", disabled: busy(), onchange: move |event| { rental_slug.set(event.value()); editor_dirty.set(true); }, for rental in rentals.iter() { option { value: "{rental.slug}", "{rental.name}" } } } }
                    div { class: "admin-date-grid", label { "Close from · 2:00 PM" input { r#type: "date", min: "{today}", value: "{starts_on}", disabled: busy(), onchange: move |event| { starts_on.set(event.value()); editor_dirty.set(true); } } } label { "Open again · 11:00 AM" input { r#type: "date", min: "{today}", value: "{ends_on}", disabled: busy(), onchange: move |event| { ends_on.set(event.value()); editor_dirty.set(true); } } } }
                    label { "Reason (admin only)" input { value: "{reason}", maxlength: 300, disabled: busy(), oninput: move |event| { reason.set(event.value()); editor_dirty.set(true); } } }
                    if !message.read().is_empty() { p { class: "admin-error", role: "alert", "{message}" } }
                    button { class: "admin-submit", r#type: "button", disabled: busy() || rental_slug().is_empty(), onclick: move |_| { let values = (rental_slug(), starts_on(), ends_on(), reason()); async move { if values.2 <= values.1 { message.set("The reopening date must be later than the closing date.".into()); return; } busy.set(true); message.set(String::new()); match api::create_admin_availability_block(&values.0, &values.1, &values.2, &values.3).await { Ok(_) => { editor_dirty.set(false); message.set("Dates closed for customers.".into()); panel_open.set(false); on_refresh.call(()); }, Err(error) => message.set(error.message) } busy.set(false); } }, if busy() { "Saving…" } else { "Close dates" } }
                }
            }
        }
        if sync_open() {
            CalendarSyncDrawer {
                rentals: rentals.clone(),
                section_dirty: editor_dirty,
                section_busy: busy,
                on_close: move |_| sync_open.set(false),
                on_calendar_changed: move |_| on_refresh.call(()),
            }
        }
    }
}

#[component]
fn ReviewsTab(rentals: Vec<api::AdminRentalSummary>) -> Element {
    let mut reviews = use_signal(Vec::<api::AdminRentalReview>::new);
    let mut search = use_signal(String::new);
    let mut rental_filter = use_signal(String::new);
    let mut offset = use_signal(|| 0_i64);
    let mut next_offset = use_signal(|| None::<i64>);
    let mut loading = use_signal(|| true);
    let mut message = use_signal(String::new);
    let mut delete_target = use_signal(|| None::<api::AdminRentalReview>);
    let mut delete_busy = use_signal(|| false);
    let mut reload_nonce = use_signal(|| 0_u32);

    use_effect(move || {
        let query = search();
        let rental = rental_filter();
        let page_offset = offset();
        let reload = reload_nonce();
        loading.set(true);
        message.set(String::new());
        spawn(async move {
            match api::admin_rental_reviews(&query, &rental, page_offset).await {
                Ok(value) => {
                    if search.peek().as_str() == query.as_str()
                        && rental_filter.peek().as_str() == rental.as_str()
                        && *offset.peek() == page_offset
                        && *reload_nonce.peek() == reload
                    {
                        if value.reviews.is_empty() && page_offset > 0 {
                            offset.set((page_offset - 50).max(0));
                            return;
                        }
                        reviews.set(value.reviews);
                        next_offset.set(value.next_offset);
                    }
                }
                Err(error)
                    if search.peek().as_str() == query.as_str()
                        && rental_filter.peek().as_str() == rental.as_str()
                        && *offset.peek() == page_offset
                        && *reload_nonce.peek() == reload =>
                {
                    reviews.set(Vec::new());
                    next_offset.set(None);
                    message.set(error.message)
                }
                Err(_) => {}
            }
            if search.peek().as_str() == query.as_str()
                && rental_filter.peek().as_str() == rental.as_str()
                && *offset.peek() == page_offset
                && *reload_nonce.peek() == reload
            {
                loading.set(false);
            }
        });
    });

    rsx! {
        section { class: "admin-panel admin-full-panel admin-reviews-panel",
            div { class: "admin-panel-head admin-list-head",
                div { h2 { "Reviews" } p { "VL Rental and trusted-platform guest comments. Deletion is permanent." } }
                div { class: "admin-audit-tools",
                    label { class: "admin-search", Icon { name: "search", size: 16, color: "var(--vl-muted)" } input { r#type: "search", maxlength: "100", placeholder: "Guest, booking or comment", value: "{search}", oninput: move |event| { offset.set(0); search.set(event.value()); } } }
                    select { class: "admin-compact-filter", aria_label: "Filter reviews by RV", value: "{rental_filter}", onchange: move |event| { offset.set(0); rental_filter.set(event.value()); }, option { value: "", "All RVs" } for rental in rentals.iter() { option { value: "{rental.slug}", "{rental.name}" } } }
                }
            }
            if !message.read().is_empty() { p { class: "admin-inline-message", role: "alert", "{message}" } }
            if loading() { AdminLoading {} }
            else if !message.read().is_empty() {
                div { class: "admin-empty",
                    p { "Reviews are unavailable, so delete actions are locked." }
                    button { r#type: "button", onclick: move |_| reload_nonce.set(reload_nonce().wrapping_add(1)), "Retry" }
                }
            }
            else if reviews.read().is_empty() { div { class: "admin-empty", "No reviews match this search." } }
            else {
                div { class: "admin-table-wrap admin-reviews-table-wrap", table { class: "admin-bookings-table admin-reviews-table",
                    thead { tr { th { "Guest / booking" } th { "RV" } th { "Rating" } th { "Comment" } th { "Likes" } th { "Published" } th { "" } } }
                    tbody { for review in reviews.read().iter() { tr { key: "{review.rental_review_id}",
                        td { strong { "{review.reviewer_name}" } small { "{review.booking_number}" } }
                        td { "{review.rental_name}" }
                        td { span { class: "admin-review-rating", "★ {review.rating}/5" } }
                        td { if !review.title.is_empty() { strong { "{review.title}" } } p { class: "admin-review-body", "{review.body}" } }
                        td { "♥ {review.like_count}" }
                        td { if review.reviewed_at_label.is_empty() { "{display_moment(&review.created_at)}" } else { "{review.reviewed_at_label}" } }
                        td { button { class: "admin-review-delete", r#type: "button", aria_label: "Delete review by {review.reviewer_name}", onclick: { let review = review.clone(); move |_| { message.set(String::new()); delete_target.set(Some(review.clone())); } }, Icon { name: "trash-2", size: 15, color: "currentColor" } "Delete" } }
                    } } }
                } }
                div { class: "admin-review-pagination",
                    button { r#type: "button", disabled: offset() == 0 || loading(), onclick: move |_| offset.set((offset() - 50).max(0)), "Previous" }
                    span { "Showing {offset() + 1}–{offset() + reviews.read().len() as i64}" }
                    button { r#type: "button", disabled: next_offset.read().is_none() || loading(), onclick: move |_| if let Some(next) = *next_offset.read() { offset.set(next); }, "Next" }
                }
            }
        }
        if let Some(target) = delete_target.read().clone() {
            div { class: "admin-overlay admin-modal-backdrop", role: "presentation", onclick: move |_| if !delete_busy() { message.set(String::new()); delete_target.set(None); },
                section { class: "admin-confirm-modal", role: "alertdialog", aria_modal: "true", aria_label: "Delete review permanently", tabindex: "-1", autofocus: true, onclick: move |event| event.stop_propagation(), onkeydown: move |event| if event.key() == Key::Escape { event.stop_propagation(); if !delete_busy() { message.set(String::new()); delete_target.set(None); } },
                    header { h3 { "Delete review permanently?" } button { r#type: "button", aria_label: "Close delete confirmation", disabled: delete_busy(), onclick: move |_| { message.set(String::new()); delete_target.set(None); }, Icon { name: "x", size: 18, color: "currentColor" } } }
                    p { "The review and all of its likes will be removed. The customer cannot publish another review for this booking." }
                    p { class: "admin-action-limit", "This cannot be undone." }
                    if !message.read().is_empty() { p { class: "admin-error", role: "alert", "{message}" } }
                    footer {
                        button { r#type: "button", disabled: delete_busy(), onclick: move |_| { message.set(String::new()); delete_target.set(None); }, "Cancel" }
                        button { class: "danger", r#type: "button", disabled: delete_busy(), onclick: { let review_id = target.rental_review_id.clone(); move |_| { let review_id = review_id.clone(); async move { delete_busy.set(true); message.set(String::new()); match api::admin_delete_rental_review(&review_id).await { Ok(()) => { delete_target.set(None); reload_nonce.set(reload_nonce().wrapping_add(1)); }, Err(error) => message.set(error.message) } delete_busy.set(false); } } }, if delete_busy() { "Deleting…" } else { "Delete permanently" } }
                    }
                }
            }
        }
    }
}

#[component]
fn AuditTab(events: Vec<api::AdminAuditEvent>, loading: bool) -> Element {
    let mut search = use_signal(String::new);
    let mut action_filter = use_signal(|| "all".to_string());
    let mut from_date = use_signal(String::new);
    let mut to_date = use_signal(String::new);
    let mut exporting = use_signal(|| false);
    let mut message = use_signal(String::new);
    let query = search().trim().to_lowercase();
    let mut actions = events
        .iter()
        .map(|event| event.action.clone())
        .filter(|action| !action.is_empty())
        .collect::<Vec<_>>();
    actions.sort();
    actions.dedup();
    let filtered = events
        .iter()
        .filter(|event| action_filter() == "all" || event.action == action_filter())
        .filter(|event| {
            let event_date = event.created_at.get(0..10).unwrap_or_default();
            (from_date().is_empty() || event_date >= from_date().as_str())
                && (to_date().is_empty() || event_date <= to_date().as_str())
        })
        .filter(|event| {
            query.is_empty()
                || format!(
                    "{} {} {} {}",
                    event.action,
                    event.actor_email.clone() + &event.actor_user_id.clone().unwrap_or_default(),
                    event.summary.clone() + &event.reason,
                    event.booking_number.clone().unwrap_or_default()
                )
                .to_lowercase()
                .contains(&query)
        })
        .cloned()
        .collect::<Vec<_>>();
    rsx! {
        section { class: "admin-panel admin-full-panel",
            div { class: "admin-panel-head admin-list-head",
                div { h2 { "Audit log" } p { "Immutable financial and administrative history." } }
                div { class: "admin-audit-tools",
                    label { class: "admin-search", Icon { name: "search", size: 16, color: "var(--vl-muted)" } input { r#type: "search", placeholder: "Actor, action or booking", value: "{search}", oninput: move |event| search.set(event.value()) } }
                    select { class: "admin-compact-filter", aria_label: "Filter audit log by action", value: "{action_filter}", onchange: move |event| action_filter.set(event.value()), option { value: "all", "All actions" } for action in actions.iter() { option { value: "{action}", "{payment_label(action)}" } } }
                    input { class: "admin-compact-filter", aria_label: "Audit events from date", r#type: "date", value: "{from_date}", onchange: move |event| from_date.set(event.value()) }
                    input { class: "admin-compact-filter", aria_label: "Audit events through date", r#type: "date", value: "{to_date}", onchange: move |event| to_date.set(event.value()) }
                    button { class: "admin-primary-small", r#type: "button", disabled: exporting(), onclick: move |_| async move { exporting.set(true); message.set(String::new()); match api::admin_audit_csv().await { Ok(csv) => match api::download_csv("vl-rental-audit.csv", &csv) { Ok(()) => message.set("CSV export downloaded.".into()), Err(error) => message.set(error) }, Err(error) => message.set(error.message) } exporting.set(false); }, Icon { name: "download", size: 15, color: "currentColor" } if exporting() { "Preparing…" } else { "Export CSV" } }
                }
            }
            if !message.read().is_empty() { p { class: "admin-inline-message", "{message}" } }
            if loading { AdminLoading {} }
            else if filtered.is_empty() { div { class: "admin-empty", "No audit events match this search." } }
            else { div { class: "admin-audit-list", for event in filtered.iter() { article { key: "{event.audit_event_id}",
                span { class: "admin-audit-icon", Icon { name: "history", size: 16, color: "currentColor" } }
                div { strong { "{payment_label(&event.action)}" } p { if event.summary.is_empty() { "{event.reason}" } else { "{event.summary}" } } small { if event.actor_email.is_empty() { if let Some(actor_id) = event.actor_user_id.as_ref() { "{actor_id}" } else { "System" } } else { "{event.actor_email}" } if let Some(number) = event.booking_number.as_ref() { " · {number}" } else if let Some(booking_id) = event.booking_id.as_ref() { " · {booking_id}" } } }
                time { "{display_moment(&event.created_at)}" }
            } } } }
        }
    }
}

#[component]
fn BookingDrawer(
    detail: api::AdminBookingDetail,
    loading: bool,
    on_close: EventHandler<()>,
    on_changed: EventHandler<api::AdminBookingDetail>,
) -> Element {
    let delivery_ready = delivery_requirements_ready(&detail);
    let refundable = refundable_amount(&detail);
    let capturable = capturable_damage_amount(&detail);
    let has_active_hold = active_damage_hold(&detail).is_some();
    let awaiting_etransfer_deposit = awaiting_etransfer_damage_hold(&detail);
    let deposit_is_etransfer = etransfer_damage_hold(&detail).is_some();
    let hold_operation_active = hold_action_in_progress(&detail);
    let booking = detail.booking.clone();
    let customer_booking_id = booking.booking_id.clone();
    let notes_booking_id = booking.booking_id.clone();
    let upload_booking_id = booking.booking_id.clone();
    let action_booking_id = booking.booking_id.clone();
    let mut first_name = use_signal(|| booking.first_name.clone());
    let mut last_name = use_signal(|| booking.last_name.clone());
    let mut email = use_signal(|| booking.email.clone());
    let mut phone = use_signal(|| booking.phone.clone());
    let mut notes = use_signal(|| detail.admin_notes.clone());
    let mut modal = use_signal(|| None::<String>);
    let mut amount = use_signal(String::new);
    let mut reason = use_signal(String::new);
    let mut evidence_ids = use_signal(Vec::<String>::new);
    let mut evidence_names = use_signal(Vec::<String>::new);
    let mut uploads_pending = use_signal(|| 0_usize);
    let mut evidence_preview_busy = use_signal(String::new);
    let mut evidence_preview_message = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut message = use_signal(String::new);
    let mut modal_message = use_signal(String::new);
    let mut contact_dirty = use_signal(|| false);
    let mut notes_dirty = use_signal(|| false);

    let mut open_action = move |action: &'static str| {
        if loading || busy() || uploads_pending() > 0 {
            return;
        }
        if action == "cancel" {
            // A cancellation request always needs both values at the API, even when no
            // trip payment has been collected. Starting with the refundable amount
            // avoids leaving the confirmation action silently disabled in Calendar.
            amount.set(format!("{refundable:.2}"));
            reason.set("Cancelled by administrator".into());
        } else {
            amount.set(String::new());
            reason.set(String::new());
        }
        evidence_ids.set(Vec::new());
        evidence_names.set(Vec::new());
        uploads_pending.set(0);
        modal_message.set(String::new());
        modal.set(Some(action.into()));
    };

    let refresh_detail = move |booking_id: String| async move {
        match api::admin_booking(&booking_id).await {
            Ok(value) => on_changed.call(value),
            Err(error) => message.set(format!(
                "The change was saved, but booking details could not refresh: {}",
                error.message
            )),
        }
    };
    let request_close = move || {
        if busy() || uploads_pending() > 0 {
            return;
        }
        let may_close = (!contact_dirty() && !notes_dirty())
            || web_sys::window()
                .and_then(|window| {
                    window
                        .confirm_with_message("Discard unsaved booking changes?")
                        .ok()
                })
                .unwrap_or(false);
        if may_close {
            on_close.call(());
        }
    };

    rsx! {
        div { class: "admin-overlay admin-drawer-backdrop", onclick: move |_| request_close(),
            aside { class: "admin-booking-drawer", role: "dialog", aria_modal: "true", aria_label: "Booking {booking.booking_number}", tabindex: "-1", autofocus: true, onclick: move |event| event.stop_propagation(), onkeydown: move |event| if event.key() == Key::Escape { event.stop_propagation(); if modal.read().is_some() { if !busy() && uploads_pending() == 0 { modal.set(None); } } else { request_close(); } },
                header { class: "admin-drawer-head",
                    div { p { "{booking.booking_number}" } h2 { "{booking.first_name} {booking.last_name}" } span { class: "admin-status admin-status-{booking.status}", "{status_label(&booking.status)}" } }
                    button { r#type: "button", disabled: busy() || uploads_pending() > 0, aria_label: "Close booking details", onclick: move |_| request_close(), Icon { name: "x", size: 21, color: "currentColor" } }
                }
                if loading { AdminLoading {} }
                div { class: "admin-drawer-scroll",
                    section { class: "admin-drawer-section",
                        div { class: "admin-section-title", h3 { "Customer" } button { r#type: "button", disabled: loading || busy() || !contact_dirty(), onclick: move |_| { let values = (first_name(), last_name(), email(), phone()); let id = customer_booking_id.clone(); async move { busy.set(true); message.set(String::new()); match api::update_admin_booking_customer(&id, &values.0, &values.1, &values.2, &values.3).await { Ok(_) => { contact_dirty.set(false); message.set("Customer contacts updated.".into()); refresh_detail(id).await; }, Err(error) => message.set(error.message) } busy.set(false); } }, "Save contacts" } }
                        div { class: "admin-drawer-fields", label { "First name" input { value: "{first_name}", disabled: loading || busy(), oninput: move |event| { first_name.set(event.value()); contact_dirty.set(true); } } } label { "Last name" input { value: "{last_name}", disabled: loading || busy(), oninput: move |event| { last_name.set(event.value()); contact_dirty.set(true); } } } label { "Email" input { r#type: "email", value: "{email}", disabled: loading || busy(), oninput: move |event| { email.set(event.value()); contact_dirty.set(true); } } } label { "Phone" input { r#type: "tel", value: "{phone}", disabled: loading || busy(), oninput: move |event| { phone.set(event.value()); contact_dirty.set(true); } } } }
                    }
                    section { class: "admin-drawer-section",
                        h3 { "Trip & quote" }
                        dl { class: "admin-detail-grid",
                            div { dt { "RV" } dd { "{booking.rental_name}" } }
                            div { dt { "Guests" } dd { "{booking.guests}" } }
                            div { dt { "Delivery" } dd { "{display_moment(&booking.starts_at)}" } }
                            div { dt { "Return" } dd { "{display_moment(&booking.ends_at)}" } }
                            div { dt { "Trip price" } dd { "{display_money(&booking.currency, &booking.total)}" } }
                            div { dt { "Payment" } dd { "{payment_label(&booking.payment_status)}" } }
                        }
                        p { class: "admin-lock-note", Icon { name: "lock-keyhole", size: 14, color: "currentColor" } "Paid dates, RV and quote are locked. Cancel and create a new booking to change them." }
                    }
                    section { class: "admin-drawer-section",
                        h3 { "Payment schedule" }
                        if detail.obligations.is_empty() { div { class: "admin-empty", "No payment obligations are available yet." } }
                        else { div { class: "admin-obligation-list", for obligation in detail.obligations.iter() { article { key: "{obligation.payment_obligation_id}",
                            div { strong { "{payment_label(&obligation.payment_type)}" } if obligation.collection_method == "e_transfer" { small { "Interac e-Transfer · protrailercare@gmail.com" } } if let Some(due) = obligation.due_at.as_ref() { small { "Due {display_moment(due)}" } } if let Some(deadline) = obligation.capture_before.as_ref() { small { "Decision deadline {display_moment(deadline)}" } } }
                            span { "{display_money(&obligation.currency, &obligation.amount)}" }
                            b { class: "admin-status admin-pay-{obligation.status}", "{payment_label(&obligation.status)}" }
                        } } } }
                        if !detail.financial_operations.is_empty() {
                            h4 { "Financial actions" }
                            div { class: "admin-obligation-list", for operation in detail.financial_operations.iter() { article { key: "{operation.operation_id}",
                                div { strong { "{payment_label(&operation.operation_type)}" } small { "Part {operation.sequence_number.max(1)} · attempt {operation.attempt_count}" } if let Some(provider_status) = operation.last_provider_status.as_ref() { small { "Stripe: {payment_label(provider_status)}" } } }
                                span { "{display_money(&operation.currency, &operation.amount)}" }
                                b { class: "admin-status admin-pay-{operation.status}", "{payment_label(&operation.status)}" }
                                if let Some(error) = operation.last_error_message.as_ref() { small { class: "admin-payment-error", "{error}" } }
                            } } }
                        }
                    }
                    if !detail.damage_claims.is_empty() {
                        section { class: "admin-drawer-section",
                            h3 { "Damage claims & evidence" }
                            p { class: "admin-lock-note", Icon { name: "lock-keyhole", size: 14, color: "currentColor" } "Private evidence opens through a short-lived admin-authorized link." }
                            div { class: "admin-damage-claims",
                                for claim in detail.damage_claims.iter() {
                                    article { key: "{claim.damage_claim_id}", class: "admin-damage-claim",
                                        header {
                                            div { strong { "{display_money(&booking.currency, &claim.claimed_amount)}" } small { if claim.reason.is_empty() { "Evidence collected before a final decision" } else { "{claim.reason}" } } }
                                            b { class: "admin-status admin-pay-{claim.status}", "{payment_label(&claim.status)}" }
                                        }
                                        if claim.evidence.is_empty() { p { class: "admin-empty", "No evidence photos are attached." } }
                                        else { ul { class: "admin-existing-evidence",
                                            for evidence in claim.evidence.iter() {
                                                li { key: "{evidence.evidence_id}",
                                                    span { Icon { name: "image", size: 15, color: "currentColor" } span { strong { "{evidence.original_filename}" } small { "{display_file_size(evidence.byte_size)} · {display_date(&evidence.created_at)}" } } }
                                                    button { r#type: "button", disabled: !evidence_preview_busy().is_empty(), onclick: { let evidence_id = evidence.evidence_id.clone(); move |_| { let evidence_id = evidence_id.clone(); let target = format!("vl-evidence-{evidence_id}"); evidence_preview_message.set(String::new()); let preview_open = open_evidence_preview_window(&target); async move { if !preview_open { evidence_preview_message.set("Allow pop-ups to open the private evidence preview.".into()); return; } evidence_preview_busy.set(evidence_id.clone()); match api::admin_damage_evidence_access(&evidence_id).await { Ok(access) => match navigate_evidence_preview(&access, &target).await { Ok(()) => evidence_preview_message.set(format!("Opened {} in a private preview window.", access.evidence_id)), Err(error) => { close_evidence_preview(&target); evidence_preview_message.set(error); } }, Err(error) => { close_evidence_preview(&target); evidence_preview_message.set(error.message); } } evidence_preview_busy.set(String::new()); } } }, if evidence_preview_busy() == evidence.evidence_id { "Opening…" } else { "View" } }
                                                }
                                            }
                                        } }
                                    }
                                }
                            }
                            if !evidence_preview_message.read().is_empty() { p { class: "admin-inline-message", role: "status", "{evidence_preview_message}" } }
                        }
                    }
                    section { class: "admin-drawer-section",
                        div { class: "admin-section-title", h3 { "Internal notes" } button { r#type: "button", disabled: loading || busy() || !notes_dirty(), onclick: move |_| { let id = notes_booking_id.clone(); let value = notes(); async move { busy.set(true); message.set(String::new()); match api::update_admin_booking_notes(&id, &value).await { Ok(_) => { notes_dirty.set(false); message.set("Internal notes saved.".into()); refresh_detail(id).await; }, Err(error) => message.set(error.message) } busy.set(false); } }, "Save notes" } }
                        textarea { value: "{notes}", placeholder: "Visible only to admins", disabled: loading || busy(), oninput: move |event| { notes.set(event.value()); notes_dirty.set(true); } }
                    }
                    section { class: "admin-drawer-section",
                        h3 { "Timeline" }
                        div { class: "admin-timeline",
                            article { span {} div { strong { "Booking created" } small { "{display_moment(&booking.created_at)}" } } }
                            if let Some(value) = detail.delivered_at.as_ref() { article { span {} div { strong { "Delivered" } small { "{display_moment(value)}" } } } }
                            if let Some(value) = detail.returned_at.as_ref() { article { span {} div { strong { "Returned" } small { "{display_moment(value)}" } } } }
                            if let Some(value) = detail.cancelled_at.as_ref() { article { span {} div { strong { "Cancelled" } small { "{display_moment(value)}" } if let Some(reason) = detail.cancellation_reason.as_ref() { p { "{reason}" } } } } }
                        }
                    }
                    if !message.read().is_empty() { p { class: if message.read().starts_with("Booking updated, but") { "admin-error admin-inline-message" } else { "admin-inline-message" }, role: "status", "{message}" } }
                }
                footer { class: "admin-drawer-actions",
                    if booking.status == "confirmed" && awaiting_etransfer_deposit { button { class: "primary", r#type: "button", disabled: loading || busy(), onclick: move |_| open_action("confirm_deposit"), "Confirm e-Transfer received" } }
                    if booking.status == "confirmed" { button { class: if delivery_ready { "primary" } else { "" }, r#type: "button", disabled: loading || busy(), onclick: move |_| open_action(if delivery_ready { "delivered" } else { "delivered_blocked" }), if delivery_ready { "Mark delivered" } else { "Delivery requirements" } } }
                    if booking.status == "active" { button { class: "primary", r#type: "button", disabled: loading || busy(), onclick: move |_| open_action("returned"), "Mark returned" } }
                    if booking.status == "completed" && has_active_hold && !hold_operation_active { button { r#type: "button", disabled: loading || busy(), onclick: move |_| open_action("release"), "Refund deposit" } button { r#type: "button", disabled: loading || busy() || capturable <= 0.0, onclick: move |_| open_action("capture"), "Settle damage" } }
                    if booking.status == "completed" && hold_operation_active { span { class: "admin-inline-message", if deposit_is_etransfer { "A damage deposit action is pending." } else { "A damage deposit action is awaiting Stripe reconciliation." } } }
                    if !matches!(booking.status.as_str(), "cancelled" | "expired" | "completed") { button { class: "danger", r#type: "button", disabled: loading || busy(), onclick: move |_| open_action("cancel"), "Cancel booking" } }
                }
            }
            if let Some(action) = modal.read().clone() {
                div { class: "admin-confirm-layer", onclick: move |event| event.stop_propagation(),
                    section { class: "admin-confirm-modal", role: "alertdialog", aria_modal: "true", tabindex: "-1", autofocus: true, onkeydown: move |event| if event.key() == Key::Escape && !busy() && uploads_pending() == 0 { event.stop_propagation(); modal.set(None); },
                        header { h3 { if action == "delivered" { "Mark delivered?" } else if action == "delivered_blocked" { "Delivery is blocked" } else if action == "returned" { "Mark returned?" } else if action == "confirm_deposit" { "Confirm e-Transfer received?" } else if action == "release" { "Refund damage deposit?" } else if action == "capture" { "Settle damage from deposit?" } else { "Cancel booking?" } } button { r#type: "button", disabled: busy() || uploads_pending() > 0, aria_label: "Close confirmation", onclick: move |_| modal.set(None), Icon { name: "x", size: 19, color: "currentColor" } } }
                        p { if action == "delivered" { "The trip is fully paid and the refundable CA$1,000 damage deposit is paid. Confirm that the RV was delivered." } else if action == "delivered_blocked" { "The trip cannot be marked Delivered until every initial/balance obligation and the refundable CA$1,000 damage deposit are paid." } else if action == "returned" { "This opens the seven-day damage deposit decision period." } else if action == "confirm_deposit" { "Confirm only after CA$1,000 has arrived at protrailercare@gmail.com. This marks the deposit Paid and emails the customer and vlrental.ca@gmail.com." } else if action == "release" && deposit_is_etransfer { "Confirm only after you have returned the full CA$1,000 by e-Transfer. This records the refund and emails the customer." } else if action == "release" { "Stripe will refund the full CA$1,000 damage deposit to the original payment method. Processing fees from the original charge are not returned to VL Rental." } else if action == "capture" && deposit_is_etransfer { "Enter the documented damage amount and attach evidence. Confirm only after returning the remaining e-Transfer deposit to the customer." } else if action == "capture" { "Enter the documented damage amount. Stripe will refund the remaining deposit to the original payment method." } else if action == "cancel" && deposit_is_etransfer && has_active_hold { "The dates are released immediately. Before confirming, return the separate CA$1,000 damage deposit to the customer by e-Transfer; the system will record that manual deposit refund. Any entered trip-price refund is handled separately through Stripe." } else { "The dates are released immediately. Any refund is tracked separately if Stripe reports a failure." } }
                        if matches!(action.as_str(), "cancel" | "capture") {
                            p { class: "admin-action-limit", if action == "cancel" { "Available to refund: CAD ${refundable:.2}" } else { "Available damage deposit: CAD ${capturable:.2}" } }
                            label { if action == "cancel" { "Refund amount (CAD)" } else { "Damage amount (CAD)" } input { inputmode: "decimal", min: "0", max: if action == "cancel" { "{refundable:.2}" } else { "{capturable:.2}" }, value: "{amount}", placeholder: "0.00", disabled: busy() || uploads_pending() > 0, oninput: move |event| amount.set(event.value()) } }
                            label { "Reason" textarea { value: "{reason}", placeholder: "Required for audit and customer communication", disabled: busy() || uploads_pending() > 0, oninput: move |event| reason.set(event.value()) } }
                            if action == "cancel" { p { class: "admin-action-limit", "Review the refund amount and reason, then confirm the cancellation." } }
                        }
                        if action == "capture" {
                            label { class: "admin-evidence-upload", "Evidence photos" input { r#type: "file", accept: "image/jpeg,image/png,image/webp", multiple: true, disabled: busy() || uploads_pending() > 0, oninput: move |event| { let files = event.files(); for file in files { let name = file.name(); if file.size() > 10 * 1024 * 1024 { modal_message.set(format!("{name} is larger than 10 MB.")); continue; } let web_file = file.inner().downcast_ref::<web_sys::File>().cloned(); if let Some(web_file) = web_file { let booking_id = upload_booking_id.clone(); uploads_pending += 1; spawn(async move { match api::upload_admin_damage_evidence(&booking_id, &web_file).await { Ok(id) => { evidence_ids.write().push(id); evidence_names.write().push(name); }, Err(error) => modal_message.set(error.message) } uploads_pending -= 1; }); } else { modal_message.set(format!("{name} could not be read by this browser.")); } } } } }
                            if uploads_pending() > 0 { p { class: "admin-inline-message", role: "status", "Uploading evidence…" } }
                            if !evidence_names.read().is_empty() { ul { class: "admin-evidence-list", for name in evidence_names.read().iter() { li { Icon { name: "image", size: 14, color: "currentColor" } "{name}" } } } }
                        }
                        if !modal_message.read().is_empty() { p { class: "admin-error", role: "alert", "{modal_message}" } }
                        footer { button { r#type: "button", disabled: busy() || uploads_pending() > 0, onclick: move |_| modal.set(None), "Go back" } if action != "delivered_blocked" { button { class: if matches!(action.as_str(), "cancel" | "capture") { "danger" } else { "primary" }, r#type: "button", disabled: busy() || !action_confirmation_ready(&action, &amount(), &reason(), evidence_ids.read().len(), uploads_pending(), if action == "capture" { capturable } else { refundable }), onclick: { let action = action.clone(); move |_| { let id = action_booking_id.clone(); let action = action.clone(); let amount_value = amount(); let reason_value = reason(); let evidence = evidence_ids.read().clone(); async move { let maximum = if action == "capture" { capturable } else { refundable }; if !action_confirmation_ready(&action, &amount_value, &reason_value, evidence.len(), uploads_pending(), maximum) { modal_message.set("Complete all required confirmation details without exceeding the available amount.".into()); return; } busy.set(true); modal_message.set(String::new()); match api::admin_booking_action(&id, &action, (!amount_value.trim().is_empty()).then_some(amount_value.as_str()), (!reason_value.trim().is_empty()).then_some(reason_value.as_str()), (action == "capture").then_some(evidence.as_slice())).await { Ok(result) => { let cleanup_message = result.provider_cleanup.iter().find(|cleanup| cleanup.attention_required).and_then(|cleanup| cleanup.message.clone()); let provider_failed = result.provider_status.as_deref() == Some("failed") || cleanup_message.is_some(); modal.set(None); message.set(if provider_failed { format!("Booking updated, but Stripe needs attention: {}", cleanup_message.unwrap_or_else(|| "the provider operation failed".into())) } else if action == "confirm_deposit" { "e-Transfer receipt confirmed. The deposit is Paid and emails are queued for the customer and vlrental.ca@gmail.com.".into() } else if deposit_is_etransfer && matches!(action.as_str(), "release" | "capture") { "e-Transfer damage deposit decision recorded and customer email queued.".into() } else if action == "cancel" && deposit_is_etransfer && has_active_hold && result.provider_status.is_none() { "Booking cancelled and the manual e-Transfer deposit return was recorded. No refund is waiting for Stripe confirmation.".into() } else if result.provider_status.is_some() { format!("{} submitted. Stripe confirmation remains pending until webhook reconciliation.", payment_label(&action)) } else if action == "cancel" { "Booking cancelled. No refund is waiting for Stripe confirmation.".into() } else { format!("{} completed.", payment_label(&action)) }); on_changed.call(result.booking); }, Err(error) => modal_message.set(error.message) } busy.set(false); } } }, if busy() { "Working…" } else { "Confirm" } } } }
                    }
                }
            }
        }
    }
}

#[component]
fn ManualBookingModal(
    rentals: Vec<api::Rental>,
    on_close: EventHandler<()>,
    on_created: EventHandler<api::CreatedBooking>,
) -> Element {
    let today = Utc::now().with_timezone(&Vancouver).date_naive();
    let tomorrow = today + Duration::days(1);
    let mut rental_slug = use_signal(|| {
        rentals
            .first()
            .map(|rental| rental.slug.clone())
            .unwrap_or_default()
    });
    let mut starts_on = use_signal(|| (today + Duration::days(7)).to_string());
    let mut ends_on = use_signal(|| (today + Duration::days(10)).to_string());
    let mut guests = use_signal(|| 2_i32);
    let mut first_name = use_signal(String::new);
    let mut last_name = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut phone = use_signal(String::new);
    let mut address = use_signal(String::new);
    let mut notes = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(String::new);
    let mut dirty = use_signal(|| false);
    let request_close = move || {
        if busy() {
            return;
        }
        let may_close = !dirty()
            || web_sys::window()
                .and_then(|window| {
                    window
                        .confirm_with_message("Discard this phone booking draft?")
                        .ok()
                })
                .unwrap_or(false);
        if may_close {
            on_close.call(());
        }
    };
    rsx! {
        div { class: "admin-overlay admin-modal-backdrop", onclick: move |_| request_close(),
            section { class: "admin-manual-modal", role: "dialog", aria_modal: "true", aria_label: "Create phone booking", tabindex: "-1", autofocus: true, onclick: move |event| event.stop_propagation(), onkeydown: move |event| if event.key() == Key::Escape { event.stop_propagation(); request_close(); },
                header { div { p { "TWO-HOUR RESERVATION" } h2 { "Create phone booking" } span { "A dynamic Stripe Checkout link is emailed to the customer." } } button { r#type: "button", disabled: busy(), aria_label: "Close phone booking", onclick: move |_| request_close(), Icon { name: "x", size: 21, color: "currentColor" } } }
                div { class: "admin-manual-scroll",
                    section { h3 { "Trip" } select { value: "{rental_slug}", disabled: busy(), onchange: move |event| { rental_slug.set(event.value()); dirty.set(true); }, for rental in rentals.iter() { option { value: "{rental.slug}", "{rental.name}" } } } div { class: "admin-date-grid", label { "Delivery" input { r#type: "date", min: "{tomorrow}", value: "{starts_on}", disabled: busy(), onchange: move |event| { starts_on.set(event.value()); dirty.set(true); } } } label { "Return" input { r#type: "date", min: "{tomorrow}", value: "{ends_on}", disabled: busy(), onchange: move |event| { ends_on.set(event.value()); dirty.set(true); } } } } label { "Guests" input { r#type: "number", min: 1, max: 10, value: "{guests}", disabled: busy(), oninput: move |event| if let Ok(value) = event.value().parse::<i32>() { guests.set(value.clamp(1, 10)); dirty.set(true); } } } }
                    section { h3 { "Customer" } div { class: "admin-drawer-fields", label { "First name" input { value: "{first_name}", disabled: busy(), oninput: move |event| { first_name.set(event.value()); dirty.set(true); } } } label { "Last name" input { value: "{last_name}", disabled: busy(), oninput: move |event| { last_name.set(event.value()); dirty.set(true); } } } label { "Email" input { r#type: "email", value: "{email}", disabled: busy(), oninput: move |event| { email.set(event.value()); dirty.set(true); } } } label { "Phone" input { r#type: "tel", value: "{phone}", disabled: busy(), oninput: move |event| { phone.set(event.value()); dirty.set(true); } } } } label { "Delivery address" input { value: "{address}", placeholder: "Exact delivery location", disabled: busy(), oninput: move |event| { address.set(event.value()); dirty.set(true); } } } label { "Internal notes" textarea { value: "{notes}", disabled: busy(), oninput: move |event| { notes.set(event.value()); dirty.set(true); } } } }
                    div { class: "admin-time-rule", Icon { name: "clock-3", size: 17, color: "var(--vl-forest)" } span { "Availability is reserved for two hours. The booking confirms only after Stripe payment is verified by webhook." } }
                    if !error.read().is_empty() { p { class: "admin-error", role: "alert", "{error}" } }
                }
                footer { button { r#type: "button", disabled: busy(), onclick: move |_| request_close(), "Cancel" } button { class: "primary", r#type: "button", disabled: busy(), onclick: move |_| { let values = (rental_slug(), starts_on(), ends_on(), guests(), first_name(), last_name(), email(), phone(), address(), notes()); async move { if let Some(message) = manual_booking_error(today, &values.0, &values.1, &values.2, values.3, &values.4, &values.5, &values.6, &values.7, &values.8) { error.set(message.into()); return; } busy.set(true); error.set(String::new()); match api::create_manual_admin_booking(&values.0, &values.1, &values.2, values.3, &values.4, &values.5, &values.6, &values.7, &values.8, &values.9).await { Ok(created) => on_created.call(created), Err(api_error) => error.set(api_error.message) } busy.set(false); } }, if busy() { "Creating reservation…" } else { "Reserve and send payment link" } } }
            }
        }
    }
}

#[component]
fn AdminLoading() -> Element {
    rsx! { div { class: "admin-bookings-loading", Icon { name: "loader-circle", size: 20, color: "var(--vl-forest)" } "Loading admin data…" } }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_backend_admin_role_opens_admin_data_loading() {
        assert!(is_admin_role("admin"));
        assert!(!is_admin_role("default"));
        assert!(!is_admin_role(""));
    }

    #[test]
    fn external_calendar_blocks_are_never_treated_as_admin_reopen_actions() {
        let external = api::AdminAvailabilityBlock {
            source: "rvezy".into(),
            external_calendar_id: Some("calendar-id".into()),
            provider: Some("rvezy".into()),
            ..api::AdminAvailabilityBlock::default()
        };
        let admin = api::AdminAvailabilityBlock {
            source: "admin".into(),
            ..api::AdminAvailabilityBlock::default()
        };

        assert!(is_external_calendar_block(&external));
        assert_eq!(calendar_block_class(&external), "rvezy");
        assert!(!is_external_calendar_block(&admin));
        assert_eq!(calendar_block_class(&admin), "block");
    }

    #[test]
    fn calendar_conflict_color_overrides_the_provider_color() {
        let block = api::AdminAvailabilityBlock {
            source: "outdoorsy".into(),
            provider: Some("outdoorsy".into()),
            has_conflict: true,
            ..api::AdminAvailabilityBlock::default()
        };

        assert_eq!(calendar_block_class(&block), "conflict");
    }

    #[test]
    fn fleet_calendar_groups_external_periods_by_provider_and_conflict_state() {
        let outdoorsy = || api::AdminAvailabilityBlock {
            source: "outdoorsy".into(),
            provider: Some("outdoorsy".into()),
            reason: "Outdoorsy booking".into(),
            ..api::AdminAvailabilityBlock::default()
        };
        let blocks = [
            outdoorsy(),
            outdoorsy(),
            api::AdminAvailabilityBlock {
                source: "rvezy".into(),
                provider: Some("rvezy".into()),
                reason: "RVezy booking".into(),
                ..api::AdminAvailabilityBlock::default()
            },
            api::AdminAvailabilityBlock {
                source: "outdoorsy".into(),
                provider: Some("outdoorsy".into()),
                has_conflict: true,
                reason: "Outdoorsy booking".into(),
                ..api::AdminAvailabilityBlock::default()
            },
            api::AdminAvailabilityBlock {
                source: "rvezy".into(),
                provider: Some("rvezy".into()),
                has_conflict: true,
                reason: "RVezy booking".into(),
                ..api::AdminAvailabilityBlock::default()
            },
            api::AdminAvailabilityBlock {
                source: "admin".into(),
                reason: "Owner use".into(),
                ..api::AdminAvailabilityBlock::default()
            },
        ];

        let groups = aggregate_external_calendar_blocks(blocks.iter());

        assert_eq!(groups.len(), 3);
        assert_eq!((groups[0].key.as_str(), groups[0].count), ("rvezy", 1));
        assert_eq!((groups[1].key.as_str(), groups[1].count), ("outdoorsy", 2));
        assert_eq!((groups[2].key.as_str(), groups[2].count), ("conflict", 2));
    }

    #[test]
    fn cancellation_allows_zero_refund_but_damage_capture_requires_positive_amount() {
        assert!(action_confirmation_ready(
            "cancel",
            "0",
            "Guest request",
            0,
            0,
            300.0
        ));
        assert!(!action_confirmation_ready(
            "cancel",
            "-1",
            "Guest request",
            0,
            0,
            300.0
        ));
        assert!(!action_confirmation_ready(
            "capture", "0", "Damage", 1, 0, 1000.0
        ));
        assert!(action_confirmation_ready(
            "capture", "125.50", "Damage", 1, 0, 1000.0
        ));
        assert!(!action_confirmation_ready(
            "capture", "1000.01", "Damage", 1, 0, 1000.0
        ));
        assert!(!action_confirmation_ready(
            "cancel",
            "300.01",
            "Guest request",
            0,
            0,
            300.0
        ));
    }

    #[test]
    fn damage_confirmation_waits_for_evidence_uploads() {
        assert!(!action_confirmation_ready(
            "capture", "125.50", "Damage", 1, 1, 1000.0
        ));
        assert!(!action_confirmation_ready(
            "capture", "125.50", "Damage", 0, 0, 1000.0
        ));
    }

    #[test]
    fn delivery_ui_requires_paid_trip_obligations_and_damage_deposit() {
        let mut detail = api::AdminBookingDetail {
            booking: api::AdminBooking {
                ends_at: "2030-07-18T18:00:00Z".into(),
                ..api::AdminBooking::default()
            },
            obligations: vec![
                api::AdminPaymentObligation {
                    payment_type: "initial".into(),
                    status: "succeeded".into(),
                    ..api::AdminPaymentObligation::default()
                },
                api::AdminPaymentObligation {
                    payment_type: "damage_hold".into(),
                    status: "succeeded".into(),
                    ..api::AdminPaymentObligation::default()
                },
            ],
            ..api::AdminBookingDetail::default()
        };
        assert!(delivery_requirements_ready(&detail));

        detail.obligations.push(api::AdminPaymentObligation {
            payment_type: "balance".into(),
            status: "scheduled".into(),
            ..api::AdminPaymentObligation::default()
        });
        assert!(!delivery_requirements_ready(&detail));
    }

    #[test]
    fn delivery_ui_rejects_an_unpaid_damage_deposit() {
        let detail = api::AdminBookingDetail {
            booking: api::AdminBooking {
                ends_at: "2030-07-18T18:00:00Z".into(),
                ..api::AdminBooking::default()
            },
            obligations: vec![
                api::AdminPaymentObligation {
                    payment_type: "initial".into(),
                    status: "succeeded".into(),
                    ..api::AdminPaymentObligation::default()
                },
                api::AdminPaymentObligation {
                    payment_type: "damage_hold".into(),
                    status: "link_created".into(),
                    ..api::AdminPaymentObligation::default()
                },
            ],
            ..api::AdminBookingDetail::default()
        };

        assert!(!delivery_requirements_ready(&detail));
    }

    #[test]
    fn payment_link_resend_is_only_available_for_active_unpaid_links() {
        let mut payment = api::AdminPaymentObligation {
            payment_type: "balance".into(),
            status: "link_created".into(),
            hosted_url: Some("https://invoice.stripe.test/i/test".into()),
            ..api::AdminPaymentObligation::default()
        };
        assert!(can_resend_payment_link(&payment));

        payment.status = "succeeded".into();
        assert!(!can_resend_payment_link(&payment));

        payment.status = "failed".into();
        assert!(!can_resend_payment_link(&payment));

        payment.status = "due".into();
        assert!(!can_resend_payment_link(&payment));

        payment.status = "link_created".into();
        payment.payment_type = "initial".into();
        assert!(!can_resend_payment_link(&payment));
    }

    #[test]
    fn refund_and_damage_limits_follow_backend_available_amounts() {
        let detail = api::AdminBookingDetail {
            obligations: vec![
                api::AdminPaymentObligation {
                    payment_type: "initial".into(),
                    status: "succeeded".into(),
                    amount: "300.00".into(),
                    amount_refunded: "25.00".into(),
                    ..api::AdminPaymentObligation::default()
                },
                api::AdminPaymentObligation {
                    payment_type: "damage_hold".into(),
                    status: "succeeded".into(),
                    amount: "1000.00".into(),
                    amount_refunded: "125.00".into(),
                    ..api::AdminPaymentObligation::default()
                },
            ],
            financial_operations: vec![api::AdminFinancialOperation {
                operation_type: "refund".into(),
                status: "submitted".into(),
                amount: "75.00".into(),
                ..api::AdminFinancialOperation::default()
            }],
            ..api::AdminBookingDetail::default()
        };

        assert_eq!(refundable_amount(&detail), 200.0);
        assert_eq!(capturable_damage_amount(&detail), 875.0);
    }

    #[test]
    fn manual_booking_requires_future_one_night_trip_and_customer_details() {
        let today = NaiveDate::from_ymd_opt(2030, 7, 10).unwrap();
        assert_eq!(
            manual_booking_error(
                today,
                "jayco26",
                "2030-07-11",
                "2030-07-14",
                4,
                "Test",
                "Guest",
                "guest@example.com",
                "2505550100",
                "Bear Creek Provincial Park"
            ),
            None
        );
        assert_eq!(
            manual_booking_error(
                today,
                "jayco26",
                "2030-07-11",
                "2030-07-12",
                4,
                "Test",
                "Guest",
                "guest@example.com",
                "2505550100",
                "Bear Creek Provincial Park"
            ),
            None
        );
        assert_eq!(
            manual_booking_error(
                today,
                "jayco26",
                "2030-07-11",
                "2030-07-11",
                4,
                "Test",
                "Guest",
                "guest@example.com",
                "2505550100",
                "Bear Creek Provincial Park"
            ),
            Some("Choose an available RV and a future trip of at least one night.")
        );
        assert_eq!(
            manual_booking_error(
                today,
                "jayco26",
                "invalid",
                "2030-07-14",
                4,
                "Test",
                "Guest",
                "guest@example.com",
                "2505550100",
                "Bear Creek Provincial Park"
            ),
            Some("Choose valid delivery and return dates.")
        );
    }

    #[test]
    fn fleet_calendar_keeps_return_and_next_delivery_on_the_same_day() {
        let turnover_day = NaiveDate::from_ymd_opt(2030, 7, 14).unwrap();
        let returning = api::AdminBooking {
            status: "confirmed".into(),
            starts_at: "2030-07-10T21:00:00Z".into(),
            ends_at: "2030-07-14T18:00:00Z".into(),
            ..api::AdminBooking::default()
        };
        let arriving = api::AdminBooking {
            status: "confirmed".into(),
            starts_at: "2030-07-14T21:00:00Z".into(),
            ends_at: "2030-07-18T18:00:00Z".into(),
            ..api::AdminBooking::default()
        };

        assert!(booking_occupies_day(&returning, turnover_day));
        assert!(booking_occupies_day(&arriving, turnover_day));
    }

    #[test]
    fn mobile_calendar_agenda_matches_the_inclusive_visible_window() {
        let window_start = NaiveDate::from_ymd_opt(2030, 7, 10).unwrap();
        let window_end = NaiveDate::from_ymd_opt(2030, 7, 23).unwrap();

        assert!(calendar_interval_overlaps_window(
            "2030-07-08T21:00:00Z",
            "2030-07-10T18:00:00Z",
            window_start,
            window_end,
        ));
        assert!(calendar_interval_overlaps_window(
            "2030-07-23T21:00:00Z",
            "2030-07-26T18:00:00Z",
            window_start,
            window_end,
        ));
        assert!(!calendar_interval_overlaps_window(
            "2030-07-24T21:00:00Z",
            "2030-07-27T18:00:00Z",
            window_start,
            window_end,
        ));
        assert!(!calendar_interval_overlaps_window(
            "invalid",
            "2030-07-12T18:00:00Z",
            window_start,
            window_end,
        ));
    }

    #[test]
    fn admin_calendar_dates_use_vancouver_day_boundaries() {
        assert_eq!(
            admin_calendar_date("2030-07-10T06:30:00Z"),
            NaiveDate::from_ymd_opt(2030, 7, 9)
        );
        assert_eq!(
            admin_calendar_date("2030-01-10T07:30:00Z"),
            NaiveDate::from_ymd_opt(2030, 1, 9)
        );
    }
}
