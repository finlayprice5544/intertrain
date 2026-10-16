use crate::config::Config;
use serde_json::{json, Value};

/// Branding payload served at /api/_branding and usable inside any route.
pub fn payload(cfg: &Config) -> Value {
    let b = &cfg.branding;
    json!({
        "name": b.site_name,
        "ticker": b.ticker,
        "contractAddress": b.contract_address,
        "socials": {
            "twitter": b.socials.twitter,
            "github": b.socials.github,
            "discord": b.socials.discord,
            "telegram": b.socials.telegram,
            "website": b.socials.website,
        },
        "theme": b.theme,
    })
}
