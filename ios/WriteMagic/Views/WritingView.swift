import SwiftUI

struct WritingView: View {
    @State private var documentContent = ""
    @State private var documentTitle = "New Document"
    @State private var isPaneMode = false
    @State private var showAIAssistant = false
    @State private var alternativeContent = ""
    @State private var isDistractionFreeMode = false
    @State private var isSaving = false
    @State private var statusMessage = ""
    @State private var showStatusMessage = false
    @State private var wordCount = 0
    @State private var characterCount = 0
    @State private var currentDocumentId: String?
    @FocusState private var isTextEditorFocused: Bool
    
    var body: some View {
        GeometryReader { geometry in
            ZStack {
                // Main content
                VStack(spacing: 0) {
                    // Enhanced toolbar with animation
                    if !isDistractionFreeMode {
                        EnhancedWritingToolbar(
                            documentTitle: $documentTitle,
                            isPaneMode: $isPaneMode,
                            showAIAssistant: $showAIAssistant,
                            isDistractionFreeMode: $isDistractionFreeMode,
                            onNewDocument: createNewDocument
                        )
                        .padding(.horizontal)
                        .padding(.bottom, 8)
                        .transition(.move(edge: .top).combined(with: .opacity))
                    }
                    
                    // Main writing area with gesture support
                    ZStack {
                        if isPaneMode {
                            HStack(spacing: 12) {
                                // Main pane
                                EnhancedWritingPane(
                                    content: $documentContent,
                                    title: "Main Draft",
                                    isDistractionFreeMode: isDistractionFreeMode,
                                    isFocused: $isTextEditorFocused
                                )
                                
                                // Alternative pane
                                EnhancedWritingPane(
                                    content: $alternativeContent,
                                    title: "Alternative",
                                    isDistractionFreeMode: isDistractionFreeMode
                                )
                            }
                            .padding(.horizontal, isDistractionFreeMode ? 4 : 16)
                        } else {
                            EnhancedWritingPane(
                                content: $documentContent,
                                title: isDistractionFreeMode ? "" : "Document",
                                isDistractionFreeMode: isDistractionFreeMode,
                                isFocused: $isTextEditorFocused
                            )
                            .padding(.horizontal, isDistractionFreeMode ? 8 : 16)
                        }
                    }
                    .gesture(
                        DragGesture()
                            .onEnded { value in
                                if abs(value.translation.x) > abs(value.translation.y) && abs(value.translation.x) > 100 {
                                    withAnimation(.spring(response: 0.6)) {
                                        isPaneMode.toggle()
                                    }
                                }
                            }
                    )
                }
                
                // Status overlay with auto-save indicator
                if showStatusMessage || isSaving {
                    VStack {
                        StatusIndicatorView(
                            message: statusMessage,
                            isSaving: isSaving
                        )
                        .padding(.top, 8)
                        .transition(.move(edge: .top).combined(with: .opacity))
                        
                        Spacer()
                    }
                }
                
                // Writing statistics overlay
                if !isDistractionFreeMode {
                    VStack {
                        Spacer()
                        
                        HStack {
                            WritingStatisticsView(
                                wordCount: wordCount,
                                characterCount: characterCount
                            )
                            .padding(.leading, 16)
                            .transition(.move(edge: .leading).combined(with: .opacity))
                            
                            Spacer()
                        }
                        .padding(.bottom, geometry.safeAreaInsets.bottom + 16)
                    }
                }
            }
        }
        .navigationBarHidden(true)
        .onChange(of: documentContent) { newValue in
            // Update statistics and trigger auto-save
            updateWritingStatistics(newValue)
            triggerAutoSave()
        }
        .sheet(isPresented: $showAIAssistant) {
            EnhancedAIAssistantSheet(
                currentContent: documentContent,
                onSuggestion: { suggestion in
                    documentContent += "\n\n\(suggestion)"
                },
                onReplaceSelection: { replacement in
                    // For iOS, we'll append for now - more complex selection replacement would need custom text editor
                    documentContent += "\n\n--- Replacement ---\n\(replacement)"
                }
            )
        }
    }
    
    // Helper functions
    private func createNewDocument() {
        // Integration with Rust core for document creation
        // For now, reset the current document
        documentContent = ""
        documentTitle = "New Document"
        currentDocumentId = nil
        statusMessage = "New document created"
        showStatusMessage = true
        DispatchQueue.main.asyncAfter(deadline: .now() + 2) {
            showStatusMessage = false
        }
    }
    
