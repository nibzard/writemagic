import SwiftUI

struct WritingView: View {
    @State private var documentContent = ""
    @State private var isPaneMode = false
    @State private var showAIAssistant = false
    @State private var alternativeContent = ""
    
    var body: some View {
        NavigationView {
            VStack {
                // Toolbar
                HStack {
                    Text("Document.md")
                        .font(.title2)
                        .fontWeight(.semibold)
                    
                    Spacer()
                    
                    HStack {
                        Button(action: { isPaneMode.toggle() }) {
                            Image(systemName: isPaneMode ? "rectangle.split.2x1" : "rectangle")
                                .font(.title3)
                        }
                        
                        Button(action: { showAIAssistant.toggle() }) {
                            Image(systemName: "brain.head.profile")
                                .font(.title3)
                        }
                    }
                }
                .padding(.horizontal)
                .padding(.bottom)
                
                // Writing area
                if isPaneMode {
                    HStack(spacing: 8) {
                        // Main pane
                        WritingPane(
                            content: $documentContent,
                            title: "Main Draft"
                        )
                        
                        // Alternative pane
                        WritingPane(
                            content: $alternativeContent,
                            title: "Alternative"
                        )
                    }
                    .padding(.horizontal)
                } else {
                    WritingPane(
                        content: $documentContent,
                        title: "Document"
                    )
                    .padding(.horizontal)
                }
                
                Spacer()
            }
            .navigationBarHidden(true)
            .sheet(isPresented: $showAIAssistant) {
                AIAssistantSheet(
                    onSuggestion: { suggestion in
                        documentContent += "\n\n\(suggestion)"
                    }
                )
            }
        }
        .navigationViewStyle(StackNavigationViewStyle())
    }
}

struct WritingPane: View {
    @Binding var content: String
    let title: String
    
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(title)
                .font(.headline)
                .foregroundColor(.secondary)
            
            ZStack(alignment: .topLeading) {
                RoundedRectangle(cornerRadius: 12)
                    .fill(Color(.systemGray6))
                    .overlay(
                        RoundedRectangle(cornerRadius: 12)
                            .stroke(Color(.systemGray4), lineWidth: 1)
                    )
                
                if content.isEmpty {
                    Text("Start writing...")
                        .foregroundColor(.secondary)
                        .padding(.horizontal, 16)
                        .padding(.vertical, 12)
                }
                
                TextEditor(text: $content)
                    .padding(.horizontal, 12)
                    .padding(.vertical, 8)
                    .background(Color.clear)
                    .font(.system(size: 16, design: .serif))
                    .lineSpacing(4)
            }
        }
    }
}

struct AIAssistantSheet: View {
    @Environment(\.dismiss) private var dismiss
    let onSuggestion: (String) -> Void
    
    var body: some View {
        NavigationView {
            VStack(spacing: 20) {
                Text("AI Writing Assistant")
                    .font(.title2)
                    .fontWeight(.semibold)
                
                VStack(spacing: 12) {
                    AIActionButton(
                        title: "Continue Writing",
                        description: "Continue from where you left off",
                        icon: "arrow.right.circle"
                    ) {
                        onSuggestion("Continue writing from here...")
                        dismiss()
                    }
                    
                    AIActionButton(
                        title: "Improve Clarity",
                        description: "Make your writing clearer",
                        icon: "sparkles"
                    ) {
                        onSuggestion("Rewrite this section with more clarity...")
                        dismiss()
                    }
                    
                    AIActionButton(
                        title: "Add Examples",
                        description: "Include supporting examples",
                        icon: "plus.circle"
                    ) {
                        onSuggestion("Add supporting examples...")
                        dismiss()
                    }
                    
                    AIActionButton(
                        title: "Alternative Phrasing",
                        description: "Suggest different ways to express ideas",
                        icon: "arrow.2.squarepath"
                    ) {
                        onSuggestion("Here's an alternative approach...")
                        dismiss()
                    }
                }
                
                Spacer()
            }
            .padding()
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("Done") {
                        dismiss()
                    }
                }
            }
        }
    }
}

struct AIActionButton: View {
    let title: String
    let description: String
    let icon: String
    let action: () -> Void
    
    var body: some View {
        Button(action: action) {
            HStack {
                Image(systemName: icon)
                    .font(.title2)
                    .foregroundColor(.purple)
                    .frame(width: 30)
                
                VStack(alignment: .leading, spacing: 2) {
                    Text(title)
                        .font(.headline)
                        .foregroundColor(.primary)
                    
                    Text(description)
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                
                Spacer()
                
                Image(systemName: "chevron.right")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            .padding()
            .background(
                RoundedRectangle(cornerRadius: 12)
                    .fill(Color(.systemGray6))
            )
        }
        .buttonStyle(PlainButtonStyle())
    }
}

#Preview {
    WritingView()
}