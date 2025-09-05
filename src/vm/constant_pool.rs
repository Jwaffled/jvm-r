use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{reader::ConstantPoolInfo, vm::{class::Class, class_loader::{ClassLoader, ConstantPool}, field::Field, jobject::{JObject, JObjectKind}, jvalue::JValue}};

#[derive(Debug)]
pub struct VMConstantPool {
    pub cp: ConstantPool,
    resolved_constants: RefCell<HashMap<u16, ResolvedConstant>>
}

impl VMConstantPool {
    pub fn empty() -> Self {
        Self {
            cp: Vec::new(),
            resolved_constants: RefCell::new(HashMap::new())
        }
    }
    
    pub fn new(cp: ConstantPool) -> Self {
        Self {
            cp,
            resolved_constants: RefCell::new(HashMap::new())
        }
    }

    pub fn get_utf8(&self, index: u16) -> String {
        match &self.cp[index as usize] {
            ConstantPoolInfo::Utf8 { string } => string.clone(),
            other => panic!("Expected Utf8, received {:?}", other)
        }
    }

    pub fn get_class_name(&self, index: u16) -> String {
        let idx = match &self.cp[index as usize] {
            ConstantPoolInfo::Class { name_index } => *name_index,
            other => panic!("Expected Class, received {:?}", other)
        };

        self.get_utf8(idx)
    }

    pub fn get_name_and_type(&self, index: u16) -> (String, String) {
        match &self.cp[index as usize] {
            ConstantPoolInfo::NameAndType { name_index, descriptor_index } => {
                let name = self.get_utf8(*name_index);
                let descriptor = self.get_utf8(*descriptor_index);
                (name, descriptor)
            }
            other => panic!("Expected NameAndType, received {:?}", other)
        }
    }

    pub fn resolve_constant_value(&self, index: u16) -> JValue {
        match &self.cp[index as usize] {
            ConstantPoolInfo::Integer { bytes } => JValue::Int(*bytes),
            ConstantPoolInfo::Float { bytes } => JValue::Float(*bytes),
            ConstantPoolInfo::Long { bytes } => JValue::Long(*bytes),
            ConstantPoolInfo::Double { bytes } => JValue::Double(*bytes),
            other => panic!("Expected Constant Value, received {:?}", other)
        }
    }

    pub fn resolve_constant(&self, index: u16, loader: &mut ClassLoader) -> ResolvedConstant {
        if let Some(resolved) = self.resolved_constants.borrow().get(&index) {
            return resolved.clone();
        }

        let resolved = match &self.cp[index as usize] {
            ConstantPoolInfo::Integer { bytes } => ResolvedConstant::Integer(*bytes),
            ConstantPoolInfo::Float { bytes } => ResolvedConstant::Float(*bytes),
            ConstantPoolInfo::String { string_index } => {
                let content = self.get_utf8(*string_index);
                ResolvedConstant::String(content)
            },
            ConstantPoolInfo::Class { name_index } => {
                let name = self.get_utf8(*name_index);

                let class = loader.load_class(&name).unwrap();
                ResolvedConstant::Class(class)
            },
            ConstantPoolInfo::FieldRef { class_index, name_and_type_index } => {
                let class_name = self.get_class_name(*class_index);
                let class = loader.load_class(&class_name).unwrap();
                let (name, descriptor) = self.get_name_and_type(*name_and_type_index);

                let field = class.fields.get(&format!("{}:{}", name, descriptor)).unwrap().clone();
                ResolvedConstant::FieldRef {  field }
            }
            _ => unimplemented!()
        };

        self.resolved_constants.borrow_mut().insert(index, resolved.clone());

        resolved
    }
}


#[derive(Debug, Clone)]
pub enum ResolvedConstant {
    Integer(i32),
    Float(f32),
    String(String),
    Class(Rc<Class>),
    FieldRef {
        // class: Rc<Class>,
        field: Rc<Field>
    }
}