//! Code Fix Agent for Genesis — Allowing agents to propose and apply source code modifications.
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeFixRequest {
    pub filepath: String,
    pub search_block: String,
    pub replace_block: String,
    pub reasoning: String,
}

pub struct CodeFixAgent;

impl CodeFixAgent {
    pub fn apply_fix(request: &CodeFixRequest) -> Result<(), String> {
        let path = Path::new(&request.filepath);
        if !path.exists() {
            return Err(format!("File {} not found", request.filepath));
        }

        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        if !content.contains(&request.search_block) {
            return Err("Search block not found in file".to_string());
        }

        let new_content = content.replace(&request.search_block, &request.replace_block);
        fs::write(path, new_content).map_err(|e| e.to_string())?;

        tracing::info!("Autonomous Code Fix applied to {}: {}", request.filepath, request.reasoning);
        Ok(())
    }
}
