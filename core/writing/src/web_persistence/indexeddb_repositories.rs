//! IndexedDB repository implementations for WriteMagic entities
//! 
//! This module provides concrete implementations of repository traits
//! using IndexedDB for persistent storage in web browsers.

use async_trait::async_trait;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::*;
use js_sys::{Array, Object, Reflect};

use writemagic_shared::{EntityId, Pagination, Repository, Result as SharedResult, WritemagicError, ContentType};
use crate::entities::{Document, Project};
use crate::repositories::{DocumentRepository, ProjectRepository, DocumentStatistics, ProjectStatistics};

use super::indexeddb_manager::IndexedDbManager;
use super::schema::{ObjectStore, SearchConfig};
use super::serialization::{IndexedDbDocument, IndexedDbProject, IndexedDbProjectDocument, BatchOperation, BatchOperationType};
use super::{IndexedDbError, Result, js_error_to_indexeddb_error};

/// IndexedDB implementation of DocumentRepository
pub struct IndexedDbDocumentRepository {
    manager: std::sync::Arc<tokio::sync::Mutex<IndexedDbManager>>,
    search_config: SearchConfig,
}

impl IndexedDbDocumentRepository {
    /// Create a new IndexedDB document repository
    pub fn new(manager: std::sync::Arc<tokio::sync::Mutex<IndexedDbManager>>) -> Self {
        Self {
            manager,
            search_config: SearchConfig::default(),
        }
    }
    
    /// Execute a batch of document operations
    pub async fn execute_batch(&self, batch: BatchOperation<IndexedDbDocument>) -> Result<Vec<Document>> {
        if batch.is_empty() {
            return Ok(Vec::new());
        }
        
        let manager = self.manager.lock().await;
        let transaction = manager.write_transaction(&[ObjectStore::Documents])?;
        let store = manager.object_store(&transaction, ObjectStore::Documents)?;
        
        let mut results = Vec::new();
        
        for operation in batch.operations {
            match operation {
                BatchOperationType::Insert(doc) => {
                    let js_doc = doc.to_js_value()?;
                    let request = store.add(&js_doc)
                        .map_err(|e| js_error_to_indexeddb_error(&e, "Batch insert"))?;
                    
                    JsFuture::from(request).await
                        .map_err(|e| js_error_to_indexeddb_error(&e, "Batch insert completion"))?;
                    
                    results.push(doc.try_into()?);
                }
                BatchOperationType::Update(doc) => {
                    let js_doc = doc.to_js_value()?;
                    let request = store.put(&js_doc)
                        .map_err(|e| js_error_to_indexeddb_error(&e, "Batch update"))?;
                    
                    JsFuture::from(request).await
                        .map_err(|e| js_error_to_indexeddb_error(&e, "Batch update completion"))?;
                    
                    results.push(doc.try_into()?);
                }
                BatchOperationType::Delete(id) => {
                    let request = store.delete(&JsValue::from_str(&id))
                        .map_err(|e| js_error_to_indexeddb_error(&e, "Batch delete"))?;
                    
                    JsFuture::from(request).await
                        .map_err(|e| js_error_to_indexeddb_error(&e, "Batch delete completion"))?;
                }
            }
        }
        
        manager.execute_transaction(transaction).await?;
        Ok(results)
    }
    
    /// Search documents by text content with scoring
    async fn search_documents_by_text(&self, query: &str, pagination: Pagination) -> Result<Vec<Document>> {
        let search_tokens = self.search_config.prepare_query(query);
        
        if search_tokens.is_empty() {
            return Ok(Vec::new());
        }
        
        let manager = self.manager.lock().await;
        let transaction = manager.read_transaction(&[ObjectStore::Documents])?;
        let store = manager.object_store(&transaction, ObjectStore::Documents)?;
        
        // Get all documents (in a real implementation, we'd use better indexing)
        let request = store.get_all()
            .map_err(|e| js_error_to_indexeddb_error(&e, "Getting all documents for search"))?;
        
        let result = JsFuture::from(request).await
            .map_err(|e| js_error_to_indexeddb_error(&e, "Search documents completion"))?;
        
        let array = Array::from(&result);
        let mut scored_docs = Vec::new();
        
        for i in 0..array.length() {
            let js_doc = array.get(i);
            let indexed_doc = IndexedDbDocument::from_js_value(&js_doc)?;
            
            // Calculate relevance score
            let score = self.calculate_relevance_score(&indexed_doc, &search_tokens);
            if score > 0.0 {
                let document: Document = indexed_doc.try_into()?;
                scored_docs.push((document, score));
            }
        }
        
        // Sort by relevance score (descending)
        scored_docs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Apply pagination
        let start = pagination.offset as usize;
        let end = start + pagination.limit as usize;
        let documents = scored_docs
            .into_iter()
            .skip(start)
            .take(pagination.limit as usize)
            .map(|(doc, _score)| doc)
            .collect();
        
        Ok(documents)
    }
    
    /// Calculate relevance score for search results
    fn calculate_relevance_score(&self, doc: &IndexedDbDocument, search_tokens: &[String]) -> f32 {
        let mut score = 0.0;
        
        for token in search_tokens {
            // Title matches get higher weight
            if doc.search_title.contains(token) {
                score += 3.0;
            }
            
            // Content matches
            if doc.search_content.contains(token) {
                score += 1.0;
            }
            
            // Exact token matches
            if doc.search_tokens.contains(token) {
                score += 2.0;
            }
        }
        
        // Normalize by document length (prefer shorter documents with higher density)
        if doc.character_count > 0 {
            score = score / (doc.character_count as f32).sqrt();
        }
        
        score
    }
    
