import SwiftUI

struct ContentView: View {
    @State private var selectedTab = 0
    
    var body: some View {
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
    }
}

#Preview {
    ContentView()
}