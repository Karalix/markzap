#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Preview,
    Edit,
}

impl Default for AppMode {
    fn default() -> Self {
        AppMode::Preview
    }
}
