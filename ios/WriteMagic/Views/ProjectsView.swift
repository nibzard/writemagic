import SwiftUI

struct ProjectsView: View {
    @State private var projects: [Project] = [
        Project(id: "1", name: "Novel Draft", description: "My first novel project", documentsCount: 12, lastModified: "2 hours ago"),
        Project(id: "2", name: "Blog Posts", description: "Collection of blog posts", documentsCount: 8, lastModified: "1 day ago"),
        Project(id: "3", name: "Technical Docs", description: "API documentation", documentsCount: 5, lastModified: "3 days ago")
    ]
    @State private var showCreateSheet = false
    
    var body: some View {
        NavigationView {
            List {
                ForEach(projects) { project in
                    ProjectRow(project: project)
                        .onTapGesture {
                            // Navigate to project details
                        }
                }
                .onDelete(perform: deleteProjects)
            }
            .navigationTitle("Projects")
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button(action: { showCreateSheet = true }) {
                        Image(systemName: "plus")
                    }
                }
            }
            .sheet(isPresented: $showCreateSheet) {
                CreateProjectSheet { name, description in
                    let newProject = Project(
                        id: UUID().uuidString,
                        name: name,
                        description: description,
                        documentsCount: 0,
                        lastModified: "Just now"
                    )
                    projects.append(newProject)
                }
            }
        }
        .navigationViewStyle(StackNavigationViewStyle())
    }
    
    private func deleteProjects(offsets: IndexSet) {
        projects.remove(atOffsets: offsets)
    }
}

struct ProjectRow: View {
    let project: Project
    
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    Text(project.name)
                        .font(.headline)
                    
                    Text(project.description)
                        .font(.subheadline)
                        .foregroundColor(.secondary)
                        .lineLimit(2)
                }
                
                Spacer()
                
                Button(action: {
                    // Show options menu
                }) {
                    Image(systemName: "ellipsis")
                        .foregroundColor(.secondary)
                }
            }
            
            HStack {
                Label("\(project.documentsCount) documents", systemImage: "doc.text")
                    .font(.caption)
                    .foregroundColor(.secondary)
                
                Spacer()
                
                Text(project.lastModified)
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
        }
        .padding(.vertical, 4)
    }
}

struct CreateProjectSheet: View {
    @Environment(\.dismiss) private var dismiss
    @State private var name = ""
    @State private var description = ""
    
    let onCreate: (String, String) -> Void
    
    var body: some View {
        NavigationView {
            Form {
                Section("Project Details") {
                    TextField("Project Name", text: $name)
                    
                    TextField("Description", text: $description, axis: .vertical)
                        .lineLimit(3...6)
                }
            }
            .navigationTitle("New Project")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarLeading) {
                    Button("Cancel") {
                        dismiss()
                    }
                }
                
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("Create") {
                        onCreate(name, description)
                        dismiss()
                    }
                    .disabled(name.isEmpty)
                }
            }
        }
    }
}

#Preview {
    ProjectsView()
}