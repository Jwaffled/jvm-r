#![allow(unused)]

use std::{
    fs::File,
    io::{self, BufRead, BufReader, Cursor, Error, Read, Take},
};

use bitflags::bitflags;
use byteorder::{BigEndian, ReadBytesExt};
use num_enum::{IntoPrimitive, TryFromPrimitive, TryFromPrimitiveError};

use crate::vm::opcode::{AType, Opcode, WideInstruction};

#[derive(Default, Debug)]
pub struct ClassFile {
    pub magic: u32,
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool_count: u16,
    pub constant_pool: Vec<ConstantPoolInfo>,
    pub access_flags: ClassAccessFlags,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces_count: u16,
    pub interfaces: Vec<ConstantPoolInfo>,
    pub fields_count: u16,
    pub fields: Vec<FieldInfo>,
    pub methods_count: u16,
    pub methods: Vec<MethodInfo>,
    pub attributes_count: u16,
    pub attributes: Vec<AttributeInfo>,
}

#[derive(Debug, Clone)]
pub enum ConstantPoolInfo {
    Utf8 {
        string: String,
    },
    Integer {
        bytes: i32,
    },
    Float {
        bytes: f32,
    },
    Long {
        bytes: i64,
    },
    Double {
        bytes: f64,
    },
    Class {
        name_index: u16,
    },
    String {
        string_index: u16,
    },
    FieldRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    MethodRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    InterfaceMethodRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    NameAndType {
        name_index: u16,
        descriptor_index: u16,
    },
    MethodHandle {
        reference_kind: u8,
        reference_index: u16,
    },
    MethodType {
        descriptor_index: u16,
    },
    Dynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    },
    InvokeDynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    },
    Module {
        name_index: u16,
    },
    Package {
        name_index: u16,
    },
}

bitflags! {
    #[derive(Default, Debug, Clone, Copy)]
    pub struct ClassAccessFlags: u16 {
        const Public = 0x0001;
        const Final = 0x0010;
        const Super = 0x0020;
        const Interface = 0x0200;
        const Abstract = 0x0400;
        const Synthetic = 0x1000;
        const Annotation = 0x2000;
        const Enum = 0x4000;
        const Module = 0x8000;
    }

    #[derive(Default, Debug, Clone, Copy)]
    pub struct FieldAccessFlags: u16 {
        const Public = 0x0001;
        const Private = 0x0002;
        const Protected = 0x0004;
        const Static = 0x0008;
        const Final = 0x0010;
        const Volatile = 0x0040;
        const Transient = 0x0080;
        const Synthetic = 0x1000;
        const Enum = 0x4000;
    }

    #[derive(Default, Debug, Clone, Copy)]
    pub struct MethodAccessFlags: u16 {
        const Public = 0x0001;
        const Private = 0x0002;
        const Protected = 0x0004;
        const Static = 0x0008;
        const Final = 0x0010;
        const Synchronized = 0x0020;
        const Bridge = 0x0040;
        const Varargs = 0x0080;
        const Native = 0x0100;
        const Abstract = 0x0400;
        const Strict = 0x0800;
        const Synthetic = 0x1000;
    }
}

#[derive(Debug)]
pub struct FieldInfo {
    pub access_flags: FieldAccessFlags,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes_count: u16,
    pub attributes: Vec<AttributeInfo>,
}

#[derive(Debug)]
pub struct MethodInfo {
    pub access_flags: MethodAccessFlags,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes_count: u16,
    pub attributes: Vec<AttributeInfo>,
}

#[derive(Debug)]
pub struct AttributeInfo {
    pub attribute_name_index: u16,
    pub attribute_length: u32,
    pub info: Attribute,
}

#[derive(Debug)]
pub struct ExceptionTable {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_type: u16,
}

