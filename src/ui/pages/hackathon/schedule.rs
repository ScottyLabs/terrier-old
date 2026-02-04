use chrono::{Duration, NaiveDate, NaiveDateTime, Timelike};
use dioxus::prelude::*;
use dioxus_free_icons::{
    Icon,
    icons::ld_icons::{LdClock, LdMapPin, LdPlus, LdTarget},
};

use crate::{
    Route,
    auth::{
        HackathonRole, HackathonRoleType, SCHEDULE_ROLES, hooks::use_require_access_or_redirect,
    },
    domain::{
        applications::handlers::get_user_schedule,
        hackathons::types::{HackathonInfo, ScheduleEvent},
    },
    ui::features::schedule::{EventDetailModal, EventModal},
};

/// Height of one hour in pixels
const HOUR_HEIGHT: f64 = 60.0;

/// Schedule display hours (0 = midnight, 23 = 11pm)
const START_HOUR: u32 = 0;
const END_HOUR: u32 = 24;

#[component]
pub fn HackathonSchedule(slug: String) -> Element {
    if let Some(no_access) = use_require_access_or_redirect(SCHEDULE_ROLES) {
        return no_access;
    }

    // Mobile detection
    let is_mobile = use_context::<Signal<bool>>();

    // Clone slug for different closures
    let slug_for_resource = slug.clone();
    let slug_for_modal = slug.clone();
    let slug_for_mobile = slug.clone();

    let hackathon = use_context::<Signal<HackathonInfo>>();

    // Get user's role from context
    let user_role = use_context::<Option<HackathonRole>>();
    let is_admin_or_organizer = user_role
        .as_ref()
        .and_then(|r| r.role_type())
        .map(|rt| rt == HackathonRoleType::Admin || rt == HackathonRoleType::Organizer)
        .unwrap_or(false);

    // Get current user ID for event highlighting
    let current_user_id = user_role.as_ref().map(|r| r.user_id);

    // Modal state - None for create, Some(event) for edit
    let mut editing_event = use_signal(|| None::<ScheduleEvent>);
    let mut show_modal = use_signal(|| false);

    // View-only modal state for non-admin event viewing
    let mut viewing_event = use_signal(|| None::<ScheduleEvent>);

    // Fetch schedule events
    let mut schedule_resource = use_resource(move || {
        let slug = slug_for_resource.clone();
        async move {
            let result: Result<Vec<ScheduleEvent>, _> = get_user_schedule(slug).await;
            result.ok()
        }
    });

    // Calculate hackathon days
    let hackathon_days = {
        let h = hackathon.read();
        let start = h.start_date.date();
        let end = h.end_date.date();
        get_days_between(start, end)
    };

    // Selected day for mobile view (default to first day)
    let mut selected_day = use_signal(|| {
        hackathon_days
            .first()
            .cloned()
            .unwrap_or_else(|| chrono::Local::now().date_naive())
    });

    // Get current time for "Current" event highlighting
    let now = chrono::Local::now().naive_local();

    // Categorize events
    let events = schedule_resource.read();
    let (current_events, upcoming_events, past_events) =
        categorize_events(events.as_ref().and_then(|e| e.as_ref()), now);

    // Mobile view
    if *is_mobile.read() {
        return rsx! {
            MobileScheduleView {
                slug: slug_for_mobile,
                hackathon_days: hackathon_days.clone(),
                events: events.as_ref().and_then(|e| e.as_ref()).cloned().unwrap_or_default(),
                selected_day,
                is_admin: is_admin_or_organizer,
            }
        };
    }

    rsx! {
        div { class: "flex flex-col gap-6 h-full overflow-hidden lg:flex-row",
            div { class: "flex-1 min-w-0 flex flex-col overflow-hidden",
                // Header with title and add button
                div { class: "flex items-center justify-between pt-11 pb-7",
                    h1 { class: "text-[30px] font-semibold leading-[38px] text-foreground-neutral-primary",
                        "Schedule"
                    }
                    if is_admin_or_organizer {
                        button {
                            class: "flex items-center gap-2 bg-foreground-neutral-primary text-white font-semibold text-sm leading-5 rounded-full px-4 py-2.5",
                            onclick: move |_| {
                                editing_event.set(None);
                                show_modal.set(true);
                            },
                            Icon {
                                width: 16,
                                height: 16,
                                icon: LdPlus,
                                class: "text-white",
                            }
                            "Add new schedule"
                        }
                    }
                }

                // Calendar grid
                div { class: "bg-background-neutral-primary rounded-[20px] flex-1 overflow-auto",
                    // Check if there are any events
                    {
                        let has_events = events
                            .as_ref()
                            .and_then(|e| e.as_ref())
                            .map(|e| !e.is_empty())
                            .unwrap_or(false);
                        if has_events {
                            rsx! {
                                div { class: "flex min-w-max",
                                    div { class: "w-24 flex-shrink-0",
                                        // Sticky header spacer (matches day header height)
                                        div { class: "h-[68px] sticky top-0 bg-background-neutral-primary z-20" }
                                        for hour in START_HOUR..END_HOUR {
                                            div {
                                                class: "h-[60px] text-[14px] text-black pr-2 text-right",
                                                style: "line-height: 60px;",
                                                "{format_hour(hour)}"
                                            }
                                        }
                                    }
                                    for day in hackathon_days.iter() {
                                        DayColumn {
                                            day: *day,
                                            events: events.as_ref().and_then(|e| e.as_ref()).cloned().unwrap_or_default(),
                                            current_user_id,
                                            is_admin: is_admin_or_organizer,
                                            now,
                                            on_click: move |event: ScheduleEvent| {
                                                viewing_event.set(Some(event));
                                            },
                                        }
                                    }
                                }
                            }
                        } else {
                            rsx! {
                                div { class: "flex flex-col items-center justify-center h-full min-h-[400px] text-center",
                                    div { class: "text-6xl mb-4", "📅" }
                                    h2 { class: "text-xl font-semibold text-foreground-neutral-primary mb-2", "No events yet" }
                                    p { class: "text-foreground-neutral-secondary mb-6",
                                        "Events will appear here once they're added to the schedule."
                                    }
                                    if is_admin_or_organizer {
                                        button {
                                            class: "flex items-center gap-2 bg-foreground-neutral-primary text-white font-semibold text-sm rounded-full px-4 py-2.5",
                                            onclick: move |_| {
                                                editing_event.set(None);
                                                show_modal.set(true);
                                            },
                                            Icon {
                                                width: 16,
                                                height: 16,
                                                icon: LdPlus,
                                                class: "text-white",
                                            }
                                            "Add first event"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Sidebar with event details
            div { class: "w-full lg:w-80 flex-shrink-0 overflow-auto",
                div { class: "pt-11",
                    // Current events
                    EventSection {
                        title: "Current",
                        events: current_events,
                        highlight: true,
                    }

                    // Upcoming events
                    EventSection {
                        title: "Upcoming",
                        events: upcoming_events,
                        highlight: false,
                    }

                    // Past events
                    EventSection {
                        title: "Past",
                        events: past_events,
                        highlight: false,
                    }
                }
            }
        }

        // Add/Edit Event Modal
        if show_modal() {
            EventModal {
                slug: slug_for_modal.clone(),
                event: editing_event(),
                hackathon_start_date: hackathon.read().start_date.date(),
                hackathon_end_date: hackathon.read().end_date.date(),
                on_close: move |_| {
                    show_modal.set(false);
                    editing_event.set(None);
                },
                on_save: move |_| {
                    show_modal.set(false);
                    editing_event.set(None);
                    schedule_resource.restart();
                },
            }
        }

        // Event Detail Modal (view-only, with Edit for admins)
        if let Some(event) = viewing_event() {
            EventDetailModal {
                slug: slug_for_modal.clone(),
                event: event.clone(),
                is_admin: is_admin_or_organizer,
                on_close: move |_| {
                    viewing_event.set(None);
                },
                on_edit: move |_| {
                    // Switch to edit mode
                    viewing_event.set(None);
                    editing_event.set(Some(event.clone()));
                    show_modal.set(true);
                },
            }
        }
    }
}

#[component]
fn DayColumn(
    day: NaiveDate,
    events: Vec<ScheduleEvent>,
    current_user_id: Option<i32>,
    is_admin: bool,
    now: NaiveDateTime,
    on_click: EventHandler<ScheduleEvent>,
) -> Element {
    let day_name = day.format("%a").to_string();
    let day_num = day.format("%d").to_string();

    // Filter events for this day - include events that span multiple days
    let day_events: Vec<_> = events
        .iter()
        .filter(|e| {
            let start_date = e.start_time.date();
            let end_date = e.end_time.date();
            // Event is visible on this day if day is between start and end (inclusive)
            start_date <= day && day <= end_date
        })
        .cloned()
        .collect();

    // Compute layout positions for overlapping events
    let event_layouts = compute_event_layout(day_events, day);

    // Calculate if current time indicator should show on this day
    let is_today = now.date() == day;
    let now_indicator_top = if is_today {
        let hours_from_start = now.hour() as f64 - START_HOUR as f64;
        let minutes_fraction = now.minute() as f64 / 60.0;
        Some((hours_from_start + minutes_fraction) * HOUR_HEIGHT)
    } else {
        None
    };

    rsx! {
        div { class: "flex-1 min-w-[150px] border-stroke-neutral-1",
            // Day header
            div { class: "text-center pt-[28px] pb-[20px] sticky top-0 bg-background-neutral-primary z-20",
                span { class: "text-[18px] font-medium text-foreground-neutral-primary",
                    "{day_name} {day_num}"
                }
            }

            // Time slots with events
            div { class: "relative top-[20px]",
                // Hour grid lines
                for hour in START_HOUR..END_HOUR {
                    div { class: "pt-[30px] mb-[29px] border-t border-stroke-neutral-1" }
                }

                // Current time indicator
                if let Some(top) = now_indicator_top {
                    div {
                        class: "absolute left-0 right-0 flex items-center z-10",
                        style: "top: {top}px;",
                        // Dot on the left
                        div { class: "w-3 h-3 rounded-full bg-foreground-neutral-primary -ml-1.5" }
                        // Line across
                        div { class: "flex-1 h-0.5 bg-foreground-neutral-primary" }
                    }
                }

                // Events positioned absolutely
                for layout in event_layouts.iter() {
                    EventBlock {
                        event: layout.event.clone(),
                        day,
                        column: layout.column,
                        total_columns: layout.total_columns,
                        current_user_id,
                        is_admin,
                        on_click,
                    }
                }
            }
        }
    }
}

#[component]
fn EventBlock(
    event: ScheduleEvent,
    day: NaiveDate,
    column: usize,
    total_columns: usize,
    current_user_id: Option<i32>,
    #[allow(unused)] is_admin: bool,
    on_click: EventHandler<ScheduleEvent>,
) -> Element {
    let event_start_date = event.start_time.date();
    let event_end_date = event.end_time.date();

    // Calculate display hours based on which day we're rendering
    let display_start_hour = if day == event_start_date {
        // First day: use actual start time
        event.start_time.hour() as f64 + event.start_time.minute() as f64 / 60.0
    } else {
        // Middle/last day: start at beginning of day
        START_HOUR as f64
    };

    let display_end_hour = if day == event_end_date {
        // Last day: use actual end time
        event.end_time.hour() as f64 + event.end_time.minute() as f64 / 60.0
    } else {
        // First/middle day: end at end of day
        END_HOUR as f64
    };

    let top = (display_start_hour - START_HOUR as f64) * HOUR_HEIGHT;
    let height = (display_end_hour - display_start_hour) * HOUR_HEIGHT;

    // Check if current user is an organizer of this event
    let is_my_event = current_user_id
        .map(|uid| event.organizer_ids.contains(&uid))
        .unwrap_or(false);

    // Color coding based on event_type
    // If user is an organizer, use bright vibrant colors (full opacity)
    // Otherwise, use softer pastel colors
    let bg_color = if is_my_event {
        // Bright, vibrant colors for user's own events
        match event.event_type.as_str() {
            "hacking" => "bg-blue-600",
            "speaker" => "bg-purple-600",
            "sponsor" => "bg-amber-500",
            "food" => "bg-orange-500",
            _ => "bg-green-600", // default
        }
    } else {
        // Soft pastel colors for other events
        match event.event_type.as_str() {
            "hacking" => "bg-blue-50",
            "speaker" => "bg-purple-50",
            "sponsor" => "bg-amber-50",
            "food" => "bg-orange-50",
            _ => "bg-green-50", // default
        }
    };

    // All events are clickable
    let cursor_class = "cursor-pointer hover:opacity-80";

    // Format time simply: "5 - 6pm", if same am/pm, otherwise "5am - 6pm"
    // check if same am/pm
    let am_pm = event.start_time.format("%P").to_string();
    let end_am_pm = event.end_time.format("%P").to_string();
    let time_str = if am_pm == end_am_pm {
        format!(
            "{} - {}",
            event.start_time.format("%-I:%M"),
            event.end_time.format("%-I:%M%P")
        )
    } else {
        format!(
            "{} - {}",
            event.start_time.format("%-I:%M%P"),
            event.end_time.format("%-I:%M%P")
        )
    };

    let event_for_click = event.clone();

    // Calculate width and position based on overlap
    let (left_percent, width_percent) = match total_columns {
        1 => (1.0, 98.0), // Full width with small margin
        2 => {
            // Side by side: each gets ~46% width with 8% total gap
            let width = 46.0;
            let left = column as f64 * 52.0 + 1.0;
            (left, width)
        }
        n => {
            // 3+ events: use minimum width with slight overlap
            let min_width = 32.0;
            let available_width = 98.0;
            // Distribute columns with overlap
            let step = (available_width - min_width) / (n as f64 - 1.0).max(1.0);
            let left = column as f64 * step + 1.0;
            (left, min_width)
        }
    };

    // Z-index: later columns appear on top
    let z_index = column + 1;

    rsx! {
        div {
            class: "absolute flex flex-col gap-1.5 rounded-xl p-3 overflow-hidden {bg_color} {cursor_class}",
            style: "top: {top}px; height: {height}px; left: {left_percent}%; width: {width_percent}%; z-index: {z_index};",
            onclick: move |_| {
                on_click.call(event_for_click.clone());
            },
            p { class: "text-base font-medium text-foreground-neutral-primary truncate",
                "{event.name}"
            }
            p { class: "text-base text-foreground-neutral-primary truncate", "{time_str}" }
        }
    }
}

#[component]
fn EventSection(title: String, events: Vec<ScheduleEvent>, highlight: bool) -> Element {
    if events.is_empty() {
        return rsx! {};
    }

    rsx! {
        div { class: "mb-6",
            h2 { class: "text-lg font-semibold text-[var(--color-foreground-neutral-primary)] mb-4",
                "{title}"
            }
            for event in events {
                EventCard { event, highlight }
            }
        }
    }
}

#[component]
fn EventCard(event: ScheduleEvent, highlight: bool) -> Element {
    let border_class = if highlight {
        "border-l-4 border-green-500"
    } else {
        ""
    };

    let time_str = format!(
        "{} · {} – {}",
        event.start_time.format("%A, %B %d"),
        event.start_time.format("%-I:%M"),
        event.end_time.format("%-I:%M%P")
    );

    // Get event type display name for category badge
    let event_type_display = match event.event_type.as_str() {
        "hacking" => "Hacking",
        "speaker" => "Speaker",
        "sponsor" => "Sponsor",
        "food" => "Food",
        _ => "Event",
    };

    rsx! {
        div { class: "bg-background-neutral-primary rounded-xl p-4 mb-3 {border_class}",
            // Title and category badge
            div { class: "flex items-start justify-between gap-3 mb-3",
                h3 { class: "text-base font-semibold text-foreground-neutral-primary",
                    "{event.name}"
                }
                span { class: "text-xs border border-stroke-neutral-1 text-foreground-neutral-secondary px-2.5 py-1 rounded-full whitespace-nowrap",
                    "{event_type_display}"
                }
            }

            // Location
            if let Some(loc) = &event.location {
                div { class: "flex items-center gap-2 text-sm text-foreground-neutral-secondary mb-1.5",
                    Icon { width: 14, height: 14, icon: LdMapPin }
                    span { "{loc}" }
                }
            }

            // Time
            div { class: "flex items-center gap-2 text-sm text-foreground-neutral-secondary mb-1.5",
                Icon { width: 14, height: 14, icon: LdClock }
                span { "{time_str}" }
            }

            // Points (only show if set)
            if let Some(pts) = event.points {
                div { class: "flex items-center gap-2 text-sm text-foreground-neutral-secondary mb-3",
                    Icon { width: 14, height: 14, icon: LdTarget }
                    span { "{pts} Points" }
                }
            }

            // Description
            if let Some(desc) = &event.description {
                p { class: "text-sm text-foreground-neutral-tertiary leading-relaxed",
                    "{desc}"
                }
            }
        }
    }
}

fn get_days_between(start: NaiveDate, end: NaiveDate) -> Vec<NaiveDate> {
    let mut days = Vec::new();
    let mut current = start;
    while current <= end {
        days.push(current);
        current += Duration::days(1);
    }
    days
}

fn categorize_events(
    events: Option<&Vec<ScheduleEvent>>,
    now: NaiveDateTime,
) -> (Vec<ScheduleEvent>, Vec<ScheduleEvent>, Vec<ScheduleEvent>) {
    let events = match events {
        Some(e) => e,
        None => return (vec![], vec![], vec![]),
    };

    let mut current = vec![];
    let mut upcoming = vec![];
    let mut past = vec![];

    for event in events {
        if event.start_time <= now && event.end_time >= now {
            current.push(event.clone());
        } else if event.start_time > now {
            upcoming.push(event.clone());
        } else {
            past.push(event.clone());
        }
    }

    (current, upcoming, past)
}

fn format_hour(hour: u32) -> String {
    match hour {
        0 => "12 AM".to_string(),
        1..=11 => format!("{} AM", hour),
        12 => "12 PM".to_string(),
        13..=23 => format!("{} PM", hour - 12),
        _ => format!("{}", hour),
    }
}

/// Represents an event's layout position within overlapping groups
struct EventLayout {
    event: ScheduleEvent,
    /// Column index (0-indexed) within the overlap group
    column: usize,
    /// Total number of columns in this overlap group
    total_columns: usize,
}

/// Compute layout positions for overlapping events using greedy column packing
fn compute_event_layout(events: Vec<ScheduleEvent>, day: NaiveDate) -> Vec<EventLayout> {
    if events.is_empty() {
        return vec![];
    }

    // Helper to calculate effective start/end hours for an event on a specific day
    let effective_hours = |event: &ScheduleEvent| -> (f64, f64) {
        let event_start_date = event.start_time.date();
        let event_end_date = event.end_time.date();

        let start_hour = if day == event_start_date {
            event.start_time.hour() as f64 + event.start_time.minute() as f64 / 60.0
        } else {
            // Multi-day event: starts at beginning of this day
            START_HOUR as f64
        };

        let end_hour = if day == event_end_date {
            event.end_time.hour() as f64 + event.end_time.minute() as f64 / 60.0
        } else {
            // Multi-day event: ends at end of this day
            END_HOUR as f64
        };

        (start_hour, end_hour)
    };

    // Sort events by their effective start time for THIS day, then by effective end time
    let mut sorted_events = events;
    sorted_events.sort_by(|a, b| {
        let (a_start, a_end) = effective_hours(a);
        let (b_start, b_end) = effective_hours(b);
        a_start
            .partial_cmp(&b_start)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a_end
                    .partial_cmp(&b_end)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });

    // Track column end times (when each column becomes free)
    // Value is the end hour as f64 for this day
    let mut column_ends: Vec<f64> = Vec::new();
    let mut assignments: Vec<(ScheduleEvent, usize, f64, f64)> = Vec::new(); // (event, column, start, end)

    for event in sorted_events {
        let (start_hour, end_hour) = effective_hours(&event);

        // Find the first column where this event fits
        let mut assigned_column = None;
        for (col_idx, col_end) in column_ends.iter().enumerate() {
            if *col_end <= start_hour {
                assigned_column = Some(col_idx);
                break;
            }
        }

        let column = match assigned_column {
            Some(col) => {
                column_ends[col] = end_hour;
                col
            }
            None => {
                // No available column, create a new one
                column_ends.push(end_hour);
                column_ends.len() - 1
            }
        };

        assignments.push((event, column, start_hour, end_hour));
    }

    // Now we need to determine total_columns for each event
    // An event's total_columns is the max columns among all events it overlaps with
    let mut layouts: Vec<EventLayout> = Vec::new();

    for (event, column, start_hour, end_hour) in &assignments {
        // Find all events that overlap with this one and get max column + 1
        let mut max_column = *column;
        for (_, other_column, other_start, other_end) in &assignments {
            // Check if they overlap (start < other_end && other_start < end)
            if start_hour < other_end && other_start < end_hour {
                max_column = max_column.max(*other_column);
            }
        }

        layouts.push(EventLayout {
            event: event.clone(),
            column: *column,
            total_columns: max_column + 1,
        });
    }

    layouts
}

/// Mobile schedule view with day tabs and event list
#[component]
fn MobileScheduleView(
    slug: String,
    hackathon_days: Vec<NaiveDate>,
    events: Vec<ScheduleEvent>,
    selected_day: Signal<NaiveDate>,
    is_admin: bool,
) -> Element {
    let nav = use_navigator();
    let slug_for_nav = slug.clone();

    // Filter events for selected day
    let selected = *selected_day.read();
    let mut day_events: Vec<_> = events
        .iter()
        .filter(|e| {
            let start_date = e.start_time.date();
            let end_date = e.end_time.date();
            start_date <= selected && selected <= end_date
        })
        .cloned()
        .collect();

    // Sort by start time
    day_events.sort_by_key(|e| e.start_time);

    // Group events by hour
    let mut grouped: std::collections::BTreeMap<u32, Vec<ScheduleEvent>> =
        std::collections::BTreeMap::new();
    for event in day_events {
        let hour = event.start_time.hour();
        grouped.entry(hour).or_default().push(event);
    }

    rsx! {
        div { class: "min-h-screen bg-background-neutral-secondary",
            // Schedule title
            div { class: "px-4 pt-6 pb-4",
                h1 { class: "text-2xl font-bold text-foreground-neutral-primary mb-5", "Schedule" }

                // Day tabs
                div { class: "flex gap-2",
                    for day in hackathon_days.iter() {
                        MobileDayTab {
                            day: *day,
                            is_selected: *day == selected,
                            on_select: move |d| selected_day.set(d),
                        }
                    }
                }
            }

            // Events list grouped by hour
            div { class: "px-4 pb-8",
                if grouped.is_empty() {
                    div { class: "text-center py-12",
                        p { class: "text-foreground-neutral-tertiary", "No events scheduled for this day" }
                    }
                } else {
                    for (hour , hour_events) in grouped.iter() {
                        // Time group with separator
                        div { class: "flex items-stretch",
                            // Time label column
                            div { class: "w-16 flex-shrink-0 pr-1",
                                // Time text
                                div { class: "text-[18px] text-black pt-4", "{format_hour(*hour)}" }
                            }
                            // Separator line + events column
                            div { class: "flex-1 border-l border-stroke-neutral-2 pl-4",
                                // Events for this hour
                                div { class: "space-y-3 py-3",
                                    for event in hour_events.iter() {
                                        MobileEventCard {
                                            event: event.clone(),
                                            slug: slug_for_nav.clone(),
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

/// Mobile day tab button
#[component]
fn MobileDayTab(day: NaiveDate, is_selected: bool, on_select: EventHandler<NaiveDate>) -> Element {
    let day_name = day.format("%a").to_string();
    let day_num = day.format("%d").to_string();

    let (bg_class, text_class) = if is_selected {
        (
            "bg-foreground-neutral-primary",
            "text-background-neutral-primary",
        )
    } else {
        ("bg-transparent", "text-foreground-neutral-primary")
    };

    rsx! {
        button {
            class: "flex-shrink-0 px-4 py-2 rounded-full text-sm font-medium transition-colors {bg_class} {text_class}",
            onclick: move |_| on_select.call(day),
            "{day_name} {day_num}"
        }
    }
}

/// Mobile event card
#[component]
fn MobileEventCard(event: ScheduleEvent, slug: String) -> Element {
    let nav = use_navigator();
    let event_id = event.id;
    let slug_for_nav = slug.clone();

    // Format location like Figma shows: "UC 1 | McConomy"
    let location_display = event.location.clone().unwrap_or_default();

    rsx! {
        button {
            class: "w-full text-left bg-background-neutral-primary rounded-2xl p-4",
            onclick: move |_| {
                nav.push(Route::HackathonScheduleEvent {
                    slug: slug_for_nav.clone(),
                    event_id,
                });
            },
            h3 { class: "font-semibold text-foreground-neutral-primary text-base", "{event.name}" }
            if !location_display.is_empty() {
                p { class: "text-sm text-foreground-neutral-tertiary mt-0.5", "{location_display}" }
            }
        }
    }
}
