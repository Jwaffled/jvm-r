use crate::vm::stack_frame::StackFrame;

pub struct JThread {
    pub stack: Vec<StackFrame>
}