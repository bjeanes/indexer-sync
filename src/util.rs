use url::Url;

/// Checks if a passed string URL is a parseable URL with a https:// or http:// scheme
pub fn is_http_url(url: &str) -> Result<(), String> {
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
