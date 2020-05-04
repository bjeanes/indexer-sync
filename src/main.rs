use clap::{crate_authors, crate_version, ArgGroup, Clap};
use reqwest::Client;
use serde::Deserialize;
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum JacketIndexerType {
    Public,
    Private,
    SemiPrivate,
}

#[derive(Debug, Deserialize)]
struct JackettIndexer {
    id: String,
    name: String,
    // description: String,
    configured: bool,

    #[serde(rename = "site_link")]
    primary_site_link: String,

    #[serde(rename = "alternativesitelinks")]
    site_links: Vec<String>,

    #[serde(rename = "type")]
    _type: JacketIndexerType,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();
    let client = Client::builder()
        .cookie_store(true)
        .gzip(true)
        .build()
        .unwrap();

    if let Some(jackett) = opts.jackett.clone() {
        // TODO: handle when auth is required
        // Fill cookie store
        let _ = client.get(jackett.clone()).send().await?;
        let res = client
            .get(
                jackett
                    .clone()
                    .join("/api/v2.0/indexers?configured=true")
                    .unwrap(),
            )
            .send()
            .await?;
        println!("{:?}", res.json::<Vec<JackettIndexer>>().await?);
    }

    println!("{:?}", opts);

    Ok(())
}
