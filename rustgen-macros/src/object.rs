use crate::rustgen_ir::{ExprField, Member};
use proc_macro2::Ident;
pub use serde_json::from_str;
pub use serde_json::{Number, Value};
use std::result;

#[allow(non_snake_case)]
mod TypeName {
    pub const STRING: &str = "string";
    pub const NUMBER: &str = "number";
    pub const NULL: &str = "null";
    pub const OBJECT: &str = "object";
    pub const ARRAY: &str = "array";
    pub const BOOL: &str = "boolean";
}

pub struct NotFoundError {
    candidates: Vec<String>,
    search_ident: Ident,
}

impl NotFoundError {
    fn into_syn_error(self) -> syn::Error {
        let message = format!(
            r#"
child element not found 
    expected : {}
    found    : {}
        "#,
            self.search_ident,
            self.candidates.join(", ")
        );
        syn::Error::new_spanned(&self.search_ident, message)
    }
}

pub struct NoChildError {
    given_type: &'static str,
    given_value: String,
    search_ident: Ident,
}

impl NoChildError {
    fn into_syn_error(self) -> syn::Error {
        let message = format!(
            r#"
`{}` has no child elements
    expected : {}
    found    : {} ({})
        "#,
            self.search_ident.to_string(),
            TypeName::OBJECT,
            self.given_type,
            self.given_value,
        );
        syn::Error::new_spanned(&self.search_ident, message)
    }
}

pub enum Error {
    NotFound(NotFoundError),
    NoChild(NoChildError),
}

impl Error {
    pub fn into_syn_error(self) -> syn::Error {
        match self {
            Error::NotFound(err) => err.into_syn_error(),
            Error::NoChild(err) => err.into_syn_error(),
        }
    }
}

pub type Result<T> = result::Result<T, Error>;

pub fn eval_as_bool(value: &Value) -> bool {
    match value {
        Value::Bool(b) => *b,
        Value::Null => false,
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Number(n) => n == &Number::from_f64(0f64).unwrap(),
        Value::Object(_) => true,
    }
}

pub fn search<'a>(value: &'a Value, field: &'_ ExprField) -> Result<&'a Value> {
    let mut current_field = field;
    let mut current_value = value;
    loop {
        // support string only
        let query = current_field.name();
        if query != "this" {
            let search_ident = current_field.ident.clone();
            match current_value {
                Value::String(s) => {
                    return Err(Error::NoChild(NoChildError {
                        search_ident,
                        given_value: s.clone(),
                        given_type: TypeName::STRING,
                    }))
                }
                Value::Bool(b) => {
                    return Err(Error::NoChild(NoChildError {
                        search_ident,
                        given_value: b.to_string(),
                        given_type: TypeName::BOOL,
                    }))
                }
                Value::Number(n) => {
                    return Err(Error::NoChild(NoChildError {
                        search_ident,
                        given_value: n.to_string(),
                        given_type: TypeName::NUMBER,
                    }))
                }
                Value::Array(_) => {
                    return Err(Error::NoChild(NoChildError {
                        search_ident,
                        given_value: "[ ... ]".to_owned(),
                        given_type: TypeName::ARRAY,
                    }))
                }
                Value::Null => {
                    return Err(Error::NoChild(NoChildError {
                        search_ident,
                        given_value: "null".to_owned(),
                        given_type: TypeName::NULL,
                    }))
                }
                Value::Object(o) => {
                    if let Some(value) = o.get(&query) {
                        current_value = value;
                    } else {
                        return Err(Error::NotFound(NotFoundError {
                            search_ident,
                            candidates: o.keys().cloned().collect(),
                        }));
                    }
                }
            };
        }
        if let Some(Member { field, .. }) = &current_field.member {
            current_field = field;
        } else {
            break;
        }
    }
    Ok(current_value)
}
