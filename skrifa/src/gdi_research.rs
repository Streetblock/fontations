//! Process-local switches for comparing GDI compatibility experiments.
//! With no switches selected, the interpreter uses its upstream arithmetic.

#[cfg(feature = "std")]
pub(crate) fn enabled(name: &str) -> bool {
    use std::{env, sync::OnceLock};

    static FEATURES: OnceLock<std::string::String> = OnceLock::new();
    FEATURES
        .get_or_init(|| env::var("SKRIFA_GDI_RESEARCH").unwrap_or_default())
        .split(',')
        .any(|feature| feature == name)
}

#[cfg(not(feature = "std"))]
pub(crate) fn enabled(_: &str) -> bool {
    false
}
