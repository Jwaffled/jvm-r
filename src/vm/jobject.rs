use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::vm::{class::Class, jvalue::JValue, opcode::AType};

#[derive(Debug)]
pub struct JObject {
    pub class: Rc<Class>,
    pub kind: JObjectKind,
    pub fields: HashMap<String, JValue>
}

impl JObject {
    pub fn new(class: Rc<Class>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            class: class.clone(),
            kind: JObjectKind::Object,
            fields: JObject::default_fields(class),
        }))
    }

    pub fn new_kind(class: Rc<Class>, kind: JObjectKind) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            class: class.clone(),
            kind,
            fields: JObject::default_fields(class)
        }))
    }

    fn default_fields(class: Rc<Class>) -> HashMap<String, JValue> {
        let mut map = HashMap::new();
        for (field_name, field) in &class.fields {
            let value = match field.descriptor.as_bytes()[0] as char {
                'B' | 'S' | 'I' | 'C' | 'Z' => JValue::Int(0),
                'J' => JValue::Long(0),
                'F' => JValue::Float(0.0),
                'D' => JValue::Double(0.0),
                'L' | '[' => JValue::Null,
                _ => panic!()
            };
            map.insert(field_name.clone(), value);
        }

        map
    }

    pub fn new_primitive_array(ty: AType, count: i32) -> Rc<RefCell<Self>> {
        if count < 0 {
            panic!("new_primitive_array expected count > 0, received {}", count);
        }
        let class = Class::primitive_array_class(ty);
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

        Rc::new(RefCell::new(Self {
            class,
            kind,
            fields: HashMap::new()
        }))
    }

    pub fn new_reference_array(class: Rc<Class>, count: i32) -> Rc<RefCell<Self>> {
        if count < 0 {
            panic!("new_reference_array expected count > 0, received {}", count);
        }

        let kind = JObjectKind::ArrayRef(vec![None; count as usize]);

        Rc::new(RefCell::new(Self {
            class,
            kind,
            fields: HashMap::new()
        }))
    }

    pub fn array_length(&self) -> i32 {
        match &self.kind {
            JObjectKind::ArrayInt(vec) => vec.len() as i32,
            JObjectKind::ArrayLong(vec) => vec.len() as i32,
            JObjectKind::ArrayFloat(vec) => vec.len() as i32,
            JObjectKind::ArrayDouble(vec) => vec.len() as i32,
            JObjectKind::ArrayRef(vec) => vec.len() as i32,
            JObjectKind::ArrayChar(vec) => vec.len() as i32,
            JObjectKind::ArrayByte(vec) => vec.len() as i32,
            JObjectKind::ArrayShort(vec) => vec.len() as i32,
            JObjectKind::ArrayBoolean(vec) => vec.len() as i32,
            JObjectKind::Object => panic!("Object was not an array")
        }
    }

    pub fn set_field(&mut self, name: &str, value: JValue) {
        self.fields.insert(name.to_string(), value);
    }

    pub fn get_field(&self, name: &str) -> JValue {
        self.fields.get(name).unwrap().clone()
    }
}

#[derive(Debug)]
pub enum JObjectKind {
    Object,
    ArrayInt(Vec<i32>),
    ArrayLong(Vec<i64>),
    ArrayFloat(Vec<f32>),
    ArrayDouble(Vec<f64>),
    ArrayRef(Vec<Option<Rc<RefCell<JObject>>>>),
    ArrayChar(Vec<u16>),
    ArrayByte(Vec<i8>),
    ArrayShort(Vec<i16>),
    ArrayBoolean(Vec<bool>)
}