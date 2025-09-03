use std::{collections::HashMap, error::Error, rc::Rc};

use crate::{reader::{ClassFileReader, ConstantPoolInfo}, vm::{class::Class, opcode::AType}};

pub type ConstantPool = Vec<ConstantPoolInfo>;

pub struct ClassLoader {
    loaded_classes: HashMap<String, Rc<Class>>
}

impl ClassLoader {
    pub fn new() -> Self {
        let mut loaded_classes = HashMap::new();
        for ty in [AType::Boolean, AType::Byte, AType::Char, AType::Double, AType::Float, AType::Int, AType::Long, AType::Short] {
            let class = Class::primitive_array_class(ty);
            loaded_classes.insert(class.name.clone(), Rc::new(class));
        }

        Self {
            loaded_classes
        }
    }

    pub fn load_class(&mut self, name: &str) -> Result<Rc<Class>, Box<dyn Error>> {
        if let Some(class) = self.loaded_classes.get(name) {
            return Ok(class.clone());
        }

        let path = format!("{}.class", name.replace('.', "/"));

        let classfile = ClassFileReader::read_file(&path)?;
        let class = Rc::new(Class::from_classfile(classfile));
        self.loaded_classes.insert(name.to_string(),  class.clone());
        Ok(class)
    }
}