// Minimal test to validate the Rust environment

use std::process::Command;

fn main() {
    println!("Testing minimal Rust compilation...");
    
    // Test basic Rust functionality
    let vec = vec![1, 2, 3, 4, 5];
    let sum: i32 = vec.iter().sum();
    println!("Basic math test: sum = {}", sum);
    assert_eq!(sum, 15);
    
    // Test async functionality (basic)
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async {
        println!("Async runtime test: OK");
    });
    
    // Test serde functionality
    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct TestStruct {
        name: String,
        value: i32,
    }
    
    let test = TestStruct {
        name: "test".to_string(),
        value: 42,
    };
    
    let json = serde_json::to_string(&test).expect("Serialization failed");
    let deserialized: TestStruct = serde_json::from_str(&json).expect("Deserialization failed");
    assert_eq!(test, deserialized);
    println!("Serde test: OK");
    
    println!("✓ All minimal tests passed!");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_basic_functionality() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
    
    #[tokio::test]
    async fn test_async_functionality() {
        let result = async { 42 }.await;
        assert_eq!(result, 42);
    }
}