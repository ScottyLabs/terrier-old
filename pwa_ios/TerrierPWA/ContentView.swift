import SwiftUI

struct ContentView: View {
    @EnvironmentObject var appState: AppState
    @StateObject private var webViewState = WebViewState()
    
    /// Combined offline state - true if either NetworkMonitor or WebView detects offline
    private var isOffline: Bool {
        appState.isOffline || webViewState.isOffline
    }
    
    var body: some View {
        ZStack {
            // Main PWA WebView
            PWAWebView(state: webViewState)
                .edgesIgnoringSafeArea(.all)
            
            // Loading overlay
            if appState.isLoading && !isOffline {
                LoadingView()
            }
            
            // Error overlay (only for non-network errors)
            if appState.hasError && !isOffline {
                ErrorView(message: appState.errorMessage ?? "An error occurred") {
                    appState.clearError()
                    webViewState.reload()
                }
            }
            
            // Offline fullscreen fallback - takes priority
            if isOffline {
                OfflineView {
                    // Clear offline states and retry
                    appState.isOffline = false
                    webViewState.isOffline = false
                    webViewState.reload()
                }
                .transition(.opacity)
            }
        }
        .onReceive(webViewState.$isLoading) { isLoading in
            appState.isLoading = isLoading
        }
        .onReceive(webViewState.$error) { error in
            if let error = error {
                appState.setError(error)
            }
        }
        .onReceive(appState.$pendingURL) { url in
            // Handle deep link URLs passed from the app
            if let url = url {
                print("[UI] 🔗 Loading pending URL: \(url.absoluteString)")
                // Sync cookies before loading the deep link to ensure session is present
                webViewState.syncCookies {
                    webViewState.webView?.load(URLRequest(url: url))
                    appState.pendingURL = nil
                }
            }
        }
    }
}

struct LoadingView: View {
    var body: some View {
        ZStack {
            Color.black.opacity(0.3)
                .edgesIgnoringSafeArea(.all)
            
            VStack(spacing: 16) {
                ProgressView()
                    .progressViewStyle(CircularProgressViewStyle(tint: .white))
                    .scaleEffect(1.5)
                
                Text("Loading Terrier...")
                    .foregroundColor(.white)
                    .font(.headline)
            }
            .padding(32)
            .background(Color.black.opacity(0.7))
            .cornerRadius(16)
        }
    }
}

struct ErrorView: View {
    let message: String
    let onRetry: () -> Void
    
    var body: some View {
        ZStack {
            Color.black.opacity(0.5)
                .edgesIgnoringSafeArea(.all)
            
            VStack(spacing: 20) {
                Image(systemName: "exclamationmark.triangle.fill")
                    .font(.system(size: 48))
                    .foregroundColor(.yellow)
                
                Text("Connection Error")
                    .font(.title2)
                    .fontWeight(.bold)
                    .foregroundColor(.white)
                
                Text(message)
                    .font(.body)
                    .foregroundColor(.white.opacity(0.8))
                    .multilineTextAlignment(.center)
                    .padding(.horizontal)
                
                Button(action: onRetry) {
                    HStack {
                        Image(systemName: "arrow.clockwise")
                        Text("Retry")
                    }
                    .padding(.horizontal, 32)
                    .padding(.vertical, 12)
                    .background(Color.blue)
                    .foregroundColor(.white)
                    .cornerRadius(8)
                }
            }
            .padding(32)
            .background(Color(UIColor.systemBackground).opacity(0.95))
            .cornerRadius(20)
            .shadow(radius: 20)
            .padding(40)
        }
    }
}

struct OfflineBanner: View {
    var body: some View {
        HStack {
            Image(systemName: "wifi.slash")
            Text("You're offline")
            Spacer()
        }
        .padding(.horizontal)
        .padding(.vertical, 8)
        .background(Color.orange)
        .foregroundColor(.white)
        .font(.footnote.weight(.medium))
    }
}

