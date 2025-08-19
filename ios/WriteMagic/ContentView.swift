import SwiftUI

struct ContentView: View {
    @State private var selectedTab = 0
    @State private var isInitialized = false
    @State private var initializationMessage = "Initializing WriteMagic..."
    
    var body: some View {
        Group {
            if isInitialized {
                TabView(selection: $selectedTab) {
                    WritingView()
                        .tabItem {
                            Image(systemName: "square.and.pencil")
                            Text("Writing")
                        }
                        .tag(0)
                    
                    ProjectsView()
                        .tabItem {
                            Image(systemName: "folder")
                            Text("Projects")
                        }
                        .tag(1)
                    
                    AIView()
                        .tabItem {
                            Image(systemName: "brain.head.profile")
                            Text("AI")
                        }
                        .tag(2)
                    
                    TimelineView()
                        .tabItem {
                            Image(systemName: "timeline.selection")
                            Text("Timeline")
                        }
                        .tag(3)
                    
                    SettingsView()
                        .tabItem {
                            Image(systemName: "gear")
                            Text("Settings")
                        }
                        .tag(4)
                }
                .accentColor(.purple)
            } else {
                VStack {
                    ProgressView()
                    Text(initializationMessage)
                        .padding()
                }
            }
        }
        .onAppear {
            initializeCore()
        }
    }
    
    private func initializeCore() {
        Task {
            let success = await WriteMagicCore.initialize()
            
            await MainActor.run {
                if success {
                    initializationMessage = "WriteMagic initialized successfully!"
                    isInitialized = true
                } else {
                    initializationMessage = "Failed to initialize WriteMagic core"
                }
            }
        }
    }
}

#Preview {
    ContentView()
}