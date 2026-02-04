use chrono::NaiveDateTime;
use dioxus::prelude::*;
use dioxus_free_icons::{
    Icon,
    icons::ld_icons::{LdClock, LdMapPin, LdStar, LdTarget, LdX},
};

use crate::{
    domain::{
        hackathons::types::ScheduleEvent,
        people::handlers::{HackathonPerson, get_hackathon_people},
    },
    ui::foundation::utils::get_avatar_color,
};

/// Read-only modal for viewing event details
#[component]
pub fn EventDetailModal(
    slug: String,
    event: ScheduleEvent,
    is_admin: bool,
    on_close: EventHandler<()>,
    on_edit: EventHandler<()>,
) -> Element {
    // Fetch people to get organizer names
    let people_resource = use_resource(move || {
        let slug = slug.clone();
        async move {
            let result: Result<Vec<HackathonPerson>, _> = get_hackathon_people(slug).await;
            result.ok()
        }
    });

    // Get organizer info from people (id, name, color)
    let organizers: Vec<(i32, String, String)> = {
        let people = people_resource.read();
        event
            .organizer_ids
            .iter()
            .filter_map(|org_id| {
                if let Some(Some(people_list)) = people.as_ref() {
                    people_list.iter().find(|p| p.user_id == *org_id).map(|p| {
                        let name = p.name.clone().unwrap_or_else(|| p.email.clone());
                        (*org_id, name.clone(), get_avatar_color(&name).to_string())
                    })
                } else {
                    Some((
                        *org_id,
                        "Loading...".to_string(),
                        "bg-background-neutral-tertiary".to_string(),
                    ))
                }
            })
            .collect()
    };

    // Format the date and time
    let formatted_date = format_event_datetime(&event.start_time, &event.end_time);

    // Get event type display name
    let event_type_display = match event.event_type.as_str() {
        "hacking" => "Hacking",
        "speaker" => "Speaker",
        "sponsor" => "Sponsor",
        "food" => "Food",
        _ => "Category",
    };

    rsx! {
        // Backdrop
        div {
            class: "fixed inset-0 bg-black/50 z-50 flex items-center justify-center p-4",
            onclick: move |_| on_close.call(()),

            // Modal
            div {
                class: "bg-[var(--color-background-neutral-secondary)] rounded-2xl  shadow-xl max-w-lg w-full max-h-[90vh] overflow-y-auto",
                onclick: move |e| e.stop_propagation(),

                // Header with close button
                div { class: "flex justify-end p-4 pb-0",
                    button {
                        class: "p-2 hover:bg-background-neutral-secondary-hover rounded-full transition-colors",
                        onclick: move |_| on_close.call(()),
                        Icon { width: 20, height: 20, icon: LdX }
                    }
                }

                // Content
                div { class: "px-6 pb-6",
                    // Title and category badge
                    div { class: "flex items-start justify-between gap-4 mb-4",
                        h2 { class: "text-2xl font-semibold text-foreground-neutral-primary",
                            "{event.name}"
                        }
                        span { class: "px-3 py-1 border border-stroke-neutral-1 text-foreground-neutral-primary text-sm rounded-full whitespace-nowrap",
                            "{event_type_display}"
                        }
                    }

                    // Location
                    if let Some(loc) = &event.location {
                        div { class: "flex items-center gap-2 text-foreground-neutral-secondary mb-2",
                            Icon { width: 16, height: 16, icon: LdMapPin }
                            span { class: "text-sm", "{loc}" }
                        }
                    }

                    // Date/Time
                    div { class: "flex items-center gap-2 text-foreground-neutral-secondary mb-2",
                        Icon { width: 16, height: 16, icon: LdClock }
                        span { class: "text-sm", "{formatted_date}" }
                    }

                    // Points (only shown if set) with target icon
                    if let Some(pts) = event.points {
                        div { class: "flex items-center gap-2 text-foreground-neutral-secondary mb-4",
                            Icon { width: 16, height: 16, icon: LdTarget }
                            span { class: "text-sm", "{pts} Points" }
                        }
                    }

                    // Required for Prizes
                    if !event.required_for_prizes.is_empty() {
                         div { class: "mb-6 bg-yellow-50 p-4 rounded-lg border border-yellow-200",
                            div { class: "flex items-center gap-2 mb-2",
                                Icon { width: 16, height: 16, icon: LdStar }
                                h3 { class: "text-sm font-semibold text-yellow-800",
                                    "Required for Prizes"
                                }
                            }
                            p { class: "text-sm text-yellow-700 mb-2",
                                "Attendance at this event is required to be eligible for the following prizes:"
                            }
                            div { class: "flex flex-wrap gap-2",
                                for prize_name in event.required_for_prizes.iter() {
                                    span { class: "px-2 py-1 bg-background-neutral-primary text-yellow-800 text-xs font-medium rounded border border-yellow-200",
                                        "{prize_name}"
                                    }
                                }
                            }
                        }
                    }

                    // Description
                    if let Some(desc) = &event.description {
                        div { class: "mb-6",
                            h3 { class: "text-sm font-medium text-foreground-neutral-primary mb-2",
                                "Description"
                            }
                            p { class: "text-sm text-foreground-neutral-secondary leading-relaxed",
                                "{desc}"
                            }
                        }
                    }

                    // Organizers
                    if !organizers.is_empty() {
                        div { class: "mb-6",
                            h3 { class: "text-sm font-medium text-foreground-neutral-primary mb-3",
                                "Organizers"
                            }
                            div { class: "space-y-3",
                                for (org_id , name , color) in organizers.iter() {
                                    div {
                                        key: "{org_id}",
                                        class: "flex items-center gap-3",
                                        div { class: "w-8 h-8 rounded-full {color}" }
                                        span { class: "text-sm font-medium text-foreground-neutral-primary",
                                            "{name}"
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Edit button (admin only)
                    if is_admin {
                        div { class: "flex justify-end",
                            button {
                                class: "px-8 py-2.5 bg-foreground-neutral-primary text-white font-medium text-sm rounded-full hover:opacity-90 transition-opacity",
                                onclick: move |_| on_edit.call(()),
                                "Edit"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Format event datetime in a readable way
fn format_event_datetime(start: &NaiveDateTime, end: &NaiveDateTime) -> String {
    let start_date = start.format("%A, %B %d").to_string();
    let start_time = start.format("%-I:%M%P").to_string();
    let end_time = end.format("%-I:%M%P").to_string();

    if start.date() == end.date() {
        // Same day
        format!("{} · {} – {}", start_date, start_time, end_time)
    } else {
        // Multi-day event
        let end_date = end.format("%A, %B %d").to_string();
        format!("{} {} – {} {}", start_date, start_time, end_date, end_time)
    }
}
