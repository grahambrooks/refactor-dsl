//! API signature types for representing function and type definitions.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::change::ApiType;

/// Extracted API signature from source code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSignature {
    /// Name of the API element.
    pub name: String,
    /// Type of API (function, class, etc.).
    pub kind: ApiType,
    /// Visibility (public, private, etc.).
    pub visibility: Visibility,
    /// Function parameters (empty for non-functions).
    pub parameters: Vec<Parameter>,
    /// Return type (for functions).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_type: Option<TypeInfo>,
    /// Generic type parameters.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub generic_params: Vec<String>,
    /// Source location.
    pub location: SourceLocation,
    /// Module/namespace path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_path: Option<String>,
    /// Whether this is exported/public API.
    #[serde(default)]
    pub is_exported: bool,
}

impl ApiSignature {
    /// Create a new function signature.
    pub fn function(name: impl Into<String>, location: SourceLocation) -> Self {
        Self {
            name: name.into(),
            kind: ApiType::Function,
            visibility: Visibility::Public,
            parameters: Vec::new(),
            return_type: None,
            generic_params: Vec::new(),
            location,
            module_path: None,
            is_exported: true,
        }
    }

    /// Create a new method signature (member function of a class/struct).
    pub fn method(name: impl Into<String>, location: SourceLocation) -> Self {
        Self {
            name: name.into(),
            kind: ApiType::Method,
            visibility: Visibility::Public,
            parameters: Vec::new(),
            return_type: None,
            generic_params: Vec::new(),
            location,
            module_path: None,
            is_exported: true,
        }
    }

    /// Create a new type signature (struct, class, interface).
    pub fn type_def(name: impl Into<String>, kind: ApiType, location: SourceLocation) -> Self {
        Self {
            name: name.into(),
            kind,
            visibility: Visibility::Public,
            parameters: Vec::new(),
            return_type: None,
            generic_params: Vec::new(),
            location,
            module_path: None,
            is_exported: true,
        }
    }

    /// Set parameters.
    pub fn with_params(mut self, params: Vec<Parameter>) -> Self {
        self.parameters = params;
        self
    }

    /// Set return type.
    pub fn with_return_type(mut self, return_type: TypeInfo) -> Self {
        self.return_type = Some(return_type);
        self
    }

    /// Set visibility.
    pub fn with_visibility(mut self, visibility: Visibility) -> Self {
        self.visibility = visibility;
        self
    }

    /// Set module path.
    pub fn with_module_path(mut self, path: impl Into<String>) -> Self {
        self.module_path = Some(path.into());
        self
    }

    /// Set exported flag.
    pub fn exported(mut self, is_exported: bool) -> Self {
        self.is_exported = is_exported;
        self
    }

    /// Get a unique identifier for this API.
    pub fn unique_id(&self) -> String {
        match &self.module_path {
            Some(path) => format!("{}::{}", path, self.name),
            None => self.name.clone(),
        }
    }

    /// Get parameter names in order.
    pub fn param_names(&self) -> Vec<&str> {
        self.parameters.iter().map(|p| p.name.as_str()).collect()
    }
}

/// Represents a function/method parameter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    /// Parameter name.
    pub name: String,
    /// Type information (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_info: Option<TypeInfo>,
    /// Whether this parameter has a default value.
    #[serde(default)]
    pub has_default: bool,
    /// Whether this parameter is optional.
    #[serde(default)]
    pub is_optional: bool,
    /// Whether this is a variadic/rest parameter.
    #[serde(default)]
    pub is_variadic: bool,
}

impl Parameter {
    /// Create a new parameter.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            type_info: None,
            has_default: false,
            is_optional: false,
            is_variadic: false,
        }
    }

    /// Set type information.
    pub fn with_type(mut self, type_info: TypeInfo) -> Self {
        self.type_info = Some(type_info);
        self
    }

    /// Mark as having a default value.
    pub fn with_default(mut self) -> Self {
        self.has_default = true;
        self
    }

    /// Mark as optional.
    pub fn optional(mut self) -> Self {
        self.is_optional = true;
        self
    }

    /// Mark as variadic.
    pub fn variadic(mut self) -> Self {
        self.is_variadic = true;
        self
    }

    /// Get a display string for this parameter.
    pub fn display(&self) -> String {
        let mut s = self.name.clone();
        if self.is_optional {
            s.push('?');
        }
        if let Some(ref ty) = self.type_info {
            s.push_str(": ");
            s.push_str(&ty.display());
        }
        if self.has_default {
            s.push_str(" = ...");
        }
        if self.is_variadic {
            s = format!("...{}", s);
        }
        s
    }
}

