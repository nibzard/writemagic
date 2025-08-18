import Foundation

struct GitCommit: Identifiable, Codable {
    let id: String
    let message: String
    let author: String
    let timestamp: String
    let branch: String
    let filesChanged: Int
    let createdAt: Date
    
    init(id: String, message: String, author: String, timestamp: String, branch: String, filesChanged: Int) {
        self.id = id
        self.message = message
        self.author = author
        self.timestamp = timestamp
        self.branch = branch
        self.filesChanged = filesChanged
        self.createdAt = Date()
    }
}

extension GitCommit {
    var shortId: String {
        String(id.prefix(7))
    }
    
    var isMainBranch: Bool {
        branch == "main" || branch == "master"
    }
}