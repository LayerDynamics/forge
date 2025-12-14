//! Compile-time symbol inventory for Forge extensions
//!
//! This module provides the infrastructure for collecting op symbols
//! at compile time using the `linkme` crate's distributed slices.

use crate::ir::{OpSymbol, WeldEnum, WeldStruct};

/// Distributed slice for collecting op symbols at compile time
#[linkme::distributed_slice]
pub static WELD_OPS: [fn() -> OpSymbol];

/// Distributed slice for collecting struct definitions at compile time
#[linkme::distributed_slice]
pub static WELD_STRUCTS: [fn() -> WeldStruct];

/// Distributed slice for collecting enum definitions at compile time
#[linkme::distributed_slice]
pub static WELD_ENUMS: [fn() -> WeldEnum];

/// Collect all registered ops from the distributed slice
pub fn collect_ops() -> Vec<OpSymbol> {
    WELD_OPS.iter().map(|f| f()).collect()
}

/// Collect all registered structs from the distributed slice
pub fn collect_structs() -> Vec<WeldStruct> {
    WELD_STRUCTS.iter().map(|f| f()).collect()
}

/// Collect all registered enums from the distributed slice
pub fn collect_enums() -> Vec<WeldEnum> {
    WELD_ENUMS.iter().map(|f| f()).collect()
}

/// Registry for manually collecting symbols (alternative to linkme)
#[derive(Debug, Default)]
pub struct SymbolRegistry {
    ops: Vec<OpSymbol>,
    structs: Vec<WeldStruct>,
    enums: Vec<WeldEnum>,
}

impl SymbolRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a registry from the distributed slices
    pub fn from_inventory() -> Self {
        Self {
            ops: collect_ops(),
            structs: collect_structs(),
            enums: collect_enums(),
        }
    }

    /// Register an op
    pub fn register_op(&mut self, op: OpSymbol) {
        self.ops.push(op);
    }

    /// Register a struct
    pub fn register_struct(&mut self, s: WeldStruct) {
        self.structs.push(s);
    }

    /// Register an enum
    pub fn register_enum(&mut self, e: WeldEnum) {
        self.enums.push(e);
    }

    /// Get all registered ops
    pub fn ops(&self) -> &[OpSymbol] {
        &self.ops
    }

    /// Get all registered structs
    pub fn structs(&self) -> &[WeldStruct] {
        &self.structs
    }

    /// Get all registered enums
    pub fn enums(&self) -> &[WeldEnum] {
        &self.enums
    }

    /// Take ownership of ops
    pub fn into_ops(self) -> Vec<OpSymbol> {
        self.ops
    }

    /// Take ownership of structs
    pub fn into_structs(self) -> Vec<WeldStruct> {
        self.structs
    }

    /// Take ownership of enums
    pub fn into_enums(self) -> Vec<WeldEnum> {
        self.enums
    }
}

/// Macro to register an op symbol in the distributed slice
#[macro_export]
macro_rules! register_op {
    ($op:expr) => {
        #[linkme::distributed_slice($crate::ir::WELD_OPS)]
        static _WELD_OP: fn() -> $crate::ir::OpSymbol = || $op;
    };
}

/// Macro to register a struct in the distributed slice
#[macro_export]
macro_rules! register_struct {
    ($s:expr) => {
        #[linkme::distributed_slice($crate::ir::WELD_STRUCTS)]
        static _WELD_STRUCT: fn() -> $crate::ir::WeldStruct = || $s;
    };
}

/// Macro to register an enum in the distributed slice
#[macro_export]
macro_rules! register_enum {
    ($e:expr) => {
        #[linkme::distributed_slice($crate::ir::WELD_ENUMS)]
        static _WELD_ENUM: fn() -> $crate::ir::WeldEnum = || $e;
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{OpParam, WeldType};

    #[test]
    fn test_symbol_registry() {
        let mut registry = SymbolRegistry::new();

        registry.register_op(
            OpSymbol::from_rust_name("op_test")
                .param(OpParam::new("path", WeldType::string()))
                .returns(WeldType::void()),
        );

        assert_eq!(registry.ops().len(), 1);
        assert_eq!(registry.ops()[0].rust_name, "op_test");
    }

    #[test]
    fn test_registry_from_inventory() {
        // This tests that the distributed slices work
        let registry = SymbolRegistry::from_inventory();
        // The slices should be accessible (may be empty in tests if no macros used)
        let _ = registry.ops();
        let _ = registry.structs();
    }
}
