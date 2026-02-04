import SwiftUI
import WebKit
import Combine
import AuthenticationServices
import SafariServices

/// Configuration for the PWA
struct PWAConfig {
    static let pwaURL = URL(string: "https://terrier.scottylabs.org/h/tartanhacks-2026")!
    
    // Main app domain
    static let allowedHosts = ["scottylabs.org"]
    
    // OAuth/OIDC providers that can stay in the WebView for authentication
    // These don't block WebViews and don't need external browser
    static let authProviderHosts = [
        "login.cmu.edu",          // CMU Shibboleth
        "idp.cmu.edu",            // CMU Identity Provider
        // "accounts.google.com",    // Google OAuth (Moved to external browser auth)
        // "google.com",             // Google domains (Moved to external browser auth)
        // "googleapis.com",         // Google APIs (Moved to external browser auth)
        "auth0.com",              // Auth0
        "okta.com",               // Okta
        "microsoftonline.com",    // Microsoft/Azure AD
        "login.microsoftonline.com",
        "api.github.com",
        "github.com",                 // GitHub OAuth
        "duosecurity.com"             // Duo Security
    ]
    
    // Providers that MUST use external browser (ASWebAuthenticationSession)
    // NOTE: Until Universal Links are fully configured (TEAMID in AASA file),
    // external browser auth won't work. Keep this list empty for now.
    // Once Universal Links are working, add providers that block WebView here.
    static let externalBrowserAuthHosts: [String] = [
        "accounts.google.com",  // Google Sign-In
        "google.com",             // Google domains
        "googleapis.com",         // Google APIs
        "github.com",           // GitHub OAuth
    ]
    
    // Callback URL scheme for deep links
    static let callbackURLScheme = "terrier"
    
    static let appName = "Terrier"
    static let backgroundColor = UIColor.systemBackground
}

/// State object to manage WebView state
class WebViewState: ObservableObject {
    @Published var isLoading = true
    @Published var error: String?
    @Published var isOffline = false
    @Published var canGoBack = false
    @Published var canGoForward = false
    @Published var currentURL: URL?
    
    weak var webView: WKWebView?
    
    func reload() {
        error = nil
        webView?.reload()
    }
    
    func goBack() {
        webView?.goBack()
    }
    
    func goForward() {
        webView?.goForward()
    }
    
    
    func loadHome() {
        webView?.load(URLRequest(url: PWAConfig.pwaURL))
    }
    
    /// Sync cookies from Safari's shared cookie storage to WKWebView
    func syncCookies(completion: @escaping () -> Void = {}) {
        let sharedCookies = HTTPCookieStorage.shared.cookies ?? []
        let wkCookieStore = WKWebsiteDataStore.default().httpCookieStore
        
        // Filter for relevant domains
        let relevantDomains = ["scottylabs.org", "google.com", "googleapis.com"]
        let cookiesToSync = sharedCookies.filter { cookie in
            relevantDomains.contains { domain in
                cookie.domain.contains(domain)
            }
        }
        
        print("[AUTH] 📋 Found \(cookiesToSync.count) cookies to sync from \(sharedCookies.count) total")
        
        if cookiesToSync.isEmpty {
            completion()
            return
        }
        
        let group = DispatchGroup()
        
        for cookie in cookiesToSync {
            group.enter()
            print("[AUTH] 🍪 Syncing cookie: \(cookie.name) for domain \(cookie.domain)")
            wkCookieStore.setCookie(cookie) {
                group.leave()
            }
        }
        
        group.notify(queue: .main) {
            print("[AUTH] ✅ Cookie sync complete")
            completion()
        }
    }
}

/// SwiftUI wrapper for WKWebView with PWA support
struct PWAWebView: UIViewRepresentable {
    @ObservedObject var state: WebViewState
    
    func makeCoordinator() -> Coordinator {
        Coordinator(self)
    }
    