    /// Get documents using an index
    async fn get_documents_by_index(&self, index_name: &str, key: &JsValue, pagination: Pagination) -> Result<Vec<Document>> {
        let manager = self.manager.lock().await;
        let transaction = manager.read_transaction(&[ObjectStore::Documents])?;
        let store = manager.object_store(&transaction, ObjectStore::Documents)?;
        
        let index = store.index(index_name)
            .map_err(|e| js_error_to_indexeddb_error(&e, &format!("Getting index {}", index_name)))?;
        
        let request = index.get_all_with_key(key)
            .map_err(|e| js_error_to_indexeddb_error(&e, &format!("Querying index {}", index_name)))?;
        
        let result = JsFuture::from(request).await
            .map_err(|e| js_error_to_indexeddb_error(&e, &format!("Index query completion for {}", index_name)))?;
        
        let array = Array::from(&result);
        let mut documents = Vec::new();
        
        let start = pagination.offset as usize;
        let end = start + pagination.limit as usize;
        
        for i in start..std::cmp::min(end, array.length() as usize) {
            let js_doc = array.get(i as u32);
            let indexed_doc = IndexedDbDocument::from_js_value(&js_doc)?;
            documents.push(indexed_doc.try_into()?);
        }
        
        Ok(documents)
    }
    
    /// Count documents matching a condition
    async fn count_documents_by_condition<F>(&self, condition: F) -> Result<u64>
    where
        F: Fn(&IndexedDbDocument) -> bool,
    {
        let manager = self.manager.lock().await;
        let transaction = manager.read_transaction(&[ObjectStore::Documents])?;
        let store = manager.object_store(&transaction, ObjectStore::Documents)?;
        
        let request = store.get_all()
            .map_err(|e| js_error_to_indexeddb_error(&e, "Getting all documents for counting"))?;
        
        let result = JsFuture::from(request).await
            .map_err(|e| js_error_to_indexeddb_error(&e, "Count documents completion"))?;
        
        let array = Array::from(&result);
        let mut count = 0;
        
        for i in 0..array.length() {
            let js_doc = array.get(i);
            let indexed_doc = IndexedDbDocument::from_js_value(&js_doc)?;
            
            if condition(&indexed_doc) {
                count += 1;
            }
        }
        
        Ok(count)
    }
}

#[async_trait]
impl Repository<Document, EntityId> for IndexedDbDocumentRepository {
    async fn find_by_id(&self, id: &EntityId) -> SharedResult<Option<Document>> {
        let manager = self.manager.lock().await;
        let transaction = manager.read_transaction(&[ObjectStore::Documents])?;
        let store = manager.object_store(&transaction, ObjectStore::Documents)?;
        
        let request = store.get(&JsValue::from_str(&id.to_string()))
            .map_err(|e| WritemagicError::database(&format!("Find by ID failed: {:?}", e)))?;
        
        let result = JsFuture::from(request).await
            .map_err(|e| WritemagicError::database(&format!("Find by ID completion failed: {:?}", e)))?;
        
        if result.is_undefined() || result.is_null() {
            return Ok(None);
        }
        
        let indexed_doc = IndexedDbDocument::from_js_value(&result)
            .map_err(|e| WritemagicError::internal(&format!("Document deserialization failed: {}", e)))?;
        
        let document = indexed_doc.try_into()
            .map_err(|e| WritemagicError::internal(&format!("Document conversion failed: {}", e)))?;
        
        Ok(Some(document))
    }
    
    async fn find_all(&self, pagination: Pagination) -> SharedResult<Vec<Document>> {
        let manager = self.manager.lock().await;
        let transaction = manager.read_transaction(&[ObjectStore::Documents])?;
        let store = manager.object_store(&transaction, ObjectStore::Documents)?;
        
        // Use the updated_at index for ordering
        let index = store.index("updated_at")
            .map_err(|e| WritemagicError::database(&format!("Getting updated_at index failed: {:?}", e)))?;
        
        let request = index.open_cursor_with_range_and_direction(
            None,
            Some(IdbCursorDirection::Prev), // Descending order
        )
        .map_err(|e| WritemagicError::database(&format!("Opening cursor failed: {:?}", e)))?;
        
        let mut documents = Vec::new();
        let mut current_offset = 0;
        let mut collected = 0;
        
        // This is a simplified cursor handling - in practice, you'd need more complex async cursor iteration
        let result = JsFuture::from(request).await
            .map_err(|e| WritemagicError::database(&format!("Cursor operation failed: {:?}", e)))?;
        
        // For now, use get_all and handle pagination manually
        let request = store.get_all()
            .map_err(|e| WritemagicError::database(&format!("Get all documents failed: {:?}", e)))?;
        
        let result = JsFuture::from(request).await
            .map_err(|e| WritemagicError::database(&format!("Get all completion failed: {:?}", e)))?;
        
        let array = Array::from(&result);
        let mut all_docs = Vec::new();
        
        for i in 0..array.length() {
            let js_doc = array.get(i);
            let indexed_doc = IndexedDbDocument::from_js_value(&js_doc)
                .map_err(|e| WritemagicError::internal(&format!("Document deserialization failed: {}", e)))?;
            
            if !indexed_doc.is_deleted {
                let document: Document = indexed_doc.try_into()
                    .map_err(|e| WritemagicError::internal(&format!("Document conversion failed: {}", e)))?;
                all_docs.push(document);
            }
        }
        
        // Sort by updated_at descending
        all_docs.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        
        // Apply pagination
        let start = pagination.offset as usize;
        let end = start + pagination.limit as usize;
        let paginated_docs = all_docs
            .into_iter()
            .skip(start)
            .take(pagination.limit as usize)
            .collect();
        
        Ok(paginated_docs)
    }
    
    async fn save(&self, entity: &Document) -> SharedResult<Document> {
        let manager = self.manager.lock().await;
        let transaction = manager.write_transaction(&[ObjectStore::Documents])?;
        let store = manager.object_store(&transaction, ObjectStore::Documents)?;
        
        let indexed_doc = IndexedDbDocument::from(entity);
        let js_doc = indexed_doc.to_js_value()
            .map_err(|e| WritemagicError::internal(&format!("Document serialization failed: {}", e)))?;
        
        let request = store.put(&js_doc)
            .map_err(|e| WritemagicError::database(&format!("Save document failed: {:?}", e)))?;
        
        JsFuture::from(request).await
            .map_err(|e| WritemagicError::database(&format!("Save completion failed: {:?}", e)))?;
        
        manager.execute_transaction(transaction).await
            .map_err(|e| WritemagicError::database(&format!("Transaction commit failed: {:?}", e)))?;
        
        Ok(entity.clone())
    }
    
