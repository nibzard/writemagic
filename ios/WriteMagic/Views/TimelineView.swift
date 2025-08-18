import SwiftUI

struct TimelineView: View {
    @State private var commits: [GitCommit] = [
        GitCommit(id: "abc123", message: "Initial draft of chapter 1", author: "You", timestamp: "2 hours ago", branch: "main", filesChanged: 1),
        GitCommit(id: "def456", message: "Add character descriptions", author: "You", timestamp: "4 hours ago", branch: "characters", filesChanged: 3),
        GitCommit(id: "ghi789", message: "Revise opening paragraph", author: "You", timestamp: "1 day ago", branch: "main", filesChanged: 1),
        GitCommit(id: "jkl012", message: "Experiment with different ending", author: "You", timestamp: "2 days ago", branch: "alternative-ending", filesChanged: 2),
        GitCommit(id: "mno345", message: "Add dialogue scenes", author: "You", timestamp: "3 days ago", branch: "main", filesChanged: 4)
    ]
    @State private var selectedBranch = "all"
    
    private var branches: [String] {
        let allBranches = commits.map { $0.branch }
        return ["all"] + Array(Set(allBranches)).sorted()
    }
    
    private var filteredCommits: [GitCommit] {
        if selectedBranch == "all" {
            return commits
        } else {
            return commits.filter { $0.branch == selectedBranch }
        }
    }
    
    var body: some View {
        NavigationView {
            VStack {
                // Branch filter
                VStack(alignment: .leading) {
                    HStack {
                        Text("Filter by branch:")
                            .font(.subheadline)
                            .foregroundColor(.secondary)
                        
                        Spacer()
                    }
                    
                    ScrollView(.horizontal, showsIndicators: false) {
                        HStack(spacing: 8) {
                            ForEach(branches, id: \.self) { branch in
                                BranchChip(
                                    branch: branch,
                                    isSelected: selectedBranch == branch
                                ) {
                                    selectedBranch = branch
                                }
                            }
                        }
                        .padding(.horizontal)
                    }
                }
                .padding()
                .background(Color(.systemGray6))
                
                // Timeline visualization
                TimelineVisualization(commits: filteredCommits)
                    .frame(height: 150)
                    .padding()
                
                // Commits list
                List(filteredCommits) { commit in
                    CommitRow(commit: commit)
                }
                .listStyle(PlainListStyle())
            }
            .navigationTitle("Timeline")
        }
        .navigationViewStyle(StackNavigationViewStyle())
    }
}

struct BranchChip: View {
    let branch: String
    let isSelected: Bool
    let action: () -> Void
    
    var body: some View {
        Button(action: action) {
            HStack(spacing: 4) {
                if branch != "all" {
                    Image(systemName: "arrow.triangle.branch")
                        .font(.caption2)
                }
                
                Text(branch)
                    .font(.caption)
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 6)
            .background(
                isSelected
                    ? Color.purple.opacity(0.2)
                    : Color(.systemBackground)
            )
            .foregroundColor(
                isSelected
                    ? .purple
                    : .primary
            )
            .cornerRadius(12)
            .overlay(
                RoundedRectangle(cornerRadius: 12)
                    .stroke(Color(.systemGray4), lineWidth: 1)
            )
        }
    }
}

struct TimelineVisualization: View {
    let commits: [GitCommit]
    
    var body: some View {
        Canvas { context, size in
            let mainLineY = size.height / 2
            let lineColor = Color.purple
            
            // Draw main timeline
            let mainLine = Path { path in
                path.move(to: CGPoint(x: 30, y: mainLineY))
                path.addLine(to: CGPoint(x: size.width - 30, y: mainLineY))
            }
            
            context.stroke(mainLine, with: .color(lineColor), lineWidth: 2)
            
            // Draw commit points
            for (index, commit) in commits.enumerated() {
                let x = 30 + (CGFloat(index) * (size.width - 60) / CGFloat(max(1, commits.count - 1)))
                let branchColor = getBranchColor(for: commit.branch)
                
                if commit.branch != "main" {
                    // Draw branch line
                    let branchY = mainLineY + (commit.branch.hashCode % 2 == 0 ? -30 : 30)
                    let branchLine = Path { path in
                        path.move(to: CGPoint(x: x, y: mainLineY))
                        path.addLine(to: CGPoint(x: x, y: branchY))
                    }
                    context.stroke(branchLine, with: .color(branchColor), lineWidth: 1.5)
                    
                    // Draw commit point on branch
                    context.fill(
                        Path(ellipseIn: CGRect(x: x - 4, y: branchY - 4, width: 8, height: 8)),
                        with: .color(branchColor)
                    )
                } else {
                    // Draw commit point on main line
                    context.fill(
                        Path(ellipseIn: CGRect(x: x - 4, y: mainLineY - 4, width: 8, height: 8)),
                        with: .color(branchColor)
                    )
                }
            }
        }
    }
    
    private func getBranchColor(for branch: String) -> Color {
        switch branch {
        case "main":
            return .purple
        case "characters":
            return .green
        case "alternative-ending":
            return .orange
        default:
            return .blue
        }
    }
}

struct CommitRow: View {
    let commit: GitCommit
    
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    Text(commit.message)
                        .font(.headline)
                    
                    HStack(spacing: 16) {
                        Label(commit.branch, systemImage: "arrow.triangle.branch")
                            .font(.caption)
                            .foregroundColor(.secondary)
                        
                        Label("\(commit.filesChanged) files", systemImage: "doc.text")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                }
                
                Spacer()
                
                VStack(alignment: .trailing, spacing: 4) {
                    Text(commit.timestamp)
                        .font(.caption)
                        .foregroundColor(.secondary)
                    
                    Text(String(commit.id.prefix(7)))
                        .font(.caption)
                        .foregroundColor(.secondary)
                        .monospaced()
                }
            }
        }
        .padding(.vertical, 4)
    }
}

extension String {
    var hashCode: Int {
        return self.hash
    }
}

#Preview {
    TimelineView()
}