// validation.rs - Executor output validation against the capability's output_schema.
//
// Validates that an executor result conforms to the output_schema declared
// in the action's VerifiedManifest.  Only the subset of JSON Schema needed
// for 0.1 runtime validation is supported:
//
//   - type          (string or array of strings)
//   - enum / const
//   - properties    (object of per-property schemas)
//   - required      (array of required property names)
//   - additionalProperties (boolean or schema)
//   - items         (one schema applied to every array element)
//
// Unsupported schema forms that reach this function are rejected explicitly
// rather than silently ignored.

use serde_json::Value;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Why output validation failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub message: String,
}

impl ValidationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Validate `result` against the capability's `output_schema`.
///
/// `schema` is the `output_schema` field from the verified manifest.
/// Returns `Ok(())` when the result conforms, or `Err(ValidationError)`
/// with a descriptive human-readable message.
pub fn validate_output(schema: &Value, result: &Value) -> Result<(), ValidationError> {
    validate_schema(schema, result, "$", true)
}

// ---------------------------------------------------------------------------
// Internal validation
// ---------------------------------------------------------------------------

fn validate_schema(
    schema: &Value,
    value: &Value,
    path: &str,
    top_level: bool,
) -> Result<(), ValidationError> {
    match schema {
        Value::Object(schema_obj) => validate_against_object_schema(schema_obj, value, path),
        Value::Bool(true) if !top_level => Ok(()),
        Value::Bool(false) => Err(ValidationError::new(format!(
            "{}: schema is boolean false (no valid value possible)",
            path
        ))),
        _ => Err(ValidationError::new(format!(
            "{}: unsupported output_schema form: {}",
            path, schema
        ))),
    }
}

fn validate_against_object_schema(
    schema: &serde_json::Map<String, Value>,
    value: &Value,
    path: &str,
) -> Result<(), ValidationError> {
    reject_unsupported_keywords(schema, path)?;

    if let Some(expected) = schema.get("const") {
        if value != expected {
            return Err(ValidationError::new(format!(
                "{}: value does not match const {}",
                path, expected
            )));
        }
    }

    if let Some(allowed) = schema.get("enum") {
        let values = allowed.as_array().ok_or_else(|| {
            ValidationError::new(format!("{}: enum must be an array in output_schema", path))
        })?;
        if values.is_empty() {
            return Err(ValidationError::new(format!(
                "{}: enum must contain at least one value",
                path
            )));
        }
        if !values.iter().any(|candidate| candidate == value) {
            return Err(ValidationError::new(format!(
                "{}: value is not one of the allowed enum values",
                path
            )));
        }
    }

    // --- type check ---
    let declared_type = schema.get("type");
    match declared_type {
        Some(Value::String(t)) => check_type(t, value, path)?,
        Some(Value::Array(types)) => {
            let mut matched = false;
            let mut last_err: Option<ValidationError> = None;
            for t in types {
                match t.as_str() {
                    Some(type_str) => match check_type(type_str, value, path) {
                        Ok(()) => {
                            matched = true;
                            break;
                        }
                        Err(e) => last_err = Some(e),
                    },
                    None => {
                        return Err(ValidationError::new(format!(
                            "{}: non-string type element in schema: {}",
                            path, t
                        )));
                    }
                }
            }
            if !matched {
                return Err(last_err.unwrap_or_else(|| {
                    ValidationError::new(format!("{}: value did not match any declared type", path))
                }));
            }
        }
        Some(other) => {
            return Err(ValidationError::new(format!(
                "{}: unsupported schema type form: {}",
                path, other
            )));
        }
        None => {
            // No type constraint declared; accept anything.
        }
    }

    // --- object-specific checks ---
    match value {
        Value::Object(result_obj) => {
            validate_object_properties(schema, result_obj, path)?;
        }
        Value::Array(items) => {
            if let Some(item_schema) = schema.get("items") {
                for (index, item) in items.iter().enumerate() {
                    validate_schema(item_schema, item, &format!("{}[{}]", path, index), false)?;
                }
            }
        }
        _ => {}
    }

    Ok(())
}

