use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PublicConfig {
    pub posthog_key: Option<String>,
    pub posthog_host: Option<String>,
    pub oidc_issuer: Option<String>,
}

#[server]
pub async fn get_public_config() -> Result<PublicConfig, ServerFnError> {
    use dioxus::fullstack::{FullstackContext, extract::State as DxState};

    let DxState(state) = FullstackContext::extract::<DxState<crate::AppState>, _>()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to extract state: {}", e)))?;

    Ok(PublicConfig {
        posthog_key: state.config.posthog_key.clone(),
        posthog_host: state.config.posthog_host.clone(),
        oidc_issuer: Some(state.config.oidc_issuer.clone()),
    })
}
