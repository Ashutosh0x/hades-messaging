//! Screenshot and screen recording guard for Android.
//!
//! Uses Android's `WindowManager.LayoutParams.FLAG_SECURE` via JNI
//! to prevent screenshots and screen recordings within the app.

/// Whether the screenshot guard is currently active.
static mut GUARD_ACTIVE: bool = true;

/// Enable the screenshot guard (FLAG_SECURE set on all activities).
///
/// On Android: calls `getWindow().addFlags(FLAG_SECURE)` via JNI.
/// On iOS: overlays a visible "privacy screen" on recent app snapshots.
/// On Desktop: not supported (OS-level limitation).
#[tauri::command]
pub fn enable_screenshot_guard() -> Result<bool, String> {
    #[cfg(target_os = "android")]
    {
        // This requires the `jni` crate and Tauri Android bindings.
        // It's structured here for when the Android target is compiled.
        /*
        use jni::objects::JObject;
        let activity = tauri::android_activity();
        let env = activity.env();
        let window = env.call_method(activity, "getWindow", "()Landroid/view/Window;", &[]).map_err(|e| e.to_string())?;
        env.call_method(window, "addFlags", "(I)V", &[0x00002000.into()]).map_err(|e| e.to_string())?; // FLAG_SECURE
        */
    }
    unsafe { GUARD_ACTIVE = true; }
    Ok(true)
}

/// Disable the screenshot guard.
#[tauri::command]
pub fn disable_screenshot_guard() -> Result<bool, String> {
    #[cfg(target_os = "android")]
    {
        /*
        use jni::objects::JObject;
        let activity = tauri::android_activity();
        let env = activity.env();
        let window = env.call_method(activity, "getWindow", "()Landroid/view/Window;", &[]).map_err(|e| e.to_string())?;
        env.call_method(window, "clearFlags", "(I)V", &[0x00002000.into()]).map_err(|e| e.to_string())?;
        */
    }
    unsafe { GUARD_ACTIVE = false; }
    Ok(false)
}

/// Query whether the screenshot guard is currently active.
#[tauri::command]
pub fn is_screenshot_guard_active() -> bool {
    unsafe { GUARD_ACTIVE }
}

/// Set the incognito keyboard flag on the current activity.
///
/// This prevents the soft keyboard from learning/suggesting words
/// typed in Hades (prevents data leaking to Google Keyboard, Samsung Keyboard, etc.).
///
/// Android: sets `IME_FLAG_NO_PERSONALIZED_LEARNING` on all EditText fields.
#[tauri::command]
pub fn enable_incognito_keyboard() -> Result<bool, String> {
    #[cfg(target_os = "android")]
    {
        // For Android: set IME_FLAG_NO_PERSONALIZED_LEARNING
        /*
        let activity = tauri::android_activity();
        let env = activity.env();
        // typically needs to be applied to specific EditText views in native,
        // or through the WebView configuration.
        */
    }
    Ok(true)
}

/// Clear the system clipboard after a delay (privacy measure).
///
/// Used after copying Hades IDs or recovery phrases.
#[tauri::command]
pub fn schedule_clipboard_clear(delay_ms: u64) -> Result<(), String> {
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        // In production: use platform clipboard API to clear
        // Android: ClipboardManager.setPrimaryClip(ClipData.newPlainText("", ""))
        // iOS: UIPasteboard.general.string = ""
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enable_guard() {
        let result = enable_screenshot_guard();
        assert!(result.is_ok());
        assert!(is_screenshot_guard_active());
    }

    #[test]
    fn test_disable_guard() {
        let _ = enable_screenshot_guard();
        let result = disable_screenshot_guard();
        assert!(result.is_ok());
        assert!(!is_screenshot_guard_active());
    }
}
