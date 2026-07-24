use dioxus::prelude::document;

use crate::api;

pub async fn permission() -> String {
    document::eval(
        r#"
            if (!window.vlPush) return "unsupported";
            return window.vlPush.permission();
        "#,
    )
    .join::<String>()
    .await
    .unwrap_or_else(|_| "unsupported".into())
}

pub async fn status() -> String {
    match permission().await.as_str() {
        "granted" if preference_enabled() => {
            let Ok(config) = configured_push().await else {
                return "available".into();
            };
            let token = current_token(&config).await;
            if token.is_empty() || api::set_push_device(&token, true).await != Ok(true) {
                return "available".into();
            }
            "enabled".into()
        }
        "granted" | "default" => "available".into(),
        "denied" => "denied".into(),
        _ => "unsupported".into(),
    }
}

pub async fn enable() -> Result<(), String> {
    let config = configured_push().await?;
    let config_json = serde_json::to_string(&config).map_err(|error| error.to_string())?;
    let token = document::eval(&format!(
        r#"
            if (!window.vlPush) throw new Error("Push client is unavailable");
            return await window.vlPush.subscribe({config_json});
        "#
    ))
    .join::<String>()
    .await
    .map_err(|_| "Notification permission was not granted.".to_string())?;
    if token.is_empty() {
        return Err("Notification permission was not granted.".into());
    }
    if !api::set_push_device(&token, true).await? {
        return Err("The browser could not be registered for notifications.".into());
    }
    set_preference(true);
    Ok(())
}

pub async fn disable() -> Result<(), String> {
    let config = configured_push().await?;
    let token = current_token(&config).await;
    if !token.is_empty() {
        api::set_push_device(&token, false).await?;
    }
    delete_local_token(&config).await;
    set_preference(false);
    Ok(())
}

pub async fn unregister_before_logout() {
    let Ok(config) = configured_push().await else {
        return;
    };
    let token = current_token(&config).await;
    if !token.is_empty() {
        let _ = api::set_push_device(&token, false).await;
    }
    delete_local_token(&config).await;
    set_preference(false);
}

fn preference_enabled() -> bool {
    web_sys::window()
        .and_then(|window| window.local_storage().ok().flatten())
        .and_then(|storage| storage.get_item("vl_push_enabled").ok().flatten())
        .is_some_and(|value| value == "1")
}

fn set_preference(enabled: bool) {
    if let Some(storage) =
        web_sys::window().and_then(|window| window.local_storage().ok().flatten())
    {
        if enabled {
            let _ = storage.set_item("vl_push_enabled", "1");
        } else {
            let _ = storage.remove_item("vl_push_enabled");
        }
    }
}

async fn configured_push() -> Result<api::PushConfig, String> {
    let config = api::push_config().await?;
    if !config.enabled
        || config.api_key.as_deref().is_none_or(str::is_empty)
        || config.auth_domain.as_deref().is_none_or(str::is_empty)
        || config.project_id.as_deref().is_none_or(str::is_empty)
        || config
            .messaging_sender_id
            .as_deref()
            .is_none_or(str::is_empty)
        || config.app_id.as_deref().is_none_or(str::is_empty)
        || config.vapid_public_key.as_deref().is_none_or(str::is_empty)
    {
        return Err("Push notifications are not configured yet.".into());
    }
    Ok(config)
}

async fn current_token(config: &api::PushConfig) -> String {
    let Ok(config_json) = serde_json::to_string(config) else {
        return String::new();
    };
    document::eval(&format!(
        r#"
            if (!window.vlPush) return "";
            return await window.vlPush.currentToken({config_json});
        "#
    ))
    .join::<String>()
    .await
    .unwrap_or_default()
}

async fn delete_local_token(config: &api::PushConfig) {
    let Ok(config_json) = serde_json::to_string(config) else {
        return;
    };
    let _ = document::eval(&format!(
        r#"
            if (!window.vlPush) return false;
            return await window.vlPush.unsubscribe({config_json});
        "#
    ))
    .join::<bool>()
    .await;
}
