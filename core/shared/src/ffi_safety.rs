//! FFI safety patterns and utilities

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::panic::{catch_unwind, UnwindSafe};
use std::ptr;

/// Result type for FFI operations
pub type FFIResult<T> = Result<T, FFIError>;

/// FFI-specific error types
#[derive(Debug, Clone)]
pub enum FFIError {
    NullPointer,
    InvalidUtf8,
    InvalidInput(String),
    Panic(String),
    InternalError(String),
}

impl FFIError {
    pub fn to_c_code(&self) -> c_int {
        match self {
            Self::NullPointer => -1,
            Self::InvalidUtf8 => -2,
            Self::InvalidInput(_) => -3,
            Self::Panic(_) => -4,
            Self::InternalError(_) => -5,
        }
    }
}

impl From<crate::WritemagicError> for FFIError {
    fn from(err: crate::WritemagicError) -> Self {
        Self::InternalError(err.to_string())
    }
}

/// Safe wrapper for C string handling
pub struct SafeCString {
    inner: CString,
}

impl SafeCString {
    pub fn new(s: impl Into<String>) -> Result<Self, FFIError> {
        let string = s.into();
        CString::new(string)
            .map(|inner| Self { inner })
            .map_err(|_| FFIError::InvalidInput("String contains null bytes".to_string()))
    }

    pub fn as_ptr(&self) -> *const c_char {
        self.inner.as_ptr()
    }

    pub fn into_raw(self) -> *mut c_char {
        self.inner.into_raw()
    }
}

/// Safe wrapper for reading C strings from FFI
pub struct SafeStringReader;

impl SafeStringReader {
    /// Safely read a C string from a pointer
    /// 
    /// # Safety
    /// Caller must ensure the pointer is valid and null-terminated
    pub unsafe fn read_c_string(ptr: *const c_char) -> FFIResult<String> {
        if ptr.is_null() {
            return Err(FFIError::NullPointer);
        }

        CStr::from_ptr(ptr)
            .to_str()
            .map(|s| s.to_owned())
            .map_err(|_| FFIError::InvalidUtf8)
    }

    /// Safely read an optional C string
    /// 
    /// # Safety  
    /// If ptr is not null, caller must ensure it's valid and null-terminated
    pub unsafe fn read_optional_c_string(ptr: *const c_char) -> FFIResult<Option<String>> {
        if ptr.is_null() {
            Ok(None)
        } else {
            Self::read_c_string(ptr).map(Some)
        }
    }
}

/// Free a C string that was allocated by Rust
/// 
/// # Safety
/// The pointer must have been created by CString::into_raw() and not freed already
#[no_mangle]
pub unsafe extern "C" fn writemagic_free_string(s: *mut c_char) {
    if !s.is_null() {
        let _ = CString::from_raw(s);
    }
}

/// Helper to catch panics at FFI boundary
pub fn catch_ffi_panic<F, T>(f: F) -> FFIResult<T>
where
    F: FnOnce() -> FFIResult<T> + UnwindSafe,
{
    match catch_unwind(f) {
        Ok(result) => result,
        Err(_) => Err(FFIError::Panic("Function panicked".to_string())),
    }
}

/// FFI handle wrapper for safe object management
#[repr(C)]
pub struct FFIHandle<T> {
    inner: *mut T,
    type_id: u64,
    magic: u32,
}

const MAGIC_NUMBER: u32 = 0xDEADBEEF;

impl<T> FFIHandle<T> {
    /// Create a new FFI handle from a boxed value
    pub fn new(value: T) -> Self {
        let boxed = Box::new(value);
        Self {
            inner: Box::into_raw(boxed),
            type_id: std::any::TypeId::of::<T>().into(), // Convert TypeId to u64
            magic: MAGIC_NUMBER,
        }
    }

    /// Get a reference to the inner value
    /// 
    /// # Safety
    /// Handle must be valid and not freed
    pub unsafe fn get(&self) -> Result<&T, FFIError> {
        self.validate()?;
        Ok(&*self.inner)
    }

    /// Get a mutable reference to the inner value
    /// 
    /// # Safety
    /// Handle must be valid and not freed
    pub unsafe fn get_mut(&mut self) -> Result<&mut T, FFIError> {
        self.validate()?;
        Ok(&mut *self.inner)
    }

    /// Validate the handle
    fn validate(&self) -> Result<(), FFIError> {
        if self.magic != MAGIC_NUMBER {
            return Err(FFIError::InvalidInput("Invalid handle magic number".to_string()));
        }
        
        if self.inner.is_null() {
            return Err(FFIError::NullPointer);
        }

        if self.type_id != std::any::TypeId::of::<T>().into() {
            return Err(FFIError::InvalidInput("Handle type mismatch".to_string()));
        }

        Ok(())
    }

