//! SQLite repository implementations for writing domain

use async_trait::async_trait;
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;
use writemagic_shared::{EntityId, Pagination, Repository, Result, WritemagicError, Timestamp, ContentType, ContentHash, FilePath};
use crate::entities::{Document, Project};
use crate::repositories::{DocumentRepository, ProjectRepository, DocumentStatistics, ProjectStatistics};

/// SQLite document repository implementation
#[derive(Debug, Clone)]
pub struct SqliteDocumentRepository {
    pool: SqlitePool,
}

impl SqliteDocumentRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

/// Document struct for SQLite serialization
#[derive(Debug, Clone, sqlx::FromRow)]
struct SqliteDocument {
    pub id: String,
    pub title: String,
    pub content: String,
    pub content_type: String,
    pub content_hash: String,
    pub file_path: Option<String>,
    pub word_count: i64,
    pub character_count: i64,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
    pub version: i64,
    pub is_deleted: bool,
    pub deleted_at: Option<String>,
}

impl From<SqliteDocument> for Document {
    fn from(doc: SqliteDocument) -> Self {
        Document {
            id: EntityId::from_string(&doc.id).unwrap_or_else(|_| EntityId::new()),
            title: doc.title,
            content: doc.content,
            content_type: ContentType::from_string(&doc.content_type).unwrap_or(ContentType::Markdown),
            content_hash: ContentHash::from_string(&doc.content_hash),
            file_path: doc.file_path.map(|p| FilePath::new(&p).unwrap_or_default()),
            word_count: doc.word_count as u32,
            character_count: doc.character_count as u32,
            created_at: Timestamp::from_string(&doc.created_at).unwrap_or_else(|_| Timestamp::now()),
            updated_at: Timestamp::from_string(&doc.updated_at).unwrap_or_else(|_| Timestamp::now()),
            created_by: doc.created_by.and_then(|s| EntityId::from_string(&s).ok()),
            updated_by: doc.updated_by.and_then(|s| EntityId::from_string(&s).ok()),
            version: doc.version as u64,
            is_deleted: doc.is_deleted,
            deleted_at: doc.deleted_at.and_then(|s| Timestamp::from_string(&s).ok()),
        }
    }
}

impl From<&Document> for SqliteDocument {
    fn from(doc: &Document) -> Self {
        SqliteDocument {
            id: doc.id.to_string(),
            title: doc.title.clone(),
            content: doc.content.clone(),
            content_type: doc.content_type.to_string(),
            content_hash: doc.content_hash.to_string(),
            file_path: doc.file_path.as_ref().map(|p| p.to_string()),
            word_count: doc.word_count as i64,
            character_count: doc.character_count as i64,
            created_at: doc.created_at.to_string(),
            updated_at: doc.updated_at.to_string(),
            created_by: doc.created_by.as_ref().map(|id| id.to_string()),
            updated_by: doc.updated_by.as_ref().map(|id| id.to_string()),
            version: doc.version as i64,
            is_deleted: doc.is_deleted,
            deleted_at: doc.deleted_at.as_ref().map(|t| t.to_string()),
        }
    }
}

#[async_trait]
impl Repository<Document, EntityId> for SqliteDocumentRepository {
    async fn find_by_id(&self, id: &EntityId) -> Result<Option<Document>> {
        let row = sqlx::query_as::<_, SqliteDocument>(
            "SELECT * FROM documents WHERE id = ?"
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| WritemagicError::database(&format!("Failed to find document by id: {}", e)))?;

        Ok(row.map(|doc| doc.into()))
    }

    async fn find_all(&self, pagination: Pagination) -> Result<Vec<Document>> {
        let rows = sqlx::query_as::<_, SqliteDocument>(
            "SELECT * FROM documents WHERE is_deleted = FALSE ORDER BY updated_at DESC LIMIT ? OFFSET ?"
        )
        .bind(pagination.limit as i64)
        .bind(pagination.offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WritemagicError::database(&format!("Failed to find all documents: {}", e)))?;

        Ok(rows.into_iter().map(|doc| doc.into()).collect())
    }

