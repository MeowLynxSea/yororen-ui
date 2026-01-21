use std::{borrow::Cow, collections::BTreeSet};

use gpui::{AssetSource, SharedString};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "assets/"]
#[include = "icons/**/*"]
#[exclude = "*.DS_Store"]
pub struct UiAsset;

impl AssetSource for UiAsset {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        Ok(Self::get(path).map(|f| f.data))
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>> {
        Ok(Self::iter()
            .filter_map(|p| {
                if p.starts_with(path) {
                    Some(p.into())
                } else {
                    None
                }
            })
            .collect())
    }
}

/// Composes two asset sources.
///
/// `Primary` takes precedence over `Fallback` when loading the same path.
pub struct CompositeAssetSource<Primary, Fallback> {
    primary: Primary,
    fallback: Fallback,
}

impl<Primary, Fallback> CompositeAssetSource<Primary, Fallback> {
    pub fn new(primary: Primary, fallback: Fallback) -> Self {
        Self { primary, fallback }
    }
}

impl<Primary, Fallback> AssetSource for CompositeAssetSource<Primary, Fallback>
where
    Primary: AssetSource,
    Fallback: AssetSource,
{
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        if let Some(asset) = self.primary.load(path)? {
            return Ok(Some(asset));
        }
        self.fallback.load(path)
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>> {
        let mut merged = BTreeSet::<SharedString>::new();
        for asset in self.primary.list(path)? {
            merged.insert(asset);
        }
        for asset in self.fallback.list(path)? {
            merged.insert(asset);
        }
        Ok(merged.into_iter().collect())
    }
}
