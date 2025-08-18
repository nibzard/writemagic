import Foundation

struct Document: Identifiable, Codable {
    let id: String
    var title: String
    var content: String
    var projectId: String?
    var createdAt: Date
    var modifiedAt: Date
    var wordCount: Int {
        content.components(separatedBy: .whitespacesAndNewlines)
            .filter { !$0.isEmpty }
            .count
    }
    
    init(id: String = UUID().uuidString, title: String, content: String = "", projectId: String? = nil) {
        self.id = id
        self.title = title
        self.content = content
        self.projectId = projectId
        self.createdAt = Date()
        self.modifiedAt = Date()
    }
    
    mutating func updateContent(_ newContent: String) {
        self.content = newContent
        self.modifiedAt = Date()
    }
    
    mutating func updateTitle(_ newTitle: String) {
        self.title = newTitle
        self.modifiedAt = Date()
    }
}