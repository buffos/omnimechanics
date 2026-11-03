//! Lossless Project Workspace Persistence (.bem format).

use serde::{Deserialize, Serialize};
use std::fs;
use crate::actions::types::{Combination, SeismicSettings};
use crate::frame::types::{CanonicalError, StructuralModel};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectDocument {
    pub version: String,
    pub structural: StructuralModel,
    pub combinations: Vec<Combination>,
    pub seismic_settings: Option<SeismicSettings>,
    pub viewport: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveWorkspaceRequest {
    pub path: Option<String>,
    pub document: ProjectDocument,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadWorkspaceRequest {
    pub path_or_json: String,
}

pub fn save_workspace(req: &SaveWorkspaceRequest) -> Result<String, CanonicalError> {
    let json = serde_json::to_string_pretty(&req.document).map_err(|e| {
        CanonicalError::new(
            "PERSISTENCE_SERIALIZATION_FAILED",
            format!("Αποτυχία σειριοποίησης του αρχείου έργου: {}", e),
            format!("Failed to serialize project document: {}", e),
        )
    })?;

    if let Some(ref path) = req.path {
        fs::write(path, &json).map_err(|e| {
            CanonicalError::new(
                "PERSISTENCE_IO_FAILED",
                format!("Αποτυχία εγγραφής αρχείου στο '{}': {}", path, e),
                format!("Failed to write file to '{}': {}", path, e),
            )
        })?;
    }

    Ok(json)
}

pub fn load_workspace(req: &LoadWorkspaceRequest) -> Result<ProjectDocument, CanonicalError> {
    let content = if req.path_or_json.trim_start().starts_with('{') {
        req.path_or_json.clone()
    } else {
        fs::read_to_string(&req.path_or_json).map_err(|e| {
            CanonicalError::new(
                "PERSISTENCE_IO_FAILED",
                format!("Αποτυχία ανάγνωσης αρχείου από '{}': {}", req.path_or_json, e),
                format!("Failed to read file from '{}': {}", req.path_or_json, e),
            )
        })?
    };

    let doc: ProjectDocument = serde_json::from_str(&content).map_err(|e| {
        CanonicalError::new(
            "PERSISTENCE_DESERIALIZATION_FAILED",
            format!("Μη έγκυρη μορφή αρχείου έργου .bem: {}", e),
            format!("Invalid .bem project file format: {}", e),
        )
    })?;

    // Validate structural model in document
    crate::frame::validation::validate_model(&doc.structural)?;

    Ok(doc)
}
