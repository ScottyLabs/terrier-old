use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{
    LdBookUser, LdBox, LdCalendar, LdClipboardPen, LdFileText, LdGavel, LdHome, LdMenu,
    LdMessageSquare, LdQrCode, LdSettings, LdTrophy, LdUser, LdUsers, LdX,
};
use dioxus_free_icons::{Icon, IconShape};

use crate::{
    Route,
    auth::{
        APPLICANTS_ROLES, APPLY_ROLES, CHECKIN_ROLES, DASHBOARD_ROLES, HackathonRole,
        HackathonRoleType, JUDGE_ROLES, JUDGING_ADMIN_ROLES, PEOPLE_ROLES, PRIZE_TRACKS_ROLES,
        RESULTS_ROLES, SCHEDULE_ROLES, SETTINGS_ROLES, SUBMISSION_ROLES, TEAM_ROLES, has_access,
    },
    domain::{applications::handlers::get_application, meta::handlers::get_public_config},
    ui::foundation::components::{Header, HeaderSize},
};
// SidebarItem is defined below in this file

#[component]
pub fn ExternalSidebarItem<I: IconShape + Clone + PartialEq + 'static>(
    label: String,
    icon: I,
    href: String,
) -> Element {
    rsx! {
        a { class: "block w-full", href: "{href}", target: "_blank",
            div { class: "bg-background-neutral-primary flex gap-2.5 items-center px-3 py-2 rounded-[24px] w-full cursor-pointer",
                Icon {
                    width: 20,
                    height: 20,
                    icon,
                    class: "text-foreground-neutral-primary",
                }
                p { class: "font-semibold text-sm leading-5 text-foreground-neutral-primary whitespace-nowrap",
                    "{label}"
                }
            }
        }
    }
}

/// Shared navigation items component to avoid duplication between mobile and desktop
#[component]
fn NavItems(
    slug: String,
    has_dashboard: bool,
    has_applicants: bool,
    has_people: bool,
    has_team: bool,
    has_schedule: bool,
    has_submission: bool,
    has_checkin: bool,
    has_apply: bool,
    has_prize_tracks: bool,
    has_judge: bool,
    has_judging_admin: bool,
    has_results: bool,
    has_settings: bool,
    include_settings_in_nav: bool,
    oidc_issuer: Option<String>,
    include_account_in_nav: bool,
    on_item_click: Option<EventHandler<()>>,
) -> Element {
    let handle_click = move |_| {
        if let Some(handler) = &on_item_click {
            handler.call(());
        }
    };

    rsx! {
        if has_dashboard {
            div { onclick: handle_click,
                SidebarItem {
                    label: "Dashboard".to_string(),
                    icon: LdHome,
                    to: Route::HackathonDashboard {
                        slug: slug.clone(),
                    },
                }
            }
        }
        if has_applicants {
            div { onclick: handle_click,
                SidebarItem {
                    label: "Applicants".to_string(),
                    icon: LdFileText,
                    to: Route::HackathonApplicants {
                        slug: slug.clone(),
                    },
                }
            }
        }
        if has_people {
            div { onclick: handle_click,
                SidebarItem {
                    label: "People".to_string(),
                    icon: LdBookUser,
                    to: Route::HackathonPeople {
                        slug: slug.clone(),
                    },
                }
            }
        }
        if has_team {
            div { onclick: handle_click,
                SidebarItem {
                    label: "Team".to_string(),
                    icon: LdUsers,
                    to: Route::HackathonTeam {
                        slug: slug.clone(),
                    },
                }
            }
        }
        if has_schedule {
            div { onclick: handle_click,
                SidebarItem {
                    label: "Schedule".to_string(),
                    icon: LdCalendar,
                    to: Route::HackathonSchedule {
                        slug: slug.clone(),
                    },
                }
            }
        }

        if has_submission {
            div { onclick: handle_click,
                SidebarItem {
                    label: "Project Submission".to_string(),
                    icon: LdBox,
                    to: Route::HackathonSubmission {
                        slug: slug.clone(),
                    },
                }
            }
        }
        if has_checkin {
            div { onclick: handle_click,
                SidebarItem {
                    label: "Event Check-In".to_string(),
                    icon: LdQrCode,
                    to: Route::HackathonCheckin {
                        slug: slug.clone(),
                    },
                }
            }
        }

        if has_apply {
            div { onclick: handle_click,
                SidebarItem {
                    label: "Apply".to_string(),
                    icon: LdClipboardPen,
                    to: Route::HackathonApply {
                        slug: slug.clone(),
                    },
                }
            }
        }
        if has_prize_tracks {
            div { onclick: handle_click,
                SidebarItem {
                    label: "Prize Tracks".to_string(),
                    icon: LdTrophy,
                    to: Route::HackathonPrizeTracks {
                        slug: slug.clone(),
                    },
                }
            }
        }
        if has_judge {
            div { onclick: handle_click,
                SidebarItem {
                    label: "Judge".to_string(),
                    icon: LdGavel,
                    to: Route::HackathonJudge {
                        slug: slug.clone(),
                    },
                }
            }
        }
        if has_judging_admin {
            div { onclick: handle_click,
                SidebarItem {
                    label: "Judging Admin".to_string(),
                    icon: LdGavel,
                    to: Route::HackathonJudgingAdmin {
                        slug: slug.clone(),
                    },
                }
            }
        }
        if has_results {
            div { onclick: handle_click,
                SidebarItem {
                    label: "Results".to_string(),
                    icon: LdTrophy,
                    to: Route::HackathonResults {
                        slug: slug.clone(),
                    },
                }
            }
        }
        if let Some(oidc_issuer) = oidc_issuer {
             if include_account_in_nav {
                div { onclick: handle_click,
                    ExternalSidebarItem {
                        label: "Manage Account".to_string(),
                        icon: LdUser,
                        href: format!("{}/account", oidc_issuer),
                    }
                }
            }
        }
        if has_settings && include_settings_in_nav {
            div { onclick: handle_click,
                SidebarItem {
                    label: "Settings".to_string(),
                    icon: LdSettings,
                    to: Route::HackathonSettings {
                        slug: slug.clone(),
                    },
                }
            }
        }
    }
}