#[derive(Debug)]
enum StackMapFrame {
    Same {
        offset_delta: u16,
    },
    SameLocals1StackItem {
        offset_delta: u16,
        stack: Vec<VerificationTypeInfo>
    },
    SameLocals1StackItemExtended {
        offset_delta: u16,
        stack: Vec<VerificationTypeInfo>
    },
    Chop {
        offset_delta: u16,
    },
    SameExtended {
        offset_delta: u16,
    },
    Append {
        offset_delta: u16,
        locals: Vec<VerificationTypeInfo>,
    },
    Full {
        offset_delta: u16,
        number_of_locals: u16,
        locals: Vec<VerificationTypeInfo>,
        number_of_stack_items: u16,
        stack: Vec<VerificationTypeInfo>,
    }
}

#[derive(Debug)]
enum VerificationTypeInfo {
    Top,
    Integer,
    Float,
    Double,
    Long,
    Null,
    UninitializedThis,
    Object {
        cpool_index: u16,
    },
    Uninitialized {
        offset: u16,
    },
}

#[derive(Debug)]
struct BootstrapMethod {
    bootstrap_method_ref: u16,
    num_bootstrap_arguments: u16,
    bootstrap_arguments: Vec<u16>,
}

#[derive(Debug)]
struct LineNumberTable {
    pub start_pc: u16,
    pub line_number: u16,
}

#[derive(Debug)]
pub enum Attribute {
    ConstantValue {
        constantvalue_index: u16,
    },
    Code {
        max_stack: u16,
        max_locals: u16,
        code_length: u32,
        code: Vec<u8>, // opcodes
        exception_table_length: u16,
        exception_table: Vec<ExceptionTable>,
        attributes_count: u16,
        attributes: Vec<AttributeInfo>,
    },
    StackMapTable {
        number_of_entries: u16,
        entries: Vec<StackMapFrame>,
    },
    Exceptions {},
    InnerClasses {},
    EnclosingMethod {},
    Synthetic {},
    Signature {},
    SourceFile {
        sourcefile_index: u16,
    },
    SourceDebugExtension {},
    LineNumberTable {
        line_number_table_length: u16,
        line_number_table: Vec<LineNumberTable>,
    },
    LocalVariableTable {},
    LocalVariableTypeTable {},
    Deprecated {},
    RuntimeVisibleAnnotations {},
    RuntimeInvisibleAnnotations {},
    RuntimeVisibleParameterAnnotations {},
    RuntimeInvisibleParameterAnnotations {},
    RuntimeVisibleTypeAnnotations {},
    RuntimeInvisibleTypeAnnotations {},
    AnnotationDefault {},
    BootstrapMethods {
        num_bootstrap_methods: u16,
        bootstrap_methods: Vec<BootstrapMethod>,
    },
    MethodParameters {},
    Module {},
    ModulePackages {},
    ModuleMainClass {},
    NestHost {
        host_class_index: u16,
    },
    NestMembers {
        number_of_classes: u16,
        classes: Vec<u16>,
    },
    Record {},
    PermittedSubclasses {
        number_of_classes: u16,
        classes: Vec<u16>,
    },
}

#[derive(Debug)]
pub struct ClassFileReader {
    buf: BufReader<File>,
    constant_pool: Vec<ConstantPoolInfo>,
}

impl ClassFileReader {
    pub fn new(path: &str) -> Self {
        let file = File::open(path).expect("Error opening file");
        Self {
            buf: BufReader::new(file),
            constant_pool: Vec::new(),
        }
    }

    pub fn read_file(path: &str) -> io::Result<ClassFile> {
        let mut reader = Self::new(path);
        reader.read()
    }

