mod discuz;
mod nga;
mod v2ex;
use std::sync::Arc;

// mod nga;
pub use discuz::{DiscuzForum, DiscuzForumConfig};
pub use nga::{NGAForum, NGAForumConfig};
use serde::{Deserialize, Serialize};
pub use v2ex::{V2exForum, V2exForumConfig};

use crate::{core::ForumProvider, error::Result};
// pub use nga::NGAForum;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ForumConfig {
    Discuz(DiscuzForumConfig),
    V2ex(V2exForumConfig),
    NGA(NGAForumConfig),
}

impl ForumConfig {
    pub async fn build_forum(&self) -> Result<Arc<dyn ForumProvider>> {
        let forum: Arc<dyn ForumProvider> = match self {
            Self::Discuz(config) => Arc::new(config.build_forum().await?),
            Self::V2ex(config) => Arc::new(config.build_forum().await?),
            Self::NGA(config) => Arc::new(config.build_forum().await?),
        };
        Ok(forum)
    }
    pub fn domain(&self) -> String {
        match self {
            Self::Discuz(config) => config.domain(),
            Self::V2ex(config) => config.domain(),
            Self::NGA(config) => config.domain(),
        }
    }
}

impl Into<ForumConfig> for DiscuzForumConfig {
    fn into(self) -> ForumConfig {
        ForumConfig::Discuz(self)
    }
}

impl Into<ForumConfig> for V2exForumConfig {
    fn into(self) -> ForumConfig {
        ForumConfig::V2ex(self)
    }
}
