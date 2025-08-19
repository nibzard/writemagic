/**
 * Simple C test program to verify SQLite FFI integration
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// FFI function declarations (normally would be in a header)
extern int writemagic_initialize(int use_sqlite);
extern char* writemagic_create_document(const char* title, const char* content, const char* content_type);
extern char* writemagic_get_document(const char* document_id);
extern int writemagic_update_document_content(const char* document_id, const char* content);
extern void writemagic_free_string(char* ptr);
extern const char* writemagic_get_version(void);

int main() {
    printf("🧪 Testing WriteMagic SQLite FFI Integration\n\n");
    
    // Get version
    const char* version = writemagic_get_version();
    printf("📦 WriteMagic Version: %s\n", version);
    
    // Initialize with SQLite
    printf("🗄️  Initializing with SQLite...\n");
    if (writemagic_initialize(1) != 1) {
        printf("❌ Failed to initialize WriteMagic with SQLite\n");
        return 1;
    }
    printf("✅ SQLite initialization successful\n");
    
    // Create a document
    printf("\n📄 Creating a test document...\n");
    char* doc_id = writemagic_create_document(
        "Test Document from C", 
        "This is a test document created from C using SQLite storage.",
        "markdown"
    );
    
    if (doc_id == NULL) {
        printf("❌ Failed to create document\n");
        return 1;
    }
    printf("✅ Document created with ID: %s\n", doc_id);
    
    // Retrieve the document
    printf("\n🔍 Retrieving the document...\n");
    char* doc_json = writemagic_get_document(doc_id);
    if (doc_json == NULL) {
        printf("❌ Failed to retrieve document\n");
        writemagic_free_string(doc_id);
        return 1;
    }
    printf("✅ Document retrieved:\n%s\n", doc_json);
    
    // Update the document
    printf("\n✏️  Updating document content...\n");
    if (writemagic_update_document_content(doc_id, "Updated content from C with SQLite persistence!") != 1) {
        printf("❌ Failed to update document\n");
        writemagic_free_string(doc_id);
        writemagic_free_string(doc_json);
        return 1;
    }
    printf("✅ Document updated successfully\n");
    
    // Retrieve the updated document
    printf("\n🔍 Retrieving updated document...\n");
    char* updated_doc_json = writemagic_get_document(doc_id);
    if (updated_doc_json == NULL) {
        printf("❌ Failed to retrieve updated document\n");
        writemagic_free_string(doc_id);
        writemagic_free_string(doc_json);
        return 1;
    }
    printf("✅ Updated document retrieved:\n%s\n", updated_doc_json);
    
    // Clean up
    writemagic_free_string(doc_id);
    writemagic_free_string(doc_json);
    writemagic_free_string(updated_doc_json);
    
    printf("\n🎉 All tests passed! SQLite FFI integration working correctly.\n");
    return 0;
}