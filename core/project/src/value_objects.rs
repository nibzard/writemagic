//! Project domain value objects

use writemagic_shared::{WritemagicError, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Project status indicating the current state of a project
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectStatus {
    Active,
    Paused,
    Completed,
    Archived,
}

impl fmt::Display for ProjectStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProjectStatus::Active => write!(f, "Active"),
            ProjectStatus::Paused => write!(f, "Paused"),
            ProjectStatus::Completed => write!(f, "Completed"),
            ProjectStatus::Archived => write!(f, "Archived"),
        }
    }
}

/// Project priority level
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ProjectPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Project color theme
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectColor {
    value: String,
}

impl ProjectColor {
    /// Create a new project color from a hex string
    pub fn new(hex_color: String) -> Result<Self> {
        if !Self::is_valid_hex_color(&hex_color) {
            return Err(WritemagicError::validation("Invalid hex color format"));
        }
        
        Ok(Self {
            value: hex_color.to_lowercase(),
        })
    }
    
    /// Get the hex value
    pub fn hex(&self) -> &str {
        &self.value
    }
    
    /// Validate hex color format
    fn is_valid_hex_color(color: &str) -> bool {
        if !color.starts_with('#') {
            return false;
        }
        
        let hex_part = &color[1..];
        if hex_part.len() != 6 && hex_part.len() != 3 {
            return false;
        }
        
        hex_part.chars().all(|c| c.is_ascii_hexdigit())
    }
}

/// Project tag for categorization
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectTag {
    value: String,
}

impl ProjectTag {
    /// Create a new project tag
    pub fn new(tag: String) -> Result<Self> {
        let normalized_tag = tag.trim().to_lowercase();
        
        if normalized_tag.is_empty() {
            return Err(WritemagicError::validation("Tag cannot be empty"));
        }
        
        if normalized_tag.len() > 50 {
            return Err(WritemagicError::validation("Tag cannot exceed 50 characters"));
        }
        
        Ok(Self {
            value: normalized_tag,
        })
    }
    
    /// Get the tag value
    pub fn value(&self) -> &str {
        &self.value
    }
}

/// Workspace pane size configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaneSize {
    percentage: f32,
}

impl PaneSize {
    /// Create a new pane size
    pub fn new(percentage: f32) -> Result<Self> {
        if percentage < 0.0 || percentage > 100.0 {
            return Err(WritemagicError::validation("Pane size must be between 0 and 100 percent"));
        }
        
        Ok(Self { percentage })
    }
    
    /// Get the percentage value
    pub fn percentage(&self) -> f32 {
        self.percentage
    }
    
    /// Convert to pixels given total container size
    pub fn to_pixels(&self, container_size: f32) -> f32 {
        (self.percentage / 100.0) * container_size
    }
}

/// Project goal or target
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectGoal {
    pub goal_type: GoalType,
    pub target_value: u32,
    pub current_value: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoalType {
    WordCount,
    DocumentCount,
    DailyWriting,
    Deadline,
}

impl ProjectGoal {
    /// Create a new project goal
    pub fn new(goal_type: GoalType, target_value: u32) -> Self {
        Self {
            goal_type,
            target_value,
            current_value: 0,
        }
    }
    
    /// Update progress towards goal
    pub fn update_progress(&mut self, new_value: u32) {
        self.current_value = new_value;
    }
    
    /// Check if goal is achieved
    pub fn is_achieved(&self) -> bool {
        self.current_value >= self.target_value
    }
    
    /// Get progress percentage
    pub fn progress_percentage(&self) -> f32 {
        if self.target_value == 0 {
            return 0.0;
        }
        
        (self.current_value as f32 / self.target_value as f32 * 100.0).min(100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_color() {
        assert!(ProjectColor::new("#ff0000".to_string()).is_ok());
        assert!(ProjectColor::new("#fff".to_string()).is_ok());
        assert!(ProjectColor::new("ff0000".to_string()).is_err());
    }
    
    #[test]
    fn test_project_tag() {
        assert!(ProjectTag::new("writing".to_string()).is_ok());
        assert!(ProjectTag::new("".to_string()).is_err());
    }
    
    #[test]
    fn test_pane_size() {
        assert!(PaneSize::new(50.0).is_ok());
        assert!(PaneSize::new(-1.0).is_err());
        assert!(PaneSize::new(101.0).is_err());
    }
}