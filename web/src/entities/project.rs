use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "projects")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    
    pub user_id: String,
    
    pub name: String,
    
    pub description: Option<String>,
    
    pub settings: Json,
    
    pub created_at: ChronoDateTimeUtc,
    
    pub updated_at: ChronoDateTimeUtc,
    
    #[sea_orm(default_value = false)]
    pub is_deleted: bool,
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
            settings: Set(Json::Object(serde_json::Map::new())),
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
    //     Ok(self)
    // }
}

impl Model {
    /// Check if project is soft deleted
    pub fn is_deleted(&self) -> bool {
        self.is_deleted
    }

    /// Get project settings as a serde_json::Value
    pub fn get_settings(&self) -> &Json {
        &self.settings
    }

    /// Set project settings from a serde_json::Value
    pub fn set_settings(&mut self, settings: serde_json::Value) {
        self.settings = Json::from(settings);
    }

    /// Get a specific setting value by key
    pub fn get_setting(&self, key: &str) -> Option<&serde_json::Value> {
        match &self.settings {
            Json::Object(map) => map.get(key),
            _ => None,
        }
    }

    /// Set a specific setting value by key
    pub fn set_setting(&mut self, key: String, value: serde_json::Value) {
        match &mut self.settings {
            Json::Object(map) => {
                map.insert(key, value);
            }
            _ => {
                let mut map = serde_json::Map::new();
                map.insert(key, value);
                self.settings = Json::Object(map);
            }
        }
    }
}

/// Project settings structure for type safety
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    pub theme: Option<String>,
    pub auto_save: Option<bool>,
    pub word_wrap: Option<bool>,
    pub font_size: Option<u32>,
    pub show_word_count: Option<bool>,
    pub default_document_template: Option<String>,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            theme: Some("default".to_string()),
            auto_save: Some(true),
            word_wrap: Some(true),
            font_size: Some(14),
            show_word_count: Some(true),
            default_document_template: None,
        }
    }
}

#[allow(dead_code)]
impl ProjectSettings {
    pub fn from_json(json: &Json) -> Result<Self, serde_json::Error> {
        serde_json::from_value(json.clone().into())
    }

    pub fn to_json(&self) -> Json {
        Json::from(serde_json::to_value(self).unwrap_or_default())
    }
}

/// Project creation helper
pub struct ProjectBuilder {
    user_id: String,
    name: String,
    description: Option<String>,
    settings: ProjectSettings,
}

impl ProjectBuilder {
    pub fn new(user_id: String, name: String) -> Self {
        Self {
            user_id,
            name,
            description: None,
            settings: ProjectSettings::default(),
        }
    }

    pub fn description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn settings(mut self, settings: ProjectSettings) -> Self {
        self.settings = settings;
        self
    }

    pub fn build(self) -> ActiveModel {
        let mut model = ActiveModel::new();
        model.user_id = Set(self.user_id);
        model.name = Set(self.name);
        
        if let Some(description) = self.description {
            model.description = Set(Some(description));
        }
        
        model.settings = Set(self.settings.to_json());
        
        model
    }
}