    async fn delete(&self, id: &EntityId) -> SharedResult<bool> {
        let manager = self.manager.lock().await;
        let transaction = manager.write_transaction(&[ObjectStore::Documents])?;
        let store = manager.object_store(&transaction, ObjectStore::Documents)?;
        
        // Check if document exists first
        let get_request = store.get(&JsValue::from_str(&id.to_string()))
            .map_err(|e| WritemagicError::database(&format!("Check document existence failed: {:?}", e)))?;
        
        let get_result = JsFuture::from(get_request).await
            .map_err(|e| WritemagicError::database(&format!("Get completion failed: {:?}", e)))?;
        
        if get_result.is_undefined() || get_result.is_null() {
            return Ok(false);
        }
        
        // Delete the document
        let delete_request = store.delete(&JsValue::from_str(&id.to_string()))
            .map_err(|e| WritemagicError::database(&format!("Delete document failed: {:?}", e)))?;
        
        JsFuture::from(delete_request).await
            .map_err(|e| WritemagicError::database(&format!("Delete completion failed: {:?}", e)))?;
        
        manager.execute_transaction(transaction).await
            .map_err(|e| WritemagicError::database(&format!("Transaction commit failed: {:?}", e)))?;
        
        Ok(true)
    }
    
    async fn exists(&self, id: &EntityId) -> SharedResult<bool> {
        let result = self.find_by_id(id).await?;
        Ok(result.is_some())
    }
    
    async fn count(&self) -> SharedResult<u64> {
        self.count_documents_by_condition(|doc| !doc.is_deleted).await
            .map_err(|e| WritemagicError::database(&format!("Count documents failed: {:?}", e)))
    }
}

#[async_trait]
impl DocumentRepository for IndexedDbDocumentRepository {
    async fn find_by_project_id(&self, project_id: &EntityId, pagination: Pagination) -> SharedResult<Vec<Document>> {
        // First get project-document relationships
        let manager = self.manager.lock().await;
        let transaction = manager.read_transaction(&[ObjectStore::ProjectDocuments, ObjectStore::Documents])?;
        let project_docs_store = manager.object_store(&transaction, ObjectStore::ProjectDocuments)?;
        
        let index = project_docs_store.index("project_id")
            .map_err(|e| WritemagicError::database(&format!("Getting project_id index failed: {:?}", e)))?;
        
        let request = index.get_all_with_key(&JsValue::from_str(&project_id.to_string()))
            .map_err(|e| WritemagicError::database(&format!("Query project documents failed: {:?}", e)))?;
        
        let result = JsFuture::from(request).await
            .map_err(|e| WritemagicError::database(&format!("Project documents query completion failed: {:?}", e)))?;
        
        let relationships_array = Array::from(&result);
        let mut document_ids = Vec::new();
        
        for i in 0..relationships_array.length() {
            let js_rel = relationships_array.get(i);
            let relationship = IndexedDbProjectDocument::from_js_value(&js_rel)
                .map_err(|e| WritemagicError::internal(&format!("Relationship deserialization failed: {}", e)))?;
            document_ids.push(relationship.document_id);
        }
        
        // Get documents by IDs
        let documents_store = manager.object_store(&transaction, ObjectStore::Documents)?;
        let mut documents = Vec::new();
        
        let start = pagination.offset as usize;
        let end = start + pagination.limit as usize;
        let paginated_ids = document_ids
            .into_iter()
            .skip(start)
            .take(pagination.limit as usize);
        
        for doc_id in paginated_ids {
            let request = documents_store.get(&JsValue::from_str(&doc_id))
                .map_err(|e| WritemagicError::database(&format!("Get document failed: {:?}", e)))?;
            
            let result = JsFuture::from(request).await
                .map_err(|e| WritemagicError::database(&format!("Get document completion failed: {:?}", e)))?;
            
            if !result.is_undefined() && !result.is_null() {
                let indexed_doc = IndexedDbDocument::from_js_value(&result)
                    .map_err(|e| WritemagicError::internal(&format!("Document deserialization failed: {}", e)))?;
                
                if !indexed_doc.is_deleted {
                    let document: Document = indexed_doc.try_into()
                        .map_err(|e| WritemagicError::internal(&format!("Document conversion failed: {}", e)))?;
                    documents.push(document);
                }
            }
        }
        
        // Sort by updated_at descending
        documents.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        
        Ok(documents)
    }
    
    async fn find_by_content_type(&self, content_type: &ContentType, pagination: Pagination) -> SharedResult<Vec<Document>> {
        self.get_documents_by_index("content_type", &JsValue::from_str(&content_type.to_string()), pagination).await
            .map_err(|e| WritemagicError::database(&format!("Find by content type failed: {:?}", e)))
    }
    
