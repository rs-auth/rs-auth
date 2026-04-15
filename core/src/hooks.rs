use async_trait::async_trait;
use tracing::warn;

use crate::error::AuthError;
use crate::events::AuthEvent;

#[async_trait]
pub trait AuthHook: Send + Sync {
    async fn on_event(&self, event: &AuthEvent) -> Result<(), AuthError> {
        let _ = event;
        Ok(())
    }
}

pub struct EventEmitter {
    hooks: Vec<Box<dyn AuthHook>>,
}

impl EventEmitter {
    pub fn new() -> Self {
        Self { hooks: vec![] }
    }

    pub fn add_hook(&mut self, hook: Box<dyn AuthHook>) {
        self.hooks.push(hook);
    }

    pub async fn emit(&self, event: AuthEvent) {
        for hook in &self.hooks {
            if let Err(e) = hook.on_event(&event).await {
                warn!(error = %e, "auth hook error");
            }
        }
    }
}

impl Default for EventEmitter {
    fn default() -> Self {
        Self::new()
    }
}
