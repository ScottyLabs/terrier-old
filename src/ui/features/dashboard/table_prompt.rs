use crate::domain::submissions::handlers::{get_submission, set_table_number};
use crate::ui::foundation::components::{Button, ButtonVariant, Input, InputVariant};
use crate::ui::foundation::modals::base::ModalBase;
use dioxus::prelude::*;

#[component]
pub fn TablePromptModal(slug: String) -> Element {
    let mut submission_resource = use_resource({
        let slug = slug.clone();
        move || {
            let slug = slug.clone();
            async move { get_submission(slug).await.ok().flatten() }
        }
    });

    let mut table_number = use_signal(|| String::new());
    let mut is_submitting = use_signal(|| false);
    let mut show_prompt = use_signal(|| false);
    let mut has_checked = use_signal(|| false);

    use_effect(move || {
        if !has_checked() {
            if let Some(res) = submission_resource.read().as_ref() {
                has_checked.set(true);
                if let Some(sub) = res {
                    if sub.table_number.is_none() {
                        show_prompt.set(true);
                    }
                }
            }
        }
    });

    if !show_prompt() {
        return rsx! {};
    }

    let handle_submit = move |_| {
        let slug = slug.clone();
        let table = table_number();
        if table.is_empty() {
            return;
        }
        spawn(async move {
            is_submitting.set(true);
            if let Ok(_) = set_table_number(slug, table).await {
                show_prompt.set(false);
            }
            is_submitting.set(false);
        });
    };

    rsx! {
        ModalBase {
            on_close: move |_| show_prompt.set(false),
            width: "450px",
            div { class: "p-8 text-center",
                div { class: "w-16 h-16 bg-background-brand-primary/10 rounded-full flex items-center justify-center mx-auto mb-6",
                    span { class: "text-2xl", "📍" }
                }
                h2 { class: "text-2xl font-bold mb-2 text-foreground-neutral-primary", "Where is your team?" }
                p { class: "text-foreground-neutral-secondary mb-8 text-sm leading-relaxed",
                    "Judges need your table number to find you during the expo. Please enter it below."
                }
                Input {
                    label: "Table Number".to_string(),
                    placeholder: "e.g. A12".to_string(),
                    value: table_number,
                    variant: InputVariant::Primary,
                    oninput: move |evt: Event<FormData>| table_number.set(evt.value()),
                }
                div { class: "mt-10 flex gap-3 justify-center",
                    Button {
                        variant: ButtonVariant::Tertiary,
                        onclick: move |_| show_prompt.set(false),
                        "Remind me later"
                    }
                    Button {
                        disabled: table_number().is_empty() || is_submitting(),
                        onclick: handle_submit,
                        if is_submitting() { "Saving..." } else { "Save Table" }
                    }
                }
            }
        }
    }
}
