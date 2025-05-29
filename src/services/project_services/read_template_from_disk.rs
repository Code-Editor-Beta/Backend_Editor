use anyhow::Result;

use base64::{engine::general_purpose::STANDARD as B64, Engine as _};

use std::{collections::HashMap, path::PathBuf};
use tokio::task;
use walkdir::WalkDir;

/**
 * this reads all files under the root and return paths+content
 * if file is text -> store text
 * if file is binary -> store binary
 */
pub async fn read_template_from_disk(root: PathBuf) -> anyhow::Result<HashMap<String, String>> {
    task::spawn_blocking(move || {
        let mut map = HashMap::new();

        for entry in WalkDir::new(&root).into_iter().filter_map(Result::ok) {
            if entry.file_type().is_file() {
                let abs = entry.path();
                let rel = abs
                    .strip_prefix(&root)?
                    .to_string_lossy()
                    .replace('\\', "/");

                let bytes = std::fs::read(abs)?;
                let value = match String::from_utf8(bytes.clone()) {
                    Ok(text) => text,
                    Err(_) => format!("__BIN__{}", B64.encode(&bytes)),
                };

                map.insert(rel, value);
            }
        }
        Ok(map)
    })
    .await?
}
