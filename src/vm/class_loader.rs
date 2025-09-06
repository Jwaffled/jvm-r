use std::{cell::RefCell, collections::HashMap, error::Error, rc::Rc};

use crate::{reader::{ClassAccessFlags, ClassFileReader, ConstantPoolInfo, FieldAccessFlags}, vm::{class::Class, constant_pool::VMConstantPool, field::Field, method::Method, opcode::AType}};

pub type ConstantPool = Vec<ConstantPoolInfo>;

pub struct ClassLoader {
    loaded_classes: HashMap<String, Rc<Class>>
}

impl ClassLoader {
    pub fn new() -> Self {
        let mut loader = Self {
            loaded_classes: HashMap::new(),
        };

        loader.load_default_classes();
        loader
    }

    fn load_default_classes(&mut self) {
        // Primitive array classes
        for ty in [AType::Boolean, AType::Byte, AType::Char, AType::Double, AType::Float, AType::Int, AType::Long, AType::Short] {
            let class = Class::primitive_array_class(ty);
            self.loaded_classes.insert(class.name.clone(), class);
        }

        let object_class = Rc::new(Class {
            name: String::from("java/lang/Object"),
            super_name: None,
            fields: HashMap::new(),
            methods: HashMap::from([
                (String::from("<init>:()V"), Rc::new(Method::empty(
                    "<init>", "()V"
                )))
            ]),
            constant_pool: VMConstantPool::empty(),
            access_flags: ClassAccessFlags::Public,
        });

        let string_class = Rc::new(Class {
            name: String::from("java/lang/String"),
            super_name: Some(String::from("java/lang/Object")),
            fields: HashMap::from([
                (String::from("value:[C"), Rc::new(Field::new(
                    "value:[C",
                    "[C",
                    FieldAccessFlags::Final | FieldAccessFlags::Private
                )))
            ]),
            methods: HashMap::new(),
            constant_pool: VMConstantPool::empty(),
            access_flags: ClassAccessFlags::Public,
        });

        let class_class = Rc::new(Class {
            name: String::from("java/lang/Class"),
            super_name: Some(String::from("java/lang/Object")),
            fields: HashMap::new(),
            methods: HashMap::new(),
            constant_pool: VMConstantPool::empty(),
            access_flags: ClassAccessFlags::Public,
        });

        self.loaded_classes.insert(object_class.name.clone(), object_class);
        self.loaded_classes.insert(string_class.name.clone(), string_class);
        self.loaded_classes.insert(class_class.name.clone(), class_class);
    }

    pub fn load_class(&mut self, name: &str) -> Result<Rc<Class>, Box<dyn Error>> {
        if let Some(class) = self.loaded_classes.get(name) {
            return Ok(class.clone());
        }

        let path = format!("{}.class", name.replace('.', "/"));

        let classfile = ClassFileReader::read_file(&path)?;
        let class = Class::from_classfile(classfile);
        self.loaded_classes.insert(name.to_string(),  class.clone());
        Ok(class)
    }
}