import Foundation

struct Project: Identifiable, Codable {
    let id: String
    var name: String
    var description: String
    var documentsCount: Int
    var lastModified: String
    var createdAt: Date
    var modifiedAt: Date
    
    init(id: String = UUID().uuidString, name: String, description: String, documentsCount: Int = 0, lastModified: String = "Just now") {
        self.id = id
        self.name = name
        self.description = description
        self.documentsCount = documentsCount
        self.lastModified = lastModified
        self.createdAt = Date()
        self.modifiedAt = Date()
    }
    
    mutating func updateInfo(name: String? = nil, description: String? = nil) {
        if let newName = name {
            self.name = newName
        }
        if let newDescription = description {
            self.description = newDescription
        }
        self.modifiedAt = Date()
        self.lastModified = "Just now"
    }
    
    mutating func updateDocumentCount(_ count: Int) {
        self.documentsCount = count
        self.modifiedAt = Date()
        self.lastModified = "Just now"
    }
}