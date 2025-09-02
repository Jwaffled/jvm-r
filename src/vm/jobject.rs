use std::{collections::HashMap, rc::Rc};

use crate::vm::{class::Class, jvalue::JValue};

#[derive(Debug)]
pub struct JObject {
    pub class: Rc<Class>,
    pub fields: HashMap<String, JValue>
}

impl JObject {
    pub fn new(class: Rc<Class>) -> Self {
        Self {
            class,
            fields: HashMap::new()
        }
    }
}