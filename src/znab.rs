use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Capability {
    #[serde(rename = "ID")]
    pub id: String,

    #[serde(rename = "Name")]
    name: String,
}

pub trait Capabilities<'a> {
    fn series(&'a self) -> Vec<Capability>;
    fn movies(&'a self) -> Vec<Capability>;

    // TODO some indexers have categories like "TV/Anime" but some are just
    // "Anime", so this may need to be a bit more nuanced in the future. For
    // now, I'll just lump all anime together.
    fn anime(&'a self) -> Vec<Capability>;
}

pub trait Ids<T> {
    fn ids(&self) -> Vec<T>;
}

impl Ids<String> for Vec<Capability> {
    fn ids(&self) -> Vec<String> {
        self.iter().map(|cap| cap.id.clone()).collect()
    }
}

impl Ids<usize> for Vec<Capability> {
    fn ids(&self) -> Vec<usize> {
        self.iter()
            .filter_map(|cap| cap.id.parse::<usize>().ok())
            .collect()
    }
}

impl<'a> Capabilities<'a> for Vec<Capability> {
    fn series(&'a self) -> Vec<Capability> {
        self.iter()
            .filter(|cap| cap.name.to_ascii_lowercase().contains("tv"))
            .filter(|cap| !cap.name.to_ascii_lowercase().contains("anime"))
            .map(|cap| cap.to_owned())
            .collect()
    }

    fn anime(&'a self) -> Vec<Capability> {
        self.iter()
            .filter(|cap| cap.name.to_ascii_lowercase().contains("anime"))
            .map(|cap| cap.to_owned())
            .collect()
    }

    fn movies(&'a self) -> Vec<Capability> {
        self.iter()
            .filter(|cap| cap.name.to_ascii_lowercase().contains("movie"))
            .filter(|cap| !cap.name.to_ascii_lowercase().contains("anime"))
            .map(|cap| cap.to_owned())
            .collect()
    }
}
