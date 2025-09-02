use crate::{reader::ConstantPoolInfo, vm::{class_loader::ConstantPool, jvalue::JValue}};

#[derive(Debug)]
pub struct VMConstantPool {
    cp: ConstantPool
}

impl VMConstantPool {
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
}