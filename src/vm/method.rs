use std::{fmt::Debug, iter::Peekable};

use crate::{reader::{Attribute, MethodInfo}, vm::{constant_pool::VMConstantPool, jvalue::DescriptorType}};

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

        println!("Descriptor from CP: {}", descriptor);

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

    pub fn empty(name: &str, descriptor: &str) -> Self {
        Self {
            name: name.to_string(),
            descriptor: descriptor.to_string(),
            code: Vec::new(),
            max_locals: 1,
            max_stack: 0,
        }
    }

    pub fn parse_method_descriptor(descriptor: &str) -> (Vec<DescriptorType>, DescriptorType) {
        let mut chars = descriptor.chars().peekable();

        if chars.next() != Some('(') {
            panic!("Method descriptor must start with '('");
        }

        let mut args = Vec::new();
        while let Some(&ch) = chars.peek() {
            if ch == ')' {
                chars.next();
                break;
            }
            args.push(Self::parse_field_type(&mut chars));
        }

        let ret = Self::parse_return_type(&mut chars);

        (args, ret)
    }

    fn parse_return_type<I: Iterator<Item = char>>(chars: &mut Peekable<I>) -> DescriptorType {
        if let Some(&ch) = chars.peek() {
            if ch == 'V' {
                chars.next();
                return DescriptorType::Void;
            }
        }

        Self::parse_field_type(chars)
    }

    fn parse_field_type<I: Iterator<Item = char>>(chars: &mut Peekable<I>) -> DescriptorType {
        let ch = chars.next().unwrap();
        match ch {
            'B' => DescriptorType::Byte,
            'C' => DescriptorType::Char,
            'D' => DescriptorType::Double,
            'F' => DescriptorType::Float,
            'I' => DescriptorType::Int,
            'J' => DescriptorType::Long,
            'S' => DescriptorType::Short,
            'Z' => DescriptorType::Boolean,
            'L' => {
                let mut name = String::new();
                while let Some(c) = chars.next() {
                    if c == ';' {
                        break;
                    }
                    name.push(c);
                }
                DescriptorType::Object(name)
            },
            '[' => {
                let inner = Self::parse_field_type(chars);
                DescriptorType::Array(Box::new(inner))
            },
            other => panic!("Unknown descriptor {:?}", other)
        }
    }
}