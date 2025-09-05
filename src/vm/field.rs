use crate::{reader::{Attribute, FieldAccessFlags, FieldInfo}, vm::{constant_pool::VMConstantPool, jvalue::JValue}};

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub descriptor: String,
    pub access_flags: FieldAccessFlags,
    pub constant_value: Option<JValue>
}

impl Field {
    pub fn from_field_info(info: &FieldInfo, cp: &VMConstantPool) -> Self {
        let name = cp.get_utf8(info.name_index);
        let descriptor = cp.get_utf8(info.descriptor_index);

        let constant_value = info
            .attributes
            .iter()
            .find_map(|x| match &x.info {
                Attribute::ConstantValue { constantvalue_index } => {
                    Some(cp.resolve_constant_value(*constantvalue_index))
                }
                _ => None
            });

        Field {
            name,
            descriptor,
            access_flags: info.access_flags.clone(),
            constant_value
        }
    }

    pub fn new(name: &str, descriptor: &str, access_flags: FieldAccessFlags) -> Self {
        Self {
            name: name.to_string(),
            descriptor: descriptor.to_string(),
            access_flags,
            constant_value: None,
        }
    }
}