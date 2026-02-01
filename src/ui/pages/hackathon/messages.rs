use dioxus::prelude::*;
use dioxus_free_icons::{
    Icon,
    icons::ld_icons::{LdCheck, LdChevronDown, LdPlus, LdSearch},
};

use crate::auth::hooks::use_hackathon_role;
use crate::domain::auth::handlers::get_current_user;
use crate::domain::hackathons::types::HackathonInfo;
use crate::domain::messages::handlers::{CreateMessageRequest, create_message, get_messages};
use crate::domain::people::handlers::{HackathonPerson, get_hackathon_people};
use crate::domain::teams::handlers::{
    get_all_teams, get_join_requests, get_my_invitations, get_outgoing_join_requests,
};
use crate::domain::teams::types::TeamListItem;
use crate::ui::foundation::components::{
    Button, ButtonSize, ButtonVariant, ButtonWithIcon, Dropdown, DropdownOption, Input, TabSwitcher,
};
use dioxus::logger::tracing;

fn format_time<T: ToString>(t: &T) -> String {
    let s = t.to_string();
    s.split('.').next().unwrap_or(&s).to_string()
}

#[derive(Clone, Copy, PartialEq)]
enum MessagesTab {
    Announcements,
    ReviewDrafts,
}

#[derive(Clone)]
struct MessageItem {
    id: Option<i32>,
    title: String,
    sender: String,
    tag: String,
    time: String,
    content: String,
    team_name: Option<String>,
    user_id: Option<i32>,
    major: Option<String>,
    graduation_year: Option<String>,
}