fn check_type(type_name: &str, value: &Value, path: &str) -> Result<(), ValidationError> {
    let ok = match type_name {
        "object" => value.is_object(),
        "array" => value.is_array(),
        "string" => value.is_string(),
        "number" => value.is_number(),
        "integer" => value.as_f64().map_or(false, |n| n.fract() == 0.0),
        "boolean" => value.is_boolean(),
        "null" => value.is_null(),
        other => {
            return Err(ValidationError::new(format!(
                "{}: unsupported schema type '{}'",
                path, other
            )));
        }
    };

    if !ok {
        let actual = json_type_name(value);
        return Err(ValidationError::new(format!(
            "{}: type mismatch: expected {}, got {}",
            path, type_name, actual
        )));
    }
    Ok(())
}

fn validate_object_properties(
    schema: &serde_json::Map<String, Value>,
    result: &serde_json::Map<String, Value>,
    path: &str,
) -> Result<(), ValidationError> {
    // --- required ---
    if let Some(required) = schema.get("required") {
        match required {
            Value::Array(names) => {
                for name in names {
                    let key = name.as_str().ok_or_else(|| {
                        ValidationError::new(format!(
                            "{}: non-string element in required array",
                            path
                        ))
                    })?;
                    if !result.contains_key(key) {
                        let prop_path = format!("{}.{}", path, escape_json_pointer(key));
                        return Err(ValidationError::new(format!(
                            "{}: missing required property {}",
                            path, prop_path
                        )));
                    }
                }
            }
            _ => {
                return Err(ValidationError::new(format!(
                    "{}: required must be an array in output_schema, got {}",
                    path, required
                )));
            }
        }
    }

    // --- properties type checking ---
    if let Some(properties) = schema.get("properties") {
        match properties {
            Value::Object(props) => {
                for (prop_name, prop_schema) in props {
                    if let Some(prop_value) = result.get(prop_name) {
                        let prop_path = format!("{}.{}", path, escape_json_pointer(prop_name));
                        validate_schema(prop_schema, prop_value, &prop_path, false)?;
                    }
                }
            }
            _ => {
                return Err(ValidationError::new(format!(
                    "{}: properties must be an object in output_schema",
                    path
                )));
            }
        }
    }

    // --- additionalProperties ---
    if let Some(additional_schema) = schema.get("additionalProperties") {
        let declared = schema.get("properties").and_then(Value::as_object);

        for (key, prop_value) in result {
            if declared.is_some_and(|properties| properties.contains_key(key)) {
                continue;
            }

            let prop_path = format!("{}.{}", path, escape_json_pointer(key));
            match additional_schema {
                Value::Bool(true) => {}
                Value::Bool(false) => {
                    return Err(ValidationError::new(format!(
                        "{}: additional property not allowed at {}",
                        path, prop_path
                    )));
                }
                Value::Object(_) => {
                    validate_schema(additional_schema, prop_value, &prop_path, false)?;
                }
                _ => {
                    return Err(ValidationError::new(format!(
                        "{}: additionalProperties must be a boolean or schema",
                        path
                    )));
                }
            }
        }
    }

    Ok(())
}

