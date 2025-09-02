use std::{collections::HashMap, rc::Rc};

use crate::{reader::{ClassAccessFlags, ClassFile, ConstantPoolInfo}, vm::{constant_pool::VMConstantPool, field::Field, method::Method}};

#[derive(Debug)]
pub struct Class {
    pub name: String,
    pub super_name: Option<String>,
    pub methods: HashMap<String, Rc<Method>>,
    pub fields: HashMap<String, Rc<Field>>,
    pub constant_pool: VMConstantPool,
    pub access_flags: ClassAccessFlags
}

impl Class {
    pub fn from_classfile(cf: ClassFile) -> Self {
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
            .map(|x| (x.name.clone(), Rc::new(x)))
            .collect::<HashMap<_, _>>();

        Self {
            name,
            super_name,
            methods,
            fields,
            constant_pool: cp,
            access_flags: cf.access_flags
        }            
    }
}