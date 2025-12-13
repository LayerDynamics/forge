//! host:wasm extension - WebAssembly support for Forge apps
//!
//! Provides WASM module loading, instantiation, function calls, memory access,
//! and WASI support with capability-based security.

use deno_core::{op2, Extension, OpState};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;
use wasmtime::{Engine, Extern, Func, Linker, Memory, Module, Store, Val, ValType};

// ============================================================================
// Error Types with Structured Codes
// ============================================================================

/// Error codes for WASM operations (for machine-readable errors)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum WasmErrorCode {
    /// Failed to compile WASM module
    CompileError = 5000,
    /// Failed to instantiate module
    InstantiateError = 5001,
    /// Function call failed
    CallError = 5002,
    /// Export not found in module
    ExportNotFound = 5003,
    /// Invalid module handle
    InvalidModuleHandle = 5004,
    /// Invalid instance handle
    InvalidInstanceHandle = 5005,
    /// Memory access error
    MemoryError = 5006,
    /// Type mismatch in function call
    TypeError = 5007,
    /// IO error (file loading)
    IoError = 5008,
    /// Permission denied by capability system
    PermissionDenied = 5009,
    /// WASI configuration error
    WasiError = 5010,
    /// Fuel exhaustion (execution limit)
    FuelExhausted = 5011,
}

/// Custom error type for WASM operations
#[derive(Debug, thiserror::Error, deno_error::JsError)]
pub enum WasmError {
    #[error("[{code}] Compile error: {message}")]
    #[class(generic)]
    CompileError { code: u32, message: String },

    #[error("[{code}] Instantiate error: {message}")]
    #[class(generic)]
    InstantiateError { code: u32, message: String },

    #[error("[{code}] Call error: {message}")]
    #[class(generic)]
    CallError { code: u32, message: String },

    #[error("[{code}] Export not found: {message}")]
    #[class(generic)]
    ExportNotFound { code: u32, message: String },

    #[error("[{code}] Invalid module handle: {message}")]
    #[class(generic)]
    InvalidModuleHandle { code: u32, message: String },

    #[error("[{code}] Invalid instance handle: {message}")]
    #[class(generic)]
    InvalidInstanceHandle { code: u32, message: String },

    #[error("[{code}] Memory error: {message}")]
    #[class(generic)]
    MemoryError { code: u32, message: String },

    #[error("[{code}] Type error: {message}")]
    #[class(generic)]
    TypeError { code: u32, message: String },

    #[error("[{code}] IO error: {message}")]
    #[class(generic)]
    IoError { code: u32, message: String },

    #[error("[{code}] Permission denied: {message}")]
    #[class(generic)]
    PermissionDenied { code: u32, message: String },

    #[error("[{code}] WASI error: {message}")]
    #[class(generic)]
    WasiError { code: u32, message: String },

    #[error("[{code}] Fuel exhausted: {message}")]
    #[class(generic)]
    FuelExhausted { code: u32, message: String },
}

impl WasmError {
    pub fn compile_error(message: impl Into<String>) -> Self {
        Self::CompileError {
            code: WasmErrorCode::CompileError as u32,
            message: message.into(),
        }
    }

    pub fn instantiate_error(message: impl Into<String>) -> Self {
        Self::InstantiateError {
            code: WasmErrorCode::InstantiateError as u32,
            message: message.into(),
        }
    }

    pub fn call_error(message: impl Into<String>) -> Self {
        Self::CallError {
            code: WasmErrorCode::CallError as u32,
            message: message.into(),
        }
    }

    pub fn export_not_found(message: impl Into<String>) -> Self {
        Self::ExportNotFound {
            code: WasmErrorCode::ExportNotFound as u32,
            message: message.into(),
        }
    }

    pub fn invalid_module_handle(message: impl Into<String>) -> Self {
        Self::InvalidModuleHandle {
            code: WasmErrorCode::InvalidModuleHandle as u32,
            message: message.into(),
        }
    }

    pub fn invalid_instance_handle(message: impl Into<String>) -> Self {
        Self::InvalidInstanceHandle {
            code: WasmErrorCode::InvalidInstanceHandle as u32,
            message: message.into(),
        }
    }

