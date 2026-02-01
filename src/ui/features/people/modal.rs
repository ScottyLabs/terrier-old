use dioxus::prelude::*;
use dioxus_free_icons::{Icon, icons::ld_icons::LdExternalLink};

use crate::ui::foundation::components::{Button, ButtonVariant};
use crate::ui::foundation::modals::base::ModalBase;

#[component]
pub fn PeopleModal(
    user_name: String,
    user_email: String,
    role: String,
    display_name: Option<String>,
    portfolio: Option<String>,
    major: Option<String>,
    graduation_year: Option<String>,
    dietary_restrictions: Option<String>,
    shirt_size: Option<String>,
    is_admin: bool,
    on_close: EventHandler<()>,
    on_remove: EventHandler<()>,
    on_send_message: EventHandler<()>,
) -> Element {
    rsx! {
        ModalBase {
            on_close,
            width: "798px",
            max_height: "calc(100dvh - env(safe-area-inset-top, 0px) - env(safe-area-inset-bottom, 0px) - 48px)",

            div { class: "flex flex-col",
                div { class: "flex flex-col h-[565px] justify-between px-7 py-0",
                    div { class: "flex flex-col gap-2",
                        p { class: "text-base font-medium leading-6 text-foreground-neutral-secondary",
                            "Team Name"
                        }
                        div { class: "flex items-center justify-between",
                            p { class: "text-2xl font-medium leading-8 text-foreground-neutral-primary",
                                "{user_name}"
                            }
                            div { class: "bg-background-neutral-primary rounded px-2.5 py-1.5",
                                p { class: "text-sm font-semibold leading-5 text-foreground-neutral-primary text-center",
                                    "{role}"
                                }
                            }
                        }
                    }

                    div { class: "flex flex-col gap-9",
                        div { class: "bg-background-neutral-primary rounded-[20px] p-7.5 flex flex-col justify-between h-[270px]",
                            div { class: "flex flex-col gap-4",
                                p { class: "text-base font-medium leading-6 text-foreground-neutral-primary",
                                    "Personal Info"
                                }
                                div { class: "flex items-center justify-between",
                                    div { class: "flex flex-col gap-1 w-[105px]",
                                        p { class: "text-base font-normal leading-6 text-foreground-neutral-secondary",
                                            "Display Name"
                                        }
                                        p { class: "text-base font-normal leading-6 text-foreground-neutral-primary",
                                            "{display_name.clone().unwrap_or_else(|| \"Not provided\".to_string())}"
                                        }
                                    }
                                    div { class: "flex flex-col gap-1 w-[91px]",
                                        p { class: "text-base font-normal leading-6 text-foreground-neutral-secondary",
                                            "Portfolio"
                                        }
                                        if let Some(url) = portfolio.clone() {
                                            div { class: "flex gap-1 items-start",
                                                Icon {
                                                    width: 20,
                                                    height: 20,
                                                    icon: LdExternalLink,
                                                }
                                                a {
                                                    href: "{url}",
                                                    target: "_blank",
                                                    class: "text-base font-normal leading-6 text-foreground-neutral-primary",
                                                    "Link"
                                                }
                                            }
                                        } else {
                                            p { class: "text-base font-normal leading-6 text-foreground-neutral-primary",
                                                "Not provided"
                                            }
                                        }
                                    }
                                }
                            }

                            div { class: "flex flex-col gap-4",
                                p { class: "text-base font-medium leading-6 text-foreground-neutral-primary",
                                    "School"
                                }
                                div { class: "flex gap-33 items-center",
                                    div { class: "flex flex-col gap-1 w-[91px]",
                                        p { class: "text-base font-normal leading-6 text-foreground-neutral-secondary",
                                            "Major"
                                        }
                                        p { class: "text-base font-normal leading-6 text-foreground-neutral-primary",
                                            "{major.clone().unwrap_or_else(|| \"Not provided\".to_string())}"
                                        }
                                    }
                                    div { class: "flex flex-col gap-1 w-[124px]",
                                        p { class: "text-base font-normal leading-6 text-foreground-neutral-secondary",
                                            "Graduation Year"
                                        }
                                        p { class: "text-base font-normal leading-6 text-foreground-neutral-primary",
                                            "{graduation_year.clone().unwrap_or_else(|| \"Not provided\".to_string())}"
                                        }
                                    }
                                }
                            }
                        }

                        div { class: "bg-background-neutral-primary rounded-[20px] p-7.5 flex flex-col h-[152px] justify-between",
                            div { class: "flex flex-col gap-4 w-[314px]",
                                p { class: "text-base font-medium leading-6 text-foreground-neutral-primary",
                                    "Extra Info"
                                }
                                div { class: "flex items-center justify-between",
                                    div { class: "flex flex-col gap-1 w-[157px]",
                                        p { class: "text-base font-normal leading-6 text-foreground-neutral-secondary",
                                            "Dietary Restrictions"
                                        }
                                        p { class: "text-base font-normal leading-6 text-foreground-neutral-primary",
                                            "{dietary_restrictions.clone().unwrap_or_else(|| \"None\".to_string())}"
                                        }
                                    }
                                    div { class: "flex flex-col gap-1 w-[91px]",
                                        p { class: "text-base font-normal leading-6 text-foreground-neutral-secondary",
                                            "Shirt Size"
                                        }
                                        p { class: "text-base font-normal leading-6 text-foreground-neutral-primary",
                                            "{shirt_size.clone().unwrap_or_else(|| \"Not provided\".to_string())}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "flex gap-2.5 items-center justify-end p-7",
                    if is_admin {
                        Button {
                            variant: ButtonVariant::Danger,
                            onclick: move |_| on_remove.call(()),
                            "Remove"
                        }
                    }
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| on_send_message.call(()),
                        "Send Message"
                    }
                }
            }
        }
    }
}
