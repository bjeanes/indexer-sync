use clap::{crate_authors, crate_version, ArgGroup, Clap};
use serde::Deserialize;
use std::time::Duration;
use url::Url;

mod destination;
mod error;
mod source;
mod util;
mod znab;

use destination::sonarr;
pub use error::*;
use source::jackett;
pub use znab::*;

/// At least one {src} and at least one {dst} must be specified in order to sync.
#[derive(Clap, Debug)]
#[clap(
    version = crate_version!(),
    author = crate_authors!(),
    group = ArgGroup::new("src").multiple(true).required(true),
    group = ArgGroup::new("dst").multiple(true).required(true),
    group = ArgGroup::new("tv").multiple(true),
)]
struct Opts {
    /// {src} Source indexers from this Jackett instance
    ///
    /// Basic Auth credentials will be extracted and used as admin password.
    #[clap(short = 'J', long, value_name = "URL", validator = util::is_http_url, env = "SYNC_JACKETT_URL", group = "src")]
    jackett: Option<Url>,

    /// {dst} Sync indexers to this Sonarr instance
    ///
    /// Encoded Basic Auth credentials will be extracted and used as the API token.
    #[clap(short = 'S', long, value_name = "URL", validator = util::is_http_url, env = "SYNC_SONARR_URL", group = "dst", group = "tv")]
    sonarr: Option<Url>,

    /// Polling mode. Sync every DURATION ("1h", "3s", etc)
    ///
    /// DURATION is parsed as per systemd. "1 hour 3 seconds", "1h", etc are all
    /// valid. If a single number with no unit is provided, it will be
    /// interpreted as seconds.
    #[clap(short, long, value_name = "DURATION", env = "SYNC_INTERVAL", parse(try_from_str = parse_duration::parse::parse))]
    interval: Option<Duration>,

    /// Limit synced endexers to those matching these terms
    ///
    /// Provide indexers that you want to update. These values will be case-insensitively substring
    /// matched against indexer/tracker names. Only those which match will be synced. If not
    /// provided, all discovered indexers will be synced.
    #[clap(value_name = "INDEXERS")]
    indexers_to_sync: Vec<String>,

    /// Target seed ratio for media media, for managers which support it ("1.0", "10", "0.1", etc)
    ///
    /// Defaults to manager default, if not provided.
    #[clap(long, value_name = "RATIO", env = "SYNC_SEED_RATIO")]
    seed_ratio: Option<f32>,

    /// Minimum time to seed media, for managers which support it ("1h", "2w", etc)
    ///
    /// Defaults to manager default, if not provided.
    #[clap(long, value_name = "DURATION", env = "SYNC_SEED_TIME", parse(try_from_str = parse_duration::parse::parse))]
    seed_time: Option<Duration>,

    /// Minimum time to seed a season pack, for managers which support it ("1h", "2w", etc)
    ///
    /// Defaults to `--seed-time`, if not provided.
    #[clap(long, value_name = "DURATION", env = "SYNC_SEASON_PACK_SEED_TIME", parse(try_from_str = parse_duration::parse::parse), requires = "tv")]
    season_pack_seed_time: Option<Duration>,

    /// target seed ratio for media from public trackers, for managers which support it ("1.0", "10", "0.1", etc)
    ///
    /// defaults to `--seed-ratio`, if not provided.
    #[clap(long, env = "sync_public_seed_ratio")]
    public_seed_ratio: Option<f32>,

    /// Minimum time to seed media from public trackers, for managers which support it ("1h", "2w", etc)
    ///
    /// Defaults to `--seed-time`, if not provided.
    #[clap(long, value_name = "DURATION", env = "SYNC_PUBLIC_SEED_TIME", parse(try_from_str = parse_duration::parse::parse))]
    public_seed_time: Option<Duration>,

    /// Minimum time to seed a season pack from public trackers, for managers which support it ("1h", "2w", etc)
    ///
    /// Defaults to `--public-seed-time`, if not provided.
    #[clap(long, value_name = "DURATION", env = "SYNC_PUBLIC_SEASON_PACK_SEED_TIME", parse(try_from_str = parse_duration::parse::parse), requires = "tv")]
    public_season_pack_seed_time: Option<Duration>,

    /// Target seed ratio for media from private trackers, for managers which support it ("1.0", "10", "0.1", etc)
    ///
    /// Defaults to `--seed-ratio`, if not provided.
    #[clap(long, env = "SYNC_PRIVATE_SEED_RATIO")]
    private_seed_ratio: Option<f32>,

    /// Minimum time to seed media from private trackers, for managers which support it ("1h", "2w", etc)
    ///
    /// Defaults to `--seed-time`, if not provided.
    #[clap(long, value_name = "DURATION", env = "SYNC_PRIVATE_SEED_TIME", parse(try_from_str = parse_duration::parse::parse))]
    private_seed_time: Option<Duration>,

    /// Minimum time to seed a season pack from private trackers, for managers which support it ("1h", "2w", etc)
    ///
    /// Defaults to `--private-seed-time`, if not provided.
    #[clap(long, value_name = "DURATION", env = "SYNC_PRIVATE_SEASON_PACK_SEED_TIME", parse(try_from_str = parse_duration::parse::parse), requires = "tv")]
    private_season_pack_seed_time: Option<Duration>,
}

