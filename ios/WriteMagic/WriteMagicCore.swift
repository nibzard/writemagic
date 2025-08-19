import Foundation

/// Swift wrapper for the WriteMagic Rust core engine
class WriteMagicCore: ObservableObject {
    private static var isInitialized = false
    
    /// Document representation
    struct Document: Codable, Identifiable {
        let id: String
        let title: String
        let content: String
        let contentType: String
        let wordCount: Int
        let characterCount: Int
        let createdAt: String
        let updatedAt: String
        let version: Int
        let isDeleted: Bool
    }
    
    /// AI response structure
    struct AIResponse: Codable {
        let completion: String?
        let error: String?
        let success: Bool
    }
    
    /// Initialize the WriteMagic core engine with persistent SQLite
    static func initialize(claudeKey: String = "", openaiKey: String = "") async -> Bool {
        if isInitialized {
            print("WriteMagic core already initialized")
            return true
        }
        
        print("Initializing WriteMagic core with persistent SQLite...")
        
        // Convert Swift strings to C strings
        let claudeKeyPtr = claudeKey.isEmpty ? nil : strdup(claudeKey)
        let openaiKeyPtr = openaiKey.isEmpty ? nil : strdup(openaiKey)
        
        defer {
            if let ptr = claudeKeyPtr { free(ptr) }
            if let ptr = openaiKeyPtr { free(ptr) }
        }
        
        let result = writemagic_initialize_with_ai(1, claudeKeyPtr, openaiKeyPtr) == 1
        isInitialized = result
        
        if result {
            print("WriteMagic core initialized successfully")
        } else {
            print("Failed to initialize WriteMagic core")
        }
        
        return result
    }
    
    /// Create a new document
    static func createDocument(title: String, content: String = "", contentType: String = "markdown") async -> Document? {
        guard isInitialized else {
            print("WriteMagic core not initialized")
            return nil
        }
        
        let titlePtr = strdup(title)
        let contentPtr = strdup(content)
        let contentTypePtr = strdup(contentType)
        
        defer {
            if let ptr = titlePtr { free(ptr) }
            if let ptr = contentPtr { free(ptr) }
            if let ptr = contentTypePtr { free(ptr) }
        }
        
        guard let resultPtr = writemagic_create_document(titlePtr, contentPtr, contentTypePtr) else {
            print("Failed to create document")
            return nil
        }
        
        defer { writemagic_free_string(resultPtr) }
        
        let resultString = String(cString: resultPtr)
        
        do {
            // Parse as document ID and fetch full document
            return await getDocument(id: resultString)
        } catch {
            print("Error parsing document creation result: \(error)")
            return nil
        }
    }
    
    /// Update document content
    static func updateDocumentContent(id: String, content: String) async -> Bool {
        guard isInitialized else {
            print("WriteMagic core not initialized")
            return false
        }
        
        let idPtr = strdup(id)
        let contentPtr = strdup(content)
        
        defer {
            if let ptr = idPtr { free(ptr) }
            if let ptr = contentPtr { free(ptr) }
        }
        
        let result = writemagic_update_document_content(idPtr, contentPtr) == 1
        
        if !result {
            print("Failed to update document \(id)")
        }
        
        return result
    }
    
    /// Get document by ID
    static func getDocument(id: String) async -> Document? {
        guard isInitialized else {
            print("WriteMagic core not initialized")
            return nil
        }
        
        let idPtr = strdup(id)
        defer { if let ptr = idPtr { free(ptr) } }
        
        guard let resultPtr = writemagic_get_document(idPtr) else {
            print("Document \(id) not found")
            return nil
        }
        
        defer { writemagic_free_string(resultPtr) }
        
        let jsonString = String(cString: resultPtr)
        
        do {
            let data = jsonString.data(using: .utf8)!
            let document = try JSONDecoder().decode(Document.self, from: data)
            return document
        } catch {
            print("Error parsing document JSON: \(error)")
            return nil
        }
    }
    
    /// Complete text using AI
    static func completeText(prompt: String, model: String? = nil) async -> AIResponse {
        guard isInitialized else {
            print("WriteMagic core not initialized")
            return AIResponse(completion: nil, error: "Core not initialized", success: false)
        }
        
        let promptPtr = strdup(prompt)
        let modelPtr = model.map { strdup($0) }
        
        defer {
            if let ptr = promptPtr { free(ptr) }
            if let ptr = modelPtr { free(ptr) }
        }
        
        guard let resultPtr = writemagic_complete_text(promptPtr, modelPtr) else {
            print("AI completion failed")
            return AIResponse(completion: nil, error: "AI completion failed", success: false)
        }
        
        defer { writemagic_free_string(resultPtr) }
        
        let jsonString = String(cString: resultPtr)
        
        do {
            let data = jsonString.data(using: .utf8)!
            let response = try JSONDecoder().decode(AIResponse.self, from: data)
            return response
        } catch {
            print("Error parsing AI response JSON: \(error)")
            return AIResponse(completion: nil, error: "Failed to parse response", success: false)
        }
    }
}

// MARK: - C FFI Function Declarations
// These match the functions exported by the iOS FFI library

@_silgen_name("writemagic_initialize_with_ai")
func writemagic_initialize_with_ai(_ use_sqlite: Int32, _ claude_key: UnsafePointer<CChar>?, _ openai_key: UnsafePointer<CChar>?) -> Int32

@_silgen_name("writemagic_create_document")
func writemagic_create_document(_ title: UnsafePointer<CChar>, _ content: UnsafePointer<CChar>, _ content_type: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?

@_silgen_name("writemagic_update_document_content")
func writemagic_update_document_content(_ document_id: UnsafePointer<CChar>, _ content: UnsafePointer<CChar>) -> Int32

@_silgen_name("writemagic_get_document")
func writemagic_get_document(_ document_id: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?

@_silgen_name("writemagic_complete_text")
func writemagic_complete_text(_ prompt: UnsafePointer<CChar>, _ model: UnsafePointer<CChar>?) -> UnsafeMutablePointer<CChar>?

@_silgen_name("writemagic_free_string")
func writemagic_free_string(_ ptr: UnsafeMutablePointer<CChar>)

@_silgen_name("writemagic_get_version")
func writemagic_get_version() -> UnsafePointer<CChar>