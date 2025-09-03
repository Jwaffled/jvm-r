use std::{cell::RefCell, rc::Rc};

use crate::vm::jobject::JObject;

#[derive(Debug, Clone)]
pub enum JValue {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Char(u16),
    Float(f32),
    Double(f64),
    Boolean(bool),
    Reference(Rc<RefCell<JObject>>),
    Null
}

impl JValue {
    pub fn is_category2(&self) -> bool {
        matches!(self, JValue::Long(_) | JValue::Double(_))
    }
}