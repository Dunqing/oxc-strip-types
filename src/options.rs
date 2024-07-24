#[derive(Debug, Clone)]
pub struct StripTypesOptions {
    /// Replace types with spaces
    /// default is `true`
    pub replace_with_space: bool,
}

impl Default for StripTypesOptions {
    fn default() -> Self {
        Self { replace_with_space: true }
    }
}