    pub fn read(&mut self) -> io::Result<ClassFile> {
        let magic = self.buf.read_u32::<BigEndian>()?;
        let minor_version = self.buf.read_u16::<BigEndian>()?;
        let major_version = self.buf.read_u16::<BigEndian>()?;
        let constant_pool_count = self.buf.read_u16::<BigEndian>()?;
        let mut constant_pool = vec![ConstantPoolInfo::String { string_index: 0 }]; // unhinged jvm spec bs
        for i in 0..constant_pool_count - 1 {
            let tag = self.buf.read_u8()?;
            constant_pool.push(self.constant_tag(tag)?);
        }
        self.constant_pool = constant_pool.to_vec();
        let access_flags = ClassAccessFlags::from_bits(self.buf.read_u16::<BigEndian>()?)
            .ok_or_else(|| self.report_error("Expected class access flags, got invalid flag"))?;
        let this_class = self.buf.read_u16::<BigEndian>()?;
        let super_class = self.buf.read_u16::<BigEndian>()?;
        let interfaces_count = self.buf.read_u16::<BigEndian>()?;
        let interfaces = self.read_interfaces(interfaces_count)?;
        let fields_count = self.buf.read_u16::<BigEndian>()?;
        let fields = self.read_fields(fields_count)?;
        let methods_count = self.buf.read_u16::<BigEndian>()?;
        let methods = self.read_methods(methods_count)?;
        let attributes_count = self.buf.read_u16::<BigEndian>()?;
        let attributes = self.read_attributes(attributes_count)?;

        Ok(ClassFile {
            magic,
            minor_version,
            major_version,
            constant_pool_count,
            constant_pool,
            access_flags,
            this_class,
            super_class,
            interfaces_count,
            interfaces,
            fields_count,
            fields,
            methods_count,
            methods,
            attributes_count,
            attributes,
        })
    }

    fn read_methods(&mut self, n: u16) -> io::Result<Vec<MethodInfo>> {
        let mut methods = Vec::new();
        for i in 0..n {
            let access_flags = MethodAccessFlags::from_bits(self.buf.read_u16::<BigEndian>()?)
                .ok_or_else(|| {
                    self.report_error("Expected method access flags, got invalid flag")
                })?;
            let name_index = self.buf.read_u16::<BigEndian>()?;
            let descriptor_index = self.buf.read_u16::<BigEndian>()?;
            let attributes_count = self.buf.read_u16::<BigEndian>()?;
            let attributes = self.read_attributes(attributes_count)?;
            methods.push(MethodInfo {
                access_flags,
                name_index,
                descriptor_index,
                attributes_count,
                attributes,
            });
        }
        Ok(methods)
    }

    fn read_exception_table(&mut self, n: u16) -> io::Result<Vec<ExceptionTable>> {
        let mut tables = Vec::new();
        for _ in 0..n {
            let start_pc = self.buf.read_u16::<BigEndian>()?;
            let end_pc = self.buf.read_u16::<BigEndian>()?;
            let handler_pc = self.buf.read_u16::<BigEndian>()?;
            let catch_type = self.buf.read_u16::<BigEndian>()?;
            tables.push(ExceptionTable {
                start_pc,
                end_pc,
                handler_pc,
                catch_type,
            })
        }

        Ok(tables)
    }

    fn read_code_attrib(&mut self) -> io::Result<Attribute> {
        let max_stack = self.buf.read_u16::<BigEndian>()?;
        let max_locals = self.buf.read_u16::<BigEndian>()?;
        let code_length = self.buf.read_u32::<BigEndian>()?;
        let mut code = vec![0_u8; code_length as usize];
        self.buf.read_exact(&mut code)?;

        let exception_table_length = self.buf.read_u16::<BigEndian>()?;
        let exception_table = self.read_exception_table(exception_table_length)?;
        let attributes_count = self.buf.read_u16::<BigEndian>()?;
        let attributes = self.read_attributes(attributes_count)?;
        Ok(Attribute::Code {
            max_stack,
            max_locals,
            code_length,
            code,
            exception_table_length,
            exception_table,
            attributes_count,
            attributes,
        })
    }

