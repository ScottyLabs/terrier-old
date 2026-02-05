use dioxus::prelude::*;
use dioxus_free_icons::{
    Icon,
    icons::ld_icons::{LdChevronDown, LdChevronLeft, LdChevronRight, LdSearch},
};
use std::collections::HashSet;

use crate::{
    auth::{HackathonRole, HackathonRoleType, PEOPLE_ROLES, hooks::use_require_access_or_redirect},
    domain::people::handlers::{
        HackathonPerson, UpdateRoleRequest, get_hackathon_people,
        mass_update::{
            MassAddPrizeTrackRequest, MassUpdateRoleRequest, mass_add_to_prize_track,
            mass_update_role,
        },
        remove_hackathon_person, update_person_role,
    },
    domain::prizes::handlers::{PrizeInfo, get_prizes},
    ui::{
        features::people::PeopleModal,
        foundation::components::{
            Button, ButtonSize, ButtonVariant, Dropdown, DropdownOption, TabSwitcher,
        },
    },
};

#[derive(Clone, Copy, PartialEq)]
enum PeopleTab {
    Individuals,
    Pairs, // Added for completeness, matching TabSwitcher usage if needed
    Teams,
}

/// Available roles for users
const AVAILABLE_ROLES: [(&str, &str); 5] = [
    ("participant", "Participant"),
    ("judge", "Judge"),
    ("sponsor", "Sponsor"),
    ("organizer", "Organizer"),
    ("admin", "Admin"),
];