#[component]
pub fn Sidebar(
    slug: String,
    hackathon_signal: Signal<crate::domain::hackathons::types::HackathonInfo>,
    role: Option<HackathonRole>,
    application_refresh_trigger: Signal<u32>,
) -> Element {
    let is_mobile = use_context::<Signal<bool>>();

    let has = |allowed: &[HackathonRoleType]| {
        role.as_ref()
            .map(|r| has_access(r, allowed))
            .unwrap_or(false)
    };

    // Fetch application to check if submitted
    let slug_for_app = slug.clone();
    let application_resource = use_resource(move || {
        let slug = slug_for_app.clone();
        let _ = application_refresh_trigger.read();
        async move { get_application(slug).await.ok() }
    });

    let public_config = use_server_future(get_public_config)?;
    let oidc_issuer = match &*public_config.read() {
        Some(Ok(config)) => config.oidc_issuer.clone(),
        _ => None,
    };

    let mut menu_open = use_signal(|| false);

    // Check if user has submitted application (status != "draft")
    let has_submitted_application = application_resource
        .read()
        .as_ref()
        .and_then(|app| app.as_ref())
        .map(|app| app.status != "draft")
        .unwrap_or(false);

    // Pre-compute role-based visibility flags
    let has_dashboard = has(DASHBOARD_ROLES);
    let has_applicants = has(APPLICANTS_ROLES);
    let has_people = has(PEOPLE_ROLES);
    let has_team = has(TEAM_ROLES) && has_submitted_application;
    let has_schedule = has(SCHEDULE_ROLES);
    let has_submission = has(SUBMISSION_ROLES);
    let has_checkin = has(CHECKIN_ROLES);
    let has_apply = has(APPLY_ROLES);
    let has_prize_tracks = has(PRIZE_TRACKS_ROLES);
    let has_judge = has(JUDGE_ROLES);
    let has_judging_admin = has(JUDGING_ADMIN_ROLES);
    let has_results = has(RESULTS_ROLES);
    let has_settings = has(SETTINGS_ROLES);

    rsx! {
        if *is_mobile.read() {
            // Mobile: Header bar + full-screen overlay when open
            div { class: "bg-background-neutral-primary flex justify-between items-center w-full px-4 py-3",
                p { class: "font-medium text-xl leading-7 text-foreground-neutral-primary",
                    "{hackathon_signal.read().name}"
                }
                button {
                    onclick: move |_| menu_open.set(true),
                    class: "p-2 cursor-pointer",
                    Icon {
                        width: 24,
                        height: 24,
                        icon: LdMenu,
                        class: "text-foreground-neutral-primary",
                    }
                }
            }

            // Full-screen overlay menu
            if *menu_open.read() {
                div { class: "fixed inset-0 z-50 bg-background-neutral-primary flex flex-col",
                    div { class: "flex justify-end p-4",
                        button {
                            onclick: move |_| menu_open.set(false),
                            class: "p-2 cursor-pointer",
                            Icon {
                                width: 24,
                                height: 24,
                                icon: LdX,
                                class: "text-foreground-neutral-primary",
                            }
                        }
                    }
                    nav { class: "flex flex-col gap-2 px-6 py-4",
                        NavItems {
                            slug: slug.clone(),
                            has_dashboard,
                            has_applicants,
                            has_people,
                            has_team,
                            has_schedule,
                            has_submission,
                            has_checkin,
                            has_apply,
                            has_prize_tracks,
                            has_judge,
                            has_judging_admin,
                            has_results,
                            has_settings,
                            include_settings_in_nav: true,
                            oidc_issuer: oidc_issuer.clone(),
                            include_account_in_nav: true,
                            on_item_click: move |_| menu_open.set(false),
                        }
                    }
                }
            }
        } else {
            // Desktop: Original sidebar
            aside { class: "bg-background-neutral-primary flex flex-col gap-8 items-start h-[calc(100vh-3rem)] w-60 p-4 rounded-[20px] shadow-[0px_2px_16px_0px_rgba(0,0,0,0.1)]",
                div { class: "flex justify-between items-center w-full p-[16px]",
                    div { class: "flex flex-col gap-3 items-start w-full",
                        div { class: "flex gap-1.5 items-center w-full",
                            Header { size: HeaderSize::Small }
                        }
                        p { class: "font-medium text-xl leading-7 text-foreground-neutral-primary w-full",
                            "{hackathon_signal.read().name}"
                        }
                    }
                }

                nav { class: "flex flex-col gap-1 items-start w-full",
                    NavItems {
                        slug: slug.clone(),
                        has_dashboard,
                        has_applicants,
                        has_people,
                        has_team,
                        has_schedule,
                        has_submission,
                        has_checkin,
                        has_apply,
                        has_prize_tracks,
                        has_judge,
                        has_judging_admin,
                        has_results,
                        has_settings,
                        include_settings_in_nav: false,
                        oidc_issuer: oidc_issuer.clone(),
                        include_account_in_nav: false,
                        on_item_click: None,
                    }
                }

                div { class: "mt-auto w-full flex flex-col gap-1",
                    if let Some(oidc_issuer) = oidc_issuer {
                        ExternalSidebarItem {
                            label: "Manage Account".to_string(),
                            icon: LdUser,
                            href: format!("{}/account", oidc_issuer),
                        }
                    }
                    if has_settings {
                        SidebarItem {
                            label: "Settings".to_string(),
                            icon: LdSettings,
                            to: Route::HackathonSettings {
                                slug: slug.clone(),
                            },
                        }
                    }
                }
            }
        }
    }
}
#[component]
pub fn SidebarItem<I: IconShape + Clone + PartialEq + 'static>(
    label: String,
    icon: I,
    to: Route,
) -> Element {
    let current_route = use_route::<Route>();

    // Check if this item's route matches the current route (ignoring slug values)
    #[allow(clippy::match_like_matches_macro)]
    let is_active = match (&current_route, &to) {
        (Route::HackathonDashboard { .. }, Route::HackathonDashboard { .. }) => true,
        (Route::HackathonApplicants { .. }, Route::HackathonApplicants { .. }) => true,
        (Route::HackathonPeople { .. }, Route::HackathonPeople { .. }) => true,
        (Route::HackathonTeam { .. }, Route::HackathonTeam { .. }) => true,
        (Route::HackathonSchedule { .. }, Route::HackathonSchedule { .. }) => true,
        (Route::HackathonMessages { .. }, Route::HackathonMessages { .. }) => true,
        (Route::HackathonSubmission { .. }, Route::HackathonSubmission { .. }) => true,
        (Route::HackathonCheckin { .. }, Route::HackathonCheckin { .. }) => true,
        (Route::HackathonProfile { .. }, Route::HackathonProfile { .. }) => true,
        (Route::HackathonApply { .. }, Route::HackathonApply { .. }) => true,
        (Route::HackathonPrizeTracks { .. }, Route::HackathonPrizeTracks { .. }) => true,
        (Route::HackathonJudge { .. }, Route::HackathonJudge { .. }) => true,
        (Route::HackathonJudgingAdmin { .. }, Route::HackathonJudgingAdmin { .. }) => true,
        (Route::HackathonResults { .. }, Route::HackathonResults { .. }) => true,
        (Route::HackathonSettings { .. }, Route::HackathonSettings { .. }) => true,
        _ => false,
    };

    let (container_class, text_class, icon_class) = if is_active {
        (
            "bg-foreground-neutral-primary flex gap-2.5 items-center px-3 py-2 rounded-[24px] w-full cursor-pointer",
            "font-semibold text-sm leading-5 text-white whitespace-nowrap",
            "text-white",
        )
    } else {
        (
            "bg-background-neutral-primary flex gap-2.5 items-center px-3 py-2 rounded-[24px] w-full cursor-pointer",
            "font-semibold text-sm leading-5 text-foreground-neutral-primary whitespace-nowrap",
            "text-foreground-neutral-primary",
        )
    };

    rsx! {
        Link { class: "block w-full", to,
            div { class: "{container_class}",
                Icon {
                    width: 20,
                    height: 20,
                    icon,
                    class: "{icon_class}",
                }
                p { class: "{text_class}", "{label}" }
            }
        }
    }
}
