use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FairshareInfo {
    pub account: String,
    pub user: Option<String>,
    pub raw_shares: i64,
    pub norm_shares: f64,
    pub raw_usage: i64,
    pub effectv_usage: f64,
    pub fairshare: f64,
}

impl FairshareInfo {
    pub fn new(account: String) -> Self {
        Self {
            account,
            user: None,
            raw_shares: 0,
            norm_shares: 0.0,
            raw_usage: 0,
            effectv_usage: 0.0,
            fairshare: 0.0,
        }
    }

    pub fn with_user(mut self, user: String) -> Self {
        self.user = Some(user);
        self
    }

    pub fn display_fairshare(&self) -> String {
        format!("{:.4}", self.fairshare)
    }

    pub fn fairshare_description(&self) -> &'static str {
        if self.fairshare >= 0.75 {
            "Well under resource usage"
        } else if self.fairshare >= 0.50 {
            "Normal resource usage"
        } else if self.fairshare >= 0.25 {
            "Approaching usage limit"
        } else {
            "Near usage limit - lower priority"
        }
    }
}

impl fmt::Display for FairshareInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref user) = self.user {
            write!(
                f,
                "{}:{} - Fairshare: {}",
                self.account,
                user,
                self.display_fairshare()
            )
        } else {
            write!(
                f,
                "{} - Fairshare: {}",
                self.account,
                self.display_fairshare()
            )
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserLimits {
    pub fairshare: Option<FairshareInfo>,
    pub account: Option<String>,
    pub user: Option<String>,
}

impl UserLimits {
    pub fn new() -> Self {
        Self::default()
    }
}