    private func updateWritingStatistics(_ content: String) {
        wordCount = content.split(whereSeparator: \.isWhitespace).count
        characterCount = content.count
    }
    
    private func triggerAutoSave() {
        // Auto-save logic with debouncing
        NSObject.cancelPreviousPerformRequests(target: self)
        perform(#selector(autoSave), with: nil, afterDelay: 1.0)
    }
    
    @objc private func autoSave() {
        guard !documentContent.isEmpty else { return }
        
        isSaving = true
        statusMessage = "Saving document..."
        showStatusMessage = true
        
        // Simulate save operation - integrate with Rust core
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
            isSaving = false
            statusMessage = "Document saved automatically"
            
            DispatchQueue.main.asyncAfter(deadline: .now() + 2) {
                showStatusMessage = false
            }
        }
    }
}

// Enhanced toolbar component for iOS
struct EnhancedWritingToolbar: View {
    @Binding var documentTitle: String
    @Binding var isPaneMode: Bool
    @Binding var showAIAssistant: Bool
    @Binding var isDistractionFreeMode: Bool
    let onNewDocument: () -> Void
    
    var body: some View {
        VStack(spacing: 12) {
            HStack {
                // Document title section
                HStack {
                    Image(systemName: "doc.text")
                        .foregroundColor(.accentColor)
                        .font(.title3)
                    
                    TextField("Document Title", text: $documentTitle)
                        .font(.title2)
                        .fontWeight(.semibold)
                        .textFieldStyle(PlainTextFieldStyle())
                        .accessibilityLabel("Document title, editable")
                }
                
                Spacer()
                
                // Action buttons
                HStack(spacing: 8) {
                    Button(action: onNewDocument) {
                        Image(systemName: "plus.circle.fill")
                            .font(.title2)
                            .foregroundColor(.accentColor)
                    }
                    .accessibilityLabel("Create new document")
                    
                    Button(action: { withAnimation(.spring()) { isPaneMode.toggle() } }) {
                        Image(systemName: isPaneMode ? "rectangle.split.2x1.fill" : "rectangle.fill")
                            .font(.title2)
                            .foregroundColor(isPaneMode ? .accentColor : .secondary)
                    }
                    .accessibilityLabel(isPaneMode ? "Switch to single pane" : "Switch to multi-pane")
                    
                    Button(action: { withAnimation(.spring()) { isDistractionFreeMode.toggle() } }) {
                        Image(systemName: isDistractionFreeMode ? "eye.fill" : "eye.slash.fill")
                            .font(.title2)
                            .foregroundColor(isDistractionFreeMode ? .accentColor : .secondary)
                    }
                    .accessibilityLabel(isDistractionFreeMode ? "Exit focus mode" : "Enter focus mode")
                    
                    Button(action: { showAIAssistant.toggle() }) {
                        Image(systemName: "brain.head.profile")
                            .font(.title2)
                            .foregroundColor(showAIAssistant ? .accentColor : .secondary)
                    }
                    .accessibilityLabel("AI Writing Assistant")
                }
            }
        }
        .padding(.vertical, 12)
        .padding(.horizontal, 16)
        .background(
            RoundedRectangle(cornerRadius: 12)
                .fill(Color(.systemBackground))
                .shadow(color: .black.opacity(0.1), radius: 2, x: 0, y: 1)
        )
    }
}

// Enhanced writing pane with better accessibility and typography
struct EnhancedWritingPane: View {
    @Binding var content: String
    let title: String
    var isDistractionFreeMode: Bool = false
    var isFocused: FocusState<Bool>.Binding?
    
