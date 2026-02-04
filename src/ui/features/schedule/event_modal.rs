use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use dioxus::prelude::*;
use dioxus_free_icons::{
    Icon,
    icons::ld_icons::{LdClock, LdMapPin, LdSearch, LdTarget, LdX},
};

use crate::{
    domain::{
        applications::handlers::{
            CreateEventRequest, UpdateEventRequest, create_event, delete_event, update_event,
        },
        hackathons::types::ScheduleEvent,
        people::handlers::{HackathonPerson, get_hackathon_people},
    },
    ui::foundation::{modals::base::ModalBase, utils::get_avatar_color},
};

/// Simple organizer info for the modal
#[derive(Debug, Clone, PartialEq)]
pub struct OrganizerInfo {
    pub user_id: i32,
    pub name: String,
    pub color: String,
}

/// Event modal for creating or editing events
#[component]
pub fn EventModal(
    slug: String,
    event: Option<ScheduleEvent>,
    hackathon_start_date: NaiveDate,
    hackathon_end_date: NaiveDate,
    on_close: EventHandler<()>,
    on_save: EventHandler<()>,
) -> Element {
    let is_edit_mode = event.is_some();

    // Clone slug for different closures
    let slug_for_people = slug.clone();
    let slug_for_save = slug.clone();
    let slug_for_delete = slug.clone();

    // Initialize form state from event if editing, otherwise use defaults
    let initial_name = event.as_ref().map(|e| e.name.clone()).unwrap_or_default();
    let initial_description = event
        .as_ref()
        .and_then(|e| e.description.clone())
        .unwrap_or_default();
    let initial_location = event
        .as_ref()
        .and_then(|e| e.location.clone())
        .unwrap_or_default();
    // Default date to hackathon start date for new events
    let initial_date = event
        .as_ref()
        .map(|e| e.start_time.date().format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| hackathon_start_date.format("%Y-%m-%d").to_string());
    let initial_start_time = event
        .as_ref()
        .map(|e| e.start_time.format("%H:%M").to_string())
        .unwrap_or_default();
    // Default end date to start date if not explicitly different
    let initial_end_date = event
        .as_ref()
        .map(|e| e.end_time.date().format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| hackathon_start_date.format("%Y-%m-%d").to_string());
    let initial_end_time = event
        .as_ref()
        .map(|e| e.end_time.format("%H:%M").to_string())
        .unwrap_or_default();
    let initial_visible_to = event.as_ref().and_then(|e| e.visible_to_role.clone());
    let initial_event_type = event
        .as_ref()
        .map(|e| e.event_type.clone())
        .unwrap_or_else(|| "default".to_string());
    let initial_is_visible = event.as_ref().map(|e| e.is_visible).unwrap_or(true); // Default to visible for new events
    let initial_organizer_ids = event
        .as_ref()
        .map(|e| e.organizer_ids.clone())
        .unwrap_or_default();
    let initial_points = event.as_ref().and_then(|e| e.points);
    let initial_checkin_type = event
        .as_ref()
        .map(|e| e.checkin_type.clone())
        .unwrap_or_else(|| "qr_scan".to_string());
    let event_id = event.as_ref().map(|e| e.id);

    // Form state
    let mut name = use_signal(|| initial_name);
    let mut location = use_signal(|| initial_location);
    let mut description = use_signal(|| initial_description);
    let mut start_date = use_signal(|| initial_date);
    let mut start_time = use_signal(|| initial_start_time);
    let mut end_date = use_signal(|| initial_end_date);
    let mut end_time = use_signal(|| initial_end_time);
    let mut visible_to_role = use_signal(|| initial_visible_to);
    let mut event_type = use_signal(|| initial_event_type);
    let mut is_visible = use_signal(|| initial_is_visible);
    let mut points = use_signal(|| initial_points.map(|p| p.to_string()).unwrap_or_default());
    let mut checkin_type = use_signal(|| initial_checkin_type);
    let mut selected_organizers = use_signal(Vec::<OrganizerInfo>::new);
    let mut has_initialized_organizers = use_signal(|| false);

    // Organizer search
    let mut organizer_search = use_signal(String::new);
    let mut show_organizer_dropdown = use_signal(|| false);

    // Error, loading, and confirmation state
    let mut error = use_signal(|| None::<String>);
    let mut is_saving = use_signal(|| false);
    let mut show_delete_confirm = use_signal(|| false);
    let mut is_deleting = use_signal(|| false);

    // Fetch all people for organizer search and to populate initial organizers
    let people_resource = use_resource(move || {
        let slug = slug_for_people.clone();
        async move {
            let result: Result<Vec<HackathonPerson>, _> = get_hackathon_people(slug).await;
            result.ok()
        }
    });

    // Initialize selected organizers from event when people are loaded (only once)
    let _ = use_memo(move || {
        if let Some(people) = people_resource.read().as_ref().and_then(|p| p.as_ref()) {
            // Only initialize once - don't re-populate after user clears organizers
            if !has_initialized_organizers() {
                let orgs: Vec<OrganizerInfo> = initial_organizer_ids
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
    });

    // Filter organizers based on search
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
                        // Only show organizers/admins
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

    let handle_save = {
        move |_| {
            let slug = slug_for_save.clone();
            let name_val = name();
            let location_val = location();
            let description_val = description();
            let start_date_val = start_date();
            let start_time_val = start_time();
            let end_date_val = end_date();
            let end_time_val = end_time();
            let visible_to_role_val = visible_to_role();
            let event_type_val = event_type();
            let is_visible_val = is_visible();
            let points_val = points();
            let checkin_type_val = checkin_type();
            let organizer_ids_val: Vec<i32> =
                selected_organizers().iter().map(|o| o.user_id).collect();

            spawn(async move {
                is_saving.set(true);
                error.set(None);

                // Validate required fields
                if name_val.trim().is_empty() {
                    error.set(Some("Event name is required".to_string()));
                    is_saving.set(false);
                    return;
                }

                if start_date_val.is_empty() {
                    error.set(Some("Date is required".to_string()));
                    is_saving.set(false);
                    return;
                }

                if start_time_val.is_empty() || end_time_val.is_empty() {
                    error.set(Some("Start and end times are required".to_string()));
                    is_saving.set(false);
                    return;
                }

                // Parse date and times
                let parsed_start_date = match NaiveDate::parse_from_str(&start_date_val, "%Y-%m-%d")
                {
                    Ok(d) => d,
                    Err(_) => {
                        error.set(Some("Invalid start date format".to_string()));
                        is_saving.set(false);
                        return;
                    }
                };

                let parsed_end_date = match NaiveDate::parse_from_str(&end_date_val, "%Y-%m-%d") {
                    Ok(d) => d,
                    Err(_) => {
                        error.set(Some("Invalid end date format".to_string()));
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

                // Validate that start is before end
                if start_datetime >= end_datetime {
                    error.set(Some(
                        "Start date/time must be before end date/time".to_string(),
                    ));
                    is_saving.set(false);
                    return;
                }

                // Validate that event is within hackathon time bounds
                if parsed_start_date < hackathon_start_date {
                    error.set(Some(
                        "Event must start on or after the hackathon start date".to_string(),
                    ));
                    is_saving.set(false);
                    return;
                }
                if parsed_end_date > hackathon_end_date {
                    error.set(Some(
                        "Event must end on or before the hackathon end date".to_string(),
                    ));
                    is_saving.set(false);
                    return;
                }

                // Parse points (empty string = None)
                let parsed_points: Option<i32> = if points_val.trim().is_empty() {
                    None
                } else {
                    match points_val.trim().parse::<i32>() {
                        Ok(p) => Some(p),
                        Err(_) => {
                            error.set(Some("Points must be a valid number".to_string()));
                            is_saving.set(false);
                            return;
                        }
                    }
                };

                if let Some(id) = event_id {
                    // Update existing event
                    let request = UpdateEventRequest {
                        id,
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
                        visible_to_role: visible_to_role_val,
                        event_type: event_type_val,
                        is_visible: is_visible_val,
                        organizer_ids: organizer_ids_val,
                        points: parsed_points,
                        checkin_type: checkin_type_val.clone(),
                    };

                    dioxus_logger::tracing::info!(
                        "Updating event {} with {} organizers: {:?}",
                        id,
                        request.organizer_ids.len(),
                        request.organizer_ids
                    );

                    match update_event(slug, request).await {
                        Ok(_) => {
                            on_save.call(());
                        }
                        Err(e) => {
                            error.set(Some(e.to_string()));
                        }
                    }
                } else {
                    // Create new event
                    let event_slug = name_val
                        .to_lowercase()
                        .chars()
                        .map(|c| if c.is_alphanumeric() { c } else { '-' })
                        .collect::<String>();

                    let request = CreateEventRequest {
                        name: name_val,
                        slug: event_slug,
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
                        visible_to_role: visible_to_role_val,
                        event_type: event_type_val,
                        is_visible: is_visible_val,
                        organizer_ids: organizer_ids_val,
                        points: parsed_points,
                        checkin_type: checkin_type_val,
                    };

                    match create_event(slug, request).await {
                        Ok(_) => {
                            on_save.call(());
                        }
                        Err(e) => {
                            error.set(Some(e.to_string()));
                        }
                    }
                }

                is_saving.set(false);
            });
        }
    };

    let handle_delete = {
        move |_| {
            let slug = slug_for_delete.clone();
            if let Some(id) = event_id {
                spawn(async move {
                    is_deleting.set(true);
                    match delete_event(slug, id).await {
                        Ok(_) => {
                            on_save.call(());
                        }
                        Err(e) => {
                            error.set(Some(e.to_string()));
                            show_delete_confirm.set(false);
                        }
                    }
                    is_deleting.set(false);
                });
            }
        }
    };

    rsx! {
        ModalBase { on_close: move |_| on_close.call(()),
            div { class: "p-6",
                // Header with name input and category badge
                div { class: "flex items-start justify-between gap-4 mb-4",
                    input {
                        r#type: "text",
                        class: "flex-1 text-2xl font-semibold text-foreground-neutral-primary bg-transparent border-none outline-none placeholder:text-foreground-neutral-tertiary",
                        placeholder: "Name of event",
                        value: "{name}",
                        oninput: move |e| name.set(e.value()),
                    }
                    // Category selector as badge
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
                div { class: "flex items-center gap-2 text-foreground-neutral-secondary mb-2",
                    Icon { width: 16, height: 16, icon: LdMapPin }
                    input {
                        r#type: "text",
                        class: "flex-1 text-sm bg-transparent border-none outline-none placeholder:text-foreground-neutral-tertiary",
                        placeholder: "McConomy",
                        value: "{location}",
                        oninput: move |e| location.set(e.value()),
                    }
                }

                // Date and Time
                div { class: "flex flex-wrap items-center gap-2 text-foreground-neutral-secondary mb-2",
                    Icon { width: 16, height: 16, icon: LdClock }
                    input {
                        r#type: "date",
                        class: "text-sm bg-transparent border-none outline-none",
                        value: "{start_date}",
                        oninput: move |e| start_date.set(e.value()),
                    }
                    span { class: "text-sm", "·" }
                    input {
                        r#type: "time",
                        class: "text-sm bg-transparent border-none outline-none",
                        value: "{start_time}",
                        oninput: move |e| start_time.set(e.value()),
                    }
                    span { class: "text-sm", "to" }
                    input {
                        r#type: "date",
                        class: "text-sm bg-transparent border-none outline-none",
                        value: "{end_date}",
                        oninput: move |e| end_date.set(e.value()),
                    }
                    span { class: "text-sm", "·" }
                    input {
                        r#type: "time",
                        class: "text-sm bg-transparent border-none outline-none",
                        value: "{end_time}",
                        oninput: move |e| end_time.set(e.value()),
                    }
                }

                // Points (inline with icon)
                div { class: "flex items-center gap-2 text-foreground-neutral-secondary mb-6",
                    Icon { width: 16, height: 16, icon: LdTarget }
                    input {
                        r#type: "number",
                        class: "w-20 text-sm bg-transparent border-none outline-none placeholder:text-foreground-neutral-tertiary",
                        placeholder: "5",
                        value: "{points}",
                        oninput: move |e| points.set(e.value()),
                    }
                    span { class: "text-sm", "Points" }
                }

                // Event Description
                div { class: "mb-6",
                    h3 { class: "text-sm font-medium text-foreground-neutral-primary mb-2",
                        "Event Description"
                    }
                    div { class: "bg-background-neutral-primary rounded-xl p-4",
                        textarea {
                            class: "w-full h-20 text-sm bg-transparent resize-none placeholder:text-foreground-neutral-tertiary border-none outline-none",
                            placeholder: "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Fusce consequat tincidunt urna placerat pulvinar.",
                            value: "{description}",
                            oninput: move |e| description.set(e.value()),
                        }
                    }
                }

                // Organizers section
                div { class: "mb-6 bg-background-neutral-primary rounded-lg p-4",
                    h3 { class: "text-sm font-medium text-foreground-neutral-primary mb-3",
                        "Organizers"
                    }

                    // Search input
                    div { class: "relative mb-3",
                        div { class: "flex items-center gap-2 px-3 py-2 border border-stroke-neutral-1 rounded-lg",
                            Icon { width: 16, height: 16, icon: LdSearch }
                            input {
                                r#type: "text",
                                class: "flex-1 text-sm bg-transparent border-none outline-none placeholder:text-foreground-neutral-tertiary",
                                placeholder: "Add Organizer",
                                value: "{organizer_search}",
                                oninput: move |e| {
                                    organizer_search.set(e.value());
                                    show_organizer_dropdown.set(true);
                                },
                                onfocus: move |_| show_organizer_dropdown.set(true),
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
                        for org in selected_organizers().iter().cloned() {
                            {
                                let org_id = org.user_id;
                                rsx! {
                                    // Avatar circle with color
                                    div { key: "{org.user_id}", class: "flex items-center justify-between p-3",
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
                                                // todo: console.log(orgs, org_id)
                                                dioxus_logger::tracing::info!(
                                                    "Removing organizer: {}, remaining orgs: {}", org_id, orgs.len()
                                                );
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
                    div { class: "mb-4 p-3 bg-red-50 border border-red-200 rounded-lg",
                        p { class: "text-red-600 text-sm", "{err}" }
                    }
                }

                // Delete confirmation dialog
                if show_delete_confirm() {
                    div { class: "mb-4 p-4 bg-red-50 border border-red-200 rounded-lg",
                        p { class: "text-red-700 font-medium mb-3",
                            "Are you sure you want to delete this event?"
                        }
                        div { class: "flex gap-2",
                            button {
                                class: "px-4 py-2 text-sm border border-stroke-neutral-1 rounded-full hover:bg-background-neutral-secondary-hover",
                                onclick: move |_| show_delete_confirm.set(false),
                                "Cancel"
                            }
                            button {
                                class: "px-4 py-2 text-sm bg-red-600 text-white rounded-full hover:bg-red-700",
                                disabled: is_deleting(),
                                onclick: handle_delete,
                                if is_deleting() {
                                    "Deleting..."
                                } else {
                                    "Yes, Delete"
                                }
                            }
                        }
                    }
                }

                // Action buttons
                div { class: "flex justify-end gap-3",
                    // Only show delete button in edit mode
                    if is_edit_mode && !show_delete_confirm() {
                        button {
                            class: "px-6 py-2.5 bg-[var(--color-status-danger-foreground)] text-white font-medium text-sm rounded-full hover:bg-red-600 transition-colors",
                            onclick: move |_| show_delete_confirm.set(true),
                            "Delete"
                        }
                    }
                    button {
                        class: "px-6 py-2.5 bg-foreground-neutral-primary text-white font-medium text-sm rounded-full hover:opacity-90 transition-opacity",
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
