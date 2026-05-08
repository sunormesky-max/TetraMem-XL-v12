// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use crate::universe::error::AppError;
use crate::universe::plugins::manifest::{
    PluginExecutionRequest, PluginExecutionResult, PluginPermissions,
};
use wasmi::{
    Engine, Linker, Memory, MemoryType, Module, ResourceLimiter, Store, StoreLimitsBuilder,
};

pub struct WasmSandbox {
    engine: Engine,
}

impl Default for WasmSandbox {
    fn default() -> Self {
        Self::new()
    }
}

impl WasmSandbox {
    pub fn new() -> Self {
        let engine = Engine::default();
        Self { engine }
    }

    // TODO: wasmi does not support fuel-based execution interruption (unlike wasmtime).
    // Callers should use `tokio::task::spawn_blocking` with a timeout to guard against
    // infinite loops in untrusted WASM modules. A future migration to wasmtime or a
    // separate watchdog thread could provide deterministic interruption.

    pub fn validate(&self, wasm_bytes: &[u8]) -> Result<(), AppError> {
        Module::new(&self.engine, wasm_bytes)
            .map(|_| ())
            .map_err(|e| AppError::BadRequest(format!("invalid WASM module: {}", e)))
    }

    pub fn execute(
        &self,
        wasm_bytes: &[u8],
        request: &PluginExecutionRequest,
        permissions: &PluginPermissions,
        energy_budget: u64,
    ) -> PluginExecutionResult {
        let start = std::time::Instant::now();

        if request.input.len() > MAX_INPUT_BYTES {
            return PluginExecutionResult {
                output: Vec::new(),
                energy_consumed: 0,
                execution_time_us: start.elapsed().as_micros() as u64,
                success: false,
                error: Some(format!(
                    "input too large: {} bytes (max {} bytes)",
                    request.input.len(),
                    MAX_INPUT_BYTES
                )),
            };
        }

        let module = match Module::new(&self.engine, wasm_bytes) {
            Ok(m) => m,
            Err(e) => {
                return PluginExecutionResult {
                    output: Vec::new(),
                    energy_consumed: 0,
                    execution_time_us: start.elapsed().as_micros() as u64,
                    success: false,
                    error: Some(format!("module load: {}", e)),
                }
            }
        };

        let host = HostState::new(energy_budget, permissions.clone(), request.input.clone());

        let mut store: Store<HostState> = Store::new(&self.engine, host);
        store.limiter(|state| &mut state.limits as &mut dyn ResourceLimiter);

        let mut linker: Linker<HostState> = Linker::new(&self.engine);

        if let Err(e) = Self::define_host_functions(&mut linker) {
            return PluginExecutionResult {
                output: Vec::new(),
                energy_consumed: 0,
                execution_time_us: start.elapsed().as_micros() as u64,
                success: false,
                error: Some(format!("host function setup: {}", e)),
            };
        }

        let memory_ty = match MemoryType::new(1, Some(256)) {
            Ok(ty) => ty,
            Err(e) => {
                return PluginExecutionResult {
                    output: Vec::new(),
                    energy_consumed: 0,
                    execution_time_us: start.elapsed().as_micros() as u64,
                    success: false,
                    error: Some(format!("memory type: {}", e)),
                }
            }
        };

        let memory = match Memory::new(&mut store, memory_ty) {
            Ok(m) => m,
            Err(e) => {
                return PluginExecutionResult {
                    output: Vec::new(),
                    energy_consumed: 0,
                    execution_time_us: start.elapsed().as_micros() as u64,
                    success: false,
                    error: Some(format!("memory alloc: {}", e)),
                }
            }
        };

        if let Err(e) = linker.define("env", "memory", memory) {
            return PluginExecutionResult {
                output: Vec::new(),
                energy_consumed: 0,
                execution_time_us: start.elapsed().as_micros() as u64,
                success: false,
                error: Some(format!("memory export: {}", e)),
            };
        }

        let instance_pre = match linker.instantiate(&mut store, &module) {
            Ok(ip) => ip,
            Err(e) => {
                return PluginExecutionResult {
                    output: Vec::new(),
                    energy_consumed: 0,
                    execution_time_us: start.elapsed().as_micros() as u64,
                    success: false,
                    error: Some(format!("instantiation: {}", e)),
                }
            }
        };

        let instance = match instance_pre.start(&mut store) {
            Ok(inst) => inst,
            Err(e) => {
                return PluginExecutionResult {
                    output: Vec::new(),
                    energy_consumed: 0,
                    execution_time_us: start.elapsed().as_micros() as u64,
                    success: false,
                    error: Some(format!("start: {}", e)),
                }
            }
        };

        if !request.input.is_empty() {
            let offset = 0usize;
            let data = &request.input;
            if offset + data.len() > memory.data(&store).len() {
                return PluginExecutionResult {
                    output: Vec::new(),
                    energy_consumed: 0,
                    execution_time_us: start.elapsed().as_micros() as u64,
                    success: false,
                    error: Some(format!(
                        "input too large for WASM memory: {} bytes",
                        data.len()
                    )),
                };
            }
            memory.data_mut(&mut store)[offset..offset + data.len()].copy_from_slice(data);
        }

        let func_name = &request.function;
        let target_func = instance
            .get_func(&store, func_name)
            .or_else(|| instance.get_func(&store, "_start"))
            .or_else(|| instance.get_func(&store, "main"));

        let Some(func) = target_func else {
            return PluginExecutionResult {
                output: Vec::new(),
                energy_consumed: store.data().energy_consumed,
                execution_time_us: start.elapsed().as_micros() as u64,
                success: false,
                error: Some(format!("function '{}' not found in module", func_name)),
            };
        };

        let mut results = [];
        match func.call(&mut store, &[], &mut results) {
            Ok(_) => {
                let state = store.data();
                PluginExecutionResult {
                    output: state.output_data.clone(),
                    energy_consumed: state.energy_consumed,
                    execution_time_us: start.elapsed().as_micros() as u64,
                    success: true,
                    error: None,
                }
            }
            Err(e) => {
                let state = store.data();
                PluginExecutionResult {
                    output: Vec::new(),
                    energy_consumed: state.energy_consumed,
                    execution_time_us: start.elapsed().as_micros() as u64,
                    success: false,
                    error: Some(format!("execution: {}", e)),
                }
            }
        }
    }

