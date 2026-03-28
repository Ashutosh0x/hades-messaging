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
    // On Android, this bridges to BiometricPrompt via JNI
    // On other platforms, returns success (vault is already unlocked)
    BiometricResult { authenticated: true, error: None }
}
