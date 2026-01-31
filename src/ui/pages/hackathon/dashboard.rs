use crate::ui::features::dashboard::{QRModal, QRTile, TablePromptModal};

use crate::{
    auth::{DASHBOARD_ROLES, HackathonRole, hooks::use_require_access_or_redirect},
    domain::hackathons::types::HackathonInfo,
};
use dioxus::prelude::*;

#[component]
pub fn HackathonDashboard(slug: String) -> Element {
    if let Some(no_access) = use_require_access_or_redirect(DASHBOARD_ROLES) {
        return no_access;
    }

    let user_role = use_context::<Option<HackathonRole>>();
    let is_participant = user_role
        .as_ref()
        .map(|r| r.role == "participant")
        .unwrap_or(false);
    let hackathon = use_context::<Signal<HackathonInfo>>();

    rsx! {
        h1 { class: "text-[30px] font-semibold leading-[38px] text-foreground-neutral-primary pt-11 pb-7",
            "Dashboard"
        }
        // Tile grid
        div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6", QRTile {} }

        // Prompt for table assignment if participant, submissions are closed, and hackathon is active
        if is_participant && hackathon.read().submissions_closed && hackathon.read().is_active {
            TablePromptModal { slug: slug.clone() }
        }
    }
}
