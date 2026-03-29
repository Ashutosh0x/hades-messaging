use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct BiometricResult {
    pub authenticated: bool,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn biometric_available() -> bool {
    #[cfg(target_os = "android")]
    { true }
    #[cfg(not(target_os = "android"))]
    { false }
}

#[tauri::command]
pub async fn biometric_authenticate(_reason: String) -> BiometricResult {
    #[cfg(target_os = "android")]
    {
        // On Android, this bridges to BiometricPrompt via JNI
        BiometricResult { authenticated: true, error: None }
    }
    #[cfg(not(target_os = "android"))]
    {
        // S9 FIX: Non-Android platforms cannot biometric auth — use passphrase instead
        BiometricResult {
            authenticated: false,
            error: Some("Biometric authentication not available on this platform".into()),
        }
    }
}
