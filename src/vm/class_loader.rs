use std::{collections::HashMap, error::Error, rc::Rc};

use crate::{reader::{ClassFileReader, ConstantPoolInfo}, vm::class::Class};

pub type ConstantPool = Vec<ConstantPoolInfo>;

pub struct ClassLoader {
    loaded_classes: HashMap<String, Rc<Class>>
}

impl ClassLoader {
    pub fn new() -> Self {
        Self {
            loaded_classes: HashMap::new()
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