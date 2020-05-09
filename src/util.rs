use url::Url;

/// Checks if a passed string URL is a parseable URL with a https:// or http:// scheme
pub fn is_http_url(url: &str) -> Result<(), String> {
    match Url::parse(url) {
        Err(e) => Err(e.to_string()),
        Ok(url) => match url.scheme() {
            "http" | "https" => Ok(()),
            scheme => Err(format!(
                "URL must be an http:// or https:// URL (given {})",
                scheme
            )),
        },
    }
}

pub fn extract_single_auth_value(url: Url) -> (Url, Option<String>) {
    match (url.username(), url.password()) {
        ("", None) => (url, None),
        (_, Some(pw)) | (pw, _) => {
            let mut url = url.clone();
            url.set_username("")
                .expect("This shouldn't fail in this use case");
            url.set_password(None)
                .expect("This shouldn't fail in this use case");
            (url, Some(pw.to_owned()))
        }
    }
}
