use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use std::env;

/**
 * Create Github_Auth Client
 * Internal function
 */
pub fn oauth_client() -> BasicClient {
    let client_id = ClientId::new(env::var("GITHUB_CLIENT_ID").expect("GITHUB_CLIENT_ID not set"));

    let client_secret =
        ClientSecret::new(env::var("GITHUB_CLIENT_SECRET").expect("GITHUB_CLIENT_SECRET not set"));

    let auth_url = AuthUrl::new("https://github.com/login/oauth/authorize".to_string())
        .expect("Invalid authorization endpoint URL");

    let token_url = TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
        .expect("Invalid token endpoint URL");

    let redirect_url = RedirectUrl::new("http://localhost:3000/auth/github/callback".to_string())
        .expect("Invalid redirect URL");

    BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
        .set_redirect_uri(redirect_url)
}