    fn read_line_number_table_attrib(&mut self) -> io::Result<Attribute> {
        let line_number_table_length = self.buf.read_u16::<BigEndian>()?;
        let mut line_number_table = Vec::new();
        for _ in 0..line_number_table_length {
            let start_pc = self.buf.read_u16::<BigEndian>()?;
            let line_number = self.buf.read_u16::<BigEndian>()?;
            line_number_table.push(LineNumberTable {
                start_pc,
                line_number,
            });
        }
        Ok(Attribute::LineNumberTable {
            line_number_table_length,
            line_number_table,
        })
    }

    fn read_stack_map_table_attrib(&mut self) -> io::Result<Attribute> {
        let number_of_entries = self.buf.read_u16::<BigEndian>()?;
        let mut entries = vec![];
        for _ in 0..number_of_entries {
            let frame_type = self.buf.read_u8()?;
            let frame_entry = match frame_type {
                0..=63 => {
                    StackMapFrame::Same { offset_delta: frame_type as u16 }
                },
                64..=127 => {
                    let offset_delta = (frame_type as u16) - 64;
                    let stack = vec![self.read_verification_type_info()?];
                    StackMapFrame::SameLocals1StackItem { offset_delta, stack }
                },
                128..=246 => panic!("Frame type tag {} reserved for future use; not supported by spec.", frame_type),
                247 => {
                    let offset_delta = self.buf.read_u16::<BigEndian>()?;
                    let stack = vec![self.read_verification_type_info()?];
                    StackMapFrame::SameLocals1StackItemExtended { offset_delta, stack }
                },
                248..=250 => {
                    let offset_delta = self.buf.read_u16::<BigEndian>()?;
                    StackMapFrame::Chop { offset_delta }
                },
                251 => {
                    let offset_delta = self.buf.read_u16::<BigEndian>()?;
                    StackMapFrame::SameExtended { offset_delta }
                },
                252..=254 => {
                    let offset_delta = self.buf.read_u16::<BigEndian>()?;
                    let mut locals = vec![];
                    for _ in 0..frame_type - 251 {
                        locals.push(self.read_verification_type_info()?);
                    }
                    StackMapFrame::Append { offset_delta, locals }
                },
                255 => {
                    let offset_delta = self.buf.read_u16::<BigEndian>()?;
                    let number_of_locals = self.buf.read_u16::<BigEndian>()?;
                    let mut locals = vec![];
                    for _ in 0..number_of_locals {
                        locals.push(self.read_verification_type_info()?);
                    }
                    let number_of_stack_items = self.buf.read_u16::<BigEndian>()?;
                    let mut stack = vec![];
                    for _ in 0..number_of_stack_items {
                        stack.push(self.read_verification_type_info()?);
                    }

                    StackMapFrame::Full { offset_delta, number_of_locals, locals, number_of_stack_items, stack }
                }
            };

            entries.push(frame_entry);
        }

        Ok(Attribute::StackMapTable { number_of_entries, entries })
    }

    fn read_verification_type_info(&mut self) -> io::Result<VerificationTypeInfo> {
        let tag = self.buf.read_u8()?;
        let result = match tag {
            0 => VerificationTypeInfo::Top,
            1 => VerificationTypeInfo::Integer,
            2 => VerificationTypeInfo::Float,
            3 => VerificationTypeInfo::Double,
            4 => VerificationTypeInfo::Long,
            5 => VerificationTypeInfo::Null,
            6 => VerificationTypeInfo::UninitializedThis,
            7 => {
                let cpool_index = self.buf.read_u16::<BigEndian>()?;
                VerificationTypeInfo::Object { cpool_index }
            },
            8 => {
                let offset = self.buf.read_u16::<BigEndian>()?;
                VerificationTypeInfo::Uninitialized { offset }
            },
            other => panic!("Invalid VerificationTypeInfo tag received: {}", tag)
        };

        Ok(result)
    }

