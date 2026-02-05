use crate::ui::foundation::components::{Button, ButtonVariant};
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum ApplicationStatusVariant {
    Submitted,
    Accepted,
    Confirmed,
}

#[component]
pub fn ApplicationStatus(
    variant: ApplicationStatusVariant,
    hackathon_slug: String,
    application_status: Resource<Option<String>>,
    application_refresh_trigger: Signal<u32>,
) -> Element {
    let mut is_loading = use_signal(|| false);

    let slug_for_team = hackathon_slug.clone();
    let slug_for_unsubmit = hackathon_slug.clone();
    let slug_for_decline = hackathon_slug.clone();
    let slug_for_confirm = hackathon_slug.clone();
    let slug_for_undo = hackathon_slug.clone();

    let mut role_refresh_trigger = use_context::<Signal<u32>>();

    match variant {
        ApplicationStatusVariant::Submitted => {
            rsx! {
                div { class: "bg-background-neutral-primary rounded-[20px] shadow-[0px_4px_12px_0px_rgba(0,0,0,0.25)] p-6 md:p-9 w-full max-w-[498px] mx-4 md:mx-0",
                    div { class: "flex flex-col gap-4 md:gap-6 mb-6 md:mb-9",
                        p { class: "text-base md:text-[18px] font-medium leading-6 md:leading-[26px] text-center w-full",
                            "Your Status"
                        }
                        div { class: "bg-background-neutral-secondary rounded-xl p-3 flex items-center justify-center",
                            p { class: "text-xl md:text-[24px] font-medium leading-7 md:leading-8 text-black",
                                "SUBMITTED"
                            }
                        }
                    }
                    p { class: "text-sm md:text-[14px] font-normal leading-5 text-black mb-6 md:mb-9",
                        "Thank you for submitting your application! We'll review it and get back to you soon."
                    }

                    div { class: "flex flex-col md:flex-row gap-3 w-full",
                        Button {
                            variant: ButtonVariant::Tertiary,
                            class: "flex-1",
                            onclick: move |_| {
                                let nav = navigator();
                                nav.push(format!("/h/{}/team", slug_for_team));
                            },
                            "Find a Team"
                        }
                        Button {
                            variant: ButtonVariant::Default,
                            class: "flex-1",
                            disabled: is_loading(),
                            onclick: move |_| {
                                let slug = slug_for_unsubmit.clone();
                                spawn(async move {
                                    is_loading.set(true);
                                    match crate::domain::applications::handlers::unsubmit_application(
                                            slug.clone(),
                                        )
                                        .await
                                    {
                                        Ok(_) => {
                                            application_status.restart();
                                            let current = *application_refresh_trigger.read();
                                            application_refresh_trigger.set(current + 1);
                                            is_loading.set(false);
                                        }
                                        Err(e) => {
                                            let error_msg = format!("Failed to unsubmit: {}", e);
                                            let _ = dioxus::document::eval(
                                                &format!("alert('{}')", error_msg.replace("'", "\\'")),
                                            );
                                            is_loading.set(false);
                                        }
                                    }
                                });
                            },
                            "Unsubmit"
                        }
                    }
                }
            }
        }
        ApplicationStatusVariant::Accepted => {
            rsx! {
                div { class: "bg-background-neutral-primary rounded-[20px] shadow-[0px_4px_12px_0px_rgba(0,0,0,0.25)] p-6 md:p-9 w-full max-w-[498px] mx-4 md:mx-0",
                    div { class: "flex flex-col gap-4 md:gap-6 mb-6 md:mb-9",
                        p { class: "text-base md:text-[18px] font-medium leading-6 md:leading-[26px] text-center w-full",
                            "Your Status"
                        }
                        div { class: "bg-background-neutral-secondary rounded-xl p-3 flex items-center justify-center",
                            p { class: "text-xl md:text-[24px] font-medium leading-7 md:leading-8 text-black",
                                "ADMITTED"
                            }
                        }
                    }
                    p { class: "text-sm md:text-[14px] font-normal leading-5 text-black mb-6 md:mb-9",
                        "Congratulations! You've been accepted. Please confirm your attendance below to see the dashboard."
                    }

                    div { class: "flex flex-col md:flex-row gap-3 w-full",
                        Button {
                            variant: ButtonVariant::Tertiary,
                            class: "flex-1",
                            disabled: is_loading(),
                            onclick: move |_| {
                                let slug = slug_for_decline.clone();
                                spawn(async move {
                                    is_loading.set(true);
                                    match crate::domain::applications::handlers::decline_attendance(slug.clone())
                                        .await
                                    {
                                        Ok(_) => {
                                            application_status.restart();
                                            let current = *application_refresh_trigger.read();
                                            application_refresh_trigger.set(current + 1);
                                            is_loading.set(false);
                                        }
                                        Err(e) => {
                                            let error_msg = format!("Failed to decline: {}", e);
                                            let _ = dioxus::document::eval(
                                                &format!("alert('{}')", error_msg.replace("'", "\\'")),
                                            );
                                            is_loading.set(false);
                                        }
                                    }
                                });
                            },
                            if is_loading() {
                                "Processing..."
                            } else {
                                "Decline Attendance"
                            }
                        }
                        Button {
                            variant: ButtonVariant::Default,
                            class: "flex-1",
                            disabled: is_loading(),
                            onclick: move |_| {
                                let slug = slug_for_confirm.clone();
                                spawn(async move {
                                    is_loading.set(true);

                                    // Redirect to dashboard after successful confirmation
                                    match crate::domain::applications::handlers::confirm_attendance(slug.clone())
                                        .await
                                    {
                                        Ok(_) => {
                                            // Refresh role before redirecting to avoid 403
                                            let current_role_trigger = *role_refresh_trigger.read();
                                            role_refresh_trigger.set(current_role_trigger + 1);

                                            application_status.restart();
                                            let current = *application_refresh_trigger.read();
                                            application_refresh_trigger.set(current + 1);
                                            is_loading.set(false);
                                            let nav = navigator();
                                            nav.push(crate::Route::HackathonDashboard {
                                                slug,
                                            });
                                        }
                                        Err(e) => {
                                            let error_msg = format!("Failed to confirm: {}", e);
                                            let _ = dioxus::document::eval(
                                                &format!("alert('{}')", error_msg.replace("'", "\\'")),
                                            );
                                            is_loading.set(false);
                                        }
                                    }
                                });
                            },
                            if is_loading() {
                                "Processing..."
                            } else {
                                "Confirm"
                            }
                        }
                    }
                }
            }
        }
        ApplicationStatusVariant::Confirmed => {
            rsx! {
                div { class: "bg-background-neutral-primary rounded-[20px] shadow-[0px_4px_12px_0px_rgba(0,0,0,0.25)] p-6 md:p-9 w-full max-w-[498px] mx-4 md:mx-0",
                    div { class: "flex flex-col gap-4 md:gap-6 mb-6 md:mb-9",
                        p { class: "text-base md:text-[18px] font-medium leading-6 md:leading-[26px] text-center w-full",
                            "Your Status"
                        }
                        div { class: "bg-background-neutral-secondary rounded-xl p-3 flex items-center justify-center",
                            p { class: "text-xl md:text-[24px] font-medium leading-7 md:leading-8 text-black",
                                "CONFIRMED"
                            }
                        }
                    }
                    p { class: "text-sm md:text-[14px] font-normal leading-5 text-black mb-6 md:mb-9",
                        "You're all set! You can now access the dashboard and start forming or joining a team."
                    }

                    div { class: "flex flex-col md:flex-row gap-3 w-full",
                        Button {
                            variant: ButtonVariant::Tertiary,
                            class: "flex-1",
                            onclick: move |_| {
                                let nav = navigator();
                                nav.push(format!("/h/{}/team", slug_for_team));
                            },
                            "Find a Team"
                        }
                        Button {
                            variant: ButtonVariant::Default,
                            class: "flex-1",
                            disabled: is_loading(),
                            onclick: move |_| {
                                let slug = slug_for_undo.clone();
                                spawn(async move {
                                    is_loading.set(true);
                                    match crate::domain::applications::handlers::undo_confirmation(slug.clone())
                                        .await
                                    {
                                        Ok(_) => {
                                            // Refresh role after undoing confirmation
                                            let current_role_trigger = *role_refresh_trigger.read();
                                            role_refresh_trigger.set(current_role_trigger + 1);

                                            application_status.restart();
                                            let current = *application_refresh_trigger.read();
                                            application_refresh_trigger.set(current + 1);
                                            is_loading.set(false);
                                        }
                                        Err(e) => {
                                            let error_msg = format!("Failed to undo: {}", e);
                                            let _ = dioxus::document::eval(
                                                &format!("alert('{}')", error_msg.replace("'", "\\'")),
                                            );
                                            is_loading.set(false);
                                        }
                                    }
                                });
                            },
                            if is_loading() {
                                "Processing..."
                            } else {
                                "Undo"
                            }
                        }
                    }
                }
            }
        }
    }
}
