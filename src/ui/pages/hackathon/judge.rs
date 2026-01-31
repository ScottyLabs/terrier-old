use dioxus::prelude::*;

use crate::{
    auth::{JUDGE_ROLES, hooks::use_require_access_or_redirect},
    domain::{
        hackathons::types::HackathonInfo,
        judging::{
            handlers::{
                complete_visit, get_unified_state, request_next_project, submit_comparisons,
            },
            types::{
                CompleteVisitRequest, CurrentProject, FeatureComparison, JudgeFeatureState,
                PrizeInfo, SubmitComparisonsRequest, UnifiedJudgingState,
            },
        },
    },
    ui::foundation::components::{Button, ButtonVariant},
};

const PRIZE_COLORS: &[&str] = &[
    "bg-blue-50",
    "bg-purple-50",
    "bg-amber-50",
    "bg-orange-50",
    "bg-green-50",
    "bg-pink-50",
    "bg-indigo-50",
];

#[component]
pub fn HackathonJudge(slug: String) -> Element {
    if let Some(no_access) = use_require_access_or_redirect(JUDGE_ROLES) {
        return no_access;
    }

    let hackathon = use_context::<Signal<HackathonInfo>>();
    let mut state: Signal<Option<UnifiedJudgingState>> = use_signal(|| None);
    let mut loading = use_signal(|| false);
    let mut error_msg: Signal<Option<String>> = use_signal(|| None);
    let mut success_msg: Signal<Option<String>> = use_signal(|| None);

    // Track selected winners for each feature
    let mut selections: Signal<std::collections::HashMap<i32, i32>> =
        use_signal(|| std::collections::HashMap::new());
    // Track notes for each feature
    let mut feature_notes: Signal<std::collections::HashMap<i32, String>> =
        use_signal(|| std::collections::HashMap::new());
    // Track which prize cards are expanded
    let mut expanded_cards: Signal<std::collections::HashSet<i32>> =
        use_signal(|| std::collections::HashSet::new());
    // Track project-level notes
    let mut project_notes: Signal<String> = use_signal(|| String::new());

    // Fetch unified state on mount
    let slug_clone = slug.clone();
    use_effect(move || {
        let slug = slug_clone.clone();
        spawn(async move {
            loading.set(true);
            match get_unified_state(slug).await {
                Ok(s) => {
                    // Initialize selections from current best
                    let mut initial_selections = std::collections::HashMap::new();
                    for feat in &s.features {
                        if let Some(best_id) = feat.current_best_submission_id {
                            initial_selections.insert(feat.feature_id, best_id);
                        }
                    }
                    selections.set(initial_selections);

                    // Initialize notes from state
                    let mut initial_notes = std::collections::HashMap::new();
                    for feat in &s.features {
                        if let Some(notes) = &feat.notes {
                            initial_notes.insert(feat.feature_id, notes.clone());
                        }
                    }
                    feature_notes.set(initial_notes);

                    state.set(Some(s));
                }
                Err(e) => {
                    error_msg.set(Some(format!("Failed to load judging state: {}", e)));
                }
            }
            loading.set(false);
        });
    });

    let start_judging = {
        let slug = slug.clone();
        move |_| {
            let slug = slug.clone();
            spawn(async move {
                loading.set(true);
                error_msg.set(None);
                success_msg.set(None);

                match request_next_project(slug.clone()).await {
                    Ok(Some(_project)) => {
                        // Refresh state to get the new project
                        if let Ok(new_state) = get_unified_state(slug).await {
                            state.set(Some(new_state));
                        }
                    }
                    Ok(None) => {
                        error_msg.set(Some(
                            "No projects available to judge right now.".to_string(),
                        ));
                    }
                    Err(e) => {
                        error_msg.set(Some(format!("Failed to start judging: {}", e)));
                    }
                }

                loading.set(false);
            });
        }
    };

    let submit_all = {
        let slug = slug.clone();
        move |_| {
            let slug = slug.clone();
            let current_state = state.read().clone();
            let current_selections = selections.read().clone();
            let current_notes = feature_notes.read().clone();
            let current_project_notes = project_notes.read().clone();

            if let Some(s) = current_state {
                if let Some(project) = s.current_project {
                    spawn(async move {
                        loading.set(true);
                        error_msg.set(None);

                        let mut comparisons = Vec::new();
                        for feat in &s.features {
                            if let Some(&winner_id) = current_selections.get(&feat.feature_id) {
                                comparisons.push(FeatureComparison {
                                    feature_id: feat.feature_id,
                                    winner_submission_id: winner_id,
                                    notes: current_notes.get(&feat.feature_id).cloned(),
                                });
                            }
                        }

                        let request = SubmitComparisonsRequest {
                            visit_id: project.visit_id,
                            comparisons,
                            notes: if current_project_notes.trim().is_empty() {
                                None
                            } else {
                                Some(current_project_notes)
                            },
                        };

                        match submit_comparisons(slug.clone(), request).await {
                            Ok(()) => {
                                success_msg
                                    .set(Some("Submitted! Getting next project...".to_string()));

                                // Clear selections and notes for next project
                                selections.set(std::collections::HashMap::new());
                                feature_notes.set(std::collections::HashMap::new());
                                project_notes.set(String::new());

                                // Get next project
                                match request_next_project(slug.clone()).await {
                                    Ok(Some(_)) => {
                                        if let Ok(new_state) = get_unified_state(slug).await {
                                            // Re-initialize selections from current best
                                            let mut new_selections =
                                                std::collections::HashMap::new();
                                            for feat in &new_state.features {
                                                if let Some(best_id) =
                                                    feat.current_best_submission_id
                                                {
                                                    new_selections.insert(feat.feature_id, best_id);
                                                }
                                            }
                                            selections.set(new_selections);
                                            state.set(Some(new_state));
                                        }
                                        success_msg.set(None);
                                    }
                                    Ok(None) => {
                                        success_msg.set(Some(
                                            "All done! No more projects to judge.".to_string(),
                                        ));
                                        if let Ok(new_state) = get_unified_state(slug).await {
                                            state.set(Some(new_state));
                                        }
                                    }
                                    Err(e) => {
                                        error_msg.set(Some(format!(
                                            "Submitted but failed to get next: {}",
                                            e
                                        )));
                                    }
                                }
                            }
                            Err(e) => {
                                error_msg.set(Some(format!("Failed to submit: {}", e)));
                            }
                        }

                        loading.set(false);
                    });
                }
            }
        }
    };

    let skip_project = {
        let slug = slug.clone();
        move |_| {
            let slug = slug.clone();
            let current_state = state.read().clone();

            spawn(async move {
                let confirmed = web_sys::window()
                    .and_then(|w| w.confirm_with_message("Are you sure you want to skip this project? It will be marked as absent/skipped.").ok())
                    .unwrap_or(false);

                if !confirmed {
                    return;
                }

                if let Some(s) = current_state {
                    if let Some(project) = s.current_project {
                        loading.set(true);
                        error_msg.set(None);

                        let request = CompleteVisitRequest {
                            notes: Some("Skipped / Absent".to_string()),
                        };

                        match complete_visit(slug.clone(), project.visit_id, request).await {
                            Ok(()) => {
                                success_msg
                                    .set(Some("Skipped project. Getting next...".to_string()));

                                // Clear selections and notes for next project
                                selections.set(std::collections::HashMap::new());
                                feature_notes.set(std::collections::HashMap::new());
                                project_notes.set(String::new());

                                // Get next project
                                match request_next_project(slug.clone()).await {
                                    Ok(Some(_)) => {
                                        if let Ok(new_state) = get_unified_state(slug).await {
                                            // Re-initialize selections from current best
                                            let mut new_selections =
                                                std::collections::HashMap::new();
                                            for feat in &new_state.features {
                                                if let Some(best_id) =
                                                    feat.current_best_submission_id
                                                {
                                                    new_selections.insert(feat.feature_id, best_id);
                                                }
                                            }
                                            selections.set(new_selections);
                                            state.set(Some(new_state));
                                        }
                                        success_msg.set(None);
                                    }
                                    Ok(None) => {
                                        success_msg.set(Some(
                                            "All done! No more projects to judge.".to_string(),
                                        ));
                                        if let Ok(new_state) = get_unified_state(slug).await {
                                            state.set(Some(new_state));
                                        }
                                    }
                                    Err(e) => {
                                        error_msg.set(Some(format!(
                                            "Skipped but failed to get next: {}",
                                            e
                                        )));
                                    }
                                }
                            }
                            Err(e) => {
                                error_msg.set(Some(format!("Failed to skip project: {}", e)));
                            }
                        }

                        loading.set(false);
                    }
                }
            });
        }
    };

    let hackathon_info = hackathon.read();
    let current_state = state.read().clone();

    rsx! {
        div { class: "pt-6 pb-7",
            // Check if judging is active
            if !hackathon_info.judging_started {
                div { class: "p-6 bg-background-warning-secondary rounded-lg",
                    p { class: "text-foreground-warning-primary font-medium",
                        "Judging has not started yet. Please wait for the organizers to begin the judging period."
                    }
                }
            } else if *loading.read() && current_state.is_none() {
                div { class: "flex items-center justify-center py-12",
                    p { class: "text-foreground-neutral-secondary", "Loading..." }
                }
            } else if let Some(ref s) = current_state {
                // Flash messages
                if let Some(err) = error_msg.read().as_ref() {
                    div { class: "mb-4 p-4 bg-background-danger-secondary rounded-lg",
                        p { class: "text-foreground-danger-primary", "{err}" }
                    }
                }
                if let Some(msg) = success_msg.read().as_ref() {
                    div { class: "mb-4 p-4 bg-background-success-secondary rounded-lg",
                        p { class: "text-foreground-success-primary", "{msg}" }
                    }
                }

                if s.assigned_prizes.is_empty() {
                    // No prizes assigned
                    div { class: "p-6 bg-background-neutral-tertiary-enabled rounded-lg text-center",
                        p { class: "text-foreground-neutral-secondary",
                            "You haven't been assigned to any prizes yet. Please contact an organizer."
                        }
                    }
                } else if s.current_project.is_none() {
                    // Pre-judging state - show assigned prizes
                    PreJudgingView {
                        prizes: s.assigned_prizes.clone(),
                        loading: *loading.read(),
                        on_start: start_judging,
                    }
                } else if let Some(ref project) = s.current_project {
                    // In-progress state
                    InProgressView {
                        project: project.clone(),
                        features: s.features.clone(),
                        selections: selections.clone(),
                        feature_notes: feature_notes.clone(),
                        expanded_cards: expanded_cards.clone(),
                        project_notes: project_notes.clone(),
                        loading: *loading.read(),
                        on_submit: submit_all,
                        on_skip: skip_project,
                    }
                }
            } else {
                div { class: "p-6 bg-background-neutral-tertiary-enabled rounded-lg text-center",
                    p { class: "text-foreground-neutral-secondary",
                        "Failed to load judging state. Please refresh the page."
                    }
                }
            }
        }
    }
}

