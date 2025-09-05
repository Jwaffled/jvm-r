use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{reader::{ClassAccessFlags, ClassFile}, vm::{class_loader::ClassLoader, constant_pool::VMConstantPool, field::Field, jobject::JObject, method::Method, opcode::AType}};

#[derive(Debug)]
pub struct Class {
    pub name: String,
    pub super_name: Option<String>,
    pub methods: HashMap<String, Rc<Method>>,
    pub fields: HashMap<String, Rc<Field>>,
    pub constant_pool: VMConstantPool,
    pub access_flags: ClassAccessFlags,
}

impl Class {
    pub fn from_classfile(cf: ClassFile) -> Rc<Self> {
        let cp = VMConstantPool::new(cf.constant_pool);

        let name = cp.get_class_name(cf.this_class);

        let super_name = if cf.super_class != 0 {
            let super_class_name = cp.get_class_name(cf.super_class);

            Some(super_class_name)
        } else {
            None
        };

        let methods = cf
            .methods
            .iter()
            .map(|x| Method::from_method_info(x, &cp))
            .map(|x| (format!("{}:{}", x.name, x.descriptor), Rc::new(x)))
            .collect::<HashMap<_, _>>();

        let fields = cf
            .fields
            .iter()
            .map(|x| Field::from_field_info(x, &cp))
            .map(|x| (format!("{}:{}", x.name, x.descriptor), Rc::new(x)))
            .collect::<HashMap<_, _>>();

        Rc::new(Self {
            name,
            super_name,
            methods,
            fields,
            constant_pool: cp,
            access_flags: cf.access_flags,
        })     
    }

    pub fn primitive_array_class(ty: AType) -> Rc<Self> {
        let ty = match ty {
            AType::Byte => "B",
            AType::Char => "C",
            AType::Double => "D",
            AType::Float => "F",
            AType::Int => "I",
            AType::Long => "J",
            AType::Short => "S",
            AType::Boolean => "Z",
        };

        Rc::new(Self {
            name: format!("[{}", ty),
            super_name: Some(String::from("java/lang/Object")),
            methods: HashMap::new(),
            fields: HashMap::new(),
            constant_pool: VMConstantPool::empty(),
            access_flags: ClassAccessFlags::Public,
        })
    }

    pub fn reference_array_class(class_name: &str) -> Rc<Self> {
        Rc::new(Self {
            name: format!("[L{};", class_name),
            super_name: Some(String::from("java/lang/Object")),
            methods: HashMap::new(),
            fields: HashMap::new(),
            constant_pool: VMConstantPool::empty(),
            access_flags: ClassAccessFlags::Public,
        })
    }
}