    async fn search_by_title(&self, query: &str, pagination: Pagination) -> SharedResult<Vec<Document>> {
        let manager = self.manager.lock().await;
        let transaction = manager.read_transaction(&[ObjectStore::Documents])?;
        let store = manager.object_store(&transaction, ObjectStore::Documents)?;
        
        let request = store.get_all()
            .map_err(|e| WritemagicError::database(&format!("Get all for title search failed: {:?}", e)))?;
        
        let result = JsFuture::from(request).await
            .map_err(|e| WritemagicError::database(&format!("Title search completion failed: {:?}", e)))?;
        
        let array = Array::from(&result);
        let mut matching_docs = Vec::new();
        let query_lower = query.to_lowercase();
        
        for i in 0..array.length() {
            let js_doc = array.get(i);
            let indexed_doc = IndexedDbDocument::from_js_value(&js_doc)
                .map_err(|e| WritemagicError::internal(&format!("Document deserialization failed: {}", e)))?;
            
            if !indexed_doc.is_deleted && indexed_doc.search_title.contains(&query_lower) {
                let document: Document = indexed_doc.try_into()
                    .map_err(|e| WritemagicError::internal(&format!("Document conversion failed: {}", e)))?;
                matching_docs.push(document);
            }
        }
        
        // Sort by relevance (title matches first, then by updated_at)
        matching_docs.sort_by(|a, b| {
            let a_exact = a.title.to_lowercase().starts_with(&query_lower);
            let b_exact = b.title.to_lowercase().starts_with(&query_lower);
            
            match (a_exact, b_exact) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => b.updated_at.cmp(&a.updated_at),
            }
        });
        
        // Apply pagination
        let start = pagination.offset as usize;
        let paginated_docs = matching_docs
            .into_iter()
            .skip(start)
            .take(pagination.limit as usize)
            .collect();
        
        Ok(paginated_docs)
    }
    
    async fn search_by_content(&self, query: &str, pagination: Pagination) -> SharedResult<Vec<Document>> {
        self.search_documents_by_text(query, pagination).await
            .map_err(|e| WritemagicError::database(&format!("Content search failed: {:?}", e)))
    }
    
    async fn find_by_creator(&self, user_id: &EntityId, pagination: Pagination) -> SharedResult<Vec<Document>> {
        self.get_documents_by_index("created_by", &JsValue::from_str(&user_id.to_string()), pagination).await
            .map_err(|e| WritemagicError::database(&format!("Find by creator failed: {:?}", e)))
    }
    
    async fn find_recently_updated(&self, pagination: Pagination) -> SharedResult<Vec<Document>> {
        // This is essentially the same as find_all since we sort by updated_at
        self.find_all(pagination).await
    }
    
    async fn find_deleted(&self, pagination: Pagination) -> SharedResult<Vec<Document>> {
        let manager = self.manager.lock().await;
        let transaction = manager.read_transaction(&[ObjectStore::Documents])?;
        let store = manager.object_store(&transaction, ObjectStore::Documents)?;
        
        let request = store.get_all()
            .map_err(|e| WritemagicError::database(&format!("Get all for deleted search failed: {:?}", e)))?;
        
        let result = JsFuture::from(request).await
            .map_err(|e| WritemagicError::database(&format!("Deleted search completion failed: {:?}", e)))?;
        
        let array = Array::from(&result);
        let mut deleted_docs = Vec::new();
        
        for i in 0..array.length() {
            let js_doc = array.get(i);
            let indexed_doc = IndexedDbDocument::from_js_value(&js_doc)
                .map_err(|e| WritemagicError::internal(&format!("Document deserialization failed: {}", e)))?;
            
            if indexed_doc.is_deleted {
                let document: Document = indexed_doc.try_into()
                    .map_err(|e| WritemagicError::internal(&format!("Document conversion failed: {}", e)))?;
                deleted_docs.push(document);
            }
        }
        
        // Sort by deleted_at descending
        deleted_docs.sort_by(|a, b| {
            match (&a.deleted_at, &b.deleted_at) {
                (Some(a_deleted), Some(b_deleted)) => b_deleted.cmp(a_deleted),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => b.updated_at.cmp(&a.updated_at),
            }
        });
        
        // Apply pagination
        let start = pagination.offset as usize;
        let paginated_docs = deleted_docs
            .into_iter()
            .skip(start)
            .take(pagination.limit as usize)
            .collect();
        
        Ok(paginated_docs)
    }
    
    async fn get_statistics(&self) -> SharedResult<DocumentStatistics> {
        let manager = self.manager.lock().await;
        let transaction = manager.read_transaction(&[ObjectStore::Documents])?;
        let store = manager.object_store(&transaction, ObjectStore::Documents)?;
        
        let request = store.get_all()
            .map_err(|e| WritemagicError::database(&format!("Get all for statistics failed: {:?}", e)))?;
        
        let result = JsFuture::from(request).await
            .map_err(|e| WritemagicError::database(&format!("Statistics query completion failed: {:?}", e)))?;
        
        let array = Array::from(&result);
        let mut total_documents = 0u64;
        let mut total_word_count = 0u64;
        let mut total_character_count = 0u64;
        let mut deleted_documents = 0u64;
        let mut documents_by_type = HashMap::new();
        
        for i in 0..array.length() {
            let js_doc = array.get(i);
            let indexed_doc = IndexedDbDocument::from_js_value(&js_doc)
                .map_err(|e| WritemagicError::internal(&format!("Document deserialization failed: {}", e)))?;
            
            total_documents += 1;
            total_word_count += indexed_doc.word_count as u64;
            total_character_count += indexed_doc.character_count as u64;
            
            if indexed_doc.is_deleted {
                deleted_documents += 1;
            }
            
            *documents_by_type.entry(indexed_doc.content_type).or_insert(0) += 1;
        }
        
        let average_word_count = if total_documents > 0 {
            total_word_count as f64 / total_documents as f64
        } else {
            0.0
        };
        
        let average_character_count = if total_documents > 0 {
            total_character_count as f64 / total_documents as f64
        } else {
            0.0
        };
        
        Ok(DocumentStatistics {
            total_documents,
            total_word_count,
            total_character_count,
            documents_by_type,
            average_word_count,
            average_character_count,
            deleted_documents,
        })
    }
}

/// IndexedDB implementation of ProjectRepository
pub struct IndexedDbProjectRepository {
    manager: std::sync::Arc<tokio::sync::Mutex<IndexedDbManager>>,
}

impl IndexedDbProjectRepository {
    /// Create a new IndexedDB project repository
    pub fn new(manager: std::sync::Arc<tokio::sync::Mutex<IndexedDbManager>>) -> Self {
        Self { manager }
    }
    