/// Full-screen offline fallback with TartanHacks theming
struct OfflineView: View {
    let onRetry: () -> Void
    
    // TartanHacks brand colors
    private let gradientStart = Color(red: 0.1, green: 0.1, blue: 0.2)
    private let gradientEnd = Color(red: 0.05, green: 0.05, blue: 0.15)
    private let accentBlue = Color(red: 0.3, green: 0.5, blue: 1.0)
    private let accentPurple = Color(red: 0.6, green: 0.4, blue: 1.0)
    
    var body: some View {
        ZStack {
            // Gradient background
            LinearGradient(
                gradient: Gradient(colors: [gradientStart, gradientEnd]),
                startPoint: .top,
                endPoint: .bottom
            )
            .edgesIgnoringSafeArea(.all)
            
            // Subtle pattern overlay
            GeometryReader { geometry in
                Path { path in
                    let gridSize: CGFloat = 30
                    for x in stride(from: 0, to: geometry.size.width, by: gridSize) {
                        path.move(to: CGPoint(x: x, y: 0))
                        path.addLine(to: CGPoint(x: x, y: geometry.size.height))
                    }
                    for y in stride(from: 0, to: geometry.size.height, by: gridSize) {
                        path.move(to: CGPoint(x: 0, y: y))
                        path.addLine(to: CGPoint(x: geometry.size.width, y: y))
                    }
                }
                .stroke(Color.white.opacity(0.03), lineWidth: 1)
            }
            
            VStack(spacing: 32) {
                Spacer()
                
                // Animated WiFi icon
                ZStack {
                    // Glow effect
                    Circle()
                        .fill(
                            RadialGradient(
                                gradient: Gradient(colors: [accentBlue.opacity(0.3), Color.clear]),
                                center: .center,
                                startRadius: 40,
                                endRadius: 100
                            )
                        )
                        .frame(width: 200, height: 200)
                    
                    // Icon background
                    Circle()
                        .fill(
                            LinearGradient(
                                gradient: Gradient(colors: [accentBlue.opacity(0.2), accentPurple.opacity(0.2)]),
                                startPoint: .topLeading,
                                endPoint: .bottomTrailing
                            )
                        )
                        .frame(width: 120, height: 120)
                    
                    // WiFi slash icon
                    Image(systemName: "wifi.slash")
                        .font(.system(size: 48, weight: .medium))
                        .foregroundStyle(
                            LinearGradient(
                                gradient: Gradient(colors: [accentBlue, accentPurple]),
                                startPoint: .topLeading,
                                endPoint: .bottomTrailing
                            )
                        )
                }
                
                // Title
                VStack(spacing: 12) {
                    Text("No Connection")
                        .font(.system(size: 28, weight: .bold, design: .rounded))
                        .foregroundColor(.white)
                    
                    Text("Check your internet connection\nand try again")
                        .font(.system(size: 16, weight: .regular))
                        .foregroundColor(.white.opacity(0.6))
                        .multilineTextAlignment(.center)
                        .lineSpacing(4)
                }
                
                Spacer()
                
                // Retry button
                Button(action: onRetry) {
                    HStack(spacing: 10) {
                        Image(systemName: "arrow.clockwise")
                            .font(.system(size: 16, weight: .semibold))
                        Text("Try Again")
                            .font(.system(size: 16, weight: .semibold))
                    }
                    .foregroundColor(.white)
                    .padding(.horizontal, 40)
                    .padding(.vertical, 16)
                    .background(
                        LinearGradient(
                            gradient: Gradient(colors: [accentBlue, accentPurple]),
                            startPoint: .leading,
                            endPoint: .trailing
                        )
                    )
                    .cornerRadius(30)
                    .shadow(color: accentBlue.opacity(0.4), radius: 20, x: 0, y: 10)
                }
                
                Spacer()
                    .frame(height: 60)
            }
            .padding(.horizontal, 40)
        }
    }
}

#Preview {
    ContentView()
        .environmentObject(AppState())
}
