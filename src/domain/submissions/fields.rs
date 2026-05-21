//! Typed accessors for submission form data stored as untyped JSON.
//!
//! Submission forms define field names in their schema (e.g. `project_name`),
//! but some legacy/mock data uses camelCase keys. `SubmissionFields` accepts
//! both conventions via serde aliases so callers never need to know which
//! format is stored.

use serde::Deserialize;

/// Canonical keys written by submission form schemas (source of truth).
pub mod form_keys {
    pub const PROJECT_NAME: &str = "project_name";
    pub const PROJECT_DESCRIPTION: &str = "project_description";
    pub const REPO_URL: &str = "repo_url";
    pub const PRESENTATION_URL: &str = "presentation_url";
    pub const VIDEO_URL: &str = "video_url";
    pub const DEMO_URL: &str = "demo_url";
    pub const PROJECT_ZIP_URL: &str = "project_zip_url";
}

/// Well-known fields extracted from `submission.submission_data`.
///
/// Deserialize this from the raw JSON blob rather than calling `.get()` with
/// hard-coded key strings — serde aliases handle snake_case vs camelCase.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct SubmissionFields {
    #[serde(alias = "projectName", alias = "title")]
    pub project_name: Option<String>,

    #[serde(alias = "project_description")]
    pub description: Option<String>,

    #[serde(alias = "repoUrl")]
    pub repo_url: Option<String>,

    #[serde(alias = "presentationUrl")]
    pub presentation_url: Option<String>,

    #[serde(alias = "videoUrl")]
    pub video_url: Option<String>,

    #[serde(alias = "demoUrl")]
    pub demo_url: Option<String>,

    #[serde(alias = "projectZipUrl")]
    pub project_zip_url: Option<String>,
}

impl SubmissionFields {
    pub fn from_json(data: &serde_json::Value) -> Self {
        serde_json::from_value(data.clone()).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_snake_case_form_submission() {
        let data = json!({
            "project_name": "Terrier",
            "project_description": "A hackathon platform",
            "repo_url": "https://github.com/example/terrier",
            "presentation_url": "https://slides.example.com",
            "video_url": "https://youtube.com/watch?v=abc",
            "demo_url": "https://demo.example.com",
            "project_zip_url": "https://drive.example.com/zip"
        });

        let fields = SubmissionFields::from_json(&data);

        assert_eq!(fields.project_name.as_deref(), Some("Terrier"));
        assert_eq!(fields.description.as_deref(), Some("A hackathon platform"));
        assert_eq!(
            fields.repo_url.as_deref(),
            Some("https://github.com/example/terrier")
        );
        assert_eq!(
            fields.presentation_url.as_deref(),
            Some("https://slides.example.com")
        );
        assert_eq!(
            fields.video_url.as_deref(),
            Some("https://youtube.com/watch?v=abc")
        );
        assert_eq!(fields.demo_url.as_deref(), Some("https://demo.example.com"));
        assert_eq!(
            fields.project_zip_url.as_deref(),
            Some("https://drive.example.com/zip")
        );
    }

    #[test]
    fn parses_camel_case_legacy_submission() {
        let data = json!({
            "projectName": "Fake Project",
            "description": "Legacy mock description",
            "repoUrl": "https://github.com/example/legacy"
        });

        let fields = SubmissionFields::from_json(&data);

        assert_eq!(fields.project_name.as_deref(), Some("Fake Project"));
        assert_eq!(
            fields.description.as_deref(),
            Some("Legacy mock description")
        );
        assert_eq!(
            fields.repo_url.as_deref(),
            Some("https://github.com/example/legacy")
        );
    }
}