    fn define_host_functions(
        linker: &mut Linker<HostState>,
    ) -> Result<(), wasmi::errors::LinkerError> {
        linker.func_wrap(
            "env",
            "tetramem_energy_remaining",
            |caller: wasmi::Caller<'_, HostState>| -> u64 { caller.data().energy_remaining },
        )?;

        linker.func_wrap(
            "env",
            "tetramem_consume_energy",
            |mut caller: wasmi::Caller<'_, HostState>, amount: u64| -> i32 {
                let state = caller.data_mut();
                if amount > state.energy_remaining {
                    state.energy_consumed += state.energy_remaining;
                    state.energy_remaining = 0;
                    0
                } else {
                    state.energy_remaining -= amount;
                    state.energy_consumed += amount;
                    1
                }
            },
        )?;

        linker.func_wrap(
            "env",
            "tetramem_input_len",
            |caller: wasmi::Caller<'_, HostState>| -> u32 { caller.data().input_data.len() as u32 },
        )?;

        linker.func_wrap(
            "env",
            "tetramem_can_memory_read",
            |caller: wasmi::Caller<'_, HostState>| -> i32 {
                if caller.data().permissions.memory_read {
                    1
                } else {
                    0
                }
            },
        )?;

        linker.func_wrap(
            "env",
            "tetramem_can_memory_write",
            |caller: wasmi::Caller<'_, HostState>| -> i32 {
                if caller.data().permissions.memory_write {
                    1
                } else {
                    0
                }
            },
        )?;

        linker.func_wrap(
            "env",
            "tetramem_can_hebbian_read",
            |caller: wasmi::Caller<'_, HostState>| -> i32 {
                if caller.data().permissions.hebbian_read {
                    1
                } else {
                    0
                }
            },
        )?;

        linker.func_wrap(
            "env",
            "tetramem_can_pulse_fire",
            |caller: wasmi::Caller<'_, HostState>| -> i32 {
                if caller.data().permissions.pulse_fire {
                    1
                } else {
                    0
                }
            },
        )?;

        linker.func_wrap(
            "env",
            "tetramem_log",
            |mut caller: wasmi::Caller<'_, HostState>, len: u32| {
                let msg = format!("[plugin] log({} bytes)", len);
                caller.data_mut().log_messages.push(msg);
            },
        )?;

        linker.func_wrap(
            "env",
            "tetramem_output_append",
            |mut caller: wasmi::Caller<'_, HostState>, value: u32| {
                let state = caller.data_mut();
                if state.output_data.len() + 4 <= MAX_OUTPUT_BYTES {
                    state.output_data.extend_from_slice(&value.to_le_bytes());
                }
            },
        )?;

        Ok(())
    }
}

const MAX_OUTPUT_BYTES: usize = 1024 * 1024;
const MAX_INPUT_BYTES: usize = 1024 * 1024;

pub struct HostState {
    pub limits: wasmi::StoreLimits,
    pub energy_remaining: u64,
    pub energy_consumed: u64,
    pub permissions: PluginPermissions,
    pub input_data: Vec<u8>,
    pub output_data: Vec<u8>,
    pub log_messages: Vec<String>,
}

impl HostState {
    fn new(energy_budget: u64, permissions: PluginPermissions, input_data: Vec<u8>) -> Self {
        Self {
            limits: StoreLimitsBuilder::new()
                .memory_size(256 * 64 * 1024)
                .build(),
            energy_remaining: energy_budget,
            energy_consumed: 0,
            permissions,
            input_data,
            output_data: Vec::with_capacity(4096),
            log_messages: Vec::new(),
        }
    }
}
