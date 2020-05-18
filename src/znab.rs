#![allow(clippy::trivially_copy_pass_by_ref)]

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Capability {
    #[serde(rename = "ID")]
    pub id: String,

    #[serde(rename = "Name")]
    name: String,
}

pub trait Capabilities {
    fn series(&self) -> Vec<Capability>;
    fn movies(&self) -> Vec<Capability>;

    // TODO some indexers have categories like "TV/Anime" but some are just
    // "Anime", so this may need to be a bit more nuanced in the future. For
    // now, I'll just lump all anime together.
    fn anime(&self) -> Vec<Capability>;
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

fn exclude_anime(cap: &&Capability) -> bool {
    let name = cap.name.to_ascii_lowercase();
    !name.contains("anime")
}

fn exclude_porn(cap: &&Capability) -> bool {
    let name = cap.name.to_ascii_lowercase();
    !(name.contains("xxx")
        || name.contains("adult")
        || name.contains("porn")
        || name.contains("hentai"))
}

impl Capabilities for Vec<Capability> {
    fn series(&self) -> Vec<Capability> {
        self.iter()
            .filter(|cap| {
                let name = cap.name.to_ascii_lowercase();
                name.contains("tv") || name.contains("series") || name.contains("episodes")
            })
            .filter(exclude_anime)
            .filter(exclude_porn)
            .map(|cap| cap.to_owned())
            .collect()
    }

    fn anime(&self) -> Vec<Capability> {
        self.iter()
            .filter(|cap| cap.name.to_ascii_lowercase().contains("anime"))
            .filter(exclude_porn)
            .map(|cap| cap.to_owned())
            .collect()
    }

    fn movies(&self) -> Vec<Capability> {
        self.iter()
            .filter(|cap| {
                let name = cap.name.to_ascii_lowercase();
                name.contains("movie") || name.contains("film") || name.contains("movs")
            })
            .filter(exclude_anime)
            .filter(exclude_porn)
            .map(|cap| cap.to_owned())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::lazy_static;

    lazy_static! {
        static ref CAPS: Vec<Capability> =
            // serde_json::from_str(include_str!("../test/some-caps.json")).unwrap();
            serde_json::from_str(include_str!("../test/all-caps.json")).unwrap();
    }

    // TODO: make a proc_macro so that the $name:ident can be generated or
    // derived from the $val:literal
    macro_rules! test_caps_filter {
        ($name:ident: $val:literal,
            included: [ $($included_filter:ident),* ],
            excluded: [ $($excluded_filter:ident),* ]) => {

            // generate an inclusion test for each $included_filter
            $(
                paste::item! {
                    #[test]
                    fn [<test_ $name _included_by_ $included_filter>]() {
                        let caps = CAPS.$included_filter();
                        let caps: Vec<_> = caps.iter().map(|cap| cap.name.as_str()).collect();

                        assert!(
                            caps.contains(&$val),
                            "{} capabilities did not contain {:?} but should have",
                            stringify!($included_filter),
                            &$val
                        )
                    }
                }
            )*

            // generate an exclusion test for each $excluded_filter
            $(
                paste::item! {
                    #[test]
                    fn [<test_ $name _excluded_by_ $excluded_filter>]() {
                        let caps = CAPS.$excluded_filter();
                        let caps: Vec<_> = caps.iter().map(|cap| cap.name.as_str()).collect();

                        assert!(
                            !caps.contains(&$val),
                            "{} capabilities included {:?} but should not have",
                            stringify!($excluded_filter),
                            &$val
                        )
                    }
                }
            )*

        }
    }

    // TV only
    test_caps_filter!(tv: "TV", included: [series], excluded: [anime, movies]);
    test_caps_filter!(tv_slash_hd: "TV/HD", included: [series], excluded: [anime, movies]);
    test_caps_filter!(tv_slash_sd: "TV/SD", included: [series], excluded: [anime, movies]);
    test_caps_filter!(tv_slash_uhd: "TV/UHD", included: [series], excluded: [anime, movies]);
    test_caps_filter!(tv_slash_4k: "TV/4K", included: [series], excluded: [anime, movies]);
    test_caps_filter!(tv_space_hd: "TV HD", included: [series], excluded: [anime, movies]);
    test_caps_filter!(tv_space_sd: "TV SD", included: [series], excluded: [anime, movies]);
    test_caps_filter!(tv_space_uhd: "TV UHD", included: [series], excluded: [anime, movies]);
    test_caps_filter!(tv_space_4k: "TV 4K", included: [series], excluded: [anime, movies]);

    // Movies only
    test_caps_filter!(movies: "Movies", included: [movies], excluded: [anime, series]);
    test_caps_filter!(movies_slash_hd: "Movies/HD", included: [movies], excluded: [anime, series]);
    test_caps_filter!(movies_slash_sd: "Movies/SD", included: [movies], excluded: [anime, series]);
    test_caps_filter!(movies_slash_uhd: "Movies/UHD", included: [movies], excluded: [anime, series]);
    test_caps_filter!(movies_slash_4k: "Movies/4K", included: [movies], excluded: [anime, series]);
    test_caps_filter!(movies_space_hd: "Movies HD", included: [movies], excluded: [anime, series]);
    test_caps_filter!(movies_space_sd: "Movies SD", included: [movies], excluded: [anime, series]);
    test_caps_filter!(movies_space_uhd: "Movies UHD", included: [movies], excluded: [anime, series]);
    test_caps_filter!(movies_space_4k: "Movies 4K", included: [movies], excluded: [anime, series]);
    test_caps_filter!(films: "Films", included: [movies], excluded: [anime, series]);
    test_caps_filter!(film: "Film", included: [movies], excluded: [anime, series]);

    // Anime only
    test_caps_filter!(anime: "Anime", included: [anime], excluded: [series, movies]);
    test_caps_filter!(anime_space_cartoons: "Anime Cartoons", included: [anime], excluded: [series, movies]);
    test_caps_filter!(anime_space_dubbed: "Anime Dubbed", included: [anime], excluded: [series, movies]);
    test_caps_filter!(anime_space_subbed: "Anime Subbed", included: [anime], excluded: [series, movies]);
    test_caps_filter!(tv_space_anime: "TV Anime", included: [anime], excluded: [series, movies]);
    test_caps_filter!(tv_slash_anime: "TV/Anime", included: [anime], excluded: [series, movies]);

    // Exclude XXX
    test_caps_filter!(xxx: "XXX", included: [], excluded: [anime, series, movies]);
    test_caps_filter!(tv_slash_xxx: "TV/XXX", included: [], excluded: [anime, series, movies]);
    test_caps_filter!(movies_slash_xxx: "Movies/XXX", included: [], excluded: [anime, series, movies]);
    test_caps_filter!(films_slash_xxx: "Films/XXX", included: [], excluded: [anime, series, movies]);
    test_caps_filter!(xxx_dash_anime: "XXX-Anime", included: [], excluded: [anime, series, movies]);
    test_caps_filter!(xxx_anime_hentai: "XXX Anime / Hentai", included: [], excluded: [anime, series, movies]);
    test_caps_filter!(anime_hentai: "Anime - Hentai", included: [], excluded: [anime, series, movies]);
}
