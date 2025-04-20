use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

use crate::function::FunctionRegistry;

#[async_trait]
pub trait Trigger: Send + Sync {
    async fn start(&self) -> Result<()>;
    async fn stop(&self) -> Result<()>;
}

pub struct SimpleTrigger {
    registry: Arc<FunctionRegistry>,
}

impl SimpleTrigger {
    pub fn new(registry: Arc<FunctionRegistry>) -> Self {
        Self { registry }
    }

    pub async fn trigger(&self, subject: &str, payload: Vec<u8>) -> Result<Vec<Option<Vec<u8>>>> {
        let functions = self.registry.get_functions_by_subject(subject);

        let mut results = Vec::new();
        for function in functions {
            let result = self
                .registry
                .invoke_function(&function.id, subject, payload.clone())?;
            results.push(result);
        }

        Ok(results)
    }
}

#[async_trait]
impl Trigger for SimpleTrigger {
    async fn start(&self) -> Result<()> {
        // TODO: start event listening
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        // TODO: stop event listening
        Ok(())
    }
}
