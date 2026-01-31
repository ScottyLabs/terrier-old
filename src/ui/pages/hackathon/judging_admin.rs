use dioxus::prelude::*;
use dioxus_free_icons::{
    Icon,
    icons::ld_icons::{LdPlus, LdTriangleAlert, LdX},
};

use crate::{
    auth::{JUDGING_ADMIN_ROLES, hooks::use_require_access_or_redirect},
    domain::{
        hackathons::types::HackathonInfo,
        judging::{
            handlers::{
                assign_all_judges, assign_prize_judges, close_submissions, create_feature,
                delete_feature, get_features, get_judging_status, get_prizes_with_judges,
                recalculate_rankings, reopen_submissions, reset_judging, start_judging,
                stop_judging, unassign_prize_judge, update_feature,
            },
            types::{
                AssignJudgesRequest, CreateFeatureRequest, FeatureInfo, JudgeInfo, JudgingStatus,
                PrizeInfo, PrizeWithJudges, UpdateFeatureRequest,
            },
        },
        people::handlers::query::{HackathonPerson, get_hackathon_people},
    },
    ui::foundation::components::{Button, ButtonVariant},
};

#[component]
pub fn HackathonJudgingAdmin(slug: String) -> Element {
    if let Some(no_access) = use_require_access_or_redirect(JUDGING_ADMIN_ROLES) {
        return no_access;
    }

    let hackathon = use_context::<Signal<HackathonInfo>>();
    let mut status: Signal<Option<JudgingStatus>> = use_signal(|| None);
    let mut features: Signal<Vec<FeatureInfo>> = use_signal(|| Vec::new());
    let mut available_users: Signal<Vec<(i32, String)>> = use_signal(|| Vec::new());
    let mut loading = use_signal(|| false);
    let mut error_msg: Signal<Option<String>> = use_signal(|| None);
    let mut success_msg: Signal<Option<String>> = use_signal(|| None);

    // Selected feature for editing (features can be managed, but judges are assigned to prize tracks now)
    let mut selected_feature: Signal<Option<FeatureInfo>> = use_signal(|| None);
    let mut edit_name = use_signal(String::new);
    let mut edit_description = use_signal(String::new);
    let mut is_creating_new = use_signal(|| false);

    // Prize track state
    let mut prizes_with_judges: Signal<Vec<PrizeWithJudges>> = use_signal(|| Vec::new());
    let mut selected_prize: Signal<Option<PrizeInfo>> = use_signal(|| None);
    let mut selected_prize_judges: Signal<Vec<JudgeInfo>> = use_signal(|| Vec::new());
    let mut show_prize_judge_picker = use_signal(|| false);
    let mut prize_judge_search = use_signal(String::new);

    // Fetch status and features on mount
    let slug_clone = slug.clone();
    use_effect(move || {
        let slug = slug_clone.clone();
        spawn(async move {
            // Fetch status
            match get_judging_status(slug.clone()).await {
                Ok(s) => status.set(Some(s)),
                Err(e) => error_msg.set(Some(format!("Failed to get status: {}", e))),
            }

            // Fetch features
            match get_features(slug.clone()).await {
                Ok(f) => features.set(f),
                Err(e) => error_msg.set(Some(format!("Failed to get features: {}", e))),
            }

            // Fetch available users for judge assignment
            match get_hackathon_people(slug.clone()).await {
                Ok(users) => {
                    let user_list: Vec<(i32, String)> = users
                        .into_iter()
                        .map(|u: HackathonPerson| (u.user_id, u.name.unwrap_or_else(|| u.email)))
                        .collect();
                    available_users.set(user_list);
                }
                Err(_) => {} // Silently fail - users just won't be able to add judges
            }

            // Fetch prizes with judges
            match get_prizes_with_judges(slug).await {
                Ok(p) => prizes_with_judges.set(p),
                Err(e) => {
                    web_sys::console::log_1(
                        &format!("Failed to fetch prizes with judges: {}", e).into(),
                    );
                }
            }
        });
    });

    let refresh_data = {
        let slug = slug.clone();
        move || {
            let slug = slug.clone();
            spawn(async move {
                if let Ok(s) = get_judging_status(slug.clone()).await {
                    status.set(Some(s));
                }
                if let Ok(f) = get_features(slug.clone()).await {
                    features.set(f);
                }
                if let Ok(p) = get_prizes_with_judges(slug).await {
                    prizes_with_judges.set(p);
                }
            });
        }
    };

    let do_close_submissions = {
        let slug = slug.clone();
        let refresh = refresh_data.clone();
        move |_| {
            let slug = slug.clone();
            let refresh = refresh.clone();
            spawn(async move {
                loading.set(true);
                error_msg.set(None);
                success_msg.set(None);

                match close_submissions(slug).await {
                    Ok(()) => {
                        success_msg.set(Some("Submissions closed successfully.".to_string()));
                        refresh();
                    }
                    Err(e) => {
                        error_msg.set(Some(format!("Failed to close submissions: {}", e)));
                    }
                }
                loading.set(false);
            });
        }
    };

    let do_reopen_submissions = {
        let slug = slug.clone();
        let refresh = refresh_data.clone();
        move |_| {
            let slug = slug.clone();
            let refresh = refresh.clone();
            spawn(async move {
                loading.set(true);
                error_msg.set(None);
                success_msg.set(None);

                match reopen_submissions(slug).await {
                    Ok(()) => {
                        success_msg.set(Some("Submissions re-opened successfully.".to_string()));
                        refresh();
                    }
                    Err(e) => {
                        error_msg.set(Some(format!("Failed to re-open submissions: {}", e)));
                    }
                }
                loading.set(false);
            });
        }
    };

    let do_start_judging = {
        let slug = slug.clone();
        let refresh = refresh_data.clone();
        move |_| {
            let slug = slug.clone();
            let refresh = refresh.clone();
            spawn(async move {
                loading.set(true);
                error_msg.set(None);
                success_msg.set(None);

                match start_judging(slug).await {
                    Ok(()) => {
                        success_msg.set(Some("Judging started successfully.".to_string()));
                        refresh();
                    }
                    Err(e) => {
                        error_msg.set(Some(format!("Failed to start judging: {}", e)));
                    }
                }
                loading.set(false);
            });
        }
    };

    let do_stop_judging = {
        let slug = slug.clone();
        let refresh = refresh_data.clone();
        move |_| {
            let slug = slug.clone();
            let refresh = refresh.clone();
            spawn(async move {
                loading.set(true);
                error_msg.set(None);
                success_msg.set(None);

                match stop_judging(slug).await {
                    Ok(()) => {
                        success_msg.set(Some("Judging stopped successfully.".to_string()));
                        refresh();
                    }
                    Err(e) => {
                        error_msg.set(Some(format!("Failed to stop judging: {}", e)));
                    }
                }
                loading.set(false);
            });
        }
    };

    let do_reset_judging = {
        let slug = slug.clone();
        let refresh = refresh_data.clone();
        move |_| {
            let slug = slug.clone();
            let refresh = refresh.clone();
            spawn(async move {
                let confirmed = web_sys::window()
                    .and_then(|w| w.confirm_with_message("Are you sure you want to reset judging? This will delete all scores and visits.").ok())
                    .unwrap_or(false);

                if !confirmed {
                    return;
                }

                loading.set(true);
                error_msg.set(None);
                success_msg.set(None);

                match reset_judging(slug).await {
                    Ok(()) => {
                        success_msg.set(Some("Judging reset successfully.".to_string()));
                        refresh();
                    }
                    Err(e) => {
                        error_msg.set(Some(format!("Failed to reset judging: {}", e)));
                    }
                }
                loading.set(false);
            });
        }
    };

    let do_recalculate_rankings = {
        let slug = slug.clone();
        let refresh = refresh_data.clone();
        move |_| {
            let slug = slug.clone();
            let refresh = refresh.clone();
            spawn(async move {
                loading.set(true);
                error_msg.set(None);
                success_msg.set(None);

                match recalculate_rankings(slug).await {
                    Ok(()) => {
                        success_msg.set(Some("Rankings recalculated successfully.".to_string()));
                        refresh();
                    }
                    Err(e) => {
                        error_msg.set(Some(format!("Failed to recalculate rankings: {}", e)));
                    }
                }
                loading.set(false);
            });
        }
    };

    let mut select_feature = move |feature: FeatureInfo| {
        edit_name.set(feature.name.clone());
        edit_description.set(feature.description.clone().unwrap_or_default());
        selected_feature.set(Some(feature));
        is_creating_new.set(false);
    };

    let start_create_feature = move |_| {
        edit_name.set(String::new());
        edit_description.set(String::new());
        selected_feature.set(None);
        is_creating_new.set(true);
    };

    let do_save_feature = {
        let slug = slug.clone();
        let refresh = refresh_data.clone();
        move |_| {
            let slug = slug.clone();
            let refresh = refresh.clone();
            let name = edit_name.read().clone();
            let description = if edit_description.read().is_empty() {
                None
            } else {
                Some(edit_description.read().clone())
            };
            let is_new = *is_creating_new.read();
            let feature_id = selected_feature.read().as_ref().map(|f| f.id);

            spawn(async move {
                loading.set(true);
                error_msg.set(None);
                success_msg.set(None);

                if is_new {
                    // Create new feature
                    let request = CreateFeatureRequest { name, description };
                    match create_feature(slug, request).await {
                        Ok(_) => {
                            success_msg.set(Some("Feature created successfully.".to_string()));
                            refresh();
                            is_creating_new.set(false);
                            selected_feature.set(None);
                        }
                        Err(e) => {
                            error_msg.set(Some(format!("Failed to create feature: {}", e)));
                        }
                    }
                } else if let Some(fid) = feature_id {
                    // Update existing feature
                    let request = UpdateFeatureRequest { name, description };
                    match update_feature(slug, fid, request).await {
                        Ok(updated) => {
                            success_msg.set(Some("Feature updated successfully.".to_string()));
                            refresh();
                            selected_feature.set(Some(updated));
                        }
                        Err(e) => {
                            error_msg.set(Some(format!("Failed to update feature: {}", e)));
                        }
                    }
                }

                loading.set(false);
            });
        }
    };

    let do_delete_feature = {
        let slug = slug.clone();
        let refresh = refresh_data.clone();
        move |_| {
            if let Some(feature) = selected_feature.read().as_ref() {
                let slug = slug.clone();
                let refresh = refresh.clone();
                let feature_id = feature.id;

                spawn(async move {
                    loading.set(true);
                    error_msg.set(None);
                    success_msg.set(None);

                    match delete_feature(slug, feature_id).await {
                        Ok(_) => {
                            success_msg.set(Some("Feature deleted successfully.".to_string()));
                            refresh();
                            selected_feature.set(None);
                        }
                        Err(e) => {
                            error_msg.set(Some(format!("Failed to delete feature: {}", e)));
                        }
                    }

                    loading.set(false);
                });
            }
        }
    };

    let cancel_edit = move |_| {
        selected_feature.set(None);
        is_creating_new.set(false);
    };

    let _hackathon_info = hackathon.read();
    let judging_not_started = status
        .read()
        .as_ref()
        .map(|s| !s.judging_started)
        .unwrap_or(true);

    rsx! {
        div { class: "pt-11 pb-7",
            // Header with Add button
            div { class: "flex items-center justify-between mb-8",
                h1 { class: "text-[30px] font-semibold leading-[38px] text-foreground-neutral-primary",
                    "Judging Admin"
                }

                // Add new feature button (only when judging not started)
                if judging_not_started {
                    button {
                        class: "flex items-center gap-2 px-4 py-2 bg-foreground-neutral-primary text-white font-semibold text-sm rounded-full cursor-pointer",
                        onclick: start_create_feature,
                        Icon { width: 16, height: 16, icon: LdPlus }
                        "Add new feature"
                    }
                }
            }

            // Error message
            if let Some(err) = error_msg.read().as_ref() {
                div { class: "mb-4 p-4 bg-background-danger-secondary rounded-lg",
                    p { class: "text-foreground-danger-primary", "{err}" }
                }
            }

            // Success message
            if let Some(msg) = success_msg.read().as_ref() {
                div { class: "mb-4 p-4 bg-background-success-secondary rounded-lg",
                    p { class: "text-foreground-success-primary", "{msg}" }
                }
            }

            // Status panel
            if let Some(s) = status.read().as_ref() {
                div { class: "mb-8 p-6 bg-background-neutral-primary rounded-[20px]",
                    h2 { class: "text-xl font-semibold text-foreground-neutral-primary mb-4",
                        "Judging Status"
                    }

                    div { class: "grid grid-cols-2 md:grid-cols-4 gap-4 mb-6",
                        div { class: "p-4 border border-stroke-neutral-1 rounded-lg",
                            div { class: "text-sm text-foreground-neutral-secondary",
                                "Submissions Closed"
                            }
                            div {
                                class: "text-lg font-semibold",
                                class: if s.submissions_closed { "text-foreground-success-primary" } else { "text-foreground-neutral-primary" },
                                if s.submissions_closed {
                                    "Yes"
                                } else {
                                    "No"
                                }
                            }
                        }
                        div { class: "p-4 border border-stroke-neutral-1 rounded-lg",
                            div { class: "text-sm text-foreground-neutral-secondary",
                                "Judging Active"
                            }
                            div {
                                class: "text-lg font-semibold",
                                class: if s.judging_started { "text-foreground-success-primary" } else { "text-foreground-neutral-primary" },
                                if s.judging_started {
                                    "Yes"
                                } else {
                                    "No"
                                }
                            }
                        }
                        div { class: "p-4 border border-stroke-neutral-1 rounded-lg",
                            div { class: "text-sm text-foreground-neutral-secondary",
                                "Total Submissions"
                            }
                            div { class: "text-lg font-semibold text-foreground-neutral-primary",
                                "{s.total_submissions}"
                            }
                        }
                        div { class: "p-4 border border-stroke-neutral-1 rounded-lg",
                            div { class: "text-sm text-foreground-neutral-secondary",
                                "Visited"
                            }
                            div { class: "text-lg font-semibold text-foreground-neutral-primary",
                                "{s.visited_submissions}/{s.total_submissions}"
                            }
                        }
                        div { class: "p-4 border border-stroke-neutral-1 rounded-lg",
                            div { class: "text-sm text-foreground-neutral-secondary",
                                "Tables Assigned"
                            }
                            div {
                                class: "text-lg font-semibold",
                                class: if s.projects_with_tables == s.total_submissions { "text-foreground-neutral-primary" } else { "text-foreground-danger-primary" },
                                "{s.projects_with_tables}/{s.total_submissions}"
                            }
                        }
                    }

                    if !s.unassigned_projects.is_empty() {
                        div { class: "mb-6 p-4 bg-background-danger-secondary rounded-lg border border-stroke-neutral-1",
                            div { class: "flex items-center gap-2 mb-2",
                                span { class: "text-foreground-danger-primary",
                                    Icon {
                                        width: 20,
                                        height: 20,
                                        icon: LdTriangleAlert,
                                    }
                                }
                                span { class: "text-foreground-danger-primary font-bold",
                                    "Projects missing table numbers:"
                                }
                                span { class: "text-xs px-2 py-0.5 bg-foreground-danger-primary text-white rounded-full",
                                    "{s.unassigned_projects.len()}"
                                }
                            }
                            div { class: "flex flex-wrap gap-2",
                                for team in s.unassigned_projects.iter() {
                                    span { class: "text-xs px-2 py-1 bg-background-neutral-primary border border-stroke-neutral-1 rounded text-foreground-neutral-primary",
                                        "{team}"
                                    }
                                }
                            }
                        }
                    }

                    div { class: "grid grid-cols-2 gap-4 mb-6",
                        div { class: "p-4 border border-stroke-neutral-1 rounded-lg",
                            div { class: "text-sm text-foreground-neutral-secondary",
                                "Total Visits"
                            }
                            div { class: "text-lg font-semibold text-foreground-neutral-primary",
                                "{s.total_visits}"
                            }
                        }
                        div { class: "p-4 border border-stroke-neutral-1 rounded-lg",
                            div { class: "text-sm text-foreground-neutral-secondary",
                                "Total Comparisons"
                            }
                            div { class: "text-lg font-semibold text-foreground-neutral-primary",
                                "{s.total_comparisons}"
                            }
                        }
                    }

                    // Control buttons
                    div { class: "flex flex-wrap gap-4",
                        if !s.submissions_closed {
                            Button {
                                disabled: *loading.read(),
                                onclick: do_close_submissions,
                                if *loading.read() {
                                    "Closing..."
                                } else {
                                    "Close Submissions"
                                }
                            }
                        }

                        if s.submissions_closed && !s.judging_started {
                            Button {
                                disabled: *loading.read(),
                                onclick: do_reopen_submissions,
                                if *loading.read() {
                                    "Re-opening..."
                                } else {
                                    "Re-open Submissions"
                                }
                            }
                        }

                        if s.submissions_closed && !s.judging_started {
                            Button {
                                disabled: *loading.read(),
                                onclick: do_start_judging,
                                if *loading.read() {
                                    "Starting..."
                                } else {
                                    "Start Judging"
                                }
                            }
                        }

                        if s.judging_started {
                            Button {
                                disabled: *loading.read(),
                                onclick: do_stop_judging,
                                if *loading.read() {
                                    "Stopping..."
                                } else {
                                    "Stop Judging"
                                }
                            }
                        }

                        if s.submissions_closed {
                            Button {
                                variant: ButtonVariant::Secondary,
                                disabled: *loading.read(),
                                onclick: do_recalculate_rankings,
                                if *loading.read() {
                                    "Recalculating..."
                                } else {
                                    "Recalculate Results"
                                }
                            }
                        }

                        if s.submissions_closed {
                            Button {
                                variant: ButtonVariant::Danger,
                                disabled: *loading.read(),
                                onclick: do_reset_judging,
                                if *loading.read() {
                                    "Resetting..."
                                } else {
                                    "Reset Judging Data"
                                }
                            }
                        }
                    }
                }
            } else {
                div { class: "mb-8 p-6 bg-background-neutral-primary rounded-[20px]",
                    p { class: "text-foreground-neutral-secondary", "Loading status..." }
                }
            }

            // Features section (only show when judging not started)
            if judging_not_started {
                div { class: "flex flex-col lg:flex-row gap-6",
                    // Left: Feature cards
                    div { class: "flex-1",
                        div { class: "p-6 bg-background-neutral-primary rounded-[20px]",
                            h2 { class: "text-xl font-semibold text-foreground-neutral-primary mb-4",
                                "Judging Features"
                            }

                            if features.read().is_empty() && !*is_creating_new.read() {
                                p { class: "text-foreground-neutral-secondary",
                                    "No judging features defined yet. Click \"Add new feature\" to create one."
                                }
                            } else {
                                div { class: "space-y-4",
                                    for feature in features.read().iter() {
                                        {
                                            let is_selected = selected_feature

                                                .read()
                                                .as_ref()
                                                .map(|f| f.id == feature.id)
                                                .unwrap_or(false);
                                            let feature_clone = feature.clone();
                                            rsx! {
                                                div {
                                                    key: "{feature.id}",
                                                    class: "p-6 border rounded-lg cursor-pointer transition-colors",
                                                    class: if is_selected { "border-foreground-neutral-primary bg-background-neutral-secondary-enabled" } else { "border-stroke-neutral-1 hover:border-stroke-neutral-2" },
                                                    onclick: move |_| select_feature(feature_clone.clone()),
                                                    div { class: "flex items-center justify-between mb-2",
                                                        h3 { class: "font-semibold text-lg text-foreground-neutral-primary", "{feature.name}" }
                                                        span { class: "px-3 py-1 text-xs font-semibold rounded-full bg-foreground-neutral-primary text-white",
                                                            "Edit"
                                                        }
                                                    }

                                                    if let Some(desc) = &feature.description {
                                                        p { class: "text-sm text-foreground-neutral-secondary line-clamp-3", "{desc}" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Right: Edit panel (only show when editing or creating)
                    if selected_feature.read().is_some() || *is_creating_new.read() {
                        div { class: "w-full lg:w-96",
                            div { class: "p-6 bg-background-neutral-primary rounded-[20px] sticky top-4",
                                // Header
                                div { class: "mb-4",
                                    label { class: "text-xs text-foreground-neutral-secondary",
                                        "Feature name"
                                    }
                                    h3 { class: "text-lg font-semibold text-foreground-neutral-primary",
                                        if *is_creating_new.read() {
                                            "New Feature"
                                        } else {
                                            "{edit_name}"
                                        }
                                    }
                                }

                                // Name input
                                div { class: "mb-4",
                                    label { class: "block text-sm font-medium text-foreground-neutral-primary mb-1",
                                        "Name"
                                    }
                                    input {
                                        class: "w-full px-3 py-2 border border-stroke-neutral-1 rounded-lg text-foreground-neutral-primary bg-background-neutral-primary",
                                        r#type: "text",
                                        value: "{edit_name}",
                                        oninput: move |e| edit_name.set(e.value()),
                                        placeholder: "Feature name",
                                    }
                                }

                                // Description textarea
                                div { class: "mb-6",
                                    label { class: "block text-sm font-medium text-foreground-neutral-primary mb-1",
                                        "Description"
                                    }
                                    textarea {
                                        class: "w-full px-3 py-2 border border-stroke-neutral-1 rounded-lg text-foreground-neutral-primary bg-background-neutral-primary resize-none",
                                        rows: 6,
                                        value: "{edit_description}",
                                        oninput: move |e| edit_description.set(e.value()),
                                        placeholder: "Describe this feature...",
                                    }
                                }

                                // Action buttons
                                div { class: "flex gap-3 justify-end",
                                    if !*is_creating_new.read() {
                                        Button {
                                            variant: ButtonVariant::Danger,
                                            disabled: *loading.read(),
                                            onclick: do_delete_feature,
                                            "Delete"
                                        }
                                    }

                                    button {
                                        class: "px-4 py-2 text-sm font-medium text-foreground-neutral-primary border border-stroke-neutral-1 rounded-full cursor-pointer",
                                        onclick: cancel_edit,
                                        "Cancel"
                                    }

                                    Button {
                                        disabled: *loading.read() || edit_name.read().is_empty(),
                                        onclick: do_save_feature,
                                        if *loading.read() {
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

            // Prize Track Judges section (only show when judging not started)
            if judging_not_started {
                div { class: "mt-8",
                    div { class: "flex flex-col lg:flex-row gap-6",
                        // Left: Prize track list
                        div { class: "flex-1",
                            div { class: "p-6 bg-background-neutral-primary rounded-[20px]",
                                h2 { class: "text-xl font-semibold text-foreground-neutral-primary mb-4",
                                    "Prize Track Judges"
                                }
                                p { class: "text-sm text-foreground-neutral-secondary mb-6",
                                    "Assign judges to prize tracks. Default tracks (no assigned judges) can be judged by all judges."
                                }

                                if prizes_with_judges.read().is_empty() {
                                    p { class: "text-foreground-neutral-secondary",
                                        "No prize tracks defined yet."
                                    }
                                } else {
                                    div { class: "space-y-4",
                                        for pwj in prizes_with_judges.read().iter() {
                                            {
                                                let is_selected = selected_prize
                                                    .read()
                                                    .as_ref()
                                                    .map(|p| p.id == pwj.prize.id)
                                                    .unwrap_or(false);
                                                let prize_clone = pwj.prize.clone();
                                                let judges_clone = pwj.judges.clone();
                                                rsx! {
                                                    div {
                                                        key: "{pwj.prize.id}",
                                                        class: "p-6 border rounded-lg cursor-pointer transition-colors",
                                                        class: if is_selected { "border-foreground-neutral-primary bg-background-neutral-secondary-enabled" } else { "border-stroke-neutral-1 hover:border-stroke-neutral-2" },
                                                        onclick: move |_| {
                                                            selected_prize.set(Some(prize_clone.clone()));
                                                            selected_prize_judges.set(judges_clone.clone());
                                                            show_prize_judge_picker.set(false);
                                                            prize_judge_search.set(String::new());
                                                        },





















                                                        div { class: "flex items-center justify-between mb-2",
                                                            h3 { class: "font-semibold text-lg text-foreground-neutral-primary", "{pwj.prize.name}" }
                                                            if pwj.is_default {
                                                                span { class: "px-3 py-1 text-xs font-semibold rounded-full bg-background-success-secondary text-foreground-success-primary",
                                                                    "Default (All Judges)"
                                                                }
                                                            } else {
                                                                span { class: "px-3 py-1 text-xs font-semibold rounded-full bg-foreground-neutral-primary text-white",
                                                                    "{pwj.judges.len()} judges"
                                                                }
                                                            }
                                                        }

                                                        if !pwj.is_default && !pwj.judges.is_empty() {
                                                            div { class: "flex flex-wrap gap-2 mt-2",
                                                                for judge in pwj.judges.iter().take(5) {
                                                                    span { class: "px-2 py-1 text-xs bg-background-neutral-secondary-enabled rounded",
                                                                        "{judge.name}"
                                                                    }
                                                                }
                                                                if pwj.judges.len() > 5 {
                                                                    span { class: "px-2 py-1 text-xs bg-background-neutral-secondary-enabled rounded",
                                                                        "+{pwj.judges.len() - 5} more"
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Right: Edit panel for selected prize track
                        if selected_prize.read().is_some() {
                            {
                                let slug = slug.clone();
                                let refresh = refresh_data.clone();
                                rsx! {
                                    div { class: "w-full lg:w-96",
                                        div { class: "p-6 bg-background-neutral-primary rounded-[20px] sticky top-4",
                                            // Header
                                            div { class: "flex items-center justify-between mb-4",
                                                h3 { class: "text-lg font-semibold text-foreground-neutral-primary",
                                                    "{selected_prize.read().as_ref().map(|p| p.name.clone()).unwrap_or_default()}"
                                                }
                                                button {
                                                    class: "text-foreground-neutral-secondary hover:text-foreground-neutral-primary",
                                                    onclick: move |_| {
                                                        selected_prize.set(None);
                                                        show_prize_judge_picker.set(false);
                                                    },

                                    // Assigned judges list

                                    // Update selected judges

                                    // Judge picker dropdown
                                    // Update selected judges

                                    // Make Default button






































































                                                    Icon { width: 20, height: 20, icon: LdX }
                                                }
                                            }

                                            div { class: "mb-4",
                                                div { class: "flex items-center justify-between mb-2",
                                                    label { class: "text-sm text-foreground-neutral-secondary", "Assigned Judges" }
                                                    button {
                                                        class: "text-xs text-foreground-neutral-primary flex items-center gap-1 hover:underline cursor-pointer",
                                                        onclick: move |_| {
                                                            let current = *show_prize_judge_picker.read();
                                                            show_prize_judge_picker.set(!current);
                                                        },
                                                        Icon { width: 14, height: 14, icon: LdPlus }
                                                        "Add judge"
                                                    }
                                                }

                                                if selected_prize_judges.read().is_empty() {
                                                    p { class: "text-sm text-foreground-success-primary",
                                                        "Default - all judges can judge this track"
                                                    }
                                                } else {
                                                    div { class: "flex flex-wrap gap-2",
                                                        for judge in selected_prize_judges.read().iter() {
                                                            {
                                                                let judge_id = judge.user_id;
                                                                let slug = slug.clone();
                                                                let refresh = refresh.clone();
                                                                let prize_id = selected_prize.read().as_ref().map(|p| p.id).unwrap_or_default();
                                                                rsx! {
                                                                    span { class: "inline-flex items-center gap-1 px-2 py-1 bg-background-neutral-secondary-enabled rounded text-sm",
                                                                        "{judge.name}"
                                                                        button {
                                                                            class: "text-foreground-neutral-secondary hover:text-foreground-danger-primary cursor-pointer",
                                                                            onclick: move |_| {
                                                                                let slug = slug.clone();
                                                                                let refresh = refresh.clone();
                                                                                spawn(async move {
                                                                                    if unassign_prize_judge(slug.clone(), prize_id, judge_id).await.is_ok() {
                                                                                        refresh();
                                                                                        if let Ok(p) = get_prizes_with_judges(slug).await {
                                                                                            prizes_with_judges.set(p.clone());
                                                                                            let judges = p
                                                                                                .iter()
                                                                                                .find(|pwj| pwj.prize.id == prize_id)
                                                                                                .map(|pwj| pwj.judges.clone())
                                                                                                .unwrap_or_default();
                                                                                            selected_prize_judges.set(judges);
                                                                                        }
                                                                                    }
                                                                                });
                                                                            },
                                                                            Icon { width: 14, height: 14, icon: LdX }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }

                                            if *show_prize_judge_picker.read() {
                                                div { class: "mt-4 p-4 border border-stroke-neutral-1 rounded-lg",
                                                    input {
                                                        r#type: "text",
                                                        class: "w-full px-3 py-2 border border-stroke-neutral-1 rounded-lg mb-2",
                                                        placeholder: "Search users...",
                                                        value: "{prize_judge_search}",
                                                        oninput: move |e| prize_judge_search.set(e.value()),
                                                    }
                                                    div { class: "max-h-48 overflow-y-auto space-y-1",
                                                        {
                                                            let search = prize_judge_search.read().to_lowercase();
                                                            let assigned_ids: Vec<i32> = selected_prize_judges
                                                                .read()
                                                                .iter()
                                                                .map(|j| j.user_id)
                                                                .collect();
                                                            let filtered: Vec<_> = available_users
                                                                .read()
                                                                .iter()
                                                                .filter(|(id, name)| {
                                                                    !assigned_ids.contains(id) && name.to_lowercase().contains(&search)
                                                                })
                                                                .take(10)
                                                                .cloned()
                                                                .collect();
                                                            rsx! {
                                                                for (id , name) in filtered {
                                                                    {
                                                                        let slug = slug.clone();
                                                                        let refresh = refresh.clone();
                                                                        let prize_id = selected_prize.read().as_ref().map(|p| p.id).unwrap_or_default();
                                                                        rsx! {
                                                                            button {
                                                                                class: "w-full text-left px-3 py-2 hover:bg-background-neutral-secondary-enabled rounded cursor-pointer",
                                                                                onclick: move |_| {
                                                                                    let slug = slug.clone();
                                                                                    let refresh = refresh.clone();
                                                                                    spawn(async move {
                                                                                        let request = AssignJudgesRequest {
                                                                                            judge_ids: vec![id],
                                                                                        };
                                                                                        if assign_prize_judges(slug.clone(), prize_id, request).await.is_ok() {
                                                                                            refresh();
                                                                                            if let Ok(p) = get_prizes_with_judges(slug).await {
                                                                                                prizes_with_judges.set(p.clone());
                                                                                                let judges = p
                                                                                                    .iter()
                                                                                                    .find(|pwj| pwj.prize.id == prize_id)
                                                                                                    .map(|pwj| pwj.judges.clone())
                                                                                                    .unwrap_or_default();
                                                                                                selected_prize_judges.set(judges);
                                                                                            }
                                                                                            show_prize_judge_picker.set(false);
                                                                                            prize_judge_search.set(String::new());
                                                                                        }
                                                                                    });
                                                                                },
                                                                                "{name}"
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }



                                            div { class: "mt-4 pt-4 border-t border-stroke-neutral-1",
                                                Button {
                                                    variant: ButtonVariant::Secondary,
                                                    onclick: move |_| {
                                                        let slug = slug.clone();
                                                        let refresh = refresh.clone();
                                                        let prize_id = selected_prize.read().as_ref().map(|p| p.id).unwrap_or_default();
                                                        let price_id_copy = prize_id;
                                                        spawn(async move {
                                                            if assign_all_judges(slug.clone(), price_id_copy).await.is_ok() {
                                                                refresh();
                                                                if let Ok(p) = get_prizes_with_judges(slug).await {
                                                                    prizes_with_judges.set(p.clone());
                                                                        let judges = p
                                                                        .iter()
                                                                        .find(|pwj| pwj.prize.id == price_id_copy)
                                                                        .map(|pwj| pwj.judges.clone())
                                                                        .unwrap_or_default();
                                                                    selected_prize_judges.set(judges);
                                                                }
                                                            }
                                                        });
                                                    },
                                                    "Assign All Judges"
                                                }

                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