    pub fn memory_error(message: impl Into<String>) -> Self {
        Self::MemoryError {
            code: WasmErrorCode::MemoryError as u32,
            message: message.into(),
        }
    }

    pub fn type_error(message: impl Into<String>) -> Self {
        Self::TypeError {
            code: WasmErrorCode::TypeError as u32,
            message: message.into(),
        }
    }

    pub fn io_error(message: impl Into<String>) -> Self {
        Self::IoError {
            code: WasmErrorCode::IoError as u32,
            message: message.into(),
        }
    }

    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self::PermissionDenied {
            code: WasmErrorCode::PermissionDenied as u32,
            message: message.into(),
        }
    }

    pub fn wasi_error(message: impl Into<String>) -> Self {
        Self::WasiError {
            code: WasmErrorCode::WasiError as u32,
            message: message.into(),
        }
    }

    pub fn fuel_exhausted(message: impl Into<String>) -> Self {
        Self::FuelExhausted {
            code: WasmErrorCode::FuelExhausted as u32,
            message: message.into(),
        }
    }
}

impl From<std::io::Error> for WasmError {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::NotFound => Self::io_error(format!("File not found: {}", e)),
            std::io::ErrorKind::PermissionDenied => {
                Self::permission_denied(format!("Permission denied: {}", e))
            }
            _ => Self::io_error(e.to_string()),
        }
    }
}

impl From<wasmtime::Error> for WasmError {
    fn from(e: wasmtime::Error) -> Self {
        let msg = e.to_string();
        if msg.contains("fuel") {
            Self::fuel_exhausted(msg)
        } else if msg.contains("type") || msg.contains("Type") {
            Self::type_error(msg)
        } else {
            Self::call_error(msg)
        }
    }
}

// ============================================================================
// Types
// ============================================================================

/// WASM value type for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum WasmValue {
    #[serde(rename = "i32")]
    I32(i32),
    #[serde(rename = "i64")]
    I64(i64),
    #[serde(rename = "f32")]
    F32(f32),
    #[serde(rename = "f64")]
    F64(f64),
}

impl WasmValue {
    /// Convert to wasmtime Val
    fn to_wasmtime(&self) -> Val {
        match self {
            WasmValue::I32(v) => Val::I32(*v),
            WasmValue::I64(v) => Val::I64(*v),
            WasmValue::F32(v) => Val::F32(v.to_bits()),
            WasmValue::F64(v) => Val::F64(v.to_bits()),
        }
    }

    /// Convert from wasmtime Val
    fn from_wasmtime(val: &Val) -> Option<Self> {
        match val {
            Val::I32(v) => Some(WasmValue::I32(*v)),
            Val::I64(v) => Some(WasmValue::I64(*v)),
            Val::F32(v) => Some(WasmValue::F32(f32::from_bits(*v))),
            Val::F64(v) => Some(WasmValue::F64(f64::from_bits(*v))),
            _ => None, // Externref, Funcref not supported in MVP
        }
    }

    /// Get the wasmtime ValType for this value
    fn val_type(&self) -> ValType {
        match self {
            WasmValue::I32(_) => ValType::I32,
            WasmValue::I64(_) => ValType::I64,
            WasmValue::F32(_) => ValType::F32,
            WasmValue::F64(_) => ValType::F64,
        }
    }
}

/// WASI configuration for instance creation
#[derive(Debug, Clone, Deserialize, Default)]
pub struct WasiConfig {
    /// Preopened directories: mapping of guest path -> host path
    pub preopens: Option<HashMap<String, String>>,
    /// Environment variables
    pub env: Option<HashMap<String, String>>,
    /// Command-line arguments
    pub args: Option<Vec<String>>,
    /// Inherit stdin from host (default: false)
    pub inherit_stdin: Option<bool>,
    /// Inherit stdout from host (default: false)
    pub inherit_stdout: Option<bool>,
    /// Inherit stderr from host (default: false)
    pub inherit_stderr: Option<bool>,
}

/// Export information
#[derive(Debug, Clone, Serialize)]
pub struct ExportInfo {
    pub name: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub results: Option<Vec<String>>,
}