    async fn save(&self, entity: &Document) -> Result<Document> {
        let sqlite_doc = SqliteDocument::from(entity);
        
        sqlx::query(
            r#"
            INSERT INTO documents (
                id, title, content, content_type, content_hash, file_path,
                word_count, character_count, created_at, updated_at,
                created_by, updated_by, version, is_deleted, deleted_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                content = excluded.content,
                content_type = excluded.content_type,
                content_hash = excluded.content_hash,
                file_path = excluded.file_path,
                word_count = excluded.word_count,
                character_count = excluded.character_count,
                updated_at = excluded.updated_at,
                updated_by = excluded.updated_by,
                version = excluded.version,
                is_deleted = excluded.is_deleted,
                deleted_at = excluded.deleted_at
            "#
        )
        .bind(&sqlite_doc.id)
        .bind(&sqlite_doc.title)
        .bind(&sqlite_doc.content)
        .bind(&sqlite_doc.content_type)
        .bind(&sqlite_doc.content_hash)
        .bind(&sqlite_doc.file_path)
        .bind(sqlite_doc.word_count)
        .bind(sqlite_doc.character_count)
        .bind(&sqlite_doc.created_at)
        .bind(&sqlite_doc.updated_at)
        .bind(&sqlite_doc.created_by)
        .bind(&sqlite_doc.updated_by)
        .bind(sqlite_doc.version)
        .bind(sqlite_doc.is_deleted)
        .bind(&sqlite_doc.deleted_at)
        .execute(&self.pool)
        .await
        .map_err(|e| WritemagicError::database(&format!("Failed to save document: {}", e)))?;

        Ok(entity.clone())
    }

    async fn delete(&self, id: &EntityId) -> Result<bool> {
        let result = sqlx::query("DELETE FROM documents WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| WritemagicError::database(&format!("Failed to delete document: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    async fn exists(&self, id: &EntityId) -> Result<bool> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM documents WHERE id = ?")
            .bind(id.to_string())
            .fetch_one(&self.pool)
            .await
            .map_err(|e| WritemagicError::database(&format!("Failed to check document existence: {}", e)))?;

        let count: i64 = row.get("count");
        Ok(count > 0)
    }

    async fn count(&self) -> Result<u64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM documents WHERE is_deleted = FALSE")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| WritemagicError::database(&format!("Failed to count documents: {}", e)))?;

        let count: i64 = row.get("count");
        Ok(count as u64)
    }
}

#[async_trait]
impl DocumentRepository for SqliteDocumentRepository {
    async fn find_by_project_id(&self, project_id: &EntityId, pagination: Pagination) -> Result<Vec<Document>> {
        let rows = sqlx::query_as::<_, SqliteDocument>(
            r#"
            SELECT d.* FROM documents d
            INNER JOIN project_documents pd ON d.id = pd.document_id
            WHERE pd.project_id = ? AND d.is_deleted = FALSE
            ORDER BY d.updated_at DESC
            LIMIT ? OFFSET ?
            "#
        )
        .bind(project_id.to_string())
        .bind(pagination.limit as i64)
        .bind(pagination.offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WritemagicError::database(&format!("Failed to find documents by project id: {}", e)))?;

        Ok(rows.into_iter().map(|doc| doc.into()).collect())
    }

    async fn find_by_content_type(&self, content_type: &ContentType, pagination: Pagination) -> Result<Vec<Document>> {
        let rows = sqlx::query_as::<_, SqliteDocument>(
            "SELECT * FROM documents WHERE content_type = ? AND is_deleted = FALSE ORDER BY updated_at DESC LIMIT ? OFFSET ?"
        )
        .bind(content_type.to_string())
        .bind(pagination.limit as i64)
        .bind(pagination.offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WritemagicError::database(&format!("Failed to find documents by content type: {}", e)))?;

        Ok(rows.into_iter().map(|doc| doc.into()).collect())
    }

    async fn search_by_title(&self, query: &str, pagination: Pagination) -> Result<Vec<Document>> {
        let search_query = format!("%{}%", query);
        let rows = sqlx::query_as::<_, SqliteDocument>(
            "SELECT * FROM documents WHERE title LIKE ? AND is_deleted = FALSE ORDER BY updated_at DESC LIMIT ? OFFSET ?"
        )
        .bind(&search_query)
        .bind(pagination.limit as i64)
        .bind(pagination.offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WritemagicError::database(&format!("Failed to search documents by title: {}", e)))?;

        Ok(rows.into_iter().map(|doc| doc.into()).collect())
    }