    fn read_attributes(&mut self, n: u16) -> io::Result<Vec<AttributeInfo>> {
        let mut res = Vec::new();
        for i in 0..n {
            let attribute_name_index = self.buf.read_u16::<BigEndian>()?;
            let attribute_length = self.buf.read_u32::<BigEndian>()?;
            let attrib = match self
                .constant_pool
                .get(attribute_name_index as usize)
                .ok_or_else(|| {
                    self.report_error(&format!(
                        "Expected valid CONSTANT_POOL index, received '{}'",
                        attribute_name_index
                    ))
                })? {
                ConstantPoolInfo::Utf8 { string } => match string.as_str() {
                    "Code" => self.read_code_attrib()?,
                    "LineNumberTable" => self.read_line_number_table_attrib()?,
                    "SourceFile" => {
                        let sourcefile_index = self.buf.read_u16::<BigEndian>()?;
                        Attribute::SourceFile { sourcefile_index }
                    }
                    "StackMapTable" => self.read_stack_map_table_attrib()?,
                    other => {
                        return Err(self
                            .report_error(&format!("Attribute '{}' not implemented yet.", other)))
                    }
                },
                other => {
                    return Err(self.report_error(&format!(
                        "Expected CONSTANT_Utf8_info in attribute name, received '{:?}'",
                        other
                    )))
                }
            };
            res.push(AttributeInfo {
                attribute_name_index,
                attribute_length,
                info: attrib,
            })
        }
        Ok(res)
    }

    fn read_fields(&mut self, n: u16) -> io::Result<Vec<FieldInfo>> {
        let mut fields = Vec::new();
        for i in 0..n {
            let access_flags = FieldAccessFlags::from_bits(self.buf.read_u16::<BigEndian>()?)
                .ok_or_else(|| {
                    self.report_error("Expected field access flags, got invalid flag")
                })?;
            let name_index = self.buf.read_u16::<BigEndian>()?;
            let descriptor_index = self.buf.read_u16::<BigEndian>()?;
            let attributes_count = self.buf.read_u16::<BigEndian>()?;
            let attributes = self.read_attributes(attributes_count)?;
            fields.push(FieldInfo {
                access_flags,
                name_index,
                descriptor_index,
                attributes_count,
                attributes,
            });
        }

        Ok(fields)
    }

    fn read_interfaces(&mut self, n: u16) -> io::Result<Vec<ConstantPoolInfo>> {
        let mut interfaces = Vec::new();
        for i in 0..n {
            let tag = self.buf.read_u8()?;
            if tag != ConstantPoolTag::Class.into() {
                return Err(self.report_error(&format!(
                    "Expected CONSTANT_Class_info tag, received tag '{}'",
                    tag
                )));
            } else {
                interfaces.push(self.constant_tag(tag)?);
            }
        }
        Ok(interfaces)
    }

