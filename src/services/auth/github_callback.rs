use crate::models::user::User;
use crate::services::auth::{fetch_gprofile, models::QueryParams, outh_client};
use crate::services::db;
use crate::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};

use oauth2::{reqwest::async_http_client, AuthorizationCode, TokenResponse};
use redis::AsyncCommands;

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

    let client = outh_client::oauth_client();

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
    let user_info = fetch_gprofile::fetch_github_user(access_token).await?;

    //create user_model using impl
    let user_model: User = (user_info, access_token.to_string()).into();

    //create user entry in DB
    let _ = db::create_db_entry::create_user(user_model, State(state))
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