    async fn search_by_content(&self, query: &str, pagination: Pagination) -> Result<Vec<Document>> {
        // Try FTS first for better performance
        let fts_result = sqlx::query_as::<_, SqliteDocument>(
            r#"
            SELECT d.* FROM documents d
            INNER JOIN documents_fts fts ON d.id = fts.id
            WHERE documents_fts MATCH ? AND d.is_deleted = FALSE
            ORDER BY bm25(documents_fts), d.updated_at DESC
            LIMIT ? OFFSET ?
            "#
        )
        .bind(query)
        .bind(pagination.limit as i64)
        .bind(pagination.offset as i64)
        .fetch_all(&self.pool)
        .await;

        if let Ok(rows) = fts_result {
            return Ok(rows.into_iter().map(|doc| doc.into()).collect());
        }

        // Fallback to LIKE search if FTS fails
        log::warn!("FTS search failed, falling back to LIKE search for query: {}", query);
        let search_query = format!("%{}%", query);
        let rows = sqlx::query_as::<_, SqliteDocument>(
            "SELECT * FROM documents WHERE content LIKE ? AND is_deleted = FALSE ORDER BY updated_at DESC LIMIT ? OFFSET ?"
        )
        .bind(&search_query)
        .bind(pagination.limit as i64)
        .bind(pagination.offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WritemagicError::database(&format!("Failed to search documents by content: {}", e)))?;

        Ok(rows.into_iter().map(|doc| doc.into()).collect())
    }

    async fn find_by_creator(&self, user_id: &EntityId, pagination: Pagination) -> Result<Vec<Document>> {
        let rows = sqlx::query_as::<_, SqliteDocument>(
            "SELECT * FROM documents WHERE created_by = ? AND is_deleted = FALSE ORDER BY updated_at DESC LIMIT ? OFFSET ?"
        )
        .bind(user_id.to_string())
        .bind(pagination.limit as i64)
        .bind(pagination.offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WritemagicError::database(&format!("Failed to find documents by creator: {}", e)))?;

        Ok(rows.into_iter().map(|doc| doc.into()).collect())
    }

    async fn find_recently_updated(&self, pagination: Pagination) -> Result<Vec<Document>> {
        let rows = sqlx::query_as::<_, SqliteDocument>(
            "SELECT * FROM documents WHERE is_deleted = FALSE ORDER BY updated_at DESC LIMIT ? OFFSET ?"
        )
        .bind(pagination.limit as i64)
        .bind(pagination.offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WritemagicError::database(&format!("Failed to find recently updated documents: {}", e)))?;

        Ok(rows.into_iter().map(|doc| doc.into()).collect())
    }

    async fn find_deleted(&self, pagination: Pagination) -> Result<Vec<Document>> {
        let rows = sqlx::query_as::<_, SqliteDocument>(
            "SELECT * FROM documents WHERE is_deleted = TRUE ORDER BY deleted_at DESC LIMIT ? OFFSET ?"
        )
        .bind(pagination.limit as i64)
        .bind(pagination.offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WritemagicError::database(&format!("Failed to find deleted documents: {}", e)))?;

        Ok(rows.into_iter().map(|doc| doc.into()).collect())
    }

    async fn get_statistics(&self) -> Result<DocumentStatistics> {
        let stats_row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total_documents,
                COALESCE(SUM(word_count), 0) as total_word_count,
                COALESCE(SUM(character_count), 0) as total_character_count,
                COUNT(CASE WHEN is_deleted THEN 1 END) as deleted_documents,
                COALESCE(AVG(CAST(word_count AS REAL)), 0) as avg_word_count,
                COALESCE(AVG(CAST(character_count AS REAL)), 0) as avg_character_count
            FROM documents
            "#
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| WritemagicError::database(&format!("Failed to get document statistics: {}", e)))?;

        let total_documents: i64 = stats_row.get("total_documents");
        let total_word_count: i64 = stats_row.get("total_word_count");
        let total_character_count: i64 = stats_row.get("total_character_count");
        let deleted_documents: i64 = stats_row.get("deleted_documents");
        let avg_word_count: f64 = stats_row.get("avg_word_count");
        let avg_character_count: f64 = stats_row.get("avg_character_count");

        // Get documents by type
        let type_rows = sqlx::query(
            "SELECT content_type, COUNT(*) as count FROM documents GROUP BY content_type"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WritemagicError::database(&format!("Failed to get documents by type: {}", e)))?;

        let mut documents_by_type = HashMap::new();
        for row in type_rows {
            let content_type: String = row.get("content_type");
            let count: i64 = row.get("count");
            documents_by_type.insert(content_type, count as u64);
        }