    fn constant_tag(&mut self, tag: u8) -> io::Result<ConstantPoolInfo> {
        let mut data: Vec<u8> = Vec::new();
        use ConstantPoolInfo as CInfo;
        use ConstantPoolTag as C;
        if let Ok(parsed_tag) = C::try_from(tag) {
            match parsed_tag {
                C::Utf8 => {
                    let length = self.buf.read_u16::<BigEndian>()?;
                    let mut bytes = vec![0u8; length as usize];
                    self.buf.read_exact(&mut bytes)?;
                    match String::from_utf8(bytes) {
                        Ok(str) => Ok(CInfo::Utf8 { string: str }),
                        Err(e) => Err(self.report_error("UTF-8 String was not in correct format.")),
                    }
                }
                C::Integer => {
                    let bytes = self.buf.read_i32::<BigEndian>()?;
                    Ok(CInfo::Integer { bytes })
                }
                C::Float => {
                    let bytes = self.buf.read_f32::<BigEndian>()?;
                    Ok(CInfo::Float { bytes })
                }
                C::Long => {
                    let bytes = self.buf.read_i64::<BigEndian>()?;
                    Ok(CInfo::Long { bytes })
                }
                C::Double => {
                    let bytes = self.buf.read_f64::<BigEndian>()?;
                    Ok(CInfo::Double { bytes })
                }
                C::Class => {
                    let name_index = self.buf.read_u16::<BigEndian>()?;
                    Ok(CInfo::Class { name_index })
                }
                C::String => {
                    let string_index = self.buf.read_u16::<BigEndian>()?;
                    Ok(CInfo::String { string_index })
                }
                C::FieldRef => {
                    let class_index = self.buf.read_u16::<BigEndian>()?;
                    let name_and_type_index = self.buf.read_u16::<BigEndian>()?;
                    Ok(CInfo::FieldRef {
                        class_index,
                        name_and_type_index,
                    })
                }
                C::MethodRef => {
                    let class_index = self.buf.read_u16::<BigEndian>()?;
                    let name_and_type_index = self.buf.read_u16::<BigEndian>()?;
                    Ok(CInfo::MethodRef {
                        class_index,
                        name_and_type_index,
                    })
                }
                C::InterfaceMethodRef => {
                    let class_index = self.buf.read_u16::<BigEndian>()?;
                    let name_and_type_index = self.buf.read_u16::<BigEndian>()?;
                    Ok(CInfo::InterfaceMethodRef {
                        class_index,
                        name_and_type_index,
                    })
                }
                C::NameAndType => {
                    let name_index = self.buf.read_u16::<BigEndian>()?;
                    let descriptor_index = self.buf.read_u16::<BigEndian>()?;
                    Ok(CInfo::NameAndType {
                        name_index,
                        descriptor_index,
                    })
                }
                C::MethodHandle => {
                    let reference_kind = self.buf.read_u8()?;
                    let reference_index = self.buf.read_u16::<BigEndian>()?;
                    Ok(CInfo::MethodHandle {
                        reference_kind,
                        reference_index,
                    })
                }
                C::MethodType => {
                    let descriptor_index = self.buf.read_u16::<BigEndian>()?;
                    Ok(CInfo::MethodType { descriptor_index })
                }
                C::Dynamic => {
                    let bootstrap_method_attr_index = self.buf.read_u16::<BigEndian>()?;
                    let name_and_type_index = self.buf.read_u16::<BigEndian>()?;
                    Ok(CInfo::Dynamic {
                        bootstrap_method_attr_index,
                        name_and_type_index,
                    })
                }
                C::InvokeDynamic => {
                    let bootstrap_method_attr_index = self.buf.read_u16::<BigEndian>()?;
                    let name_and_type_index = self.buf.read_u16::<BigEndian>()?;
                    Ok(CInfo::InvokeDynamic {
                        bootstrap_method_attr_index,
                        name_and_type_index,
                    })
                }
                C::Module => {
                    let name_index = self.buf.read_u16::<BigEndian>()?;
                    Ok(CInfo::Module { name_index })
                }
                C::Package => {
                    let name_index = self.buf.read_u16::<BigEndian>()?;
                    Ok(CInfo::Package { name_index })
                }
            }
        } else {
            Err(self.report_error(&format!("Invalid constant tag '{}' at ", tag)))
        }
    }

    fn report_error(&self, message: &str) -> io::Error {
        println!("[ERROR]: {}", message);
        println!("BYTES: {:?}", self.buf.buffer());
        io::Error::new(
            io::ErrorKind::InvalidData,
            "Bytecode file was not in the correct format!",
        )
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
enum ConstantPoolTag {
    Utf8 = 1,
    Integer = 3,
    Float = 4,
    Long = 5,
    Double = 6,
    Class = 7,
    String = 8,
    FieldRef = 9,
    MethodRef = 10,
    InterfaceMethodRef = 11,
    NameAndType = 12,
    MethodHandle = 15,
    MethodType = 16,
    Dynamic = 17,
    InvokeDynamic = 18,
    Module = 19,
    Package = 20,
}