    func makeUIView(context: Context) -> WKWebView {
        // Configure web view preferences for PWA support
        let preferences = WKWebpagePreferences()
        preferences.allowsContentJavaScript = true
        
        let configuration = WKWebViewConfiguration()
        configuration.defaultWebpagePreferences = preferences
        
        // Configure data store for offline support and cookie persistence
        // Using default data store ensures cookies persist across app launches
        let dataStore = WKWebsiteDataStore.default()
        configuration.websiteDataStore = dataStore
        
        // IMPORTANT: We disable app-bound domains to allow OAuth to work properly
        // App-bound domains restrict cookie access which breaks OAuth flows
        // Service workers will still work for the main domain
        configuration.limitsNavigationsToAppBoundDomains = false
        
        // Enable inline media playback
        configuration.allowsInlineMediaPlayback = true
        configuration.mediaTypesRequiringUserActionForPlayback = []
        
        // Allow picture-in-picture
        if #available(iOS 14.5, *) {
            configuration.allowsPictureInPictureMediaPlayback = true
        }
        
        // Add user script for PWA enhancements
        let userScript = WKUserScript(
            source: Self.pwaEnhancementScript,
            injectionTime: .atDocumentStart,
            forMainFrameOnly: true
        )
        configuration.userContentController.addUserScript(userScript)
        
        // Add message handler for native communication
        configuration.userContentController.add(
            context.coordinator,
            name: "nativeApp"
        )
        
        // Add message handler for console logging
        configuration.userContentController.add(
            context.coordinator,
            name: "consoleLog"
        )
        
        // Create the web view
        let webView = WKWebView(frame: .zero, configuration: configuration)
        webView.navigationDelegate = context.coordinator
        webView.uiDelegate = context.coordinator
        webView.scrollView.contentInsetAdjustmentBehavior = .automatic
        webView.backgroundColor = PWAConfig.backgroundColor
        webView.isOpaque = false
        
        // Disable zooming for app-like feel
        webView.scrollView.minimumZoomScale = 1.0
        webView.scrollView.maximumZoomScale = 1.0
        webView.scrollView.bouncesZoom = false
        webView.scrollView.isMultipleTouchEnabled = false
        webView.scrollView.delegate = context.coordinator
        
        // Disable pinch gesture recognizers
        for gestureRecognizer in webView.scrollView.gestureRecognizers ?? [] {
            if gestureRecognizer is UIPinchGestureRecognizer {
                gestureRecognizer.isEnabled = false
            }
        }
        
        // Pull-to-refresh disabled - not needed for this app
        webView.scrollView.refreshControl = nil
        
        // Allow back/forward swipe gestures
        webView.allowsBackForwardNavigationGestures = true
        
        // Store reference
        state.webView = webView
        
        // Load the PWA
        print("[NET] ========================================")
        print("[NET] 🚀 TerrierPWA Network Logging Enabled")
        print("[NET] ========================================")
        print("[NET] Loading initial URL: \(PWAConfig.pwaURL.absoluteString)")
        print("[NET] Allowed hosts: \(PWAConfig.allowedHosts)")
        print("[NET] App-bound domains enabled: \(configuration.limitsNavigationsToAppBoundDomains)")
        print("[NET] ========================================")
        
        let request = URLRequest(url: PWAConfig.pwaURL)
        webView.load(request)
        
