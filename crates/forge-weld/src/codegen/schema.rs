use crate::build::schema::{SchemaConfig, SchemaError, SchemaFormat};
use crate::ir::{OpSymbol, WeldEnum, WeldModule, WeldPrimitive, WeldStruct, WeldType};
use serde_json::{json, Value};

/// Generator for JSON Schema and OpenAPI specifications
///
/// Creates standards-compliant schema documents from Forge extension metadata.
/// Supports JSON Schema Draft 2020-12 and OpenAPI 3.1.0.
pub struct SchemaGenerator<'a> {
    module: &'a WeldModule,
    config: &'a SchemaConfig,
}

/// A generated schema file
pub struct GeneratedSchema {
    pub format: SchemaFormat,
    pub filename: String,
    pub content: String,
}

impl<'a> SchemaGenerator<'a> {
    /// Create a new schema generator
    pub fn new(module: &'a WeldModule, config: &'a SchemaConfig) -> Self {
        Self { module, config }
    }

    /// Generate all requested schema formats
    pub fn generate_all(&self) -> Result<Vec<GeneratedSchema>, SchemaError> {
        let mut schemas = Vec::new();

        for format in &self.config.formats {
            match format {
                SchemaFormat::JsonSchema => {
                    schemas.push(self.generate_json_schema()?);
                }
                SchemaFormat::OpenApi => {
                    schemas.push(self.generate_openapi()?);
                }
                SchemaFormat::TypeScriptSdk => {
                    // Handled separately via SdkClassGenerator
                    // Skip here to avoid duplication
                }
            }
        }

        Ok(schemas)
    }