    /// Add a document to a project (manage relationship)
    pub async fn add_document_to_project(&self, project_id: &EntityId, document_id: &EntityId) -> SharedResult<()> {
        let manager = self.manager.lock().await;
        let transaction = manager.write_transaction(&[ObjectStore::ProjectDocuments])?;
        let store = manager.object_store(&transaction, ObjectStore::ProjectDocuments)?;
        
        let relationship = IndexedDbProjectDocument::new(project_id, document_id);
        let js_rel = relationship.to_js_value()
            .map_err(|e| WritemagicError::internal(&format!("Relationship serialization failed: {}", e)))?;
        
        let request = store.add(&js_rel)
            .map_err(|e| WritemagicError::database(&format!("Add project document relationship failed: {:?}", e)))?;
        
        JsFuture::from(request).await
            .map_err(|e| WritemagicError::database(&format!("Add relationship completion failed: {:?}", e)))?;
        
        manager.execute_transaction(transaction).await
            .map_err(|e| WritemagicError::database(&format!("Transaction commit failed: {:?}", e)))?;
        
        Ok(())
    }
    
    /// Remove a document from a project
    pub async fn remove_document_from_project(&self, project_id: &EntityId, document_id: &EntityId) -> SharedResult<()> {
        let composite_key = format!("{}|{}", project_id.to_string(), document_id.to_string());
        
        let manager = self.manager.lock().await;
        let transaction = manager.write_transaction(&[ObjectStore::ProjectDocuments])?;
        let store = manager.object_store(&transaction, ObjectStore::ProjectDocuments)?;
        
        let request = store.delete(&JsValue::from_str(&composite_key))
            .map_err(|e| WritemagicError::database(&format!("Remove project document relationship failed: {:?}", e)))?;
        
        JsFuture::from(request).await
            .map_err(|e| WritemagicError::database(&format!("Remove relationship completion failed: {:?}", e)))?;
        
        manager.execute_transaction(transaction).await
            .map_err(|e| WritemagicError::database(&format!("Transaction commit failed: {:?}", e)))?;
        
        Ok(())
    }
    
    /// Load document IDs for a project
    async fn load_project_document_ids(&self, project_id: &EntityId) -> Result<Vec<EntityId>> {
        let manager = self.manager.lock().await;
        let transaction = manager.read_transaction(&[ObjectStore::ProjectDocuments])?;
        let store = manager.object_store(&transaction, ObjectStore::ProjectDocuments)?;
        
        let index = store.index("project_id")
            .map_err(|e| js_error_to_indexeddb_error(&e, "Getting project_id index"))?;
        
        let request = index.get_all_with_key(&JsValue::from_str(&project_id.to_string()))
            .map_err(|e| js_error_to_indexeddb_error(&e, "Query project documents"))?;
        
        let result = JsFuture::from(request).await
            .map_err(|e| js_error_to_indexeddb_error(&e, "Project documents query completion"))?;
        
        let array = Array::from(&result);
        let mut document_ids = Vec::new();
        
        for i in 0..array.length() {
            let js_rel = array.get(i);
            let relationship = IndexedDbProjectDocument::from_js_value(&js_rel)?;
            
            if let Ok(doc_id) = EntityId::from_string(&relationship.document_id) {
                document_ids.push(doc_id);
            }
        }
        
        Ok(document_ids)
    }
    
    /// Update project with document IDs loaded
    async fn enhance_project_with_documents(&self, mut project: Project) -> Result<Project> {
        project.document_ids = self.load_project_document_ids(&project.id).await?;
        Ok(project)
    }
}

#[async_trait]
impl Repository<Project, EntityId> for IndexedDbProjectRepository {
    async fn find_by_id(&self, id: &EntityId) -> SharedResult<Option<Project>> {
        let manager = self.manager.lock().await;
        let transaction = manager.read_transaction(&[ObjectStore::Projects, ObjectStore::ProjectDocuments])?;
        let store = manager.object_store(&transaction, ObjectStore::Projects)?;
        
        let request = store.get(&JsValue::from_str(&id.to_string()))
            .map_err(|e| WritemagicError::database(&format!("Find project by ID failed: {:?}", e)))?;
        
        let result = JsFuture::from(request).await
            .map_err(|e| WritemagicError::database(&format!("Find project completion failed: {:?}", e)))?;
        
        if result.is_undefined() || result.is_null() {
            return Ok(None);
        }
        
        let indexed_proj = IndexedDbProject::from_js_value(&result)
            .map_err(|e| WritemagicError::internal(&format!("Project deserialization failed: {}", e)))?;
        
        let mut project: Project = indexed_proj.try_into()
            .map_err(|e| WritemagicError::internal(&format!("Project conversion failed: {}", e)))?;
        
        // Load document IDs
        project = self.enhance_project_with_documents(project).await
            .map_err(|e| WritemagicError::database(&format!("Loading project documents failed: {:?}", e)))?;
        
        Ok(Some(project))
    }
    
    async fn find_all(&self, pagination: Pagination) -> SharedResult<Vec<Project>> {
        let manager = self.manager.lock().await;
        let transaction = manager.read_transaction(&[ObjectStore::Projects, ObjectStore::ProjectDocuments])?;
        let store = manager.object_store(&transaction, ObjectStore::Projects)?;
        
        let request = store.get_all()
            .map_err(|e| WritemagicError::database(&format!("Get all projects failed: {:?}", e)))?;
        
        let result = JsFuture::from(request).await
            .map_err(|e| WritemagicError::database(&format!("Get all projects completion failed: {:?}", e)))?;
        
        let array = Array::from(&result);
        let mut projects = Vec::new();
        
        for i in 0..array.length() {
            let js_proj = array.get(i);
            let indexed_proj = IndexedDbProject::from_js_value(&js_proj)
                .map_err(|e| WritemagicError::internal(&format!("Project deserialization failed: {}", e)))?;
            
            if !indexed_proj.is_deleted {
                let mut project: Project = indexed_proj.try_into()
                    .map_err(|e| WritemagicError::internal(&format!("Project conversion failed: {}", e)))?;
                
                // Load document IDs
                project = self.enhance_project_with_documents(project).await
                    .map_err(|e| WritemagicError::database(&format!("Loading project documents failed: {:?}", e)))?;
                
                projects.push(project);
            }
        }
        
        // Sort by updated_at descending
        projects.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        
        // Apply pagination
        let start = pagination.offset as usize;
        let paginated_projects = projects
            .into_iter()
            .skip(start)
            .take(pagination.limit as usize)
            .collect();
        
        Ok(paginated_projects)
    }
    
