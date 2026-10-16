// Config: environment variables  >  config/branding.json  >  defaults.
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DataMode {
    Seed,
    Live,
    Auto,
}
impl DataMode {
    fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "seed" => DataMode::Seed,
            "live" => DataMode::Live,
            _ => DataMode::Auto,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Socials {
    pub twitter: String,
    pub github: String,
    pub discord: String,
    pub telegram: String,
    pub website: String,
}

#[derive(Clone, Debug)]
pub struct Branding {
    pub site_name: String,
    pub ticker: String,
    pub contract_address: String,
    pub socials: Socials,
    pub theme: BTreeMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct ClickhouseCfg {
    pub url: String,
    pub database: String,
    pub user: String,
    pub password: String,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub port: u16,
    pub data_mode: DataMode,
    pub public_dir: String,
    pub rocksdb_path: String,
    pub branding: Branding,
    pub clickhouse: ClickhouseCfg,
    pub solana_rpc_url: String,
    pub solana_cluster: String,
    pub dexscreener_base: String,
    pub jupiter_base: String,
    pub birdeye_api_key: String,
    pub pinata_jwt: String,
    pub ipfs_gateway: String,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct BrandingFile {
    site_name: String,
    ticker: String,
    contract_address: String,
    socials: Socials,
    theme: BTreeMap<String, String>,
}

fn env_opt(key: &str) -> Option<String> {
    std::env::var(key).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}
fn env_or(key: &str, default: &str) -> String {
    env_opt(key).unwrap_or_else(|| default.to_string())
}
/// env value, else the value from branding.json, else default.
fn pick(env_key: &str, file_val: &str, default: &str) -> String {
    env_opt(env_key).unwrap_or_else(|| {
        if !file_val.is_empty() { file_val.to_string() } else { default.to_string() }
    })
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let file: BrandingFile = std::fs::read_to_string("config/branding.json")
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        let branding = Branding {
            site_name: pick("SITE_NAME", &file.site_name, "My Project"),
            ticker: pick("TOKEN_TICKER", &file.ticker, "TICKER"),
            contract_address: pick("CONTRACT_ADDRESS", &file.contract_address, ""),
            socials: Socials {
                twitter: pick("TWITTER_URL", &file.socials.twitter, ""),
                github: pick("GITHUB_URL", &file.socials.github, ""),
                discord: pick("DISCORD_URL", &file.socials.discord, ""),
                telegram: pick("TELEGRAM_URL", &file.socials.telegram, ""),
                website: pick("WEBSITE_URL", &file.socials.website, ""),
            },
            theme: file.theme,
        };

        let solana_rpc_url = match env_opt("SOLANA_RPC_URL") {
            Some(u) => u,
            None => match env_opt("HELIUS_API_KEY") {
                Some(k) => format!("https://mainnet.helius-rpc.com/?api-key={k}"),
                None => "https://api.mainnet-beta.solana.com".to_string(),
            },
        };

        Ok(Config {
            port: env_or("PORT", "8787").parse().unwrap_or(8787),
            data_mode: DataMode::parse(&env_or("DATA_MODE", "auto")),
            public_dir: env_or("PUBLIC_DIR", "../frontend"),
            rocksdb_path: env_or("ROCKSDB_PATH", "./data/rocks"),
            branding,
            clickhouse: ClickhouseCfg {
                url: env_or("CLICKHOUSE_URL", "http://localhost:8123"),
                database: env_or("CLICKHOUSE_DATABASE", "app"),
                user: env_or("CLICKHOUSE_USER", "default"),
                password: env_or("CLICKHOUSE_PASSWORD", ""),
            },
            solana_rpc_url,
            solana_cluster: env_or("SOLANA_CLUSTER", "mainnet-beta"),
            dexscreener_base: env_or("DEXSCREENER_BASE", "https://api.dexscreener.com"),
            jupiter_base: env_or("JUPITER_BASE", "https://lite-api.jup.ag"),
            birdeye_api_key: env_or("BIRDEYE_API_KEY", ""),
            pinata_jwt: env_or("PINATA_JWT", ""),
            ipfs_gateway: env_or("IPFS_GATEWAY", "https://ipfs.io/ipfs/"),
        })
    }
}