/// Type information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeInfo {
    /// Type name.
    pub name: String,
    /// Generic type arguments.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub generic_args: Vec<TypeInfo>,
    /// Whether this is an optional type (`T | undefined`, `Option<T>`).
    #[serde(default)]
    pub is_optional: bool,
    /// Whether this is a reference type.
    #[serde(default)]
    pub is_reference: bool,
    /// Whether this is mutable (Rust &mut).
    #[serde(default)]
    pub is_mutable: bool,
}

impl TypeInfo {
    /// Create a simple type.
    pub fn simple(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            generic_args: Vec::new(),
            is_optional: false,
            is_reference: false,
            is_mutable: false,
        }
    }

    /// Create a generic type.
    pub fn generic(name: impl Into<String>, args: Vec<TypeInfo>) -> Self {
        Self {
            name: name.into(),
            generic_args: args,
            is_optional: false,
            is_reference: false,
            is_mutable: false,
        }
    }

    /// Mark as optional.
    pub fn optional(mut self) -> Self {
        self.is_optional = true;
        self
    }

    /// Mark as reference.
    pub fn reference(mut self) -> Self {
        self.is_reference = true;
        self
    }

    /// Mark as mutable reference.
    pub fn mutable(mut self) -> Self {
        self.is_mutable = true;
        self.is_reference = true;
        self
    }

    /// Get a display string for this type.
    pub fn display(&self) -> String {
        let mut s = String::new();

        if self.is_reference {
            s.push('&');
            if self.is_mutable {
                s.push_str("mut ");
            }
        }

        s.push_str(&self.name);

        if !self.generic_args.is_empty() {
            s.push('<');
            let args: Vec<_> = self.generic_args.iter().map(|a| a.display()).collect();
            s.push_str(&args.join(", "));
            s.push('>');
        }

        if self.is_optional && !self.name.starts_with("Option") {
            s.push('?');
        }

        s
    }
}

/// Source code location.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SourceLocation {
    /// File path.
    pub file: PathBuf,
    /// Line number (1-indexed).
    pub line: usize,
    /// Column number (1-indexed).
    pub column: usize,
    /// Byte offset in file.
    #[serde(default)]
    pub byte_offset: usize,
}

impl SourceLocation {
    /// Create a new source location.
    pub fn new(file: impl Into<PathBuf>, line: usize, column: usize) -> Self {
        Self {
            file: file.into(),
            line,
            column,
            byte_offset: 0,
        }
    }

    /// Set byte offset.
    pub fn with_byte_offset(mut self, offset: usize) -> Self {
        self.byte_offset = offset;
        self
    }
}

/// Visibility of an API element.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    #[default]
    Public,
    Private,
    Protected,
    Internal,
    /// Rust pub(crate)
    Crate,
    /// Rust pub(super)
    Super,
}

impl Visibility {
    /// Check if this visibility is public (exported).
    pub fn is_public(&self) -> bool {
        matches!(self, Visibility::Public)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_display() {
        let p1 = Parameter::new("name").with_type(TypeInfo::simple("String"));
        assert_eq!(p1.display(), "name: String");

        let p2 = Parameter::new("age")
            .with_type(TypeInfo::simple("i32"))
            .with_default();
        assert_eq!(p2.display(), "age: i32 = ...");

        let p3 = Parameter::new("args").variadic();
        assert_eq!(p3.display(), "...args");

        let p4 = Parameter::new("callback")
            .optional()
            .with_type(TypeInfo::simple("Function"));
        assert_eq!(p4.display(), "callback?: Function");
    }

    #[test]
    fn test_type_info_display() {
        let t1 = TypeInfo::simple("String");
        assert_eq!(t1.display(), "String");

        let t2 = TypeInfo::generic("Vec", vec![TypeInfo::simple("i32")]);
        assert_eq!(t2.display(), "Vec<i32>");

        let t3 = TypeInfo::simple("T").reference();
        assert_eq!(t3.display(), "&T");

        let t4 = TypeInfo::simple("T").mutable();
        assert_eq!(t4.display(), "&mut T");

        let t5 = TypeInfo::generic(
            "HashMap",
            vec![TypeInfo::simple("String"), TypeInfo::simple("Value")],
        );
        assert_eq!(t5.display(), "HashMap<String, Value>");
    }

    #[test]
    fn test_api_signature_unique_id() {
        let loc = SourceLocation::new("test.rs", 1, 1);

        let sig1 = ApiSignature::function("foo", loc.clone());
        assert_eq!(sig1.unique_id(), "foo");

        let sig2 = ApiSignature::function("bar", loc).with_module_path("mymod");
        assert_eq!(sig2.unique_id(), "mymod::bar");
    }
}
