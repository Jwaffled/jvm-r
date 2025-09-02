use std::rc::Rc;

use crate::vm::{class::Class, jvalue::JValue, method::Method};

#[derive(Debug)]
pub struct StackFrame {
    pub locals: Vec<JValue>,
    pub operand_stack: Vec<JValue>,
    pub class: Rc<Class>,
    pub method: Rc<Method>,
    pub pc: usize,
}