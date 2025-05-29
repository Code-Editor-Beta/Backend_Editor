use serde::Deserialize;

#[derive(Deserialize)]
pub struct OpenFileParams {
    project_id: String,
    filename: String,
}
