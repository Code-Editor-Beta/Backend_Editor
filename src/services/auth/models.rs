use crate::models::user::User;
use chrono::Utc;
use mongodb::bson::oid::ObjectId;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct QueryParams {
    pub code: Option<String>,
    pub state: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct GithubUser {
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
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