/// Pre-judging view showing assigned prizes
#[component]
fn PreJudgingView(prizes: Vec<PrizeInfo>, loading: bool, on_start: EventHandler<()>) -> Element {
    rsx! {
        div { class: "max-w-6xl mx-auto",
            h1 { class: "text-2xl font-semibold text-foreground-neutral-primary mb-6",
                "Judging"
            }

            p { class: "text-foreground-neutral-secondary mb-6",
                "You are judging the following prizes:"
            }

            div { class: "grid grid-cols-1 md:grid-cols-3 gap-4 mb-8",
                for (i, prize) in prizes.iter().enumerate() {
                    div {
                        key: "{prize.id}",
                        class: "p-4 {PRIZE_COLORS[i % PRIZE_COLORS.len()]} rounded-lg flex items-center justify-between",
                        div {
                            p { class: "font-medium text-foreground-neutral-primary",
                                "{prize.name}"
                            }
                            if let Some(desc) = &prize.description {
                                p { class: "text-sm text-foreground-neutral-secondary mt-1",
                                    "{desc}"
                                }
                            }
                        }
                        // Checkbox icon
                        svg {
                            class: "w-5 h-5 text-foreground-neutral-secondary",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z",
                            }
                        }
                    }
                }
            }

            div { class: "flex justify-center",
                Button { disabled: loading, onclick: move |_| on_start.call(()),
                    if loading {
                        "Starting..."
                    } else {
                        "Start Judging"
                    }
                }
            }
        }
    }
}

