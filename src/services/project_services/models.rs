use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateProjectRequest {
    pub user_id: String,
    pub project_name: String,
    pub framework: String,
}
