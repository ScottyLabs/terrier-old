use dioxus::prelude::*;
use dioxus_free_icons::{Icon, icons::ld_icons::LdBookOpen};

#[component]
pub fn HackerGuideTile() -> Element {
    let guide_url = "https://docs.google.com/document/d/1RLg7SqXPuZhxk_aycPPn9UffR3fPlr0O68XuokCfWKQ/edit?tab=t.0#heading=h.spvf7wlw05w9";

    let is_mobile = use_context::<Signal<bool>>();

    rsx! {
        if *is_mobile.read() {
             a {
                class: "flex items-center gap-3 bg-background-neutral-primary rounded-lg p-4 w-full text-left hover:opacity-80 transition-opacity",
                href: "{guide_url}",
                target: "_blank",
                Icon { icon: LdBookOpen, class: "text-foreground-neutral-primary" }
                span { class: "text-foreground-neutral-primary font-medium", "Hacker Guide" }
            }
        } else {
            a {
                class: "flex flex-col gap-4 bg-background-neutral-primary rounded-lg p-6 aspect-square hover:opacity-80 transition-opacity",
                href: "{guide_url}",
                target: "_blank",
                div { class: "flex items-center gap-2",
                    Icon {
                        icon: LdBookOpen,
                        class: "text-foreground-neutral-primary",
                    }
                    span { class: "text-foreground-neutral-primary font-medium", "Hacker Guide" }
                }
                div { class: "flex-1 flex items-center justify-center",
                    Icon {
                        icon: LdBookOpen,
                        class: "text-foreground-neutral-primary w-16 h-16 opacity-20",
                    }
                }
            }
        }
    }
}