    var body: some View {
        VStack(alignment: .leading, spacing: isDistractionFreeMode ? 4 : 12) {
            if !title.isEmpty {
                Text(title)
                    .font(.headline)
                    .fontWeight(.medium)
                    .foregroundColor(.accentColor)
                    .accessibilityLabel("Writing pane: \(title)")
            }
            
            ZStack(alignment: .topLeading) {
                RoundedRectangle(cornerRadius: isDistractionFreeMode ? 0 : 16)
                    .fill(isDistractionFreeMode ? Color.clear : Color(.systemGray6))
                    .overlay(
                        RoundedRectangle(cornerRadius: isDistractionFreeMode ? 0 : 16)
                            .stroke(isDistractionFreeMode ? Color.clear : Color(.systemGray4), lineWidth: 1)
                    )
                
                if content.isEmpty {
                    Text("Start writing your thoughts...")
                        .foregroundColor(.secondary)
                        .font(.system(size: isDistractionFreeMode ? 18 : 16, design: .serif))
                        .padding(.horizontal, isDistractionFreeMode ? 16 : 20)
                        .padding(.vertical, isDistractionFreeMode ? 20 : 16)
                        .accessibilityLabel("Placeholder text: Start writing your thoughts")
                }
                
                TextEditor(text: $content)
                    .padding(.horizontal, isDistractionFreeMode ? 12 : 16)
                    .padding(.vertical, isDistractionFreeMode ? 16 : 12)
                    .background(Color.clear)
                    .font(.system(size: isDistractionFreeMode ? 18 : 16, design: .serif))
                    .lineSpacing(isDistractionFreeMode ? 8 : 6)
                    .focused(isFocused ?? FocusState<Bool>().projectedValue)
                    .accessibilityLabel("Main text editor")
                    .accessibilityValue(content.isEmpty ? "Empty document" : "Document with \(content.count) characters")
            }
        }
    }
}

// Status indicator for iOS
struct StatusIndicatorView: View {
    let message: String
    let isSaving: Bool
    
    var body: some View {
        HStack(spacing: 8) {
            if isSaving {
                ProgressView()
                    .scaleEffect(0.8)
                    .progressViewStyle(CircularProgressViewStyle(tint: .white))
                Text("Saving...")
                    .font(.caption)
                    .fontWeight(.medium)
                    .foregroundColor(.white)
            } else {
                Image(systemName: "checkmark.circle.fill")
                    .foregroundColor(.white)
                    .font(.caption)
                Text(message)
                    .font(.caption)
                    .fontWeight(.medium)
                    .foregroundColor(.white)
            }
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
        .background(
            Capsule()
                .fill(isSaving ? Color.blue : Color.green)
        )
    }
}

// Writing statistics view
struct WritingStatisticsView: View {
    let wordCount: Int
    let characterCount: Int
    
    var body: some View {
        HStack(spacing: 16) {
            VStack(alignment: .center, spacing: 2) {
                Text("\(wordCount)")
                    .font(.caption)
                    .fontWeight(.bold)
                    .foregroundColor(.primary)
                Text("words")
                    .font(.caption2)
                    .foregroundColor(.secondary)
            }
            
            VStack(alignment: .center, spacing: 2) {
                Text("\(characterCount)")
                    .font(.caption)
                    .fontWeight(.bold)
                    .foregroundColor(.primary)
                Text("chars")
                    .font(.caption2)
                    .foregroundColor(.secondary)
            }
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
        .background(
            RoundedRectangle(cornerRadius: 8)
                .fill(Color(.systemGray5).opacity(0.8))
        )
    }
}

// Enhanced AI Assistant Sheet for iOS
struct EnhancedAIAssistantSheet: View {
    @Environment(\.dismiss) private var dismiss
    let currentContent: String
    let onSuggestion: (String) -> Void
    let onReplaceSelection: (String) -> Void
    
    @State private var customPrompt = ""
    @State private var isGenerating = false
    @State private var lastResponse = ""
    
