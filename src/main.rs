use clap::{crate_authors, crate_version, ArgGroup, Clap};
use serde::Deserialize;
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

use util::is_http_url;

/// At least one [src] and at least one [dst] must be specified in order to sync.
#[derive(Clap, Debug)]
#[clap(version = crate_version!(), author = crate_authors!(), group = ArgGroup::with_name("src").multiple(true).required(true), group = ArgGroup::with_name("dst").multiple(true).required(true))]
struct Opts {
    /// [src] URL to Jackett instance from where indexers should be sourced
    /// Basic Auth credentials will be extracted and used as admin password.
    #[clap(long, validator = is_http_url, env = "SYNC_JACKETT_URL", group = "src")]
    jackett: Option<Url>,

    /// [dst] URL to Sonarr instance where indexers should be updated. Encoded
    /// Basic Auth credentials will be extracted and used as the API token.
    #[clap(long, validator = is_http_url, env = "SYNC_SONARR_URL", group = "dst")]
    sonarr: Option<Url>,

    /// [dst] URL to Radarr instance where indexers should be updated. Encoded
    /// Basic Auth credentials will be extracted and used as the API token.
    #[clap(long, validator = is_http_url, env = "SYNC_RADARR_URL", group = "dst")]
    radarr: Option<Url>,

    /// Provide indexers that you want to update. These values will be case-insensitively substring
    /// matched against indexer/tracker names. Only those which match will be synced. If not
    /// provided, all discovered indexers will be synced.
    #[clap(name = "INDEXER")]
    indexers_to_sync: Vec<String>,
    //

    // /// [dst] URL to Lidarr instance where indexers should be updated
    // #[clap(long, validator = is_http_url, env = "SYNC_LIDARR_URL", group = "dst")]
    // lidarr: Option<Url>,

    // /// [dst] URL to CouchPotato instance where indexers should be updated
    // #[clap(long = "cp", validator = is_http_url, env = "SYNC_LIDARR_URL", group = "dst")]
    // couch_potato: Option<Url>,

    // /// [src] URL to Cardigann instance from where indexers should be sourced
    // #[clap(long, validator = is_http_url, env = "SYNC_CARDIGANN_URL", group = "src")]
    // cardigann: Option<Url>,

    // /// [src] URL to NZBHydra2 instance from where indexers should be sourced
    // #[clap(long, validator = is_http_url, env = "SYNC_NZBHYDRA2_URL", group = "src")]
    // nzbhydra2: Option<Url>,

    // /// The interval (in seconds) between sync runs.
    // #[clap(short, long, default_value = "300", env = "SYNC_INTERVAL", group = "dst")]
    // interval: usize,

    // /// Run the sync once, then exit
    // #[clap(short, long)]
    // once_off: bool
}

#[derive(Debug, Deserialize, Clone, Copy)]
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

    let opts: Opts = Opts::parse();

    let mut indexers = vec![];
    let mut updates = vec![];
    let sonarr: sonarr::Sonarr;

    // FETCH

    if let Some(url) = opts.jackett.clone() {
        log::info!("Fetching indexers from Jackett");
        let jackett = jackett::new(url).await?;
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

    if opts.indexers_to_sync.len() > 0 {
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
        if let Some(url) = opts.sonarr.clone() {
            log::info!("Updating indexers in Sonarr");
            sonarr = sonarr::new(url)?;
            updates.push(sonarr.update_indexers(&indexers));
        }

        for future in updates {
            future.await?;
        }
    }

    Ok(())
}