/// Stored compiled module
pub struct WasmModule {
    pub module: Module,
    pub name: Option<String>,
}

/// Store context for WASM instances (WASI support planned for future milestone)
#[derive(Default)]
pub struct WasmStoreData {
    // WASI support will be added in a future milestone
    _placeholder: (),
}

/// Stored instance with its store
pub struct WasmInstance {
    pub store: Store<WasmStoreData>,
    pub instance: wasmtime::Instance,
    pub module_id: String,
}

/// Main state for WASM extension
pub struct WasmState {
    pub engine: Engine,
    pub modules: HashMap<String, WasmModule>,
    pub instances: HashMap<String, Arc<Mutex<WasmInstance>>>,
    pub next_module_id: u64,
    pub next_instance_id: u64,
    pub max_instances: usize,
}

impl WasmState {
    pub fn new(max_instances: usize) -> Self {
        let engine = Engine::default();
        Self {
            engine,
            modules: HashMap::new(),
            instances: HashMap::new(),
            next_module_id: 1,
            next_instance_id: 1,
            max_instances,
        }
    }

    fn generate_module_id(&mut self) -> String {
        let id = format!("mod_{}", self.next_module_id);
        self.next_module_id += 1;
        id
    }

    fn generate_instance_id(&mut self) -> String {
        let id = format!("inst_{}", self.next_instance_id);
        self.next_instance_id += 1;
        id
    }
}

impl Default for WasmState {
    fn default() -> Self {
        Self::new(10) // Default max 10 concurrent instances
    }
}

// ============================================================================
// Capability Checker
// ============================================================================

/// Capability checker trait for WASM operations
pub trait WasmCapabilityChecker: Send + Sync {
    /// Check if loading WASM from this path is allowed
    fn check_load(&self, path: &str) -> Result<(), String>;
    /// Check if this preopened directory is allowed
    fn check_preopen(&self, host_path: &str) -> Result<(), String>;
}

/// Default permissive checker (for dev mode)
pub struct PermissiveWasmChecker;

impl WasmCapabilityChecker for PermissiveWasmChecker {
    fn check_load(&self, _path: &str) -> Result<(), String> {
        Ok(())
    }
    fn check_preopen(&self, _host_path: &str) -> Result<(), String> {
        Ok(())
    }
}

/// Wrapper to store the capability checker in OpState
pub struct WasmCapabilities {
    pub checker: Arc<dyn WasmCapabilityChecker>,
}