        Ok(DocumentStatistics {
            total_documents: total_documents as u64,
            total_word_count: total_word_count as u64,
            total_character_count: total_character_count as u64,
            documents_by_type,
            average_word_count: avg_word_count,
            average_character_count: avg_character_count,
            deleted_documents: deleted_documents as u64,
        })
    }
}

/// SQLite project repository implementation
#[derive(Debug, Clone)]
pub struct SqliteProjectRepository {
    pool: SqlitePool,
}

impl SqliteProjectRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

/// Project struct for SQLite serialization
#[derive(Debug, Clone, sqlx::FromRow)]
struct SqliteProject {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
    pub version: i64,
    pub is_deleted: bool,
    pub deleted_at: Option<String>,
}

impl From<SqliteProject> for Project {
    fn from(proj: SqliteProject) -> Self {
        Project {
            id: EntityId::from_string(&proj.id).unwrap_or_else(|_| EntityId::new()),
            name: proj.name,
            description: proj.description,
            document_ids: Vec::new(), // Will be loaded separately
            created_at: Timestamp::from_string(&proj.created_at).unwrap_or_else(|_| Timestamp::now()),
            updated_at: Timestamp::from_string(&proj.updated_at).unwrap_or_else(|_| Timestamp::now()),
            created_by: proj.created_by.and_then(|s| EntityId::from_string(&s).ok()),
            updated_by: proj.updated_by.and_then(|s| EntityId::from_string(&s).ok()),
            version: proj.version as u64,
            is_deleted: proj.is_deleted,
            deleted_at: proj.deleted_at.and_then(|s| Timestamp::from_string(&s).ok()),
        }
    }
}

impl From<&Project> for SqliteProject {
    fn from(proj: &Project) -> Self {
        SqliteProject {
            id: proj.id.to_string(),
            name: proj.name.clone(),
            description: proj.description.clone(),
            created_at: proj.created_at.to_string(),
            updated_at: proj.updated_at.to_string(),
            created_by: proj.created_by.as_ref().map(|id| id.to_string()),
            updated_by: proj.updated_by.as_ref().map(|id| id.to_string()),
            version: proj.version as i64,
            is_deleted: proj.is_deleted,
            deleted_at: proj.deleted_at.as_ref().map(|t| t.to_string()),
        }
    }
}