    /// Free the handle and its contained value
    /// 
    /// # Safety
    /// Handle must be valid and not freed already
    pub unsafe fn free(mut self) -> Result<T, FFIError> {
        self.validate()?;
        let value = Box::from_raw(self.inner);
        self.inner = ptr::null_mut(); // Mark as freed
        self.magic = 0; // Invalidate magic
        Ok(*value)
    }
}

// Custom implementation because TypeId doesn't implement Into<u64>
trait TypeIdExt {
    fn into(self) -> u64;
}

impl TypeIdExt for std::any::TypeId {
    fn into(self) -> u64 {
        // Use a hash of the TypeId for conversion
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

/// FFI-safe error result that can be returned to C
#[repr(C)]
pub struct FFIErrorResult {
    pub code: c_int,
    pub message: *mut c_char,
}

impl FFIErrorResult {
    /// Create an error result from an FFI error
    pub fn from_error(error: FFIError) -> Self {
        let message = SafeCString::new(format!("{:?}", error))
            .map(|s| s.into_raw())
            .unwrap_or(ptr::null_mut());

        Self {
            code: error.to_c_code(),
            message,
        }
    }

    /// Create a success result
    pub fn success() -> Self {
        Self {
            code: 0,
            message: ptr::null_mut(),
        }
    }
}

/// Macro to wrap FFI functions with panic catching and error handling
#[macro_export]
macro_rules! ffi_wrapper {
    ($func_name:ident, $body:expr) => {
        #[no_mangle]
        pub extern "C" fn $func_name() -> $crate::ffi_safety::FFIErrorResult {
            use $crate::ffi_safety::{catch_ffi_panic, FFIErrorResult};
            
            match catch_ffi_panic(|| $body) {
                Ok(_) => FFIErrorResult::success(),
                Err(e) => FFIErrorResult::from_error(e),
            }
        }
    };
    
    ($func_name:ident($($param:ident: $param_type:ty),*) -> $ret_type:ty, $body:expr) => {
        #[no_mangle]
        pub extern "C" fn $func_name($($param: $param_type),*) -> $ret_type {
            use $crate::ffi_safety::{catch_ffi_panic, FFIError};
            
            match catch_ffi_panic(|| $body) {
                Ok(result) => result,
                Err(e) => {
                    // Log the error and return default/error value
                    log::error!("FFI function {} failed: {:?}", stringify!($func_name), e);
                    Default::default()
                }
            }
        }
    };
}

/// Thread-safe singleton for FFI state management
pub struct FFISingleton<T> {
    value: std::sync::OnceLock<T>,
}

impl<T> FFISingleton<T> {
    pub const fn new() -> Self {
        Self {
            value: std::sync::OnceLock::new(),
        }
    }

    pub fn get_or_init<F>(&self, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        self.value.get_or_init(f)
    }

    pub fn get(&self) -> Option<&T> {
        self.value.get()
    }
}

unsafe impl<T: Send> Send for FFISingleton<T> {}
unsafe impl<T: Send + Sync> Sync for FFISingleton<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_c_string() {
        let s = SafeCString::new("Hello, World!").unwrap();
        assert!(!s.as_ptr().is_null());
        
        // Test with null bytes
        let result = SafeCString::new("Hello\0World");
        assert!(matches!(result, Err(FFIError::InvalidInput(_))));
    }

    #[test]
    fn test_ffi_handle() {
        let handle = FFIHandle::new(42u32);
        
        unsafe {
            let value = handle.get().unwrap();
            assert_eq!(*value, 42);
            
            let owned = handle.free().unwrap();
            assert_eq!(owned, 42);
        }
    }

    #[test]
    fn test_ffi_singleton() {
        static SINGLETON: FFISingleton<String> = FFISingleton::new();
        
        let value = SINGLETON.get_or_init(|| "Hello".to_string());
        assert_eq!(value, "Hello");
        
        let value2 = SINGLETON.get().unwrap();
        assert_eq!(value2, "Hello");
    }

    #[test]
    fn test_string_reader() {
        let c_string = CString::new("test").unwrap();
        let ptr = c_string.as_ptr();
        
        unsafe {
            let result = SafeStringReader::read_c_string(ptr).unwrap();
            assert_eq!(result, "test");
            
            let result = SafeStringReader::read_c_string(std::ptr::null());
            assert!(matches!(result, Err(FFIError::NullPointer)));
        }
    }
}