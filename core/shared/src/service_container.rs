//! Service container patterns to reduce Arc overhead

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;

/// Service container that avoids Arc for single-threaded access patterns
pub struct ServiceContainer {
    services: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl ServiceContainer {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    /// Register a service by type
    pub fn register<T: Send + Sync + 'static>(&mut self, service: T) {
        self.services.insert(TypeId::of::<T>(), Box::new(service));
    }

    /// Get a reference to a service by type
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.services
            .get(&TypeId::of::<T>())
            .and_then(|service| service.downcast_ref::<T>())
    }

    /// Get a mutable reference to a service by type
    pub fn get_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
        self.services
            .get_mut(&TypeId::of::<T>())
            .and_then(|service| service.downcast_mut::<T>())
    }
}

impl Default for ServiceContainer {
    fn default() -> Self {
        Self::new()
    }
}

/// Service reference that avoids Arc cloning
pub struct ServiceRef<'a, T> {
    service: &'a T,
    _phantom: PhantomData<T>,
}

impl<'a, T> ServiceRef<'a, T> {
    pub fn new(service: &'a T) -> Self {
        Self {
            service,
            _phantom: PhantomData,
        }
    }

    pub fn get(&self) -> &T {
        self.service
    }
}

impl<'a, T> std::ops::Deref for ServiceRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.service
    }
}

/// Provider registry using generics instead of trait objects to avoid Arc
pub struct ProviderRegistry<C, O> 
where
    C: Send + Sync,
    O: Send + Sync,
{
    claude_provider: Option<C>,
    openai_provider: Option<O>,
}

impl<C, O> ProviderRegistry<C, O>
where
    C: Send + Sync,
    O: Send + Sync,
{
    pub fn new() -> Self {
        Self {
            claude_provider: None,
            openai_provider: None,
        }
    }

    pub fn with_claude(mut self, provider: C) -> Self {
        self.claude_provider = Some(provider);
        self
    }

    pub fn with_openai(mut self, provider: O) -> Self {
        self.openai_provider = Some(provider);
        self
    }

    pub fn claude(&self) -> Option<&C> {
        self.claude_provider.as_ref()
    }

    pub fn openai(&self) -> Option<&O> {
        self.openai_provider.as_ref()
    }

    pub fn claude_mut(&mut self) -> Option<&mut C> {
        self.claude_provider.as_mut()
    }

    pub fn openai_mut(&mut self) -> Option<&mut O> {
        self.openai_provider.as_mut()
    }
}

impl<C, O> Default for ProviderRegistry<C, O>
where
    C: Send + Sync,
    O: Send + Sync,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Static service registry for read-only services
pub struct StaticServiceRegistry {
    _private: (),
}

impl StaticServiceRegistry {
    /// Store a service as a static reference
    /// 
    /// # Safety
    /// The service must have a static lifetime and be initialized once
    pub fn register_static<T: Send + Sync + 'static>(service: &'static T) -> StaticServiceRef<T> {
        StaticServiceRef::new(service)
    }
}

/// Reference to a static service that avoids Arc overhead
pub struct StaticServiceRef<T: Send + Sync> {
    service: &'static T,
}

impl<T: Send + Sync> StaticServiceRef<T> {
    fn new(service: &'static T) -> Self {
        Self { service }
    }

    pub fn get(&self) -> &'static T {
        self.service
    }
}

impl<T: Send + Sync> Clone for StaticServiceRef<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Send + Sync> Copy for StaticServiceRef<T> {}

impl<T: Send + Sync> std::ops::Deref for StaticServiceRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.service
    }
}

/// Service locator pattern that uses interior mutability instead of Arc
pub struct ServiceLocator {
    container: std::cell::RefCell<ServiceContainer>,
}

impl ServiceLocator {
    pub fn new() -> Self {
        Self {
            container: std::cell::RefCell::new(ServiceContainer::new()),
        }
    }

    pub fn register<T: Send + Sync + 'static>(&self, service: T) {
        self.container.borrow_mut().register(service);
    }

    pub fn with_service<T: Send + Sync + 'static, R>(&self, f: impl FnOnce(&T) -> R) -> Option<R> {
        let container = self.container.borrow();
        container.get::<T>().map(f)
    }

    pub fn with_service_mut<T: Send + Sync + 'static, R>(
        &self, 
        f: impl FnOnce(&mut T) -> R
    ) -> Option<R> {
        let mut container = self.container.borrow_mut();
        container.get_mut::<T>().map(f)
    }
}

unsafe impl Send for ServiceLocator {}
unsafe impl Sync for ServiceLocator {}

impl Default for ServiceLocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct TestService {
        value: u32,
    }

    impl TestService {
        fn new(value: u32) -> Self {
            Self { value }
        }
        
        fn increment(&mut self) {
            self.value += 1;
        }
    }

    #[test]
    fn test_service_container() {
        let mut container = ServiceContainer::new();
        container.register(TestService::new(42));

        let service = container.get::<TestService>().unwrap();
        assert_eq!(service.value, 42);

        let service_mut = container.get_mut::<TestService>().unwrap();
        service_mut.increment();
        assert_eq!(service_mut.value, 43);
    }

    #[test] 
    fn test_provider_registry() {
        let registry = ProviderRegistry::new()
            .with_claude(TestService::new(1))
            .with_openai(TestService::new(2));

        assert_eq!(registry.claude().unwrap().value, 1);
        assert_eq!(registry.openai().unwrap().value, 2);
    }

    #[test]
    fn test_service_locator() {
        let locator = ServiceLocator::new();
        locator.register(TestService::new(100));

        let result = locator.with_service::<TestService, u32>(|service| service.value);
        assert_eq!(result, Some(100));

        locator.with_service_mut::<TestService, ()>(|service| service.increment());
        
        let result = locator.with_service::<TestService, u32>(|service| service.value);
        assert_eq!(result, Some(101));
    }
}