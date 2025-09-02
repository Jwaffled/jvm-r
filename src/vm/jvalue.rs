use std::rc::Rc;

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
    Reference(Rc<JObject>),
    Null
}