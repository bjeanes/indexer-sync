use clap::{crate_authors, crate_version, ArgGroup, Clap};
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use url::Url;

fn is_http_url(url: &str) -> Result<(), String> {
    match Url::parse(url) {
        Err(e) => Err(e.to_string()),
        Ok(url) => match url.scheme() {
            "http" | "https" => Ok(()),
            scheme => Err(
                format!("URL must be an http:// or https:// URL (given {})", scheme).to_string(),
            ),
        },
    }
}

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

#[derive(Debug, Deserialize)]
struct JackettIndexer {
    id: String,
    name: String,

    #[serde(rename = "type")]
    privacy: IndexerPrivacy,
}

#[derive(Debug)]
enum SourceIndexer {
    Jackett(JackettIndexer),
}

#[derive(Debug)]
enum FeedUrl {
    Torznab { url: Url, api_key: Option<String> },
    Potato { url: Url, api_key: Option<String> },
    RSS(Url),
}

#[derive(Debug)]
struct Indexer {
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
    let client = Client::builder()
        .cookie_store(true)
        .gzip(true)
        .build()
        .unwrap();

    let mut indexers: Vec<Indexer> = vec![];

    if let Some(url) = opts.jackett.clone() {
        let url = url.join("/api/v2.0/").unwrap();

        // TODO: handle when auth is required
        // Fill cookie store
        let _ = client.get(url.clone()).send().await?;
        let server_config: Value = client
            .get(url.clone().join("server/config").unwrap())
            .send()
            .await?
            .json()
            .await?;

        let jackett_api_key = server_config["api_key"]
            .as_str()
            .expect("API Key for Jackett could not be found.");

        println!("{:?}", jackett_api_key);

        let jacket_indexers: Vec<JackettIndexer> = client
            .get(url.clone().join("indexers?configured=true").unwrap())
            .send()
            .await?
            .json()
            .await?;

        indexers.extend(jacket_indexers.into_iter().map(|ind| {
            let results_url = url
                .join(&format!("indexers/{}/results/torznab", &ind.id))
                .unwrap();
            Indexer {
                name: format!("{} [jackett:{}]", &ind.name, &ind.id),
                urls: vec![
                    FeedUrl::Torznab {
                        url: results_url.join("torznab").unwrap(),
                        api_key: Some(jackett_api_key.to_owned()),
                    },
                    FeedUrl::Potato {
                        url: results_url.join("potato").unwrap(),
                        api_key: Some(jackett_api_key.to_owned()),
                    },
                    FeedUrl::RSS({
                        let mut rss_url = url.join("rss").unwrap();
                        rss_url
                            .query_pairs_mut()
                            .append_pair("api_key", &jackett_api_key);
                        rss_url
                    }),
                ],
                privacy: ind.privacy,
                source: SourceIndexer::Jackett(ind),
            }
        }));

        println!("{:?}", indexers);
    }

    println!("{:?}", opts);

    Ok(())
}
