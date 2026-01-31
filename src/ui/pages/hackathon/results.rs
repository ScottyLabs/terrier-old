use dioxus::prelude::*;
use dioxus_free_icons::{
    Icon,
    icons::ld_icons::{LdChevronDown, LdClipboard, LdLoader, LdSparkles},
};

use crate::{
    auth::{RESULTS_ROLES, hooks::use_require_access_or_redirect},
    domain::{
        judging::{
            handlers::{generate_ai_summary, get_my_visit_notes, get_prize_track_results},
            types::{AiSummaryResponse, JudgeVisitNotes, PrizeTrackResults, ProjectResultInfo},
        },
        prizes::handlers::get_prizes,
    },
    ui::foundation::modals::base::ModalBase,
};

#[component]
pub fn HackathonResults(slug: String) -> Element {
    // Results page should only be accessible to judges, organizers, and admins
    // (not participants)
    if let Some(no_access) = use_require_access_or_redirect(RESULTS_ROLES) {
        return no_access;
    }

    // State
    let mut selected_prize_id: Signal<Option<i32>> = use_signal(|| None);
    let mut selected_project: Signal<Option<ProjectResultInfo>> = use_signal(|| None);
    let mut dropdown_open = use_signal(|| false);
    let mut my_notes: Signal<Option<JudgeVisitNotes>> = use_signal(|| None);

    // Fetch prizes
    let prizes_resource = use_resource({
        let slug = slug.clone();
        move || {
            let slug = slug.clone();
            async move { get_prizes(slug).await.ok() }
        }
    });

    // Fetch results for selected prize
    let results_resource = use_resource({
        let slug = slug.clone();
        move || {
            let slug = slug.clone();
            let prize_id = selected_prize_id();
            async move {
                if let Some(pid) = prize_id {
                    get_prize_track_results(slug, pid).await.ok()
                } else {
                    None
                }
            }
        }
    });

    // Fetch notes when project is selected
    let slug_for_notes = slug.clone();
    use_effect(move || {
        if let Some(project) = selected_project() {
            let slug = slug_for_notes.clone();
            spawn(async move {
                if let Ok(notes) = get_my_visit_notes(slug, project.submission_id).await {
                    my_notes.set(Some(notes));
                }
            });
        } else {
            my_notes.set(None);
        }
    });

    // Get the selected prize name for display
    let selected_prize_name = {
        let prizes = prizes_resource.read();
        let prize_id = selected_prize_id();
        if let (Some(Some(prizes)), Some(pid)) = (prizes.as_ref(), prize_id) {
            prizes.iter().find(|p| p.id == pid).map(|p| p.name.clone())
        } else {
            None
        }
    };

    rsx! {
        div { class: "flex flex-col h-full",
            // Header
            h1 { class: "text-[30px] font-semibold leading-[38px] text-foreground-neutral-primary pt-11 pb-7",
                "Results"
            }

            // Prize selector dropdown
            div { class: "mb-6 relative inline-block w-fit",
                button {
                    class: "flex items-center justify-between gap-2 px-4 py-2 min-w-[200px] border border-stroke-neutral-1 rounded-lg bg-background-neutral-primary text-foreground-neutral-primary cursor-pointer",
                    onclick: move |_| dropdown_open.toggle(),
                    span {
                        if let Some(name) = &selected_prize_name {
                            "{name}"
                        } else {
                            "Prize Name"
                        }
                    }
                    Icon {
                        width: 20,
                        height: 20,
                        icon: LdChevronDown,
                        class: "text-foreground-neutral-tertiary",
                    }
                }

                // Dropdown menu
                if dropdown_open() {
                    div { class: "absolute top-[calc(100%+4px)] left-0 z-10 w-full min-w-[200px] bg-background-neutral-primary border border-stroke-neutral-1 rounded-lg shadow-lg",
                        match prizes_resource.read().as_ref() {
                            Some(Some(prizes)) if !prizes.is_empty() => rsx! {
                                for prize in prizes.iter() {
                                    {
                                        let prize_id = prize.id;
                                        let prize_name = prize.name.clone();
                                        rsx! {
                                            button {
                                                key: "{prize.id}",
                                                class: "w-full px-4 py-2 text-left hover:bg-background-neutral-secondary-enabled text-foreground-neutral-primary first:rounded-t-lg last:rounded-b-lg",
                                                onclick: move |_| {
                                                    selected_prize_id.set(Some(prize_id));
                                                    dropdown_open.set(false);
                                                },
                                                "{prize_name}"
                                            }
                                        }
                                    }
                                }
                            },
                            Some(Some(_)) => rsx! {
                                div { class: "px-4 py-2 text-foreground-neutral-secondary", "No prizes available" }
                            },
                            _ => rsx! {
                                div { class: "px-4 py-2 text-foreground-neutral-secondary", "Loading..." }
                            },
                        }
                    }
                }
            }

            // Results table
            div { class: "flex-1 overflow-auto",
                div { class: "bg-background-neutral-primary rounded-[20px] p-7",
                    match results_resource.read().as_ref() {
                        Some(Some(results)) => rsx! {
                            ResultsTable {
                                results: results.clone(),
                                on_project_click: move |project: ProjectResultInfo| {
                                    selected_project.set(Some(project));
                                },
                            }
                        },
                        Some(None) if selected_prize_id().is_some() => rsx! {
                            p { class: "text-foreground-neutral-secondary text-center py-8", "Failed to load results" }
                        },
                        _ => rsx! {
                            p { class: "text-foreground-neutral-secondary text-center py-8",
                                "Select a prize track to view results"
                            }
                        },
                    }
                }
            }
        }

        // Project detail modal
        if let Some(project) = selected_project() {
            ProjectDetailModal {
                slug: slug.clone(),
                project: project.clone(),
                my_notes: my_notes(),
                on_close: move |_| selected_project.set(None),
            }
        }
    }
}

