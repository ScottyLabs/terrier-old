//! QR Code Scanner Modal
//!
//! Uses the html5-qrcode JavaScript library to scan QR codes from the device camera.
//! Designed for organizers to scan participant QR codes for event check-in.

use dioxus::prelude::*;
use dioxus_free_icons::{
    Icon,
    icons::ld_icons::{LdCamera, LdCheck, LdLoader, LdX},
};

use crate::domain::applications::handlers::organizer_checkin;

/// QR Scanner Modal - opens camera to scan QR codes and checks in directly
#[component]
pub fn QRScannerModal(
    slug: String,
    event_id: i32,
    on_close: EventHandler<()>,
    on_checkin_success: EventHandler<()>, // Called when check-in succeeds to refresh attendee list
) -> Element {
    let mut is_scanning = use_signal(|| false);
    let mut error_message: Signal<Option<String>> = use_signal(|| None);
    let mut success_message: Signal<Option<String>> = use_signal(|| None);
    let mut scan_complete = use_signal(|| false);

    // Main scanning effect using dioxus.send for JS->Rust communication
    let slug_for_scan = slug.clone();
    use_future(move || {
        let slug = slug_for_scan.clone();
        async move {
            is_scanning.set(true);

            // Create eval with JS that uses dioxus.send to push scanned QR codes
            let mut eval = dioxus::document::eval(
                r#"
                (async function() {
                    try {
                        // Wait for DOM element
                        await new Promise(resolve => setTimeout(resolve, 100));
                        
                        const readerElement = document.getElementById('qr-reader');
                        if (!readerElement) {
                            dioxus.send({ error: 'Scanner element not found' });
                            return;
                        }
                        
                        // Cleanup any existing scanner
                        if (window.html5QrCodeScanner) {
                            try {
                                await window.html5QrCodeScanner.stop();
                            } catch (e) {}
                        }
                        
                        window.html5QrCodeScanner = new Html5Qrcode('qr-reader');
                        
                        await window.html5QrCodeScanner.start(
                            { facingMode: 'environment' },
                            { 
                                fps: 10, 
                                qrbox: { width: 250, height: 250 },
                                aspectRatio: 1.0
                            },
                            (decodedText) => {
                                console.log('QR Scanned:', decodedText);
                                // Send the scanned URL to Rust
                                dioxus.send({ url: decodedText });
                            },
                            (errorMessage) => {
                                // Ignore - just means no QR found yet
                            }
                        );
                    } catch (err) {
                        console.error('Scanner error:', err);
                        dioxus.send({ error: err.message || 'Failed to start camera' });
                    }
                })()
                "#,
            );

            // Wait for messages from JavaScript
            loop {
                if scan_complete() {
                    break;
                }

                match eval.recv::<serde_json::Value>().await {
                    Ok(msg) => {
                        dioxus_logger::tracing::info!("Received from JS: {:?}", msg);

                        if let Some(error) = msg.get("error").and_then(|e| e.as_str()) {
                            error_message.set(Some(error.to_string()));
                            is_scanning.set(false);
                            break;
                        }

                        if let Some(url) = msg.get("url").and_then(|u| u.as_str()) {
                            dioxus_logger::tracing::info!("Scanned URL: {}", url);

                            if let Some(user_id) = parse_scan_url(url) {
                                dioxus_logger::tracing::info!(
                                    "Parsed user_id: {}, checking in to event_id: {}",
                                    user_id,
                                    event_id
                                );

                                // Check in the user directly
                                match organizer_checkin(slug.clone(), event_id, user_id).await {
                                    Ok(()) => {
                                        dioxus_logger::tracing::info!(
                                            "Check-in successful for user {}",
                                            user_id
                                        );

                                        // Stop the scanner
                                        let _ = dioxus::document::eval(
                                            r#"
                                            (async function() {
                                                if (window.html5QrCodeScanner) {
                                                    try {
                                                        await window.html5QrCodeScanner.stop();
                                                    } catch (e) {}
                                                }
                                            })()
                                            "#,
                                        )
                                        .await;

                                        scan_complete.set(true);
                                        success_message.set(Some(format!(
                                            "User {} checked in successfully!",
                                            user_id
                                        )));
                                        on_checkin_success.call(());

                                        // Auto-close after a brief delay to show success
                                        gloo_timers::future::TimeoutFuture::new(1500).await;
                                        on_close.call(());
                                        break;
                                    }
                                    Err(e) => {
                                        let error_str = e.to_string();
                                        dioxus_logger::tracing::error!(
                                            "Check-in failed: {}",
                                            error_str
                                        );

                                        if error_str.contains("ALREADY_CHECKED_IN") {
                                            error_message.set(Some("This participant is already checked in to this event.".to_string()));
                                        } else {
                                            error_message.set(Some(format!(
                                                "Check-in failed: {}",
                                                error_str
                                            )));
                                        }
                                        // Don't break - allow scanning another QR code
                                    }
                                }
                            } else {
                                error_message.set(Some("Invalid QR code format. Please scan a valid participant QR code.".to_string()));
                            }
                        }
                    }
                    Err(e) => {
                        dioxus_logger::tracing::error!("Recv error: {:?}", e);
                        // Don't break on recv errors, keep trying
                    }
                }
            }
        }
    });

    // Cleanup function
    let mut scan_complete_cleanup = scan_complete.clone();
    let cleanup_and_close = move |_| {
        scan_complete_cleanup.set(true);
        spawn(async move {
            let _ = dioxus::document::eval(
                r#"
                (async function() {
                    if (window.html5QrCodeScanner) {
                        try {
                            await window.html5QrCodeScanner.stop();
                            window.html5QrCodeScanner = null;
                        } catch (e) {}
                    }
                })()
                "#,
            )
            .await;
        });
        on_close.call(());
    };

    rsx! {
        div {
            class: "fixed inset-0 flex items-center justify-center z-50",
            style: "background-color: rgba(0, 0, 0, 0.85);",
            onclick: cleanup_and_close,

            div {
                class: "relative flex flex-col items-center w-full max-w-md mx-4",
                onclick: move |e| e.stop_propagation(),

                // Header
                div { class: "flex items-center justify-between w-full mb-4",
                    h2 { class: "text-white text-xl font-semibold", "Scan QR Code" }
                    button {
                        class: "p-2 rounded-full hover:bg-white/10 transition-colors",
                        onclick: cleanup_and_close,
                        Icon {
                            width: 24,
                            height: 24,
                            icon: LdX,
                            class: "text-white",
                        }
                    }
                }

                // Scanner container
                div {
                    class: "w-full aspect-square bg-black rounded-2xl overflow-hidden relative",
                    div { id: "qr-reader", class: "w-full h-full" }

                    // Success overlay
                    if success_message().is_some() {
                        div { class: "absolute inset-0 flex flex-col items-center justify-center bg-green-600/90",
                            Icon {
                                width: 64,
                                height: 64,
                                icon: LdCheck,
                                class: "text-white mb-4",
                            }
                            p { class: "text-white text-lg font-semibold", "Checked In!" }
                        }
                    } else if !is_scanning() || error_message().is_some() {
                        div { class: "absolute inset-0 flex items-center justify-center bg-black/50",
                            if let Some(ref error) = error_message() {
                                div { class: "text-center p-4",
                                    p { class: "text-red-400 mb-2", "{error}" }
                                    button {
                                        class: "px-4 py-2 bg-white/10 rounded-lg text-white hover:bg-white/20",
                                        onclick: move |_| {
                                            error_message.set(None);
                                            is_scanning.set(true);
                                        },
                                        "Scan Again"
                                    }
                                }
                            } else {
                                Icon {
                                    width: 32,
                                    height: 32,
                                    icon: LdLoader,
                                    class: "text-white animate-spin",
                                }
                            }
                        }
                    }
                }

                // Instructions
                div { class: "mt-4 text-center",
                    div { class: "flex items-center justify-center gap-2 text-white/80",
                        Icon {
                            width: 20,
                            height: 20,
                            icon: LdCamera,
                            class: "text-white/60",
                        }
                        span { "Point camera at participant's QR code" }
                    }
                }
            }
        }
    }
}

/// Parse a scan URL to extract the user_id
fn parse_scan_url(url: &str) -> Option<i32> {
    dioxus_logger::tracing::info!("Parsing scan URL: {}", url);
    if let Some(scan_pos) = url.find("/scan/") {
        let after_scan = &url[scan_pos + 6..];
        let user_id_str: String = after_scan
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();

        if !user_id_str.is_empty() {
            return user_id_str.parse().ok();
        }
    }
    None
}
