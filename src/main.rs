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

    rsx! {
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        div { class: "font-sans text-primary bg-background-neutral-secondary-enabled min-h-screen",
            Router::<Route> {}
        }
    }
}