#[component]
fn ResultsTable(
    results: PrizeTrackResults,
    on_project_click: EventHandler<ProjectResultInfo>,
) -> Element {
    rsx! {
        div { class: "overflow-x-auto",
            table { class: "w-full",
                thead {
                    tr { class: "border-b border-stroke-neutral-1",
                        th { class: "text-left py-3 px-2 text-sm font-semibold text-foreground-neutral-primary",
                            "Project Name"
                        }
                        th { class: "text-left py-3 px-2 text-sm font-semibold text-foreground-neutral-primary",
                            "Team Name"
                        }
                        th { class: "text-left py-3 px-2 text-sm font-semibold text-foreground-neutral-primary",
                            "Table"
                        }
                        th { class: "text-left py-3 px-2 text-sm font-semibold text-foreground-neutral-primary",
                            "Score"
                            span { class: "ml-1 text-xs text-foreground-neutral-tertiary",
                                "ⓘ"
                            }
                        }
                        // Dynamic feature columns
                        for feature in results.features.iter() {
                            th {
                                key: "{feature.id}",
                                class: "text-left py-3 px-2 text-sm font-semibold text-foreground-neutral-primary",
                                "{feature.name}"
                            }
                        }
                    }
                }
                tbody {
                    if results.projects.is_empty() {
                        tr {
                            td {
                                colspan: "{4 + results.features.len()}",
                                class: "text-center py-8 text-foreground-neutral-secondary",
                                "No submissions yet"
                            }
                        }
                    } else {
                        for project in results.projects.iter() {
                            {
                                let project_clone = project.clone();
                                rsx! {
                                    tr {
                                        key: "{project.submission_id}",
                                        class: "border-b border-stroke-neutral-1 hover:bg-background-neutral-secondary-enabled cursor-pointer transition-colors",
                                        onclick: move |_| on_project_click.call(project_clone.clone()),
                                        td { class: "py-3 px-2 text-sm text-foreground-neutral-primary",
                                            {project.project_name.clone().unwrap_or_else(|| "Untitled".to_string())}
                                        }
                                        // Feature rank columns
                                        td { class: "py-3 px-2 text-sm text-foreground-neutral-primary", "{project.team_name}" }
                                        td { class: "py-3 px-2 text-sm text-foreground-neutral-primary font-mono",
                                            {project.table_number.clone().unwrap_or_else(|| "-".to_string())}
                                        }
                                        td { class: "py-3 px-2 text-sm text-foreground-neutral-primary",
                                            {format!("{:.2}", project.weighted_score.unwrap_or(0.0))}
                                        }
                                        for rank_info in project.feature_ranks.iter() {
                                            td {
                                                key: "{rank_info.feature_id}",
                                                class: "py-3 px-2 text-sm text-foreground-neutral-primary",
                                                {rank_info.rank.map(|r| format!("#{}", r)).unwrap_or_else(|| "-".to_string())}
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

#[component]
fn ProjectDetailModal(
    slug: String,
    project: ProjectResultInfo,
    my_notes: Option<JudgeVisitNotes>,
    on_close: EventHandler<()>,
) -> Element {
    let mut ai_summary: Signal<Option<String>> = use_signal(|| project.ai_summary.clone());
    let mut loading_summary = use_signal(|| false);
    let mut summary_error: Signal<Option<String>> = use_signal(|| None);

    let submission_id = project.submission_id;
    let slug_clone = slug.clone();

    let do_generate_summary = move |_| {
        let slug = slug_clone.clone();
        spawn(async move {
            loading_summary.set(true);
            summary_error.set(None);

            match generate_ai_summary(slug, submission_id).await {
                Ok(response) => {
                    ai_summary.set(Some(response.summary));
                }
                Err(e) => {
                    summary_error.set(Some(format!("Failed to generate summary: {}", e)));
                }
            }

            loading_summary.set(false);
        });
    };

    rsx! {
        ModalBase { on_close, width: "500px", max_height: "90vh",

            div { class: "px-7 pb-7",
                // Project name with copy icon
                div { class: "flex items-start justify-between mb-2",
                    h2 { class: "text-2xl font-semibold text-foreground-neutral-primary",
                        {project.project_name.clone().unwrap_or_else(|| "Untitled Project".to_string())}
                    }
                    button {
                        class: "p-2 text-foreground-neutral-tertiary hover:text-foreground-neutral-primary transition-colors",
                        title: "Copy project info",
                        Icon { width: 20, height: 20, icon: LdClipboard }
                    }
                }

                // Project Description label
                p { class: "text-sm font-medium text-foreground-neutral-primary mb-2",
                    "Project Description"
                }

                // Description box
                div { class: "bg-background-neutral-secondary rounded-lg p-4 mb-4",
                    p { class: "text-sm text-foreground-neutral-secondary",
                        {
                            project
                                .description
                                .clone()
                                .unwrap_or_else(|| "No description provided.".to_string())
                        }
                    }
                }

                // Repo URL
                if project.repo_url.is_some() {
                    div { class: "mb-3",
                        label { class: "block text-sm font-medium text-foreground-neutral-primary mb-1",
                            "Repo URL"
                        }
                        div { class: "bg-background-neutral-secondary rounded-lg px-4 py-2",
                            a {
                                href: "{project.repo_url.clone().unwrap_or_default()}",
                                target: "_blank",
                                class: "text-sm text-foreground-brand-primary hover:underline",
                                {project.repo_url.clone().unwrap_or_default()}
                            }
                        }
                    }
                }

                // Presentation URL
                if project.presentation_url.is_some() {
                    div { class: "mb-3",
                        label { class: "block text-sm font-medium text-foreground-neutral-primary mb-1",
                            "Presentation URL"
                        }
                        div { class: "bg-background-neutral-secondary rounded-lg px-4 py-2",
                            a {
                                href: "{project.presentation_url.clone().unwrap_or_default()}",
                                target: "_blank",
                                class: "text-sm text-foreground-brand-primary hover:underline",
                                {project.presentation_url.clone().unwrap_or_default()}
                            }
                        }
                    }
                }

                // Video URL
                if project.video_url.is_some() {
                    div { class: "mb-4",
                        label { class: "block text-sm font-medium text-foreground-neutral-primary mb-1",
                            "Video URL"
                        }
                        div { class: "bg-background-neutral-secondary rounded-lg px-4 py-2",
                            a {
                                href: "{project.video_url.clone().unwrap_or_default()}",
                                target: "_blank",
                                class: "text-sm text-foreground-brand-primary hover:underline",
                                {project.video_url.clone().unwrap_or_default()}
                            }
                        }
                    }
                }

                // AI Summary section
                div { class: "mb-4",
                    div { class: "flex items-center justify-between mb-1",
                        label { class: "block text-sm font-medium text-foreground-neutral-primary",
                            "AI Summary"
                        }
                        button {
                            class: "flex items-center gap-1 px-3 py-1 text-xs font-medium rounded-full bg-foreground-brand-primary text-black hover:bg-foreground-brand-secondary transition-colors disabled:opacity-50",
                            disabled: loading_summary(),
                            onclick: do_generate_summary,
                            if loading_summary() {
                                Icon {
                                    width: 14,
                                    height: 14,
                                    icon: LdLoader,
                                    class: "animate-spin",
                                }
                                "Generating..."
                            } else {
                                Icon { width: 14, height: 14, icon: LdSparkles }
                                "Generate AI Summary"
                            }
                        }
                    }

                    // Error message
                    if let Some(err) = summary_error() {
                        div { class: "bg-background-danger-secondary rounded-lg p-3 mb-2",
                            p { class: "text-sm text-foreground-danger-primary", "{err}" }
                        }
                    }

                    // Summary content
                    div { class: "bg-background-neutral-secondary rounded-lg p-4",
                        if let Some(summary) = ai_summary() {
                            p { class: "text-sm text-foreground-neutral-secondary italic",
                                "{summary}"
                            }
                        } else {
                            p { class: "text-sm text-foreground-neutral-tertiary italic",
                                "Click 'Generate AI Summary' to create a summary based on judge notes and project description."
                            }
                        }
                    }
                }

                // Your Notes section
                if let Some(notes_data) = my_notes {
                    if notes_data.visited {
                        div { class: "mb-4",
                            label { class: "block text-sm font-medium text-foreground-neutral-primary mb-1",
                                "Your Notes"
                            }
                            div { class: "bg-background-neutral-secondary rounded-lg p-4",
                                if let Some(notes) = &notes_data.notes {
                                    p { class: "text-sm text-foreground-neutral-secondary",
                                        "{notes}"
                                    }
                                } else {
                                    p { class: "text-sm text-foreground-neutral-tertiary italic",
                                        "No notes recorded"
                                    }
                                }
                            }
                        }
                    } else {
                        div { class: "mb-4",
                            p { class: "text-sm text-foreground-neutral-tertiary italic",
                                "You have not judged this project"
                            }
                        }
                    }
                }

                // Ranking info
                div { class: "pt-4 border-t border-stroke-neutral-1",
                    div { class: "flex items-center gap-4",
                        span { class: "text-sm font-medium text-foreground-neutral-primary",
                            "Overall Rank: #"
                            "{project.rank}"
                        }
                        span { class: "text-sm text-foreground-neutral-secondary",
                            "Score: "
                            {
                                project
                                    .weighted_score
                                    .map(|s| format!("{:.2}", s))
                                    .unwrap_or_else(|| "-".to_string())
                            }
                        }
                    }
                }
            }
        }
    }
}
