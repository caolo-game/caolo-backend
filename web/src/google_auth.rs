use crate::Config;
use oauth2::basic::BasicClient;
use oauth2::prelude::*;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, Scope, TokenUrl};
use url::Url;

pub fn oauth_client(config: &Config) -> BasicClient {
    BasicClient::new(
        ClientId::new(config.google_id.clone()),
        Some(ClientSecret::new(config.google_secret.clone())),
        AuthUrl::new(Url::parse("https://accounts.google.com/o/oauth2/v2/auth").unwrap()),
        Some(TokenUrl::new(
            Url::parse("https://oauth2.googleapis.com/token").unwrap(),
        )),
    )
    .add_scope(Scope::new("email".to_owned()))
    .set_redirect_url(RedirectUrl::new(
        Url::parse(&format!("{}/login/google/redirect", config.base_url)).unwrap(),
    ))
}
