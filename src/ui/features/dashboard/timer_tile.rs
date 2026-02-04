use crate::domain::hackathons::types::HackathonInfo;
use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use dioxus_free_icons::{Icon, icons::ld_icons::LdTimer};

#[component]
pub fn TimerTile() -> Element {
    let hackathon = use_context::<Signal<HackathonInfo>>();
    let mut now = use_signal(|| Utc::now().naive_utc());

    use_future(move || async move {
        loop {
            gloo_timers::future::sleep(std::time::Duration::from_secs(1)).await;
            now.set(Utc::now().naive_utc());
        }
    });

    let info = hackathon.read();
    let start = info.start_date;
    let end = info.end_date;
    let current = *now.read();

    let (label, target_time, is_ended) = if current < start {
        ("Hacking starts in", Some(start), false)
    } else if current < end {
        ("Hacking ends in", Some(end), false)
    } else {
        ("Hacking ended", None, true)
    };

    let time_display = if let Some(target) = target_time {
        let duration = target - current;
        let days = duration.num_days();
        let hours = duration.num_hours() % 24;
        let minutes = duration.num_minutes() % 60;
        let seconds = duration.num_seconds() % 60;

        if days > 0 {
            format!("{}d {}h {}m", days, hours, minutes)
        } else {
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        }
    } else {
        "00:00:00".to_string()
    };

    let is_mobile = use_context::<Signal<bool>>();

    rsx! {
        if *is_mobile.read() {
            div {
                class: "flex items-center gap-3 bg-background-neutral-primary rounded-lg p-4 w-full",
                Icon { icon: LdTimer, class: "text-foreground-neutral-primary" }
                span { class: "text-foreground-neutral-primary font-medium", "{label}" }
                if !is_ended {
                    span { class: "ml-auto text-foreground-neutral-primary font-bold tabular-nums", "{time_display}" }
                } else {
                    span { class: "ml-auto text-foreground-neutral-primary font-medium", "Ended" }
                }
            }
        } else {
            div { class: "flex flex-col gap-4 bg-background-neutral-primary rounded-lg p-6 aspect-square",
                div { class: "flex items-center gap-2",
                    Icon {
                        icon: LdTimer,
                        class: "text-foreground-neutral-primary",
                    }
                    span { class: "text-foreground-neutral-primary font-medium", "Hacking Timer" }
                }
                div { class: "flex-1 flex flex-col items-center justify-center gap-2",
                    span { class: "text-foreground-neutral-secondary text-sm font-medium uppercase tracking-wider", "{label}" }
                    if !is_ended {
                        span { class: "text-4xl font-bold text-foreground-neutral-primary tabular-nums", "{time_display}" }
                    } else {
                        Icon {
                            icon: LdTimer,
                            class: "text-foreground-neutral-tertiary w-16 h-16",
                        }
                    }
                }
            }
        }
    }
}