    /// Generate JSON Schema Draft 2020-12
    fn generate_json_schema(&self) -> Result<GeneratedSchema, SchemaError> {
        let mut schema = json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": self.schema_id(),
            "title": self.module.specifier.clone(),
            "description": self.module.doc.as_deref().unwrap_or("Forge extension schema"),
            "type": "object",
        });

        // Add definitions for all structs and enums
        let mut definitions = json!({});

        for s in &self.module.structs {
            definitions[&s.ts_name] = self.struct_to_json_schema(s)?;
        }

        for e in &self.module.enums {
            definitions[&e.ts_name] = self.enum_to_json_schema(e);
        }

        if !definitions.as_object().unwrap().is_empty() {
            schema["$defs"] = definitions;
        }

        // Add operations as a custom extension
        let mut operations = json!({});
        for op in &self.module.ops {
            operations[&op.ts_name] = self.op_to_json_schema(op)?;
        }

        schema["x-operations"] = operations;

        let content = if self.config.include_examples {
            serde_json::to_string_pretty(&schema)?
        } else {
            serde_json::to_string_pretty(&schema)?
        };

        let filename = self.generate_filename("schema.json");

        Ok(GeneratedSchema {
            format: SchemaFormat::JsonSchema,
            filename,
            content,
        })
    }

    /// Generate OpenAPI 3.1.0 specification
    fn generate_openapi(&self) -> Result<GeneratedSchema, SchemaError> {
        let mut openapi = json!({
            "openapi": "3.1.0",
            "info": {
                "title": self.module.specifier.clone(),
                "version": "1.0.0",
                "description": self.module.doc.as_deref().unwrap_or("Forge extension API"),
            },
            "paths": {},
        });

        // Add schemas for all types
        let mut components = json!({
            "schemas": {}
        });

        for s in &self.module.structs {
            components["schemas"][&s.ts_name] = self.struct_to_json_schema(s)?;
        }

        for e in &self.module.enums {
            components["schemas"][&e.ts_name] = self.enum_to_json_schema(e);
        }

        if !components["schemas"].as_object().unwrap().is_empty() {
            openapi["components"] = components;
        }

        // Add operations as a custom extension (OpenAPI 3.1 doesn't have a standard way for RPC-style ops)
        let mut operations = json!({});
        for op in &self.module.ops {
            operations[&op.ts_name] = self.op_to_openapi_operation(op)?;
        }

        openapi["x-operations"] = operations;

        let content = serde_json::to_string_pretty(&openapi)?;
        let filename = self.generate_filename("openapi.json");

        Ok(GeneratedSchema {
            format: SchemaFormat::OpenApi,
            filename,
            content,
        })
    }

    /// Convert WeldStruct to JSON Schema
    fn struct_to_json_schema(&self, s: &WeldStruct) -> Result<Value, SchemaError> {
        let mut schema = json!({
            "type": "object",
        });

        if let Some(ref doc) = s.doc {
            schema["description"] = json!(doc);
        }

        let mut properties = json!({});
        let mut required = Vec::new();

        for field in &s.fields {
            let field_schema = self.weld_type_to_json_schema(&field.ty)?;

            // Add description if available
            let mut field_schema = if let Some(ref doc) = field.doc {
                let mut fs = field_schema;
                fs["description"] = json!(doc);
                fs
            } else {
                field_schema
            };

            if field.readonly {
                field_schema["readOnly"] = json!(true);
            }

            properties[&field.ts_name] = field_schema;

            if !field.optional {
                required.push(field.ts_name.clone());
            }
        }

        schema["properties"] = properties;

        if !required.is_empty() {
            schema["required"] = json!(required);
        }

        Ok(schema)
    }

    /// Convert WeldEnum to JSON Schema
    fn enum_to_json_schema(&self, e: &WeldEnum) -> Value {
        let mut schema = json!({
            "type": "string",
            "enum": e.variants.iter().map(|v| &v.name).collect::<Vec<_>>(),
        });

        if let Some(ref doc) = e.doc {
            schema["description"] = json!(doc);
        }

        schema
    }

    /// Convert operation to JSON Schema operation description
    fn op_to_json_schema(&self, op: &OpSymbol) -> Result<Value, SchemaError> {
        let mut operation = json!({
            "type": "function",
            "async": op.is_async,
        });

        if let Some(ref doc) = op.doc {
            operation["description"] = json!(doc);
        }

        // Parameters
        let mut parameters = json!({});
        for param in op.visible_params() {
            let param_schema = self.weld_type_to_json_schema(&param.ty)?;
            parameters[&param.ts_name] = param_schema;
        }

        operation["parameters"] = parameters;

        // Return type
        operation["returns"] = self.weld_type_to_json_schema(&op.return_type)?;

        Ok(operation)
    }

    /// Convert operation to OpenAPI operation object
    fn op_to_openapi_operation(&self, op: &OpSymbol) -> Result<Value, SchemaError> {
        let mut operation = json!({
            "operationId": op.ts_name.clone(),
            "description": op.doc.as_deref().unwrap_or(&format!("{} operation", op.ts_name)),
        });

        // Parameters as request body
        let mut param_props = json!({});
        let mut required = Vec::new();

        for param in op.visible_params() {
            param_props[&param.ts_name] = self.weld_type_to_json_schema(&param.ty)?;
            if !param.optional {
                required.push(param.ts_name.clone());
            }
        }

        if !param_props.as_object().unwrap().is_empty() {
            operation["requestBody"] = json!({
                "content": {
                    "application/json": {
                        "schema": {
                            "type": "object",
                            "properties": param_props,
                            "required": required,
                        }
                    }
                }
            });
        }

        // Response
        operation["responses"] = json!({
            "200": {
                "description": "Successful operation",
                "content": {
                    "application/json": {
                        "schema": self.weld_type_to_json_schema(&op.return_type)?
                    }
                }
            }
        });

        Ok(operation)
    }

    /// Convert WeldType to JSON Schema type
    fn weld_type_to_json_schema(&self, ty: &WeldType) -> Result<Value, SchemaError> {
        use WeldType::*;
        Ok(match ty {
            Primitive(p) => self.primitive_to_json_schema(p),

            Option(inner) => json!({
                "anyOf": [
                    self.weld_type_to_json_schema(inner)?,
                    {"type": "null"}
                ]
            }),

            Vec(inner) => json!({
                "type": "array",
                "items": self.weld_type_to_json_schema(inner)?
            }),

            Bytes => json!({
                "type": "string",
                "contentEncoding": "base64"
            }),

            Result { ok, .. } => {
                // Results become the success type (errors are exceptions)
                self.weld_type_to_json_schema(ok)?
            }

            HashMap { key: _, value } | BTreeMap { key: _, value } => json!({
                "type": "object",
                "additionalProperties": self.weld_type_to_json_schema(value)?
            }),

            HashSet(inner) | BTreeSet(inner) => json!({
                "type": "array",
                "items": self.weld_type_to_json_schema(inner)?,
                "uniqueItems": true
            }),

            Tuple(types) => {
                let items: std::vec::Vec<Value> = types
                    .iter()
                    .map(|t| self.weld_type_to_json_schema(t))
                    .collect::<std::result::Result<_, _>>()?;
                json!({
                    "type": "array",
                    "prefixItems": items,
                    "items": false,
                    "minItems": types.len(),
                    "maxItems": types.len()
                })
            }

            Array { element, size } => json!({
                "type": "array",
                "items": self.weld_type_to_json_schema(element)?,
                "minItems": size,
                "maxItems": size
            }),

            Generic { base, params } => {
                // For generics, we can't fully represent them in JSON Schema
                // Use a reference to the base type and add a comment
                json!({
                    "$ref": format!("#/$defs/{}", base),
                    "description": format!("Generic type with parameters: {}<{}>", base,
                        params.iter()
                            .map(|p| format!("{:?}", p))
                            .collect::<std::vec::Vec<_>>()
                            .join(", "))
                })
            }

            Struct(name) | Enum(name) => json!({
                "$ref": format!("#/$defs/{}", name)
            }),

            JsonValue => json!({
                "description": "Dynamic JSON value"
            }),

            OpState => {
                // Internal state, shouldn't appear in public API
                json!({"type": "null", "description": "Internal runtime state"})
            }

            // Wrapper types - unwrap and recurse
            Box(inner) | Arc(inner) | Rc(inner) | RefCell(inner) | Mutex(inner) | RwLock(inner) => {
                self.weld_type_to_json_schema(inner)?
            }

            Reference { inner, .. } => self.weld_type_to_json_schema(inner)?,

            Pointer { inner, .. } => {
                // Pointers are unsafe and shouldn't appear in public API
                let mut schema = self.weld_type_to_json_schema(inner)?;
                schema["x-unsafe"] = json!(true);
                schema["description"] = json!("Unsafe pointer type");
                schema
            }

            Never => json!({
                "not": {}
            }),

            Unknown => json!({
                "description": "Unknown type"
            }),
        })
    }

    /// Convert primitive type to JSON Schema
    fn primitive_to_json_schema(&self, p: &WeldPrimitive) -> Value {
        use WeldPrimitive::*;
        match p {
            // Integer types
            U8 | U16 | U32 | I8 | I16 | I32 => json!({
                "type": "integer"
            }),

            // Large integers (bigint in TS)
            U64 | I64 => json!({
                "type": "integer",
                "format": "int64"
            }),

            Usize | Isize => json!({
                "type": "integer",
                "description": "Platform-dependent integer size"
            }),

            // Floating point
            F32 | F64 => json!({
                "type": "number"
            }),

            // Boolean
            Bool => json!({
                "type": "boolean"
            }),

            // String types
            String | Str | Char => json!({
                "type": "string"
            }),

            // Void/null
            Unit => json!({
                "type": "null"
            }),
        }
    }

    /// Generate schema $id URL
    fn schema_id(&self) -> String {
        let base_url = self
            .config
            .schema_base_url
            .as_deref()
            .unwrap_or("https://forge.dev/schemas");

        let module_name = self
            .module
            .specifier
            .replace(':', ".")
            .replace('_', "-");

        format!("{}/{}.json", base_url, module_name)
    }

    /// Generate output filename
    fn generate_filename(&self, extension: &str) -> String {
        let module_name = self
            .module
            .specifier
            .replace(':', ".")
            .replace('_', "-");

        if self.config.versioned {
            format!("{}.v1.{}", module_name, extension)
        } else {
            format!("{}.{}", module_name, extension)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{EnumVariant, OpParam, StructField, WeldPrimitive};

    #[test]
    fn test_filename_generation() {
        let module = WeldModule::new("runtime_fs", "runtime:fs");
        let config = SchemaConfig::default();
        let gen = SchemaGenerator::new(&module, &config);

        assert_eq!(gen.generate_filename("schema.json"), "runtime.fs.schema.json");

        let config_versioned = SchemaConfig {
            versioned: true,
            ..Default::default()
        };
        let gen = SchemaGenerator::new(&module, &config_versioned);
        assert_eq!(
            gen.generate_filename("schema.json"),
            "runtime.fs.v1.schema.json"
        );
    }

    #[test]
    fn test_primitive_to_json_schema() {
        let module = WeldModule::new("test", "test");
        let config = SchemaConfig::default();
        let gen = SchemaGenerator::new(&module, &config);

        let int_schema = gen.primitive_to_json_schema(&WeldPrimitive::I32);
        assert_eq!(int_schema["type"], "integer");

        let float_schema = gen.primitive_to_json_schema(&WeldPrimitive::F64);
        assert_eq!(float_schema["type"], "number");

        let bool_schema = gen.primitive_to_json_schema(&WeldPrimitive::Bool);
        assert_eq!(bool_schema["type"], "boolean");

        let string_schema = gen.primitive_to_json_schema(&WeldPrimitive::String);
        assert_eq!(string_schema["type"], "string");
    }

    #[test]
    fn test_weld_type_to_json_schema() {
        let module = WeldModule::new("test", "test");
        let config = SchemaConfig::default();
        let gen = SchemaGenerator::new(&module, &config);

        // Option type
        let option_schema = gen
            .weld_type_to_json_schema(&WeldType::Option(Box::new(WeldType::string())))
            .unwrap();
        assert!(option_schema["anyOf"].is_array());

        // Array type
        let array_schema = gen
            .weld_type_to_json_schema(&WeldType::Vec(Box::new(WeldType::primitive(WeldPrimitive::I32))))
            .unwrap();
        assert_eq!(array_schema["type"], "array");
        assert_eq!(array_schema["items"]["type"], "integer");

        // Struct reference
        let struct_schema = gen
            .weld_type_to_json_schema(&WeldType::Struct("TestStruct".to_string()))
            .unwrap();
        assert_eq!(struct_schema["$ref"], "#/$defs/TestStruct");
    }

    #[test]
    fn test_struct_to_json_schema() {
        let module = WeldModule::new("test", "test");
        let config = SchemaConfig::default();
        let gen = SchemaGenerator::new(&module, &config);

        let mut test_struct = WeldStruct::new("TestData");
        test_struct
            .fields
            .push(StructField::new("name", WeldType::string()));
        test_struct
            .fields
            .push(StructField::new("age", WeldType::primitive(WeldPrimitive::I32)).optional());

        let schema = gen.struct_to_json_schema(&test_struct).unwrap();

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["name"].is_object());
        assert!(schema["properties"]["age"].is_object());
        assert_eq!(schema["required"].as_array().unwrap().len(), 1);
        assert_eq!(schema["required"][0], "name");
    }

    #[test]
    fn test_enum_to_json_schema() {
        let module = WeldModule::new("test", "test");
        let config = SchemaConfig::default();
        let gen = SchemaGenerator::new(&module, &config);

        let test_enum = WeldEnum::new("Status")
            .variant(EnumVariant {
                name: "Active".to_string(),
                value: None,
                data: None,
                doc: None,
            })
            .variant(EnumVariant {
                name: "Inactive".to_string(),
                value: None,
                data: None,
                doc: None,
            });

        let schema = gen.enum_to_json_schema(&test_enum);

        assert_eq!(schema["type"], "string");
        let enum_values = schema["enum"].as_array().unwrap();
        assert_eq!(enum_values.len(), 2);
    }

    #[test]
    fn test_op_to_json_schema() {
        let module = WeldModule::new("test", "test");
        let config = SchemaConfig::default();
        let gen = SchemaGenerator::new(&module, &config);

        let op = OpSymbol::from_rust_name("op_test_add")
            .param(OpParam::new("a", WeldType::primitive(WeldPrimitive::F64)))
            .param(OpParam::new("b", WeldType::primitive(WeldPrimitive::F64)))
            .returns(WeldType::primitive(WeldPrimitive::F64))
            .with_doc("Add two numbers");

        let operation = gen.op_to_json_schema(&op).unwrap();

        assert_eq!(operation["type"], "function");
        assert_eq!(operation["description"], "Add two numbers");
        assert!(operation["parameters"].is_object());
        assert!(operation["returns"].is_object());
    }

    #[test]
    fn test_json_schema_generation() {
        let mut module = WeldModule::new("runtime_test", "runtime:test");
        module.ops.push(
            OpSymbol::from_rust_name("op_test_hello")
                .param(OpParam::new("name", WeldType::string()))
                .returns(WeldType::string()),
        );

        let config = SchemaConfig::default();
        let gen = SchemaGenerator::new(&module, &config);

        let result = gen.generate_json_schema();
        assert!(result.is_ok());

        let schema = result.unwrap();
        assert_eq!(schema.format, SchemaFormat::JsonSchema);
        assert!(schema.filename.ends_with("schema.json"));

        // Parse to verify valid JSON
        let parsed: Value = serde_json::from_str(&schema.content).unwrap();
        assert_eq!(parsed["$schema"], "https://json-schema.org/draft/2020-12/schema");
        assert_eq!(parsed["title"], "runtime:test");
    }
}
