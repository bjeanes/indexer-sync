use crate::znab::{Capabilities, Ids};
use crate::FeedUrls;
use serde::{Deserialize, Serialize};
use url::Url;

pub struct Sonarr {
    url: Url,
    client: reqwest::Client,
}

pub fn new(url: Url) -> Result<Sonarr, Box<dyn std::error::Error>> {
    use reqwest::header::{self, HeaderMap, HeaderValue};

    let (url, api_key) = crate::util::extract_single_auth_value(url);
    let api_key = api_key.ok_or("Sonarr API key is required")?;

    let mut headers = HeaderMap::new();
    headers.insert("X-Api-Key", HeaderValue::from_str(&api_key)?);
    headers.insert(header::ACCEPT, HeaderValue::from_str("application/json")?);

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;
    Ok(Sonarr { client, url })
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Protocol {
    Torrent,
    Usenet,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ConfiguredProtocol {
    Torrent {
        minimum_seeders: usize,
        seed_ratio: f32,
        seed_time: usize,             // minutes
        season_pack_seed_time: usize, // minutes
    },
    Usenet,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum Implementation {
    Torznab,
    Newznab,
    TorrentRssIndexer,

    #[serde(other)]
    Other,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum ConfigContract {
    TorznabSettings,
    NewznabSettings,
    TorrentRssIndexerSettings,

    #[serde(other)]
    Other,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(into = "SonarrIndexerSchema", try_from = "SonarrIndexerSchema")]
struct SonarrIndexer {
    id: Option<usize>,
    name: String,
    api_key: String,
    categories: Vec<usize>,
    anime_categories: Vec<usize>,
    url: url::Url,
    implementation: Implementation,
    config_contract: ConfigContract,
    protocol: ConfiguredProtocol,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SonarrIndexerSchema {
    id: Option<usize>,
    config_contract: ConfigContract,
    enable_automatic_search: bool,
    enable_interactive_search: bool,
    enable_rss: bool,
    implementation: Implementation,
    // implementationName: String,
    // infoLink: url::Url,
    protocol: Protocol,
    name: String,
    supports_rss: bool,
    supports_search: bool,
    fields: Vec<Field>,
}

impl std::convert::From<SonarrIndexerSchema> for SonarrIndexer {
    fn from(from: SonarrIndexerSchema) -> Self {
        let api_key = from
            .fields
            .iter()
            .find_map({
                |f| match f {
                    Field::ApiKey { value } => Some(value.to_owned()),
                    _ => None,
                }
            })
            .unwrap_or_default();

        let base_url = from
            .fields
            .iter()
            .find_map({
                |f| match f {
                    Field::BaseUrl { value } => Some(value.to_owned()),
                    _ => None,
                }
            })
            // .unwrap_or_else(default_url);
            .unwrap_or_default();

        let categories = from
            .fields
            .iter()
            .find_map({
                |f| match f {
                    Field::Categories { value } => Some(value.to_owned()),
                    _ => None,
                }
            })
            .unwrap_or_default();

        let anime_categories = from
            .fields
            .iter()
            .find_map({
                |f| match f {
                    Field::AnimeCategories { value } => Some(value.to_owned()),
                    _ => None,
                }
            })
            .unwrap_or_default();

        let protocol = match from.protocol {
            Protocol::Usenet => ConfiguredProtocol::Usenet,
            Protocol::Torrent => ConfiguredProtocol::Torrent {
                seed_ratio: from
                    .fields
                    .iter()
                    .find_map(|f| match f {
                        Field::SeedRatio { value } => Some(value.to_owned()),
                        _ => None,
                    })
                    .expect("seedCriteria.seedRatio field should always be present for torrent indexers"),
                minimum_seeders: 1,
                seed_time: 300,
                season_pack_seed_time: 3000,
            },
        };

        SonarrIndexer {
            id: from.id,
            name: from.name,
            api_key: api_key,
            anime_categories: anime_categories,
            categories: categories,
            url: Url::parse(&base_url).ok().unwrap_or_else(default_url),
            implementation: from.implementation,
            config_contract: from.config_contract,
            protocol: protocol,
        }
    }
}

impl std::convert::From<SonarrIndexer> for SonarrIndexerSchema {
    fn from(from: SonarrIndexer) -> Self {
        let mut fields = vec![
            Field::BaseUrl {
                value: from.url.to_string(),
            },
            Field::ApiPath {
                value: "/api".to_owned(),
            },
            Field::ApiKey {
                value: from.api_key,
            },
            Field::Categories {
                value: from.categories,
            },
            Field::AnimeCategories {
                value: from.anime_categories,
            },
            Field::AdditionalParameters {
                value: "".to_owned(),
            },
        ];

        let mut protocol = Protocol::Usenet;
        if let ConfiguredProtocol::Torrent {
            season_pack_seed_time,
            seed_ratio,
            seed_time,
            minimum_seeders,
        } = from.protocol
        {
            protocol = Protocol::Torrent;
            fields.append(&mut vec![
                Field::MinimumSeeders {
                    value: minimum_seeders,
                },
                Field::SeedRatio { value: seed_ratio },
                Field::SeedTime { value: seed_time },
                Field::SeasonPackSeedTime {
                    value: season_pack_seed_time,
                },
            ]);
        }

        SonarrIndexerSchema {
            id: from.id,
            config_contract: from.config_contract,
            enable_automatic_search: true,
            enable_interactive_search: true,
            enable_rss: true,
            implementation: from.implementation,
            name: from.name,
            supports_rss: true,
            supports_search: true,
            // tags: vec![],
            fields: fields,
            protocol: protocol,
        }
    }
}

fn default_url() -> Url {
    Url::parse("http://example.com").unwrap()
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "name", rename_all = "camelCase")]
enum Field {
    BaseUrl {
        #[serde(default)]
        value: String,
    },
    ApiPath {
        #[serde(default)]
        value: String,
    },
    ApiKey {
        #[serde(default)]
        value: String,
    },
    Categories {
        #[serde(default)]
        value: Vec<usize>,
    },
    AnimeCategories {
        #[serde(default)]
        value: Vec<usize>,
    },
    MinimumSeeders {
        #[serde(default)]
        value: usize,
    },
    AdditionalParameters {
        #[serde(default)]
        value: String,
    },
    #[serde(rename = "seedCriteria.seedRatio")]
    SeedRatio {
        #[serde(default)]
        value: f32,
    },
    #[serde(rename = "seedCriteria.seedTime")]
    SeedTime {
        #[serde(default)]
        value: usize,
    },
    #[serde(rename = "seedCriteria.seasonPackSeedTime")]
    SeasonPackSeedTime {
        #[serde(default)]
        value: usize,
    },
    #[serde(other)]
    Other,
}

impl SonarrIndexer {
    async fn save(&mut self, target: &Sonarr) -> Result<(), Box<dyn std::error::Error>> {
        let (method, path) = if self.id.is_some() {
            (
                reqwest::Method::PUT,
                format!("/api/v3/indexer/{}", &self.id.unwrap()),
            )
        } else {
            (reqwest::Method::POST, "/api/v3/indexer".to_owned())
        };

        let response = target
            .client
            .request(method, target.url.join(&path)?)
            .json(&self)
            .send()
            .await?;

        println!("{} for saving {}", response.status(), &self.name);

        let response = response.error_for_status()?;

        *self = response.json().await?;

        Ok(())
    }
}

impl Sonarr {
    pub async fn update_indexers(
        self,
        indexers: &[crate::Indexer],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut schemas: Vec<SonarrIndexer> = self
            .client
            .get(self.url.join("/api/v3/indexer/schema")?)
            .send()
            .await?
            .json()
            .await?;

        let mut existing_indexers: Vec<SonarrIndexer> = self
            .client
            .get(self.url.join("/api/v3/indexer")?)
            .send()
            .await?
            .json()
            .await?;

        for indexer in indexers {
            let mut sonarr_indexer: &mut SonarrIndexer;
            let existing_indexer = existing_indexers
                .iter_mut()
                .find(|i| i.name.contains(&indexer.source.name_id()));

            match indexer.urls {
                FeedUrls {
                    newznab: Some(ref feed),
                    ..
                } => {
                    let existing_indexer =
                        existing_indexer.filter(|i| i.implementation == Implementation::Newznab);

                    if let Some(existing_indexer) = existing_indexer {
                        sonarr_indexer = existing_indexer;
                    } else {
                        sonarr_indexer = schemas
                            .iter_mut()
                            .find(|schema| schema.implementation == Implementation::Newznab)
                            .expect(&format!(
                                "A schema of type {:?} is expected",
                                Implementation::Newznab
                            ));
                    }

                    sonarr_indexer.api_key = feed.api_key.as_deref().unwrap_or("").to_owned();
                    sonarr_indexer.url = feed.url.to_owned();
                    sonarr_indexer.categories = feed.capabilities.series().ids();
                    sonarr_indexer.anime_categories = feed.capabilities.anime().ids();
                }
                FeedUrls {
                    torznab: Some(ref feed),
                    ..
                } => {
                    let existing_indexer =
                        existing_indexer.filter(|i| i.implementation == Implementation::Torznab);

                    if let Some(existing_indexer) = existing_indexer {
                        sonarr_indexer = existing_indexer;
                    } else {
                        sonarr_indexer = schemas
                            .iter_mut()
                            .find(|schema| schema.implementation == Implementation::Torznab)
                            .expect(&format!(
                                "A schema of type {:?} is expected",
                                Implementation::Torznab
                            ));
                    }

                    sonarr_indexer.api_key = feed.api_key.as_deref().unwrap_or("").to_owned();
                    sonarr_indexer.url = feed.url.to_owned();
                    sonarr_indexer.categories = feed.capabilities.series().ids();
                    sonarr_indexer.anime_categories = feed.capabilities.anime().ids();
                }
                FeedUrls {
                    rss: Some(ref feed),
                    ..
                } => {
                    let existing_indexer = existing_indexer
                        .filter(|i| i.implementation == Implementation::TorrentRssIndexer);

                    if let Some(existing_indexer) = existing_indexer {
                        sonarr_indexer = existing_indexer;
                    } else {
                        sonarr_indexer = schemas
                            .iter_mut()
                            .find(|schema| {
                                schema.implementation == Implementation::TorrentRssIndexer
                            })
                            .expect(&format!(
                                "A schema of type {:?} is expected",
                                Implementation::TorrentRssIndexer
                            ));
                    }

                    sonarr_indexer.url = feed.0.to_owned();
                }
                _ => {
                    continue;
                }
            }

            sonarr_indexer.name = format!("{} - {}", indexer.name, indexer.source.name_id());
            // sonarr_indexer.save(&self).await?;
            sonarr_indexer.save(&self).await;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_deserialize_sonarr_schemas() -> serde_json::Result<()> {
        let schema_blob = include_str!("../test/sonarr-schemas.json");
        let schemas = serde_json::from_str::<Vec<SonarrIndexer>>(schema_blob)?;
        assert_eq!(schemas[0].id, None);
        Ok(())
    }

    #[test]
    fn test_deserialize_sonarr_existing_indexer() -> serde_json::Result<()> {
        let schema_blob = include_str!("../test/sonarr-indexers.json");
        let indexers = serde_json::from_str::<Vec<SonarrIndexer>>(schema_blob)?;
        assert_eq!(indexers[0].id, Some(1));
        Ok(())
    }
}
