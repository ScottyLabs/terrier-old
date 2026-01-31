use dioxus::prelude::*;
use dioxus_free_icons::{
    Icon,
    icons::ld_icons::{LdCheck, LdChevronDown, LdPlus, LdSearch},
};

use crate::auth::hooks::use_hackathon_role;
use crate::domain::auth::handlers::get_current_user;
use crate::domain::hackathons::types::HackathonInfo;
use crate::domain::messages::handlers::{CreateMessageRequest, create_message, get_messages};
use crate::ui::foundation::components::{
    Button, ButtonSize, ButtonVariant, ButtonWithIcon, Dropdown, DropdownOption, Input, TabSwitcher,
};
use dioxus::logger::tracing;

#[derive(Clone, Copy, PartialEq)]
enum MessagesTab {
    Announcements,
    ReviewDrafts,
}

#[derive(Clone)]
struct MessageItem {
    title: String,
    sender: String,
    tag: String,
    time: String,
    content: String,
}

#[component]
pub fn HackathonMessages(slug: String) -> Element {
    let _hackathon = use_context::<Signal<HackathonInfo>>();

    let active_tab = use_signal(|| MessagesTab::Announcements);
    let search = use_signal(String::new);
    let mut filter_values = use_signal(Vec::<String>::new);
    let mut selected = use_signal(|| None::<usize>);
    let mut filter_open = use_signal(|| false);
    let mut compose_open = use_signal(|| false);
    let mut new_title = use_signal(String::new);
    let mut new_recipients = use_signal(Vec::<String>::new);
    let mut new_content = use_signal(String::new);
    let mut new_recipients_display = use_signal(String::new);

    // role resource to decide whether to show create button
    let role_resource = use_hackathon_role(slug.clone()).ok();
    let is_admin = match &role_resource {
        Some(res) => match res.read().as_ref() {
            Some(Ok(Some(role))) => role.role == "admin" || role.role == "organizer",
            _ => false,
        },
        None => false,
    };

    // sample data
    let mut items = use_signal(|| {
        vec![
            MessageItem {
                title: "Announcement title".into(),
                sender: "Admin organizer".into(),
                tag: "All".into(),
                time: "2h".into(),
                content: "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Fusce consequat tincidunt urna placerat pulvinar. Sed facilisis felis sed vehicula consequat.".into(),
            },
            MessageItem {
                title: "Message title".into(),
                sender: "Admin organizer".into(),
                tag: "Team name".into(),
                time: "2h".into(),
                content: "Short team message content sample.".into(),
            },
            MessageItem {
                title: "Announcement title".into(),
                sender: "Admin organizer".into(),
                tag: "All".into(),
                time: "1d".into(),
                content: "Another announcement content.".into(),
            },
        ]
    });

    let tabs = vec![
        (MessagesTab::Announcements, "Announcements".to_string()),
        (MessagesTab::ReviewDrafts, "Review drafts".to_string()),
    ];

    // Dropdown options
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

    // when messages_resource populates, map responses into local `items` signal
    {
        let mut items_signal = items.clone();
        use_effect(move || {
            if let Some(res_opt) = messages_resource.read().as_ref() {
                if let Some(msgs) = res_opt.as_ref() {
                    let mapped = msgs
                        .iter()
                        .map(|m| MessageItem {
                            title: m
                                .content
                                .split('.')
                                .next()
                                .unwrap_or("Announcement")
                                .to_string(),
                            sender: format!("User {}", m.sender_user_id),
                            tag: "All".into(),
                            time: m.created_at.to_string(),
                            content: m.content.clone(),
                        })
                        .collect::<Vec<_>>();
                    items_signal.set(mapped);
                }
            }
        });
    }

    // compute filtered items based on filter_values and search
    let filtered_items = {
        let filter_vals = filter_values.read().clone();
        let search_q = search.read().to_lowercase();
        items
            .read()
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                if !search_q.is_empty() {
                    let hay =
                        format!("{} {} {}", item.title, item.sender, item.content).to_lowercase();
                    if !hay.contains(&search_q) {
                        return false;
                    }
                }

                if filter_vals.is_empty() || filter_vals.contains(&"all".to_string()) {
                    return true;
                }

                if filter_vals.contains(&"team".to_string()) {
                    return item.tag.to_lowercase() != "all";
                }

                if filter_vals.contains(&"mine".to_string()) {
                    return item.sender.to_lowercase() == "you";
                }

                true
            })
            .map(|(i, it)| (i, it.clone()))
            .collect::<Vec<(usize, MessageItem)>>()
    };

    // Precompute selected message text values to avoid complex inline RSX expressions
    let selected_idx_opt = selected.read().as_ref().copied();
    let title_text =
        selected_idx_opt.and_then(|idx| items.read().get(idx).map(|it| it.title.clone()));
    let sender_time_text = selected_idx_opt.and_then(|idx| {
        items
            .read()
            .get(idx)
            .map(|it| format!("{} • {}", it.sender, it.time))
    });
    let tag_text = selected_idx_opt.and_then(|idx| items.read().get(idx).map(|it| it.tag.clone()));
    let content_text =
        selected_idx_opt.and_then(|idx| items.read().get(idx).map(|it| it.content.clone()));

    rsx! {
        div { class: "pt-11 pb-7",
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

            div { class: "mt-6 w-full bg-white rounded-[12px] shadow p-6 flex gap-6",
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
                                                        p { class: "text-sm leading-5 text-foreground-neutral-primary flex-1", "{option_label}" }
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
                                            "{item.title}"
                                        }
                                        p { class: "text-sm text-foreground-neutral-secondary",
                                            "{item.sender}"
                                        }
                                    }
                                    p { class: "text-sm text-foreground-neutral-secondary",
                                        "{item.time}"
                                    }
                                }
                                div { class: "mt-2 flex items-center gap-2",
                                    div { class: "text-xs bg-background-neutral-primary text-foreground-neutral-primary px-2 py-1 rounded-full",
                                        "{item.tag}"
                                    }
                                }
                            }
                        }
                    }
                }

                // Right column (detail / composer)
                div { class: "flex-[2] min-w-0 border-l border-stroke-neutral-1 pl-6",
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
                                // Simple recipients input placeholder - comma-separated
                                Input {
                                    label: "",
                                    placeholder: Some("All".into()),
                                    value: new_recipients_display.clone(),
                                    height: crate::ui::foundation::components::InputHeight::Default,
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

                                        spawn({
                                            async move {
                                                // build recipients
                                                let recips = new_recipients_display
                                                    .read()
                                                    .split(',')
                                                    .map(|s| s.trim().to_string())
                                                    .filter(|s| !s.is_empty())
                                                    .collect::<Vec<String>>();

                                                // get current user
                                                if let Ok(Some(user_info)) = get_current_user().await {
                                                    if let Ok(user_id) = user_info.id.parse::<i32>() {
                                                        let req = CreateMessageRequest {
                                                            sender_user_id: user_id,
                                                            recipient_id: None,
                                                            recipient_type: None,
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
                                                                            title: m
                                                                                .content
                                                                                .split('.')
                                                                                .next()
                                                                                .unwrap_or("Announcement")
                                                                                .to_string(),
                                                                            sender: format!("User {}", m.sender_user_id),
                                                                            tag: "All".into(),
                                                                            time: m.created_at.to_string(),
                                                                            content: m.content.clone(),
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
                                    "{title_text.as_ref().unwrap()}"
                                }
                                p { class: "text-sm text-foreground-neutral-secondary mb-4",
                                    "{sender_time_text.as_ref().unwrap()}"
                                }
                                div { class: "mb-4",
                                    div { class: "inline-block bg-background-neutral-subtle-pressed text-foreground-neutral-primary px-3 py-1 rounded-full text-sm",
                                        "{tag_text.as_ref().unwrap()}"
                                    }
                                }
                                p { class: "text-foreground-neutral-primary",
                                    "{content_text.as_ref().unwrap()}"
                                }
                            }
                        } else {
                            div { class: "flex items-center justify-center h-full text-foreground-neutral-secondary",
                                "Select a message to view"
                            }
                        }
                    }
                }
            }
        }
    }
}
