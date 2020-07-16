use crate::Config;
use oauth2::basic::BasicClient;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};

pub fn oauth_client(config: &Config) -> BasicClient {
    BasicClient::new(
        ClientId::new(config.google_id.clone()),
        Some(ClientSecret::new(config.google_secret.clone())),
        AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_owned()).unwrap(),
        Some(TokenUrl::new("https://oauth2.googleapis.com/token".to_owned()).unwrap()),
    )
    .set_redirect_url(
        RedirectUrl::new(format!("{}/login/google/redirect", config.base_url)).unwrap(),
    )
}
