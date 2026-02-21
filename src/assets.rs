use gpui::{AssetSource, Result, SharedString};
use gpui_component::IconNamed;
use rust_embed::RustEmbed;
use std::borrow::Cow;

#[derive(RustEmbed)]
#[folder = "assets"]
#[include = "icons/**/*.svg"]
struct LocalAssets;

/// Combined asset source: local assets first, then gpui-component defaults.
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path.is_empty() {
            return Ok(None);
        }

        // Try local assets first
        if let Some(file) = LocalAssets::get(path) {
            return Ok(Some(file.data));
        }

        // Fall back to gpui-component assets
        gpui_component_assets::Assets.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut items: Vec<SharedString> = LocalAssets::iter()
            .filter(|p| p.starts_with(path))
            .map(|p| SharedString::from(p.to_string()))
            .collect();

        items.extend(gpui_component_assets::Assets.list(path)?);
        Ok(items)
    }
}

/// Custom icon names for icons not included in gpui-component.
#[derive(Clone)]
pub enum AppIconName {
    Pencil,
    Presentation,
}

impl IconNamed for AppIconName {
    fn path(self) -> SharedString {
        match self {
            Self::Pencil => "icons/pencil.svg",
            Self::Presentation => "icons/presentation.svg",
        }
        .into()
    }
}