/// In-progress judging view
#[component]
fn InProgressView(
    project: CurrentProject,
    features: Vec<JudgeFeatureState>,
    mut selections: Signal<std::collections::HashMap<i32, i32>>,
    mut feature_notes: Signal<std::collections::HashMap<i32, String>>,
    mut expanded_cards: Signal<std::collections::HashSet<i32>>,
    mut project_notes: Signal<String>,
    loading: bool,
    on_submit: EventHandler<()>,
    on_skip: EventHandler<()>,
) -> Element {
    let project_name = project
        .project_name
        .clone()
        .unwrap_or_else(|| project.team_name.clone());
    let description = project.description.clone().unwrap_or_default();
    let location = project
        .table_number
        .clone()
        .or_else(|| project.location.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let mut show_json = use_signal(|| false);

    rsx! {
        div {
            h1 { class: "text-2xl font-semibold text-foreground-neutral-primary mb-6",
                "Judging Projects Now"
            }

            // Project header
            div { class: "bg-background-neutral-primary p-6 rounded-lg mb-6",
                div { class: "flex flex-wrap gap-x-8 gap-y-2 mb-4",
                    div {
                        span { class: "text-foreground-neutral-secondary", "Project: " }
                        span { class: "font-medium text-foreground-neutral-primary",
                            "{project_name}"
                        }
                    }
                    div {
                        span { class: "text-foreground-neutral-secondary", "Team: " }
                        span { class: "font-medium text-foreground-neutral-primary",
                            "{project.team_name}"
                        }
                    }
                    div {
                        span { class: "text-foreground-neutral-secondary", "Location: " }
                        span { class: "font-medium text-foreground-neutral-primary",
                            "{location}"
                        }
                    }
                }

                div {
                    div { class: "flex justify-between items-start mb-2",
                        h3 { class: "font-medium text-foreground-neutral-primary",
                            "Description:"
                        }
                        button {
                            class: "text-xs text-foreground-brand-primary hover:underline cursor-pointer",
                            onclick: move |_| show_json.toggle(),
                            if *show_json.read() { "Hide Data" } else { "Show Data" }
                        }
                    }
                    p { class: "text-foreground-neutral-secondary mb-4", "{description}" }

                    if *show_json.read() {
                        div { class: "mb-4 p-4 bg-background-neutral-secondary-enabled rounded-lg overflow-x-auto",
                            pre { class: "text-xs text-foreground-neutral-primary font-mono",
                                "{serde_json::to_string_pretty(&project.submission_data).unwrap_or_default()}"
                            }
                        }
                    }
                }
            }

            // Prize cards
            div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 mb-8",
                for (i, feature) in features.iter().enumerate() {
                    PrizeCard {
                        key: "{feature.feature_id}",
                        feature: feature.clone(),
                        current_project: project.clone(),
                        header_bg_color: PRIZE_COLORS[i % PRIZE_COLORS.len()].to_string(),
                        selections: selections.clone(),
                        feature_notes: feature_notes.clone(),
                        expanded_cards: expanded_cards.clone(),
                    }
                }
            }

            // General project notes
            div { class: "mb-8 bg-background-neutral-primary p-6 rounded-lg",
                h3 { class: "font-medium text-foreground-neutral-primary mb-2", "General Notes" }
                p { class: "text-sm text-foreground-neutral-secondary mb-3",
                    "Add any general thoughts about this project (optional). These are for your reference."
                }
                textarea {
                    class: "w-full p-3 text-sm rounded-lg bg-background-neutral-secondary-enabled text-foreground-neutral-primary resize-y min-h-[100px]",
                    placeholder: "Enter your notes here...",
                    value: "{project_notes.read()}",
                    oninput: move |e| {
                        project_notes.set(e.value().clone());
                    },
                }
            }

            // Submit button
            div { class: "flex justify-center items-center gap-4",
                Button {
                    disabled: loading,
                    onclick: move |_| on_skip.call(()),
                    if loading {
                        "Skipping..."
                    } else {
                        "Skip Project (Absent)"
                    }
                }
                Button { disabled: loading, onclick: move |_| on_submit.call(()),
                    if loading {
                        "Submitting..."
                    } else {
                        "Submit"
                    }
                }
            }
        }
    }
}

