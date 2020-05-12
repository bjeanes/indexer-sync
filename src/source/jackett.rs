use serde::Deserialize;
use serde_json::Value;
use url::Url;

use crate::FeedUrls;
use crate::IndexerPrivacy;
use crate::SourceIndexer;
use crate::{Potato, Torznab, RSS};

#[derive(Debug, Deserialize)]
pub struct Indexer {
    pub id: String,
    pub name: String,

    #[serde(rename = "potatoenabled")]
    pub potato_enabled: bool,

    #[serde(rename = "type")]
    pub privacy: IndexerPrivacy,

    #[serde(rename = "caps")]
    pub capabilities: Vec<crate::Capability>,
}

pub struct Jackett {
    url: Url,
    feed_api_key: String,
    client: reqwest::Client,
}

// Jackett does not have an "official" API. The endpoints we need to hit to
// get the list of indexers and the API key use session authentication. So,
// we hit the homepage once to fill the cookie store with the requisite
// cookie.
async fn auth(url: Url) -> Result<(Url, reqwest::Client), Box<dyn std::error::Error>> {
    let (url, pw) = crate::util::extract_single_auth_value(url);
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .gzip(true)
        .build()?;

    let dashboard = url.join("/UI/Dashboard")?;
    let err = Err(
        Box::new(crate::Error("Jackett password incorrect".to_owned()))
            as Box<dyn std::error::Error>,
    );

    let initial_request = client.get(dashboard.clone()).send().await?;

    if initial_request.url().path() != dashboard.path() {
        if let Some(pw) = pw {
            let login_request = client
                .post(dashboard.clone())
                .form(&[("password", pw)])
                .send()
                .await?;
            if login_request.url().path() != dashboard.path() {
                return err;
            }
        } else {
            return err;
        }
    }

    Ok((url, client))
}

pub async fn new(url: Url) -> Result<Jackett, Box<dyn std::error::Error>> {
    let (url, client) = auth(url).await?;
    let url = url.join("/api/v2.0/")?;

    // Fetch Jackett configuration as JSON
    let config: Value = client
        .get(url.clone().join("server/config")?)
        .send()
        .await?
        .json()
        .await?;

    let api_key = config["api_key"].as_str().ok_or("Unable to find API key")?;

    Ok(Jackett {
        url,
        client,
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
                    name: ind.name.clone(),
                    urls: FeedUrls {
                        newznab: None,
                        torznab: Some(Torznab {
                            url: results_url.join("torznab").unwrap(),
                            api_key: Some(self.feed_api_key.to_owned()),
                            capabilities: ind.capabilities.clone(),
                        }),
                        potato: if ind.potato_enabled {
                            Some(Potato {
                                url: results_url.join("potato").unwrap(),
                                api_key: Some(self.feed_api_key.to_owned()),
                            })
                        } else {
                            None
                        },
                        rss: Some(RSS({
                            let mut rss_url = self.url.join("rss").unwrap();
                            rss_url
                                .query_pairs_mut()
                                .append_pair("api_key", &self.feed_api_key);
                            rss_url
                        })),
                    },
                    privacy: ind.privacy,
                    source: SourceIndexer::Jackett(ind),
                }
            })
            .collect();
        Ok(indexers)
    }
}
