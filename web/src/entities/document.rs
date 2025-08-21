use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "documents")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    
    pub user_id: String,
    
    pub title: String,
    
    pub description: Option<String>,
    
    pub content: Option<String>,
    
    pub tags: Json,
    
    pub created_at: ChronoDateTimeUtc,
    
    pub updated_at: ChronoDateTimeUtc,
    
    #[sea_orm(default_value = false)]
    pub is_deleted: bool,
    
    #[sea_orm(default_value = 0)]
    pub word_count: i32,
    
    #[sea_orm(default_value = 0)]
    pub char_count: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    User,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: Set(uuid::Uuid::new_v4().to_string()),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            is_deleted: Set(false),
            word_count: Set(0),
            char_count: Set(0),
            tags: Set(Json::Array(vec![])),
            ..ActiveModelTrait::default()
        }
    }

    // TODO: Fix SeaORM lifetime issues with before_save
    // async fn before_save<C>(mut self, _db: &C, insert: bool) -> Result<Self, DbErr>
    // where
    //     C: ConnectionTrait,
    // {
    //     if !insert {
    //         self.updated_at = Set(chrono::Utc::now());
    //     }
    //
    //     // Update word and character counts if content changed
    //     if let Set(Some(ref content_str)) = self.content {
    //         let word_count = content_str.split_whitespace().count() as i32;
    //         let char_count = content_str.chars().count() as i32;
    //         self.word_count = Set(word_count);
    //         self.char_count = Set(char_count);
    //     }
    //
    //     Ok(self)
    // }
}

impl Model {
    /// Get document tags as a vector of strings
    pub fn get_tags(&self) -> Vec<String> {
        match &self.tags {
            Json::Array(tags) => tags
                .iter()
                .filter_map(|tag| tag.as_str().map(String::from))
                .collect(),
            _ => vec![],
        }
    }

    /// Set document tags from a vector of strings
    pub fn set_tags(&mut self, tags: Vec<String>) {
        self.tags = Json::Array(
            tags.into_iter()
                .map(serde_json::Value::String)
                .collect()
        );
    }

    /// Check if document is soft deleted
    pub fn is_deleted(&self) -> bool {
        self.is_deleted
    }

    /// Get content length statistics
    pub fn content_stats(&self) -> (i32, i32) {
        (self.word_count, self.char_count)
    }

    /// Check if document has content
    pub fn has_content(&self) -> bool {
        self.content.as_ref().map_or(false, |c| !c.trim().is_empty())
    }

    /// Get content preview (first 100 characters)
    pub fn content_preview(&self) -> Option<String> {
        self.content.as_ref().map(|content| {
            if content.len() > 100 {
                format!("{}...", &content[..97])
            } else {
                content.clone()
            }
        })
    }
}

/// Document creation helper
#[allow(dead_code)]
pub struct DocumentBuilder {
    user_id: String,
    title: String,
    description: Option<String>,
    content: Option<String>,
    tags: Vec<String>,
}

#[allow(dead_code)]
impl DocumentBuilder {
    pub fn new(user_id: String, title: String) -> Self {
        Self {
            user_id,
            title,
            description: None,
            content: None,
            tags: vec![],
        }
    }

    pub fn description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn content(mut self, content: String) -> Self {
        self.content = Some(content);
        self
    }

    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn build(self) -> ActiveModel {
        let mut model = ActiveModel::new();
        model.user_id = Set(self.user_id);
        model.title = Set(self.title);
        
        if let Some(description) = self.description {
            model.description = Set(Some(description));
        }
        
        if let Some(content) = self.content {
            let word_count = content.split_whitespace().count() as i32;
            let char_count = content.chars().count() as i32;
            model.content = Set(Some(content));
            model.word_count = Set(word_count);
            model.char_count = Set(char_count);
        }
        
        if !self.tags.is_empty() {
            model.tags = Set(Json::Array(
                self.tags.into_iter()
                    .map(serde_json::Value::String)
                    .collect()
            ));
        }
        
        model
    }
}