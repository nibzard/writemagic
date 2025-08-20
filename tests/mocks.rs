//! Mock services for testing

pub struct MockAIService {
    port: u16,
}

impl MockAIService {
    pub fn new(port: u16) -> Self {
        Self { port }
    }
    
    pub async fn start(&self) -> anyhow::Result<()> {
        // Mock service implementation would go here
        Ok(())
    }
}