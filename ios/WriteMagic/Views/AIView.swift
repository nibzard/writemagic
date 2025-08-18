import SwiftUI

struct AIView: View {
    @State private var messages: [ChatMessage] = [
        ChatMessage(content: "Hello! I'm your AI writing assistant. How can I help you today?", isUser: false)
    ]
    @State private var inputText = ""
    @State private var selectedProvider = "Claude"
    @State private var isProcessing = false
    
    let providers = ["Claude", "GPT-4", "Local Model"]
    
    var body: some View {
        NavigationView {
            VStack(spacing: 0) {
                // Provider selection
                VStack {
                    HStack {
                        Image(systemName: "brain.head.profile")
                            .foregroundColor(.purple)
                        
                        Text("Provider:")
                            .font(.subheadline)
                            .foregroundColor(.secondary)
                        
                        Spacer()
                    }
                    
                    ScrollView(.horizontal, showsIndicators: false) {
                        HStack(spacing: 8) {
                            ForEach(providers, id: \.self) { provider in
                                Button(action: { selectedProvider = provider }) {
                                    Text(provider)
                                        .font(.caption)
                                        .padding(.horizontal, 12)
                                        .padding(.vertical, 6)
                                        .background(
                                            selectedProvider == provider
                                                ? Color.purple.opacity(0.2)
                                                : Color(.systemGray6)
                                        )
                                        .foregroundColor(
                                            selectedProvider == provider
                                                ? .purple
                                                : .primary
                                        )
                                        .cornerRadius(12)
                                }
                            }
                        }
                        .padding(.horizontal)
                    }
                }
                .padding()
                .background(Color(.systemGray6))
                
                // Chat messages
                ScrollViewReader { proxy in
                    ScrollView {
                        LazyVStack(alignment: .leading, spacing: 12) {
                            ForEach(messages) { message in
                                MessageBubble(message: message)
                                    .id(message.id)
                            }
                            
                            if isProcessing {
                                ProcessingIndicator()
                            }
                        }
                        .padding()
                    }
                    .onChange(of: messages.count) { _ in
                        withAnimation {
                            proxy.scrollTo(messages.last?.id, anchor: .bottom)
                        }
                    }
                }
                
                // Quick actions
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 8) {
                        QuickActionChip(title: "Continue Writing") {
                            inputText = "Continue writing from where I left off"
                        }
                        
                        QuickActionChip(title: "Improve Clarity") {
                            inputText = "Improve the clarity of this paragraph"
                        }
                        
                        QuickActionChip(title: "Rephrase") {
                            inputText = "Suggest alternative phrasings"
                        }
                    }
                    .padding(.horizontal)
                }
                .padding(.vertical, 8)
                
                // Input area
                HStack(spacing: 12) {
                    TextField("Ask me anything about your writing...", text: $inputText, axis: .vertical)
                        .textFieldStyle(RoundedBorderTextFieldStyle())
                        .lineLimit(1...4)
                    
                    Button(action: sendMessage) {
                        Image(systemName: "arrow.up.circle.fill")
                            .font(.title2)
                            .foregroundColor(inputText.isEmpty || isProcessing ? .gray : .purple)
                    }
                    .disabled(inputText.isEmpty || isProcessing)
                }
                .padding()
            }
            .navigationTitle("AI Assistant")
        }
        .navigationViewStyle(StackNavigationViewStyle())
    }
    
    private func sendMessage() {
        guard !inputText.isEmpty && !isProcessing else { return }
        
        let userMessage = ChatMessage(content: inputText, isUser: true)
        messages.append(userMessage)
        
        let prompt = inputText
        inputText = ""
        isProcessing = true
        
        // Simulate AI response
        DispatchQueue.main.asyncAfter(deadline: .now() + 2) {
            let response = ChatMessage(
                content: "I'd be happy to help you with that! Here's my suggestion based on your request...",
                isUser: false
            )
            messages.append(response)
            isProcessing = false
        }
        
        // In real implementation, this would call:
        // RustFFI.shared.processAIRequest(prompt: prompt, context: "")
    }
}

struct MessageBubble: View {
    let message: ChatMessage
    
    var body: some View {
        HStack {
            if message.isUser {
                Spacer()
            } else {
                Image(systemName: "brain.head.profile")
                    .foregroundColor(.purple)
                    .font(.title3)
            }
            
            Text(message.content)
                .padding(12)
                .background(
                    message.isUser
                        ? Color.purple
                        : Color(.systemGray5)
                )
                .foregroundColor(
                    message.isUser
                        ? .white
                        : .primary
                )
                .cornerRadius(16)
                .frame(maxWidth: 280, alignment: message.isUser ? .trailing : .leading)
            
            if !message.isUser {
                Spacer()
            } else {
                Image(systemName: "person.circle.fill")
                    .foregroundColor(.purple)
                    .font(.title3)
            }
        }
    }
}

struct ProcessingIndicator: View {
    var body: some View {
        HStack {
            Image(systemName: "brain.head.profile")
                .foregroundColor(.purple)
                .font(.title3)
            
            HStack {
                ProgressView()
                    .scaleEffect(0.7)
                
                Text("AI is thinking...")
                    .font(.subheadline)
                    .foregroundColor(.secondary)
            }
            .padding(12)
            .background(Color(.systemGray5))
            .cornerRadius(16)
            
            Spacer()
        }
    }
}

struct QuickActionChip: View {
    let title: String
    let action: () -> Void
    
    var body: some View {
        Button(action: action) {
            Text(title)
                .font(.caption)
                .padding(.horizontal, 12)
                .padding(.vertical, 6)
                .background(Color(.systemGray6))
                .foregroundColor(.primary)
                .cornerRadius(12)
        }
    }
}

struct ChatMessage: Identifiable {
    let id = UUID()
    let content: String
    let isUser: Bool
    let timestamp = Date()
}

#Preview {
    AIView()
}