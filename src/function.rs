use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use crate::runtime::{WasmInstance, WasmRuntime};

const REGISTRY_FILE: &str = "functions.json";

#[derive(Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub id: String,
    pub name: String,
    pub wasm_path: PathBuf,
    pub trigger_subjects: Vec<String>,
}

pub struct FunctionRegistry {
    runtime: WasmRuntime,
    functions: RwLock<HashMap<String, FunctionInfo>>,
    subject_map: RwLock<HashMap<String, Vec<String>>>, // subject -> function_ids
}

impl FunctionRegistry {
    pub fn new(runtime: WasmRuntime) -> Self {
        let registry = Self {
            runtime,
            functions: RwLock::new(HashMap::new()),
            subject_map: RwLock::new(HashMap::new()),
        };

        registry.load_functions().unwrap_or_else(|e| {
            eprintln!("Failed to load registry: {}", e);
        });

        registry
    }

    fn load_functions(&self) -> Result<()> {
        if !Path::new(REGISTRY_FILE).exists() {
            return Ok(());
        }

        let data = fs::read_to_string(REGISTRY_FILE)?;
        let info_list: Vec<FunctionInfo> = serde_json::from_str(&data)?;

        let mut functions = self.functions.write().unwrap();
        let mut subject_map = self.subject_map.write().unwrap();

        for info in info_list {
            functions.insert(info.id.clone(), info.clone());

            for subject in &info.trigger_subjects {
                subject_map
                    .entry(subject.clone())
                    .or_default()
                    .push(info.id.clone());
            }
        }

        Ok(())
    }

    fn save_functions(&self) -> Result<()> {
        let functions = self.functions.read().unwrap();
        let info_list: Vec<FunctionInfo> = functions.values().cloned().collect();
        let data = serde_json::to_string_pretty(&info_list)?;
        fs::write(REGISTRY_FILE, data)?;
        Ok(())
    }

    pub fn register_function(
        &self,
        name: &str,
        wasm_path: &Path,
        triggers: Vec<String>,
    ) -> Result<String> {
        let id = generate_id();

        let function = FunctionInfo {
            id: id.clone(),
            name: name.to_string(),
            wasm_path: wasm_path.to_path_buf(),
            trigger_subjects: triggers.clone(),
        };

        self.functions.write().unwrap().insert(id.clone(), function);

        for subject in triggers {
            let mut subject_map = self.subject_map.write().unwrap();
            subject_map.entry(subject).or_default().push(id.clone());
        }

        self.save_functions()?;

        Ok(id)
    }

    pub fn get_function_by_id(&self, id: &str) -> Option<FunctionInfo> {
        self.functions.read().unwrap().get(id).cloned()
    }

    pub fn get_functions_by_subject(&self, subject: &str) -> Vec<FunctionInfo> {
        let subject_map = self.subject_map.read().unwrap();
        let functions = self.functions.read().unwrap();

        match subject_map.get(subject) {
            Some(ids) => ids
                .iter()
                .filter_map(|id| functions.get(id).cloned())
                .collect(),
            None => Vec::new(),
        }
    }

    pub fn list_functions(&self) -> Vec<FunctionInfo> {
        self.functions.read().unwrap().values().cloned().collect()
    }

    pub fn invoke_function(
        &self,
        function_id: &str,
        trigger: &str,
        payload: Vec<u8>,
    ) -> Result<Option<Vec<u8>>> {
        let function = self
            .get_function_by_id(function_id)
            .ok_or_else(|| anyhow::anyhow!("Function not found: {}", function_id))?;

        let wasm_bytes = fs::read(&function.wasm_path)?;
        let module = self.runtime.compile_module(&wasm_bytes)?;

        let mut instance = WasmInstance::new(&self.runtime, module, trigger, payload)?;
        instance.invoke()
    }
}

fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("fn_{}", now)
}
