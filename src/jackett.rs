use serde::Deserialize;
use serde_json::Value;
use url::Url;

use crate::FeedUrl;
use crate::IndexerPrivacy;
use crate::SourceIndexer;

#[derive(Debug, Deserialize)]
pub struct Indexer {
    id: String,
    name: String,

    #[serde(rename = "type")]
    privacy: IndexerPrivacy,
}

pub struct Jackett {
    url: Url,
    feed_api_key: String,
    client: reqwest::Client,
}

pub async fn new(url: Url) -> Result<Jackett, Box<dyn std::error::Error>> {
    let url = url.join("/api/v2.0/")?;

    let client = reqwest::Client::builder()
        .cookie_store(true)
        .gzip(true)
        .build()?;

    // Jackett does not have an "official" API. The endpoints we need to hit to
    // get the list of indexers and the API key use session authentication. So,
    // we hit the homepage once to fill the cookie store with the requisite
    // cookie.
    let _ = client.get(url.clone()).send().await?;

    // Fetch Jackett configuration as JSON
    let config: Value = client
        .get(url.clone().join("server/config")?)
        .send()
        .await?
        .json()
        .await?;

    let api_key = config["api_key"].as_str().ok_or("Unable to find API key")?;

    Ok(Jackett {
        url: url,
        client: client,
        feed_api_key: api_key.to_owned(),
    })
}

impl Jackett {
    pub async fn fetch_indexers(&self) -> Result<Vec<crate::Indexer>, Box<dyn std::error::Error>> {
        let jackett_indexers: Vec<Indexer> = self
            .client
            .get(self.url.clone().join("indexers?configured=true")?)
            .send()
            .await?
            .json()
            .await?;

        let indexers = jackett_indexers
            .into_iter()
            .map(|ind| {
                let results_url = self
                    .url
                    .join(&format!("indexers/{}/results/torznab", &ind.id))
                    .unwrap();

                crate::Indexer {
                    name: format!("{} [jackett:{}]", &ind.name, &ind.id),
                    urls: vec![
                        FeedUrl::Torznab {
                            url: results_url.join("torznab").unwrap(),
                            api_key: Some(self.feed_api_key.to_owned()),
                        },
                        FeedUrl::Potato {
                            url: results_url.join("potato").unwrap(),
                            api_key: Some(self.feed_api_key.to_owned()),
                        },
                        FeedUrl::RSS({
                            let mut rss_url = self.url.join("rss").unwrap();
                            rss_url
                                .query_pairs_mut()
                                .append_pair("api_key", &self.feed_api_key);
                            rss_url
                        }),
                    ],
                    privacy: ind.privacy,
                    source: SourceIndexer::Jackett(ind),
                }
            })
            .collect();
        Ok(indexers)
    }
}
