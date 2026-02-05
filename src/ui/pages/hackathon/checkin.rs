use chrono::NaiveDateTime;
use dioxus::prelude::*;
use dioxus_free_icons::{
    Icon,
    icons::ld_icons::{
        LdCalendar, LdCheck, LdChevronLeft, LdExpand, LdLoader, LdLock, LdQrCode, LdSearch,
        LdSquare, LdStar, LdTarget,
    },
};

use crate::{
    Route,
    auth::{
        CHECKIN_ROLES, HackathonRole, HackathonRoleType, hooks::use_require_access_or_redirect,
    },
    domain::applications::handlers::{
        Attendee, ParticipantInfo, UserPoints, get_attendees, get_participant_info,
        get_user_points, get_user_schedule, organizer_checkin, organizer_remove_checkin,
        remove_self_checkin, self_checkin,
    },
    domain::hackathons::types::{HackathonInfo, ScheduleEvent},
    ui::features::checkin::QRScannerModal,
    ui::features::dashboard::QRModal,
    ui::foundation::utils::generate_qr_svg,
};

#[component]
pub fn HackathonCheckin(slug: String) -> Element {
    if let Some(no_access) = use_require_access_or_redirect(CHECKIN_ROLES) {
        return no_access;
    }

    let slug_for_resource = slug.clone();
    let slug_for_points = slug.clone();

    let _hackathon = use_context::<Signal<HackathonInfo>>();

    // Get user's role from context
    let user_role = use_context::<Signal<Option<HackathonRole>>>();
    let is_organizer_or_admin = user_role
        .read()
        .as_ref()
        .and_then(|r| r.role_type())
        .map(|rt| rt == HackathonRoleType::Admin || rt == HackathonRoleType::Organizer)
        .unwrap_or(false);

    // Fetch schedule events
    let mut schedule_resource = use_resource(move || {
        let slug = slug_for_resource.clone();
        async move {
            let result: Result<Vec<ScheduleEvent>, _> = get_user_schedule(slug).await;
            result.ok()
        }
    });

    // Fetch user points
    let points_resource = use_resource(move || {
        let slug = slug_for_points.clone();
        async move {
            let result: Result<UserPoints, _> = get_user_points(slug).await;
            result.ok()
        }
    });

    // Get events from resource
    let events = schedule_resource
        .read()
        .as_ref()
        .and_then(|e| e.as_ref())
        .cloned()
        .unwrap_or_default();

    let points = points_resource
        .read()
        .as_ref()
        .and_then(|p| p.as_ref())
        .map(|p| p.total_points)
        .unwrap_or(0);

    if is_organizer_or_admin {
        rsx! {
            OrganizerCheckinView {
                slug: slug.clone(),
                events,
                on_refresh: move |_| schedule_resource.restart(),
            }
        }
    } else {
        rsx! {
            ParticipantCheckinView {
                slug: slug.clone(),
                events,
                points,
                on_refresh: move |_| schedule_resource.restart(),
            }
        }
    }
}

