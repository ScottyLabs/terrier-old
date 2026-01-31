use crate::domain::mock_expo::handlers::{
    assign_prizes_randomly, assign_random_scores, assign_tables, clear_fake_data,
    generate_fake_data,
};
use crate::ui::foundation::components::{Button, ButtonVariant};
use dioxus::prelude::*;

#[component]
pub fn HackathonMockExpo(slug: String) -> Element {
    let mut message = use_signal(|| None::<String>);
    let mut loading = use_signal(|| false);

    let do_generate = {
        let slug = slug.clone();
        move |count: usize| {
            let slug = slug.clone();
            spawn(async move {
                loading.set(true);
                message.set(None);
                match generate_fake_data(slug, count).await {
                    Ok(res) => message.set(Some(res.message)),
                    Err(e) => message.set(Some(format!("Error: {}", e))),
                }
                loading.set(false);
            });
        }
    };

    let do_clear = {
        let slug = slug.clone();
        move |_| {
            let slug = slug.clone();
            spawn(async move {
                let confirmed = web_sys::window()
                    .and_then(|w| {
                        w.confirm_with_message("Are you sure you want to delete all fake data?")
                            .ok()
                    })
                    .unwrap_or(false);

                if !confirmed {
                    return;
                }

                loading.set(true);
                message.set(None);
                match clear_fake_data(slug).await {
                    Ok(res) => message.set(Some(res.message)),
                    Err(e) => message.set(Some(format!("Error: {}", e))),
                }
                loading.set(false);
            });
        }
    };

    let do_assign_tables = {
        let slug = slug.clone();
        move |_| {
            let slug = slug.clone();
            spawn(async move {
                loading.set(true);
                message.set(None);
                match assign_tables(slug).await {
                    Ok(res) => message.set(Some(res.message)),
                    Err(e) => message.set(Some(format!("Error: {}", e))),
                }
                loading.set(false);
            });
        }
    };

    let do_assign_prizes = {
        let slug = slug.clone();
        move |_| {
            let slug = slug.clone();
            spawn(async move {
                loading.set(true);
                message.set(None);
                match assign_prizes_randomly(slug).await {
                    Ok(res) => message.set(Some(res.message)),
                    Err(e) => message.set(Some(format!("Error: {}", e))),
                }
                loading.set(false);
            });
        }
    };

    let do_assign_scores = {
        let slug = slug.clone();
        move |_| {
            let slug = slug.clone();
            spawn(async move {
                loading.set(true);
                message.set(None);
                match assign_random_scores(slug).await {
                    Ok(res) => message.set(Some(res.message)),
                    Err(e) => message.set(Some(format!("Error: {}", e))),
                }
                loading.set(false);
            });
        }
    };

    rsx! {
        div { class: "pt-11 pb-7",
            h1 { class: "text-[30px] font-semibold leading-[38px] text-foreground-neutral-primary mb-8",
                "Mock Expo Tools"
            }

            if let Some(msg) = message.read().as_ref() {
                div { class: "mb-6 p-4 bg-background-neutral-secondary-enabled text-foreground-neutral-primary rounded-lg border border-stroke-neutral-1",
                    "{msg}"
                }
            }

            div { class: "space-y-8",
                // Generation
                div { class: "p-6 bg-background-neutral-primary rounded-[20px]",
                    h2 { class: "text-xl font-semibold text-foreground-neutral-primary mb-4", "Data Generation" }
                    div { class: "flex gap-4",
                        Button {
                            disabled: *loading.read(),
                            onclick: {
                                let do_generate = do_generate.clone();
                                move |_| do_generate(10)
                            },
                            "Generate 10 Projects"
                        }
                        Button {
                            disabled: *loading.read(),
                            onclick: {
                                let do_generate = do_generate.clone();
                                move |_| do_generate(50)
                            },
                            "Generate 50 Projects"
                        }
                    }
                }

                // Assignment
                div { class: "p-6 bg-background-neutral-primary rounded-[20px]",
                    h2 { class: "text-xl font-semibold text-foreground-neutral-primary mb-4", "Setup & Assignments" }
                    div { class: "flex gap-4",
                        Button {
                            disabled: *loading.read(),
                            onclick: do_assign_tables,
                            "Assign Tables (T-1, T-2...)"
                        }
                        Button {
                            disabled: *loading.read(),
                            onclick: do_assign_prizes,
                            "Assign Random Prizes"
                        }
                        Button {
                            disabled: *loading.read(),
                            onclick: do_assign_scores,
                            "Assign Random Scores"
                        }
                    }
                }

                // Cleanup
                div { class: "p-6 bg-background-neutral-primary rounded-[20px] border border-stroke-danger-1",
                    h2 { class: "text-xl font-semibold text-foreground-danger-primary mb-4", "Danger Zone" }
                    Button {
                        variant: ButtonVariant::Danger,
                        disabled: *loading.read(),
                        onclick: do_clear,
                        "Clear All Fake Data"
                    }
                }
            }
        }
    }
}
