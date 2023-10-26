#![allow(unused)]

use std::{
    fs::File,
    io::{self, BufRead, BufReader, Cursor, Error, Read},
};

use bitflags::bitflags;
use byteorder::{BigEndian, ReadBytesExt};
use num_enum::{IntoPrimitive, TryFromPrimitive, TryFromPrimitiveError};

#[derive(Default, Debug)]
pub struct ClassFile {
    magic: u32,
    minor_version: u16,
    major_version: u16,
    constant_pool_count: u16,
    constant_pool: Vec<ConstantPoolInfo>,
    access_flags: ClassAccessFlags,
    this_class: u16,
    super_class: u16,
    interfaces_count: u16,
    interfaces: Vec<ConstantPoolInfo>,
    fields_count: u16,
    fields: Vec<FieldInfo>,
    methods_count: u16,
    methods: Vec<MethodInfo>,
    attributes_count: u16,
    attributes: Vec<AttributeInfo>,
}

#[derive(Debug, Clone)]
enum ConstantPoolInfo {
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
    #[derive(Default, Debug)]
    struct ClassAccessFlags: u16 {
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

    #[derive(Default, Debug)]
    struct FieldAccessFlags: u16 {
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

    #[derive(Default, Debug)]
    struct MethodAccessFlags: u16 {
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
struct FieldInfo {
    access_flags: FieldAccessFlags,
    name_index: u16,
    descriptor_index: u16,
    attributes_count: u16,
    attributes: Vec<AttributeInfo>,
}

#[derive(Debug)]
struct MethodInfo {
    access_flags: MethodAccessFlags,
    name_index: u16,
    descriptor_index: u16,
    attributes_count: u16,
    attributes: Vec<AttributeInfo>,
}

#[derive(Debug)]
struct AttributeInfo {
    attribute_name_index: u16,
    attribute_length: u32,
    info: Attribute,
}

#[derive(Debug)]
struct ExceptionTable {
    start_pc: u16,
    end_pc: u16,
    handler_pc: u16,
    catch_type: u16,
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
    start_pc: u16,
    line_number: u16,
}

#[derive(Debug)]
enum Attribute {
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
        let mut code = vec![0u8; code_length as usize];
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
