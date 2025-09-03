use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::vm::{class::Class, jvalue::JValue, opcode::AType};

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

    pub fn new_primitive_array(ty: AType, count: i32) -> Self {
        if count < 0 {
            panic!("new_primitive_array expected count > 0, received {}", count);
        }
        let class = Rc::new(Class::primitive_array_class(ty));
        let kind = match ty {
            AType::Boolean => JObjectKind::ArrayBoolean(vec![false; count as usize]),
            AType::Char => JObjectKind::ArrayChar(vec![0; count as usize]),
            AType::Float => JObjectKind::ArrayFloat(vec![0.0; count as usize]),
            AType::Double => JObjectKind::ArrayDouble(vec![0.0; count as usize]),
            AType::Byte => JObjectKind::ArrayByte(vec![0; count as usize]),
            AType::Short => JObjectKind::ArrayShort(vec![0; count as usize]),
            AType::Int => JObjectKind::ArrayInt(vec![0; count as usize]),
            AType::Long => JObjectKind::ArrayLong(vec![0; count as usize]),
        };

        Self {
            class,
            kind,
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
    ArrayRef(Vec<Rc<RefCell<JObject>>>),
    ArrayChar(Vec<u16>),
    ArrayByte(Vec<i8>),
    ArrayShort(Vec<i16>),
    ArrayBoolean(Vec<bool>)
}