/// Participant view - shows check-in list and QR code
#[component]
fn ParticipantCheckinView(
    slug: String,
    events: Vec<ScheduleEvent>,
    points: i32,
    on_refresh: EventHandler<()>,
) -> Element {
    let user_role = use_context::<Signal<Option<HackathonRole>>>();
    let is_mobile = use_context::<Signal<bool>>();
    let user_id = user_role.read().as_ref().map(|r| r.user_id).unwrap_or(-1);
    // Use HTTPS URL for Universal Links (works with iOS camera app)
    let checkin_url = format!("https://terrier.scottylabs.org/h/{}/scan/{}", slug, user_id);
    let qr_svg = generate_qr_svg(&checkin_url);
    let mut show_qr_modal = use_signal(|| false);

    let mut search_query = use_signal(|| String::new());

    // Filter events by search query
    let filtered_events: Vec<ScheduleEvent> = events
        .iter()
        .filter(|e| {
            let query = search_query().to_lowercase();
            query.is_empty() || e.name.to_lowercase().contains(&query)
        })
        .cloned()
        .collect();

    rsx! {
        div { class: "overflow-hidden h-full flex flex-col",
            h1 { class: "text-[30px] font-semibold leading-[38px] text-foreground-neutral-primary pt-11 pb-7",
                "Event Check-In"
            }

            if *is_mobile.read() {
                // Mobile layout
                div { class: "flex-1 overflow-y-auto pb-7",
                    // Points and QR Code buttons
                    div { class: "flex gap-3 mb-6",
                        button { class: "flex-1 flex items-center justify-center gap-2 px-4 py-3 bg-background-neutral-primary rounded-2xl",
                            Icon { width: 18, height: 18, icon: LdTarget }
                            span { class: "font-medium", "Points earned: {points}" }
                        }
                        button {
                            class: "flex-1 flex items-center justify-center gap-2 px-4 py-3 bg-background-neutral-primary rounded-2xl",
                            onclick: move |_| show_qr_modal.set(true),
                            span { class: "font-medium", "QR Code" }
                        }
                    }

                    // Check-ins section
                    div { class: "bg-background-neutral-primary rounded-[20px] p-5",
                        p { class: "text-heading/5 font-semibold leading-[28px] text-foreground-neutral-primary mb-4",
                            "Check-ins"
                        }
                        div { class: "flex flex-col gap-3",
                            for event in filtered_events.iter() {
                                {
                                    let event_clone = event.clone();
                                    let slug_for_action = slug.clone();
                                    let refresh = on_refresh.clone();
                                    let event_id = event.id;
                                    let is_self_checkin = event.checkin_type == "self_checkin";
                                    let is_checked_in = event.is_checked_in;
                                    rsx! {
                                        EventCard {
                                            event: event_clone.clone(),
                                            on_click: move |_: ScheduleEvent| {
                                                if is_self_checkin {
                                                    let slug = slug_for_action.clone();
                                                    let refresh = refresh.clone();
                                                    spawn(async move {
                                                        if is_checked_in {
                                                            let _ = remove_self_checkin(slug, event_id).await;
                                                        } else {
                                                            let _ = self_checkin(slug, event_id).await;
                                                        }
                                                        refresh.call(());
                                                    });
                                                }
                                            },
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                // Desktop layout
                div { class: "flex flex-col gap-6 flex-1 overflow-hidden lg:flex-row",
                    // Left panel - Event list
                    div { class: "mb-7 bg-background-neutral-primary rounded-[20px] flex-1 p-7 flex flex-col overflow-hidden",
                        div { class: "flex items-center justify-between pb-4",
                            p { class: "text-heading/5 font-semibold leading-[28px] text-foreground-neutral-primary",
                                "Check-ins"
                            }
                        }
                        // Search input
                        div { class: "flex items-center gap-2 px-3 py-2 rounded-xl border border-stroke-neutral-1 bg-background-neutral-secondary mb-4",
                            Icon {
                                width: 16,
                                height: 16,
                                icon: LdSearch,
                                class: "text-foreground-neutral-tertiary",
                            }
                            input {
                                r#type: "text",
                                class: "flex-1 bg-transparent text-foreground-neutral-primary placeholder:text-foreground-neutral-tertiary outline-none text-sm",
                                placeholder: "Search events...",
                                value: "{search_query}",
                                oninput: move |e| search_query.set(e.value()),
                            }
                        }
                        div { class: "flex-1 overflow-y-auto",
                            div { class: "flex flex-col gap-3",
                                for event in filtered_events.iter() {
                                    {
                                        let event_clone = event.clone();
                                        let slug_for_action = slug.clone();
                                        let refresh = on_refresh.clone();
                                        let event_id = event.id;
                                        let is_self_checkin = event.checkin_type == "self_checkin";
                                        let is_checked_in = event.is_checked_in;
                                        rsx! {
                                            EventCard {
                                                event: event_clone.clone(),
                                                on_click: move |_: ScheduleEvent| {
                                                    if is_self_checkin {
                                                        let slug = slug_for_action.clone();
                                                        let refresh = refresh.clone();
                                                        spawn(async move {
                                                            if is_checked_in {
                                                                let _ = remove_self_checkin(slug, event_id).await;
                                                            } else {
                                                                let _ = self_checkin(slug, event_id).await;
                                                            }
                                                            refresh.call(());
                                                        });
                                                    }
                                                },
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Right panel - Points and QR code
                    div { class: "w-full lg:w-80 flex-shrink-0 mb-7",
                        div { class: "bg-background-neutral-primary rounded-[20px] p-6",
                            // Points badge
                            div { class: "flex items-center gap-2 mb-6",
                                Icon {
                                    width: 20,
                                    height: 20,
                                    icon: LdTarget,
                                    class: "text-foreground-neutral-primary",
                                }
                                span { class: "text-lg font-semibold text-foreground-neutral-primary",
                                    "Points earned: {points}"
                                }
                            }

                            // QR code
                            div { class: "mb-4",
                                div { class: "flex items-center justify-between mb-3",
                                    p { class: "text-sm text-foreground-neutral-secondary",
                                        "Your QR code"
                                    }
                                    button {
                                        class: "p-1 hover:bg-background-neutral-secondary rounded transition-colors",
                                        onclick: move |_| show_qr_modal.set(true),
                                        Icon {
                                            width: 16,
                                            height: 16,
                                            icon: LdExpand,
                                            class: "text-foreground-neutral-secondary",
                                        }
                                    }
                                }
                                div {
                                    class: "bg-background-neutral-primary rounded-xl p-4",
                                    dangerous_inner_html: "{qr_svg}",
                                }
                                p { class: "text-xs text-foreground-neutral-tertiary text-center mt-2",
                                    "User ID: {user_id}"
                                }
                            }
                        }
                    }
                }
            }
        }

        // QR Modal
        if show_qr_modal() {
            QRModal {
                qr_svg: qr_svg.clone(),
                user_id,
                on_close: move |_| show_qr_modal.set(false),
            }
        }
    }
}

/// Unified event card component for all views
#[component]
fn EventCard(
    event: ScheduleEvent,
    #[props(default = false)] is_selected: bool,
    on_click: EventHandler<ScheduleEvent>,
) -> Element {
    let is_self_checkin = event.checkin_type == "self_checkin";
    let is_checked_in = event.is_checked_in;
    let event_for_click = event.clone();

    let bg_class = if is_selected {
        "bg-background-neutral-secondary-pressed"
    } else {
        "bg-background-neutral-secondary"
    };

    rsx! {
        button {
            class: "w-full rounded-2xl px-4 py-4 flex items-center gap-4 text-left shadow-sm {bg_class} hover:bg-background-neutral-tertiary-hover transition-colors",
            onclick: move |_| on_click.call(event_for_click.clone()),
            // Icon/checkbox
            div { class: "w-8 h-8 rounded-lg flex items-center justify-center flex-shrink-0",
                if is_checked_in {
                    Icon {
                        width: 16,
                        height: 16,
                        icon: LdCheck,
                        class: "text-green-600",
                    }
                } else if is_self_checkin {
                    Icon {
                        width: 16,
                        height: 16,
                        icon: LdSquare,
                        class: "text-foreground-neutral-secondary",
                    }
                } else {
                    Icon {
                        width: 16,
                        height: 16,
                        icon: LdLock,
                        class: "text-foreground-neutral-secondary",
                    }
                }
            }

            // Event info
            div { class: "flex flex-col gap-0.5 flex-1",
                p { class: "font-medium text-foreground-neutral-primary", "{event.name}" }
                p { class: "text-sm text-foreground-neutral-secondary",
                    if is_self_checkin {
                        "Self Check-In"
                    } else {
                        "Requires QR Scan by Organizer"
                    }
                }
            }

            // Points badge if any
            if let Some(pts) = event.points {
                span { class: "text-xs bg-background-neutral-primary text-foreground-neutral-secondary px-2 py-1 rounded-full",
                    "{pts} pts"
                }
            }
        }
    }
}

/// Organizer view - event management with attendees
#[component]
fn OrganizerCheckinView(
    slug: String,
    events: Vec<ScheduleEvent>,
    on_refresh: EventHandler<()>,
) -> Element {
    let now = chrono::Local::now().naive_local();
    let is_mobile = use_context::<Signal<bool>>();

    // Search state
    let mut search_query = use_signal(|| String::new());

    // Filter events by search query
    let filtered_events: Vec<ScheduleEvent> = events
        .iter()
        .filter(|e| {
            let query = search_query().to_lowercase();
            query.is_empty() || e.name.to_lowercase().contains(&query)
        })
        .cloned()
        .collect();

    // Categorize filtered events
    let (current, upcoming, past) = categorize_events(&filtered_events, now);

    // Selected event state (only used on desktop)
    let mut selected_event = use_signal(|| None::<ScheduleEvent>);

    rsx! {
        div { class: "overflow-hidden h-full flex flex-col",
            h1 { class: "text-[30px] font-semibold leading-[38px] text-foreground-neutral-primary pt-11 pb-7",
                "Event Check-In"
            }

            if *is_mobile.read() {
                // Mobile: full-width event list only
                div { class: "flex-1 overflow-y-auto pb-7",
                    div { class: "bg-background-neutral-primary rounded-[20px] p-5",
                        // Search input
                        div { class: "flex items-center gap-2 px-3 py-2 rounded-xl border border-stroke-neutral-1 bg-background-neutral-secondary mb-4",
                            Icon {
                                width: 16,
                                height: 16,
                                icon: LdSearch,
                                class: "text-foreground-neutral-tertiary",
                            }
                            input {
                                r#type: "text",
                                class: "flex-1 bg-transparent text-foreground-neutral-primary placeholder:text-foreground-neutral-tertiary outline-none text-sm",
                                placeholder: "Search events...",
                                value: "{search_query}",
                                oninput: move |e| search_query.set(e.value()),
                            }
                        }

                        // Current
                        if !current.is_empty() {
                            MobileEventCategorySection {
                                title: "Current".to_string(),
                                events: current.clone(),
                                slug: slug.clone(),
                            }
                        }

                        // Upcoming
                        if !upcoming.is_empty() {
                            MobileEventCategorySection {
                                title: "Upcoming".to_string(),
                                events: upcoming.clone(),
                                slug: slug.clone(),
                            }
                        }

                        // Past
                        if !past.is_empty() {
                            MobileEventCategorySection {
                                title: "Past".to_string(),
                                events: past.clone(),
                                slug: slug.clone(),
                            }
                        }
                    }
                }
            } else {
                // Desktop: side-by-side layout
                div { class: "flex flex-col gap-6 flex-1 overflow-hidden lg:flex-row",
                    // Left panel - Event list by category
                    div { class: "mb-7 bg-background-neutral-primary rounded-[20px] flex-1 p-7 flex flex-col overflow-hidden",
                        // Search input
                        div { class: "flex items-center gap-2 px-3 py-2 rounded-xl border border-stroke-neutral-1 bg-background-neutral-secondary mb-4",
                            Icon {
                                width: 16,
                                height: 16,
                                icon: LdSearch,
                                class: "text-foreground-neutral-tertiary",
                            }
                            input {
                                r#type: "text",
                                class: "flex-1 bg-transparent text-foreground-neutral-primary placeholder:text-foreground-neutral-tertiary outline-none text-sm",
                                placeholder: "Search events...",
                                value: "{search_query}",
                                oninput: move |e| search_query.set(e.value()),
                            }
                        }

                        div { class: "flex-1 overflow-y-auto",
                            // Current
                            if !current.is_empty() {
                                EventCategorySection {
                                    title: "Current".to_string(),
                                    events: current.clone(),
                                    selected_event,
                                    on_select: move |e| selected_event.set(Some(e)),
                                }
                            }

                            // Upcoming
                            if !upcoming.is_empty() {
                                EventCategorySection {
                                    title: "Upcoming".to_string(),
                                    events: upcoming.clone(),
                                    selected_event,
                                    on_select: move |e| selected_event.set(Some(e)),
                                }
                            }

                            // Past
                            if !past.is_empty() {
                                EventCategorySection {
                                    title: "Past".to_string(),
                                    events: past.clone(),
                                    selected_event,
                                    on_select: move |e| selected_event.set(Some(e)),
                                }
                            }
                        }
                    }

                    // Right panel - Event detail
                    div { class: "w-full lg:w-96 flex-shrink-0 mb-7",
                        if let Some(event) = selected_event() {
                            EventDetailPanel {
                                slug: slug.clone(),
                                event: event.clone(),
                                on_refresh,
                            }
                        } else {
                            div { class: "bg-background-neutral-primary rounded-[20px] p-6 h-full flex items-center justify-center",
                                p { class: "text-foreground-neutral-secondary text-center",
                                    "Select an event to manage check-ins"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Mobile event category section - navigates on click
#[component]
fn MobileEventCategorySection(title: String, events: Vec<ScheduleEvent>, slug: String) -> Element {
    let nav = use_navigator();

    rsx! {
        div { class: "mb-6",
            h2 { class: "text-sm font-semibold text-foreground-neutral-secondary mb-3 tracking-wide",
                "{title}"
            }
            div { class: "flex flex-col gap-2",
                for event in events.iter() {
                    {
                        let event_clone = event.clone();
                        let slug_for_nav = slug.clone();
                        let nav_clone = nav.clone();
                        rsx! {
                            EventCard {
                                event: event_clone.clone(),
                                on_click: move |e: ScheduleEvent| {
                                    nav_clone
                                        .push(Route::HackathonCheckinEvent {
                                            slug: slug_for_nav.clone(),
                                            event_id: e.id,
                                        });
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Event category section (Current/Upcoming/Past) for desktop organizer
#[component]
fn EventCategorySection(
    title: String,
    events: Vec<ScheduleEvent>,
    selected_event: Signal<Option<ScheduleEvent>>,
    on_select: EventHandler<ScheduleEvent>,
) -> Element {
    rsx! {
        div { class: "mb-6",
            h2 { class: "text-sm font-semibold text-foreground-neutral-secondary mb-3 tracking-wide",
                "{title}"
            }
            div { class: "flex flex-col gap-2",
                for event in events.iter() {
                    EventCard {
                        event: event.clone(),
                        is_selected: selected_event().map(|e| e.id == event.id).unwrap_or(false),
                        on_click: on_select,
                    }
                }
            }
        }
    }
}

/// Event detail panel for organizer
#[component]
fn EventDetailPanel(slug: String, event: ScheduleEvent, on_refresh: EventHandler<()>) -> Element {
    let slug_for_attendees = slug.clone();
    let slug_for_scanner = slug.clone();
    let nav = use_navigator();
    let event_id = event.id;

    // Participant ID input
    let mut participant_id_input = use_signal(|| String::new());
    let mut search_query = use_signal(|| String::new());

    // QR Scanner modal state
    let mut show_qr_scanner = use_signal(|| false);

    // Confirmation modal state
    let mut pending_participant: Signal<Option<ParticipantInfo>> = use_signal(|| None);
    let mut is_confirming = use_signal(|| false);
    let mut show_scanner_modal = use_signal(|| false);
    let mut error_message: Signal<Option<String>> = use_signal(|| None);
    let mut skip_confirmation = use_signal(|| {
        // Check localStorage for skip preference
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    if let Ok(Some(val)) = storage.get_item("skip_checkin_confirmation") {
                        return val == "true";
                    }
                }
            }
        }
        false
    });

    // Fetch attendees - use use_resource with event_id as dependency
    // The resource will re-run when event_id changes because we use it in the async block
    let mut attendees_resource = use_resource(use_reactive!(|(event_id,)| {
        let slug = slug_for_attendees.clone();
        async move {
            let result: Result<Vec<Attendee>, _> = get_attendees(slug, event_id).await;
            result.ok().unwrap_or_default()
        }
    }));

    let is_loading = attendees_resource.read().is_none();
    let attendees = attendees_resource.read().clone().unwrap_or_default();

    // Filter attendees by search
    let filtered_attendees: Vec<Attendee> = attendees
        .iter()
        .filter(|a| {
            let query = search_query().to_lowercase();
            query.is_empty()
                || a.name.to_lowercase().contains(&query)
                || a.email.to_lowercase().contains(&query)
        })
        .cloned()
        .collect();

    rsx! {
        div { class: "bg-background-neutral-primary rounded-[20px] p-6 flex flex-col h-full",
            // Header
            div { class: "flex items-start justify-between mb-6",
                h2 { class: "text-lg font-semibold text-foreground-neutral-primary",
                    "{event.name}"
                }
                if let Some(pts) = event.points {
                    span { class: "flex items-center gap-1 text-sm bg-background-neutral-secondary text-foreground-neutral-secondary px-3 py-1 rounded-full",
                        Icon { width: 14, height: 14, icon: LdTarget }
                        "{pts} Points"
                    }
                }
            }

            // Check-in section
            div { class: "mb-6",
                p { class: "text-sm font-medium text-foreground-neutral-primary mb-3",
                    "Check-in"
                }

                // Scan QR button
                button {
                    class: "w-full flex items-center gap-3 px-4 py-3 rounded-xl border border-stroke-neutral-1 hover:bg-background-neutral-secondary transition-colors mb-3",
                    onclick: move |_| show_qr_scanner.set(true),
                    Icon { width: 20, height: 20, icon: LdQrCode }
                    span { class: "text-foreground-neutral-primary", "Scan QR code" }
                }

                // Manual ID entry
                div { class: "flex gap-2",
                    input {
                        r#type: "text",
                        class: "flex-1 px-4 py-2 rounded-xl border border-stroke-neutral-1 bg-background-neutral-secondary text-foreground-neutral-primary placeholder:text-foreground-neutral-tertiary",
                        placeholder: "Enter participant ID here...",
                        value: "{participant_id_input}",
                        oninput: move |e| participant_id_input.set(e.value()),
                    }
                    button {
                        class: "px-4 py-2 rounded-xl bg-foreground-neutral-primary text-white font-medium",
                        onclick: move |_| {
                            let slug = slug.clone();
                            let id_str = participant_id_input();
                            let refresh = on_refresh.clone();
                            let should_skip = skip_confirmation();

                            if let Ok(user_id) = id_str.parse::<i32>() {
                                if should_skip {
                                    // Skip confirmation, check in directly
                                    spawn(async move {
                                        match organizer_checkin(slug, event_id, user_id).await {
                                            Ok(()) => {
                                                refresh.call(());
                                                attendees_resource.restart();
                                            }
                                            Err(e) => {
                                                let error_str = e.to_string();
                                                if error_str.contains("ALREADY_CHECKED_IN") {
                                                    error_message
                                                        .set(
                                                            // Show confirmation modal
                                                            Some(
                                                                // User not found, check in anyway
                                                                "This participant has already been checked in to this event."
                                                                    .to_string(),
                                                            ),
                                                        );
                                                } else {
                                                    error_message
                                                        .set(Some(format!("Check-in failed: {}", error_str)));
                                                }
                                            }
                                        }
                                    });
                                    participant_id_input.set(String::new());
                                } else {
                                    is_confirming.set(true);
                                    spawn(async move {
                                        if let Ok(info) = get_participant_info(slug, user_id).await {
                                            pending_participant.set(Some(info));
                                        } else {
                                            pending_participant
                                                .set(
                                                    Some(ParticipantInfo {
                                                        user_id,
                                                        name: "Unknown User".to_string(),
                                                        email: String::new(),
                                                    }),
                                                );
                                        }
                                        is_confirming.set(false);
                                    });
                                }
                            }
                        },
                        if is_confirming() {
                            Icon {
                                width: 16,
                                height: 16,
                                icon: LdLoader,
                                class: "animate-spin",
                            }
                        } else {
                            "Go"
                        }
                    }
                }
            }

            // Attendees section
            div { class: "flex-1 flex flex-col overflow-hidden",
                p { class: "text-sm font-medium text-foreground-neutral-primary mb-3",
                    "Attendees ({filtered_attendees.len()})"
                }

                // Search
                div { class: "flex items-center gap-2 px-3 py-2 rounded-xl border border-stroke-neutral-1 bg-background-neutral-secondary mb-3",
                    Icon {
                        width: 16,
                        height: 16,
                        icon: LdSearch,
                        class: "text-foreground-neutral-tertiary",
                    }
                    input {
                        r#type: "text",
                        class: "flex-1 bg-transparent text-foreground-neutral-primary placeholder:text-foreground-neutral-tertiary outline-none",
                        placeholder: "Search for a participant...",
                        value: "{search_query}",
                        oninput: move |e| search_query.set(e.value()),
                    }
                }

                // Attendee list
                div { class: "flex-1 overflow-y-auto",
                    if is_loading {
                        div { class: "flex items-center justify-center py-8",
                            Icon {
                                width: 24,
                                height: 24,
                                icon: LdLoader,
                                class: "text-foreground-neutral-tertiary animate-spin",
                            }
                        }
                    } else if filtered_attendees.is_empty() {
                        div { class: "flex items-center justify-center py-8",
                            p { class: "text-sm text-foreground-neutral-tertiary",
                                "No attendees checked in yet"
                            }
                        }
                    } else {
                        div { class: "flex flex-col gap-2",
                            for attendee in filtered_attendees.iter() {
                                AttendeeRow {
                                    key: "{attendee.user_id}",
                                    slug: slug.clone(),
                                    event_id,
                                    attendee: attendee.clone(),
                                    on_remove: move |_| attendees_resource.restart(),
                                }
                            }
                        }
                    }
                }
            }
        }

        // Scanner Modal
        if show_scanner_modal() {
            QRModal {
                // Props for QRModal (dummy values for scanner mode)
                qr_svg: String::new(),
                user_id: 0,
                on_close: move |_| show_scanner_modal.set(false),
                on_scan: move |scanned_id: String| {
                    let slug = slug_for_scanner.clone();
                    show_scanner_modal.set(false);
                    if let Ok(user_id) = scanned_id.parse::<i32>() {
                        nav.push(Route::HackathonScan {
                            slug: slug.clone(),
                            user_id,
                        });
                    }
                },
            }
        }

        // Confirmation modal
        if let Some(participant) = pending_participant() {
            div {
                class: "fixed inset-0 flex items-center justify-center z-50",
                style: "background-color: rgba(0, 0, 0, 0.5);",
                onclick: move |_| {
                    pending_participant.set(None);
                    participant_id_input.set(String::new());
                },

                div {
                    class: "bg-background-neutral-secondary rounded-2xl shadow-xl p-6 max-w-sm w-full mx-4",
                    onclick: move |e| e.stop_propagation(),

                    h2 { class: "text-lg font-semibold text-foreground-neutral-primary text-center mb-6",
                        "Add Participant?"
                    }

                    // Participant info card
                    div { class: "flex items-center gap-3 px-4 py-3 bg-background-neutral-primary rounded-xl mb-6",
                        div { class: "w-10 h-10 rounded-full bg-background-neutral-tertiary flex-shrink-0" }
                        div { class: "flex-1",
                            p { class: "font-medium text-foreground-neutral-primary",
                                "{participant.name}"
                            }
                        }
                    }

                    // Skip checkbox
                    label { class: "flex items-center gap-2 mb-6 cursor-pointer",
                        input {
                            r#type: "checkbox",
                            class: "w-4 h-4 rounded",
                            checked: skip_confirmation(),
                            onchange: move |e| {
                                let checked = e.checked();
                                skip_confirmation.set(checked);
                                // Save to localStorage
                                #[cfg(target_arch = "wasm32")]
                                {
                                    if let Some(window) = web_sys::window() {
                                        if let Ok(Some(storage)) = window.local_storage() {
                                            let _ = storage
                                                .set_item(
                                                    "skip_checkin_confirmation",
                                                    if checked { "true" } else { "false" },
                                                );
                                        }
                                    }
                                }
                            },
                        }
                        span { class: "text-sm text-foreground-neutral-secondary",
                            "Skip this in the future"
                        }
                    }

                    // Buttons
                    div { class: "flex gap-3",
                        button {
                            class: "flex-1 px-4 py-2 rounded-xl bg-foreground-neutral-primary text-white font-medium",
                            onclick: {
                                let slug = slug.clone();
                                let refresh = on_refresh.clone();
                                let user_id = participant.user_id;
                                move |_| {
                                    let slug = slug.clone();
                                    let refresh = refresh.clone();
                                    spawn(async move {
                                        match organizer_checkin(slug, event_id, user_id).await {
                                            Ok(()) => {
                                                refresh.call(());
                                                attendees_resource.restart();
                                            }
                                            Err(e) => {
                                                let error_str = e.to_string();
                                                if error_str.contains("ALREADY_CHECKED_IN") {
                                                    error_message
                                                        .set(
                                                            Some(
                                                                "This participant has already been checked in to this event."
                                                                    .to_string(),
                                                            ),
                                                        );
                                                } else {
                                                    error_message
                                                        .set(Some(format!("Check-in failed: {}", error_str)));
                                                }
                                            }
                                        }
                                    });
                                    pending_participant.set(None);
                                    participant_id_input.set(String::new());
                                }
                            },
                            "Confirm"
                        }
                        button {
                            class: "flex-1 px-4 py-2 rounded-xl bg-red-500 text-white font-medium",
                            onclick: move |_| {
                                pending_participant.set(None);
                                participant_id_input.set(String::new());
                            },
                            "Cancel"
                        }
                    }
                }
            }
        }

        // QR Scanner modal
        if show_qr_scanner() {
            QRScannerModal {
                slug: slug.clone(),
                event_id,
                on_close: move |_| show_qr_scanner.set(false),
                on_checkin_success: move |_| {
                    // Refresh the attendee list after successful check-in
                    attendees_resource.restart();
                    on_refresh.call(());
                },
            }
        }

        // Error popup
        if let Some(error) = error_message() {
            div {
                class: "fixed inset-0 flex items-center justify-center z-50",
                style: "background-color: rgba(0, 0, 0, 0.5);",
                onclick: move |_| error_message.set(None),

                div {
                    class: "bg-background-neutral-secondary rounded-2xl shadow-xl p-6 max-w-sm w-full mx-4",
                    onclick: move |e| e.stop_propagation(),

                    // Error icon
                    div { class: "flex justify-center mb-4",
                        div { class: "w-12 h-12 rounded-full bg-red-100 flex items-center justify-center",
                            span { class: "text-red-500 text-2xl font-bold", "!" }
                        }
                    }

                    h2 { class: "text-lg font-semibold text-foreground-neutral-primary text-center mb-2",
                        "Already Checked In"
                    }

                    p { class: "text-foreground-neutral-secondary text-center mb-6",
                        "{error}"
                    }

                    button {
                        class: "w-full px-4 py-2 rounded-xl bg-foreground-neutral-primary text-white font-medium",
                        onclick: move |_| error_message.set(None),
                        "OK"
                    }
                }
            }
        }
    }
}

/// Attendee row in the list
#[component]
fn AttendeeRow(
    slug: String,
    event_id: i32,
    attendee: Attendee,
    on_remove: EventHandler<()>,
) -> Element {
    let user_id = attendee.user_id;

    rsx! {
        div { class: "flex items-center gap-3 px-3 py-2 rounded-lg bg-background-neutral-secondary",
            // Avatar placeholder
            div { class: "w-8 h-8 rounded-full bg-background-neutral-tertiary flex-shrink-0" }

            // Name
            div { class: "flex-1",
                p { class: "text-sm font-medium text-foreground-neutral-primary",
                    "{attendee.name}"
                }
            }

            // Remove button
            button {
                class: "px-3 py-1 rounded-lg text-xs font-medium text-foreground-neutral-secondary bg-background-neutral-primary hover:bg-red-50 hover:text-red-600 transition-colors",
                onclick: move |_| {
                    let slug = slug.clone();
                    let remove = on_remove.clone();
                    spawn(async move {
                        let _ = organizer_remove_checkin(slug, event_id, user_id).await;
                        remove.call(());
                    });
                },
                "Remove"
            }
        }
    }
}

fn categorize_events(
    events: &[ScheduleEvent],
    now: NaiveDateTime,
) -> (Vec<ScheduleEvent>, Vec<ScheduleEvent>, Vec<ScheduleEvent>) {
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

/// Mobile event detail page for organizer check-in
#[component]
pub fn HackathonCheckinEvent(slug: String, event_id: i32) -> Element {
    if let Some(no_access) = use_require_access_or_redirect(CHECKIN_ROLES) {
        return no_access;
    }

    let slug_for_resource = slug.clone();
    let nav = use_navigator();

    // Fetch schedule events to find the one we need
    let schedule_resource = use_resource(move || {
        let slug = slug_for_resource.clone();
        async move {
            let result: Result<Vec<ScheduleEvent>, _> = get_user_schedule(slug).await;
            result.ok()
        }
    });

    // Find the event
    let event = schedule_resource
        .read()
        .as_ref()
        .and_then(|events| events.as_ref())
        .and_then(|events| events.iter().find(|e| e.id == event_id).cloned());

    // Create a refresh handler
    let mut schedule_resource_clone = schedule_resource.clone();
    let on_refresh = move |_| {
        schedule_resource_clone.restart();
    };

    rsx! {
        div { class: "overflow-hidden h-full flex flex-col p-4",
            // Back button
            button {
                class: "flex items-center gap-2 text-foreground-neutral-secondary mb-4",
                onclick: move |_| {
                    nav.push(Route::HackathonCheckin {
                        slug: slug.clone(),
                    });
                },
                Icon { width: 20, height: 20, icon: LdChevronLeft }
                span { "Back to Events" }
            }

            if let Some(event) = event {
                // Event header
                div { class: "flex items-center justify-between mb-4",
                    h1 { class: "text-2xl font-semibold text-foreground-neutral-primary",
                        "{event.name}"
                    }
                    if let Some(pts) = event.points {
                        div { class: "flex items-center gap-1 px-3 py-1 border border-stroke-neutral-1 rounded-lg",
                            Icon { width: 16, height: 16, icon: LdStar }
                            span { class: "text-sm", "{pts} Points" }
                        }
                    }
                }

                // EventDetailPanel content
                EventDetailPanel {
                    slug: slug.clone(),
                    event: event.clone(),
                    on_refresh,
                }
            } else {
                div { class: "flex-1 flex items-center justify-center",
                    if schedule_resource.read().is_none() {
                        Icon {
                            width: 24,
                            height: 24,
                            icon: LdLoader,
                            class: "text-foreground-neutral-tertiary animate-spin",
                        }
                    } else {
                        p { class: "text-foreground-neutral-secondary", "Event not found" }
                    }
                }
            }
        }
    }
}