impl Default for WasmCapabilities {
    fn default() -> Self {
        Self {
            checker: Arc::new(PermissiveWasmChecker),
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Helper to check wasm load capability
fn check_wasm_load(state: &OpState, path: &str) -> Result<(), WasmError> {
    if let Some(caps) = state.try_borrow::<WasmCapabilities>() {
        caps.checker
            .check_load(path)
            .map_err(WasmError::permission_denied)
    } else {
        // No capabilities configured, allow (dev mode)
        Ok(())
    }
}

/// Helper to check wasm preopen capability
fn check_wasm_preopen(state: &OpState, host_path: &str) -> Result<(), WasmError> {
    if let Some(caps) = state.try_borrow::<WasmCapabilities>() {
        caps.checker
            .check_preopen(host_path)
            .map_err(WasmError::permission_denied)
    } else {
        // No capabilities configured, allow (dev mode)
        Ok(())
    }
}

/// Convert ValType to string for export info
fn val_type_to_string(ty: &ValType) -> String {
    match ty {
        ValType::I32 => "i32".to_string(),
        ValType::I64 => "i64".to_string(),
        ValType::F32 => "f32".to_string(),
        ValType::F64 => "f64".to_string(),
        ValType::V128 => "v128".to_string(),
        ValType::Ref(r) => format!("ref:{:?}", r),
    }
}

// ============================================================================
// Module Operations
// ============================================================================

/// Compile WASM bytes to a module
#[op2(async)]
#[string]
async fn op_wasm_compile(
    state: Rc<RefCell<OpState>>,
    #[serde] bytes: Vec<u8>,
) -> Result<String, WasmError> {
    debug!(len = bytes.len(), "wasm.compile");

    let (engine, module_id) = {
        let mut s = state.borrow_mut();
        let ws = s
            .try_borrow_mut::<WasmState>()
            .ok_or_else(|| WasmError::compile_error("WASM state not initialized"))?;
        let engine = ws.engine.clone();
        let module_id = ws.generate_module_id();
        (engine, module_id)
    };

    // Compile outside of borrow
    let module =
        Module::new(&engine, &bytes).map_err(|e| WasmError::compile_error(e.to_string()))?;

    // Store the module
    {
        let mut s = state.borrow_mut();
        let ws = s
            .try_borrow_mut::<WasmState>()
            .ok_or_else(|| WasmError::compile_error("WASM state not initialized"))?;
        ws.modules.insert(
            module_id.clone(),
            WasmModule {
                module,
                name: None,
            },
        );
    }

    debug!(module_id = %module_id, "wasm.compile complete");
    Ok(module_id)
}

/// Compile WASM from file path
#[op2(async)]
#[string]
async fn op_wasm_compile_file(
    state: Rc<RefCell<OpState>>,
    #[string] path: String,
) -> Result<String, WasmError> {
    debug!(path = %path, "wasm.compile_file");

    // Check capabilities
    {
        let s = state.borrow();
        check_wasm_load(&s, &path)?;
    }

    // Read file
    let bytes = tokio::fs::read(&path).await?;

    let (engine, module_id) = {
        let mut s = state.borrow_mut();
        let ws = s
            .try_borrow_mut::<WasmState>()
            .ok_or_else(|| WasmError::compile_error("WASM state not initialized"))?;
        let engine = ws.engine.clone();
        let module_id = ws.generate_module_id();
        (engine, module_id)
    };

    // Compile outside of borrow
    let module =
        Module::new(&engine, &bytes).map_err(|e| WasmError::compile_error(e.to_string()))?;

    // Store the module
    {
        let mut s = state.borrow_mut();
        let ws = s
            .try_borrow_mut::<WasmState>()
            .ok_or_else(|| WasmError::compile_error("WASM state not initialized"))?;
        ws.modules.insert(
            module_id.clone(),
            WasmModule {
                module,
                name: Some(path.clone()),
            },
        );
    }

    debug!(module_id = %module_id, path = %path, "wasm.compile_file complete");
    Ok(module_id)
}

/// Drop a compiled module
#[op2(async)]
async fn op_wasm_drop_module(
    state: Rc<RefCell<OpState>>,
    #[string] module_id: String,
) -> Result<(), WasmError> {
    debug!(module_id = %module_id, "wasm.drop_module");

    let mut s = state.borrow_mut();
    let ws = s
        .try_borrow_mut::<WasmState>()
        .ok_or_else(|| WasmError::invalid_module_handle("WASM state not initialized"))?;

    if ws.modules.remove(&module_id).is_none() {
        return Err(WasmError::invalid_module_handle(format!(
            "Module '{}' not found",
            module_id
        )));
    }

    debug!(module_id = %module_id, "wasm.drop_module complete");
    Ok(())
}

// ============================================================================
// Instance Operations
// ============================================================================

/// Instantiate a module, optionally with WASI
#[op2(async)]
#[string]
async fn op_wasm_instantiate(
    state: Rc<RefCell<OpState>>,
    #[string] module_id: String,
    #[serde] wasi_config: Option<WasiConfig>,
) -> Result<String, WasmError> {
    debug!(module_id = %module_id, has_wasi = wasi_config.is_some(), "wasm.instantiate");

    // Check preopen paths if WASI config provided
    if let Some(ref config) = wasi_config {
        if let Some(ref preopens) = config.preopens {
            let s = state.borrow();
            for host_path in preopens.values() {
                check_wasm_preopen(&s, host_path)?;
            }
        }
    }

    // Get engine and module
    let (engine, module, instance_id) = {
        let mut s = state.borrow_mut();
        let ws = s
            .try_borrow_mut::<WasmState>()
            .ok_or_else(|| WasmError::instantiate_error("WASM state not initialized"))?;

        // Check instance limit
        if ws.instances.len() >= ws.max_instances {
            return Err(WasmError::instantiate_error(format!(
                "Maximum instance limit ({}) reached",
                ws.max_instances
            )));
        }

        let wasm_module = ws.modules.get(&module_id).ok_or_else(|| {
            WasmError::invalid_module_handle(format!("Module '{}' not found", module_id))
        })?;

        let engine = ws.engine.clone();
        let module = wasm_module.module.clone();
        let instance_id = ws.generate_instance_id();
        (engine, module, instance_id)
    };

    // WASI support is planned for a future milestone
    // For now, log if WASI config was provided but ignore it
    if wasi_config.is_some() {
        debug!("WASI config provided but WASI support is not yet implemented - instantiating without WASI");
    }

    // Create store (without WASI for now)
    let mut store = Store::new(&engine, WasmStoreData::default());

    // Create linker
    let linker: Linker<WasmStoreData> = Linker::new(&engine);

    // Instantiate the module
    let instance = linker
        .instantiate(&mut store, &module)
        .map_err(|e| WasmError::instantiate_error(e.to_string()))?;

    // Store the instance
    {
        let mut s = state.borrow_mut();
        let ws = s
            .try_borrow_mut::<WasmState>()
            .ok_or_else(|| WasmError::instantiate_error("WASM state not initialized"))?;
        ws.instances.insert(
            instance_id.clone(),
            Arc::new(Mutex::new(WasmInstance {
                store,
                instance,
                module_id: module_id.clone(),
            })),
        );
    }

    debug!(instance_id = %instance_id, module_id = %module_id, "wasm.instantiate complete");
    Ok(instance_id)
}

/// Drop an instance
#[op2(async)]
async fn op_wasm_drop_instance(
    state: Rc<RefCell<OpState>>,
    #[string] instance_id: String,
) -> Result<(), WasmError> {
    debug!(instance_id = %instance_id, "wasm.drop_instance");

    let mut s = state.borrow_mut();
    let ws = s
        .try_borrow_mut::<WasmState>()
        .ok_or_else(|| WasmError::invalid_instance_handle("WASM state not initialized"))?;

    if ws.instances.remove(&instance_id).is_none() {
        return Err(WasmError::invalid_instance_handle(format!(
            "Instance '{}' not found",
            instance_id
        )));
    }

    debug!(instance_id = %instance_id, "wasm.drop_instance complete");
    Ok(())
}

/// Get list of exports from an instance
#[op2(async)]
#[serde]
async fn op_wasm_get_exports(
    state: Rc<RefCell<OpState>>,
    #[string] instance_id: String,
) -> Result<Vec<ExportInfo>, WasmError> {
    debug!(instance_id = %instance_id, "wasm.get_exports");

    let instance_arc = {
        let s = state.borrow();
        let ws = s
            .try_borrow::<WasmState>()
            .ok_or_else(|| WasmError::invalid_instance_handle("WASM state not initialized"))?;
        ws.instances
            .get(&instance_id)
            .cloned()
            .ok_or_else(|| {
                WasmError::invalid_instance_handle(format!("Instance '{}' not found", instance_id))
            })?
    };

    let mut inst = instance_arc.lock().await;
    let mut exports = Vec::new();

    // Destructure to get separate mutable borrows
    let WasmInstance { store, instance, .. } = &mut *inst;

    // Collect export names first to avoid borrow issues
    let export_names: Vec<String> = instance.exports(&mut *store)
        .map(|e| e.name().to_string())
        .collect();

    for name in export_names {
        let ext = instance.get_export(&mut *store, &name);
        let (kind, params, results) = match ext {
            Some(Extern::Func(f)) => {
                let ty = f.ty(&*store);
                let params: Vec<String> = ty.params().map(|p| val_type_to_string(&p)).collect();
                let results: Vec<String> = ty.results().map(|r| val_type_to_string(&r)).collect();
                ("function".to_string(), Some(params), Some(results))
            }
            Some(Extern::Memory(_)) => ("memory".to_string(), None, None),
            Some(Extern::Table(_)) => ("table".to_string(), None, None),
            Some(Extern::Global(_)) => ("global".to_string(), None, None),
            Some(Extern::SharedMemory(_)) => ("shared_memory".to_string(), None, None),
            None => continue,
        };
        exports.push(ExportInfo {
            name,
            kind,
            params,
            results,
        });
    }

    debug!(instance_id = %instance_id, count = exports.len(), "wasm.get_exports complete");
    Ok(exports)
}

// ============================================================================
// Function Call Operation
// ============================================================================

/// Call an exported function
#[op2(async)]
#[serde]
async fn op_wasm_call(
    state: Rc<RefCell<OpState>>,
    #[string] instance_id: String,
    #[string] func_name: String,
    #[serde] args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, WasmError> {
    debug!(instance_id = %instance_id, func_name = %func_name, args_count = args.len(), "wasm.call");

    let instance_arc = {
        let s = state.borrow();
        let ws = s
            .try_borrow::<WasmState>()
            .ok_or_else(|| WasmError::call_error("WASM state not initialized"))?;
        ws.instances
            .get(&instance_id)
            .cloned()
            .ok_or_else(|| {
                WasmError::invalid_instance_handle(format!("Instance '{}' not found", instance_id))
            })?
    };

    let mut inst = instance_arc.lock().await;

    // Destructure to get separate mutable borrows
    let WasmInstance { store, instance, .. } = &mut *inst;

    // Get the function
    let func: Func = instance
        .get_func(&mut *store, &func_name)
        .ok_or_else(|| {
            WasmError::export_not_found(format!(
                "Function '{}' not found in instance '{}'",
                func_name, instance_id
            ))
        })?;

    // Verify argument types
    let func_ty = func.ty(&*store);
    let expected_params: Vec<ValType> = func_ty.params().collect();

    if args.len() != expected_params.len() {
        return Err(WasmError::type_error(format!(
            "Expected {} arguments, got {}",
            expected_params.len(),
            args.len()
        )));
    }

    for (i, (arg, expected)) in args.iter().zip(expected_params.iter()).enumerate() {
        // Compare types by string representation since ValType doesn't implement PartialEq
        let arg_type_str = val_type_to_string(&arg.val_type());
        let expected_type_str = val_type_to_string(expected);
        if arg_type_str != expected_type_str {
            return Err(WasmError::type_error(format!(
                "Argument {} type mismatch: expected {}, got {}",
                i,
                expected_type_str,
                arg_type_str
            )));
        }
    }

    // Convert arguments
    let wasm_args: Vec<Val> = args.iter().map(|a| a.to_wasmtime()).collect();

    // Prepare results buffer
    let result_count = func_ty.results().count();
    let mut results = vec![Val::I32(0); result_count];

    // Call the function
    func.call(&mut *store, &wasm_args, &mut results)?;

    // Convert results
    let output: Vec<WasmValue> = results
        .iter()
        .filter_map(WasmValue::from_wasmtime)
        .collect();

    debug!(instance_id = %instance_id, func_name = %func_name, results_count = output.len(), "wasm.call complete");
    Ok(output)
}

// ============================================================================
// Memory Operations
// ============================================================================

/// Helper to get memory from instance
async fn get_instance_memory(
    state: &Rc<RefCell<OpState>>,
    instance_id: &str,
) -> Result<(Arc<Mutex<WasmInstance>>, String), WasmError> {
    let instance_arc = {
        let s = state.borrow();
        let ws = s
            .try_borrow::<WasmState>()
            .ok_or_else(|| WasmError::memory_error("WASM state not initialized"))?;
        ws.instances
            .get(instance_id)
            .cloned()
            .ok_or_else(|| {
                WasmError::invalid_instance_handle(format!("Instance '{}' not found", instance_id))
            })?
    };

    // Find the memory export
    let mut inst = instance_arc.lock().await;
    let memory_name = {
        // Destructure to get separate mutable borrows
        let WasmInstance { store, instance, .. } = &mut *inst;
        let mut found_name = None;
        for export in instance.exports(&mut *store) {
            let name = export.name().to_string();
            // Check if this export is a memory
            if let Extern::Memory(_) = export.into_extern() {
                found_name = Some(name);
                break;
            }
        }
        found_name
    };
    drop(inst);

    let memory_name = memory_name
        .ok_or_else(|| WasmError::memory_error("No memory export found in instance"))?;

    Ok((instance_arc, memory_name))
}

/// Read bytes from instance memory
#[op2(async)]
#[serde]
async fn op_wasm_memory_read(
    state: Rc<RefCell<OpState>>,
    #[string] instance_id: String,
    offset: u32,
    length: u32,
) -> Result<Vec<u8>, WasmError> {
    debug!(instance_id = %instance_id, offset = offset, length = length, "wasm.memory_read");

    let (instance_arc, memory_name) = get_instance_memory(&state, &instance_id).await?;
    let mut inst = instance_arc.lock().await;

    // Get memory by destructuring to avoid borrow issues
    let WasmInstance { store, instance, .. } = &mut *inst;
    let memory: Memory = instance
        .get_memory(&mut *store, &memory_name)
        .ok_or_else(|| WasmError::memory_error("Memory export not found"))?;

    let data = memory.data(&*store);
    let start = offset as usize;
    let end = start + length as usize;

    if end > data.len() {
        return Err(WasmError::memory_error(format!(
            "Memory access out of bounds: offset {} + length {} > size {}",
            offset,
            length,
            data.len()
        )));
    }

    let result = data[start..end].to_vec();
    debug!(instance_id = %instance_id, read_len = result.len(), "wasm.memory_read complete");
    Ok(result)
}

/// Write bytes to instance memory
#[op2(async)]
async fn op_wasm_memory_write(
    state: Rc<RefCell<OpState>>,
    #[string] instance_id: String,
    offset: u32,
    #[serde] data: Vec<u8>,
) -> Result<(), WasmError> {
    debug!(instance_id = %instance_id, offset = offset, len = data.len(), "wasm.memory_write");

    let (instance_arc, memory_name) = get_instance_memory(&state, &instance_id).await?;
    let mut inst = instance_arc.lock().await;

    // Get memory by destructuring to avoid borrow issues
    let WasmInstance { store, instance, .. } = &mut *inst;
    let memory: Memory = instance
        .get_memory(&mut *store, &memory_name)
        .ok_or_else(|| WasmError::memory_error("Memory export not found"))?;

    let mem_data = memory.data_mut(&mut *store);
    let start = offset as usize;
    let end = start + data.len();

    if end > mem_data.len() {
        return Err(WasmError::memory_error(format!(
            "Memory access out of bounds: offset {} + length {} > size {}",
            offset,
            data.len(),
            mem_data.len()
        )));
    }

    mem_data[start..end].copy_from_slice(&data);
    debug!(instance_id = %instance_id, "wasm.memory_write complete");
    Ok(())
}

/// Get memory size in pages (64KB each)
#[op2(async)]
async fn op_wasm_memory_size(
    state: Rc<RefCell<OpState>>,
    #[string] instance_id: String,
) -> Result<u32, WasmError> {
    debug!(instance_id = %instance_id, "wasm.memory_size");

    let (instance_arc, memory_name) = get_instance_memory(&state, &instance_id).await?;
    let mut inst = instance_arc.lock().await;

    // Get memory by destructuring to avoid borrow issues
    let WasmInstance { store, instance, .. } = &mut *inst;
    let memory: Memory = instance
        .get_memory(&mut *store, &memory_name)
        .ok_or_else(|| WasmError::memory_error("Memory export not found"))?;

    let size = memory.size(&*store) as u32;
    debug!(instance_id = %instance_id, size_pages = size, "wasm.memory_size complete");
    Ok(size)
}

/// Grow memory by specified pages
#[op2(async)]
async fn op_wasm_memory_grow(
    state: Rc<RefCell<OpState>>,
    #[string] instance_id: String,
    pages: u32,
) -> Result<u32, WasmError> {
    debug!(instance_id = %instance_id, pages = pages, "wasm.memory_grow");

    let (instance_arc, memory_name) = get_instance_memory(&state, &instance_id).await?;
    let mut inst = instance_arc.lock().await;

    // Get memory by destructuring to avoid borrow issues
    let WasmInstance { store, instance, .. } = &mut *inst;
    let memory: Memory = instance
        .get_memory(&mut *store, &memory_name)
        .ok_or_else(|| WasmError::memory_error("Memory export not found"))?;

    let prev_size = memory
        .grow(&mut *store, pages as u64)
        .map_err(|e| WasmError::memory_error(format!("Failed to grow memory: {}", e)))?;

    debug!(instance_id = %instance_id, prev_size = prev_size, new_pages = pages, "wasm.memory_grow complete");
    Ok(prev_size as u32)
}

// ============================================================================
// State Initialization
// ============================================================================

/// Initialize WASM state in OpState
pub fn init_wasm_state(
    op_state: &mut OpState,
    capabilities: Option<Arc<dyn WasmCapabilityChecker>>,
    max_instances: Option<usize>,
) {
    op_state.put(WasmState::new(max_instances.unwrap_or(10)));
    if let Some(caps) = capabilities {
        op_state.put(WasmCapabilities { checker: caps });
    }
}

// ============================================================================
// Extension Registration
// ============================================================================

deno_core::extension!(
    host_wasm,
    ops = [
        op_wasm_compile,
        op_wasm_compile_file,
        op_wasm_drop_module,
        op_wasm_instantiate,
        op_wasm_drop_instance,
        op_wasm_get_exports,
        op_wasm_call,
        op_wasm_memory_read,
        op_wasm_memory_write,
        op_wasm_memory_size,
        op_wasm_memory_grow,
    ],
    esm_entry_point = "ext:host_wasm/init.js",
    esm = ["ext:host_wasm/init.js" = "js/init.js"]
);

pub fn wasm_extension() -> Extension {
    host_wasm::ext()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        let err = WasmError::compile_error("test");
        match err {
            WasmError::CompileError { code, .. } => {
                assert_eq!(code, WasmErrorCode::CompileError as u32);
            }
            _ => panic!("Wrong error type"),
        }

        let err = WasmError::permission_denied("test");
        match err {
            WasmError::PermissionDenied { code, .. } => {
                assert_eq!(code, WasmErrorCode::PermissionDenied as u32);
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_wasm_value_conversion() {
        let val = WasmValue::I32(42);
        let wt_val = val.to_wasmtime();
        assert_eq!(wt_val.i32(), Some(42));

        let back = WasmValue::from_wasmtime(&wt_val);
        assert!(matches!(back, Some(WasmValue::I32(42))));

        let val64 = WasmValue::I64(1234567890123);
        let wt_val64 = val64.to_wasmtime();
        assert_eq!(wt_val64.i64(), Some(1234567890123));

        let valf32 = WasmValue::F32(3.14);
        let wt_valf32 = valf32.to_wasmtime();
        let back_f32 = WasmValue::from_wasmtime(&wt_valf32);
        assert!(matches!(back_f32, Some(WasmValue::F32(f)) if (f - 3.14).abs() < 0.001));
    }

    #[test]
    fn test_state_initialization() {
        let state = WasmState::new(5);
        assert_eq!(state.max_instances, 5);
        assert!(state.modules.is_empty());
        assert!(state.instances.is_empty());
        assert_eq!(state.next_module_id, 1);
        assert_eq!(state.next_instance_id, 1);
    }

    #[test]
    fn test_state_id_generation() {
        let mut state = WasmState::default();
        assert_eq!(state.generate_module_id(), "mod_1");
        assert_eq!(state.generate_module_id(), "mod_2");
        assert_eq!(state.generate_instance_id(), "inst_1");
        assert_eq!(state.generate_instance_id(), "inst_2");
    }

    #[test]
    fn test_wasi_config_default() {
        let config = WasiConfig::default();
        assert!(config.preopens.is_none());
        assert!(config.env.is_none());
        assert!(config.args.is_none());
        assert!(config.inherit_stdin.is_none());
        assert!(config.inherit_stdout.is_none());
        assert!(config.inherit_stderr.is_none());
    }

    #[test]
    fn test_permissive_checker() {
        let checker = PermissiveWasmChecker;
        assert!(checker.check_load("/any/path.wasm").is_ok());
        assert!(checker.check_preopen("/any/dir").is_ok());
    }
}
