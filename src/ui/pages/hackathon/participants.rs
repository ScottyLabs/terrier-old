use crate::domain::people::handlers::{
    HackathonPerson, get_hackathon_people,
    mass_update::{
        MassAddPrizeTrackRequest, MassUpdateRoleRequest, mass_add_to_prize_track, mass_update_role,
    },
};
use crate::domain::prizes::handlers::{PrizeInfo, get_prizes};
use crate::ui::foundation::components::{
    Button, ButtonSize, ButtonVariant, Dropdown, DropdownOption, TabSwitcher,
};
use dioxus::prelude::*;
use dioxus_free_icons::{
    Icon,
    icons::ld_icons::{LdChevronDown, LdFilter, LdSearch},
};

#[derive(Clone, Copy, PartialEq)]
enum ParticipantTab {
    Participants,
    Teams,
}

#[component]
pub fn HackathonParticipants(slug: String) -> Element {
    let mut filter_open = use_signal(|| false);
    let mut selected_filters = use_signal(Vec::new);
    let mut search_query = use_signal(|| String::new());
    let active_tab = use_signal(|| ParticipantTab::Participants);
    let mut selected_user_ids = use_signal(std::collections::HashSet::<i32>::new);
    let mut is_bulk_action_modal_open = use_signal(|| false);
    let mut bulk_action_type = use_signal(|| "role".to_string()); // "role" or "prize_track"
    let mut bulk_action_value = use_signal(|| String::new());

    // Resources
    let people_resource = use_resource(use_reactive(&slug, |slug| async move {
        get_hackathon_people(slug).await
    }));

    let prizes_resource =
        use_resource(use_reactive(
            &slug,
            |slug| async move { get_prizes(slug).await },
        ));

    // Filtered People Logic
    let filtered_people = use_memo(move || {
        let people = match people_resource.read().as_ref() {
            Some(Ok(p)) => p.clone(),
            _ => return Vec::new(),
        };

        let query = search_query();
        let query_terms: Vec<String> = query
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();

        // If no query terms, return all (unless other filters apply)
        people
            .into_iter()
            .filter(|person| {
                // Text Search (OR logic for comma separated values)
                let matches_search = if query_terms.is_empty() {
                    true
                } else {
                    query_terms.iter().any(|term| {
                        person
                            .name
                            .clone()
                            .unwrap_or_default()
                            .to_lowercase()
                            .contains(term)
                            || person.email.to_lowercase().contains(term)
                            || person.role.to_lowercase().contains(term)
                    })
                };

                // Dropdown Filters
                let matches_filters = if selected_filters().is_empty() {
                    true
                } else {
                    // Example filter logic mapping
                    let filters = selected_filters();
                    let mut matches = false;
                    if filters.contains(&"organizers".to_string()) && person.role == "organizer" {
                        matches = true;
                    }
                    if filters.contains(&"sponsors".to_string()) && person.role == "sponsor" {
                        matches = true;
                    }
                    if filters.contains(&"cmu_students".to_string())
                        && person.email.ends_with("@cmu.edu")
                    {
                        matches = true;
                    } // simplistic check
                    matches
                };

                matches_search && matches_filters
            })
            .collect::<Vec<HackathonPerson>>()
    });

    // Helper to get target users for bulk action
    let get_target_users = move || {
        let selected = selected_user_ids();
        if !selected.is_empty() {
            selected.into_iter().collect::<Vec<i32>>()
        } else {
            // If none selected, apply to all filtered
            filtered_people().iter().map(|p| p.user_id).collect()
        }
    };

    let handle_bulk_apply = move |_| {
        // Trigger modal
        is_bulk_action_modal_open.set(true);
    };

    let confirm_bulk_action = move |_| async move {
        let target_users = get_target_users();
        if target_users.is_empty() {
            return;
        }

        let action = bulk_action_type();
        let value = bulk_action_value();
        let slug_clone = slug.clone();

        if action == "role" {
            let req = MassUpdateRoleRequest {
                user_ids: target_users,
                role: value,
            };
            let _ = mass_update_role(slug_clone, req).await;
        } else if action == "prize_track" {
            if let Ok(prize_id) = value.parse::<i32>() {
                let req = MassAddPrizeTrackRequest {
                    user_ids: target_users,
                    prize_track_id: prize_id,
                };
                let _ = mass_add_to_prize_track(slug_clone, req).await;
            }
        }

        is_bulk_action_modal_open.set(false);
        people_resource.restart();
        selected_user_ids.write().clear();
    };

    let toggle_selection = move |id: i32| {
        let mut ids = selected_user_ids.write();
        if ids.contains(&id) {
            ids.remove(&id);
        } else {
            ids.insert(id);
        }
    };

    let toggle_all = move |_| {
        let current_filtered = filtered_people();
        let all_ids: std::collections::HashSet<i32> =
            current_filtered.iter().map(|p| p.user_id).collect();
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
    };

    // UI Configuration
    let filter_options = vec![
        DropdownOption {
            label: "CMU Students".to_string(),
            value: "cmu_students".to_string(),
            selected: selected_filters().contains(&"cmu_students".to_string()),
        },
        DropdownOption {
            label: "Organizers".to_string(),
            value: "organizers".to_string(),
            selected: selected_filters().contains(&"organizers".to_string()),
        },
        DropdownOption {
            label: "Sponsors".to_string(),
            value: "sponsors".to_string(),
            selected: selected_filters().contains(&"sponsors".to_string()),
        },
    ];

    let tabs = vec![
        (ParticipantTab::Participants, "Participants".to_string()),
        (ParticipantTab::Teams, "Teams".to_string()),
    ];

    let search_placeholder = match active_tab() {
        ParticipantTab::Participants => "Search participants (comma helper for multiple)",
        ParticipantTab::Teams => "Search teams",
    };

    // Determine button text
    let target_count = if selected_user_ids().is_empty() {
        filtered_people().len()
    } else {
        selected_user_ids().len()
    };
    let button_text = format!("Apply to {} Users", target_count);

    rsx! {
        div { class: "flex flex-col h-full relative",
            h1 { class: "text-[30px] font-semibold leading-[38px] text-foreground-neutral-primary pt-11 pb-7",
                "Participants"
            }

            div { class: "mb-6",
                TabSwitcher { active_tab, tabs }
            }

            div { class: "flex flex-col gap-7 flex-1 min-h-0",
                div { class: "flex items-center justify-between",
                    div { class: "flex items-center gap-2",
                        // Search bar
                        div { class: "w-[405px] h-10 border border-stroke-neutral-1 rounded-full flex items-center px-3 py-1",
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
                                oninput: move |evt| search_query.set(evt.value()),
                            }
                        }

                        // Filter button
                        div { class: "relative",
                            button {
                                class: "bg-foreground-neutral-primary text-white font-semibold text-sm leading-5 rounded-full px-4 py-[9px] flex gap-2 items-center cursor-pointer",
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
                                        },
                                    }
                                }
                            }
                        }
                    }

                    if active_tab() == ParticipantTab::Participants {
                        Button {
                            size: ButtonSize::Compact,
                            onclick: handle_bulk_apply,
                            "{button_text}"
                        }
                    }
                }

                // List
                div { class: "bg-background-neutral-primary rounded-[20px] p-7 flex flex-col overflow-y-auto flex-1",
                    // Header Row for Select All
                    if active_tab() == ParticipantTab::Participants {
                        div { class: "flex items-center gap-4 py-2 border-b border-stroke-neutral-1 mb-2",
                            input {
                                r#type: "checkbox",
                                class: "w-4 h-4 rounded border-gray-300",
                                onchange: toggle_all,
                                checked: {
                                    let all_ids: std::collections::HashSet<i32> = filtered_people().iter().map(|p| p.user_id).collect();
                                    !all_ids.is_empty() && all_ids.iter().all(|id| selected_user_ids().contains(id))
                                }
                            }
                            span { class: "text-sm font-semibold text-foreground-neutral-secondary", "Select All Filtered" }
                        }
                    }

                    if active_tab() == ParticipantTab::Participants {
                         for person in filtered_people() {
                            div {
                                key: "{person.user_id}",
                                class: "flex items-center justify-between py-3 border-b border-stroke-neutral-1 last:border-0",
                                div { class: "flex items-center gap-4",
                                    input {
                                        r#type: "checkbox",
                                        class: "w-4 h-4 rounded border-gray-300",
                                        checked: selected_user_ids().contains(&person.user_id),
                                        onchange: move |_| toggle_selection(person.user_id),
                                    }
                                    div {
                                        p { class: "text-base font-medium leading-6 text-foreground-neutral-primary",
                                            "{person.name.clone().unwrap_or_else(|| \"Unknown\".to_string())}"
                                        }
                                        p { class: "text-xs text-foreground-neutral-tertiary", "{person.email}" }
                                    }
                                }

                                p { class: "text-xs font-medium leading-4 text-foreground-neutral-primary px-4 bg-background-neutral-secondary rounded-full py-1",
                                    "{person.role}"
                                }
                            }
                        }
                    } else {
                        // Teams Placeholder
                        div { class: "text-center text-foreground-neutral-tertiary py-10", "Teams view not implemented yet" }
                    }
                }
            }

            // Bulk Action Modal
            if is_bulk_action_modal_open() {
                div { class: "absolute inset-0 bg-black/50 flex items-center justify-center z-50",
                    div { class: "bg-white rounded-[20px] p-6 w-[400px] flex flex-col gap-4 shadow-xl",
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
