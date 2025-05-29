use crate::services::auth::models::GithubUser;
use axum::http::StatusCode;
use reqwest::Client;

/**
 * fetch user profile when auth_token is fetched
 * Internal function
 */
pub async fn fetch_github_user(auth_token: &str) -> Result<GithubUser, (StatusCode, String)> {
    let client = Client::new();

    let response = client
        .get("https://api.github.com/user")
        .bearer_auth(auth_token)
        .header("User-Agent", "Backend_Rust")
        .send()
        .await
        .map_err(|_err| (StatusCode::BAD_REQUEST, "Error getting user".to_string()))?;

    let response = response.error_for_status().map_err(|err| {
        (
            StatusCode::BAD_REQUEST,
            format!("GitHub API error: {:?}", err),
        )
    })?;

    let user_info = response.json::<GithubUser>().await.map_err(|err| {
        (
            StatusCode::BAD_REQUEST,
            format!("Failed to parse user: {:?}", err),
        )
    })?;
    Ok(user_info)
}
