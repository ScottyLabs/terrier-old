use dioxus::prelude::*;
use dioxus_free_icons::{
    Icon,
    icons::ld_icons::{
        LdArrowUpDown, LdChevronDown, LdChevronUp, LdClipboard, LdLoader, LdSearch, LdSparkles,
    },
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

// Sorting types
#[derive(Clone, Copy, PartialEq, Default)]
enum SortColumn {
    #[default]
    Score,
    ProjectName,
    TeamName,
    Table,
    Feature(i32),
}

#[derive(Clone, Copy, PartialEq, Default)]
enum SortDirection {
    Asc,
    #[default]
    Desc,
}

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
            // Header - sticky
            div { class: "sticky top-0 z-10",
                h1 { class: "text-[30px] font-semibold leading-[38px] text-foreground-neutral-primary pt-11 pb-7",
                    "Results"
                }
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
    // Sorting state
    let mut sort_column: Signal<SortColumn> = use_signal(SortColumn::default);
    let mut sort_direction: Signal<SortDirection> = use_signal(SortDirection::default);

    // Search state
    let mut search_query: Signal<String> = use_signal(String::new);

    // Filter and sort projects
    let filtered_projects: Vec<ProjectResultInfo> = {
        let query = search_query().to_lowercase();
        let mut projects: Vec<_> = results
            .projects
            .iter()
            .filter(|p| {
                if query.is_empty() {
                    true
                } else {
                    p.project_name
                        .as_ref()
                        .map(|n| n.to_lowercase().contains(&query))
                        .unwrap_or(false)
                        || p.team_name.to_lowercase().contains(&query)
                }
            })
            .cloned()
            .collect();

        // Sort
        let col = sort_column();
        let dir = sort_direction();
        projects.sort_by(|a, b| {
            let ordering = match col {
                SortColumn::ProjectName => {
                    let a_name = a.project_name.as_deref().unwrap_or("");
                    let b_name = b.project_name.as_deref().unwrap_or("");
                    a_name.to_lowercase().cmp(&b_name.to_lowercase())
                }
                SortColumn::TeamName => a.team_name.to_lowercase().cmp(&b.team_name.to_lowercase()),
                SortColumn::Table => {
                    let a_table = a.table_number.as_deref().unwrap_or("");
                    let b_table = b.table_number.as_deref().unwrap_or("");
                    // Try to parse as numbers for numeric sort
                    match (a_table.parse::<i32>(), b_table.parse::<i32>()) {
                        (Ok(a_num), Ok(b_num)) => a_num.cmp(&b_num),
                        _ => a_table.cmp(b_table),
                    }
                }
                SortColumn::Score => {
                    let a_score = a.weighted_score.unwrap_or(0.0);
                    let b_score = b.weighted_score.unwrap_or(0.0);
                    a_score
                        .partial_cmp(&b_score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }
                SortColumn::Feature(feature_id) => {
                    let a_rank = a
                        .feature_ranks
                        .iter()
                        .find(|r| r.feature_id == feature_id)
                        .and_then(|r| r.rank)
                        .unwrap_or(i32::MAX);
                    let b_rank = b
                        .feature_ranks
                        .iter()
                        .find(|r| r.feature_id == feature_id)
                        .and_then(|r| r.rank)
                        .unwrap_or(i32::MAX);
                    a_rank.cmp(&b_rank)
                }
            };
            match dir {
                SortDirection::Asc => ordering,
                SortDirection::Desc => ordering.reverse(),
            }
        });
        projects
    };

    // Helper component for sortable header
    let render_sort_icon = |col: SortColumn| -> Element {
        let current_col = sort_column();
        let current_dir = sort_direction();
        if current_col == col {
            match current_dir {
                SortDirection::Asc => rsx! {
                    Icon {
                        width: 14,
                        height: 14,
                        icon: LdChevronUp,
                        class: "text-foreground-brand-primary",
                    }
                },
                SortDirection::Desc => rsx! {
                    Icon {
                        width: 14,
                        height: 14,
                        icon: LdChevronDown,
                        class: "text-foreground-brand-primary",
                    }
                },
            }
        } else {
            rsx! {
                Icon {
                    width: 14,
                    height: 14,
                    icon: LdArrowUpDown,
                    class: "text-foreground-neutral-tertiary",
                }
            }
        }
    };

    rsx! {
        div { class: "space-y-4",
            // Search input
            div { class: "h-10 max-w-xs border border-stroke-neutral-1 rounded-full flex items-center px-3 py-1",
                Icon {
                    width: 20,
                    height: 20,
                    icon: LdSearch,
                    class: "text-foreground-neutral-tertiary",
                }
                input {
                    r#type: "text",
                    class: "flex-1 px-2.5 text-sm leading-5 text-foreground-neutral-primary placeholder:text-foreground-neutral-tertiary outline-none bg-transparent",
                    placeholder: "Search projects...",
                    value: "{search_query}",
                    oninput: move |evt| search_query.set(evt.value()),
                }
            }

            // Table
            div { class: "overflow-x-auto",
                table { class: "w-full",
                    thead {
                        tr { class: "border-b border-stroke-neutral-1",
                            th {
                                class: "text-left py-3 px-2 text-sm font-semibold text-foreground-neutral-primary cursor-pointer hover:bg-background-neutral-secondary-enabled select-none",
                                onclick: move |_| {
                                    let col = SortColumn::ProjectName;
                                    if sort_column() == col {
                                        sort_direction
                                            .set(
                                                match sort_direction() {
                                                    SortDirection::Asc => SortDirection::Desc,
                                                    SortDirection::Desc => SortDirection::Asc,
                                                },
                                            );
                                    } else {
                                        sort_column.set(col);
                                        sort_direction.set(SortDirection::Desc);
                                    }
                                },
                                div { class: "flex items-center gap-1",
                                    "Project Name"
                                    {render_sort_icon(SortColumn::ProjectName)}
                                }
                            }
                            th {
                                class: "text-left py-3 px-2 text-sm font-semibold text-foreground-neutral-primary cursor-pointer hover:bg-background-neutral-secondary-enabled select-none",
                                onclick: move |_| {
                                    let col = SortColumn::TeamName;
                                    if sort_column() == col {
                                        sort_direction
                                            .set(
                                                match sort_direction() {
                                                    SortDirection::Asc => SortDirection::Desc,
                                                    SortDirection::Desc => SortDirection::Asc,
                                                },
                                            );
                                    } else {
                                        sort_column.set(col);
                                        sort_direction.set(SortDirection::Desc);
                                    }
                                },
                                div { class: "flex items-center gap-1",
                                    "Team Name"
                                    {render_sort_icon(SortColumn::TeamName)}
                                }
                            }
                            th {
                                class: "text-left py-3 px-2 text-sm font-semibold text-foreground-neutral-primary cursor-pointer hover:bg-background-neutral-secondary-enabled select-none",
                                onclick: move |_| {
                                    let col = SortColumn::Table;
                                    if sort_column() == col {
                                        sort_direction
                                            .set(
                                                match sort_direction() {
                                                    SortDirection::Asc => SortDirection::Desc,
                                                    SortDirection::Desc => SortDirection::Asc,
                                                },
                                            );
                                    } else {
                                        sort_column.set(col);
                                        sort_direction.set(SortDirection::Desc);
                                    }
                                },
                                div { class: "flex items-center gap-1",
                                    "Table"
                                    {render_sort_icon(SortColumn::Table)}
                                }
                            }
                            th {
                                class: "text-left py-3 px-2 text-sm font-semibold text-foreground-neutral-primary cursor-pointer hover:bg-background-neutral-secondary-enabled select-none",
                                onclick: move |_| {
                                    let col = SortColumn::Score;
                                    if sort_column() == col {
                                        sort_direction
                                            .set(
                                                match sort_direction() {
                                                    SortDirection::Asc => SortDirection::Desc,
                                                    SortDirection::Desc => SortDirection::Asc,
                                                },
                                            );
                                    } else {
                                        sort_column.set(col);
                                        sort_direction.set(SortDirection::Desc);
                                    }
                                },
                                div { class: "flex items-center gap-1",
                                    "Score"
                                    {render_sort_icon(SortColumn::Score)}
                                }
                            }
                            // Dynamic feature columns
                            for feature in results.features.iter() {
                                {
                                    let feature_id = feature.id;
                                    let feature_name = feature.name.clone();
                                    rsx! {
                                        th {
                                            key: "{feature.id}",
                                            class: "text-left py-3 px-2 text-sm font-semibold text-foreground-neutral-primary cursor-pointer hover:bg-background-neutral-secondary-enabled select-none",
                                            onclick: move |_| {
                                                let col = SortColumn::Feature(feature_id);
                                                if sort_column() == col {
                                                    sort_direction
                                                        .set(
                                                            match sort_direction() {
                                                                SortDirection::Asc => SortDirection::Desc,
                                                                SortDirection::Desc => SortDirection::Asc,
                                                            },
                                                        );
                                                } else {
                                                    sort_column.set(col);
                                                    sort_direction.set(SortDirection::Desc);
                                                }
                                            },
                                            div { class: "flex items-center gap-1",
                                                "{feature_name}"
                                                {render_sort_icon(SortColumn::Feature(feature_id))}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    tbody {
                        if filtered_projects.is_empty() {
                            tr {
                                td {
                                    colspan: "{4 + results.features.len()}",
                                    class: "text-center py-8 text-foreground-neutral-secondary",
                                    if search_query().is_empty() {
                                        "No submissions yet"
                                    } else {
                                        "No projects match your search"
                                    }
                                }
                            }
                        } else {
                            for project in filtered_projects.iter() {
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
                    {
                        let project_name_copy = project
                            .project_name
                            .clone()
                            .unwrap_or_else(|| "Untitled Project".to_string());
                        let team_name_copy = project.team_name.clone();
                        let table_copy = project.table_number.clone().unwrap_or_else(|| "-".to_string());
                        let score_copy = project
                            .weighted_score
                            .map(|s| format!("{:.2}", s))
                            .unwrap_or_else(|| "-".to_string());
                        let desc_copy = project
                            .description
                            .clone()
                            .unwrap_or_else(|| "No description".to_string());
                        rsx! {
                            button {
                                class: "p-2 text-foreground-neutral-tertiary hover:text-foreground-neutral-primary transition-colors",
                                title: "Copy project info",
                                onclick: move |_| {
                                    let text = format!(
                                        "Project: {}\nTeam: {}\nTable: {}\nScore: {}\n\n{}",
                                        project_name_copy,
                                        team_name_copy,
                                        table_copy,
                                        score_copy,
                                        desc_copy,
                                    );
                                    let _ = document::eval(
                                        &format!(
                                            "navigator.clipboard.writeText({});",
                                            serde_json::to_string(&text).unwrap_or_default(),
                                        ),
                                    );
                                },
                                Icon { width: 20, height: 20, icon: LdClipboard }
                            }
                        }
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
                                rel: "noopener noreferrer",
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
                                rel: "noopener noreferrer",
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
                                rel: "noopener noreferrer",
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
