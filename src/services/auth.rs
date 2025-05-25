use crate::models::user::User;
use crate::services::db;
use crate::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    response::Redirect,
};
use chrono::Utc;
use mongodb::bson::oid::ObjectId;
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use redis::AsyncCommands;
use reqwest::Client;
use serde::Deserialize;
use std::env;

/**
 * Create Github_Auth Client
 * Internal function
 */
fn oauth_client() -> BasicClient {
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

/**
 * fetch user profile when auth_token is fetched
 * Internal function
 */
async fn fetch_github_user(auth_token: &str) -> Result<GithubUser, (StatusCode, String)> {
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

/**
 * Route to redirect to GithubAuth
 * Public function
*/
pub async fn github_login(State(state): State<AppState>) -> Redirect {
    let client = oauth_client();

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

/**
 * get auth_token if csrf_token is present in redis
 * create user with fetched github details
 * Public function
*/
pub async fn github_callback(
    Query(params): Query<QueryParams>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    //get params
    let code = params
        .code
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing code".to_string()))?;

    let received_state = params
        .state
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing State".to_string()))?;

    let state_key = format!("oauth_state:{}", received_state);

    //get redisConnectionManager
    let mut redis_conn = state.redis.clone();

    //check if state is present in redis or not
    let exits: bool = redis_conn.exists(&state_key).await.map_err(|_err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error getting state from redis".to_string(),
        )
    })?;

    if !exits {
        return Err((
            StatusCode::BAD_REQUEST,
            "Can't authorise, Please Try Again".to_string(),
        ));
    }

    //delete redis entry after authorising user
    let _: () = redis_conn.del(&state_key).await.map_err(|_err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to delete state in Redis".to_string(),
        )
    })?;

    let client = oauth_client();

    //create token exchange response
    let token_res = client
        .exchange_code(AuthorizationCode::new(code))
        .request_async(async_http_client)
        .await
        .map_err(|_err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Token exchange failed"),
            )
        })?;

    //get access token
    let access_token = token_res.access_token().secret();

    //fetch user_info
    let user_info = fetch_github_user(access_token).await?;

    //create user_model using impl
    let user_model: User = (user_info, access_token.to_string()).into();

    //create user entry in DB
    let _ = db::create_user(user_model, State(state))
        .await
        .map_err(|_error| {
            (
                StatusCode::BAD_REQUEST,
                format!("Error creating User in DB"),
            )
        })?;

    Ok((
        StatusCode::OK,
        format!("User created succesfully").to_string(),
    ))
}

//Structs

#[derive(Deserialize)]
pub struct QueryParams {
    code: Option<String>,
    state: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct GithubUser {
    login: String,
    name: Option<String>,
    email: Option<String>,
    avatar_url: Option<String>,
}

impl From<(GithubUser, String)> for User {
    fn from((github_user, access_token): (GithubUser, String)) -> Self {
        User {
            id: ObjectId::new(),
            github_id: github_user.login.to_string(),
            name: github_user.name,
            email: github_user.email,
            avatar_url: github_user.avatar_url,
            access_token: Some(access_token),
            projects: vec![],
            created_at: Utc::now(),
        }
    }
}