    async fn save(&self, entity: &Project) -> SharedResult<Project> {
        let manager = self.manager.lock().await;
        let transaction = manager.write_transaction(&[ObjectStore::Projects, ObjectStore::ProjectDocuments])?;
        let projects_store = manager.object_store(&transaction, ObjectStore::Projects)?;
        let project_docs_store = manager.object_store(&transaction, ObjectStore::ProjectDocuments)?;
        
        // Save project
        let indexed_proj = IndexedDbProject::from(entity);
        let js_proj = indexed_proj.to_js_value()
            .map_err(|e| WritemagicError::internal(&format!("Project serialization failed: {}", e)))?;
        
        let save_request = projects_store.put(&js_proj)
            .map_err(|e| WritemagicError::database(&format!("Save project failed: {:?}", e)))?;
        
        JsFuture::from(save_request).await
            .map_err(|e| WritemagicError::database(&format!("Save project completion failed: {:?}", e)))?;
        
        // Clear existing project-document relationships
        let index = project_docs_store.index("project_id")
            .map_err(|e| WritemagicError::database(&format!("Getting project_id index failed: {:?}", e)))?;
        
        let get_relationships_request = index.get_all_with_key(&JsValue::from_str(&entity.id.to_string()))
            .map_err(|e| WritemagicError::database(&format!("Get project relationships failed: {:?}", e)))?;
        
        let relationships_result = JsFuture::from(get_relationships_request).await
            .map_err(|e| WritemagicError::database(&format!("Get relationships completion failed: {:?}", e)))?;
        
        let relationships_array = Array::from(&relationships_result);
        for i in 0..relationships_array.length() {
            let js_rel = relationships_array.get(i);
            let relationship = IndexedDbProjectDocument::from_js_value(&js_rel)
                .map_err(|e| WritemagicError::internal(&format!("Relationship deserialization failed: {}", e)))?;
            
            let delete_request = project_docs_store.delete(&JsValue::from_str(&relationship.composite_key))
                .map_err(|e| WritemagicError::database(&format!("Delete old relationship failed: {:?}", e)))?;
            
            JsFuture::from(delete_request).await
                .map_err(|e| WritemagicError::database(&format!("Delete relationship completion failed: {:?}", e)))?;
        }
        
        // Add new project-document relationships
        for doc_id in &entity.document_ids {
            let relationship = IndexedDbProjectDocument::new(&entity.id, doc_id);
            let js_rel = relationship.to_js_value()
                .map_err(|e| WritemagicError::internal(&format!("Relationship serialization failed: {}", e)))?;
            
            let add_request = project_docs_store.add(&js_rel)
                .map_err(|e| WritemagicError::database(&format!("Add relationship failed: {:?}", e)))?;
            
            JsFuture::from(add_request).await
                .map_err(|e| WritemagicError::database(&format!("Add relationship completion failed: {:?}", e)))?;
        }
        
        manager.execute_transaction(transaction).await
            .map_err(|e| WritemagicError::database(&format!("Transaction commit failed: {:?}", e)))?;
        
        Ok(entity.clone())
    }
    
    async fn delete(&self, id: &EntityId) -> SharedResult<bool> {
        let manager = self.manager.lock().await;
        let transaction = manager.write_transaction(&[ObjectStore::Projects, ObjectStore::ProjectDocuments])?;
        let projects_store = manager.object_store(&transaction, ObjectStore::Projects)?;
        let project_docs_store = manager.object_store(&transaction, ObjectStore::ProjectDocuments)?;
        
        // Check if project exists
        let get_request = projects_store.get(&JsValue::from_str(&id.to_string()))
            .map_err(|e| WritemagicError::database(&format!("Check project existence failed: {:?}", e)))?;
        
        let get_result = JsFuture::from(get_request).await
            .map_err(|e| WritemagicError::database(&format!("Get project completion failed: {:?}", e)))?;
        
        if get_result.is_undefined() || get_result.is_null() {
            return Ok(false);
        }
        
        // Delete project-document relationships first
        let index = project_docs_store.index("project_id")
            .map_err(|e| WritemagicError::database(&format!("Getting project_id index failed: {:?}", e)))?;
        
        let get_relationships_request = index.get_all_with_key(&JsValue::from_str(&id.to_string()))
            .map_err(|e| WritemagicError::database(&format!("Get project relationships failed: {:?}", e)))?;
        
        let relationships_result = JsFuture::from(get_relationships_request).await
            .map_err(|e| WritemagicError::database(&format!("Get relationships completion failed: {:?}", e)))?;
        
        let relationships_array = Array::from(&relationships_result);
        for i in 0..relationships_array.length() {
            let js_rel = relationships_array.get(i);
            let relationship = IndexedDbProjectDocument::from_js_value(&js_rel)
                .map_err(|e| WritemagicError::internal(&format!("Relationship deserialization failed: {}", e)))?;
            
            let delete_request = project_docs_store.delete(&JsValue::from_str(&relationship.composite_key))
                .map_err(|e| WritemagicError::database(&format!("Delete relationship failed: {:?}", e)))?;
            
            JsFuture::from(delete_request).await
                .map_err(|e| WritemagicError::database(&format!("Delete relationship completion failed: {:?}", e)))?;
        }
        
        // Delete project
        let delete_request = projects_store.delete(&JsValue::from_str(&id.to_string()))
            .map_err(|e| WritemagicError::database(&format!("Delete project failed: {:?}", e)))?;
        
        JsFuture::from(delete_request).await
            .map_err(|e| WritemagicError::database(&format!("Delete project completion failed: {:?}", e)))?;
        
        manager.execute_transaction(transaction).await
            .map_err(|e| WritemagicError::database(&format!("Transaction commit failed: {:?}", e)))?;
        
        Ok(true)
    }
    