    var body: some View {
        NavigationView {
            VStack(spacing: 0) {
                // Header
                VStack(spacing: 16) {
                    HStack {
                        Image(systemName: "brain.head.profile")
                            .foregroundColor(.accentColor)
                            .font(.title2)
                        Text("AI Writing Assistant")
                            .font(.title2)
                            .fontWeight(.semibold)
                        Spacer()
                    }
                    
                    // Custom prompt input
                    HStack {
                        TextField("Ask AI to help with your writing...", text: $customPrompt)
                            .textFieldStyle(RoundedBorderTextFieldStyle())
                            .disabled(isGenerating)
                            .accessibilityLabel("AI prompt input field")
                        
                        Button(action: sendCustomPrompt) {
                            if isGenerating {
                                ProgressView()
                                    .scaleEffect(0.8)
                                    .progressViewStyle(CircularProgressViewStyle())
                            } else {
                                Image(systemName: "paperplane.fill")
                                    .foregroundColor(.accentColor)
                            }
                        }
                        .disabled(customPrompt.isEmpty || isGenerating)
                    }
                }
                .padding()
                .background(Color(.systemGray6))
                
                // Quick actions
                ScrollView {
                    LazyVStack(spacing: 12) {
                        Text("Quick Actions")
                            .font(.headline)
                            .fontWeight(.medium)
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .padding(.top)
                        
                        AIActionCard(
                            title: "Continue Writing",
                            description: "Continue from where you left off",
                            icon: "arrow.right.circle",
                            isEnabled: !isGenerating && !currentContent.isEmpty
                        ) {
                            generateAIContent(prompt: "Continue writing from this text:\n\n\(currentContent.suffix(300))")
                        }
                        
                        AIActionCard(
                            title: "Improve Clarity",
                            description: "Make your writing clearer and more concise",
                            icon: "lightbulb",
                            isEnabled: !isGenerating && !currentContent.isEmpty
                        ) {
                            generateAIContent(prompt: "Rewrite this text with more clarity and better flow:\n\n\(currentContent.suffix(300))")
                        }
                        
                        AIActionCard(
                            title: "Add Examples",
                            description: "Include supporting examples and details",
                            icon: "plus.circle",
                            isEnabled: !isGenerating && !currentContent.isEmpty
                        ) {
                            generateAIContent(prompt: "Add supporting examples and details to this content:\n\n\(currentContent.suffix(300))")
                        }
                        
                        AIActionCard(
                            title: "Summarize",
                            description: "Create a concise summary of your content",
                            icon: "compress",
                            isEnabled: !isGenerating && currentContent.count > 100
                        ) {
                            generateAIContent(prompt: "Summarize this content in a few concise sentences:\n\n\(currentContent)")
                        }
                        
                        AIActionCard(
                            title: "Fix Grammar",
                            description: "Correct grammar and improve style",
                            icon: "checkmark.circle",
                            isEnabled: !isGenerating && !currentContent.isEmpty
                        ) {
                            generateAIContent(prompt: "Fix grammar, spelling, and improve the style of this text:\n\n\(currentContent)")
                        }
                        
                        AIActionCard(
                            title: "Generate Outline",
                            description: "Create a structured outline for your content",
                            icon: "list.bullet",
                            isEnabled: !isGenerating && !currentContent.isEmpty
                        ) {
                            generateAIContent(prompt: "Create a structured outline based on this content:\n\n\(currentContent)")
                        }
                    }
                    .padding()
                }
            }
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
    
    private func sendCustomPrompt() {
        guard !customPrompt.isEmpty && !isGenerating else { return }
        generateAIContent(prompt: "\(customPrompt)\n\nContext: \(currentContent.prefix(500))")
        customPrompt = ""
    }
    
    private func generateAIContent(prompt: String) {
        isGenerating = true
        
        // Simulate AI generation - integrate with Rust core
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.0) {
            let mockResponse = "This is a mock AI response for: \(prompt.prefix(50))..."
            onSuggestion(mockResponse)
            lastResponse = mockResponse
            isGenerating = false
            dismiss()
        }
    }
}

// Enhanced AI Action Card for iOS
struct AIActionCard: View {
    let title: String
    let description: String
    let icon: String
    let isEnabled: Bool
    let action: () -> Void
    
    var body: some View {
        Button(action: action) {
            HStack(spacing: 16) {
                Image(systemName: icon)
                    .font(.title2)
                    .foregroundColor(isEnabled ? .accentColor : .secondary)
                    .frame(width: 30)
                
                VStack(alignment: .leading, spacing: 4) {
                    Text(title)
                        .font(.headline)
                        .fontWeight(.medium)
                        .foregroundColor(isEnabled ? .primary : .secondary)
                    
                    Text(description)
                        .font(.caption)
                        .foregroundColor(isEnabled ? .secondary : .secondary.opacity(0.7))
                        .multilineTextAlignment(.leading)
                }
                
                Spacer()
                
                Image(systemName: "chevron.right")
                    .font(.caption)
                    .foregroundColor(isEnabled ? .secondary : .secondary.opacity(0.5))
            }
            .padding()
            .background(
                RoundedRectangle(cornerRadius: 12)
                    .fill(isEnabled ? Color(.systemGray6) : Color(.systemGray6).opacity(0.5))
            )
        }
        .disabled(!isEnabled)
        .buttonStyle(PlainButtonStyle())
    }
}

#Preview {
    WritingView()
}