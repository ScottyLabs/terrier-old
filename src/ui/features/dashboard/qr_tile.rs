use dioxus::prelude::*;
use dioxus_free_icons::{
    Icon,
    icons::ld_icons::{LdExpand, LdQrCode},
};

use crate::{
    auth::HackathonRole, domain::hackathons::types::HackathonInfo,
    ui::foundation::utils::generate_qr_svg,
};

/// QR Tile component - QR code (using nayuki/QR-Code-generator) with deep link to check-in page with hacker id,
/// "Check in QR Code" text, and an expandable QR code
#[component]
pub fn QRTile() -> Element {
    let user_role = use_context::<Option<HackathonRole>>();
    let hackathon = use_context::<Signal<HackathonInfo>>();
    let user_id = user_role.as_ref().map(|r| r.user_id).unwrap_or(-1);
    let slug = hackathon.read().slug.clone();
    // Use HTTPS URL for Universal Links (works with iOS camera app)
    let checkin_url = format!("https://terrier.scottylabs.org/h/{}/scan/{}", slug, user_id);
    let qr_svg = generate_qr_svg(&checkin_url);
    let mut show_modal = use_signal(|| false);
    let is_mobile = use_context::<Signal<bool>>();

    rsx! {
        // Mobile: compact clickable card
        if *is_mobile.read() {
            button {
                class: "flex items-center gap-3 bg-background-neutral-primary rounded-lg p-4 w-full text-left",
                onclick: move |_| show_modal.set(true),
                Icon { icon: LdQrCode, class: "text-foreground-neutral-primary" }
                span { class: "text-foreground-neutral-primary font-medium", "Check-in QR code" }
            }
        } else {
            // Desktop: full tile with QR code visible
            div { class: "flex flex-col gap-4 bg-background-neutral-primary rounded-lg p-6 aspect-square",
                div { class: "flex items-center gap-2",
                    Icon {
                        icon: LdQrCode,
                        class: "text-foreground-neutral-primary",
                    }
                    "Check-in QR Code"
                    button {
                        class: "ml-auto text-black font-semibold text-sm leading-5 rounded-full pl-4 py-2.5",
                        onclick: move |_| show_modal.set(true),
                        Icon {
                            width: 16,
                            height: 16,
                            icon: LdExpand,
                            class: "text-black",
                        }
                    }
                }
                // QR code itself
                QRDisplay { qr_svg: qr_svg.clone(), user_id }
            }
        }

        // Fullscreen QR modal
        if show_modal() {
            QRModal {
                qr_svg: qr_svg.clone(),
                user_id,
                on_close: move |_| show_modal.set(false),
            }
        }
    }
}

/// Reusable QR code display component
#[component]
fn QRDisplay(qr_svg: String, user_id: i32) -> Element {
    rsx! {
        div { class: "w-full h-full flex-col gap-1 flex items-center justify-center p-3",
            div {
                class: "w-full h-full bg-background-neutral-secondary rounded-xl",
                dangerous_inner_html: "{qr_svg}",
            }
            // user id for backup
            div { class: "text-black font-semibold text-sm leading-5", "User ID: {user_id}" }
        }
    }
}

/// Fullscreen QR code modal
#[component]
pub fn QRModal(
    qr_svg: String,
    user_id: i32,
    on_close: EventHandler<()>,
    on_scan: Option<EventHandler<String>>,
) -> Element {
    let show_scanner = on_scan.is_some();

    rsx! {
        // Backdrop - covers entire screen with semi-transparent grey
        div {
            class: "fixed inset-0 flex items-center justify-center z-50",
            style: "background-color: rgba(0, 0, 0, 0.7);",
            onclick: move |_| {
                // Attempt to stop scanner JS
                if show_scanner {
                    let mut eval = document::eval("if (window.stopQrScanner) window.stopQrScanner();");
                    // We don't need to wait for it
                }
                on_close.call(());
            },

            // Modal content - centered QR code or scanner
            div {
                class: "relative flex flex-col items-center justify-center",
                onclick: move |e| e.stop_propagation(),

                // Display Area
                div { class: "w-[95vmin] h-[95vmin] max-w-[500px] max-h-[500px] flex flex-col items-center justify-center gap-4 bg-background-neutral-primary rounded-2xl p-4",
                    if let Some(handler) = on_scan {
                        Scanner { on_scan: handler }
                    } else {
                        div {
                            class: "w-full h-full bg-background-neutral-primary rounded-2xl",
                            dangerous_inner_html: "{qr_svg}",
                        }
                        div { class: "text-white font-semibold text-lg", "User ID: {user_id}" }
                    }
                }
            }
        }
    }
}

#[component]
fn Scanner(on_scan: EventHandler<String>) -> Element {
    // Use eval to initialize the scanner
    // We use use_future to run this once on mount
    let mut eval = document::eval(
        r#"
        const scanHandler = await dioxus.recv();
        
        function onScanSuccess(decodedText, decodedResult) {

            // Check if URL matches our expected format
            if (decodedText.includes("/scan/")) {
                const parts = decodedText.split("/scan/");
                if (parts.length === 2) {
                    const userId = parts[1];
                    
                    // Stop scanning immediately
                    if (window.html5QrcodeScanner) {
                        window.html5QrcodeScanner.clear().then(() => {
                            // Send back to Rust after clearing
                            dioxus.send(userId);
                        }).catch(err => {
                            console.error("Failed to clear scanner", err);
                            dioxus.send(userId);
                        });
                    } else {
                        dioxus.send(userId);
                    }
                }
            }
        }

        function onScanFailure(error) {
            // handle scan failure
        }
        
        window.stopQrScanner = function() {
             if (window.html5QrcodeScanner) {
                window.html5QrcodeScanner.clear();
             }
        };

        setTimeout(() => {
            if (document.getElementById('reader')) {
                window.html5QrcodeScanner = new Html5QrcodeScanner(
                "reader",
                { fps: 10, qrbox: {width: 250, height: 250} },
                false);
                window.html5QrcodeScanner.render(onScanSuccess, onScanFailure);
            }
        }, 100);
        "#,
    );

    // Handle scan result
    use_future(move || {
        let mut eval = eval.clone();
        let scan_handler = on_scan.clone();
        async move {
            eval.send(true).unwrap(); // Start the script
            if let Ok(scanned_user_id) = eval.recv::<String>().await {
                scan_handler.call(scanned_user_id);
            }
        }
    });

    rsx! {
        div { class: "w-full text-center text-lg font-semibold mb-2", "Scan Participant QR" }
        div {
            id: "reader",
            class: "w-full bg-black rounded-xl overflow-hidden shadow-lg border border-gray-800",
            style: "min-height: 300px;"
        }
        div { class: "text-sm text-foreground-neutral-secondary text-center mt-2",
            "Point camera at a check-in QR Code"
        }
    }
}