    async fn exists(&self, id: &EntityId) -> SharedResult<bool> {
        let result = self.find_by_id(id).await?;
        Ok(result.is_some())
    }
    
    async fn count(&self) -> SharedResult<u64> {
        let manager = self.manager.lock().await;
        let transaction = manager.read_transaction(&[ObjectStore::Projects])?;
        let store = manager.object_store(&transaction, ObjectStore::Projects)?;
        
        let request = store.get_all()
            .map_err(|e| WritemagicError::database(&format!("Get all for count failed: {:?}", e)))?;
        
        let result = JsFuture::from(request).await
            .map_err(|e| WritemagicError::database(&format!("Count completion failed: {:?}", e)))?;
        
        let array = Array::from(&result);
        let mut count = 0;
        
        for i in 0..array.length() {
            let js_proj = array.get(i);
            let indexed_proj = IndexedDbProject::from_js_value(&js_proj)
                .map_err(|e| WritemagicError::internal(&format!("Project deserialization failed: {}", e)))?;
            
            if !indexed_proj.is_deleted {
                count += 1;
            }
        }
        
        Ok(count)
    }
}

#[async_trait]
impl ProjectRepository for IndexedDbProjectRepository {
    async fn find_by_creator(&self, user_id: &EntityId, pagination: Pagination) -> SharedResult<Vec<Project>> {
        let manager = self.manager.lock().await;
        let transaction = manager.read_transaction(&[ObjectStore::Projects, ObjectStore::ProjectDocuments])?;
        let store = manager.object_store(&transaction, ObjectStore::Projects)?;
        
        let index = store.index("created_by")
            .map_err(|e| WritemagicError::database(&format!("Getting created_by index failed: {:?}", e)))?;
        
        let request = index.get_all_with_key(&JsValue::from_str(&user_id.to_string()))
            .map_err(|e| WritemagicError::database(&format!("Query by creator failed: {:?}", e)))?;
        
        let result = JsFuture::from(request).await
            .map_err(|e| WritemagicError::database(&format!("Query by creator completion failed: {:?}", e)))?;
        
        let array = Array::from(&result);
        let mut projects = Vec::new();
        
        for i in 0..array.length() {
            let js_proj = array.get(i);
            let indexed_proj = IndexedDbProject::from_js_value(&js_proj)
                .map_err(|e| WritemagicError::internal(&format!("Project deserialization failed: {}", e)))?;
            
            if !indexed_proj.is_deleted {
                let mut project: Project = indexed_proj.try_into()
                    .map_err(|e| WritemagicError::internal(&format!("Project conversion failed: {}", e)))?;
                
                // Load document IDs
                project = self.enhance_project_with_documents(project).await
                    .map_err(|e| WritemagicError::database(&format!("Loading project documents failed: {:?}", e)))?;
                
                projects.push(project);
            }
        }
        
        // Sort by updated_at descending
        projects.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        
        // Apply pagination
        let start = pagination.offset as usize;
        let paginated_projects = projects
            .into_iter()
            .skip(start)
            .take(pagination.limit as usize)
            .collect();
        
        Ok(paginated_projects)
    }
    
    async fn search_by_name(&self, query: &str, pagination: Pagination) -> SharedResult<Vec<Project>> {
        let manager = self.manager.lock().await;
        let transaction = manager.read_transaction(&[ObjectStore::Projects, ObjectStore::ProjectDocuments])?;
        let store = manager.object_store(&transaction, ObjectStore::Projects)?;
        
        let request = store.get_all()
            .map_err(|e| WritemagicError::database(&format!("Get all for name search failed: {:?}", e)))?;
        
        let result = JsFuture::from(request).await
            .map_err(|e| WritemagicError::database(&format!("Name search completion failed: {:?}", e)))?;
        
        let array = Array::from(&result);
        let mut matching_projects = Vec::new();
        let query_lower = query.to_lowercase();
        
        for i in 0..array.length() {
            let js_proj = array.get(i);
            let indexed_proj = IndexedDbProject::from_js_value(&js_proj)
                .map_err(|e| WritemagicError::internal(&format!("Project deserialization failed: {}", e)))?;
            
            if !indexed_proj.is_deleted && indexed_proj.search_name.contains(&query_lower) {
                let mut project: Project = indexed_proj.try_into()
                    .map_err(|e| WritemagicError::internal(&format!("Project conversion failed: {}", e)))?;
                
                // Load document IDs
                project = self.enhance_project_with_documents(project).await
                    .map_err(|e| WritemagicError::database(&format!("Loading project documents failed: {:?}", e)))?;
                
                matching_projects.push(project);
            }
        }
        
        // Sort by relevance
        matching_projects.sort_by(|a, b| {
            let a_exact = a.name.to_lowercase().starts_with(&query_lower);
            let b_exact = b.name.to_lowercase().starts_with(&query_lower);
            
            match (a_exact, b_exact) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => b.updated_at.cmp(&a.updated_at),
            }
        });
        
        // Apply pagination
        let start = pagination.offset as usize;
        let paginated_projects = matching_projects
            .into_iter()
            .skip(start)
            .take(pagination.limit as usize)
            .collect();
        
