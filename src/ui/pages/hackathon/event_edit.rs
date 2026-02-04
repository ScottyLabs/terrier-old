use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use dioxus::prelude::*;
use dioxus_free_icons::{
    Icon,
    icons::ld_icons::{LdArrowLeft, LdClock, LdMapPin, LdSearch, LdTarget},
};

use crate::{
    Route,
    auth::{
        HackathonRole, HackathonRoleType, SCHEDULE_ROLES, hooks::use_require_access_or_redirect,
    },
    domain::{
        applications::handlers::{
            UpdateEventRequest, delete_event, get_user_schedule, update_event,
        },
        hackathons::types::{HackathonInfo, ScheduleEvent},
        people::handlers::{HackathonPerson, get_hackathon_people},
    },
    ui::foundation::utils::get_avatar_color,
};

/// Simple organizer info for the edit page
#[derive(Debug, Clone, PartialEq)]
struct OrganizerInfo {
    user_id: i32,
    name: String,
    color: String,
}

/// Mobile event edit page - full screen form for editing an event
#[component]
pub fn HackathonScheduleEdit(slug: String, event_id: i32) -> Element {
    if let Some(no_access) = use_require_access_or_redirect(SCHEDULE_ROLES) {
        return no_access;
    }

    let nav = use_navigator();
    let slug_for_nav = slug.clone();
    let slug_for_resource = slug.clone();
    let slug_for_save = slug.clone();
    let slug_for_delete = slug.clone();
    let slug_for_people = slug.clone();

    // Get hackathon info for date bounds
    let hackathon = use_context::<Signal<HackathonInfo>>();

    // Get user's role from context to check admin/organizer status
    let user_role = use_context::<Option<HackathonRole>>();
    let is_admin_or_organizer = user_role
        .as_ref()
        .and_then(|r: &HackathonRole| r.role_type())
        .map(|rt| rt == HackathonRoleType::Admin || rt == HackathonRoleType::Organizer)
        .unwrap_or(false);

    // Fetch the event
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

    // Form state - start with empty, will be populated when event loads
    let mut name = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut location = use_signal(String::new);
    let mut start_date = use_signal(String::new);
    let mut end_date = use_signal(String::new);
    let mut start_time = use_signal(String::new);
    let mut end_time = use_signal(String::new);
    let mut event_type = use_signal(|| "default".to_string());
    let mut points = use_signal(String::new);

    let mut selected_organizers = use_signal(Vec::<OrganizerInfo>::new);
    let mut has_initialized_form = use_signal(|| false);
    let mut organizer_search = use_signal(String::new);
    let mut show_organizer_dropdown = use_signal(|| false);

    // Populate form when event data loads
    let _ = use_memo(move || {
        if let Some(Some(event)) = event_resource.read().as_ref() {
            if !has_initialized_form() {
                name.set(event.name.clone());
                description.set(event.description.clone().unwrap_or_default());
                location.set(event.location.clone().unwrap_or_default());
                start_date.set(event.start_time.date().format("%Y-%m-%d").to_string());
                end_date.set(event.end_time.date().format("%Y-%m-%d").to_string());
                start_time.set(event.start_time.time().format("%H:%M").to_string());
                end_time.set(event.end_time.time().format("%H:%M").to_string());
                event_type.set(event.event_type.clone());
                points.set(event.points.map(|p| p.to_string()).unwrap_or_default());
                has_initialized_form.set(true);
            }
        }
    });

    // State
    let mut error = use_signal(|| None::<String>);
    let mut is_saving = use_signal(|| false);
    let mut show_delete_confirm = use_signal(|| false);
    let mut is_deleting = use_signal(|| false);

    // Fetch people for organizer selection
    let people_resource = use_resource(move || {
        let slug = slug_for_people.clone();
        async move {
            let result: Result<Vec<HackathonPerson>, _> = get_hackathon_people(slug).await;
            result.ok()
        }
    });

    // Initialize organizers from event when both event and people are loaded
    let mut has_initialized_organizers = use_signal(|| false);
    let _ = use_memo(move || {
        if let Some(Some(event)) = event_resource.read().as_ref() {
            if let Some(people) = people_resource.read().as_ref().and_then(|p| p.as_ref()) {
                if !has_initialized_organizers() {
                    let orgs: Vec<OrganizerInfo> = event
                        .organizer_ids
                        .iter()
                        .filter_map(|id| {
                            people.iter().find(|p| p.user_id == *id).map(|p| {
                                let name = p.name.clone().unwrap_or_else(|| p.email.clone());
                                OrganizerInfo {
                                    user_id: p.user_id,
                                    name: name.clone(),
                                    color: get_avatar_color(&name).to_string(),
                                }
                            })
                        })
                        .collect();
                    selected_organizers.set(orgs);
                    has_initialized_organizers.set(true);
                }
            }
        }
    });

    // Filter organizers for dropdown
    let filtered_organizers = {
        let search = organizer_search().to_lowercase();
        let selected_ids: Vec<i32> = selected_organizers().iter().map(|o| o.user_id).collect();

        people_resource
            .read()
            .as_ref()
            .and_then(|p| p.as_ref())
            .map(|people| {
                people
                    .iter()
                    .filter(|p| {
                        (p.role == "organizer" || p.role == "admin")
                            && !selected_ids.contains(&p.user_id)
                            && (search.is_empty()
                                || p.name
                                    .as_ref()
                                    .map(|n| n.to_lowercase().contains(&search))
                                    .unwrap_or(false)
                                || p.email.to_lowercase().contains(&search))
                    })
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };

    // Redirect non-admins
    if !is_admin_or_organizer {
        nav.push(Route::HackathonSchedule {
            slug: slug_for_nav.clone(),
        });
        return rsx! {};
    }

    // Handle save
    let handle_save = {
        move |_| {
            let slug = slug_for_save.clone();
            let name_val = name();
            let location_val = location();
            let description_val = description();
            let start_date_val = start_date();
            let start_time_val = start_time();
            let end_time_val = end_time();
            let end_date_val = end_date();
            let event_type_val = event_type();
            let points_val = points();
            let organizer_ids_val: Vec<i32> =
                selected_organizers().iter().map(|o| o.user_id).collect();

            spawn(async move {
                is_saving.set(true);
                error.set(None);

                // Validate
                if name_val.trim().is_empty() {
                    error.set(Some("Event name is required".to_string()));
                    is_saving.set(false);
                    return;
                }

                // Parse date/time
                let parsed_start_date = match NaiveDate::parse_from_str(&start_date_val, "%Y-%m-%d")
                {
                    Ok(d) => d,
                    Err(_) => {
                        error.set(Some("Invalid date format".to_string()));
                        is_saving.set(false);
                        return;
                    }
                };

                let parsed_end_date = match NaiveDate::parse_from_str(&end_date_val, "%Y-%m-%d") {
                    Ok(d) => d,
                    Err(_) => {
                        error.set(Some("Invalid date format".to_string()));
                        is_saving.set(false);
                        return;
                    }
                };

                let parsed_start_time = match NaiveTime::parse_from_str(&start_time_val, "%H:%M") {
                    Ok(t) => t,
                    Err(_) => {
                        error.set(Some("Invalid start time format".to_string()));
                        is_saving.set(false);
                        return;
                    }
                };

                let parsed_end_time = match NaiveTime::parse_from_str(&end_time_val, "%H:%M") {
                    Ok(t) => t,
                    Err(_) => {
                        error.set(Some("Invalid end time format".to_string()));
                        is_saving.set(false);
                        return;
                    }
                };

                let start_datetime = NaiveDateTime::new(parsed_start_date, parsed_start_time);
                let end_datetime = NaiveDateTime::new(parsed_end_date, parsed_end_time);

                // reject if end time is before start time
                if end_datetime < start_datetime {
                    error.set(Some("End time must be after start time".to_string()));
                    is_saving.set(false);
                    return;
                }

                let parsed_points: Option<i32> = if points_val.is_empty() {
                    None
                } else {
                    match points_val.parse() {
                        Ok(p) => Some(p),
                        Err(_) => {
                            error.set(Some("Points must be a number".to_string()));
                            is_saving.set(false);
                            return;
                        }
                    }
                };

                let request = UpdateEventRequest {
                    id: event_id,
                    name: name_val,
                    description: if description_val.is_empty() {
                        None
                    } else {
                        Some(description_val)
                    },
                    location: if location_val.is_empty() {
                        None
                    } else {
                        Some(location_val)
                    },
                    start_time: start_datetime,
                    end_time: end_datetime,
                    visible_to_role: None,
                    event_type: event_type_val,
                    is_visible: true,
                    organizer_ids: organizer_ids_val,
                    points: parsed_points,
                    checkin_type: "self".to_string(),
                };

                match update_event(slug.clone(), request).await {
                    Ok(_) => {
                        nav.push(Route::HackathonScheduleEvent { slug, event_id });
                    }
                    Err(e) => {
                        error.set(Some(e.to_string()));
                    }
                }

                is_saving.set(false);
            });
        }
    };

    // Handle delete
    let handle_delete = {
        move |_| {
            let slug = slug_for_delete.clone();
            spawn(async move {
                is_deleting.set(true);
                match delete_event(slug.clone(), event_id).await {
                    Ok(_) => {
                        nav.push(Route::HackathonSchedule { slug });
                    }
                    Err(e) => {
                        error.set(Some(e.to_string()));
                        is_deleting.set(false);
                    }
                }
            });
        }
    };

    rsx! {
        div { class: "bg-background-neutral-secondary-enabled flex flex-col",
            // Header with back button
            div { class: "flex-shrink-0 p-4 flex items-center justify-between",
                button {
                    class: "p-2 -ml-2 hover:bg-background-neutral-secondary-hover rounded-full transition-colors",
                    onclick: move |_| {
                        nav.push(Route::HackathonScheduleEvent {
                            slug: slug_for_nav.clone(),
                            event_id,
                        });
                    },
                    Icon { width: 24, height: 24, icon: LdArrowLeft }
                }
            }

            // Form content - scrollable
            div { class: "flex flex-col flex-1 px-4 pb-4 overflow-y-auto",
                // Title and category row
                div { class: "height-[calc(100%-100px)] overflow-y-auto",
                    div { class: "flex items-start justify-between gap-4 mb-4",
                        input {
                            r#type: "text",
                            class: "flex-1 text-2xl font-semibold text-foreground-neutral-primary bg-transparent border-none outline-none placeholder:text-foreground-neutral-tertiary",
                            placeholder: "Name of event",
                            value: "{name}",
                            oninput: move |e| name.set(e.value()),
                        }
                        select {
                            class: "text-sm border border-stroke-neutral-1 rounded-full px-3 py-1 bg-transparent",
                            value: "{event_type}",
                            onchange: move |e| event_type.set(e.value()),
                            option { value: "default", "Category" }
                            option { value: "hacking", "Hacking" }
                            option { value: "speaker", "Speaker" }
                            option { value: "sponsor", "Sponsor" }
                            option { value: "food", "Food" }
                        }
                    }

                    // Location
                    div { class: "flex items-center gap-2 mb-2",
                        Icon {
                            width: 16,
                            height: 16,
                            icon: LdMapPin,
                            class: "text-foreground-neutral-secondary",
                        }
                        input {
                            r#type: "text",
                            class: "flex-1 text-sm bg-transparent border-none outline-none placeholder:text-foreground-neutral-tertiary",
                            placeholder: "Location",
                            value: "{location}",
                            oninput: move |e| location.set(e.value()),
                        }
                    }

                    // Date and time
                    div { class: "flex items-center gap-2 mb-2",
                        Icon {
                            width: 16,
                            height: 16,
                            icon: LdClock,
                            class: "text-foreground-neutral-secondary",
                        }
                        input {
                            r#type: "date",
                            class: "text-sm bg-transparent border-none outline-none",
                            value: "{start_date}",
                            oninput: move |e| start_date.set(e.value()),
                        }
                        span { class: "text-foreground-neutral-tertiary", "·" }
                        input {
                            r#type: "time",
                            class: "text-sm bg-transparent border-none outline-none",
                            value: "{start_time}",
                            oninput: move |e| start_time.set(e.value()),
                        }
                        input {
                            r#type: "date",
                            class: "text-sm bg-transparent border-none outline-none",
                            value: "{end_date}",
                            oninput: move |e| end_date.set(e.value()),
                        }
                        span { class: "text-foreground-neutral-tertiary", "–" }
                        input {
                            r#type: "time",
                            class: "text-sm bg-transparent border-none outline-none",
                            value: "{end_time}",
                            oninput: move |e| end_time.set(e.value()),
                        }
                    }

                    // Points
                    div { class: "flex items-center gap-2 mb-6",
                        Icon {
                            width: 16,
                            height: 16,
                            icon: LdTarget,
                            class: "text-foreground-neutral-secondary",
                        }
                        input {
                            r#type: "number",
                            class: "w-20 text-sm bg-transparent border-none outline-none placeholder:text-foreground-neutral-tertiary",
                            placeholder: "Points",
                            value: "{points}",
                            oninput: move |e| points.set(e.value()),
                        }
                        span { class: "text-sm text-foreground-neutral-secondary", "Points" }
                    }

                    // Event Description
                    div { class: "mb-6",
                        h3 { class: "text-sm font-medium text-foreground-neutral-primary mb-2",
                            "Event Description"
                        }
                        div { class: "bg-background-neutral-primary rounded-xl p-4",
                            textarea {
                                class: "w-full h-24 text-sm bg-transparent resize-none placeholder:text-foreground-neutral-tertiary border-none outline-none",
                                placeholder: "Describe this event...",
                                value: "{description}",
                                oninput: move |e| description.set(e.value()),
                            }
                        }
                    }

                    // Organizers
                    div { class: "mb-6 bg-background-neutral-primary rounded-xl p-4",
                        h3 { class: "text-sm font-medium text-foreground-neutral-primary mb-3",
                            "Organizers"
                        }

                        // Search input
                        div { class: "relative mb-3",
                            div { class: "flex items-center gap-2 px-3 py-2 border border-stroke-neutral-1 rounded-lg",
                                Icon {
                                    width: 16,
                                    height: 16,
                                    icon: LdSearch,
                                    class: "text-foreground-neutral-tertiary",
                                }
                                input {
                                    r#type: "text",
                                    class: "flex-1 text-sm bg-transparent outline-none placeholder:text-foreground-neutral-tertiary",
                                    placeholder: "Add Organizer",
                                    value: "{organizer_search}",
                                    onfocus: move |_| show_organizer_dropdown.set(true),
                                    oninput: move |e| {
                                        organizer_search.set(e.value());
                                        show_organizer_dropdown.set(true);
                                    },
                                }
                            }

                            // Dropdown
                            if show_organizer_dropdown() && !filtered_organizers.is_empty() {
                                div { class: "absolute left-0 right-0 top-full mt-1 bg-background-neutral-primary border border-stroke-neutral-1 rounded-lg shadow-lg max-h-48 overflow-y-auto z-10",
                                    for person in filtered_organizers.iter() {
                                        button {
                                            key: "{person.user_id}",
                                            class: "w-full px-4 py-2 text-left hover:bg-background-neutral-secondary-enabled flex items-center gap-2",
                                            onclick: {
                                                let p = person.clone();
                                                let name = p.name.clone().unwrap_or_else(|| p.email.clone());
                                                move |_| {
                                                    let mut orgs = selected_organizers();
                                                    orgs.push(OrganizerInfo {
                                                        user_id: p.user_id,
                                                        name: name.clone(),
                                                        color: get_avatar_color(&name).to_string(),
                                                    });
                                                    selected_organizers.set(orgs);
                                                    organizer_search.set(String::new());
                                                    show_organizer_dropdown.set(false);
                                                }
                                            },
                                            div { class: "w-6 h-6 rounded-full bg-background-neutral-tertiary" }
                                            span { class: "text-sm",
                                                "{person.name.clone().unwrap_or_else(|| person.email.clone())}"
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Selected organizers
                        div { class: "space-y-2",
                            for org in selected_organizers().iter() {
                                {
                                    let org_id = org.user_id;
                                    rsx! {
                                        div { key: "{org.user_id}", class: "flex items-center justify-between py-2",
                                            div { class: "flex items-center gap-3",
                                                div { class: "w-8 h-8 rounded-full {org.color}" }
                                                span { class: "text-sm font-medium text-foreground-neutral-primary", "{org.name}" }
                                            }
                                            button {
                                                class: "text-sm text-foreground-neutral-secondary border border-stroke-neutral-1 rounded-full px-3 py-1 hover:bg-red-50",
                                                onclick: move |_| {
                                                    let orgs: Vec<_> = selected_organizers()
                                                        .into_iter()
                                                        .filter(|o| o.user_id != org_id)
                                                        .collect();
                                                    selected_organizers.set(orgs);
                                                },
                                                "Remove"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Error display
                    if let Some(err) = error() {
                        div { class: "mb-4 p-3 bg-red-50 text-red-600 rounded-lg text-sm",
                            "{err}"
                        }
                    }
                }

                // Bottom buttons
                div { class: "p-4 mt-auto flex-none",
                    div { class: "flex gap-3",
                        if show_delete_confirm() {
                            button {
                                class: "flex-1 py-3 bg-red-600 text-white font-medium rounded-full",
                                disabled: is_deleting(),
                                onclick: handle_delete,
                                if is_deleting() {
                                    "Deleting..."
                                } else {
                                    "Confirm Delete"
                                }
                            }
                            button {
                                class: "flex-1 py-3 bg-background-neutral-tertiary text-foreground-neutral-primary font-medium rounded-full",
                                onclick: move |_| show_delete_confirm.set(false),
                                "Cancel"
                            }
                        } else {
                            button {
                                class: "flex-1 py-3 bg-red-100 text-red-600 font-medium rounded-full",
                                onclick: move |_| show_delete_confirm.set(true),
                                "Remove"
                            }
                            button {
                                class: "flex-1 py-3 bg-foreground-neutral-primary text-white font-medium rounded-full",
                                disabled: is_saving(),
                                onclick: handle_save,
                                if is_saving() {
                                    "Saving..."
                                } else {
                                    "Save"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