#[derive(Default)]
pub struct SeedCriteria {
    seed_ratio: Option<f32>,
    seed_time: Option<Duration>,
    season_pack_seed_time: Option<Duration>,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum IndexerPrivacy {
    Public,
    Private,
    SemiPrivate,
}

#[derive(Debug)]
enum SourceIndexer {
    Jackett(jackett::Indexer),
}

impl SourceIndexer {
    pub fn name_id(&self) -> String {
        match self {
            Self::Jackett(ind) => format!("jackett:{}", &ind.id),
        }
    }
}

#[derive(Debug)]
struct Torznab {
    api_key: Option<String>,
    url: Url,
    capabilities: Vec<Capability>,
}
#[derive(Debug)]
struct Newznab {
    api_key: Option<String>,
    url: Url,
    capabilities: Vec<Capability>,
}

#[derive(Debug)]
struct Potato {
    api_key: Option<String>,
    url: Url,
}

#[derive(Debug)]
struct RSS(Url);

#[derive(Debug)]
struct FeedUrls {
    potato: Option<Potato>,
    rss: Option<RSS>,
    torznab: Option<Torznab>,
    newznab: Option<Newznab>,
}

#[derive(Debug)]
pub struct Indexer {
    source: SourceIndexer,
    name: String,
    urls: FeedUrls,
    privacy: IndexerPrivacy,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut log_builder = pretty_env_logger::formatted_builder();
    match ::std::env::var("RUST_LOG") {
        Ok(s) if !s.is_empty() => {
            log_builder.parse_filters(&s);
        }
        _ => {
            #[cfg(not(debug_assertions))] // default production log-level
            log_builder.parse_filters("info");

            #[cfg(debug_assertions)] // default test/debug log-level
            log_builder.parse_filters("info,indexer_sync=debug");
        }
    }
    log_builder.try_init()?;

    let mut opts = Opts::parse();

    opts.private_seed_time = opts.private_seed_time.or_else(|| opts.seed_time);
    opts.public_seed_time = opts.public_seed_time.or_else(|| opts.seed_time);
    opts.private_seed_ratio = opts.private_seed_ratio.or_else(|| opts.seed_ratio);
    opts.public_seed_ratio = opts.public_seed_ratio.or_else(|| opts.seed_ratio);
    opts.season_pack_seed_time = opts.season_pack_seed_time.or_else(|| opts.seed_time);
    opts.private_season_pack_seed_time = opts
        .private_season_pack_seed_time
        .or_else(|| opts.private_seed_time);
    opts.public_season_pack_seed_time = opts
        .public_season_pack_seed_time
        .or_else(|| opts.public_seed_time);

    let opts = opts; // drop mut marker

    loop {
        let mut indexers = vec![];
        let mut updates = vec![];
        let sonarr: sonarr::Sonarr;

        // FETCH

        if let Some(ref url) = opts.jackett {
            log::info!("Fetching indexers from Jackett");
            let jackett = jackett::new(url.clone()).await?;
            let jackett_indexers = jackett.fetch_indexers().await?;
            log::debug!("Fetched: {}", {
                let mut i = jackett_indexers
                    .iter()
                    .map(|i| i.name.as_ref())
                    .collect::<Vec<&str>>();
                i.sort();
                i.join(", ")
            });
            indexers.extend(jackett_indexers);
        }

        // FILTER

        if !opts.indexers_to_sync.is_empty() {
            let filters: Vec<_> = opts
                .indexers_to_sync
                .iter()
                .map(|f| f.to_lowercase())
                .collect();

            indexers = indexers
                .into_iter()
                .filter(|i| filters.iter().any(|f| i.name.to_lowercase().contains(f)))
                .collect();

            log::debug!(
                "Filtered indexers to {}",
                if indexers.is_empty() {
                    "empty list".to_owned()
                } else {
                    indexers
                        .iter()
                        .map(|i| i.name.as_ref())
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            );
        }

        // UPDATE

        if indexers.is_empty() {
            log::warn!("No indexers to sync");
        } else {
            if let Some(ref url) = opts.sonarr {
                log::info!("Updating indexers in Sonarr");
                sonarr = sonarr::new(url.clone())?
                    .private_seed_criteria(SeedCriteria {
                        seed_time: opts.private_seed_time,
                        seed_ratio: opts.private_seed_ratio,
                        season_pack_seed_time: opts.private_season_pack_seed_time,
                    })
                    .public_seed_criteria(SeedCriteria {
                        seed_time: opts.public_seed_time,
                        seed_ratio: opts.public_seed_ratio,
                        season_pack_seed_time: opts.public_season_pack_seed_time,
                    });
                updates.push(sonarr.update_indexers(&indexers));
            }

            for future in updates {
                future.await?;
            }
        }

        if let Some(interval) = opts.interval {
            log::info!("Sleeping for {} seconds", interval.as_secs_f64());
            std::thread::sleep(interval);
        } else {
            break;
        }
    }

    Ok(())
}
