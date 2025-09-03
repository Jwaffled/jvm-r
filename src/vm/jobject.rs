use std::{collections::HashMap, rc::Rc};

use crate::vm::{class::Class, jvalue::JValue};

#[derive(Debug)]
pub struct JObject {
    pub class: Rc<Class>,
    pub kind: JObjectKind,
    pub fields: HashMap<String, JValue>
}

impl JObject {
    pub fn new(class: Rc<Class>) -> Self {
        Self {
            class,
            kind: JObjectKind::Object,
            fields: HashMap::new()
        }
    }
}

#[derive(Debug)]
pub enum JObjectKind {
    Object,
    ArrayInt(Vec<i32>),
    ArrayLong(Vec<i64>),
    ArrayFloat(Vec<f32>),
    ArrayDouble(Vec<f64>),
    ArrayRef(Vec<Rc<JObject>>),
    ArrayChar(Vec<u16>),
    ArrayByte(Vec<i8>),
    ArrayShort(Vec<i16>),
    ArrayBoolean(Vec<bool>)
}