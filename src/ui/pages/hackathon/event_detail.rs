use dioxus::prelude::*;
use dioxus_free_icons::{
    Icon,
    icons::ld_icons::{LdArrowLeft, LdClock, LdMapPin, LdPencil},
};

use crate::{
    Route,
    auth::{SCHEDULE_ROLES, hooks::use_require_access_or_redirect},
    domain::{applications::handlers::get_user_schedule, hackathons::types::ScheduleEvent},
};

/// Mobile event detail page - full screen view of a single event
#[component]
pub fn HackathonScheduleEvent(slug: String, event_id: i32) -> Element {
    if let Some(no_access) = use_require_access_or_redirect(SCHEDULE_ROLES) {
        return no_access;
    }

    let nav = use_navigator();
    let slug_for_nav = slug.clone();
    let slug_for_edit = slug.clone();
    let slug_for_resource = slug.clone();

    // Fetch schedule to find the specific event
    let event_resource = use_resource(move || {
        let slug = slug_for_resource.clone();
        let id = event_id;
        async move {
            let result: Result<Vec<ScheduleEvent>, _> = get_user_schedule(slug).await;
            result
                .ok()
                .and_then(|events| events.into_iter().find(|e| e.id == id))
        }
    });

    let event = event_resource.read();

    rsx! {
        div { class: "min-h-screen bg-background-neutral-secondary",
            // Back button header
            div { class: "p-4",
                button {
                    class: "p-2 -ml-2 hover:bg-background-neutral-secondary-hover rounded-full transition-colors",
                    onclick: move |_| {
                        nav.push(Route::HackathonSchedule {
                            slug: slug_for_nav.clone(),
                        });
                    },
                    Icon { width: 24, height: 24, icon: LdArrowLeft }
                }
            }

            // Event content
            div { class: "px-4 pb-8",
                match event.as_ref() {
                    Some(Some(event)) => rsx! {
                        // Title row with edit button on right
                        div { class: "flex items-start justify-between mb-6",
                            h1 { class: "text-2xl font-bold text-foreground-neutral-primary flex-1", "{event.name}" }
                            button {
                                class: "p-2 hover:bg-background-neutral-secondary-hover rounded-full transition-colors ml-2",
                                onclick: move |_| {
                                    nav.push(Route::HackathonScheduleEdit {
                                        slug: slug_for_edit.clone(),
                                        event_id,
                                    });
                                },
                                Icon {





                                    width: 20,
                                    height: 20,
                                    icon: LdPencil,
                                    class: "text-foreground-neutral-secondary",
                                }
                            }
                        }
                        if let Some(loc) = &event.location {
                            div { class: "flex items-center gap-3 mb-3",
                                Icon {
                                    width: 18,
                                    height: 18,
                                    icon: LdMapPin,
                                    class: "text-foreground-neutral-secondary",
                                }
                                span { class: "text-foreground-neutral-primary", "{loc}" }
                            }
                        }















                        div { class: "flex items-center gap-3 mb-6",
                            Icon {
                                width: 18,
                                height: 18,
                                icon: LdClock,
                                class: "text-foreground-neutral-secondary",
                            }
                            span { class: "text-foreground-neutral-primary",
                                "{format_event_datetime(&event.start_time, &event.end_time)}"
                            }
                        }

                        if let Some(desc) = &event.description {
                            p { class: "text-foreground-neutral-secondary leading-relaxed", "{desc}" }
                        }
                    },
                    Some(None) => rsx! {
                        div { class: "text-center py-12",
                            p { class: "text-foreground-neutral-secondary", "Event not found" }
                        }
                    },
                    None => rsx! {
                        div { class: "text-center py-12",
                            p { class: "text-foreground-neutral-secondary", "Loading..." }
                        }
                    },
                }
            }
        }
    }
}

/// Format event datetime in a readable way
fn format_event_datetime(start: &chrono::NaiveDateTime, end: &chrono::NaiveDateTime) -> String {
    let start_date = start.format("%A, %B %d").to_string();
    let start_time = start.format("%-I:%M").to_string();
    let end_time = end.format("%-I:%M%P").to_string();

    if start.date() == end.date() {
        format!("{} · {} – {}", start_date, start_time, end_time)
    } else {
        let end_date = end.format("%A, %B %d").to_string();
        format!("{} {} – {} {}", start_date, start_time, end_date, end_time)
    }
}
