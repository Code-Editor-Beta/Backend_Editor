use crate::{services::auth, AppState};
use axum::{extract::State, response::Redirect};

use oauth2::{CsrfToken, Scope};
use redis::AsyncCommands;

/**
 * Route to redirect to GithubAuth
 * Public function
*/
pub async fn github_login(State(state): State<AppState>) -> Redirect {
    let client = auth::outh_client::oauth_client();

    //get crsf_token to prevent hacking attacks
    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("read:user".to_string()))
        .url();

    //create state_key
    let state_key = format!("oauth_state:{}", csrf_token.secret());

    //get redisConnectionManager
    let mut redis_conn = state.redis.clone();

    // Save state token with 10 min expiry
    let _: () = redis_conn
        .set_ex(state_key, "valid", 600)
        .await
        .expect("Failed to store state in Redis");

    Redirect::to(auth_url.as_str())
}
