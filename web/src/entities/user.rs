use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    
    #[sea_orm(unique)]
    pub username: String,
    
    #[sea_orm(unique)]
    pub email: String,
    
    pub password_hash: String,
    
    pub created_at: ChronoDateTimeUtc,
    
    pub updated_at: ChronoDateTimeUtc,
    
    #[sea_orm(default_value = true)]
    pub is_active: bool,
    
    #[sea_orm(default_value = "user")]
    pub role: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::document::Entity")]
    Documents,
    
    #[sea_orm(has_many = "super::project::Entity")]
    Projects,
}

impl Related<super::document::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Documents.def()
    }
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Projects.def()
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: Set(uuid::Uuid::new_v4().to_string()),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            is_active: Set(true),
            role: Set("user".to_string()),
            ..ActiveModelTrait::default()
        }
    }

    fn before_save<C>(mut self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if !insert {
            self.updated_at = Set(chrono::Utc::now());
        }
        Ok(self)
    }
}

impl Model {
    /// Check if user has admin role
    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }

    /// Check if user account is active
    pub fn is_account_active(&self) -> bool {
        self.is_active
    }

    /// Get display name (username for now, could be extended to include first/last name)
    pub fn display_name(&self) -> &str {
        &self.username
    }
}

/// User roles enum for type safety
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UserRole {
    Admin,
    User,
}

impl UserRole {
    pub fn as_str(&self) -> &str {
        match self {
            UserRole::Admin => "admin",
            UserRole::User => "user",
        }
    }

    pub fn from_str(role: &str) -> Self {
        match role {
            "admin" => UserRole::Admin,
            _ => UserRole::User,
        }
    }
}

impl From<UserRole> for String {
    fn from(role: UserRole) -> Self {
        role.as_str().to_string()
    }
}

impl From<String> for UserRole {
    fn from(role: String) -> Self {
        UserRole::from_str(&role)
    }
}