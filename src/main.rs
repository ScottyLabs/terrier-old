mod auth;
mod config;
mod core;
#[cfg(feature = "server")]
mod docs;
mod domain;
#[cfg(feature = "server")]
mod entities;
#[cfg(feature = "server")]
mod server;
mod ui;

#[cfg(feature = "server")]
use config::Config;
use dioxus::prelude::*;
#[cfg(feature = "server")]
use dioxus_fullstack::FullstackContext;
#[cfg(feature = "server")]
use dioxus_fullstack::extract::FromRef;
#[cfg(target_arch = "wasm32")]
use ui::foundation::hooks::use_window_width;
use ui::pages::*;

#[cfg(feature = "server")]
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: sea_orm::DatabaseConnection,
    pub s3: minio::s3::client::Client,
}

#[cfg(feature = "server")]
impl FromRef<FullstackContext> for AppState {
    fn from_ref(state: &FullstackContext) -> Self {
        state.extension::<AppState>().unwrap()
    }
}

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[route("/")]
    Home {},
    #[route("/h/new")]
    CreateHackathon {},
    #[nest("/h/:slug")]
        #[layout(HackathonLayout)]
            #[route("/")]
                HackathonDashboard {
                    slug: String
                },
            #[route("/applicants")]
                HackathonApplicants {
                    slug: String
                },
            #[route("/people")]
                HackathonPeople {
                    slug: String
                },
            #[route("/team")]
                HackathonTeam {
                    slug: String
                },
            #[route("/schedule")]
                HackathonSchedule {
                    slug: String
                },
            #[route("/schedule/event/:event_id")]
                HackathonScheduleEvent {
                    slug: String,
                    event_id: i32
                },
            #[route("/schedule/event/:event_id/edit")]
                HackathonScheduleEdit {
                    slug: String,
                    event_id: i32
                },
            #[route("/messages")]
                HackathonMessages {
                    slug: String
                },
            #[route("/submission")]
                HackathonSubmission {
                    slug: String
                },
            #[route("/checkin")]
                HackathonCheckin {
                    slug: String
                },
            #[route("/checkin/event/:event_id")]
                HackathonCheckinEvent {
                    slug: String,
                    event_id: i32
                },
            #[route("/scan/:user_id")]
                HackathonScan {
                    slug: String,
                    user_id: i32
                },
            #[route("/table-checkin/:table_number")]
                HackathonTableCheckin {
                    slug: String,
                    table_number: String
                },
            #[route("/profile")]
                HackathonProfile {
                    slug: String
                },
            #[route("/settings")]
                HackathonSettings {
                    slug: String
                },
            #[route("/apply")]
                HackathonApply {
                    slug: String
                },
            #[route("/prize-tracks")]
                HackathonPrizeTracks {
                    slug: String
                },
            #[route("/judge")]
                HackathonJudge {
                    slug: String
                },
            #[route("/judging-admin")]
                HackathonJudgingAdmin {
                    slug: String
                },
            #[route("/results")]
                HackathonResults {
                    slug: String
                },
            #[route("/mock-expo")]
                HackathonMockExpo {
                    slug: String
                },
        #[end_layout]
    #[end_nest]
    #[route("/:..route")]
    NotFound {
        route: Vec<String>
    },
}

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    #[cfg(feature = "server")]
    {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                server::setup().await;
            });
    }

    #[cfg(not(feature = "server"))]
    {
        dioxus_logger::init(dioxus_logger::tracing::Level::DEBUG).expect("failed to init logger");
        dioxus::launch(App);
    }
}

#[component]
fn App() -> Element {
    let user_future = use_server_future(domain::auth::handlers::get_current_user)?;
    let user = use_signal(|| user_future().and_then(|r| r.ok()).flatten());
    use_context_provider(|| user);

    let mut is_mobile = use_signal(|| false);
    use_context_provider(|| is_mobile);

    // Update is_mobile on client-side after hydration
    #[cfg(target_arch = "wasm32")]
    {
        let width = use_window_width();
        use_effect(move || {
            is_mobile.set(*width.read() < 768.0);
        });
    }

    let posthog_config = use_server_future(domain::meta::handlers::get_public_config)?;
    let posthog_script = use_signal(|| {
        if let Some(Ok(config)) = posthog_config() {
            if let Some(key) = &config.posthog_key {
                let host = config
                    .posthog_host
                    .as_deref()
                    .unwrap_or("https://us.i.posthog.com");
                return Some(format!(
                    r#"
                    !function(t,e){{var o,n,p,r;e.__SV||(window.posthog && window.posthog.__loaded)||(window.posthog=e,e._i=[],e.init=function(i,s,a){{function g(t,e){{var o=e.split(".");2==o.length&&(t=t[o[0]],e=o[1]),t[e]=function(){{t.push([e].concat(Array.prototype.slice.call(arguments,0)))}}}}(p=t.createElement("script")).type="text/javascript",p.crossOrigin="anonymous",p.async=!0,p.src=s.api_host.replace(".i.posthog.com","-assets.i.posthog.com")+"/static/array.js",(r=t.getElementsByTagName("script")[0]).parentNode.insertBefore(p,r);var u=e;for(void 0!==a?u=e[a]=[]:a="posthog",u.people=u.people||[],u.toString=function(t){{var e="posthog";return"posthog"!==a&&(e+="."+a),t||(e+=" (stub)"),e}},u.people.toString=function(){{return u.toString(1)+".people (stub)"}},o="init rs ls yi ns us ts ss capture calculateEventProperties vs register register_once register_for_session unregister unregister_for_session gs getFeatureFlag getFeatureFlagPayload getFeatureFlagResult isFeatureEnabled reloadFeatureFlags updateFlags updateEarlyAccessFeatureEnrollment getEarlyAccessFeatures on onFeatureFlags onSurveysLoaded onSessionId getSurveys getActiveMatchingSurveys renderSurvey displaySurvey cancelPendingSurvey canRenderSurvey canRenderSurveyAsync identify setPersonProperties group resetGroups setPersonPropertiesForFlags resetPersonPropertiesForFlags setGroupPropertiesForFlags resetGroupPropertiesForFlags reset get_distinct_id getGroups get_session_id get_session_replay_url alias set_config startSessionRecording stopSessionRecording sessionRecordingStarted captureException startExceptionAutocapture stopExceptionAutocapture loadToolbar get_property getSessionProperty fs ds createPersonProfile setTestUser ps Qr opt_in_capturing opt_out_capturing has_opted_in_capturing has_opted_out_capturing get_explicit_consent_status is_capturing clear_opt_in_out_capturing hs debug M cs getPageViewId captureTraceFeedback captureTraceMetric Kr".split(" "),n=0;n<o.length;n++)g(u,o[n]);e._i.push([i,s,a])}},e.__SV=1}}(document,window.posthog||[]);
                    posthog.init('{}', {{
                        api_host: '{}',
                        defaults: '2025-11-30',
                        person_profiles: 'identified_only',
                    }})
                "#,
                    key, host
                ));
            }
        }
        None
    });

    rsx! {
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        if let Some(script) = posthog_script() {
            script { dangerous_inner_html: "{script}" }
        }
        div { class: "font-sans text-primary bg-background-neutral-secondary-enabled min-h-screen",
            Router::<Route> {}
        }
    }
}
