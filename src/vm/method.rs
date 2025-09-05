use crate::{reader::{Attribute, MethodInfo}, vm::constant_pool::VMConstantPool};

#[derive(Debug)]
pub struct Method {
    pub name: String,
    pub descriptor: String,
    pub code: Vec<u8>,
    pub max_locals: u16,
    pub max_stack: u16,
}

impl Method {
    pub fn from_method_info(info: &MethodInfo, cp: &VMConstantPool) -> Self {
        let name = cp.get_utf8(info.name_index);

        let descriptor = cp.get_utf8(info.descriptor_index);

        let code_attr = info
            .attributes
            .iter()
            .find_map(|x| match &x.info {
                Attribute::Code { max_stack, max_locals,  code, .. } => Some((code.to_vec(), *max_stack, *max_locals)),
                _ => None,
            });

        let (code, max_stack, max_locals) = code_attr.expect("Method missing code attribute");
        
        Self {
            name,
            descriptor,
            code,
            max_locals,
            max_stack
        }
    }
}