        return webView
    }
    
    func updateUIView(_ uiView: WKWebView, context: Context) {
        // Update state from web view
    }
    
    /// JavaScript to enhance PWA behavior in native app context
    static let pwaEnhancementScript = """
    (function() {
        // ========================================
        // Console logging bridge to native
        // ========================================
        (function() {
            function formatArgs(args) {
                return Array.from(args).map(function(arg) {
                    if (arg === null) return 'null';
                    if (arg === undefined) return 'undefined';
                    if (typeof arg === 'object') {
                        try {
                            return JSON.stringify(arg, null, 2);
                        } catch (e) {
                            return String(arg);
                        }
                    }
                    return String(arg);
                }).join(' ');
            }
            
            var originalConsole = {
                log: console.log,
                warn: console.warn,
                error: console.error,
                info: console.info,
                debug: console.debug
            };
            
            function sendToNative(level, args) {
                var message = formatArgs(args);
                if (window.webkit && window.webkit.messageHandlers.consoleLog) {
                    window.webkit.messageHandlers.consoleLog.postMessage({
                        level: level,
                        message: message,
                        url: window.location.href,
                        timestamp: new Date().toISOString()
                    });
                }
            }
            
            console.log = function() {
                originalConsole.log.apply(console, arguments);
                sendToNative('log', arguments);
            };
            
            console.warn = function() {
                originalConsole.warn.apply(console, arguments);
                sendToNative('warn', arguments);
            };
            
            console.error = function() {
                originalConsole.error.apply(console, arguments);
                sendToNative('error', arguments);
            };
            
            console.info = function() {
                originalConsole.info.apply(console, arguments);
                sendToNative('info', arguments);
            };
            
            console.debug = function() {
                originalConsole.debug.apply(console, arguments);
                sendToNative('debug', arguments);
            };
            
            // Capture unhandled errors
            window.addEventListener('error', function(event) {
                sendToNative('error', ['Uncaught Error: ' + event.message + ' at ' + event.filename + ':' + event.lineno + ':' + event.colno]);
            });
            
            // Capture unhandled promise rejections
            window.addEventListener('unhandledrejection', function(event) {
                sendToNative('error', ['Unhandled Promise Rejection: ' + event.reason]);
            });
        })();
        
        // ========================================
        // PWA Native App Detection
        // ========================================
        // Notify the PWA that it's running in a native wrapper
        window.isNativeApp = true;
        window.isiOSApp = true;
        
        // Disable zooming via viewport meta tag
        var viewport = document.querySelector('meta[name="viewport"]');
        if (viewport) {
            viewport.setAttribute('content', 'width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no');
        } else {
            viewport = document.createElement('meta');
            viewport.name = 'viewport';
            viewport.content = 'width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no';
            document.head.appendChild(viewport);
        }
        
        // Disable double-tap zoom
        document.addEventListener('touchend', function(e) {
            var now = Date.now();
            if (now - (window.lastTouchEnd || 0) < 300) {
                e.preventDefault();
            }
            window.lastTouchEnd = now;
        }, { passive: false });
        
        // Override standalone detection
        Object.defineProperty(navigator, 'standalone', {
            get: function() { return true; }
        });
        
        // Override display-mode media query to report standalone
        if (window.matchMedia) {
            const originalMatchMedia = window.matchMedia;
            window.matchMedia = function(query) {
                if (query.includes('display-mode: standalone')) {
                    return {
                        matches: true,
                        media: query,
                        addListener: function() {},
                        removeListener: function() {},
                        addEventListener: function() {},
                        removeEventListener: function() {}
                    };
                }
                return originalMatchMedia.call(window, query);
            };
        }
        
        // Expose native bridge
        window.TerrierNative = {
            postMessage: function(message) {
                if (window.webkit && window.webkit.messageHandlers.nativeApp) {
                    window.webkit.messageHandlers.nativeApp.postMessage(message);
                }
            },
            share: function(data) {
                this.postMessage({ type: 'share', data: data });
            },
            haptic: function(style) {
                this.postMessage({ type: 'haptic', style: style || 'medium' });
            }
        };
        
        // Disable long-press context menu on images (optional, for app-like feel)
        document.addEventListener('contextmenu', function(e) {
            if (e.target.tagName === 'IMG') {
                e.preventDefault();
            }
        });
        
        // Workaround for WKWebView cookie timing issues after OAuth redirects
        // If we detect a "Redirecting..." state that persists, trigger a reload
        let redirectingCheckCount = 0;
        const maxRedirectingChecks = 3;
        
        function checkForStuckRedirect() {
            // Look for the "Redirecting..." text that indicates auth completed but session not loaded
            const bodyText = document.body ? document.body.innerText : '';
            const isRedirecting = bodyText.includes('Redirecting...') || bodyText.includes('Loading...');
            const isOnTerrier = window.location.hostname.includes('scottylabs.org');
            const isNotLoginPage = !window.location.pathname.includes('/realms/');
            
            if (isRedirecting && isOnTerrier && isNotLoginPage) {
                redirectingCheckCount++;
                console.log('[TerrierPWA] Detected redirect state, check ' + redirectingCheckCount + '/' + maxRedirectingChecks);
                
                if (redirectingCheckCount >= maxRedirectingChecks) {
                    console.log('[TerrierPWA] Stuck on redirect, forcing reload to sync cookies...');
                    window.location.reload();
                    return;
                }
                // Check again in 1 second
                setTimeout(checkForStuckRedirect, 1000);
            } else {
                redirectingCheckCount = 0;
            }
        }
        
        // Start checking after page load
        if (document.readyState === 'complete') {
            setTimeout(checkForStuckRedirect, 1500);
        } else {
            window.addEventListener('load', function() {
                setTimeout(checkForStuckRedirect, 1500);
            });
        }
        
        console.log('[TerrierPWA] Native bridge initialized');
    })();
    """
    
    // MARK: - Coordinator
    
    class Coordinator: NSObject, WKNavigationDelegate, WKUIDelegate, WKScriptMessageHandler, UIScrollViewDelegate, ASWebAuthenticationPresentationContextProviding {
        var parent: PWAWebView
        
        init(_ parent: PWAWebView) {
            self.parent = parent
        }
        
        // MARK: - ASWebAuthenticationPresentationContextProviding
        
        func presentationAnchor(for session: ASWebAuthenticationSession) -> ASPresentationAnchor {
            guard let scene = UIApplication.shared.connectedScenes.first as? UIWindowScene,
                  let window = scene.windows.first else {
                return ASPresentationAnchor()
            }
            return window
        }
        
        // MARK: - External Browser Authentication (for Google, etc.)
        
        var authSession: ASWebAuthenticationSession?
        
        /// Start authentication using ASWebAuthenticationSession
        /// This provides passkey support and password autofill from the system keychain
        /// Using nil callbackURLScheme to work with Universal Links (HTTPS callbacks)
        func startExternalBrowserAuth(url: URL) {
            print("[AUTH] 🌐 Starting ASWebAuthenticationSession for: \(url.absoluteString)")
            
            // Cancel any existing auth session
            authSession?.cancel()
            
            // Use nil callbackURLScheme to enable Universal Links (HTTPS callbacks)
            // This requires apple-app-site-association file on the server
            let session = ASWebAuthenticationSession(
                url: url,
                callbackURLScheme: nil  // nil enables Universal Links
            ) { [weak self] callbackURL, error in
                guard let self = self else { return }
                
                if let error = error {
                    let nsError = error as NSError
                    if nsError.domain == ASWebAuthenticationSessionErrorDomain,
                       nsError.code == ASWebAuthenticationSessionError.canceledLogin.rawValue {
                        print("[AUTH] ⏹️ User cancelled authentication")
                    } else {
                        print("[AUTH] ❌ Auth error: \(error.localizedDescription)")
                    }
                    return
                }
                
                guard let callbackURL = callbackURL else {
                    print("[AUTH] ❌ No callback URL received")
                    return
                }
                
                print("[AUTH] ✅ Received callback: \(callbackURL.absoluteString)")
                
                // Callback URL is already HTTPS, load it directly in WebView
                DispatchQueue.main.async {
                    self.parent.state.webView?.load(URLRequest(url: callbackURL))
                }
            }
            
            session.presentationContextProvider = self
            // Share cookies with Safari so user's existing sessions can be used
            session.prefersEphemeralWebBrowserSession = false
            
            authSession = session
            
            if session.start() {
                print("[AUTH] ✅ ASWebAuthenticationSession started")
            } else {
                print("[AUTH] ❌ Failed to start ASWebAuthenticationSession")
            }
        }
        
        /// Sync cookies from Safari's shared cookie storage to WKWebView

        
        // MARK: - UIScrollViewDelegate (Prevent Zooming)
        
        func scrollViewWillBeginZooming(_ scrollView: UIScrollView, with view: UIView?) {
            // Prevent any zooming
            scrollView.pinchGestureRecognizer?.isEnabled = false
        }
        
        func scrollViewDidZoom(_ scrollView: UIScrollView) {
            // Reset zoom if it somehow changed
            if scrollView.zoomScale != 1.0 {
                scrollView.zoomScale = 1.0
            }
        }
        
        func viewForZooming(in scrollView: UIScrollView) -> UIView? {
            // Return nil to disable zooming
            return nil
        }
        
        // MARK: - Pull to Refresh
        
        @objc func handleRefresh(_ refreshControl: UIRefreshControl) {
            parent.state.webView?.reload()
            DispatchQueue.main.asyncAfter(deadline: .now() + 1.0) {
                refreshControl.endRefreshing()
            }
        }
        
        // MARK: - WKScriptMessageHandler
        
        func userContentController(_ userContentController: WKUserContentController, didReceive message: WKScriptMessage) {
            // Handle console log messages
            if message.name == "consoleLog" {
                handleConsoleLog(message)
                return
            }
            
            guard let body = message.body as? [String: Any],
                  let type = body["type"] as? String else {
                return
            }
            
            switch type {
            case "share":
                handleShare(body["data"] as? [String: Any])
            case "haptic":
                handleHaptic(body["style"] as? String)
            default:
                print("[TerrierPWA] Unknown message type: \(type)")
            }
        }
        
        private func handleConsoleLog(_ message: WKScriptMessage) {
            guard let body = message.body as? [String: Any],
                  let level = body["level"] as? String,
                  let logMessage = body["message"] as? String else {
                return
            }
            
            let icon: String
            switch level {
            case "error":
                icon = "❌"
            case "warn":
                icon = "⚠️"
            case "info":
                icon = "ℹ️"
            case "debug":
                icon = "🔍"
            default:
                icon = "📝"
            }
            
            print("[JS] \(icon) [\(level.uppercased())] \(logMessage)")
        }
        
        private func handleShare(_ data: [String: Any]?) {
            guard let data = data else { return }
            
            var items: [Any] = []
            
            if let text = data["text"] as? String {
                items.append(text)
            }
            if let urlString = data["url"] as? String,
               let url = URL(string: urlString) {
                items.append(url)
            }
            
            guard !items.isEmpty else { return }
            
            DispatchQueue.main.async {
                let activityVC = UIActivityViewController(activityItems: items, applicationActivities: nil)
                
                if let scene = UIApplication.shared.connectedScenes.first as? UIWindowScene,
                   let rootVC = scene.windows.first?.rootViewController {
                    // For iPad
                    if let popover = activityVC.popoverPresentationController {
                        popover.sourceView = rootVC.view
                        popover.sourceRect = CGRect(x: rootVC.view.bounds.midX, y: rootVC.view.bounds.midY, width: 0, height: 0)
                        popover.permittedArrowDirections = []
                    }
                    rootVC.present(activityVC, animated: true)
                }
            }
        }
        
        private func handleHaptic(_ style: String?) {
            let generator: UIImpactFeedbackGenerator
            
            switch style {
            case "light":
                generator = UIImpactFeedbackGenerator(style: .light)
            case "heavy":
                generator = UIImpactFeedbackGenerator(style: .heavy)
            case "rigid":
                generator = UIImpactFeedbackGenerator(style: .rigid)
            case "soft":
                generator = UIImpactFeedbackGenerator(style: .soft)
            default:
                generator = UIImpactFeedbackGenerator(style: .medium)
            }
            
            generator.prepare()
            generator.impactOccurred()
        }
        
        // Track if we just completed auth callback
        private var justCompletedAuth = false
        
        // MARK: - WKNavigationDelegate
        
        func webView(_ webView: WKWebView, didStartProvisionalNavigation navigation: WKNavigation!) {
            print("[NET] 🚀 Started provisional navigation")
            if let url = webView.url {
                print("      Current URL: \(url.absoluteString)")
                
                // Detect auth callback
                if url.absoluteString.contains("/auth/callback") {
                    print("[AUTH] 🔐 Auth callback detected - will sync cookies after completion")
                    justCompletedAuth = true
                }
            }
            DispatchQueue.main.async {
                self.parent.state.isLoading = true
                self.parent.state.error = nil
            }
        }
        
        func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
            print("[NET] ✅ Navigation finished successfully")
            if let url = webView.url {
                print("      Final URL: \(url.absoluteString)")
            }
            print("      Title: \(webView.title ?? "(none)")")
            
            // If we just completed auth, log cookies and potentially reload
            if justCompletedAuth {
                justCompletedAuth = false
                print("[AUTH] 🍪 Auth completed, checking cookies...")
                
                // Log all cookies for debugging
                WKWebsiteDataStore.default().httpCookieStore.getAllCookies { cookies in
                    print("[AUTH] 📋 Cookies after auth (\(cookies.count) total):")
                    for cookie in cookies {
                        if cookie.domain.contains("scottylabs") {
                            print("      🍪 \(cookie.name) = \(cookie.value.prefix(20))... (domain: \(cookie.domain), path: \(cookie.path), httpOnly: \(cookie.isHTTPOnly), secure: \(cookie.isSecure))")
                        }
                    }
                }
            }
            
            DispatchQueue.main.async {
                self.parent.state.isLoading = false
                self.parent.state.canGoBack = webView.canGoBack
                self.parent.state.canGoForward = webView.canGoForward
                self.parent.state.currentURL = webView.url
            }
        }
        
        func webView(_ webView: WKWebView, didFail navigation: WKNavigation!, withError error: Error) {
            print("[NET] ❌ Navigation failed")
            handleNavigationError(error)
        }
        
        func webView(_ webView: WKWebView, didFailProvisionalNavigation navigation: WKNavigation!, withError error: Error) {
            print("[NET] ❌ Provisional navigation failed")
            handleNavigationError(error)
        }
        
        // MARK: - Response handling
        
        func webView(_ webView: WKWebView, decidePolicyFor navigationResponse: WKNavigationResponse, decisionHandler: @escaping (WKNavigationResponsePolicy) -> Void) {
            if let httpResponse = navigationResponse.response as? HTTPURLResponse {
                let statusIcon = (200...299).contains(httpResponse.statusCode) ? "✅" : "⚠️"
                print("[NET] \(statusIcon) Response: \(httpResponse.statusCode)")
                print("      URL: \(httpResponse.url?.absoluteString ?? "unknown")")
                print("      MIME: \(navigationResponse.response.mimeType ?? "unknown")")
                if !httpResponse.allHeaderFields.isEmpty {
                    print("      Response Headers:")
                    for (key, value) in httpResponse.allHeaderFields {
                        print("        \(key): \(value)")
                    }
                }
                
                // Log Set-Cookie headers for debugging auth issues
                if let setCookie = httpResponse.allHeaderFields["Set-Cookie"] {
                    print("[COOKIE] 🍪 Set-Cookie received: \(setCookie)")
                }
            }
            decisionHandler(.allow)
        }
        
        func webView(_ webView: WKWebView, didReceiveServerRedirectForProvisionalNavigation navigation: WKNavigation!) {
            print("[NET] 🔀 Server redirect received")
            if let url = webView.url {
                print("      Redirected to: \(url.absoluteString)")
            }
        }
        
        func webView(_ webView: WKWebView, didCommit navigation: WKNavigation!) {
            print("[NET] 📥 Navigation committed (content starting to arrive)")
            if let url = webView.url {
                print("      URL: \(url.absoluteString)")
            }
        }
        
        private func handleNavigationError(_ error: Error) {
            let nsError = error as NSError
            
            print("[NET] ❌ Error Details:")
            print("      Domain: \(nsError.domain)")
            print("      Code: \(nsError.code)")
            print("      Description: \(nsError.localizedDescription)")
            if let failingURL = nsError.userInfo[NSURLErrorFailingURLStringErrorKey] {
                print("      Failing URL: \(failingURL)")
            }
            if let underlyingError = nsError.userInfo[NSUnderlyingErrorKey] {
                print("      Underlying Error: \(underlyingError)")
            }
            
            // Ignore cancelled requests and frame load interrupted
            // Frame load interrupted (error 102) is normal during OAuth redirects
            let ignoredErrors = [
                NSURLErrorCancelled,
                102  // WebKitErrorFrameLoadInterruptedError
            ]
            
            if ignoredErrors.contains(nsError.code) || nsError.domain == "WebKitErrorDomain" && nsError.code == 102 {
                print("[NET] ⏭️ Ignoring expected error: \(nsError.code) (\(nsError.domain))")
                return
            }
            
            DispatchQueue.main.async {
                self.parent.state.isLoading = false
                
                // Check if this is a network connectivity error - use offline view instead
                let isNetworkError = [
                    NSURLErrorNotConnectedToInternet,
                    NSURLErrorNetworkConnectionLost,
                    NSURLErrorDataNotAllowed,
                    NSURLErrorInternationalRoamingOff,
                    NSURLErrorCallIsActive,
                    NSURLErrorTimedOut,
                    NSURLErrorCannotFindHost,
                    NSURLErrorCannotConnectToHost,
                    NSURLErrorDNSLookupFailed
                ].contains(nsError.code)
                
                if isNetworkError {
                    // Set isOffline flag - this will trigger the themed offline view
                    print("[NET] 📴 Network error detected, showing offline view")
                    self.parent.state.isOffline = true
                    self.parent.state.error = nil  // Don't show error overlay
                } else {
                    // Non-network error - show error overlay
                    self.parent.state.error = "Failed to load page: \(error.localizedDescription)"
                }
            }
        }
        
        func webView(_ webView: WKWebView, decidePolicyFor navigationAction: WKNavigationAction, decisionHandler: @escaping (WKNavigationActionPolicy) -> Void) {
            guard let url = navigationAction.request.url else {
                print("[NET] ❌ Navigation cancelled - no URL")
                decisionHandler(.cancel)
                return
            }
            
            // Log the navigation request
            let navigationType = navigationTypeString(navigationAction.navigationType)
            print("[NET] 🔵 Navigation Request:")
            print("      URL: \(url.absoluteString)")
            print("      Type: \(navigationType)")
            print("      Method: \(navigationAction.request.httpMethod ?? "GET")")
            if let mainFrame = navigationAction.targetFrame?.isMainFrame {
                print("      Main Frame: \(mainFrame)")
            }
            if let headers = navigationAction.request.allHTTPHeaderFields, !headers.isEmpty {
                print("      Headers: \(headers)")
            }
            
            // Handle special URL schemes
            let scheme = url.scheme?.lowercased() ?? ""
            
            switch scheme {
            case "tel", "mailto", "sms", "facetime", "facetime-audio":
                // Open these in their respective apps
                print("[NET] 📱 Opening in external app: \(scheme)://")
                UIApplication.shared.open(url, options: [:], completionHandler: nil)
                decisionHandler(.cancel)
                return
                
            case "http", "https":
                // Check if this is an allowed host
                if let host = url.host?.lowercased() {
                    // Check main app domains
                    let isAllowedHost = PWAConfig.allowedHosts.contains { allowedHost in
                        host == allowedHost || host.hasSuffix(".\(allowedHost)")
                    }
                    
                    // Check OAuth/auth provider domains
                    let isAuthProvider = PWAConfig.authProviderHosts.contains { authHost in
                        host == authHost || host.hasSuffix(".\(authHost)")
                    }
                    
                    // Check if this requires external browser auth (Google, etc.)
                    let requiresExternalBrowser = PWAConfig.externalBrowserAuthHosts.contains { authHost in
                        host == authHost || host.hasSuffix(".\(authHost)")
                    }
                    
                    if isAllowedHost {
                        print("[NET] ✅ ALLOW - Host '\(host)' is in allowed list")
                        decisionHandler(.allow)
                    } else if requiresExternalBrowser {
                        // Google and similar providers block WebView sign-in
                        // Use ASWebAuthenticationSession instead
                        print("[NET] 🌐 EXTERNAL BROWSER AUTH - Host '\(host)' requires system browser")
                        decisionHandler(.cancel)
                        startExternalBrowserAuth(url: url)
                    } else if isAuthProvider {
                        print("[NET] 🔐 ALLOW (AUTH) - Host '\(host)' is an auth provider")
                        decisionHandler(.allow)
                    } else {
                        // Open external links in Safari
                        print("[NET] 🌐 EXTERNAL - Host '\(host)' not allowed, opening in Safari")
                        print("      Allowed hosts: \(PWAConfig.allowedHosts)")
                        print("      Auth providers: \(PWAConfig.authProviderHosts)")
                        UIApplication.shared.open(url, options: [:], completionHandler: nil)
                        decisionHandler(.cancel)
                    }
                } else {
                    print("[NET] ✅ ALLOW - No host specified")
                    decisionHandler(.allow)
                }
                return
            
            case "terrier":
                // Handle terrier:// deep links (e.g., terrier://auth/callback)
                print("[NET] 🔗 Terrier deep link: \(url.absoluteString)")
                // Convert terrier:// to https://terrier.scottylabs.org/
                // terrier://auth/callback?code=xxx -> https://terrier.scottylabs.org/auth/callback?code=xxx
                var pathComponents = [String]()
                if let host = url.host {
                    pathComponents.append(host)
                }
                if !url.path.isEmpty && url.path != "/" {
                    pathComponents.append(String(url.path.dropFirst()))
                }
                let path = pathComponents.joined(separator: "/")
                
                var webURLString = "https://terrier.scottylabs.org/\(path)"
                if let query = url.query {
                    webURLString += "?\(query)"
                }
                
                if let webURL = URL(string: webURLString) {
                    print("[NET] 🔄 Converting to web URL: \(webURL.absoluteString)")
                    parent.state.webView?.load(URLRequest(url: webURL))
                }
                decisionHandler(.cancel)
                return
                
            case "about", "data", "blob", "javascript":
                // Allow internal WebView schemes used for iframes, popups, and scripts
                // Google OAuth uses about:blank for popup authentication
                print("[NET] ✅ ALLOW - Internal scheme '\(scheme)'")
                decisionHandler(.allow)
                return
                
            default:
                // Try to open unknown schemes externally
                print("[NET] ❓ Unknown scheme '\(scheme)', attempting external open")
                if UIApplication.shared.canOpenURL(url) {
                    UIApplication.shared.open(url, options: [:], completionHandler: nil)
                }
                decisionHandler(.cancel)
                return
            }
        }
        
        private func navigationTypeString(_ type: WKNavigationType) -> String {
            switch type {
            case .linkActivated: return "Link Activated"
            case .formSubmitted: return "Form Submitted"
            case .backForward: return "Back/Forward"
            case .reload: return "Reload"
            case .formResubmitted: return "Form Resubmitted"
            case .other: return "Other"
            @unknown default: return "Unknown"
            }
        }
        
        // MARK: - WKUIDelegate
        
        func webView(_ webView: WKWebView, createWebViewWith configuration: WKWebViewConfiguration, for navigationAction: WKNavigationAction, windowFeatures: WKWindowFeatures) -> WKWebView? {
            // Handle target="_blank" links by loading in same webview
            if navigationAction.targetFrame == nil {
                webView.load(navigationAction.request)
            }
            return nil
        }
        
        func webView(_ webView: WKWebView, runJavaScriptAlertPanelWithMessage message: String, initiatedByFrame frame: WKFrameInfo, completionHandler: @escaping () -> Void) {
            DispatchQueue.main.async {
                if let scene = UIApplication.shared.connectedScenes.first as? UIWindowScene,
                   let rootVC = scene.windows.first?.rootViewController {
                    let alert = UIAlertController(title: PWAConfig.appName, message: message, preferredStyle: .alert)
                    alert.addAction(UIAlertAction(title: "OK", style: .default) { _ in
                        completionHandler()
                    })
                    rootVC.present(alert, animated: true)
                } else {
                    completionHandler()
                }
            }
        }
        
        func webView(_ webView: WKWebView, runJavaScriptConfirmPanelWithMessage message: String, initiatedByFrame frame: WKFrameInfo, completionHandler: @escaping (Bool) -> Void) {
            DispatchQueue.main.async {
                if let scene = UIApplication.shared.connectedScenes.first as? UIWindowScene,
                   let rootVC = scene.windows.first?.rootViewController {
                    let alert = UIAlertController(title: PWAConfig.appName, message: message, preferredStyle: .alert)
                    alert.addAction(UIAlertAction(title: "Cancel", style: .cancel) { _ in
                        completionHandler(false)
                    })
                    alert.addAction(UIAlertAction(title: "OK", style: .default) { _ in
                        completionHandler(true)
                    })
                    rootVC.present(alert, animated: true)
                } else {
                    completionHandler(false)
                }
            }
        }
        
        func webView(_ webView: WKWebView, runJavaScriptTextInputPanelWithPrompt prompt: String, defaultText: String?, initiatedByFrame frame: WKFrameInfo, completionHandler: @escaping (String?) -> Void) {
            DispatchQueue.main.async {
                if let scene = UIApplication.shared.connectedScenes.first as? UIWindowScene,
                   let rootVC = scene.windows.first?.rootViewController {
                    let alert = UIAlertController(title: PWAConfig.appName, message: prompt, preferredStyle: .alert)
                    alert.addTextField { textField in
                        textField.text = defaultText
                    }
                    alert.addAction(UIAlertAction(title: "Cancel", style: .cancel) { _ in
                        completionHandler(nil)
                    })
                    alert.addAction(UIAlertAction(title: "OK", style: .default) { _ in
                        completionHandler(alert.textFields?.first?.text)
                    })
                    rootVC.present(alert, animated: true)
                } else {
                    completionHandler(nil)
                }
            }
        }
    }
}
