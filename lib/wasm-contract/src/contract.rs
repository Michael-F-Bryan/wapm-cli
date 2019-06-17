//! The definition of a WASM contract

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Contract {
    /// Things that the module can import
    pub imports: HashMap<(String, String), Import>,
    /// Things that the module must export
    pub exports: HashMap<String, Export>,
}

impl Contract {
    pub fn merge(&self, other: Contract) -> Result<Contract, String> {
        let mut base = self.clone();

        for (key, val) in other.imports.into_iter() {
            if base.imports.contains_key(&key) {
                if val != base.imports[&key] {
                    return Err(format!("Conflict detected: the import \"{}\" \"{}\" was found but the definitions were different: {:?} {:?}", &key.0, &key.1, base.imports[&key], val));
                }
            } else {
                let res = base.imports.insert(key, val);
                debug_assert!(res.is_none());
            }
        }

        for (key, val) in other.exports.into_iter() {
            if base.exports.contains_key(&key) {
                if val != base.exports[&key] {
                    return Err(format!("Conflict detected: the key {} was found in exports but the definitions were different: {:?} {:?}", key, base.exports[&key], val));
                }
            } else {
                let res = base.exports.insert(key, val);
                debug_assert!(res.is_none());
            }
        }
        Ok(base)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Import {
    Func {
        namespace: String,
        name: String,
        params: Vec<WasmType>,
        result: Vec<WasmType>,
    },
    Global {
        namespace: String,
        name: String,
        var_type: WasmType,
    },
}

impl Import {
    pub fn format_key(ns: &str, name: &str) -> (String, String) {
        (ns.to_string(), name.to_string())
    }

    /// Get the key used to look this import up in the Contract's import hashmap
    pub fn get_key(&self) -> (String, String) {
        match self {
            Import::Func {
                namespace, name, ..
            } => Self::format_key(&namespace, &name),
            Import::Global {
                namespace, name, ..
            } => Self::format_key(&namespace, &name),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Export {
    Func {
        name: String,
        params: Vec<WasmType>,
        result: Vec<WasmType>,
    },
    Global {
        name: String,
        var_type: WasmType,
    },
}

impl Export {
    pub fn format_key(name: &str) -> String {
        name.to_string()
    }

    /// Get the key used to look this export up in the Contract's export hashmap
    pub fn get_key(&self) -> String {
        match self {
            Export::Func { name, .. } => Self::format_key(&name),
            Export::Global { name, .. } => Self::format_key(&name),
        }
    }
}

/// Primitive wasm type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WasmType {
    I32,
    I64,
    F32,
    F64,
}

impl std::fmt::Display for WasmType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                WasmType::I32 => "i32",
                WasmType::I64 => "i64",
                WasmType::F32 => "f32",
                WasmType::F64 => "f64",
            }
        )
    }
}

#[cfg(test)]
mod test {
    use crate::parser;

    #[test]
    fn merging_works() {
        let contract1_src = r#"(assert_import (func "env" "plus_one" (param i32) (result i32)))"#;
        let contract2_src = r#"(assert_import (func "env" "plus_one" (param i64) (result i64)))"#;
        let contract3_src = r#"(assert_import (func "env" "times_two" (param i64) (result i64)))"#;
        let contract4_src =
            r#"(assert_import (func "env" "times_two" (param i64 i64) (result i64)))"#;
        let contract5_src = r#"(assert_export (func "empty_bank_account" (param) (result)))"#;
        let contract6_src = r#"(assert_export (func "empty_bank_account" (param) (result i64)))"#;

        let contract1 = parser::parse_contract(contract1_src).unwrap();
        let contract2 = parser::parse_contract(contract2_src).unwrap();
        let contract3 = parser::parse_contract(contract3_src).unwrap();
        let contract4 = parser::parse_contract(contract4_src).unwrap();
        let contract5 = parser::parse_contract(contract5_src).unwrap();
        let contract6 = parser::parse_contract(contract6_src).unwrap();

        assert!(contract1.merge(contract2.clone()).is_err());
        assert!(contract2.merge(contract1.clone()).is_err());
        assert!(contract1.merge(contract3.clone()).is_ok());
        assert!(contract2.merge(contract3.clone()).is_ok());
        assert!(contract3.merge(contract2.clone()).is_ok());
        assert!(
            contract1.merge(contract1.clone()).is_ok(),
            "exact matches are accepted"
        );
        assert!(contract3.merge(contract4.clone()).is_err());
        assert!(contract5.merge(contract5.clone()).is_ok());
        assert!(contract5.merge(contract6.clone()).is_err());
    }
}