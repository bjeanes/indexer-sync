use clap::{crate_authors, crate_version, ArgGroup, Clap};
use serde::Deserialize;
use url::Url;

mod jackett;
mod util;

use util::is_http_url;

/// At least one [src] and at least one [dst] must be specified in order to sync.
#[derive(Clap, Debug)]
#[clap(version = crate_version!(), author = crate_authors!(), group = ArgGroup::with_name("src").multiple(true).required(true), group = ArgGroup::with_name("dst").multiple(true).required(true))]
struct Opts {
    /// [src] URL to Jackett instance from where indexers should be sourced
    #[clap(long, validator = is_http_url, env = "SYNC_JACKETT_URL", group = "src")]
    jackett: Option<Url>,

    /// [dst] URL to Sonarr instance where indexers should be updated
    #[clap(long, validator = is_http_url, env = "SYNC_SONARR_URL", group = "dst")]
    sonarr: Option<Url>,

    /// [dst] URL to Radarr instance where indexers should be updated
    #[clap(long, validator = is_http_url, env = "SYNC_RADARR_URL", group = "dst")]
    radarr: Option<Url>,
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
    // once_off: bool,
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
enum IndexerPrivacy {
    Public,
    Private,
    SemiPrivate,
}

#[derive(Debug)]
enum SourceIndexer {
    Jackett(jackett::Indexer),
}

#[derive(Debug)]
enum FeedUrl {
    Torznab { url: Url, api_key: Option<String> },
    Potato { url: Url, api_key: Option<String> },
    RSS(Url),
}

#[derive(Debug)]
pub struct Indexer {
    source: SourceIndexer,
    name: String,
    // TODO: is there a way to model this to only allow construction of an
    // Indexer struct that has one or more URLs?
    urls: Vec<FeedUrl>,
    privacy: IndexerPrivacy,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();

    if let Some(url) = opts.jackett.clone() {
        let jackett = jackett::new(url).await?;
        println!("{:?}", jackett.fetch_indexers().await?);
    }

    println!("{:?}", opts);

    Ok(())
}