#[async_trait]
impl Repository<Project, EntityId> for SqliteProjectRepository {
    async fn find_by_id(&self, id: &EntityId) -> Result<Option<Project>> {
        let row = sqlx::query_as::<_, SqliteProject>(
            "SELECT * FROM projects WHERE id = ?"
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| WritemagicError::database(&format!("Failed to find project by id: {}", e)))?;

        if let Some(proj) = row {
            let mut project = Project::from(proj);
            
            // Load document IDs
            let doc_rows = sqlx::query(
                "SELECT document_id FROM project_documents WHERE project_id = ?"
            )
            .bind(id.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(|e| WritemagicError::database(&format!("Failed to load project documents: {}", e)))?;

            project.document_ids = doc_rows.into_iter()
                .filter_map(|row| {
                    let doc_id: String = row.get("document_id");
                    EntityId::from_string(&doc_id).ok()
                })
                .collect();

            return Ok(Some(project));
        }

        Ok(None)
    }

    async fn find_all(&self, pagination: Pagination) -> Result<Vec<Project>> {
        let rows = sqlx::query_as::<_, SqliteProject>(
            "SELECT * FROM projects WHERE is_deleted = FALSE ORDER BY updated_at DESC LIMIT ? OFFSET ?"
        )
        .bind(pagination.limit as i64)
        .bind(pagination.offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WritemagicError::database(&format!("Failed to find all projects: {}", e)))?;

        let mut projects = Vec::new();
        for proj in rows {
            let mut project = Project::from(proj);
            
            // Load document IDs for each project
            let doc_rows = sqlx::query(
                "SELECT document_id FROM project_documents WHERE project_id = ?"
            )
            .bind(project.id.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(|e| WritemagicError::database(&format!("Failed to load project documents: {}", e)))?;

            project.document_ids = doc_rows.into_iter()
                .filter_map(|row| {
                    let doc_id: String = row.get("document_id");
                    EntityId::from_string(&doc_id).ok()
                })
                .collect();

            projects.push(project);
        }

        Ok(projects)
    }

    async fn save(&self, entity: &Project) -> Result<Project> {
        let mut tx = self.pool.begin().await
            .map_err(|e| WritemagicError::database(&format!("Failed to begin transaction: {}", e)))?;

        let sqlite_proj = SqliteProject::from(entity);
        
        // Save project
        sqlx::query(
            r#"
            INSERT INTO projects (
                id, name, description, created_at, updated_at,
                created_by, updated_by, version, is_deleted, deleted_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                description = excluded.description,
                updated_at = excluded.updated_at,
                updated_by = excluded.updated_by,
                version = excluded.version,
                is_deleted = excluded.is_deleted,
                deleted_at = excluded.deleted_at
            "#
        )
        .bind(&sqlite_proj.id)
        .bind(&sqlite_proj.name)
        .bind(&sqlite_proj.description)
        .bind(&sqlite_proj.created_at)
        .bind(&sqlite_proj.updated_at)
        .bind(&sqlite_proj.created_by)
        .bind(&sqlite_proj.updated_by)
        .bind(sqlite_proj.version)
        .bind(sqlite_proj.is_deleted)
        .bind(&sqlite_proj.deleted_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| WritemagicError::database(&format!("Failed to save project: {}", e)))?;

        // Clear existing document relationships
        sqlx::query("DELETE FROM project_documents WHERE project_id = ?")
            .bind(&sqlite_proj.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| WritemagicError::database(&format!("Failed to clear project documents: {}", e)))?;

        // Insert new document relationships
        for doc_id in &entity.document_ids {
            sqlx::query(
                "INSERT INTO project_documents (project_id, document_id) VALUES (?, ?)"
            )
            .bind(&sqlite_proj.id)
            .bind(doc_id.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| WritemagicError::database(&format!("Failed to save project document relationship: {}", e)))?;
        }

        tx.commit().await
            .map_err(|e| WritemagicError::database(&format!("Failed to commit transaction: {}", e)))?;

        Ok(entity.clone())
    }

    async fn delete(&self, id: &EntityId) -> Result<bool> {
        let mut tx = self.pool.begin().await
            .map_err(|e| WritemagicError::database(&format!("Failed to begin transaction: {}", e)))?;

        // Delete project documents relationships
        sqlx::query("DELETE FROM project_documents WHERE project_id = ?")
            .bind(id.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| WritemagicError::database(&format!("Failed to delete project documents: {}", e)))?;

        // Delete project
        let result = sqlx::query("DELETE FROM projects WHERE id = ?")
            .bind(id.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| WritemagicError::database(&format!("Failed to delete project: {}", e)))?;

        tx.commit().await
            .map_err(|e| WritemagicError::database(&format!("Failed to commit transaction: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    async fn exists(&self, id: &EntityId) -> Result<bool> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM projects WHERE id = ?")
            .bind(id.to_string())
            .fetch_one(&self.pool)
            .await
            .map_err(|e| WritemagicError::database(&format!("Failed to check project existence: {}", e)))?;

        let count: i64 = row.get("count");
        Ok(count > 0)
    }

    async fn count(&self) -> Result<u64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM projects WHERE is_deleted = FALSE")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| WritemagicError::database(&format!("Failed to count projects: {}", e)))?;

        let count: i64 = row.get("count");
        Ok(count as u64)
    }
}

#[async_trait]
impl ProjectRepository for SqliteProjectRepository {
    async fn find_by_creator(&self, user_id: &EntityId, pagination: Pagination) -> Result<Vec<Project>> {
        let rows = sqlx::query_as::<_, SqliteProject>(
            "SELECT * FROM projects WHERE created_by = ? AND is_deleted = FALSE ORDER BY updated_at DESC LIMIT ? OFFSET ?"
        )
        .bind(user_id.to_string())
        .bind(pagination.limit as i64)
        .bind(pagination.offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WritemagicError::database(&format!("Failed to find projects by creator: {}", e)))?;

        let mut projects = Vec::new();
        for proj in rows {
            let mut project = Project::from(proj);
            
            // Load document IDs
            let doc_rows = sqlx::query(
                "SELECT document_id FROM project_documents WHERE project_id = ?"
            )
            .bind(project.id.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(|e| WritemagicError::database(&format!("Failed to load project documents: {}", e)))?;

            project.document_ids = doc_rows.into_iter()
                .filter_map(|row| {
                    let doc_id: String = row.get("document_id");
                    EntityId::from_string(&doc_id).ok()
                })
                .collect();

            projects.push(project);
        }

        Ok(projects)
    }

    async fn search_by_name(&self, query: &str, pagination: Pagination) -> Result<Vec<Project>> {
        let search_query = format!("%{}%", query);
        let rows = sqlx::query_as::<_, SqliteProject>(
            "SELECT * FROM projects WHERE name LIKE ? AND is_deleted = FALSE ORDER BY updated_at DESC LIMIT ? OFFSET ?"
        )
        .bind(&search_query)
        .bind(pagination.limit as i64)
        .bind(pagination.offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WritemagicError::database(&format!("Failed to search projects by name: {}", e)))?;

        let mut projects = Vec::new();
        for proj in rows {
            let mut project = Project::from(proj);
            
            // Load document IDs
            let doc_rows = sqlx::query(
                "SELECT document_id FROM project_documents WHERE project_id = ?"
            )
            .bind(project.id.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(|e| WritemagicError::database(&format!("Failed to load project documents: {}", e)))?;

            project.document_ids = doc_rows.into_iter()
                .filter_map(|row| {
                    let doc_id: String = row.get("document_id");
                    EntityId::from_string(&doc_id).ok()
                })
                .collect();

            projects.push(project);
        }

        Ok(projects)
    }

    async fn find_containing_document(&self, document_id: &EntityId, pagination: Pagination) -> Result<Vec<Project>> {
        let rows = sqlx::query_as::<_, SqliteProject>(
            r#"
            SELECT p.* FROM projects p
            INNER JOIN project_documents pd ON p.id = pd.project_id
            WHERE pd.document_id = ? AND p.is_deleted = FALSE
            ORDER BY p.updated_at DESC
            LIMIT ? OFFSET ?
            "#
        )
        .bind(document_id.to_string())
        .bind(pagination.limit as i64)
        .bind(pagination.offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WritemagicError::database(&format!("Failed to find projects containing document: {}", e)))?;

        let mut projects = Vec::new();
        for proj in rows {
            let mut project = Project::from(proj);
            
            // Load document IDs
            let doc_rows = sqlx::query(
                "SELECT document_id FROM project_documents WHERE project_id = ?"
            )
            .bind(project.id.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(|e| WritemagicError::database(&format!("Failed to load project documents: {}", e)))?;

            project.document_ids = doc_rows.into_iter()
                .filter_map(|row| {
                    let doc_id: String = row.get("document_id");
                    EntityId::from_string(&doc_id).ok()
                })
                .collect();

            projects.push(project);
        }

        Ok(projects)
    }

    async fn find_recently_updated(&self, pagination: Pagination) -> Result<Vec<Project>> {
        self.find_all(pagination).await
    }

    async fn get_statistics(&self) -> Result<ProjectStatistics> {
        let stats_row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total_projects,
                (SELECT COUNT(*) FROM project_documents) as total_documents_in_projects
            FROM projects
            WHERE is_deleted = FALSE
            "#
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| WritemagicError::database(&format!("Failed to get project statistics: {}", e)))?;

        let total_projects: i64 = stats_row.get("total_projects");
        let total_documents_in_projects: i64 = stats_row.get("total_documents_in_projects");

        let average_documents_per_project = if total_projects > 0 {
            total_documents_in_projects as f64 / total_projects as f64
        } else {
            0.0
        };

        // Get project size statistics
        let size_row = sqlx::query(
            r#"
            SELECT 
                COALESCE(MAX(doc_count), 0) as largest_project_size,
                COALESCE(MIN(doc_count), 0) as smallest_project_size
            FROM (
                SELECT COUNT(*) as doc_count 
                FROM project_documents pd
                INNER JOIN projects p ON pd.project_id = p.id
                WHERE p.is_deleted = FALSE
                GROUP BY pd.project_id
            )
            "#
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| WritemagicError::database(&format!("Failed to get project size statistics: {}", e)))?;

        let (largest_project_size, smallest_project_size) = if let Some(row) = size_row {
            let largest: i64 = row.get("largest_project_size");
            let smallest: i64 = row.get("smallest_project_size");
            (largest as u64, smallest as u64)
        } else {
            (0, 0)
        };

        Ok(ProjectStatistics {
            total_projects: total_projects as u64,
            total_documents_in_projects: total_documents_in_projects as u64,
            average_documents_per_project,
            largest_project_size,
            smallest_project_size,
        })
    }
}