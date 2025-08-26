pub mod app;

pub use app::App;
#[cfg(not(target_arch = "wasm32"))]
pub use app::{ServerApp, ServerAppProps};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeviceInfo;

impl DeviceInfo {
    pub fn parse_from(_: &str) -> Self {
        DeviceInfo
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SupportedLanguage;

#[cfg(not(target_arch = "wasm32"))]
impl SupportedLanguage {
    pub const SUPPORTED_LANGUAGES: [&'static str; 1] = ["en"];

    pub fn match_tag(_: &str) -> Option<Self> {
        Some(SupportedLanguage)
    }
}

