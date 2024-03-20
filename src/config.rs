use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::PathBuf,
    sync::OnceLock,
};

use serde::{de::Error as DeError, Deserialize, Deserializer};
use url::Url;

static GLOBAL_SECRETS: OnceLock<Secrets> = OnceLock::new();

pub fn init_secrets() -> anyhow::Result<()> {
    let secrets = Secrets::new()?;
    GLOBAL_SECRETS.get_or_init(|| secrets);
    Ok(())
}

pub fn expect_secrets() -> &'static Secrets {
    GLOBAL_SECRETS.get().unwrap()
}

static GLOBAL_CONFIG: OnceLock<Config> = OnceLock::new();

pub fn init_config() -> anyhow::Result<()> {
    let config = Config::new()?;
    GLOBAL_CONFIG.get_or_init(|| config);
    Ok(())
}

pub fn expect_config() -> &'static Config {
    GLOBAL_CONFIG.get().unwrap()
}

#[derive(Deserialize)]
pub struct Secrets {
    pub reddit: RedditOauth2,
}

impl Secrets {
    pub fn new() -> anyhow::Result<Self> {
        let secrets_path = env::var("SECRETS_PATH")?;

        tracing::info!(secrets_path, "Reading secrets at path");
        let secrets_text = fs::read_to_string(&secrets_path)?;
        let secrets = toml::from_str(&secrets_text)?;

        Ok(secrets)
    }
}

#[derive(Deserialize)]
pub struct RedditOauth2 {
    pub client_id: String,
    pub client_secret: String,
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(rename = "url")]
    pub url_filters: UrlFilters,
}

impl Config {
    pub fn new() -> anyhow::Result<Self> {
        // TODO: Have this default to /var/opt/auto_shadow0133/config.toml on linux
        let config_path = env::var("CONFIG_PATH")?;

        tracing::info!(config_path, "Reading config at path");
        let config_path = PathBuf::from(&config_path);
        let config_text = fs::read_to_string(&config_path)?;
        let config = toml::from_str(&config_text)?;

        Ok(config)
    }
}

#[derive(Deserialize, Debug)]
pub struct UrlFilters {
    pub allow: UrlSet,
    pub block: UrlSet,
}

/// A set formed from Url's domains
///
/// Maps TLDs -> domains -> optional sub-domains
///
/// This complex structure is just to avoid a linear scan over all the domains
#[derive(Debug, Default)]
pub struct UrlSet(BTreeMap<String, BTreeMap<String, Option<BTreeSet<String>>>>);

impl UrlSet {
    fn new(needles: Vec<UrlNeedle>) -> Self {
        let mut inner = Self::default().0;

        for UrlNeedle {
            maybe_sub,
            base,
            top,
        } in needles
        {
            let domain_set = inner.entry(top).or_default();

            if let Some(sub) = maybe_sub {
                if let Some(subs) = domain_set
                    .entry(base)
                    .or_insert_with(|| Some(BTreeSet::new()))
                {
                    subs.insert(sub);
                }
            } else {
                let subs = domain_set.entry(base).or_default();
                *subs = None;
            }
        }

        Self(inner)
    }

    pub fn contains(&self, url: &Url) -> bool {
        fn inner(set: &UrlSet, url: &Url) -> Option<bool> {
            let domain = url.domain()?;
            let UrlNeedle {
                maybe_sub,
                base,
                top,
            } = UrlNeedle::new(domain).ok()?;

            let bases = set.0.get(&top)?;
            let maybe_subs = bases.get(&base)?;

            let contains = match (maybe_subs, maybe_sub) {
                // Filter allows for all subdomains
                (None, _) => true,
                // Filter is only for specific subdomains
                (Some(_), None) => false,
                // Check subdomain match
                (Some(subs), Some(sub)) => subs.contains(&sub),
            };

            Some(contains)
        }

        inner(self, url).unwrap_or_default()
    }
}

impl<'de> Deserialize<'de> for UrlSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let urls = <Vec<UrlNeedle>>::deserialize(deserializer)?;
        Ok(UrlSet::new(urls))
    }
}

struct UrlNeedle {
    maybe_sub: Option<String>,
    base: String,
    top: String,
}

impl UrlNeedle {
    fn new(domain: &str) -> anyhow::Result<Self> {
        let mut pieces = domain.split('.');

        let top = pieces.next_back();
        let base = pieces.next_back();
        let maybe_sub = pieces.next().map(ToOwned::to_owned);
        let (base, top) = match (base, top, pieces.next()) {
            (Some(base), Some(top), None) => (base.to_owned(), top.to_owned()),
            _ => anyhow::bail!("Unrecognized domain format"),
        };

        Ok(Self {
            maybe_sub,
            base,
            top,
        })
    }
}

impl<'de> Deserialize<'de> for UrlNeedle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let domain = String::deserialize(deserializer)?;
        UrlNeedle::new(&domain).map_err(|err| DeError::custom(err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanity() {
        let sample_config = r#"
        [url]
        allow = [
            "sub-less.allow",
            "sub-less.allow2",
            "a.sub.allow",
            "b.sub.allow",
            "overrides-subdomain.allow",
            "override-me.overrides-subdomain.allow",
        ]
        block = ["sub-less.block"]
        "#;

        let config: Config = toml::from_str(sample_config).unwrap();
        insta::assert_debug_snapshot!(config);

        let Config {
            url_filters: UrlFilters { allow, .. },
        } = config;

        let contains = [
            "https://sub-less.allow",
            "http://with-sub.sub-less.allow",
            "http://a.sub.allow",
        ];
        for url_str in contains {
            let url = Url::parse(url_str).unwrap();
            assert!(allow.contains(&url), "Url: {url_str}");
        }

        let no_contains = [
            "https://sub.allow",
            "https://c.sub.allow",
            "http://something.random",
        ];
        for url_str in no_contains {
            let url = Url::parse(url_str).unwrap();
            assert!(!allow.contains(&url), "Url: {url_str}");
        }
    }
}
