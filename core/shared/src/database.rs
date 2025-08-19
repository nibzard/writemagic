//! Database initialization and migration system

use sqlx::{Row, SqliteConnection, SqlitePool};
use crate::{Result, WritemagicError};

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub database_url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub enable_wal: bool,
    pub enable_foreign_keys: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            database_url: "sqlite://writemagic.db".to_string(),
            max_connections: 10,
            min_connections: 1,
            enable_wal: true,
            enable_foreign_keys: true,
        }
    }
}

/// Database manager for SQLite operations
pub struct DatabaseManager {
    pool: SqlitePool,
    _config: DatabaseConfig,
}

impl DatabaseManager {
    /// Create a new database manager with configuration
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        let pool = if config.database_url == "sqlite::memory:" {
            // Special handling for in-memory database
            SqlitePool::connect("sqlite::memory:").await.map_err(|e| {
                WritemagicError::database(&format!("Failed to connect to database: {}", e))
            })?
        } else {
            SqlitePool::connect_with(
                sqlx::sqlite::SqliteConnectOptions::new()
                    .filename(&config.database_url.replace("sqlite://", ""))
                    .create_if_missing(true)
                    .journal_mode(if config.enable_wal {
                        sqlx::sqlite::SqliteJournalMode::Wal
                    } else {
                        sqlx::sqlite::SqliteJournalMode::Delete
                    })
                    .foreign_keys(config.enable_foreign_keys)
                    .busy_timeout(std::time::Duration::from_secs(30))
            ).await.map_err(|e| {
                WritemagicError::database(&format!("Failed to connect to database: {}", e))
            })?
        };

        let manager = Self { pool, _config: config };
        
        // Run initial setup
        manager.setup().await?;
        
        Ok(manager)
    }

    /// Create database manager with default configuration
    pub async fn new_default() -> Result<Self> {
        Self::new(DatabaseConfig::default()).await
    }

    /// Create database manager with in-memory database for testing
    pub async fn new_in_memory() -> Result<Self> {
        let config = DatabaseConfig {
            database_url: "sqlite::memory:".to_string(),
            max_connections: 1,
            min_connections: 1,
            enable_wal: false,
            enable_foreign_keys: true,
        };
        Self::new(config).await
    }

    /// Get the connection pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Setup database with initial configuration
    async fn setup(&self) -> Result<()> {
        let mut conn = self.pool.acquire().await.map_err(|e| {
            WritemagicError::database(&format!("Failed to acquire connection: {}", e))
        })?;

        // Enable pragmas for performance and integrity
        sqlx::query("PRAGMA journal_mode = WAL")
            .execute(&mut *conn)
            .await
            .map_err(|e| WritemagicError::database(&format!("Failed to set journal mode: {}", e)))?;

        sqlx::query("PRAGMA synchronous = NORMAL")
            .execute(&mut *conn)
            .await
            .map_err(|e| WritemagicError::database(&format!("Failed to set synchronous mode: {}", e)))?;

        sqlx::query("PRAGMA cache_size = 1000")
            .execute(&mut *conn)
            .await
            .map_err(|e| WritemagicError::database(&format!("Failed to set cache size: {}", e)))?;

        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&mut *conn)
            .await
            .map_err(|e| WritemagicError::database(&format!("Failed to enable foreign keys: {}", e)))?;

        // Run migrations
        self.run_migrations(&mut conn).await?;

        Ok(())
    }

    /// Run database migrations
    async fn run_migrations(&self, conn: &mut SqliteConnection) -> Result<()> {
        // Create migrations table if it doesn't exist
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS migrations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&mut *conn)
        .await
        .map_err(|e| WritemagicError::database(&format!("Failed to create migrations table: {}", e)))?;

        // Run each migration
        for migration in MIGRATIONS {
            if !self.is_migration_applied(conn, migration.name).await? {
                log::info!("Applying migration: {}", migration.name);
                
                // Execute migration
                sqlx::query(migration.sql)
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| WritemagicError::database(&format!("Failed to apply migration {}: {}", migration.name, e)))?;

                // Record migration as applied
                sqlx::query(
                    "INSERT INTO migrations (name) VALUES (?)"
                )
                .bind(migration.name)
                .execute(&mut *conn)
                .await
                .map_err(|e| WritemagicError::database(&format!("Failed to record migration {}: {}", migration.name, e)))?;
            }
        }

        Ok(())
    }

    /// Check if migration has been applied
    async fn is_migration_applied(&self, conn: &mut SqliteConnection, name: &str) -> Result<bool> {
        let row = sqlx::query(
            "SELECT COUNT(*) as count FROM migrations WHERE name = ?"
        )
        .bind(name)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| WritemagicError::database(&format!("Failed to check migration status: {}", e)))?;

        let count: i64 = row.get("count");
        Ok(count > 0)
    }

    /// Get migration status
    pub async fn get_migration_status(&self) -> Result<Vec<MigrationStatus>> {
        let mut conn = self.pool.acquire().await.map_err(|e| {
            WritemagicError::database(&format!("Failed to acquire connection: {}", e))
        })?;

        let rows = sqlx::query(
            "SELECT name, applied_at FROM migrations ORDER BY applied_at"
        )
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| WritemagicError::database(&format!("Failed to get migration status: {}", e)))?;

        let mut status = Vec::new();
        for migration in MIGRATIONS {
            let applied = rows.iter().find(|row| {
                let name: String = row.get("name");
                name == migration.name
            });

            status.push(MigrationStatus {
                name: migration.name.to_string(),
                applied: applied.is_some(),
                applied_at: applied.and_then(|row| row.get("applied_at")),
            });
        }

        Ok(status)
    }

    /// Close the database connection pool
    pub async fn close(&self) {
        self.pool.close().await;
    }
}

