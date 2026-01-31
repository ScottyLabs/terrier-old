use crate::Route;
use crate::domain::submissions::handlers::set_table_number;
use crate::ui::foundation::components::Button;
use dioxus::prelude::*;

#[component]
pub fn HackathonTableCheckin(slug: String, table_number: String) -> Element {
    let navigator = use_navigator();
    let mut status = use_signal(|| "Checking in...".to_string());
    let mut error = use_signal(|| None::<String>);

    let slug_future = slug.clone();
    let table_number_future = table_number.clone();
    use_future(move || {
        let slug = slug_future.clone();
        let table_number = table_number_future.clone();
        async move {
            match set_table_number(slug.clone(), table_number.clone()).await {
                Ok(_) => {
                    status.set("Successfully checked in to table!".to_string());
                    gloo_timers::future::TimeoutFuture::new(1500).await;
                    navigator.push(Route::HackathonDashboard { slug });
                }
                Err(e) => {
                    error.set(Some(e.to_string()));
                    status.set("Check-in failed.".to_string());
                }
            }
        }
    });

    rsx! {
        div { class: "max-w-md mx-auto mt-20 p-8 bg-background-neutral-primary rounded-xl shadow-lg text-center",
            h2 { class: "text-2xl font-bold mb-4 text-foreground-neutral-primary", "Table Check-in" }
            p { class: "text-lg mb-6 text-foreground-neutral-secondary", "Table: {table_number}" }

            if let Some(err) = error() {
                div { class: "p-4 mb-6 bg-status-danger-background text-status-danger-foreground rounded-lg text-sm",
                    "{err}"
                }
                Button {
                    onclick: move |_: MouseEvent| {
                        navigator.push(Route::HackathonDashboard { slug: slug.clone() });
                    },
                    "Go to Dashboard"
                }
            } else {
                div { class: "flex flex-col items-center",
                    div { class: "animate-spin rounded-full h-12 w-12 border-b-2 border-foreground-brand-primary mb-4" }
                    p { class: "text-foreground-neutral-secondary", "{status}" }
                }
            }
        }
    }
}