#[component]
pub fn HackathonMessages(slug: String) -> Element {
    let _hackathon = use_context::<Signal<HackathonInfo>>();

    // role resource (may early-return if not available)
    let role_resource = use_hackathon_role(slug.clone())?;

    // compute if current user is an admin
    let is_admin: bool = role_resource
        .read()
        .as_ref()
        .and_then(|res| res.as_ref().ok())
        .and_then(|opt| opt.as_ref().map(|r| r.role == "admin"))
        .unwrap_or(false);

    // UI state
    let mut filter_open = use_signal(|| false);
    let filter_values: Signal<Vec<String>> = use_signal(|| Vec::new());
    let active_tab = use_signal(|| MessagesTab::Announcements);
    let tabs: Vec<(MessagesTab, String)> = vec![
        (MessagesTab::Announcements, "Announcements".to_string()),
        (MessagesTab::ReviewDrafts, "Team Requests".to_string()),
    ];
    let search = use_signal(|| String::new());
    let items: Signal<Vec<MessageItem>> = use_signal(|| Vec::new());
    let selected: Signal<Option<usize>> = use_signal(|| None);

    // Composer state
    let mut compose_open = use_signal(|| false);
    let mut new_title = use_signal(|| String::new());
    let mut new_recipients = use_signal(|| Vec::<(String, Option<i32>)>::new());
    let mut new_recipients_display = use_signal(|| String::new());
    let mut new_content = use_signal(|| String::new());
    let mut selected_recipient: Signal<Option<(String, Option<i32>)>> = use_signal(|| None);
    let mut recipients_search = use_signal(|| String::new());
    let mut recipients_open = use_signal(|| false);

    // (no action buttons or comment box for requests/invites)
    let dropdown_options = vec![
        DropdownOption {
            label: "All".into(),
            value: "all".into(),
            selected: true,
        },
        DropdownOption {
            label: "Team".into(),
            value: "team".into(),
            selected: false,
        },
        DropdownOption {
            label: "Mine".into(),
            value: "mine".into(),
            selected: false,
        },
    ];

    // clones for closures
    let mut filter_values_clone = filter_values.clone();
    let mut selected_clone = selected.clone();

    // Fetch messages from server for the current user and populate `items`
    let slug_for_messages = slug.clone();
    let mut messages_resource = use_resource(move || {
        let slug = slug_for_messages.clone();
        async move {
            // Get current user; if none, return None
            match get_current_user().await {
                Ok(Some(user_info)) => {
                    if let Ok(user_id) = user_info.id.parse::<i32>() {
                        match get_messages(slug, user_id).await {
                            Ok(json) => Some(json),
                            Err(e) => {
                                tracing::error!("Failed to fetch messages: {:?}", e);
                                None
                            }
                        }
                    } else {
                        None
                    }
                }
                Ok(None) => None,
                Err(e) => {
                    tracing::error!("Failed to get current user: {:?}", e);
                    None
                }
            }
        }
    });

    // (groups removed)

    // Fetch people list for hackathon (admins only) — only call server when role indicates admin/organizer
    let slug_for_people = slug.clone();
    let role_res_clone_for_people = role_resource.clone();
    let mut people_resource = use_resource(move || {
        let slug = slug_for_people.clone();
        let role_res = role_res_clone_for_people.clone();
        async move {
            // Read the role resource; if not present or user isn't admin/organizer, skip the call
            let rr = role_res.read();
            if let Some(Ok(Some(role))) = rr.as_ref() {
                if role.role == "admin" || role.role == "organizer" {
                    match get_hackathon_people(slug).await {
                        Ok(ps) => Some(ps),
                        Err(e) => {
                            tracing::error!("Failed to fetch people: {:?}", e);
                            None
                        }
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }
    });

    // compute filtered items (index into original items, item) and selected index
    let search_lc = search.read().to_lowercase();
    let filtered_items: Vec<(usize, MessageItem)> = items
        .read()
        .iter()
        .enumerate()
        .filter_map(|(i, it)| {
            let hay =
                format!("{} {} {} {}", it.title, it.sender, it.tag, it.content).to_lowercase();
            if search_lc.is_empty() || hay.contains(&search_lc) {
                Some((i, it.clone()))
            } else {
                None
            }
        })
        .collect();

    let selected_idx_opt = selected.read().as_ref().copied();

    let title_text =
        selected_idx_opt.and_then(|idx| items.read().get(idx).map(|it| it.title.clone()));
    // Prepare owned copies of people so we can render them without borrowing resources in onclick handlers.

    let people_owned: Vec<HackathonPerson> = people_resource
        .read()
        .as_ref()
        .and_then(|opt| opt.as_ref().map(|v| v.clone()))
        .unwrap_or_default();

    // Fetch teams for the hackathon so we can display real team names
    let slug_for_teams = slug.clone();
    let mut teams_resource = use_resource(move || {
        let slug = slug_for_teams.clone();
        async move {
            match get_all_teams(slug, None).await {
                Ok(ts) => Some(ts),
                Err(e) => {
                    tracing::error!("Failed to fetch teams: {:?}", e);
                    None
                }
            }
        }
    });

    // Fetch my invitations for the current user (Team Requests / Invites tab)
    let slug_for_invitations = slug.clone();
    let mut invitations_resource = use_resource(move || {
        let slug = slug_for_invitations.clone();
        async move {
            match get_my_invitations(slug).await {
                Ok(inv) => Some(inv),
                Err(e) => {
                    tracing::error!("Error fetching invitations: {:?}", e);
                    None
                }
            }
        }
    });

    // Fetch pending join requests for user's team (owner view)
    // Only call if the current user has a team (avoids "User is not in a team" server error)
    let slug_for_join_requests = slug.clone();
    let role_res_clone_for_join = role_resource.clone();
    let mut join_requests_resource = use_resource(move || {
        let slug = slug_for_join_requests.clone();
        let role_res = role_res_clone_for_join.clone();
        async move {
            let rr = role_res.read();
            if let Some(Ok(Some(role))) = rr.as_ref() {
                if role.team_id.is_some() {
                    match get_join_requests(slug).await {
                        Ok(reqs) => Some(reqs),
                        Err(e) => {
                            tracing::error!("Error fetching join requests: {:?}", e);
                            None
                        }
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }
    });

    // Fetch outgoing join requests made by current user
    let slug_for_outgoing = slug.clone();
    let mut outgoing_requests_resource = use_resource(move || {
        let slug = slug_for_outgoing.clone();
        async move {
            match get_outgoing_join_requests(slug).await {
                Ok(reqs) => Some(reqs),
                Err(e) => {
                    tracing::error!("Error fetching outgoing join requests: {:?}", e);
                    None
                }
            }
        }
    });

    // Derive teams list from teams_resource (id + name)
    let teams_owned: Vec<(String, Option<i32>)> = teams_resource
        .read()
        .as_ref()
        .and_then(|opt| {
            opt.as_ref().map(|v| {
                v.iter()
                    .map(|t: &TeamListItem| (t.name.clone(), Some(t.id)))
                    .collect()
            })
        })
        .unwrap_or_default();
    let sender_time_text = selected_idx_opt.and_then(|idx| {
        items
            .read()
            .get(idx)
            .map(|it| format!("{} • {}", it.sender, it.time))
    });
    let tag_text = selected_idx_opt.and_then(|idx| items.read().get(idx).map(|it| it.tag.clone()));
    let content_text =
        selected_idx_opt.and_then(|idx| items.read().get(idx).map(|it| it.content.clone()));
    let selected_team_name =
        selected_idx_opt.and_then(|idx| items.read().get(idx).and_then(|it| it.team_name.clone()));

    // Precompute simple people list for dropdown rendering: (user_id, label)
    let people_list_for_dropdown: Vec<(i32, String)> = people_owned
        .iter()
        .map(|p| (p.user_id, p.name.clone().unwrap_or(p.email.clone())))
        .collect();

    // Selected person's major / graduation year (from people API)
    // Prefer major/graduation_year present on the selected MessageItem (from join requests/invitations).
    // Fall back to people API lookup when available (admins).
    let (selected_major_text, selected_grad_text) = if let Some(idx) = selected_idx_opt {
        if let Some(it) = items.read().get(idx) {
            let maj = it.major.clone().or_else(|| {
                it.user_id.and_then(|uid| {
                    people_owned
                        .iter()
                        .find(|p| p.user_id == uid)
                        .and_then(|p| p.major.clone())
                })
            });
            let grad = it.graduation_year.clone().or_else(|| {
                it.user_id.and_then(|uid| {
                    people_owned
                        .iter()
                        .find(|p| p.user_id == uid)
                        .and_then(|p| p.graduation_year.clone())
                })
            });

            (
                maj.unwrap_or_else(|| "Not provided".to_string()),
                grad.unwrap_or_else(|| "Not provided".to_string()),
            )
        } else {
            ("Not provided".to_string(), "Not provided".to_string())
        }
    } else {
        ("Not provided".to_string(), "Not provided".to_string())
    };

    // Counts for debug/info
    let people_count = people_resource
        .read()
        .as_ref()
        .and_then(|r| r.as_ref().map(|v| v.len()))
        .unwrap_or(0);

    // Precompute recipients search string (lowercase) to avoid `let` inside rsx!
    let recipients_search_lc = recipients_search.read().to_lowercase();

    // Prepare stable dropdown entries (clone labels so RSX rendering and onclick closures
    // can each own a String without conflicting moves)
    let people_dropdown_entries: Vec<(i32, String, String)> = people_list_for_dropdown
        .iter()
        .map(|(id, label)| (*id, label.clone(), label.clone()))
        .collect();

    let teams_dropdown_entries: Vec<(String, String, Option<i32>)> = teams_owned
        .iter()
        .map(|(label, tid)| (label.clone(), label.clone(), *tid))
        .collect();

    // when messages_resource or invitations_resource populates, map responses into local `items` signal
    {
        let mut items_signal = items.clone();
        let role_res_clone = role_resource.clone();
        let teams_res_clone = teams_resource.clone();
        let invitations_res_clone = invitations_resource.clone();
        let join_requests_res_clone = join_requests_resource.clone();
        let outgoing_requests_res_clone = outgoing_requests_resource.clone();
        let active_tab_clone = active_tab.clone();
        use_effect(move || {
            // determine current user's id and team_id (if available)
            let mut current_user_id: Option<i32> = None;
            let mut current_team_id: Option<i32> = None;
            let rr = role_res_clone.read();
            if let Some(Ok(Some(role))) = rr.as_ref() {
                current_user_id = Some(role.user_id);
                current_team_id = role.team_id;
            }

            let teams_list: Vec<(String, Option<i32>)> = teams_res_clone
                .read()
                .as_ref()
                .and_then(|opt| {
                    opt.as_ref().map(|v| {
                        v.iter()
                            .map(|t: &TeamListItem| (t.name.clone(), Some(t.id)))
                            .collect()
                    })
                })
                .unwrap_or_default();

            // If announcements tab is active, map messages
            if *active_tab_clone.read() == MessagesTab::Announcements {
                if let Some(res_opt) = messages_resource.read().as_ref() {
                    if let Some(msgs) = res_opt.as_ref() {
                        let mapped = msgs
                            .iter()
                            .map(|m| {
                                // default tag
                                let tag = match m.recipient_type.as_deref() {
                                    Some("team") => {
                                        // find team name from teams list
                                        if let Some(rid) = m.recipient_id {
                                            if Some(rid) == current_team_id {
                                                "Your Team".to_string()
                                            } else if let Some((name, _)) = teams_list
                                                .iter()
                                                .find(|(_, id_opt)| *id_opt == Some(rid))
                                            {
                                                name.clone()
                                            } else {
                                                "Team".to_string()
                                            }
                                        } else {
                                            "Team".to_string()
                                        }
                                    }
                                    Some("user") => "Private".to_string(),
                                    _ => "All".to_string(),
                                };

                                MessageItem {
                                    id: None,
                                    title: m
                                        .title
                                        .split('.')
                                        .next()
                                        .unwrap_or("Announcement")
                                        .to_string(),
                                    sender: format!("User {}", m.sender_user_id),
                                    tag,
                                    time: format_time(&m.created_at),
                                    content: m.content.clone(),
                                    team_name: None,
                                    user_id: Some(m.sender_user_id),
                                    major: None,
                                    graduation_year: None,
                                }
                            })
                            .collect::<Vec<_>>();
                        items_signal.set(mapped);
                    }
                }
            } else {
                // Team Requests / Invites tab: clear left list immediately while invitations load
                if invitations_res_clone.read().is_none() {
                    items_signal.set(vec![]);
                }

                // Combine team-related items: join requests to your team (owner view), outgoing requests you made, and invitations
                let mut combined: Vec<MessageItem> = Vec::new();

                // Incoming join requests for your team (owners)
                if let Some(res_opt) = join_requests_res_clone.read().as_ref() {
                    if let Some(reqs) = res_opt.as_ref() {
                        for r in reqs.iter() {
                            // find team name by id if available
                            let team_name = teams_list
                                .iter()
                                .find(|(_, tid)| *tid == Some(r.team_id))
                                .map(|(n, _)| n.clone());

                            // Build a display title that includes the requestor's graduation year when available
                            let base_name =
                                r.user_name.clone().unwrap_or_else(|| r.user_email.clone());
                            let display_title = if let Some(gy) = r.graduation_year.clone() {
                                if gy.trim().is_empty() {
                                    base_name.clone()
                                } else {
                                    format!("{} • {}", base_name, gy)
                                }
                            } else {
                                base_name.clone()
                            };

                            combined.push(MessageItem {
                                id: Some(r.id),
                                title: display_title,
                                sender: r.user_email.clone(),
                                tag: "Request to join".to_string(),
                                time: format_time(&r.created_at),
                                content: r.message.clone().unwrap_or_default(),
                                team_name,
                                user_id: Some(r.user_id),
                                major: r.major.clone(),
                                graduation_year: r.graduation_year.clone(),
                            });
                        }
                    }
                }

                // Outgoing join requests made by the current user
                if let Some(res_opt) = outgoing_requests_res_clone.read().as_ref() {
                    if let Some(reqs) = res_opt.as_ref() {
                        for r in reqs.iter() {
                            combined.push(MessageItem {
                                id: Some(r.id),
                                title: r.team_name.clone(),
                                sender: "You".to_string(),
                                tag: "Request to join".to_string(),
                                time: format_time(&r.created_at),
                                content: r.message.clone().unwrap_or_default(),
                                team_name: Some(r.team_name.clone()),
                                user_id: None,
                                major: r.major.clone(),
                                graduation_year: r.graduation_year.clone(),
                            });
                        }
                    }
                }

                // Invitations (incoming invitations to you)
                if let Some(res_opt) = invitations_res_clone.read().as_ref() {
                    if let Some(invs) = res_opt.as_ref() {
                        for inv in invs.iter() {
                            combined.push(MessageItem {
                                id: Some(inv.id),
                                title: inv.team_name.clone(),
                                sender: inv
                                    .user_name
                                    .clone()
                                    .unwrap_or_else(|| inv.user_email.clone()),
                                tag: "Invitation".to_string(),
                                time: format_time(&inv.created_at),
                                content: inv.message.clone().unwrap_or_default(),
                                team_name: Some(inv.team_name.clone()),
                                user_id: Some(inv.user_id),
                                major: inv.major.clone(),
                                graduation_year: inv.graduation_year.clone(),
                            });
                        }
                    }
                }

                items_signal.set(combined);
            }
        });
    }

    // Ensure the recipients display always reflects the selected recipient (prevent accidental overrides)
    {
        let mut people_entries = people_dropdown_entries.clone();
        let mut teams_entries = teams_dropdown_entries.clone();
        let mut nr_display = new_recipients_display.clone();
        let mut sel_rec = selected_recipient.clone();
        use_effect(move || {
            if let Some((typ, id_opt)) = sel_rec.read().as_ref().cloned() {
                match typ.as_str() {
                    "all" => nr_display.set("All".to_string()),
                    "user" => {
                        if let Some(uid) = id_opt {
                            if let Some((_id, label, _)) =
                                people_entries.iter().find(|(id, _, _)| *id == uid)
                            {
                                nr_display.set(label.clone());
                            }
                        }
                    }
                    "team" => {
                        if let Some(tid) = id_opt {
                            if let Some((label, _, _)) =
                                teams_entries.iter().find(|(_, _, opt)| *opt == Some(tid))
                            {
                                nr_display.set(label.clone());
                            }
                        }
                    }
                    _ => {}
                }
            }
        });
    }

    rsx! {
        div { class: "pt-11 pb-7 bg-[#f5f7f9] min-h-screen",
            div { class: "flex items-center justify-between",
                h1 { class: "text-[30px] font-semibold leading-[38px] text-foreground-neutral-primary",
                    "Messages"
                }

                div { class: "flex gap-3 items-center",
                    // show create button only for admins/organizers
                    if is_admin {
                        ButtonWithIcon {
                            icon: LdPlus,
                            variant: ButtonVariant::Outline,
                            size: ButtonSize::Normal,
                            class: "",
                            onclick: move |_| compose_open.set(true),
                            "Create Announcement"
                        }
                    }
                }
            }

            // Tab switcher (Announcements / Team Requests)
            div { class: "mt-4",
                TabSwitcher { active_tab, tabs }
            }

            div { class: "mt-6 w-full bg-white rounded-[16px] shadow p-8 flex gap-6",
                // Left column
                div { class: "w-[260px]",
                    div { class: "flex items-center gap-3",
                        Input {
                            label: "",
                            placeholder: Some("Search".into()),
                            value: search.clone(),
                            height: crate::ui::foundation::components::InputHeight::Default,
                        }
                    }
                    div { class: "mt-4",
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
                                div { class: "absolute top-[calc(100%+5px)] left-0 z-10 bg-background-neutral-primary border border-stroke-neutral-1 rounded-lg shadow-lg w-[232px] py-1",
                                    for opt in dropdown_options.clone().iter() {
                                        {
                                            let option_value = opt.value.clone();
                                            let option_label = opt.label.clone();
                                            let option_selected = filter_values_clone.read().contains(&option_value);
                                            let option_value_clone = option_value.clone();

                                            rsx! {
                                                div {
                                                    key: "{option_value}",
                                                    class: if option_selected { "px-3.5 py-2 h-9 bg-background-neutral-subtle-pressed" } else { "px-3.5 py-2 h-9 hover:bg-background-neutral-secondary-enabled cursor-pointer" },
                                                    onclick: move |_| {
                                                        filter_values_clone.set(vec![option_value_clone.clone()]);
                                                        filter_open.set(false);
                                                    },
                                                    div { class: "flex gap-3 items-center",
                                                        div { class: "flex items-center justify-center p-2",
                                                            if option_selected {
                                                                div { class: "w-4 h-4 bg-foreground-neutral-primary rounded flex items-center justify-center",
                                                                    Icon {
                                                                        width: 12,
                                                                        height: 12,
                                                                        icon: LdCheck,
                                                                        class: "text-white",
                                                                    }
                                                                }
                                                            } else {
                                                                div { class: "w-4 h-4 border border-foreground-neutral-primary rounded" }
                                                            }
                                                        }
                                                        p { class: "text-sm leading-5 text-foreground-neutral-primary flex-1", {option_label} }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    div { class: "mt-6 flex flex-col gap-3",
                        for (orig_idx , item) in filtered_items.clone().into_iter() {
                            div {
                                key: "{orig_idx}",
                                class: if selected.read().as_ref().copied().map_or(false, |s| s == orig_idx) { "p-3 bg-background-neutral-subtle-pressed rounded" } else { "p-3 rounded hover:bg-background-neutral-secondary-enabled cursor-pointer" },
                                onclick: move |_| selected_clone.set(Some(orig_idx)),
                                div { class: "flex justify-between items-start",
                                    div { class: "flex flex-col",
                                        p { class: "font-semibold text-foreground-neutral-primary",
                                            {item.title}
                                        }
                                        p { class: "text-sm text-foreground-neutral-secondary",
                                            {item.sender}
                                        }
                                    }
                                    p { class: "text-sm text-foreground-neutral-secondary",
                                        {item.time}
                                    }
                                }
                                div { class: "mt-2 flex items-center gap-2",
                                    div { class: "text-xs bg-background-neutral-primary text-foreground-neutral-primary px-2 py-1 rounded-full",
                                        {item.tag}
                                    }
                                }
                            }
                        }
                    }
                }

                // Right column (detail / composer)
                div { class: "flex-[2] min-w-0 border-l border-stroke-neutral-1 pl-6",
                    // Render different right-column content depending on active tab
                    if *active_tab.read() == MessagesTab::Announcements {
                        if compose_open() {
                            div { class: "flex flex-col gap-4",
                                div { class: "flex items-center justify-between",
                                    h2 { class: "text-2xl font-semibold", "New Announcement" }
                                }
                                div { class: "flex flex-col gap-2",
                                    p { class: "font-medium text-sm text-foreground-neutral-primary",
                                        "Title"
                                    }
                                    Input {
                                        label: "",
                                        placeholder: Some("Placeholder text".into()),
                                        value: new_title.clone(),
                                        height: crate::ui::foundation::components::InputHeight::Default,
                                    }
                                }

                                div { class: "flex flex-col gap-2",
                                    p { class: "font-medium text-sm text-foreground-neutral-primary",
                                        "Send to"
                                    }
                                    // Searchable recipients dropdown (All | Teams | Individuals)
                                    div { class: "relative",
                                        input {
                                            class: "w-full px-3 py-2 rounded bg-background-neutral-primary border border-stroke-neutral-1",
                                            placeholder: "All",
                                            value: new_recipients_display.read().to_string(),
                                            oninput: move |e| {
                                                let v = e.value().clone();
                                                recipients_search.set(v.clone());
                                                new_recipients_display.set(v);
                                                recipients_open.set(true);
                                            },
                                            onfocusin: move |_| recipients_open.set(true),
                                        }

                                        if recipients_open() {
                                            // Simplified single-pass dropdown rendering
                                            div {
                                                class: "absolute z-20 bg-white border border-stroke-neutral-1 rounded mt-1 w-[320px] max-h-64 overflow-auto shadow-lg",
                                                // All option
                                                div {
                                                    class: "px-3 py-2 hover:bg-background-neutral-secondary-enabled cursor-pointer",
                                                    onclick: move |_| {
                                                        selected_recipient.set(Some(("all".to_string(), None)));
                                                        new_recipients_display.set("All".to_string());
                                                        recipients_open.set(false);
                                                    },
                                                    "All"
                                                }

                                                // People section header
                                                div { class: "px-3 pt-2 pb-1 text-xs font-semibold text-foreground-neutral-primary",
                                                    "People"
                                                }

                                                for (user_id , label_text , label_for_click) in people_dropdown_entries.clone().into_iter() {
                                                    if recipients_search_lc.is_empty()
                                                        || label_text.to_lowercase().starts_with(&recipients_search_lc)
                                                    {
                                                        div {
                                                            class: "px-3 py-2 hover:bg-background-neutral-secondary-enabled cursor-pointer bg-yellow-50 text-foreground-neutral-primary",
                                                            onclick: move |_| {
                                                                selected_recipient.set(Some(("user".to_string(), Some(user_id))));
                                                                new_recipients_display.set(label_for_click.clone());
                                                                recipients_open.set(false);
                                                            },
                                                            {label_text}
                                                        }
                                                    }
                                                }

                                                // Teams section header
                                                div { class: "px-3 pt-2 pb-1 text-xs font-semibold text-foreground-neutral-primary",
                                                    "Teams"
                                                }
                                                for (label_text , label_for_click , tid) in teams_dropdown_entries.clone().into_iter() {
                                                    if recipients_search_lc.is_empty()
                                                        || label_text.to_lowercase().starts_with(&recipients_search_lc)
                                                    {
                                                        div {
                                                            class: "px-3 py-2 hover:bg-background-neutral-secondary-enabled cursor-pointer",
                                                            onclick: move |_| {
                                                                selected_recipient.set(Some(("team".to_string(), tid)));
                                                                new_recipients_display.set(label_for_click.clone());
                                                                recipients_open.set(false);
                                                            },
                                                            {label_text}
                                                        }
                                                    }
                                                }

                                            // groups removed
                                            }
                                        }
                                    }
                                }
                                div { class: "flex flex-col gap-2",
                                    p { class: "font-medium text-sm text-foreground-neutral-primary",
                                        "Message"
                                    }
                                    textarea {
                                        class: "w-full h-48 p-4 rounded bg-background-neutral-primary text-foreground-neutral-primary border border-stroke-neutral-1",
                                        oninput: move |e| new_content.set(e.value().clone()),
                                    }
                                }
                                div { class: "flex items-center justify-end gap-3 mt-4",
                                    Button {
                                        variant: ButtonVariant::Outline,
                                        onclick: move |_| {
                                            // Cancel
                                            new_title.set(String::new());
                                            new_recipients.set(Vec::new());
                                            new_recipients_display.set(String::new());
                                            new_content.set(String::new());
                                            compose_open.set(false);
                                        },
                                        "Cancel"
                                    }

                                    Button {
                                        onclick: move |_| {
                                            // Post immediately (no drafts, no scheduling)
                                            let mut slug = slug.clone();
                                            let mut new_title = new_title.clone();
                                            let mut new_recipients_display = new_recipients_display.clone();
                                            let mut new_content = new_content.clone();
                                            let mut items = items.clone();
                                            let mut selected = selected.clone();
                                            let mut compose_open = compose_open.clone();
                                            let mut selected_recipient = selected_recipient.clone();

                                            spawn({
                                                async move {
                                                    // get current user
                                                    if let Ok(Some(user_info)) = get_current_user().await {
                                                        if let Ok(user_id) = user_info.id.parse::<i32>() {
                                                            let sel = selected_recipient.read().as_ref().cloned();
                                                            let (r_type, r_id) = sel.unwrap_or(("all".to_string(), None));
                                                            let req = CreateMessageRequest {
                                                                sender_user_id: user_id,
                                                                recipient_id: r_id,
                                                                recipient_type: Some(r_type.clone()),
                                                                title: new_title.read().clone(),
                                                                content: new_content.read().clone(),
                                                            };

                                                            match create_message(slug.clone(), req).await {
                                                                Ok(_) => {
                                                                    // refresh messages from server
                                                                    if let Ok(msgs) = get_messages(slug.clone(), user_id).await
                                                                    {
                                                                        let mapped = msgs
                                                                            .into_iter()
                                                                            .map(|m| MessageItem {
                                                                                id: None,
                                                                                title: m
                                                                                    .title
                                                                                    .split('.')
                                                                                    .next()
                                                                                    .unwrap_or("Announcement")
                                                                                    .to_string(),
                                                                                sender: format!("User {}", m.sender_user_id),
                                                                                tag: "All".to_string(),
                                                                                time: format_time(&m.created_at),
                                                                                content: m.content.clone(),
                                                                                team_name: None,
                                                                                user_id: Some(m.sender_user_id),
                                                                                major: None,
                                                                                graduation_year: None,
                                                                            })
                                                                            .collect::<Vec<_>>();
                                                                        items.set(mapped);
                                                                        selected.set(Some(0));
                                                                    }
                                                                    new_title.set(String::new());
                                                                    new_recipients_display.set(String::new());
                                                                    new_content.set(String::new());
                                                                    compose_open.set(false);
                                                                }
                                                                Err(e) => {
                                                                    tracing::error!("Failed to create message: {:?}", e);
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            });
                                        },
                                        "Post"
                                    }
                                }
                            }
                        } else {
                            if title_text.is_some() {
                                div {
                                    h2 { class: "text-2xl font-semibold mb-2",
                                        {title_text.as_deref().unwrap()}
                                    }
                                    p { class: "text-sm text-foreground-neutral-secondary mb-4",
                                        {sender_time_text.as_deref().unwrap()}
                                    }
                                    div { class: "mb-4",
                                        div { class: "inline-block bg-background-neutral-subtle-pressed text-foreground-neutral-primary px-3 py-1 rounded-full text-sm",
                                            {tag_text.as_deref().unwrap()}
                                        }
                                    }
                                    p { class: "text-foreground-neutral-primary",
                                        {content_text.as_deref().unwrap()}
                                    }
                                }
                            } else {
                                div { class: "flex items-center justify-center h-full text-foreground-neutral-secondary",
                                    "Select a message to view"
                                }
                            }
                        }
                    } else {
                        // Team Requests / Invites tab
                        if title_text.is_some() {
                            div { class: "flex flex-col gap-6",
                                div { class: "flex items-center gap-4",
                                    div {
                                        p { class: "text-lg font-semibold",
                                            {title_text.as_deref().unwrap()}
                                        }
                                        p { class: "text-sm text-foreground-neutral-secondary",
                                            {sender_time_text.as_deref().unwrap()}
                                        }
                                        if selected_team_name.is_some() {
                                            p { class: "text-sm text-foreground-neutral-secondary",
                                                {selected_team_name.as_deref().unwrap()}
                                            }
                                        }
                                    }
                                }

                                p { class: "text-foreground-neutral-primary",
                                    {content_text.as_deref().unwrap()}
                                }

                                // no comment box here per design
                                div { class: "mt-8",
                                    h3 { class: "text-lg font-semibold mb-4", "Personal Info" }
                                    div { class: "grid grid-cols-2 gap-6",
                                        div { class: "",
                                            p { class: "text-sm text-foreground-neutral-secondary",
                                                "Major"
                                            }
                                            p { class: "font-medium", {selected_major_text} }
                                        }
                                        div { class: "",
                                            p { class: "text-sm text-foreground-neutral-secondary",
                                                "Graduation Year"
                                            }
                                            p { class: "font-medium", {selected_grad_text} }
                                        }
                                    }
                                }
                            }
                        } else {
                            div { class: "flex items-center justify-center h-full text-foreground-neutral-secondary",
                                "Select a request to view"
                            }
                        }
                    }
                }
            }
        }
    }
}
