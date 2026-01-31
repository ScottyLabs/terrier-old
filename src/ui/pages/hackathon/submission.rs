use dioxus::prelude::*;
use dioxus_free_icons::{
    Icon,
    icons::ld_icons::{LdCheck, LdChevronDown, LdChevronUp, LdFile, LdPencil, LdTrophy, LdUpload},
};

use crate::{
    auth::{SUBMISSION_ROLES, hooks::use_require_access_or_redirect},
    domain::{
        applications::types::{FieldType, FormSchema},
        hackathons::types::HackathonInfo,
        prizes::handlers::{PrizeInfo, get_prizes},
        submissions::handlers::{
            SubmitProjectRequest, UpdatePrizeTracksRequest, get_submission, submit_project,
            update_prize_tracks,
        },
    },
    ui::{
        features::prizes::PrizeCard,
        foundation::{
            components::{Button, ButtonSize, ButtonVariant, Input, InputHeight, InputVariant},
            modals::base::ModalBase,
        },
    },
};

#[component]
pub fn HackathonSubmission(slug: String) -> Element {
    if let Some(no_access) = use_require_access_or_redirect(SUBMISSION_ROLES) {
        return no_access;
    }

    let hackathon = use_context::<Signal<HackathonInfo>>();
    let mut selected_prize = use_signal(|| None::<PrizeInfo>);
    let mut show_submission_modal = use_signal(|| false);

    // State for submission data
    let mut submission_data = use_signal(|| std::collections::HashMap::<String, String>::new());
    let mut selected_prize_tracks = use_signal(|| std::collections::HashSet::<i32>::new());

    // Parse submission form config from hackathon
    let form_schema = use_memo(move || {
        hackathon
            .read()
            .submission_form
            .as_ref()
            .and_then(|config| serde_json::from_value::<FormSchema>(config.clone()).ok())
    });

    // Fetch existing submission from database
    let mut submission_resource = use_resource({
        let slug = slug.clone();
        move || {
            let slug = slug.clone();
            async move { get_submission(slug).await.ok().flatten() }
        }
    });

    // Sync submission data from resource when loaded
    use_effect(move || {
        if let Some(Some(sub)) = submission_resource.read().as_ref() {
            // Parse submission_data JSON to HashMap
            if let Some(obj) = sub.submission_data.as_object() {
                let data: std::collections::HashMap<String, String> = obj
                    .iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect();
                submission_data.set(data);
            }
            // Load prize tracks
            let tracks: std::collections::HashSet<i32> =
                sub.prize_track_ids.iter().copied().collect();
            selected_prize_tracks.set(tracks);
        }
    });

    // Derive has_submitted from loaded data
    let has_submitted = use_memo(move || {
        submission_resource
            .read()
            .as_ref()
            .map(|opt| opt.is_some())
            .unwrap_or(false)
    });

    // Fetch prizes
    let prizes_resource = use_resource({
        let slug = slug.clone();
        move || {
            let slug = slug.clone();
            async move { get_prizes(slug).await.ok() }
        }
    });

    // Timer signal that updates every second
    let mut now = use_signal(|| chrono::Utc::now().naive_utc());
    use_future(move || async move {
        loop {
            gloo_timers::future::sleep(std::time::Duration::from_secs(1)).await;
            now.set(chrono::Utc::now().naive_utc());
        }
    });

    // Calculate time remaining until hackathon end
    let time_remaining = use_memo(move || {
        let end_date = hackathon.read().end_date;
        let current_time = now();
        let duration = end_date.signed_duration_since(current_time);

        if duration.num_seconds() <= 0 {
            return "Submissions closed".to_string();
        }

        let days = duration.num_days();
        let hours = (duration.num_hours() % 24).abs();
        let minutes = (duration.num_minutes() % 60).abs();
        let seconds = (duration.num_seconds() % 60).abs();
        format!("{:02}d {:02}h {:02}m {:02}s", days, hours, minutes, seconds)
    });

    rsx! {
        div { class: "flex flex-col h-full",
            // Header
            div { class: "flex flex-col md:flex-row justify-between md:items-center gap-3 pt-6 md:pt-11 pb-4 md:pb-7",
                div { class: "flex items-center gap-4",
                    h1 { class: "text-2xl md:text-[30px] font-semibold leading-8 md:leading-[38px] text-foreground-neutral-primary",
                        "Project Submission"
                    }
                    // Countdown timer
                    div { class: "flex items-center gap-2 px-3 py-1.5 bg-background-neutral-primary rounded-lg",
                        Icon {
                            width: 16,
                            height: 16,
                            icon: LdFile,
                            class: "text-foreground-neutral-secondary inline-block",
                        }
                        span { class: "text-sm font-mono text-foreground-neutral-primary",
                            "{time_remaining}"
                        }
                    }
                }
                if form_schema().is_some() && !has_submitted() {
                    Button {
                        size: ButtonSize::Compact,
                        onclick: move |_| show_submission_modal.set(true),
                        Icon {
                            width: 16,
                            height: 16,
                            icon: LdUpload,
                            class: "text-white mr-1 inline-block",
                        }
                        "Submit Now"
                    }
                }
            }

            // Main content
            div { class: "flex-1 overflow-y-auto",
                if has_submitted() {
                    // Post-submission view
                    SubmittedView {
                        schema: form_schema()
                            .unwrap_or_else(|| FormSchema {
                                title: "Project Submission".to_string(),
                                description: None,
                                version: "1.0".to_string(),
                                fields: vec![],
                            }),
                        submission_data: submission_data(),
                        prizes: prizes_resource.read().clone().flatten().unwrap_or_default(),
                        selected_prize_tracks,
                        hackathon_slug: slug.clone(),
                        submitted_at: submission_resource
                            .read()
                            .as_ref()
                            .and_then(|opt| opt.as_ref())
                            .map(|s| s.submitted_at.clone())
                            .unwrap_or_default(),
                        table_number: submission_resource
                            .read()
                            .as_ref()
                            .and_then(|opt| opt.as_ref())
                            .and_then(|s| s.table_number.clone()),
                        on_edit: move |_| show_submission_modal.set(true),
                    }
                } else {
                    // Pre-submission view - show prizes
                    div {
                        h2 { class: "text-lg font-semibold text-foreground-neutral-primary mb-4",
                            "Prizes"
                        }

                        match prizes_resource.read().as_ref() {
                            Some(Some(prizes)) if !prizes.is_empty() => rsx! {
                                div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4",
                                    for prize in prizes.iter() {
                                        {
                                            let prize_clone = prize.clone();
                                            rsx! {
                                                PrizeCard {
                                                    key: "{prize.id}",
                                                    prize: prize.clone(),
                                                    on_click: move |_| selected_prize.set(Some(prize_clone.clone())),
                                                }
                                            }
                                        }
                                    }
                                }
                            },
                            Some(Some(_)) => rsx! {
                                div { class: "bg-background-neutral-primary rounded-2xl p-6 text-center",
                                    p { class: "text-foreground-neutral-secondary", "No prizes have been announced yet." }
                                }
                            },
                            Some(None) => rsx! {
                                div { class: "bg-background-neutral-primary rounded-2xl p-6 text-center",
                                    p { class: "text-status-danger-foreground", "Failed to load prizes." }
                                }
                            },
                            None => rsx! {
                                div { class: "bg-background-neutral-primary rounded-2xl p-6 text-center",
                                    p { class: "text-foreground-neutral-secondary", "Loading prizes..." }
                                }
                            },
                        }
                    }
                }
            }
        }

        // Submission form modal
        if show_submission_modal() {
            if let Some(schema) = form_schema() {
                SubmissionFormModal {
                    schema,
                    hackathon_slug: slug.clone(),
                    initial_values: if has_submitted() { Some(submission_data()) } else { None },
                    initial_table_number: submission_resource.read().as_ref().and_then(|s| s.as_ref()).and_then(|s| s.table_number.clone()),
                    on_close: move |_| show_submission_modal.set(false),
                    on_submit: move |(data, table_num): (std::collections::HashMap<String, String>, Option<String>)| {
                        submission_data.set(data);
                        // Refetch submission data to update has_submitted state
                        submission_resource.restart();
                        show_submission_modal.set(false);
                    },
                }
            }
        }


        // Prize detail modal (read-only for participants)
        if let Some(prize) = selected_prize() {
            ModalBase {
                on_close: move |_| selected_prize.set(None),
                width: "500px",
                max_height: "90vh",

                div { class: "p-7",
                    if let Some(img_url) = &prize.image_url {
                        div { class: "mb-4 rounded-lg overflow-hidden",
                            img {
                                src: "{img_url}",
                                alt: "{prize.name}",
                                class: "w-full h-48 object-cover",
                            }
                        }
                    }

                    h2 { class: "text-2xl font-semibold text-foreground-neutral-primary mb-2",
                        "{prize.name}"
                    }

                    if let Some(cat) = &prize.category {
                        span { class: "inline-block px-3 py-1 bg-background-neutral-secondary text-foreground-neutral-secondary text-sm rounded-full mb-4",
                            "{cat}"
                        }
                    }

                    div { class: "mb-4",
                        p { class: "text-lg font-medium text-foreground-brand-primary",
                            "{prize.value}"
                        }
                    }

                    if let Some(desc) = &prize.description {
                        p { class: "text-foreground-neutral-secondary mb-6", "{desc}" }
                    }

                    div { class: "flex gap-3 justify-end",
                        Button {
                            variant: ButtonVariant::Default,
                            onclick: move |_| selected_prize.set(None),
                            "Close"
                        }
                    }
                }
            }
        }
    }
}

