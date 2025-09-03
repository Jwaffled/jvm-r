use std::{collections::HashMap, error::Error, rc::Rc};

use crate::vm::{class::Class, class_loader::ClassLoader, jobject::JObject, jthread::JThread, jvalue::JValue, opcode::Opcode, stack_frame::StackFrame};

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

    pub fn run_class(&mut self, name: &str) -> JVMResult {
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

    fn execute_opcode(&mut self, thread: &mut JThread, opcode: Opcode) -> JVMResult {
        println!("EXECUTING {:?}", opcode);
        
        if let Some(frame) = thread.stack.last_mut() {
            match opcode {
                // Constants
                Opcode::Nop => {},
                Opcode::AConstNull => self.aconst_null(frame)?,
                Opcode::IConstM1 => self.iconst_i(frame, -1)?,
                Opcode::IConst0 => self.iconst_i(frame, 0)?,
                Opcode::IConst1 => self.iconst_i(frame, 1)?,
                Opcode::IConst2 => self.iconst_i(frame, 2)?,
                Opcode::IConst3 => self.iconst_i(frame, 3)?,
                Opcode::IConst4 => self.iconst_i(frame, 4)?,
                Opcode::IConst5 => self.iconst_i(frame, 5)?,
                Opcode::LConst0  => self.lconst_i(frame, 0)?,
                Opcode::LConst1 => self.lconst_i(frame, 1)?,
                Opcode::FConst0 => self.fconst_i(frame, 0.0)?,
                Opcode::FConst1 => self.fconst_i(frame, 1.0)?,
                Opcode::FConst2 => self.fconst_i(frame, 2.0)?,
                Opcode::DConst0 => self.dconst_i(frame, 0.0)?,
                Opcode::DConst1 => self.dconst_i(frame, 1.0)?,
                Opcode::BIPush(byte) => self.bipush(frame, byte)?,
                Opcode::SIPush(short) => self.sipush(frame, short)?,
                Opcode::Ldc(index) => self.ldc(frame, index as u16)?,
                Opcode::LdcW(index) => self.ldc(frame, index)?,
                Opcode::Ldc2W(index) => self.ldc(frame, index)?,

                // Loads
                Opcode::ILoad(index) => self.iload(frame, index)?,
                Opcode::LLoad(index) => self.lload(frame, index)?,
                Opcode::FLoad(index) => self.fload(frame, index)?,
                Opcode::DLoad(index) => self.dload(frame, index)?,
                Opcode::ALoad(index) => self.aload(frame, index)?,

                Opcode::ILoad0 => self.iload(frame, 0)?,
                Opcode::ILoad1 => self.iload(frame, 1)?,
                Opcode::ILoad2 => self.iload(frame, 2)?,
                Opcode::ILoad3 => self.iload(frame, 3)?,

                Opcode::LLoad0 => self.lload(frame, 0)?,
                Opcode::LLoad1 => self.lload(frame, 1)?,
                Opcode::LLoad2 => self.lload(frame, 2)?,
                Opcode::LLoad3 => self.lload(frame, 3)?,

                Opcode::FLoad0 => self.fload(frame, 0)?,
                Opcode::FLoad1 => self.fload(frame, 1)?,
                Opcode::FLoad2 => self.fload(frame, 2)?,
                Opcode::FLoad3 => self.fload(frame, 3)?,

                Opcode::DLoad0 => self.dload(frame, 0)?,
                Opcode::DLoad1 => self.dload(frame, 1)?,
                Opcode::DLoad2 => self.dload(frame, 2)?,
                Opcode::DLoad3 => self.dload(frame, 3)?,

                Opcode::ALoad0 => self.aload(frame, 0)?,
                Opcode::ALoad1 => self.aload(frame, 1)?,
                Opcode::ALoad2 => self.aload(frame, 2)?,
                Opcode::ALoad3 => self.aload(frame, 3)?,

                Opcode::IALoad => self.iaload(frame)?,

                // Stores
                Opcode::IStore(index) => self.istore(frame, index)?,
                Opcode::LStore(index) => self.lstore(frame, index)?,
                Opcode::FStore(index) => self.fstore(frame, index)?,
                Opcode::DStore(index) => self.dstore(frame, index)?,
                Opcode::AStore(index) => self.astore(frame, index)?,

                Opcode::IStore0 => self.istore(frame, 0)?,
                Opcode::IStore1 => self.istore(frame, 1)?,
                Opcode::IStore2 => self.istore(frame, 2)?,
                Opcode::IStore3 => self.istore(frame, 3)?,

                Opcode::LStore0 => self.lstore(frame, 0)?,
                Opcode::LStore1 => self.lstore(frame, 1)?,
                Opcode::LStore2 => self.lstore(frame, 2)?,
                Opcode::LStore3 => self.lstore(frame, 3)?,

                Opcode::FStore0 => self.fstore(frame, 0)?,
                Opcode::FStore1 => self.fstore(frame, 1)?,
                Opcode::FStore2 => self.fstore(frame, 2)?,
                Opcode::FStore3 => self.fstore(frame, 3)?,

                Opcode::DStore0 => self.dstore(frame, 0)?,
                Opcode::DStore1 => self.dstore(frame, 1)?,
                Opcode::DStore2 => self.dstore(frame, 2)?,
                Opcode::DStore3 => self.dstore(frame, 3)?,

                Opcode::AStore0 => self.astore(frame, 0)?,
                Opcode::AStore1 => self.astore(frame, 1)?,
                Opcode::AStore2 => self.astore(frame, 2)?,
                Opcode::AStore3 => self.astore(frame, 3)?,

                // Stack
                Opcode::Dup => self.dup(frame)?,

                // Math
                Opcode::IAdd => self.iadd(frame)?,
                Opcode::LAdd => self.ladd(frame)?,
                Opcode::FAdd => self.fadd(frame)?,
                Opcode::DAdd => self.dadd(frame)?,
                Opcode::ISub => self.isub(frame)?,
                Opcode::LSub => self.lsub(frame)?,
                Opcode::FSub => self.fsub(frame)?,
                Opcode::DSub => self.dsub(frame)?,

                // Conversions

                // Comparisons

                // Control

                // References
                Opcode::New(index) => self.new_op(frame, index)?,
                // Extended

                // Reserved
                _ => {}
            }

            frame.pc += 1;
        }

        Ok(())
        
    }

    /* Extended */

    /* End Extended */

    /* References */

    fn new_op(&mut self, frame: &mut StackFrame, index: u16) -> JVMResult {
        let class_name = frame.class.constant_pool.get_class_name(index);
        let class = self.class_loader.load_class(&class_name).unwrap();
        let object = Rc::new(JObject::new(class));
        frame.operand_stack.push(JValue::Reference(object));
        Ok(())
    }

    /* End References */

    /* Control */

    /* End Control */

    /* Comparisons */

    /* End Comparisons */

    /* Conversions */

    /* End Conversions */

    /* Math */

    fn iadd(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Int(value1), JValue::Int(value2)) => frame.operand_stack.push(JValue::Int(value1 + value2)),
            (other1, other2) => panic!("iadd expected both values to be int, received {:?} + {:?}", other1, other2)
        }

        Ok(())
    }

    fn ladd(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Long(value1), JValue::Long(value2)) => frame.operand_stack.push(JValue::Long(value1 + value2)),
            (other1, other2) => panic!("ladd expected both values to be long, received {:?} + {:?}", other1, other2)
        }

        Ok(())
    }

    fn fadd(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Float(value1), JValue::Float(value2)) => frame.operand_stack.push(JValue::Float(value1 + value2)),
            (other1, other2) => panic!("fadd expected both values to be float, received {:?} + {:?}", other1, other2)
        }

        Ok(())
    }

    fn dadd(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Double(value1), JValue::Double(value2)) => frame.operand_stack.push(JValue::Double(value1 + value2)),
            (other1, other2) => panic!("dadd expected both values to be double, received {:?} + {:?}", other1, other2)
        }

        Ok(())
    }

    fn isub(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Int(value1), JValue::Int(value2)) => frame.operand_stack.push(JValue::Int(value1 - value2)),
            (other1, other2) => panic!("isub expected both values to be int, received {:?} - {:?}", other1, other2)
        }

        Ok(())
    }

    fn lsub(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Long(value1), JValue::Long(value2)) => frame.operand_stack.push(JValue::Long(value1 - value2)),
            (other1, other2) => panic!("lsub expected both values to be long, received {:?} - {:?}", other1, other2)
        }

        Ok(())
    }

    fn fsub(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Float(value1), JValue::Float(value2)) => frame.operand_stack.push(JValue::Float(value1 - value2)),
            (other1, other2) => panic!("fsub expected both values to be float, received {:?} - {:?}", other1, other2)
        }

        Ok(())
    }

    fn dsub(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Double(value1), JValue::Double(value2)) => frame.operand_stack.push(JValue::Double(value1 - value2)),
            (other1, other2) => panic!("dsub expected both values to be double, received {:?} - {:?}", other1, other2)
        }

        Ok(())
    }

    /* End Math */

    /* Stack */

    fn dup(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value = frame.operand_stack.last().unwrap().clone();
        frame.operand_stack.push(value);
        Ok(())
    }

    /* End Stack */

    /* Stores */

    fn istore(&mut self, frame: &mut StackFrame, index: u8) -> JVMResult {
        let value = frame.operand_stack.pop().unwrap();
        match value {
            JValue::Int(value) => frame.locals[index as usize] = JValue::Int(value),
            other => panic!("istore instruction expected int, received {:?}", other)
        }

        Ok(())
    }

    fn lstore(&mut self, frame: &mut StackFrame, index: u8) -> JVMResult {
        let value = frame.operand_stack.pop().unwrap();
        match value {
            JValue::Long(value) => frame.locals[index as usize] = JValue::Long(value),
            other => panic!("lstore instruction expected long, received {:?}", other)
        }

        Ok(())
    }

    fn fstore(&mut self, frame: &mut StackFrame, index: u8) -> JVMResult {
        let value = frame.operand_stack.pop().unwrap();
        match value {
            JValue::Float(value) => frame.locals[index as usize] = JValue::Float(value),
            other => panic!("fstore instruction expected float, received {:?}", other)
        }

        Ok(())
    }

    fn dstore(&mut self, frame: &mut StackFrame, index: u8) -> JVMResult {
        let value = frame.operand_stack.pop().unwrap();
        match value {
            JValue::Double(value) => frame.locals[index as usize] = JValue::Double(value),
            other => panic!("dstore instruction expected double, received {:?}", other)
        }

        Ok(())
    }

    fn astore(&mut self, frame: &mut StackFrame, index: u8) -> JVMResult {
        let value = frame.operand_stack.pop().unwrap();
        match value {
            JValue::Reference(value) => frame.locals[index as usize] = JValue::Reference(value),
            JValue::Null => frame.locals[index as usize] = JValue::Null,
            other => panic!("astore instruction expected reference, received {:?}", other)
        }

        Ok(())
    }

    fn iaload(&mut self, frame: &mut StackFrame) -> JVMResult {
        let index = match frame.operand_stack.pop().unwrap() {
            JValue::Int(i) => i,
            other => panic!("iaload expected int, got {:?}", other)
        };

        let 
    }

    /* End Stores */

    /* Loads */

    fn iload(&mut self, frame: &mut StackFrame, index: u8) -> JVMResult {
        let value = &frame.locals[index as usize];
        match value {
            JValue::Int(value) => frame.operand_stack.push(JValue::Int(*value)),
            other => panic!("iload instruction expected int, received {:?}", other)
        }

        Ok(())
    }

    fn lload(&mut self, frame: &mut StackFrame, index: u8) -> JVMResult {
        let value = &frame.locals[index as usize];
        match value {
            JValue::Long(value) => frame.operand_stack.push(JValue::Long(*value)),
            other => panic!("lload instruction expected long, received {:?}", other)
        }

        Ok(())
    }

    fn fload(&mut self, frame: &mut StackFrame, index: u8) -> JVMResult {
        let value = &frame.locals[index as usize];
        match value {
            JValue::Float(value) => frame.operand_stack.push(JValue::Float(*value)),
            other => panic!("fload instruction expected float, received {:?}", other)
        }

        Ok(())
    }

    fn dload(&mut self, frame: &mut StackFrame, index: u8) -> JVMResult {
        let value = &frame.locals[index as usize];
        match value {
            JValue::Double(value) => frame.operand_stack.push(JValue::Double(*value)),
            other => panic!("dload instruction expected double, received {:?}", other)
        }

        Ok(())
    }

    fn aload(&mut self, frame: &mut StackFrame, index: u8) -> JVMResult {
        let value = &frame.locals[index as usize];
        match value {
            JValue::Reference(value) => frame.operand_stack.push(JValue::Reference(value.clone())),
            JValue::Null => frame.operand_stack.push(JValue::Null),
            other => panic!("aload instruction expected reference, received {:?}", other)
        }

        Ok(())
    }

    /* End Loads */

    /* Constants */

    fn aconst_null(&mut self, frame: &mut StackFrame) -> JVMResult {
        frame.operand_stack.push(JValue::Null);
        Ok(())
    }

    fn iconst_i(&mut self, frame: &mut StackFrame, constant: i32) -> JVMResult {
        frame.operand_stack.push(JValue::Int(constant));
        Ok(())
    }

    fn lconst_i(&mut self, frame: &mut StackFrame, constant: i64) -> JVMResult {
        frame.operand_stack.push(JValue::Long(constant));
        Ok(())
    }

    fn fconst_i(&mut self, frame: &mut StackFrame, constant: f32) -> JVMResult {
        frame.operand_stack.push(JValue::Float(constant));
        Ok(())
    }

    fn dconst_i(&mut self, frame: &mut StackFrame, constant: f64) -> JVMResult {
        frame.operand_stack.push(JValue::Double(constant));
        Ok(())
    }

    fn bipush(&mut self, frame: &mut StackFrame, byte: i8) -> JVMResult {
        frame.operand_stack.push(JValue::Int(byte as i32));
        Ok(())
    }

    fn sipush(&mut self, frame: &mut StackFrame, short: i16) -> JVMResult {
        frame.operand_stack.push(JValue::Int(short as i32));
        Ok(())
    }

    fn ldc(&mut self, frame: &mut StackFrame, index: u16) -> JVMResult {
        let entry = frame.class.constant_pool.resolve_ldc_constant(index as u16);
        frame.operand_stack.push(entry);
        Ok(())
    }

    /* End Constants */
}

type JVMResult = Result<(), JVMError>;

#[derive(Debug)]
pub enum JVMError {

}