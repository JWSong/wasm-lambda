use anyhow::{Result, anyhow};
use std::sync::Arc;
use wasmtime::{Engine, Instance, Module, Store};

pub struct WasmRuntime {
    engine: Engine,
}

impl WasmRuntime {
    pub fn new() -> Self {
        let engine = Engine::default();
        Self { engine }
    }

    pub fn compile_module(&self, wasm_bytes: &[u8]) -> Result<Arc<Module>> {
        let module = Module::new(&self.engine, wasm_bytes)?;
        Ok(Arc::new(module))
    }
}

pub struct WasmInstance {
    #[allow(dead_code)]
    module: Arc<Module>,
    store: Store<HostState>,
    instance: Instance,
}

#[derive(Default)]
pub struct HostState {
    pub input: Vec<u8>,
    pub output: Option<Vec<u8>>,
    pub trigger_subject: String,
}

impl WasmInstance {
    pub fn new(
        runtime: &WasmRuntime,
        module: Arc<Module>,
        trigger_subject: &str,
        input: Vec<u8>,
    ) -> Result<Self> {
        let mut store = Store::new(
            &runtime.engine,
            HostState {
                input,
                output: None,
                trigger_subject: trigger_subject.to_string(),
            },
        );

        let get_input = wasmtime::Func::wrap(
            &mut store,
            |mut caller: wasmtime::Caller<'_, HostState>, ptr: i32, len: i32| -> i32 {
                let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
                    Some(mem) => mem,
                    None => return -1,
                };

                let data = caller.data().input.clone();
                let len = std::cmp::min(len as usize, data.len());

                let mem_slice = memory.data_mut(&mut caller);
                let dst = &mut mem_slice[ptr as usize..(ptr as usize + len)];
                dst.copy_from_slice(&data[..len]);

                len as i32
            },
        );

        let get_trigger = wasmtime::Func::wrap(
            &mut store,
            |mut caller: wasmtime::Caller<'_, HostState>, ptr: i32, len: i32| -> i32 {
                let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
                    Some(mem) => mem,
                    None => return -1,
                };

                let subject = caller.data().trigger_subject.as_bytes().to_vec();
                let len = std::cmp::min(len as usize, subject.len());

                let mem_slice = memory.data_mut(&mut caller);
                let dst = &mut mem_slice[ptr as usize..(ptr as usize + len)];
                dst.copy_from_slice(&subject[..len]);

                subject.len() as i32
            },
        );

        let set_output = wasmtime::Func::wrap(
            &mut store,
            |mut caller: wasmtime::Caller<'_, HostState>, ptr: i32, len: i32| {
                let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
                    Some(mem) => mem,
                    None => return,
                };

                let mut output = vec![0u8; len as usize];

                let mem_slice = memory.data(&caller);
                output.copy_from_slice(&mem_slice[ptr as usize..(ptr as usize + len as usize)]);

                caller.data_mut().output = Some(output);
            },
        );

        let instance = Instance::new(
            &mut store,
            &module,
            &[get_input.into(), get_trigger.into(), set_output.into()],
        )?;

        Ok(Self {
            module,
            store,
            instance,
        })
    }

    pub fn invoke(&mut self) -> Result<Option<Vec<u8>>> {
        let handle = self
            .instance
            .get_func(&mut self.store, "handle")
            .ok_or_else(|| anyhow!("WASM module does not have 'handle' function"))?;

        handle.call(&mut self.store, &[], &mut [])?;

        Ok(self.store.data().output.clone())
    }
}