/// View shown after project has been submitted
#[component]
fn SubmittedView(
    schema: FormSchema,
    submission_data: std::collections::HashMap<String, String>,
    prizes: Vec<PrizeInfo>,
    selected_prize_tracks: Signal<std::collections::HashSet<i32>>,
    hackathon_slug: String,
    submitted_at: String,
    table_number: Option<String>,
    on_edit: EventHandler<()>,
) -> Element {
    // Format the submitted_at timestamp for display
    let formatted_time = use_memo(move || {
        chrono::NaiveDateTime::parse_from_str(&submitted_at, "%Y-%m-%d %H:%M:%S%.f")
            .or_else(|_| chrono::NaiveDateTime::parse_from_str(&submitted_at, "%Y-%m-%d %H:%M:%S"))
            .map(|dt| {
                let now = chrono::Utc::now().naive_utc();
                let diff = now.signed_duration_since(dt);
                if diff.num_minutes() < 1 {
                    "Submitted just now".to_string()
                } else if diff.num_minutes() < 60 {
                    format!("Submitted {} min ago", diff.num_minutes())
                } else if diff.num_hours() < 24 {
                    format!("Submitted {} hr ago", diff.num_hours())
                } else {
                    format!("Submitted {}", dt.format("%b %d, %Y at %H:%M"))
                }
            })
            .unwrap_or_else(|_| "Submitted".to_string())
    });
    rsx! {
        div { class: "flex flex-col gap-8",
            // Enter for Prizes section
            div { class: "bg-background-neutral-primary rounded-2xl p-6",
                div { class: "flex items-center gap-2 mb-4",
                    Icon {
                        width: 20,
                        height: 20,
                        icon: LdTrophy,
                        class: "text-foreground-brand-primary inline-block",
                    }
                    h2 { class: "text-lg font-semibold text-foreground-neutral-primary",
                        "Enter for Prizes"
                    }
                }

                div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3",
                    for prize in prizes.iter() {
                        PrizeTrackSelector {
                            prize: prize.clone(),
                            is_selected: selected_prize_tracks.read().contains(&prize.id),
                            on_toggle: {
                                let prize_id = prize.id;
                                let slug = hackathon_slug.clone();
                                move |_| {
                                    let slug = slug.clone();
                                    // Update local state immediately for responsiveness
                                    let new_tracks: Vec<i32> = {
                                        let mut tracks = selected_prize_tracks.write();
                                        if tracks.contains(&prize_id) {
                                            tracks.remove(&prize_id);
                                        } else {
                                            tracks.insert(prize_id);
                                        }
                                        tracks.iter().copied().collect()
                                    };
                                    // Persist to database
                                    spawn(async move {
                                        let request = UpdatePrizeTracksRequest {
                                            prize_track_ids: new_tracks,
                                        };
                                        let _ = update_prize_tracks(slug, request).await;
                                    });
                                }
                            },
                        }
                    }
                }
            }

            // Your Submission section
            div { class: "bg-background-neutral-primary rounded-2xl p-6",
                div { class: "flex items-center justify-between mb-6",
                    div { class: "flex items-center gap-2",
                        Icon {
                            width: 20,
                            height: 20,
                            icon: LdFile,
                            class: "text-foreground-brand-primary inline-block",
                        }
                        h2 { class: "text-lg font-semibold text-foreground-neutral-primary",
                            "Your Submission"
                        }
                        if let Some(table) = table_number {
                            span { class: "px-2 py-0.5 bg-background-brand-primary text-white text-xs font-bold rounded",
                                "Table {table}"
                            }
                        }
                        span { class: "text-sm text-foreground-neutral-secondary ml-2",
                            "{formatted_time}"
                        }
                    }
                    Button {
                        variant: ButtonVariant::Default,
                        size: ButtonSize::Compact,
                        onclick: move |_| on_edit.call(()),
                        Icon {
                            width: 14,
                            height: 14,
                            icon: LdPencil,
                            class: "text-white mr-1 inline-block",
                        }
                        "Edit"
                    }
                }

                // Display submitted fields
                div { class: "flex flex-col gap-6",
                    for field in schema.fields.iter() {
                        {
                            let value = submission_data.get(&field.name).cloned().unwrap_or_default();
                            rsx! {
                                SubmissionFieldDisplay {
                                    label: field.label.clone(),
                                    value,
                                    field_type: field.field_type.clone(),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Selectable prize track with expandable description
#[component]
fn PrizeTrackSelector(prize: PrizeInfo, is_selected: bool, on_toggle: EventHandler<()>) -> Element {
    let mut is_expanded = use_signal(|| false);

    rsx! {
        div { class: "rounded-lg overflow-hidden",
            // Header row
            div {
                class: "flex items-center gap-3 p-3 cursor-pointer bg-background-neutral-secondary hover:bg-background-neutral-subtle-pressed transition-colors",
                onclick: move |_| on_toggle.call(()),

                // Checkbox
                div {
                    class: format!(
                        "w-5 h-5 rounded border-2 flex items-center justify-center transition-colors {}",
                        if is_selected {
                            "bg-black border-background-brand-primary"
                        } else {
                            "border-border-neutral-primary bg-background-neutral-primary"
                        },
                    ),
                    if is_selected {
                        Icon {
                            width: 14,
                            height: 14,
                            icon: LdCheck,
                            class: "text-white",
                        }
                    }
                }

                // Prize name
                span { class: "flex-1 text-sm font-medium text-foreground-neutral-primary truncate",
                    "{prize.name}"
                }

                // Expand button
                button {
                    class: "p-1 hover:bg-background-neutral-tertiary rounded transition-colors",
                    r#type: "button",
                    onclick: move |evt| {
                        evt.stop_propagation();
                        is_expanded.set(!is_expanded());
                    },
                    if is_expanded() {
                        Icon {
                            width: 16,
                            height: 16,
                            icon: LdChevronUp,
                            class: "text-foreground-neutral-secondary",
                        }
                    } else {
                        Icon {
                            width: 16,
                            height: 16,
                            icon: LdChevronDown,
                            class: "text-foreground-neutral-secondary",
                        }
                    }
                }
            }

            // Expandable description
            if is_expanded() {
                div { class: "px-3 pb-3 pt-0 bg-background-neutral-secondary rounded-b-lg",
                    if let Some(desc) = &prize.description {
                        p { class: "text-sm text-foreground-neutral-secondary pt-3",
                            "{desc}"
                        }
                    }
                    if let Some(cat) = &prize.category {
                        p { class: "text-xs text-foreground-neutral-tertiary pt-2",
                            "Category: {cat}"
                        }
                    }
                    p { class: "text-sm font-medium text-foreground-brand-primary pt-2",
                        "{prize.value}"
                    }
                }
            }
        }
    }
}

/// Read-only display of a submission field
#[component]
fn SubmissionFieldDisplay(label: String, value: String, field_type: FieldType) -> Element {
    rsx! {
        div { class: "flex flex-col gap-2",
            label { class: "text-sm font-medium text-foreground-neutral-secondary", "{label}" }
            match field_type {
                FieldType::Textarea { .. } => rsx! {
                    div { class: "bg-background-neutral-secondary rounded-lg p-4 min-h-[80px]",
                        p { class: "text-sm text-foreground-neutral-primary whitespace-pre-wrap",
                            if value.is_empty() {
                                span { class: "text-foreground-neutral-tertiary italic", "No content" }
                            } else {
                                "{value}"
                            }
                        }
                    }
                },
                FieldType::Url { .. } => rsx! {
                    div { class: "bg-background-neutral-secondary rounded-lg p-3",
                        if value.is_empty() {
                            span { class: "text-sm text-foreground-neutral-tertiary italic", "Not provided" }
                        } else {
                            a {
                                href: "{value}",
                                target: "_blank",
                                class: "text-sm text-foreground-brand-primary hover:underline",
                                "{value}"
                            }
                        }
                    }
                },
                FieldType::File { .. } => rsx! {
                    if value.is_empty() {
                        Button {
                            variant: ButtonVariant::Default,
                            size: ButtonSize::Compact,
                            disabled: true,
                            Icon {
                                width: 14,
                                height: 14,
                                icon: LdUpload,
                                class: "text-foreground-neutral-secondary mr-1 inline-block",
                            }
                            "Upload"
                        }
                    } else {
                        a {
                            href: "{value}",
                            target: "_blank",
                            class: "inline-flex items-center gap-2 px-3 py-1.5 bg-background-neutral-secondary rounded-lg text-sm text-foreground-brand-primary hover:underline",
                            Icon {
                                width: 14,
                                height: 14,
                                icon: LdFile,
                                class: "inline-block",
                            }
                            "View file"
                        }
                    }
                },
                _ => rsx! {
                    div { class: "bg-background-neutral-secondary rounded-lg p-3",
                        p { class: "text-sm text-foreground-neutral-primary",
                            if value.is_empty() {
                                span { class: "text-foreground-neutral-tertiary italic", "Not provided" }
                            } else {
                                "{value}"
                            }
                        }
                    }
                },
            }
        }
    }
}

#[component]
fn SubmissionFormModal(
    schema: FormSchema,
    hackathon_slug: String,
    initial_values: Option<std::collections::HashMap<String, String>>,
    initial_table_number: Option<String>,
    on_close: EventHandler<()>,
    on_submit: EventHandler<(std::collections::HashMap<String, String>, Option<String>)>,
) -> Element {
    let mut form_values = use_signal(|| initial_values.clone().unwrap_or_default());
    let mut table_number = use_signal(|| initial_table_number.clone().unwrap_or_default());
    let mut is_submitting = use_signal(|| false);
    let mut error_message = use_signal(|| None::<String>);

    // Group fields by section
    let sections = use_memo(move || {
        let mut grouped: std::collections::HashMap<
            String,
            Vec<crate::domain::applications::types::FormField>,
        > = std::collections::HashMap::new();

        for field in schema.fields.iter() {
            let section_name = field
                .section
                .clone()
                .unwrap_or_else(|| "Project Details".to_string());
            grouped.entry(section_name).or_default().push(field.clone());
        }

        let mut sections_vec: Vec<(String, Vec<crate::domain::applications::types::FormField>)> =
            grouped.into_iter().collect();
        sections_vec.sort_by_key(|(_, fields)| fields.iter().map(|f| f.order).min().unwrap_or(0));

        sections_vec
    });

    let handle_submit = {
        let hackathon_slug = hackathon_slug.clone();
        move |evt: Event<FormData>| {
            evt.prevent_default();
            let slug = hackathon_slug.clone();
            let data = form_values();
            spawn(async move {
                is_submitting.set(true);
                error_message.set(None);

                // Convert HashMap<String, String> to serde_json::Value
                let submission_json = serde_json::to_value(&data).unwrap_or_default();

                let request = SubmitProjectRequest {
                    submission_data: submission_json,
                    table_number: if table_number().is_empty() {
                        None
                    } else {
                        Some(table_number())
                    },
                    prize_track_ids: vec![], // Prize tracks are selected separately after submission
                };

                match submit_project(slug, request).await {
                    Ok(_) => {
                        is_submitting.set(false);
                        on_submit.call((
                            data,
                            if table_number().is_empty() {
                                None
                            } else {
                                Some(table_number())
                            },
                        ));
                    }
                    Err(e) => {
                        is_submitting.set(false);
                        error_message.set(Some(format!("Failed to submit: {}", e)));
                    }
                }
            });
        }
    };

    rsx! {
        ModalBase {
            on_close: move |_| on_close.call(()),
            width: "700px",
            max_height: "95vh",

            div { class: "flex flex-col p-0 m-0 max-h-[85vh]",
                // Header (fixed)
                div { class: "p-7 pb-4 shrink-0",
                    h2 { class: "text-2xl font-semibold text-foreground-neutral-primary",
                        "{schema.title}"
                    }
                    if let Some(desc) = &schema.description {
                        p { class: "text-foreground-neutral-secondary mt-2", "{desc}" }
                    }
                }

                // Scrollable form content
                div { class: "flex-1 overflow-y-scroll px-7",
                    if let Some(error) = error_message() {
                        div { class: "mb-4 p-4 bg-status-danger-background text-status-danger-foreground rounded-lg",
                            "{error}"
                        }
                    }

                    form { class: "flex flex-col gap-6", onsubmit: handle_submit,
                        // Table Assignment section
                        div { class: "bg-background-neutral-primary rounded-lg p-6",
                            h3 { class: "text-lg font-semibold text-foreground-neutral-primary mb-4",
                                "Table Assignment"
                            }
                            Input {
                                label: "Table Number".to_string(),
                                placeholder: "e.g. A12".to_string(),
                                value: table_number,
                                variant: InputVariant::Primary,
                                help_text: Some("Optionally provide your team's table number if you have one.".to_string()),
                                oninput: move |evt: Event<FormData>| table_number.set(evt.value()),
                            }
                        }

                        for (section_name , fields) in sections().iter() {
                            div { class: "bg-background-neutral-primary rounded-lg p-6",
                                h3 { class: "text-lg font-semibold text-foreground-neutral-primary mb-4",
                                    "{section_name}"
                                }
                                div { class: "flex flex-col gap-4",
                                    for field in fields.iter() {
                                        SubmissionFieldRenderer {
                                            field: field.clone(),
                                            form_values,
                                            hackathon_slug: hackathon_slug.clone(),
                                        }
                                    }
                                }
                            }
                        }

                        // Buttons (inside form)
                        div { class: "flex gap-3 justify-end py-4",
                            Button {
                                variant: ButtonVariant::Tertiary,
                                button_type: "button".to_string(),
                                onclick: move |_| on_close.call(()),
                                "Cancel"
                            }
                            Button {
                                button_type: "submit".to_string(),
                                disabled: is_submitting(),
                                if is_submitting() {
                                    "Submitting..."
                                } else {
                                    "Submit Project"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SubmissionFieldRenderer(
    field: crate::domain::applications::types::FormField,
    form_values: Signal<std::collections::HashMap<String, String>>,
    hackathon_slug: String,
) -> Element {
    let field_id = field.id.clone();
    let field_name = field.name.clone();
    let field_label = field.label.clone();
    let field_type = field.field_type.clone();
    let field_required = field.required;
    let field_help_text = field.help_text.clone();
    let field_default = field.default_value.clone().unwrap_or_default();

    let field_name_for_handlers = field_name.clone();
    let field_name_for_sync = field_name.clone();

    let initial_value = form_values
        .peek()
        .get(&field_name)
        .cloned()
        .unwrap_or(field_default.clone());
    let mut value = use_signal(|| initial_value);

    // Sync value signal when form_values changes
    use_effect(move || {
        let current_form_value = form_values.read().get(&field_name_for_sync).cloned();
        if let Some(new_value) = current_form_value
            && new_value != *value.peek()
        {
            value.set(new_value);
        }
    });

    rsx! {
        div { class: "flex flex-col gap-2 bg-background-neutral-primary",
            match field_type.clone() {
                // For submission forms, file fields are URL fields
                // Fallback for unsupported field types
                FieldType::Text { placeholder, .. } | FieldType::Url { placeholder } => {
                    let input_type = match field_type {
                        FieldType::Url { .. } => "url",
                        _ => "text",
                    };
                    rsx! {
                        Input {
                            label: field_label,
                            placeholder,
                            value,
                            variant: InputVariant::Primary,
                            input_type: input_type.to_string(),
                            name: Some(field_name),
                            id: Some(field_id),
                            required: field_required,
                            help_text: field_help_text.clone(),
                            oninput: move |evt: Event<FormData>| {
                                let new_value = evt.value();
                                {
                                    let mut values = form_values.write();
                                    if !new_value.is_empty() {
                                        values.insert(field_name_for_handlers.clone(), new_value.clone());
                                    } else {
                                        values.remove(&field_name_for_handlers);
                                    }
                                }
                            },
                        }
                    }
                }
                FieldType::Textarea { placeholder } => {
                    rsx! {
                        Input {
                            label: field_label,
                            placeholder,
                            value,
                            height: InputHeight::Tall,
                            variant: InputVariant::Primary,
                            name: Some(field_name),
                            id: Some(field_id),
                            required: field_required,
                            help_text: field_help_text.clone(),
                            oninput: move |evt: Event<FormData>| {
                                let new_value = evt.value();
                                {
                                    let mut values = form_values.write();
                                    if !new_value.is_empty() {
                                        values.insert(field_name_for_handlers.clone(), new_value.clone());
                                    } else {
                                        values.remove(&field_name_for_handlers);
                                    }
                                }
                            },
                        }
                    }
                }
                FieldType::File { .. } => {
                    rsx! {
                        Input {
                            label: field_label,
                            placeholder: Some("https://...".to_string()),
                            value,
                            variant: InputVariant::Primary,
                            input_type: "url".to_string(),
                            name: Some(field_name),
                            id: Some(field_id),
                            required: field_required,
                            help_text: field_help_text.clone(),
                            oninput: move |evt: Event<FormData>| {
                                let new_value = evt.value();
                                {
                                    let mut values = form_values.write();
                                    if !new_value.is_empty() {
                                        values.insert(field_name_for_handlers.clone(), new_value.clone());
                                    } else {
                                        values.remove(&field_name_for_handlers);
                                    }
                                }
                            },
                        }
                    }
                }
                _ => {
                    rsx! {
                        Input {
                            label: field_label,
                            value,
                            variant: InputVariant::Primary,
                            input_type: "text".to_string(),
                            name: Some(field_name),
                            id: Some(field_id),
                            required: field_required,
                            help_text: field_help_text.clone(),
                            oninput: move |evt: Event<FormData>| {
                                let new_value = evt.value();
                                {
                                    let mut values = form_values.write();
                                    if !new_value.is_empty() {
                                        values.insert(field_name_for_handlers.clone(), new_value.clone());
                                    } else {
                                        values.remove(&field_name_for_handlers);
                                    }
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}