        Ok(paginated_projects)
    }
    
    async fn find_containing_document(&self, document_id: &EntityId, pagination: Pagination) -> SharedResult<Vec<Project>> {
        let manager = self.manager.lock().await;
        let transaction = manager.read_transaction(&[ObjectStore::Projects, ObjectStore::ProjectDocuments])?;
        let project_docs_store = manager.object_store(&transaction, ObjectStore::ProjectDocuments)?;
        
        let index = project_docs_store.index("document_id")
            .map_err(|e| WritemagicError::database(&format!("Getting document_id index failed: {:?}", e)))?;
        
        let request = index.get_all_with_key(&JsValue::from_str(&document_id.to_string()))
            .map_err(|e| WritemagicError::database(&format!("Query containing document failed: {:?}", e)))?;
        
        let result = JsFuture::from(request).await
            .map_err(|e| WritemagicError::database(&format!("Containing document query completion failed: {:?}", e)))?;
        
        let relationships_array = Array::from(&result);
        let mut project_ids = Vec::new();
        
        for i in 0..relationships_array.length() {
            let js_rel = relationships_array.get(i);
            let relationship = IndexedDbProjectDocument::from_js_value(&js_rel)
                .map_err(|e| WritemagicError::internal(&format!("Relationship deserialization failed: {}", e)))?;
            
            if let Ok(project_id) = EntityId::from_string(&relationship.project_id) {
                project_ids.push(project_id);
            }
        }
        
        // Get projects by IDs
        let projects_store = manager.object_store(&transaction, ObjectStore::Projects)?;
        let mut projects = Vec::new();
        
        for project_id in project_ids {
            if let Some(project) = self.find_by_id(&project_id).await? {
                projects.push(project);
            }
        }
        
        // Sort by updated_at descending
        projects.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        
        // Apply pagination
        let start = pagination.offset as usize;
        let paginated_projects = projects
            .into_iter()
            .skip(start)
            .take(pagination.limit as usize)
            .collect();
        
        Ok(paginated_projects)
    }
    
    async fn find_recently_updated(&self, pagination: Pagination) -> SharedResult<Vec<Project>> {
        self.find_all(pagination).await
    }
    
    async fn get_statistics(&self) -> SharedResult<ProjectStatistics> {
        let manager = self.manager.lock().await;
        let transaction = manager.read_transaction(&[ObjectStore::Projects, ObjectStore::ProjectDocuments])?;
        let projects_store = manager.object_store(&transaction, ObjectStore::Projects)?;
        let project_docs_store = manager.object_store(&transaction, ObjectStore::ProjectDocuments)?;
        
        // Get all projects
        let projects_request = projects_store.get_all()
            .map_err(|e| WritemagicError::database(&format!("Get all projects for statistics failed: {:?}", e)))?;
        
        let projects_result = JsFuture::from(projects_request).await
            .map_err(|e| WritemagicError::database(&format!("Projects statistics completion failed: {:?}", e)))?;
        
        // Get all project-document relationships
        let relationships_request = project_docs_store.get_all()
            .map_err(|e| WritemagicError::database(&format!("Get all relationships for statistics failed: {:?}", e)))?;
        
        let relationships_result = JsFuture::from(relationships_request).await
            .map_err(|e| WritemagicError::database(&format!("Relationships statistics completion failed: {:?}", e)))?;
        
        let projects_array = Array::from(&projects_result);
        let relationships_array = Array::from(&relationships_result);
        
        let mut total_projects = 0u64;
        let mut project_doc_counts = Vec::new();
        
        for i in 0..projects_array.length() {
            let js_proj = projects_array.get(i);
            let indexed_proj = IndexedDbProject::from_js_value(&js_proj)
                .map_err(|e| WritemagicError::internal(&format!("Project deserialization failed: {}", e)))?;
            
            if !indexed_proj.is_deleted {
                total_projects += 1;
            }
        }
        
        // Count documents per project
        let mut project_document_counts: HashMap<String, u64> = HashMap::new();
        for i in 0..relationships_array.length() {
            let js_rel = relationships_array.get(i);
            let relationship = IndexedDbProjectDocument::from_js_value(&js_rel)
                .map_err(|e| WritemagicError::internal(&format!("Relationship deserialization failed: {}", e)))?;
            
            *project_document_counts.entry(relationship.project_id).or_insert(0) += 1;
        }
        
        let total_documents_in_projects = relationships_array.length() as u64;
        
        let average_documents_per_project = if total_projects > 0 {
            total_documents_in_projects as f64 / total_projects as f64
        } else {
            0.0
        };
        
        let largest_project_size = project_document_counts.values().max().copied().unwrap_or(0);
        let smallest_project_size = if project_document_counts.is_empty() {
            0
        } else {
            project_document_counts.values().min().copied().unwrap_or(0)
        };
        
        Ok(ProjectStatistics {
            total_projects,
            total_documents_in_projects,
            average_documents_per_project,
            largest_project_size,
            smallest_project_size,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use writemagic_shared::Timestamp;
    use wasm_bindgen_test::*;
    
    // Note: These tests would need to run in a browser environment with IndexedDB support
    // In practice, you'd use tools like wasm-pack test for browser testing
    
    #[test]
    fn test_search_relevance_scoring() {
        let repo = IndexedDbDocumentRepository::new(
            std::sync::Arc::new(tokio::sync::Mutex::new(
                IndexedDbManager::with_defaults()
            ))
        );
        
        let doc = IndexedDbDocument {
            id: "test-id".to_string(),
            title: "Test Document".to_string(),
            content: "This is a test document with some content.".to_string(),
            content_type: "markdown".to_string(),
            content_hash: "hash".to_string(),
            file_path: None,
            word_count: 8,
            character_count: 42,
            created_at: Timestamp::now().to_string(),
            updated_at: Timestamp::now().to_string(),
            created_by: None,
            updated_by: None,
            version: 1,
            is_deleted: false,
            deleted_at: None,
            search_title: "test document".to_string(),
            search_content: "this is a test document with some content.".to_string(),
            search_tokens: vec!["test".to_string(), "document".to_string(), "content".to_string()],
        };
        
        let search_tokens = vec!["test".to_string(), "document".to_string()];
        let score = repo.calculate_relevance_score(&doc, &search_tokens);
        
        assert!(score > 0.0);
    }
}