fn reject_unsupported_keywords(
    schema: &serde_json::Map<String, Value>,
    path: &str,
) -> Result<(), ValidationError> {
    const SUPPORTED: &[&str] = &[
        "type",
        "enum",
        "const",
        "properties",
        "required",
        "additionalProperties",
        "items",
        "$schema",
        "$id",
        "$comment",
        "title",
        "description",
        "default",
        "examples",
        "deprecated",
        "readOnly",
        "writeOnly",
    ];

    for keyword in schema.keys() {
        if !SUPPORTED.contains(&keyword.as_str()) {
            return Err(ValidationError::new(format!(
                "{}: unsupported output_schema keyword '{}'; refusing partial validation",
                path, keyword
            )));
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn json_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(n) => {
            if n.as_f64().map_or(true, |f| f.fract() != 0.0) {
                "number"
            } else {
                "integer"
            }
        }
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Escape a JSON Pointer segment: replace ~ with ~0 and / with ~1.
fn escape_json_pointer(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // -----------------------------------------------------------------------
    // Test V1: Correct object output passes.
    // -----------------------------------------------------------------------

    #[test]
    fn valid_object_output_passes() {
        let schema = json!({
            "type": "object",
            "properties": {
                "status": {"type": "string"}
            },
            "required": ["status"],
            "additionalProperties": false
        });
        let result = json!({"status": "recorded"});
        assert!(validate_output(&schema, &result).is_ok());
    }

    // -----------------------------------------------------------------------
    // Test V2: Missing required field fails.
    // -----------------------------------------------------------------------

    #[test]
    fn missing_required_field_fails() {
        let schema = json!({
            "type": "object",
            "properties": {
                "status": {"type": "string"}
            },
            "required": ["status"],
            "additionalProperties": false
        });
        let result = json!({"wrong_field": 1});
        let err = validate_output(&schema, &result).unwrap_err();
        assert!(
            err.message.contains("missing required property"),
            "expected missing required property error, got: {}",
            err.message
        );
        assert!(err.message.contains("status"), "expected 'status' in error");
    }

    // -----------------------------------------------------------------------
    // Test V3: Wrong declared type fails.
    // -----------------------------------------------------------------------

    #[test]
    fn wrong_property_type_fails() {
        let schema = json!({
            "type": "object",
            "properties": {
                "status": {"type": "string"}
            },
            "required": ["status"],
            "additionalProperties": false
        });
        let result = json!({"status": 123});
        let err = validate_output(&schema, &result).unwrap_err();
        assert!(
            err.message.contains("type mismatch"),
            "expected type mismatch error, got: {}",
            err.message
        );
        assert!(err.message.contains("string"), "expected 'string' in error");
        assert!(
            err.message.contains("integer"),
            "expected 'integer' in error"
        );
    }

    // -----------------------------------------------------------------------
    // Test V4: Additional property fails when additionalProperties: false.
    // -----------------------------------------------------------------------

    #[test]
    fn additional_property_rejected() {
        let schema = json!({
            "type": "object",
            "properties": {
                "status": {"type": "string"}
            },
            "required": ["status"],
            "additionalProperties": false
        });
        let result = json!({"status": "ok", "extra": true});
        let err = validate_output(&schema, &result).unwrap_err();
        assert!(
            err.message.contains("additional property not allowed"),
            "expected additional property error, got: {}",
            err.message
        );
        assert!(err.message.contains("extra"), "expected 'extra' in error");
    }

    // -----------------------------------------------------------------------
    // Test V5: Additional property accepted when additionalProperties: true.
    // -----------------------------------------------------------------------

    #[test]
    fn additional_property_accepted_when_allowed() {
        let schema = json!({
            "type": "object",
            "properties": {
                "status": {"type": "string"}
            },
            "required": ["status"],
            "additionalProperties": true
        });
        let result = json!({"status": "ok", "extra": true});
        assert!(validate_output(&schema, &result).is_ok());
    }

    // -----------------------------------------------------------------------
    // Test V6: Integer type rejects fractional number.
    // -----------------------------------------------------------------------

    #[test]
    fn integer_type_rejects_fractional() {
        let schema = json!({
            "type": "object",
            "properties": {
                "count": {"type": "integer"}
            },
            "required": ["count"]
        });
        let result = json!({"count": 3.5});
        let err = validate_output(&schema, &result).unwrap_err();
        assert!(
            err.message.contains("type mismatch"),
            "expected type mismatch error, got: {}",
            err.message
        );
        assert!(
            err.message.contains("integer"),
            "expected 'integer' in error"
        );
    }

    // -----------------------------------------------------------------------
    // Test V7: Integer type accepts whole number.
    // -----------------------------------------------------------------------

    #[test]
    fn integer_type_accepts_whole_number() {
        let schema = json!({
            "type": "object",
            "properties": {
                "count": {"type": "integer"}
            },
            "required": ["count"]
        });
        let result = json!({"count": 3});
        assert!(validate_output(&schema, &result).is_ok());
    }

    // -----------------------------------------------------------------------
    // Test V8: Boolean false schema rejects any result.
    // -----------------------------------------------------------------------

    #[test]
    fn boolean_false_schema_rejects() {
        let schema = Value::Bool(false);
        let result = json!({"anything": 1});
        let err = validate_output(&schema, &result).unwrap_err();
        assert!(
            err.message.contains("boolean false"),
            "expected boolean false error, got: {}",
            err.message
        );
    }

    // -----------------------------------------------------------------------
    // Test V9: Unsupported schema shape is rejected.
    // -----------------------------------------------------------------------

    #[test]
    fn unsupported_schema_shape_rejected() {
        let schema = json!(123); // number is not a valid output_schema here
        let result = json!({});
        let err = validate_output(&schema, &result).unwrap_err();
        assert!(
            err.message.contains("unsupported output_schema form"),
            "expected unsupported form error, got: {}",
            err.message
        );
    }

    // -----------------------------------------------------------------------
    // Test V10: Missing optional property is not an error.
    // -----------------------------------------------------------------------

    #[test]
    fn missing_optional_property_accepted() {
        let schema = json!({
            "type": "object",
            "properties": {
                "status": {"type": "string"},
                "optional": {"type": "string"}
            },
            "required": ["status"],
            "additionalProperties": false
        });
        let result = json!({"status": "done"});
        assert!(validate_output(&schema, &result).is_ok());
    }

    #[test]
    fn array_items_are_validated() {
        let schema = json!({
            "type": "array",
            "items": {"type": "string"}
        });

        assert!(validate_output(&schema, &json!(["one", "two"])).is_ok());
        let err = validate_output(&schema, &json!(["one", 2])).unwrap_err();
        assert!(err.message.contains("$[1]"));
        assert!(err.message.contains("type mismatch"));
    }

    #[test]
    fn enum_values_are_validated() {
        let schema = json!({"enum": ["succeeded", "failed"]});

        assert!(validate_output(&schema, &json!("succeeded")).is_ok());
        let err = validate_output(&schema, &json!("unknown")).unwrap_err();
        assert!(err.message.contains("allowed enum values"));
    }

    #[test]
    fn additional_property_schema_is_enforced() {
        let schema = json!({
            "type": "object",
            "properties": {"status": {"type": "string"}},
            "additionalProperties": {"type": "integer"}
        });

        assert!(validate_output(&schema, &json!({"status": "ok", "count": 2})).is_ok());
        let err = validate_output(&schema, &json!({"status": "ok", "count": "two"})).unwrap_err();
        assert!(err.message.contains("$.count"));
        assert!(err.message.contains("type mismatch"));
    }

    #[test]
    fn unsupported_assertion_is_rejected_instead_of_ignored() {
        let schema = json!({"type": "string", "minLength": 5});
        let err = validate_output(&schema, &json!("x")).unwrap_err();
        assert!(err
            .message
            .contains("unsupported output_schema keyword 'minLength'"));
        assert!(err.message.contains("refusing partial validation"));
    }

    // -----------------------------------------------------------------------
    // Test V11: Deterministic error messages.
    // -----------------------------------------------------------------------

    #[test]
    fn validation_errors_are_deterministic() {
        let schema = json!({
            "type": "object",
            "properties": {
                "status": {"type": "string"}
            },
            "required": ["status"],
            "additionalProperties": false
        });
        let result = json!({"status": 123});

        let err1 = validate_output(&schema, &result).unwrap_err();
        let err2 = validate_output(&schema, &result).unwrap_err();
        assert_eq!(err1.message, err2.message);
    }
}
