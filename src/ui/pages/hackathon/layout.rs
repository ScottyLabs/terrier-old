use dioxus::prelude::*;

use crate::auth::HackathonRole;
use crate::{Route, auth::hooks::use_hackathon_role, ui::foundation::layout::Sidebar};

#[component]
pub fn HackathonLayout(slug: String) -> Element {
    let nav = navigator();
    let is_mobile = use_context::<Signal<bool>>();

    // 1. Refresh Triggers
    let mut role_refresh_trigger = use_context_provider(|| Signal::new(0u32));
    let application_refresh_trigger = use_context_provider(|| Signal::new(0u32));

    // 2. State Signals (Context Providers)
    // We provide these at the top level so they are always available to hooks,
    // even during resource loading transitions.
    let mut role_signal = use_context_provider(|| Signal::new(None::<HackathonRole>));
    // Hackathon signal provided inside successful match to ensure it has valid data
    // but handled via a helper signal if needed. Actually, we'll keep it inside
    // for now as it doesn't change during redirects like the role does.

    // 3. Fetch Resources
    let slug_for_hackathon = slug.clone();
    let hackathon_resource = use_resource(move || {
        let s = slug_for_hackathon.clone();
        async move { crate::domain::hackathons::handlers::query::get_hackathon_by_slug(s).await }
    });

    let slug_for_role = slug.clone();
    let role_resource = use_resource(move || {
        role_refresh_trigger();
        let s = slug_for_role.clone();
        async move { crate::auth::hooks::get_hackathon_role(s).await }
    });

    // 4. Update effects for role_signal
    use_effect(move || {
        if let Some(Ok(role_opt)) = role_resource.read().as_ref() {
            role_signal.set(role_opt.clone());
        }
    });

    // Clear role signal on refresh trigger to show loading state
    use_effect(move || {
        role_refresh_trigger.read();
        role_signal.set(None);
    });

    // 5. Redirect Logic
    // This is now reactive to the role_signal instead of the resource directly.
    // If role_signal is None (loading), we don't redirect.
    let current_route = use_route::<Route>();
    let role_val = role_signal.read();
    let is_applicant = role_val
        .as_ref()
        .map(|r| r.role == "applicant")
        .unwrap_or(false);
    let role_loading = role_resource.read().is_none();
    let should_redirect_applicant =
        matches!(current_route, Route::HackathonDashboard { .. }) && is_applicant && !role_loading;

    let slug_for_redirect = slug.clone();
    use_effect(move || {
        if should_redirect_applicant {
            nav.push(Route::HackathonApply {
                slug: slug_for_redirect.clone(),
            });
        }
    });

    // 6. Rendering Main Layout
    let h_res = hackathon_resource.read();
    let r_res = role_resource.read();

    match (h_res.as_ref(), r_res.as_ref()) {
        (Some(Ok(Some(hackathon))), Some(Ok(_))) => {
            // Provide hackathon signal context
            let hackathon_signal = use_context_provider(|| Signal::new(hackathon.clone()));

            rsx! {
                div {
                    class: "flex bg-cover bg-center bg-no-repeat w-screen h-screen flex-col md:flex-row md:h-screen md:gap-9 md:p-7",
                    style: if let Some(bg_url) = &hackathon.background_url { format!("background-image: url('{}')", bg_url) } else { String::new() },
                    Sidebar {
                        slug,
                        hackathon_signal,
                        role: role_signal.read().clone(),
                        application_refresh_trigger,
                    }
                    main { class: "flex-1 p-2 min-w-0 overflow-auto", Outlet::<Route> {} }
                }
            }
        }
        (Some(Ok(None)), _) => {
            // Hackathon not found, navigate to 404
            use_effect(move || {
                nav.push(Route::NotFound {
                    route: vec!["h".to_string(), slug.clone()],
                });
            });

            rsx! {
                div { class: "flex items-center justify-center h-screen",
                    p { class: "text-foreground-neutral-primary", "Redirecting..." }
                }
            }
        }
        (Some(Err(_)), _) | (_, Some(Err(_))) => {
            // Error fetching hackathon or role - redirect to home
            use_effect(move || {
                nav.push(Route::Home {});
            });

            rsx! {
                div { class: "flex items-center justify-center h-screen",
                    p { class: "text-foreground-neutral-primary", "Redirecting..." }
                }
            }
        }
        _ => {
            // Loading state
            rsx! {
                div { class: "flex items-center justify-center h-screen",
                    p { class: "text-foreground-neutral-primary", "Loading..." }
                }
            }
        }
    }
}
