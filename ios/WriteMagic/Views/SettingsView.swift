import SwiftUI

struct SettingsView: View {
    @State private var darkModeEnabled = false
    @State private var autoSaveEnabled = true
    @State private var aiProvider = "Claude"
    @State private var syncEnabled = false
    
    var body: some View {
        NavigationView {
            List {
                // Appearance Section
                Section("Appearance") {
                    SettingsRow(
                        title: "Dark Mode",
                        subtitle: "Use dark theme throughout the app",
                        icon: "moon.fill"
                    ) {
                        Toggle("", isOn: $darkModeEnabled)
                    }
                }
                
                // Writing Section
                Section("Writing") {
                    SettingsRow(
                        title: "Auto Save",
                        subtitle: "Automatically save documents while writing",
                        icon: "square.and.arrow.down"
                    ) {
                        Toggle("", isOn: $autoSaveEnabled)
                    }
                    
                    SettingsRow(
                        title: "Word Count Goal",
                        subtitle: "Set daily writing goals",
                        icon: "target"
                    ) {
                        Image(systemName: "chevron.right")
                            .foregroundColor(.secondary)
                            .font(.caption)
                    }
                }
                
                // AI Section
                Section("AI Assistant") {
                    SettingsRow(
                        title: "AI Provider",
                        subtitle: "Current: \(aiProvider)",
                        icon: "brain.head.profile"
                    ) {
                        Image(systemName: "chevron.right")
                            .foregroundColor(.secondary)
                            .font(.caption)
                    }
                    
                    SettingsRow(
                        title: "API Configuration",
                        subtitle: "Configure API keys and settings",
                        icon: "key"
                    ) {
                        Image(systemName: "chevron.right")
                            .foregroundColor(.secondary)
                            .font(.caption)
                    }
                    
                    SettingsRow(
                        title: "Usage & Billing",
                        subtitle: "View API usage and costs",
                        icon: "receipt"
                    ) {
                        Image(systemName: "chevron.right")
                            .foregroundColor(.secondary)
                            .font(.caption)
                    }
                }
                
                // Sync Section
                Section("Sync & Backup") {
                    SettingsRow(
                        title: "Cloud Sync",
                        subtitle: "Sync documents across devices",
                        icon: "icloud"
                    ) {
                        Toggle("", isOn: $syncEnabled)
                    }
                    
                    SettingsRow(
                        title: "Export Data",
                        subtitle: "Export all documents and projects",
                        icon: "square.and.arrow.up"
                    ) {
                        Image(systemName: "chevron.right")
                            .foregroundColor(.secondary)
                            .font(.caption)
                    }
                    
                    SettingsRow(
                        title: "Import Data",
                        subtitle: "Import documents from other apps",
                        icon: "square.and.arrow.down.on.square"
                    ) {
                        Image(systemName: "chevron.right")
                            .foregroundColor(.secondary)
                            .font(.caption)
                    }
                }
                
                // Support Section
                Section("Support") {
                    SettingsRow(
                        title: "Help & Tutorials",
                        subtitle: "Learn how to use WriteMagic",
                        icon: "questionmark.circle"
                    ) {
                        Image(systemName: "chevron.right")
                            .foregroundColor(.secondary)
                            .font(.caption)
                    }
                    
                    SettingsRow(
                        title: "Send Feedback",
                        subtitle: "Report bugs or suggest features",
                        icon: "envelope"
                    ) {
                        Image(systemName: "chevron.right")
                            .foregroundColor(.secondary)
                            .font(.caption)
                    }
                    
                    SettingsRow(
                        title: "About",
                        subtitle: "Version info and credits",
                        icon: "info.circle"
                    ) {
                        Image(systemName: "chevron.right")
                            .foregroundColor(.secondary)
                            .font(.caption)
                    }
                }
            }
            .navigationTitle("Settings")
        }
        .navigationViewStyle(StackNavigationViewStyle())
    }
}

struct SettingsRow<Content: View>: View {
    let title: String
    let subtitle: String
    let icon: String
    let content: () -> Content
    
    var body: some View {
        HStack {
            Image(systemName: icon)
                .foregroundColor(.purple)
                .frame(width: 24, height: 24)
            
            VStack(alignment: .leading, spacing: 2) {
                Text(title)
                    .font(.body)
                
                Text(subtitle)
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            
            Spacer()
            
            content()
        }
        .padding(.vertical, 2)
    }
}

#Preview {
    SettingsView()
}