/// Individual prize card component
#[component]
fn PrizeCard(
    feature: JudgeFeatureState,
    current_project: CurrentProject,
    header_bg_color: String,
    mut selections: Signal<std::collections::HashMap<i32, i32>>,
    mut feature_notes: Signal<std::collections::HashMap<i32, String>>,
    mut expanded_cards: Signal<std::collections::HashSet<i32>>,
) -> Element {
    let feature_id = feature.feature_id;
    let current_submission_id = current_project.submission_id;
    let current_team_name = current_project.team_name.clone();
    let best_team_name = feature
        .current_best_team_name
        .clone()
        .unwrap_or_else(|| "None".to_string());
    let best_description = feature.current_best_description.clone();
    let has_previous_best = feature.current_best_submission_id.is_some();

    let is_expanded = expanded_cards.read().contains(&feature_id);
    let selected_winner = selections.read().get(&feature_id).copied();
    let current_notes = feature_notes
        .read()
        .get(&feature_id)
        .cloned()
        .unwrap_or_default();

    // Determine card background based on state
    let selection_border = if selected_winner.is_some() {
        "ring-2 ring-green-500"
    } else {
        ""
    };

    rsx! {
        div { class: "rounded-lg overflow-hidden {selection_border}",
            // Header
            div { class: "p-4 {header_bg_color}",
                div { class: "flex items-center justify-between",
                    h3 { class: "font-semibold text-foreground-neutral-primary text-base",
                        "{feature.feature_name}"
                    }
                    button {
                        class: "text-foreground-brand-primary hover:text-foreground-brand-secondary",
                        onclick: move |_| {
                            let mut cards = expanded_cards.write();
                            if cards.contains(&feature_id) {
                                cards.remove(&feature_id);
                            } else {
                                cards.insert(feature_id);
                            }
                        },
                        svg {
                            class: "w-5 h-5",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            if is_expanded {
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M5 15l7-7 7 7",
                                }
                            } else {
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M19 9l-7 7-7-7",
                                }
                            }
                        }
                    }
                }
                if let Some(desc) = &feature.feature_description {
                    p { class: "text-xs text-foreground-neutral-secondary mt-1", "{desc}" }
                }
            }

            // Body
            div { class: "p-4 bg-background-neutral-primary",
                // Project to compare
                if has_previous_best {
                    div { class: "mb-4",
                        div { class: "flex items-center justify-between mb-2",
                            span { class: "text-sm text-foreground-neutral-secondary",
                                "Project to Compare: "
                            }
                            span { class: "font-medium text-foreground-neutral-primary",
                                "{best_team_name}"
                            }
                        }

                        if is_expanded {
                            if let Some(desc) = &best_description {
                                p { class: "text-sm text-foreground-neutral-tertiary bg-background-neutral-secondary-enabled p-3 rounded-lg mb-3",
                                    "{desc}"
                                }
                            }
                        }
                    }

                    // Selection radios
                    div {
                        p { class: "text-sm text-foreground-neutral-secondary mb-2",
                            "Select the better project in this track:"
                        }

                        div { class: "space-y-2",
                            // Current project option
                            label { class: "flex items-center gap-2 cursor-pointer",
                                input {
                                    r#type: "radio",
                                    name: "feature-{feature_id}",
                                    checked: selected_winner == Some(current_submission_id),
                                    onchange: move |_| {
                                        selections.write().insert(feature_id, current_submission_id);
                                    },
                                    class: "w-4 h-4 accent-black",
                                }
                                span { class: "text-foreground-neutral-primary", "{current_team_name}" }
                            }

                            // Previous best option
                            if let Some(best_id) = feature.current_best_submission_id {
                                label { class: "flex items-center gap-2 cursor-pointer",
                                    input {
                                        r#type: "radio",
                                        name: "feature-{feature_id}",
                                        checked: selected_winner == Some(best_id),
                                        onchange: move |_| {
                                            selections.write().insert(feature_id, best_id);
                                        },
                                        class: "w-4 h-4 accent-black",
                                    }
                                    span { class: "text-foreground-neutral-primary",
                                        "{best_team_name}"
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // First project for this feature - auto-select as best
                    div {
                        p { class: "text-sm text-foreground-neutral-secondary mb-2",
                            "This is the first project you're seeing for this prize."
                        }
                        label { class: "flex items-center gap-2 cursor-pointer",
                            input {
                                r#type: "checkbox",
                                checked: selected_winner == Some(current_submission_id),
                                onchange: move |_| {
                                    let mut sels = selections.write();
                                    if sels.contains_key(&feature_id) {
                                        sels.remove(&feature_id);
                                    } else {
                                        sels.insert(feature_id, current_submission_id);
                                    }
                                },
                                class: "w-4 h-4 accent-black",
                            }
                            span { class: "text-foreground-neutral-primary",
                                "Mark as current best for this prize"
                            }
                        }
                    }
                }

                // Notes section (expanded only)
                if is_expanded {
                    div { class: "mt-4",
                        label { class: "block text-sm text-foreground-neutral-secondary mb-1",
                            "Notes"
                        }
                        textarea {
                            class: "w-full p-2 text-sm rounded-lg bg-background-neutral-primary-enabled text-foreground-neutral-primary",
                            rows: 3,
                            placeholder: "Add notes for this prize...",
                            value: "{current_notes}",
                            oninput: move |e| {
                                feature_notes.write().insert(feature_id, e.value().clone());
                            },
                        }
                    }
                }
            }
        }
    }
}
