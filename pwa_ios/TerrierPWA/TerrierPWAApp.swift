import SwiftUI

@main
struct TerrierPWAApp: App {
    @StateObject private var appState = AppState()
    @StateObject private var networkMonitor = NetworkMonitor.shared
    
    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(appState)
                .onAppear {
                    setupAppearance()
                }
                .onOpenURL { url in
                    handleDeepLink(url)
                }
                // Handle Universal Links (HTTPS links that open the app)
                .onContinueUserActivity(NSUserActivityTypeBrowsingWeb) { userActivity in
                    handleUniversalLink(userActivity)
                }
                // Monitor network status changes
                .onReceive(networkMonitor.$isConnected) { isConnected in
                    appState.isOffline = !isConnected
                }
        }
    }
    
    private func setupAppearance() {
        // Configure status bar and navigation bar appearance
        UINavigationBar.appearance().tintColor = UIColor(Color.accentColor)
    }
    
    private func handleUniversalLink(_ userActivity: NSUserActivity) {
        guard let url = userActivity.webpageURL else {
            print("[APP] ⚠️ Universal Link activity without URL")
            return
        }
        
        print("[APP] 🔗 Received Universal Link: \(url.absoluteString)")
        
        // If this is a terrier.scottylabs.org URL, load it in the WebView
        if let host = url.host?.lowercased(), host.contains("scottylabs.org") {
            print("[APP] ✅ Loading Universal Link in WebView")
            appState.pendingURL = url
        }
    }
    
    private func handleDeepLink(_ url: URL) {
        print("[APP] 🔗 Received deep link: \(url.absoluteString)")
        
        // Handle HTTPS Universal Links from Safari after OAuth
        if url.scheme == "https" {
            if let host = url.host?.lowercased(), host.contains("scottylabs.org") {
                print("[APP] 🔐 Universal Link after auth - loading in WebView")
                appState.pendingURL = url
                return
            }
        }
        
        // Handle terrier:// URLs by converting to web URLs
        if url.scheme == "terrier" {
            // Build the path from host + path (terrier://auth/callback -> auth/callback)
            var pathComponents = [String]()
            if let host = url.host {
                pathComponents.append(host)
            }
            if !url.path.isEmpty && url.path != "/" {
                pathComponents.append(String(url.path.dropFirst())) // Remove leading /
            }
            let path = pathComponents.joined(separator: "/")
            
            // Construct the web URL
            var webURLString = "https://terrier.scottylabs.org/\(path)"
            if let query = url.query {
                webURLString += "?\(query)"
            }
            
            print("[APP] 🔄 Converting to: \(webURLString)")
            
            if let webURL = URL(string: webURLString) {
                // Store the URL to be loaded - the ContentView will pick this up
                appState.pendingURL = webURL
            }
        }
    }
}

/// Global app state management
class AppState: ObservableObject {
    @Published var isLoading = true
    @Published var hasError = false
    @Published var errorMessage: String?
    @Published var isOffline = false
    @Published var pendingURL: URL? = nil
    
    func setError(_ message: String) {
        DispatchQueue.main.async {
            self.hasError = true
            self.errorMessage = message
        }
    }
    
    func clearError() {
        DispatchQueue.main.async {
            self.hasError = false
            self.errorMessage = nil
        }
    }
}
