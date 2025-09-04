use std::rc::Rc;

use crate::{reader::ConstantPoolInfo, vm::{class::Class, class_loader::{ClassLoader, ConstantPool}, jobject::JObject, jvalue::JValue}};

#[derive(Debug)]
pub struct VMConstantPool {
    cp: ConstantPool
}

impl VMConstantPool {
    pub fn empty() -> Self {
        Self {
            cp: Vec::new()
        }
    }
    
    pub fn new(cp: ConstantPool) -> Self {
        Self {
            cp
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

    pub fn resolve_constant_value(&self, index: u16) -> JValue {
        match &self.cp[index as usize] {
            ConstantPoolInfo::Integer { bytes } => JValue::Int(*bytes),
            ConstantPoolInfo::Float { bytes } => JValue::Float(*bytes),
            ConstantPoolInfo::Long { bytes } => JValue::Long(*bytes),
            ConstantPoolInfo::Double { bytes } => JValue::Double(*bytes),
            other => panic!("Expected Constant Value, received {:?}", other)
        }
    }

    pub fn resolve_class(&self, class_index: u16, loader: &mut ClassLoader) -> Rc<Class> {
        let class_name = self.get_class_name(class_index);
        loader.load_class(&class_name).unwrap()
    }

    pub fn resolve_ldc_constant(&self, index: u16) -> JValue {
        match &self.cp[index as usize] {
            ConstantPoolInfo::Integer { bytes } => JValue::Int(*bytes),
            ConstantPoolInfo::Float { bytes} => JValue::Float(*bytes),
            ConstantPoolInfo::String { string_index } => {
                if let ConstantPoolInfo::Utf8 { string } = &self.cp[*string_index as usize] {
                    unimplemented!("String ldc not supported yet");
                } else {
                    panic!("Invalid String constant pool entry at {}", string_index);
                }
            }
            ConstantPoolInfo::Class { name_index } => {
                if let ConstantPoolInfo::Utf8 { string } = &self.cp[*name_index as usize] {
                    unimplemented!("Class ldc not supported yet");
                } else {
                    panic!("Invalid String constant pool entry at {}", name_index);
                }
            }
            other => panic!("Unsupported ldc constant: {:?}", other)
        }
    }
}