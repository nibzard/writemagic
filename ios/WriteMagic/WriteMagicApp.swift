import SwiftUI

@main
struct WriteMagicApp: App {
    
    init() {
        // Initialize Rust core on app startup
        RustFFI.shared.initializeCore()
    }
    
    var body: some Scene {
        WindowGroup {
            ContentView()
                .preferredColorScheme(.automatic)
        }
    }
}