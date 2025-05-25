use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// #[derive(Serialize, Deserialize, Clone)]
// pub struct Project {
//     #[serde(rename = "_id")]
//     pub id: ObjectId,
//     pub name: String,
//     pub owner: ObjectId,
//     pub collaborators: Vec<ObjectId>,
//     pub files: Vec<ObjectId>,
//     pub github_repo: Option<String>,
//     pub is_private: bool,
//     pub template: String,
//     pub env_vars: Vec<EnvVar>,
//     pub created_at: DateTime<Utc>,
//     pub updated_at: DateTime<Utc>,
// }
#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub user_id: String,
    pub project_name: String,
    pub framework: String,
    pub created_at: DateTime<Utc>,
    pub files: HashMap<String, String>, // path => content
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EnvVar {
    pub key: String,
    pub value: String,
}
