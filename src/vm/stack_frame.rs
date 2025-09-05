use std::{cell::RefCell, rc::Rc};

use crate::vm::{class::Class, jobject::JObject, jvalue::JValue, method::Method};

#[derive(Debug)]
pub struct StackFrame {
    pub locals: Vec<JValue>,
    pub operand_stack: Vec<JValue>,
    pub class: Rc<Class>,
    pub method: Rc<Method>,
    pub pc: usize,
}

impl StackFrame {
    pub fn pop_int(&mut self) -> i32 {
        match self.operand_stack.pop().unwrap() {
            JValue::Int(value) => value,
            other => panic!("Attempted to pop integer off of stack, received {:?}", other)
        }
    }

    pub fn pop_long(&mut self) -> i64 {
        match self.operand_stack.pop().unwrap() {
            JValue::Long(value) => value,
            other => panic!("Attempted to pop long off of stack, received {:?}", other)
        }
    }

    pub fn pop_float(&mut self) -> f32 {
        match self.operand_stack.pop().unwrap() {
            JValue::Float(value) => value,
            other => panic!("Attempted to pop float off of stack, received {:?}", other)
        }
    }

    pub fn pop_double(&mut self) -> f64 {
        match self.operand_stack.pop().unwrap() {
            JValue::Double(value) => value,
            other => panic!("Attempted to pop double off of stack, received {:?}", other)
        }
    }

    pub fn pop_ref(&mut self) -> Rc<RefCell<JObject>> {
        match self.operand_stack.pop().unwrap() {
            JValue::Reference(value) => value,
            other => panic!("Attempted to pop reference off of stack, received {:?}", other)
        }
    }

    pub fn push_int(&mut self, num: i32) {
        self.operand_stack.push(JValue::Int(num));
    }

    pub fn push_long(&mut self, num: i64) {
        self.operand_stack.push(JValue::Long(num));
    }

    pub fn push_float(&mut self, num: f32) {
        self.operand_stack.push(JValue::Float(num));
    }

    pub fn push_double(&mut self, num: f64) {
        self.operand_stack.push(JValue::Double(num));
    }

    pub fn push_ref(&mut self, reference: Rc<RefCell<JObject>>) {
        self.operand_stack.push(JValue::Reference(reference));
    }
}