/// Migration definition
#[derive(Debug)]
struct Migration {
    name: &'static str,
    sql: &'static str,
}

/// Migration status
#[derive(Debug, Clone)]
pub struct MigrationStatus {
    pub name: String,
    pub applied: bool,
    pub applied_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// All database migrations in order
const MIGRATIONS: &[Migration] = &[
    Migration {
        name: "001_create_documents",
        sql: r#"
            CREATE TABLE documents (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                content_type TEXT NOT NULL,
                content_hash TEXT NOT NULL,
                file_path TEXT,
                word_count INTEGER NOT NULL DEFAULT 0,
                character_count INTEGER NOT NULL DEFAULT 0,
                created_at DATETIME NOT NULL,
                updated_at DATETIME NOT NULL,
                created_by TEXT,
                updated_by TEXT,
                version INTEGER NOT NULL DEFAULT 1,
                is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
                deleted_at DATETIME
            )
        "#,
    },
    Migration {
        name: "002_create_projects",
        sql: r#"
            CREATE TABLE projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                created_at DATETIME NOT NULL,
                updated_at DATETIME NOT NULL,
                created_by TEXT,
                updated_by TEXT,
                version INTEGER NOT NULL DEFAULT 1,
                is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
                deleted_at DATETIME
            )
        "#,
    },
    Migration {
        name: "003_create_project_documents",
        sql: r#"
            CREATE TABLE project_documents (
                project_id TEXT NOT NULL,
                document_id TEXT NOT NULL,
                added_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (project_id, document_id),
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
                FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE
            )
        "#,
    },
    Migration {
        name: "004_create_indexes",
        sql: r#"
            -- Document indexes for performance
            CREATE INDEX idx_documents_title ON documents(title);
            CREATE INDEX idx_documents_content_type ON documents(content_type);
            CREATE INDEX idx_documents_created_by ON documents(created_by);
            CREATE INDEX idx_documents_updated_at ON documents(updated_at);
            CREATE INDEX idx_documents_created_at ON documents(created_at);
            CREATE INDEX idx_documents_is_deleted ON documents(is_deleted);
            
            -- Project indexes for performance
            CREATE INDEX idx_projects_name ON projects(name);
            CREATE INDEX idx_projects_created_by ON projects(created_by);
            CREATE INDEX idx_projects_updated_at ON projects(updated_at);
            CREATE INDEX idx_projects_created_at ON projects(created_at);
            CREATE INDEX idx_projects_is_deleted ON projects(is_deleted);
            
            -- Project documents indexes
            CREATE INDEX idx_project_documents_project_id ON project_documents(project_id);
            CREATE INDEX idx_project_documents_document_id ON project_documents(document_id);
        "#,
    },
    Migration {
        name: "005_create_fts_documents",
        sql: r#"
            -- Full-text search for documents
            CREATE VIRTUAL TABLE documents_fts USING fts5(
                id,
                title,
                content,
                content=documents,
                content_rowid=rowid
            );
            
            -- Trigger to keep FTS table synchronized
            CREATE TRIGGER documents_fts_insert AFTER INSERT ON documents BEGIN
                INSERT INTO documents_fts(rowid, id, title, content) 
                VALUES (new.rowid, new.id, new.title, new.content);
            END;
            
            CREATE TRIGGER documents_fts_delete AFTER DELETE ON documents BEGIN
                INSERT INTO documents_fts(documents_fts, rowid, id, title, content) 
                VALUES('delete', old.rowid, old.id, old.title, old.content);
            END;
            
            CREATE TRIGGER documents_fts_update AFTER UPDATE ON documents BEGIN
                INSERT INTO documents_fts(documents_fts, rowid, id, title, content) 
                VALUES('delete', old.rowid, old.id, old.title, old.content);
                INSERT INTO documents_fts(rowid, id, title, content) 
                VALUES (new.rowid, new.id, new.title, new.content);
            END;
        "#,
    },
];