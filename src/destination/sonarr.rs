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
        seed_ratio: Option<f32>,
        seed_time: Option<usize>,             // minutes
        season_pack_seed_time: Option<usize>, // minutes
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
                minimum_seeders: from
                    .fields
                    .iter()
                    .find_map(|f| match f {
                        Field::MinimumSeeders { value } => Some(value.to_owned()),
                        _ => None,
                    })
                    .expect("minimumSeeders field should always be present for torrent indexers"),
                seed_time: from
                    .fields
                    .iter()
                    .find_map(|f| match f {
                        Field::SeedTime { value } => Some(value.to_owned()),
                        _ => None,
                    })
                    .expect("seedCriteria.seedTime field should always be present for torrent indexers"),
                season_pack_seed_time: from
                    .fields
                    .iter()
                    .find_map(|f| match f {
                        Field::SeasonPackSeedTime { value } => Some(value.to_owned()),
                        _ => None,
                    })
                    .expect("seedCriteria.seasonPackSeedTime field should always be present for torrent indexers"),
            },
        };

        SonarrIndexer {
            id: from.id,
            name: from.name,
            api_key,
            anime_categories,
            categories,
            url: Url::parse(&base_url).ok().unwrap_or_else(default_url),
            implementation: from.implementation,
            config_contract: from.config_contract,
            protocol,
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
            fields,
            protocol,
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

    /// Additional Torznab/Newznab parameters (appended to request URL by Sonarr)
    AdditionalParameters {
        #[serde(default)]
        value: String,
    },

    /// The ratio a torrent should reach before stopping, empty is download client's default
    #[serde(rename = "seedCriteria.seedRatio")]
    SeedRatio {
        #[serde(default)]
        value: Option<f32>,
    },

    /// The time a torrent should be seeded before stopping, empty is download client's default
    #[serde(rename = "seedCriteria.seedTime")]
    SeedTime {
        #[serde(default)]
        value: Option<usize>,
    },

    /// The time a torrent should be seeded before stopping, empty is download client's default
    #[serde(rename = "seedCriteria.seasonPackSeedTime")]
    SeasonPackSeedTime {
        #[serde(default)]
        value: Option<usize>,
    },
    #[serde(other)]
    Other,
}

impl SonarrIndexer {
    async fn save(&mut self, target: &Sonarr) -> Result<(), Box<dyn std::error::Error>> {
        let (method, path) = if let Some(id) = &self.id {
            log::info!("Updating {} in Sonarr (id: {})", &self.name, &id);
            (reqwest::Method::PUT, format!("/api/v3/indexer/{}", &id))
        } else {
            log::info!("Creating {} in Sonarr", &self.name);
            (reqwest::Method::POST, "/api/v3/indexer".to_owned())
        };

        let response: reqwest::Response = target
            .client
            .request(method.clone(), target.url.join(&path)?)
            .json(&self)
            .send()
            .await?;

        log::debug!(
            "    -> {} {} ({}) - {}",
            &method,
            &path,
            &self.name,
            response.status()
        );
        log::debug!("    -> TV categories: {:?}", &self.categories);
        log::debug!("    -> Anime categories: {:?}", &self.anime_categories);

        match response.status() {
            status if status.is_success() => {
                if status == reqwest::StatusCode::CREATED || status == reqwest::StatusCode::ACCEPTED
                {
                    *self = response.json().await?;
                    log::debug!("    <- {}", &self.id.unwrap());
                }
                Ok(())
            }
            status if status.is_client_error() => {
                let response = response.text().await?;
                log::debug!("    <- {}", &response);

                let response = serde_json::from_str::<serde_json::Value>(&response)?;
                log::error!("    <- {}", response[0]["errorMessage"].as_str().unwrap());

                Err(Box::new(crate::Error("Save rejected".to_owned()))
                    as Box<dyn std::error::Error>)
            }
            _ => Ok(response.error_for_status().map(|_| ())?),
        }
    }
}
struct IndexerSchemas(Vec<SonarrIndexer>);

impl<'a> IndexerSchemas {
    fn find(&'a self, kind: Implementation) -> &SonarrIndexer {
        let Self(schemas) = self;
        schemas
            .iter()
            .find(|schema| schema.implementation == kind)
            .unwrap_or_else(|| panic!("A schema of type {:?} is expected", kind))
    }
}

impl Sonarr {
    async fn schemas(&self) -> Result<IndexerSchemas, Box<dyn std::error::Error>> {
        let schemas: Vec<SonarrIndexer> = self
            .client
            .get(self.url.join("/api/v3/indexer/schema")?)
            .send()
            .await?
            .json()
            .await?;

        // Filter out schemas we don't know anything about
        let schemas = schemas
            .into_iter()
            .filter(|i| i.config_contract != ConfigContract::Other)
            .collect();

        log::trace!("Fetched indexer schemas {:?}", schemas);

        Ok(IndexerSchemas(schemas))
    }

    async fn existing_indexers(&self) -> Result<Vec<SonarrIndexer>, Box<dyn std::error::Error>> {
        let indexers: Vec<SonarrIndexer> = self
            .client
            .get(self.url.join("/api/v3/indexer")?)
            .send()
            .await?
            .json()
            .await?;

        // Filter out schemas we don't know anything about
        let indexers = indexers
            .into_iter()
            .filter(|i| i.config_contract != ConfigContract::Other)
            .collect();

        log::trace!("Fetched existing indexers {:?}", indexers);

        Ok(indexers)
    }

    pub async fn update_indexers(
        self,
        indexers: &[crate::Indexer],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let schemas = self.schemas().await?;
        let mut existing_indexers = self.existing_indexers().await?;

        for indexer in indexers {
            log::trace!("Processing {:?}", indexer);

            let mut new_indexer;
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
                        sonarr_indexer = existing_indexer
                    } else {
                        new_indexer = schemas.find(Implementation::Newznab).clone();
                        sonarr_indexer = &mut new_indexer;
                    };

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
                        new_indexer = schemas.find(Implementation::Torznab).clone();
                        sonarr_indexer = &mut new_indexer;
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
                        new_indexer = schemas.find(Implementation::TorrentRssIndexer).clone();
                        sonarr_indexer = &mut new_indexer;
                    }

                    sonarr_indexer.url = feed.0.to_owned();
                }
                _ => {
                    continue;
                }
            }

            sonarr_indexer.name = format!("{} {{{}}}", indexer.name, indexer.source.name_id());

            // We want to proceed, even if save() returns an Err. For now, the
            // error-handling (just logging) is inlined into the save() method.
            // I'd like it to live here but unfortunately Reqwest's error won't
            // give access to the response body, which has further error
            // details. Eventually, I'll wrap my own error type in there and do
            // some handling here.
            let _ = sonarr_indexer.save(&self).await;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_sonarr_schemas() -> serde_json::Result<()> {
        let schema_blob = include_str!("../../test/sonarr-schemas.json");
        let schemas = serde_json::from_str::<Vec<SonarrIndexer>>(schema_blob)?;
        assert_eq!(schemas[0].id, None);
        Ok(())
    }

    #[test]
    fn test_deserialize_sonarr_existing_indexer() -> serde_json::Result<()> {
        let schema_blob = include_str!("../../test/sonarr-indexers.json");
        let indexers = serde_json::from_str::<Vec<SonarrIndexer>>(schema_blob)?;
        assert_eq!(indexers[0].id, Some(1));
        Ok(())
    }
}
