use dioxus::prelude::*;
use dioxus_free_icons::{
    Icon,
    icons::ld_icons::{LdPlus, LdX},
};

use crate::{
    auth::{PRIZE_TRACKS_ROLES, hooks::use_require_access_or_redirect},
    domain::applications::handlers::get_user_schedule,
    domain::hackathons::types::ScheduleEvent,
    domain::judging::handlers::get_features,
    domain::prizes::handlers::{
        CreatePrizeRequest, PrizeFeatureWeightInfo, PrizeInfo, UpdatePrizeFeatureWeightsRequest,
        create_prize, delete_prize, get_prizes, update_prize_feature_weights,
    },
    ui::{
        features::prizes::PrizeCard,
        foundation::{
            components::{Button, ButtonSize, ButtonVariant},
            modals::base::ModalBase,
        },
    },
};

#[component]
pub fn HackathonPrizeTracks(slug: String) -> Element {
    if let Some(no_access) = use_require_access_or_redirect(PRIZE_TRACKS_ROLES) {
        return no_access;
    }

    let mut show_create_modal = use_signal(|| false);
    let mut selected_prize = use_signal(|| None::<PrizeInfo>);

    // Form state
    let mut name = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut image_url = use_signal(String::new);
    let mut category = use_signal(String::new);
    let mut value = use_signal(String::new);
    let mut required_event_ids = use_signal(|| Vec::<i32>::new());

    // State for editing weights
    let mut editing_weights = use_signal(|| Vec::<PrizeFeatureWeightInfo>::new());
    let mut editing_required_events = use_signal(|| Vec::<i32>::new());
    let mut show_add_feature = use_signal(|| false);
    let mut weight_error = use_signal(|| None::<String>);

    // Reset weights when prize is selected
    use_effect(move || {
        if let Some(prize) = selected_prize() {
            editing_weights.set(prize.feature_weights.clone());
            editing_required_events.set(prize.required_event_ids.clone());
        } else {
            editing_weights.set(Vec::new());
            editing_required_events.set(Vec::new());
            weight_error.set(None);
            show_add_feature.set(false);
        }
    });

    // Fetch prizes
    let mut prizes_resource = use_resource({
        let slug = slug.clone();
        move || {
            let slug = slug.clone();
            async move { get_prizes(slug).await.ok() }
        }
    });

    // Fetch schedule for required events selection
    let schedule_resource = use_resource({
        let slug = slug.clone();
        move || {
            let slug = slug.clone();
            async move { get_user_schedule(slug).await.ok() }
        }
    });

    // Fetch features
    let features_resource = use_resource({
        let slug = slug.clone();
        move || {
            let slug = slug.clone();
            async move { get_features(slug).await.ok() }
        }
    });

    let mut reset_form = move || {
        name.set(String::new());
        description.set(String::new());
        image_url.set(String::new());
        category.set(String::new());
        value.set(String::new());
        required_event_ids.set(Vec::new());
    };

    rsx! {
        div { class: "flex flex-col h-full",
            // Header
            div { class: "flex flex-col md:flex-row justify-between md:items-center gap-3 pt-6 md:pt-11 pb-4 md:pb-7",
                h1 { class: "text-2xl md:text-[30px] font-semibold leading-8 md:leading-[38px] text-foreground-neutral-primary",
                    "Prize Tracks"
                }
                Button {
                    size: ButtonSize::Compact,
                    onclick: move |_| show_create_modal.set(true),
                    Icon {
                        width: 16,
                        height: 16,
                        icon: LdPlus,
                        class: "text-white mr-1 inline-block",
                    }
                    "Add Prize"
                }
            }

            // Prize grid
            div { class: "flex-1 overflow-y-auto",
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
                            p { class: "text-foreground-neutral-secondary",
                                "No prizes configured yet. Click \"Add Prize\" to create one."
                            }
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

        // Create prize modal
        if show_create_modal() {
            ModalBase {
                on_close: move |_| {
                    show_create_modal.set(false);
                    reset_form();
                },
                width: "500px",
                max_height: "90vh",

                div { class: "p-7",
                    h2 { class: "text-2xl font-semibold text-foreground-neutral-primary mb-6",
                        "Create New Prize"
                    }

                    form {
                        class: "flex flex-col gap-4",
                        onsubmit: {
                            let slug = slug.clone();
                            move |evt: FormEvent| {
                                evt.prevent_default();
                                let slug = slug.clone();
                                let request = CreatePrizeRequest {
                                    name: name(),
                                    description: if description().is_empty() {
                                        None
                                    } else {
                                        Some(description())
                                    },
                                    image_url: if image_url().is_empty() { None } else { Some(image_url()) },
                                    category: if category().is_empty() { None } else { Some(category()) },
                                    value: value(),
                                    required_event_ids: required_event_ids(),
                                };
                                spawn(async move {
                                    if create_prize(slug, request).await.is_ok() {
                                        show_create_modal.set(false);
                                        reset_form();
                                        prizes_resource.restart();
                                    }
                                });
                            }
                        },

                        // Name field
                        div { class: "flex flex-col gap-2",
                            label { class: "text-sm font-medium text-foreground-neutral-primary",
                                "Name *"
                            }
                            input {
                                class: "px-4 h-12 bg-background-neutral-secondary text-foreground-neutral-primary text-sm font-normal rounded-[0.625rem] border border-border-neutral-primary",
                                r#type: "text",
                                placeholder: "Prize name",
                                required: true,
                                value: "{name}",
                                oninput: move |e| name.set(e.value()),
                            }
                        }

                        // Description field
                        div { class: "flex flex-col gap-2",
                            label { class: "text-sm font-medium text-foreground-neutral-primary",
                                "Description"
                            }
                            textarea {
                                class: "px-4 py-3 bg-background-neutral-secondary text-foreground-neutral-primary text-sm font-normal rounded-[0.625rem] border border-border-neutral-primary min-h-[100px]",
                                placeholder: "Prize description",
                                value: "{description}",
                                oninput: move |e| description.set(e.value()),
                            }
                        }

                        // Category field
                        div { class: "flex flex-col gap-2",
                            label { class: "text-sm font-medium text-foreground-neutral-primary",
                                "Category"
                            }
                            input {
                                class: "px-4 h-12 bg-background-neutral-secondary text-foreground-neutral-primary text-sm font-normal rounded-[0.625rem] border border-border-neutral-primary",
                                r#type: "text",
                                placeholder: "e.g., Grand Prize, Best Design, etc.",
                                value: "{category}",
                                oninput: move |e| category.set(e.value()),
                            }
                        }

                        // Value field
                        div { class: "flex flex-col gap-2",
                            label { class: "text-sm font-medium text-foreground-neutral-primary",
                                "Value *"
                            }
                            input {
                                class: "px-4 h-12 bg-background-neutral-secondary text-foreground-neutral-primary text-sm font-normal rounded-[0.625rem] border border-border-neutral-primary",
                                r#type: "text",
                                placeholder: "e.g., $1000, MacBook Pro, etc.",
                                required: true,
                                value: "{value}",
                                oninput: move |e| value.set(e.value()),
                            }
                        }

                        // Image URL field
                        div { class: "flex flex-col gap-2",
                            label { class: "text-sm font-medium text-foreground-neutral-primary",
                                "Image URL"
                            }
                            input {
                                class: "px-4 h-12 bg-background-neutral-secondary text-foreground-neutral-primary text-sm font-normal rounded-[0.625rem] border border-border-neutral-primary",
                                r#type: "url",
                                placeholder: "https://example.com/prize-image.jpg",
                                value: "{image_url}",
                                oninput: move |e| image_url.set(e.value()),
                            }
                        }

                        // Required Events
                        div { class: "flex flex-col gap-2",
                            label { class: "text-sm font-medium text-foreground-neutral-primary",
                                "Required Events (Prerequisites)"
                            }
                            div { class: "min-h-[100px] max-h-[200px] overflow-y-auto border border-border-neutral-primary rounded-[0.625rem] p-4 bg-background-neutral-secondary",
                                if let Some(Some(events)) = schedule_resource.read().as_ref() {
                                    div { class: "flex flex-col gap-2",
                                        for event in events.iter() {
                                            {
                                                let event_id = event.id;
                                                let is_selected = required_event_ids().contains(&event_id);
                                                let formatted_date = event.start_time.format("%b %d %H:%M").to_string();
                                                rsx! {
                                                    label { class: "flex items-center gap-2 cursor-pointer hover:bg-background-neutral-tertiary-hover p-2 rounded transition-colors",
                                                        input {
                                                            r#type: "checkbox",
                                                            class: "w-4 h-4 rounded border-stroke-neutral-1 text-brand-primary focus:ring-brand-primary",
                                                            checked: "{is_selected}",
                                                            onchange: move |e| {
                                                                let mut current = required_event_ids();
                                                                if e.value() == "true" {
                                                                    current.push(event_id);
                                                                } else {
                                                                    current.retain(|&id| id != event_id);
                                                                }
                                                                required_event_ids.set(current);
                                                            },
                                                        }
                                                        span { class: "text-sm text-foreground-neutral-primary", "{event.name}" }
                                                        span { class: "text-xs text-foreground-neutral-tertiary ml-auto", "{formatted_date}" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    p { class: "text-sm text-foreground-neutral-secondary",
                                        "Loading events..."
                                    }
                                }
                            }
                        }

                        // Buttons
                        div { class: "flex gap-3 justify-end mt-4",
                            Button {
                                variant: ButtonVariant::Tertiary,
                                button_type: "button".to_string(),
                                onclick: move |_| {
                                    show_create_modal.set(false);
                                    reset_form();
                                },
                                "Cancel"
                            }
                            Button { button_type: "submit".to_string(), "Create Prize" }
                        }
                    }
                }
            }
        }

        // Prize detail modal
        if let Some(prize) = selected_prize() {
            {
                let prize_id = prize.id;
                rsx! {
                    ModalBase {
                        on_close: move |_| selected_prize.set(None),
                        width: "600px",
                        max_height: "90vh",

                    // Feature Weights Section

                    // Add Feature Dropdown

                    // Weights List

                    // Sum validation









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

                            div { class: "flex justify-between items-start mb-2",
                                h2 { class: "text-2xl font-semibold text-foreground-neutral-primary", "{prize.name}" }
                                Button {
                                    variant: ButtonVariant::Danger,
                                    size: ButtonSize::Compact,
                                    onclick: {
                                        let slug = slug.clone();
                                        move |_| {
                                            let slug = slug.clone();
                                            spawn(async move {
                                                if delete_prize(slug, prize_id).await.is_ok() {
                                                    selected_prize.set(None);
                                                    prizes_resource.restart();
                                                }
                                            });
                                        }
                                    },
                                    "Delete Prize"
                                }
                            }

                            if let Some(cat) = &prize.category {
                                span { class: "inline-block px-3 py-1 bg-background-neutral-secondary text-foreground-neutral-secondary text-sm rounded-full mb-4",
                                    "{cat}"
                                }
                            }

                            div { class: "mb-4",
                                p { class: "text-lg font-medium text-foreground-brand-primary", "{prize.value}" }
                            }

                            if let Some(desc) = &prize.description {
                                p { class: "text-foreground-neutral-secondary mb-6", "{desc}" }
                            }

                            hr { class: "border-border-neutral-tertiary my-6" }

                            // Required Events
                            div { class: "mb-6",
                                h3 { class: "text-lg font-semibold text-foreground-neutral-primary mb-2",
                                    "Required Events"
                                }
                                div { class: "min-h-[100px] max-h-[200px] overflow-y-auto border border-border-neutral-primary rounded-[0.625rem] p-4 bg-background-neutral-secondary",
                                    if let Some(Some(events)) = schedule_resource.read().as_ref() {
                                        div { class: "flex flex-col gap-2",
                                            for event in events.iter() {
                                                {
                                                    let event_id = event.id;
                                                    let is_selected = editing_required_events().contains(&event_id);
                                                    let formatted_date = event.start_time.format("%b %d %H:%M").to_string();
                                                    rsx! {
                                                        label { class: "flex items-center gap-2 cursor-pointer hover:bg-background-neutral-tertiary-hover p-2 rounded transition-colors",
                                                            input {
                                                                r#type: "checkbox",
                                                                class: "w-4 h-4 rounded border-stroke-neutral-1 text-brand-primary focus:ring-brand-primary",
                                                                checked: "{is_selected}",
                                                                onchange: move |e| {
                                                                    let mut current = editing_required_events();
                                                                    if e.value() == "true" {
                                                                        current.push(event_id);
                                                                    } else {
                                                                        current.retain(|&id| id != event_id);
                                                                    }
                                                                    editing_required_events.set(current);
                                                                },
                                                            }
                                                            span { class: "text-sm text-foreground-neutral-primary", "{event.name}" }
                                                            span { class: "text-xs text-foreground-neutral-tertiary ml-auto", "{formatted_date}" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        p { class: "text-sm text-foreground-neutral-secondary",
                                            "Loading events..."
                                        }
                                    }
                                }
                                div { class: "flex justify-end mt-2",
                                    Button {
                                        size: ButtonSize::Compact,
                                        onclick: {
                                            let slug = slug.clone();
                                            move |_| {
                                                let slug = slug.clone();
                                                spawn(async move {
                                                    // We use the same update_prize_feature_weights for now? No, we need update_prize.
                                                    // But update_prize requires all fields. This is tricky.
                                                    // Let's use update_prize but we need to pass existing values for other fields.
                                                    // Does update_prize support Option<None> meaning "no change"?
                                                    // Yes, UpdatePrizeRequest fields are Option. If None, they are not updated.
                                                    // So we just pass required_event_ids.

                                                    // Wait, we need to import update_prize
                                                    use crate::domain::prizes::handlers::update_prize;

                                                    match update_prize(
                                                        slug,
                                                        prize_id,
                                                        crate::domain::prizes::handlers::UpdatePrizeRequest {
                                                            name: None,
                                                            description: None,
                                                            image_url: None,
                                                            category: None,
                                                            value: None,
                                                            required_event_ids: Some(editing_required_events()),
                                                        }
                                                    ).await {
                                                        Ok(_) => {
                                                             selected_prize.set(None);
                                                             prizes_resource.restart();
                                                        }
                                                        Err(e) => {
                                                            weight_error.set(Some(e.to_string()));
                                                        }
                                                    }
                                                });
                                            }
                                        },
                                        "Save Requirements"
                                    }
                                }
                            }

                            div { class: "mb-6",
                                div { class: "flex justify-between items-center mb-4",
                                    h3 { class: "text-lg font-semibold text-foreground-neutral-primary",
                                        "Feature Weights"
                                    }
                                    Button {
                                        size: ButtonSize::Compact,
                                        variant: ButtonVariant::Tertiary,
                                        onclick: move |_| show_add_feature.set(true),
                                        if show_add_feature() {
                                            "Cancel Add"
                                        } else {
                                            "Add Feature"
                                        }
                                    }
                                }

                                if show_add_feature() {
                                    if let Some(Some(features)) = features_resource.read().as_ref() {
                                        div { class: "bg-background-neutral-secondary rounded-lg p-3 mb-4",
                                            p { class: "text-sm text-foreground-neutral-secondary mb-2",
                                                "Select a feature to add:"
                                            }
                                            div { class: "flex flex-wrap gap-2",
                                                for feature in features.iter().filter(|f| !editing_weights().iter().any(|w| w.feature_id == f.id)) {
                                                    {
                                                        let feature = feature.clone();
                                                        rsx! {
                                                            button {
                                                                class: "px-3 py-1.5 bg-background-neutral-primary hover:bg-background-neutral-tertiary-hover rounded-md text-sm transition-colors",
                                                                onclick: move |_| {
                                                                    let mut current = editing_weights();
                                                                    current
                                                                        .push(PrizeFeatureWeightInfo {
                                                                            feature_id: feature.id,
                                                                            weight: 0.0,
                                                                        });
                                                                    editing_weights.set(current);
                                                                    show_add_feature.set(false);
                                                                },
                                                                "{feature.name}"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                div { class: "flex flex-col gap-3",
                                    if editing_weights().is_empty() {
                                        p { class: "text-sm text-foreground-neutral-secondary italic",
                                            "No features assigned to this prize (all features will be weighted equally)."
                                        }
                                    } else {
                                        for (idx , w) in editing_weights().iter().enumerate() {
                                            if let Some(Some(features)) = features_resource.read().as_ref() {
                                                if let Some(feature) = features.iter().find(|f| f.id == w.feature_id) {
                                                    div { class: "flex items-center justify-between bg-background-neutral-secondary rounded-lg px-3 py-2",
                                                        span { class: "text-sm font-medium", "{feature.name}" }
                                                        div { class: "flex items-center gap-2",
                                                            input {
                                                                class: "w-20 px-2 h-8 bg-background-neutral-primary rounded border border-border-neutral-primary text-sm text-right",
                                                                r#type: "number",
                                                                step: "0.01",
                                                                min: "0",
                                                                max: "1",
                                                                value: "{w.weight}",
                                                                oninput: move |e| {
                                                                    if let Ok(val) = e.value().parse::<f32>() {
                                                                        let mut current = editing_weights();
                                                                        current[idx].weight = val;
                                                                        editing_weights.set(current);
                                                                    }
                                                                },
                                                            }
                                                            button {
                                                                class: "text-foreground-neutral-secondary hover:text-status-danger-foreground p-1",
                                                                onclick: move |_| {
                                                                    let mut current = editing_weights();
                                                                    current.remove(idx);
                                                                    editing_weights.set(current);
                                                                },
                                                                Icon {
                                                                    width: 16,
                                                                    height: 16,
                                                                    icon: LdX,
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                {
                                    let total: f32 = editing_weights().iter().map(|w| w.weight).sum();
                                    let weight_class = if (total - 1.0).abs() > 0.001

                                        && !editing_weights().is_empty()
                                    {

                                        "text-status-danger-foreground"
                                    } else {
                                        "text-foreground-neutral-secondary"
                                    };
                                    rsx! {
                                        div { class: "mt-4 flex justify-between items-center text-sm",
                                            span { class: "{weight_class}", "Total Weight: {total:.2}" }
                                            if let Some(err) = weight_error() {
                                                span { class: "text-status-danger-foreground", "{err}" }
                                            }
                                            Button {
                                                size: ButtonSize::Compact,
                                                disabled: (total - 1.0).abs() > 0.001 && !editing_weights().is_empty(),
                                                onclick: {
                                                    let slug = slug.clone();
                                                    move |_| {
                                                        let slug = slug.clone();
                                                        spawn(async move {
                                                            let weights = editing_weights();
                                                            if !weights.is_empty() {
                                                                let total: f32 = weights.iter().map(|w| w.weight).sum();
                                                                if (total - 1.0).abs() > 0.001 {
                                                                    weight_error.set(Some("Weights must sum to 1.0".to_string()));
                                                                    return;
                                                                }
                                                            }


                                                            match update_prize_feature_weights(
                                                                    slug.clone(),
                                                                    prize_id,
                                                                    UpdatePrizeFeatureWeightsRequest {
                                                                        weights,
                                                                    },
                                                                )
                                                                .await
                                                            {
                                                                Ok(_) => {
                                                                    selected_prize.set(None);
                                                                    prizes_resource.restart();
                                                                }
                                                                Err(e) => {
                                                                    weight_error.set(Some(e.to_string()));
                                                                }
                                                            }
                                                        });
                                                    }
                                                },
                                                "Save Weights"
                                            }
                                        }
                                    }
                                }
                            }

                            div { class: "flex gap-3 justify-end mt-6",
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
    }
}