#[component]
pub fn HackathonPeople(slug: String) -> Element {
    if let Some(no_access) = use_require_access_or_redirect(PEOPLE_ROLES) {
        return no_access;
    }

    let slug_for_remove = slug.clone();
    let slug_for_role_update = slug.clone();

    let mut filter_open = use_signal(|| false);
    let mut selected_filters = use_signal(Vec::new);
    let mut active_tab = use_signal(|| PeopleTab::Individuals);
    let mut search_query = use_signal(String::new);
    let mut selected_person = use_signal(|| None::<HackathonPerson>);
    let mut show_modal = use_signal(|| false);
    let mut updating_role: Signal<Option<i32>> = use_signal(|| None); // Track which user's role is being updated

    // Pagination State
    let mut page = use_signal(|| 0);
    const PER_PAGE: u64 = 50;

    // Mass Update State
    let mut selected_user_ids = use_signal(HashSet::<i32>::new);
    let mut is_bulk_action_modal_open = use_signal(|| false);
    let mut bulk_action_type = use_signal(|| "role".to_string()); // "role" or "prize_track"
    let mut bulk_action_value = use_signal(|| String::new());

    // Get user's role from context
    let user_role = use_context::<Signal<Option<HackathonRole>>>();
    let is_admin = user_role
        .read()
        .as_ref()
        .and_then(|r| r.role_type())
        .map(|rt| rt == HackathonRoleType::Admin)
        .unwrap_or(false);

    // Fetch hackathon people
    let mut people_resource = use_resource(use_reactive(
        &(slug.clone(), page(), search_query(), selected_filters()),
        move |(slug, page_val, search_val, filters_val)| async move {
            let roles = if filters_val.is_empty() {
                None
            } else {
                Some(filters_val)
            };
            let search = if search_val.is_empty() {
                None
            } else {
                Some(search_val)
            };
            get_hackathon_people(slug, Some(page_val as u64), Some(PER_PAGE), search, roles).await
        },
    ));

    // Fetch prizes for bulk add (only needed if admin)
    let prizes_resource = use_resource(use_reactive(&slug, move |slug| async move {
        if is_admin {
            get_prizes(slug).await
        } else {
            Ok(Vec::new())
        }
    }));

    // Filter options for different roles
    let filter_options = vec![
        DropdownOption {
            label: "Participant".to_string(),
            value: "participant".to_string(),
            selected: selected_filters().contains(&"participant".to_string()),
        },
        DropdownOption {
            label: "Judge".to_string(),
            value: "judge".to_string(),
            selected: selected_filters().contains(&"judge".to_string()),
        },
        DropdownOption {
            label: "Sponsor".to_string(),
            value: "sponsor".to_string(),
            selected: selected_filters().contains(&"sponsor".to_string()),
        },
        DropdownOption {
            label: "Organizer".to_string(),
            value: "organizer".to_string(),
            selected: selected_filters().contains(&"organizer".to_string()),
        },
        DropdownOption {
            label: "Admin".to_string(),
            value: "admin".to_string(),
            selected: selected_filters().contains(&"admin".to_string()),
        },
    ];

    let tabs = vec![
        (PeopleTab::Individuals, "Individuals".to_string()),
        (PeopleTab::Teams, "Teams".to_string()),
    ];

    let search_placeholder = match active_tab() {
        PeopleTab::Individuals => "Search individuals",
        PeopleTab::Teams => "Search teams",
        _ => "Search",
    };

    let show_filter = matches!(active_tab(), PeopleTab::Individuals);

    // Bulk Action Logic
    let toggle_all = move |_| {
        if let Some(Ok(response)) = people_resource.read().as_ref() {
            let current_filtered = &response.people;
            let all_ids: HashSet<i32> = current_filtered.iter().map(|p| p.user_id).collect();
            let mut selected = selected_user_ids.write();

            // If all currently filtered are selected, deselect them. Otherwise, select them.
            let all_selected = all_ids.iter().all(|id| selected.contains(id));

            if all_selected {
                for id in all_ids {
                    selected.remove(&id);
                }
            } else {
                for id in all_ids {
                    selected.insert(id);
                }
            }
        }
    };

    let get_target_users = move || {
        let selected = selected_user_ids();
        if !selected.is_empty() {
            selected.into_iter().collect::<Vec<i32>>()
        } else {
            // If none selected, apply to all filtered (loaded on current page)
            if let Some(Ok(response)) = people_resource.read().as_ref() {
                response.people.iter().map(|p| p.user_id).collect()
            } else {
                Vec::new()
            }
        }
    };

    let confirm_bulk_action = move |_| {
        let slug = slug.clone();
        async move {
            if !is_admin {
                return;
            }

            let target_users = get_target_users();
            if target_users.is_empty() {
                return;
            }

            let action = bulk_action_type();
            let value = bulk_action_value();

            if action == "role" {
                let req = MassUpdateRoleRequest {
                    user_ids: target_users,
                    role: value,
                };
                let _ = mass_update_role(slug, req).await;
            } else if action == "prize_track" {
                if let Ok(prize_id) = value.parse::<i32>() {
                    let req = MassAddPrizeTrackRequest {
                        user_ids: target_users,
                        prize_track_id: prize_id,
                    };
                    let _ = mass_add_to_prize_track(slug, req).await;
                }
            }

            is_bulk_action_modal_open.set(false);
            people_resource.restart();
            selected_user_ids.write().clear();
        }
    };

    let target_count = if selected_user_ids().is_empty() {
        if let Some(Ok(response)) = people_resource.read().as_ref() {
            response.people.len()
        } else {
            0
        }
    } else {
        selected_user_ids().len()
    };
    let bulk_button_text = format!("Apply to {} Users", target_count);

    rsx! {
        div { class: "flex flex-col h-full relative",
            h1 { class: "text-[30px] font-semibold leading-[38px] text-foreground-neutral-primary pt-11 pb-7",
                "People"
            }

            div { class: "mb-6",
                TabSwitcher { active_tab, tabs }
            }

            div { class: "flex flex-col gap-7 flex-1 min-h-0",
                div { class: "flex items-center justify-between max-w-full flex-col gap-7 md:gap-0 md:flex-row",
                    div { class: "flex items-center gap-2",
                        // Search bar
                        div { class: "flex-1 h-10 border border-stroke-neutral-1 rounded-full flex items-center px-3 py-1",
                            Icon {
                                width: 20,
                                height: 20,
                                icon: LdSearch,
                                class: "text-foreground-neutral-tertiary",
                            }
                            input {
                                class: "flex-1 px-2.5 text-sm leading-5 text-foreground-neutral-tertiary outline-none bg-transparent",
                                placeholder: "{search_placeholder}",
                                r#type: "text",
                                value: "{search_query}",
                                oninput: move |e| {
                                    search_query.set(e.value());
                                    page.set(0); // Reset to first page on search
                                },
                            }
                        }

                        // Filter button and dropdown (only on Individuals tab)
                        if show_filter {
                            div { class: "relative",
                                button {
                                    class: "flex-3 bg-foreground-neutral-primary text-white font-semibold text-sm leading-5 rounded-full px-4 py-[9px] flex gap-2 items-center cursor-pointer",
                                    onclick: move |_| filter_open.set(!filter_open()),
                                    "Filter"
                                    Icon {
                                        width: 20,
                                        height: 20,
                                        icon: LdChevronDown,
                                        class: "text-white inline-block",
                                    }
                                }

                                if filter_open() {
                                    div { class: "absolute top-[calc(100%+5px)] right-0 z-10",
                                        Dropdown {
                                            options: filter_options.clone(),
                                            on_change: move |new_values| {
                                                selected_filters.set(new_values);
                                                page.set(0); // Reset to first page on filter change
                                            },
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if is_admin && show_filter {
                        Button {
                            size: ButtonSize::Compact,
                            onclick: move |_| is_bulk_action_modal_open.set(true),
                            "{bulk_button_text}"
                        }
                    }
                }

                // People list
                div { class: "bg-background-neutral-primary rounded-[20px] p-7 flex flex-col overflow-y-auto flex-1",
                    {
                        match people_resource.read().as_ref() {
                            Some(Ok(response)) => {
                                let people = response.people.clone();
                                let total = response.total;
                                let total_pages = (total + PER_PAGE - 1) / PER_PAGE;

                                rsx! {
                                    // Header Row for Select All (Admin only)
                                    if is_admin && show_filter && !people.is_empty() {
                                        div { class: "flex items-center gap-4 py-2 border-b border-stroke-neutral-1 mb-2",
                                            input {
                                                r#type: "checkbox",
                                                class: "w-4 h-4 rounded border-gray-300",
                                                onchange: toggle_all,
                                                checked: {
                                                    let all_ids: HashSet<i32> = people
                                                        .iter()
                                                        .map(|p| p.user_id)
                                                        .collect();
                                                    !all_ids.is_empty() && all_ids.iter().all(|id| selected_user_ids().contains(id))
                                                },
                                            }
                                            span { class: "text-sm font-semibold text-foreground-neutral-secondary",
                                                "Select All on Page"
                                            }
                                        }
                                    }

                                    if people.is_empty() {
                                        div { class: "flex items-center justify-center h-full",
                                            p { class: "text-foreground-neutral-secondary", "No people found" }
                                        }
                                    } else {
                                        for person in people {
                                            // Custom person row with role dropdown
                                            div {
                                                key: "{person.user_id}",
                                                class: "flex flex-col sm:flex-row sm:items-center justify-between py-3 border-b border-stroke-neutral-1 gap-3",

                                                div { class: "flex items-center gap-4 flex-1",
                                                    if is_admin && show_filter {
                                                        input {
                                                            r#type: "checkbox",
                                                            class: "w-4 h-4 rounded border-gray-300",
                                                            checked: selected_user_ids().contains(&person.user_id),
                                                            onchange: move |_| {
                                                                let mut ids = selected_user_ids.write();
                                                                let id = person.user_id;
                                                                if ids.contains(&id) {
                                                                    ids.remove(&id);
                                                                } else {
                                                                    ids.insert(id);
                                                                }
                                                            },
                                                        }
                                                    }

                                                    // Left side: Name and email
                                                    div { class: "flex flex-col min-w-0 flex-1",
                                                        p { class: "text-base font-medium leading-6 text-foreground-neutral-primary truncate",
                                                            "{person.name.clone().unwrap_or_else(|| \"Unknown\".to_string())}"
                                                        }
                                                        p { class: "text-sm text-foreground-neutral-secondary truncate",
                                                            "{person.email}"
                                                        }
                                                    }
                                                }

                                                // Right side: Role selector and View button
                                                div { class: "flex items-center gap-3 flex-shrink-0",
                                                    // Role dropdown (only for admins)
                                                    if is_admin {
                                                        {
                                                            let person_id = person.user_id;
                                                            let current_role = person.role.clone();
                                                            let slug = slug_for_role_update.clone();
                                                            let is_updating = updating_role().map(|id| id == person_id).unwrap_or(false);

                                                            rsx! {
                                                                select {
                                                                    class: "px-3 py-1.5 text-sm font-medium rounded-lg border border-stroke-neutral-1 bg-background-neutral-primary text-foreground-neutral-primary cursor-pointer",
                                                                    disabled: is_updating,
                                                                    value: "{current_role}",
                                                                    onchange: move |evt| {
                                                                        let new_role = evt.value();
                                                                        let slug = slug.clone();
                                                                        spawn(async move {
                                                                            updating_role.set(Some(person_id));
                                                                            let request = UpdateRoleRequest {
                                                                                role: new_role,
                                                                            };
                                                                            let _ = update_person_role(slug, person_id, request).await;
                                                                            people_resource.restart();
                                                                            updating_role.set(None);
                                                                        });
                                                                    },
                                                                    for (value , label) in AVAILABLE_ROLES.iter() {
                                                                        option { value: *value, selected: current_role.to_lowercase() == *value, "{label}" }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    } else {
                                                        // Just show role badge for non-admins
                                                        span { class: "px-3 py-1 text-xs font-semibold leading-4 rounded-full bg-background-neutral-secondary-enabled text-foreground-neutral-primary",
                                                            "{format_role(&person.role)}"
                                                        }
                                                    }

                                                    // View button
                                                    {
                                                        let person = person.clone();
                                                        rsx! {
                                                            button {
                                                                class: "px-4 py-1.5 text-sm font-semibold rounded-full bg-foreground-neutral-primary text-white cursor-pointer",
                                                                onclick: move |_| {
                                                                    selected_person.set(Some(person.clone()));
                                                                    show_modal.set(true);
                                                                },
                                                                "View"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        // Pagination Controls
                                        if total_pages > 1 {
                                            div { class: "flex items-center justify-between pt-4 border-t border-stroke-neutral-1 mt-auto",
                                                div { class: "text-sm text-foreground-neutral-secondary",
                                                    "Showing {page() * PER_PAGE + 1} to {std::cmp::min((page() + 1) * PER_PAGE, total)} of {total} results"
                                                }
                                                div { class: "flex gap-2",
                                                    button {
                                                        class: "p-2 rounded-full hover:bg-background-neutral-secondary-hover disabled:opacity-50 disabled:cursor-not-allowed",
                                                        disabled: page() == 0,
                                                        onclick: move |_| page.set(page() - 1),
                                                        Icon {
                                                            width: 20,
                                                            height: 20,
                                                            icon: LdChevronLeft,
                                                            class: "text-foreground-neutral-primary",
                                                        }
                                                    }
                                                    div { class: "flex items-center px-4 py-1 text-sm font-medium",
                                                        "Page {page() + 1} of {total_pages}"
                                                    }
                                                    button {
                                                        class: "p-2 rounded-full hover:bg-background-neutral-secondary-hover disabled:opacity-50 disabled:cursor-not-allowed",
                                                        disabled: page() >= total_pages - 1,
                                                        onclick: move |_| page.set(page() + 1),
                                                        Icon {
                                                            width: 20,
                                                            height: 20,
                                                            icon: LdChevronRight,
                                                            class: "text-foreground-neutral-primary",
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Some(Err(err)) => rsx! {
                                div { class: "flex items-center justify-center h-full text-red-500",
                                    "Error loading people: {err}"
                                }
                            },
                            None => rsx! {
                                div { class: "flex items-center justify-center h-full",
                                    "Loading..."
                                }
                            },
                        }
                    }
                }
            }

            if show_modal() {
                {
                    let person = selected_person().unwrap();
                    rsx! {
                        PeopleModal {
                            user_name: person.name.clone().unwrap_or_else(|| "Unknown".to_string()),
                            user_email: person.email.clone(),
                            role: person.role.clone(),
                            display_name: None,
                            portfolio: None,
                            major: None,
                            graduation_year: None,
                            dietary_restrictions: None,
                            shirt_size: None,
                            is_admin: is_admin,
                            on_close: move |_| show_modal.set(false),
                            on_remove: move |_| {
                                let slug = slug_for_remove.clone();
                                let person_id = person.user_id;
                                spawn(async move {
                                    let _ = remove_hackathon_person(slug, person_id).await;
                                    people_resource.restart();
                                    show_modal.set(false);
                                });
                            },
                            on_send_message: move |_| {},
                        }
                    }
                }
            }

            if is_bulk_action_modal_open() {
                div {
                    class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4",
                    onclick: move |_| is_bulk_action_modal_open.set(false),
                    div {
                        class: "bg-background-neutral-primary rounded-[20px] p-8 max-w-md w-full shadow-2xl",
                        onclick: move |e| e.stop_propagation(),

                        div { class: "flex flex-col gap-6",
                            h2 { class: "text-xl font-bold", "Bulk Actions" }
                            p { "Applying to {target_count} users." }

                            div { class: "flex flex-col gap-2",
                                label { class: "text-sm font-medium", "Action Type" }
                                select {
                                    class: "border rounded p-2",
                                    onchange: move |evt| bulk_action_type.set(evt.value()),
                                    value: "{bulk_action_type}",
                                    option { value: "role", "Change Role" }
                                    option { value: "prize_track", "Add to Prize Track" }
                                }
                            }

                            if bulk_action_type() == "role" {
                                div { class: "flex flex-col gap-2",
                                    label { class: "text-sm font-medium", "Select Role" }
                                    select {
                                        class: "border rounded p-2",
                                        onchange: move |evt| bulk_action_value.set(evt.value()),
                                        value: "{bulk_action_value}",
                                        option { value: "", "Select a role..." }
                                        option { value: "participant", "Participant" }
                                        option { value: "judge", "Judge" }
                                        option { value: "sponsor", "Sponsor" }
                                        option { value: "organizer", "Organizer" }
                                    }
                                }
                            } else if bulk_action_type() == "prize_track" {
                                div { class: "flex flex-col gap-2",
                                    label { class: "text-sm font-medium", "Select Prize Track" }
                                    select {
                                        class: "border rounded p-2",
                                        onchange: move |evt| bulk_action_value.set(evt.value()),
                                        value: "{bulk_action_value}",
                                        option { value: "", "Select a prize track..." }
                                        if let Some(Ok(prizes)) = prizes_resource.read().as_ref() {
                                            for prize in prizes {
                                                option { value: "{prize.id}", "{prize.name}" }
                                            }
                                        }
                                    }
                                }
                            }

                            div { class: "flex justify-end gap-2 mt-4",
                                Button {
                                    variant: ButtonVariant::Secondary,
                                    onclick: move |_| is_bulk_action_modal_open.set(false),
                                    "Cancel"
                                }
                                Button {
                                    variant: ButtonVariant::Success,
                                    onclick: confirm_bulk_action,
                                    "Confirm"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn format_role(role: &str) -> String {
    let mut chars = role.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
