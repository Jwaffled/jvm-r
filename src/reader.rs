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
struct StackMapFrame {}

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
        code: Vec<Opcode>, // opcodes
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
        let mut code = Vec::with_capacity(code_length as usize);
        
        let mut opcode_reader = self.buf.by_ref().take(code_length as u64);
        let mut bytes_read = 0;
        while let Ok(opcode) = opcode_reader.read_u8() {
            bytes_read += 1;
            let op = Self::parse_opcode(opcode, &mut opcode_reader, &mut bytes_read)?;
            code.push(op);
        }

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

    pub fn parse_opcode<R: Read>(code: u8, reader: &mut R, bytes_read: &mut u32) -> io::Result<Opcode> {
        let op = match code {
            /* Constants */
            0x00 => Opcode::Nop,
            0x01 => Opcode::AConstNull,
            0x02 => Opcode::IConstM1,
            0x03 => Opcode::IConst0,
            0x04 => Opcode::IConst1,
            0x05 => Opcode::IConst2,
            0x06 => Opcode::IConst3,
            0x07 => Opcode::IConst4,
            0x08 => Opcode::IConst5,
            0x09 => Opcode::LConst0,
            0x0A => Opcode::LConst1,
            0x0B => Opcode::FConst0,
            0x0C => Opcode::FConst1,
            0x0D => Opcode::FConst2,
            0x0E => Opcode::DConst0,
            0x0F => Opcode::DConst1,
            0x10 => {
                let idx = reader.read_i8()?;
                *bytes_read += 1;
                Opcode::BIPush(idx)
            }
            0x11 => {
                let idx = reader.read_i16::<BigEndian>()?;
                *bytes_read += 2;
                Opcode::SIPush(idx)
            }
            0x12 => {
                let idx = reader.read_u8()?; 
                *bytes_read += 1;
                Opcode::Ldc(idx) 
            }
            0x13 => {
                let idx = reader.read_u16::<BigEndian>()?;
                *bytes_read += 2;
                Opcode::LdcW(idx)
            }
            0x14 => {
                let idx = reader.read_u16::<BigEndian>()?;
                *bytes_read += 2;
                Opcode::Ldc2W(idx)
            }

            // Loads
            0x15 => {
                let idx = reader.read_u8()?;
                *bytes_read += 1;
                Opcode::ILoad(idx)
            }
            0x16 => {
                let idx = reader.read_u8()?;
                *bytes_read += 1;
                Opcode::LLoad(idx)
            }
            0x17 => {
                let idx = reader.read_u8()?;
                *bytes_read += 1;
                Opcode::FLoad(idx)
            }
            0x18 => {
                let idx = reader.read_u8()?;
                *bytes_read += 1;
                Opcode::DLoad(idx)
            }
            0x19 => {
                let idx = reader.read_u8()?;
                *bytes_read += 1;
                Opcode::ALoad(idx)
            }
            0x1A => Opcode::ILoad0,
            0x1B => Opcode::ILoad1,
            0x1C => Opcode::ILoad2,
            0x1D => Opcode::ILoad3,
            0x1E => Opcode::LLoad0,
            0x1F => Opcode::LLoad1,
            0x20 => Opcode::LLoad2,
            0x21 => Opcode::LLoad3,
            0x22 => Opcode::FLoad0,
            0x23 => Opcode::FLoad1,
            0x24 => Opcode::FLoad2,
            0x25 => Opcode::FLoad3,
            0x26 => Opcode::DLoad0,
            0x27 => Opcode::DLoad1,
            0x28 => Opcode::DLoad2,
            0x29 => Opcode::DLoad3,
            0x2A => Opcode::ALoad0,
            0x2B => Opcode::ALoad1,
            0x2C => Opcode::ALoad2,
            0x2D => Opcode::ALoad3,
            0x2E => Opcode::IALoad,
            0x2F => Opcode::LALoad,
            0x30 => Opcode::FALoad,
            0x31 => Opcode::DALoad,
            0x32 => Opcode::AALoad,
            0x33 => Opcode::BALoad,
            0x34 => Opcode::CALoad,
            0x35 => Opcode::SALoad,

            /* Stores */
            0x36 => { /* ISTORE */ 
                let idx = reader.read_u8()?;
                *bytes_read += 1;
                Opcode::IStore(idx)
            }
            0x37 => { /* LSTORE */ 
                let idx = reader.read_u8()?;
                *bytes_read += 1;
                Opcode::LStore(idx)
            }
            0x38 => { /* FSTORE */
                let idx = reader.read_u8()?;
                *bytes_read += 1;
                Opcode::FStore(idx)
            }
            0x39 => { /* DSTORE */
                let idx = reader.read_u8()?;
                *bytes_read += 1;
                Opcode::DStore(idx)
            }
            0x3A => { /* ASTORE */
                let idx = reader.read_u8()?;
                *bytes_read += 1;
                Opcode::AStore(idx)
            }

            0x3B => Opcode::IStore0,
            0x3C => Opcode::IStore1,
            0x3D => Opcode::IStore2,
            0x3E => Opcode::IStore3,
            0x3F => Opcode::LStore0,
            0x40 => Opcode::LStore1,
            0x41 => Opcode::LStore2,
            0x42 => Opcode::LStore3,
            0x43 => Opcode::FStore0,
            0x44 => Opcode::FStore1,
            0x45 => Opcode::FStore2,
            0x46 => Opcode::FStore3,
            0x47 => Opcode::DStore0,
            0x48 => Opcode::DStore1,
            0x49 => Opcode::DStore2,
            0x4A => Opcode::DStore3,
            0x4B => Opcode::AStore0,
            0x4C => Opcode::AStore1,
            0x4D => Opcode::AStore2,
            0x4E => Opcode::AStore3,
            0x4F => Opcode::IAStore,
            0x50 => Opcode::LAStore,
            0x51 => Opcode::FAStore,
            0x52 => Opcode::DAStore,
            0x53 => Opcode::AAStore,
            0x54 => Opcode::BAStore,
            0x55 => Opcode::CAStore,
            0x56 => Opcode::SAStore,

            /* Stack */
            0x57 => Opcode::Pop,
            0x58 => Opcode::Pop2,
            0x59 => Opcode::Dup,
            0x5A => Opcode::DupX1,
            0x5B => Opcode::DupX2,
            0x5C => Opcode::Dup2,
            0x5D => Opcode::Dup2X1,
            0x5E => Opcode::Dup2X2,
            0x5F => Opcode::Swap,

            /* Math */
            0x60 => Opcode::IAdd,
            0x61 => Opcode::LAdd,
            0x62 => Opcode::FAdd,
            0x63 => Opcode::DAdd,
            0x64 => Opcode::ISub,
            0x65 => Opcode::LSub,
            0x66 => Opcode::FSub,
            0x67 => Opcode::DSub,
            0x68 => Opcode::IMul,
            0x69 => Opcode::LMul,
            0x6A => Opcode::FMul,
            0x6B => Opcode::DMul,
            0x6C => Opcode::IDiv,
            0x6D => Opcode::LDiv,
            0x6E => Opcode::FDiv,
            0x6F => Opcode::DDiv,
            0x70 => Opcode::IRem,
            0x71 => Opcode::LRem,
            0x72 => Opcode::FRem,
            0x73 => Opcode::DRem,
            0x74 => Opcode::INeg,
            0x75 => Opcode::LNeg,
            0x76 => Opcode::FNeg,
            0x77 => Opcode::DNeg,
            0x78 => Opcode::IShl,
            0x79 => Opcode::LShl,
            0x7A => Opcode::IShr,
            0x7B => Opcode::LShr,
            0x7C => Opcode::IUShr,
            0x7D => Opcode::LUshr,
            0x7E => Opcode::IAnd,
            0x7F => Opcode::LAnd,
            0x80 => Opcode::IOr,
            0x81 => Opcode::LOr,
            0x82 => Opcode::IXor,
            0x83 => Opcode::LXor,
            0x84 => {
                let idx = reader.read_u8()?;
                let constant = reader.read_i8()?;
                *bytes_read += 2;
                Opcode::IInc(idx, constant)
            }

            /* Conversions */
            0x85 => Opcode::I2L,
            0x86 => Opcode::I2F,
            0x87 => Opcode::I2D,
            0x88 => Opcode::L2I,
            0x89 => Opcode::L2F,
            0x8A => Opcode::L2D,
            0x8B => Opcode::F2I,
            0x8C => Opcode::F2L,
            0x8D => Opcode::F2D,
            0x8E => Opcode::D2I,
            0x8F => Opcode::D2L,
            0x90 => Opcode::D2F,
            0x91 => Opcode::I2B,
            0x92 => Opcode::I2C,
            0x93 => Opcode::I2S,

            /* Comparisons */
            0x94 => Opcode::LCmp,
            0x95 => Opcode::FCmpL,
            0x96 => Opcode::FCmpG,
            0x97 => Opcode::DCmpL,
            0x98 => Opcode::DCmpG,
            0x99 => {
                let offset = reader.read_i16::<BigEndian>()?;
                *bytes_read += 2;
                Opcode::IfEq(offset)
            }
            0x9A => {
                let offset = reader.read_i16::<BigEndian>()?;
                *bytes_read += 2;
                Opcode::IfNe(offset)
            }
            0x9B => {
                let offset = reader.read_i16::<BigEndian>()?;
                *bytes_read += 2;
                Opcode::IfLt(offset)
            }
            0x9C => {
                let offset = reader.read_i16::<BigEndian>()?;
                *bytes_read += 2;
                Opcode::IfGe(offset)
            }
            0x9D => {
                let offset = reader.read_i16::<BigEndian>()?;
                *bytes_read += 2;
                Opcode::IfGt(offset)
            }
            0x9E => {
                let offset = reader.read_i16::<BigEndian>()?;
                *bytes_read += 2;
                Opcode::IfLe(offset)
            }
            0x9F => {
                let offset = reader.read_i16::<BigEndian>()?;
                *bytes_read += 2;
                Opcode::IfICmpEq(offset)
            }
            0xA0 => {
                let offset = reader.read_i16::<BigEndian>()?;
                *bytes_read += 2;
                Opcode::IfICmpNe(offset)
            }
            0xA1 => {
                let offset = reader.read_i16::<BigEndian>()?;
                *bytes_read += 2;
                Opcode::IfICmpLt(offset)
            }
            0xA2 => {
                let offset = reader.read_i16::<BigEndian>()?;
                *bytes_read += 2;
                Opcode::IfICmpGe(offset)
            }
            0xA3 => {
                let offset = reader.read_i16::<BigEndian>()?;
                *bytes_read += 2;
                Opcode::IfICmpGt(offset)
            }
            0xA4 => {
                let offset = reader.read_i16::<BigEndian>()?;
                *bytes_read += 2;
                Opcode::IfICmpLe(offset)
            }
            0xA5 => {
                let offset = reader.read_i16::<BigEndian>()?;
                *bytes_read += 2;
                Opcode::IfACmpEq(offset)
            }
            0xA6 => {
                let offset = reader.read_i16::<BigEndian>()?;
                *bytes_read += 2;
                Opcode::IfACmpNe(offset)
            }

            /* Control */
            0xA7 => {
                let offset = reader.read_i16::<BigEndian>()?;
                *bytes_read += 2;
                Opcode::Goto(offset)
            }
            0xA8 => {
                let offset = reader.read_i16::<BigEndian>()?;
                *bytes_read += 2;
                Opcode::Jsr(offset)
            }
            0xA9 => {
                let idx = reader.read_u8()?;
                *bytes_read += 1;
                Opcode::Ret(idx)
            }
            0xAA => {
                let padding = (4 - (*bytes_read % 4)) % 4;
                for _ in 0..padding {
                    reader.read_u8()?;
                    *bytes_read += 1;
                }

                let default = reader.read_i32::<BigEndian>()?;
                let low = reader.read_i32::<BigEndian>()?;
                let high = reader.read_i32::<BigEndian>()?;

                *bytes_read += 12;

                let mut offsets = Vec::with_capacity((high - low + 1) as usize);
                for _ in low..high {
                    offsets.push(reader.read_i32::<BigEndian>()?);
                    *bytes_read += 4;
                }

                Opcode::TableSwitch { default_offset: default, low, high, jump_offsets: offsets }
            }
            0xAB => {
                let padding = (4 - (*bytes_read % 4)) % 4;
                for _ in 0..padding {
                    reader.read_u8()?;
                    *bytes_read += 1;
                }

                let default = reader.read_i32::<BigEndian>()?;
                let npairs = reader.read_i32::<BigEndian>()?;
                *bytes_read += 8;

                let mut pairs = Vec::with_capacity(npairs as usize);
                for _ in 0..npairs {
                    let match_val = reader.read_i32::<BigEndian>()?;
                    let offset = reader.read_i32::<BigEndian>()?;
                    pairs.push((match_val, offset));
                    *bytes_read += 8;
                }

                Opcode::LookupSwitch { default_offset: default, npairs: npairs, match_offsets: pairs }
            }
            0xAC => Opcode::IReturn,
            0xAD => Opcode::LReturn,
            0xAE => Opcode::FReturn,
            0xAF => Opcode::DReturn,
            0xB0 => Opcode::AReturn,
            0xB1 => Opcode::Return,
            
            /* References */
            0xB2 => {
                let index = reader.read_u16::<BigEndian>()?;
                *bytes_read += 2;
                Opcode::GetStatic(index)
            }
            0xB3 => {
                let index = reader.read_u16::<BigEndian>()?;
                *bytes_read += 2;
                Opcode::PutStatic(index)
            }
            0xB4 => {
                let index = reader.read_u16::<BigEndian>()?;
                *bytes_read += 2;
                Opcode::GetField(index)
            }
            0xB5 => {
                let index = reader.read_u16::<BigEndian>()?;
                *bytes_read += 2;
                Opcode::PutField(index)
            }
            0xB6 => {
                let idx = reader.read_u16::<BigEndian>()?;
                *bytes_read += 2;
                Opcode::InvokeVirtual(idx)
            }
            0xB7 => {
                let idx = reader.read_u16::<BigEndian>()?;
                *bytes_read += 2;
                Opcode::InvokeSpecial(idx)
            }
            0xB8 => {
                let idx = reader.read_u16::<BigEndian>()?;
                *bytes_read += 2;
                Opcode::InvokeStatic(idx)
            }
            0xB9 => {
                let idx = reader.read_u16::<BigEndian>()?;
                let count = reader.read_u8()?;
                if reader.read_u8()? == 0 {
                    panic!("Expected 0 after invokeinterface count");
                }
                *bytes_read += 4;
                Opcode::InvokeInterface(idx, count)
            }
            0xBA => {
                let idx = reader.read_u16::<BigEndian>()?;
                *bytes_read += 2;
                Opcode::InvokeDynamic(idx)
            }
            0xBB => {
                let index = reader.read_u16::<BigEndian>()?;
                *bytes_read += 2;
                Opcode::New(index)
            }
            0xBC => {
                let atype = reader.read_u8()?;
                let atype = AType::try_from_primitive(atype).expect("AType after newarray instruction was invalid.");
                *bytes_read += 1;
                Opcode::NewArray(atype)
            }
            0xBD => {
                let index = reader.read_u16::<BigEndian>()?;
                *bytes_read += 2;
                Opcode::ANewArray(index)
            }
            0xBE => Opcode::ArrayLength,
            0xBF => Opcode::AThrow,
            0xC0 => {
                let index = reader.read_u16::<BigEndian>()?;
                *bytes_read += 2;
                Opcode::CheckCast(index)
            }
            0xC1 => {
                let index = reader.read_u16::<BigEndian>()?;
                *bytes_read += 2;
                Opcode::InstanceOf(index)
            }
            0xC2 => Opcode::MonitorEnter,
            0xC3 => Opcode::MonitorExit,

            /* Extended */
            0xC4 => {
                let wide_opcode = reader.read_u8()?;
                *bytes_read += 1;

                let instr = match wide_opcode {
                    // Loads
                    0x15 => {
                        let index = reader.read_u16::<BigEndian>()?;
                        *bytes_read += 2;
                        WideInstruction::ILoad(index)
                    }
                    0x16 => {
                        let index = reader.read_u16::<BigEndian>()?;
                        *bytes_read += 2;
                        WideInstruction::LLoad(index)
                    }
                    0x17 => {
                        let index = reader.read_u16::<BigEndian>()?;
                        *bytes_read += 2;
                        WideInstruction::FLoad(index)
                    }
                    0x18 => {
                        let index = reader.read_u16::<BigEndian>()?;
                        *bytes_read += 2;
                        WideInstruction::DLoad(index)
                    }
                    0x19 => {
                        let index = reader.read_u16::<BigEndian>()?;
                        *bytes_read += 2;
                        WideInstruction::ALoad(index)
                    }

                    // Stores
                    0x36 => {
                        let index = reader.read_u16::<BigEndian>()?;
                        *bytes_read += 2;
                        WideInstruction::IStore(index)
                    }
                    0x37 => {
                        let index = reader.read_u16::<BigEndian>()?;
                        *bytes_read += 2;
                        WideInstruction::LStore(index)
                    }
                    0x38 => {
                        let index = reader.read_u16::<BigEndian>()?;
                        *bytes_read += 2;
                        WideInstruction::FStore(index)
                    }
                    0x39 => {
                        let index = reader.read_u16::<BigEndian>()?;
                        *bytes_read += 2;
                        WideInstruction::DStore(index)
                    }
                    0x3A => {
                        let index = reader.read_u16::<BigEndian>()?;
                        *bytes_read += 2;
                        WideInstruction::AStore(index)
                    }

                    // Increment
                    0x84 => {
                        let index = reader.read_u16::<BigEndian>()?;
                        let constant = reader.read_i16::<BigEndian>()?;
                        *bytes_read += 4;
                        WideInstruction::IInc(index, constant)
                    }

                    _ => panic!("Unsupported opcode after WIDE: {}", wide_opcode),
                };

                Opcode::Wide(instr)
            }
            0xC5 => {
                let index = reader.read_u16::<BigEndian>()?;
                let dimensions = reader.read_u8()?;
                *bytes_read += 3;
                Opcode::MultiANewArray(index, dimensions)
            }
            0xC6 => {
                let offset = reader.read_i16::<BigEndian>()?;
                *bytes_read += 2;
                Opcode::IfNull(offset)
            }
            0xC7 => {
                let offset = reader.read_i16::<BigEndian>()?;
                *bytes_read += 2;
                Opcode::IfNonNull(offset)
            }
            0xC8 => {
                let offset = reader.read_i32::<BigEndian>()?;
                *bytes_read += 4;
                Opcode::GotoW(offset)
            }
            0xC9 => {
                let offset = reader.read_i32::<BigEndian>()?;
                *bytes_read += 4;
                Opcode::JsrW(offset)
            }
            0xCA => Opcode::Breakpoint,

            0xCB..=0xFF => panic!("Unknown opcode {:X}", code),
        };

        Ok(op)
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
