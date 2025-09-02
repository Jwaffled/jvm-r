use std::{collections::HashMap, error::Error, rc::Rc};

use crate::vm::{class::Class, class_loader::ClassLoader, jthread::JThread, jvalue::JValue, opcode::Opcode, stack_frame::StackFrame};

pub struct JVM {
    class_loader: ClassLoader,
    classes: HashMap<String, Rc<Class>>,
    threads: Vec<JThread>
}

impl JVM {
    pub fn new() -> Self {
        Self {
            class_loader: ClassLoader::new(),
            classes: HashMap::new(),
            threads: Vec::new(),
        }
    }

    pub fn run_class(&mut self, name: &str) -> Result<(), JVMError> {
        let class = self.class_loader.load_class(name).unwrap();

        println!("{:#?}", class);
        
        self.classes.insert(class.name.clone(), class.clone());

        let main_method = class.methods.get("main:([Ljava/lang/String;)V")
            .ok_or("Main method not found").unwrap()
            .clone();

        let frame = StackFrame {
            class: class.clone(),
            operand_stack: Vec::with_capacity(main_method.max_stack as usize),
            locals: vec![JValue::Null; main_method.max_locals as usize],
            method: main_method,
            pc: 0
        };

        let mut thread = JThread { stack: vec![frame] };

        while let Some(frame) = thread.stack.last_mut() {
            if frame.pc >= frame.method.code.len() {
                println!("Popping frame: {:#?}", frame);
                thread.stack.pop();
                continue;
            }
            let opcode = frame.method.code[frame.pc].clone();
            self.execute_opcode(&mut thread, opcode)?;
        }

        Ok(())
    }

    fn execute_opcode(&mut self, thread: &mut JThread, opcode: Opcode) -> Result<(), JVMError> {
        println!("EXECUTING {:?}", opcode);
        
        if let Some(frame) = thread.stack.last_mut() {
            match opcode {
                Opcode::Nop => {},
                Opcode::AConstNull => self.aconst_null(frame)?,
                _ => {}
            }

            frame.pc += 1;
        }

        Ok(())
        
    }

    fn aconst_null(&mut self, frame: &mut StackFrame) -> Result<(), JVMError> {
        frame.operand_stack.push(JValue::Null);
        Ok(())
    }
}

#[derive(Debug)]
pub enum JVMError {

}