use std::{cell::RefCell, collections::HashMap, error::Error, rc::Rc};

use crate::vm::{class::Class, class_loader::ClassLoader, jobject::{JObject, JObjectKind}, jthread::JThread, jvalue::JValue, opcode::{AType, Opcode}, stack_frame::StackFrame};

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
                Opcode::LALoad => self.laload(frame)?,
                Opcode::FALoad => self.faload(frame)?,
                Opcode::DALoad => self.daload(frame)?,
                Opcode::AALoad => self.aaload(frame)?,
                Opcode::BALoad => self.baload(frame)?,
                Opcode::CALoad => self.caload(frame)?,
                Opcode::SALoad => self.saload(frame)?,

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

                Opcode::IAStore => self.iastore(frame)?,
                Opcode::LAStore => self.lastore(frame)?,
                Opcode::FAStore => self.fastore(frame)?,
                Opcode::DAStore => self.dastore(frame)?,
                Opcode::AAStore => self.aastore(frame)?,
                Opcode::BAStore => self.bastore(frame)?,
                Opcode::CAStore => self.castore(frame)?,
                Opcode::SAStore => self.sastore(frame)?,

                // Stack
                Opcode::Pop => self.pop(frame)?,
                Opcode::Pop2 => self.pop2(frame)?,
                Opcode::Dup => self.dup(frame)?,
                Opcode::DupX1 => self.dup_x1(frame)?,
                Opcode::DupX2 => self.dup_x2(frame)?,
                Opcode::Dup2 => self.dup2(frame)?,
                Opcode::Dup2X1 => self.dup2_x1(frame)?,
                Opcode::Dup2X2 => self.dup2_x2(frame)?,
                Opcode::Swap => self.swap(frame)?,

                // Math
                Opcode::IAdd => self.iadd(frame)?,
                Opcode::LAdd => self.ladd(frame)?,
                Opcode::FAdd => self.fadd(frame)?,
                Opcode::DAdd => self.dadd(frame)?,

                Opcode::ISub => self.isub(frame)?,
                Opcode::LSub => self.lsub(frame)?,
                Opcode::FSub => self.fsub(frame)?,
                Opcode::DSub => self.dsub(frame)?,

                Opcode::IMul => self.imul(frame)?,
                Opcode::LMul => self.lmul(frame)?,
                Opcode::FMul => self.fmul(frame)?,
                Opcode::DMul => self.dmul(frame)?,

                Opcode::IDiv => self.idiv(frame)?,
                Opcode::LDiv => self.ldiv(frame)?,
                Opcode::FDiv => self.fdiv(frame)?,
                Opcode::DDiv => self.ddiv(frame)?,

                Opcode::IRem => self.irem(frame)?,
                Opcode::LRem => self.lrem(frame)?,
                Opcode::FRem => self.frem(frame)?,
                Opcode::DRem => self.drem(frame)?,

                Opcode::INeg => self.ineg(frame)?,
                Opcode::LNeg => self.lneg(frame)?,
                Opcode::FNeg => self.fneg(frame)?,
                Opcode::DNeg => self.dneg(frame)?,

                Opcode::IShl => self.ishl(frame)?,
                Opcode::LShl => self.lshl(frame)?,
                Opcode::IShr => self.ishr(frame)?,
                Opcode::LShr => self.lshr(frame)?,
                Opcode::IUShr => self.iushr(frame)?,
                Opcode::LUshr => self.lushr(frame)?,

                Opcode::IAnd => self.iand(frame)?,
                Opcode::LAnd => self.land(frame)?,
                Opcode::IOr => self.ior(frame)?,
                Opcode::LOr => self.lor(frame)?,
                Opcode::IXor => self.ixor(frame)?,
                Opcode::LXor => self.lxor(frame)?,

                Opcode::IInc(index, constant) => self.iinc(frame, index, constant)?,

                // Conversions
                Opcode::I2L => self.i2l(frame)?,
                Opcode::I2F => self.i2f(frame)?,
                Opcode::I2D => self.i2d(frame)?,

                Opcode::L2I => self.l2i(frame)?,
                Opcode::L2F => self.l2f(frame)?,
                Opcode::L2D => self.l2d(frame)?,

                Opcode::F2I => self.f2i(frame)?,
                Opcode::F2L => self.f2l(frame)?,
                Opcode::F2D => self.f2d(frame)?,

                Opcode::D2I => self.d2i(frame)?,
                Opcode::D2L => self.d2l(frame)?,
                Opcode::D2F => self.d2f(frame)?,

                Opcode::I2B => self.i2b(frame)?,
                Opcode::I2C => self.i2c(frame)?,
                Opcode::I2S => self.i2s(frame)?,

                // Comparisons
                Opcode::LCmp => self.lcmp(frame)?,
                Opcode::FCmpL => self.fcmpl(frame)?,
                Opcode::FCmpG => self.fcmpg(frame)?,
                Opcode::DCmpL => self.dcmpl(frame)?,
                Opcode::DCmpG => self.dcmpg(frame)?,
                // Control

                // References
                Opcode::New(index) => self.new_op(frame, index)?,
                Opcode::NewArray(ty) => self.newarray(frame, ty)?,
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
        let object = Rc::new(RefCell::new(JObject::new(class)));
        frame.operand_stack.push(JValue::Reference(object));
        Ok(())
    }
    
    fn newarray(&mut self, frame: &mut StackFrame, ty: AType) -> JVMResult {
        let count = match frame.operand_stack.pop().unwrap() {
            JValue::Int(count) => count,
            other => panic!("newarray expected int, received {:?}", other)
        };

        let reference = JObject::new_primitive_array(ty, count);
        frame.operand_stack.push(JValue::Reference(Rc::new(RefCell::new(reference))));
        Ok(())
    }

    /* End References */

    /* Control */

    /* End Control */

    /* Comparisons */

    fn lcmp(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Long(value1), JValue::Long(value2)) => frame.operand_stack.push(JValue::Int(value1.cmp(&value2) as i32)),
            (other1, other2) => panic!("lcmp expected both values to be long, received {:?} and {:?}", other1, other2)
        }

        Ok(())
    }

    fn fcmpl(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Float(value1), JValue::Float(value2)) => {
                let result = if value1.is_nan() || value2.is_nan() {
                    -1
                } else {
                    value1.partial_cmp(&value2).unwrap() as i32
                };

                frame.operand_stack.push(JValue::Int(result));
            }
            (other1, other2) => panic!("fcmpl expected both values to be float, received {:?} and {:?}", other1, other2)
        }

        Ok(())
    }

    fn fcmpg(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Float(value1), JValue::Float(value2)) => {
                let result = if value1.is_nan() || value2.is_nan() {
                    1
                } else {
                    value1.partial_cmp(&value2).unwrap() as i32
                };

                frame.operand_stack.push(JValue::Int(result));
            }
            (other1, other2) => panic!("fcmpg expected both values to be float, received {:?} and {:?}", other1, other2)
        }

        Ok(())
    }

    fn dcmpl(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Double(value1), JValue::Double(value2)) => {
                let result = if value1.is_nan() || value2.is_nan() {
                    -1
                } else {
                    value1.partial_cmp(&value2).unwrap() as i32
                };

                frame.operand_stack.push(JValue::Int(result));
            }
            (other1, other2) => panic!("dcmpl expected both values to be double, received {:?} and {:?}", other1, other2)
        }

        Ok(())
    }

    fn dcmpg(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Double(value1), JValue::Double(value2)) => {
                let result = if value1.is_nan() || value2.is_nan() {
                    1
                } else {
                    value1.partial_cmp(&value2).unwrap() as i32
                };

                frame.operand_stack.push(JValue::Int(result));
            }
            (other1, other2) => panic!("dcmpg expected both values to be double, received {:?} and {:?}", other1, other2)
        }

        Ok(())
    }

    fn ifeq(&mut self, frame: &mut StackFrame, offset: i16) -> JVMResult {
        match frame.operand_stack.pop().unwrap() {
            JValue::Int(value) => {
                if value == 0 {
                    frame.pc.checked_add_signed(offset as isize).unwrap();
                }
            }
            other => panic!("ifeq expected int, received {:?}", other)
        }

        Ok(())
    }

    /* End Comparisons */

    /* Conversions */

    fn i2l(&mut self, frame: &mut StackFrame) -> JVMResult {
        match frame.operand_stack.pop().unwrap() {
            JValue::Int(value) => frame.operand_stack.push(JValue::Long(value as i64)),
            other => panic!("i2l expected int, received {:?}", other)
        }

        Ok(())
    }

    fn i2f(&mut self, frame: &mut StackFrame) -> JVMResult {
        match frame.operand_stack.pop().unwrap() {
            JValue::Int(value) => frame.operand_stack.push(JValue::Float(value as f32)),
            other => panic!("i2f expected int, received {:?}", other)
        }

        Ok(())
    }

    fn i2d(&mut self, frame: &mut StackFrame) -> JVMResult {
        match frame.operand_stack.pop().unwrap() {
            JValue::Int(value) => frame.operand_stack.push(JValue::Double(value as f64)),
            other => panic!("i2d expected int, received {:?}", other)
        }

        Ok(())
    }

    fn l2i(&mut self, frame: &mut StackFrame) -> JVMResult {
        match frame.operand_stack.pop().unwrap() {
            JValue::Long(value) => frame.operand_stack.push(JValue::Int(value as i32)),
            other => panic!("l2i expected long, received {:?}", other)
        }

        Ok(())
    }

    fn l2f(&mut self, frame: &mut StackFrame) -> JVMResult {
        match frame.operand_stack.pop().unwrap() {
            JValue::Long(value) => frame.operand_stack.push(JValue::Float(value as f32)),
            other => panic!("l2f expected long, received {:?}", other)
        }

        Ok(())
    }

    fn l2d(&mut self, frame: &mut StackFrame) -> JVMResult {
        match frame.operand_stack.pop().unwrap() {
            JValue::Long(value) => frame.operand_stack.push(JValue::Double(value as f64)),
            other => panic!("l2d expected long, received {:?}", other)
        }

        Ok(())
    }

    fn f2i(&mut self, frame: &mut StackFrame) -> JVMResult {
        match frame.operand_stack.pop().unwrap() {
            JValue::Float(value) => frame.operand_stack.push(JValue::Int(value as i32)),
            other => panic!("f2i expected float, received {:?}", other)
        }

        Ok(())
    }

    fn f2l(&mut self, frame: &mut StackFrame) -> JVMResult {
        match frame.operand_stack.pop().unwrap() {
            JValue::Float(value) => frame.operand_stack.push(JValue::Long(value as i64)),
            other => panic!("f2l expected float, received {:?}", other)
        }

        Ok(())
    }

    fn f2d(&mut self, frame: &mut StackFrame) -> JVMResult {
        match frame.operand_stack.pop().unwrap() {
            JValue::Float(value) => frame.operand_stack.push(JValue::Double(value as f64)),
            other => panic!("f2i expected float, received {:?}", other)
        }

        Ok(())
    }

    fn d2i(&mut self, frame: &mut StackFrame) -> JVMResult {
        match frame.operand_stack.pop().unwrap() {
            JValue::Double(value) => frame.operand_stack.push(JValue::Int(value as i32)),
            other => panic!("d2i expected double, received {:?}", other)
        }

        Ok(())
    }

    fn d2l(&mut self, frame: &mut StackFrame) -> JVMResult {
        match frame.operand_stack.pop().unwrap() {
            JValue::Double(value) => frame.operand_stack.push(JValue::Long(value as i64)),
            other => panic!("d2l expected double, received {:?}", other)
        }

        Ok(())
    }

    fn d2f(&mut self, frame: &mut StackFrame) -> JVMResult {
        match frame.operand_stack.pop().unwrap() {
            JValue::Double(value) => frame.operand_stack.push(JValue::Float(value as f32)),
            other => panic!("d2f expected double, received {:?}", other)
        }

        Ok(())
    }

    fn i2b(&mut self, frame: &mut StackFrame) -> JVMResult {
        match frame.operand_stack.pop().unwrap() {
            JValue::Int(value) => frame.operand_stack.push(JValue::Byte(value as i8)),
            other => panic!("i2b expected int, received {:?}", other)
        }

        Ok(())
    }

    fn i2c(&mut self, frame: &mut StackFrame) -> JVMResult {
        match frame.operand_stack.pop().unwrap() {
            JValue::Int(value) => frame.operand_stack.push(JValue::Char(value as u16)),
            other => panic!("i2c expected int, received {:?}", other)
        }

        Ok(())
    }

    fn i2s(&mut self, frame: &mut StackFrame) -> JVMResult {
        match frame.operand_stack.pop().unwrap() {
            JValue::Int(value) => frame.operand_stack.push(JValue::Short(value as i16)),
            other => panic!("i2s expected int, received {:?}", other)
        }

        Ok(())
    }
    

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

    fn imul(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Int(value1), JValue::Int(value2)) => frame.operand_stack.push(JValue::Int(value1 * value2)),
            (other1, other2) => panic!("imul expected both values to be int, received {:?} * {:?}", other1, other2)
        }

        Ok(())
    }

    fn lmul(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Long(value1), JValue::Long(value2)) => frame.operand_stack.push(JValue::Long(value1 * value2)),
            (other1, other2) => panic!("lmul expected both values to be long, received {:?} * {:?}", other1, other2)
        }

        Ok(())
    }

    fn fmul(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Float(value1), JValue::Float(value2)) => frame.operand_stack.push(JValue::Float(value1 * value2)),
            (other1, other2) => panic!("fmul expected both values to be float, received {:?} * {:?}", other1, other2)
        }

        Ok(())
    }

    fn dmul(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Double(value1), JValue::Double(value2)) => frame.operand_stack.push(JValue::Double(value1 * value2)),
            (other1, other2) => panic!("dmul expected both values to be double, received {:?} * {:?}", other1, other2)
        }

        Ok(())
    }

    fn idiv(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Int(value1), JValue::Int(value2)) => frame.operand_stack.push(JValue::Int(value1 / value2)),
            (other1, other2) => panic!("idiv expected both values to be int, received {:?} / {:?}", other1, other2)
        }

        Ok(())
    }

    fn ldiv(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Long(value1), JValue::Long(value2)) => frame.operand_stack.push(JValue::Long(value1 / value2)),
            (other1, other2) => panic!("ldiv expected both values to be long, received {:?} / {:?}", other1, other2)
        }

        Ok(())
    }

    fn fdiv(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Float(value1), JValue::Float(value2)) => frame.operand_stack.push(JValue::Float(value1 / value2)),
            (other1, other2) => panic!("fdiv expected both values to be float, received {:?} / {:?}", other1, other2)
        }

        Ok(())
    }

    fn ddiv(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Double(value1), JValue::Double(value2)) => frame.operand_stack.push(JValue::Double(value1 / value2)),
            (other1, other2) => panic!("ddiv expected both values to be double, received {:?} / {:?}", other1, other2)
        }

        Ok(())
    }

    fn irem(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Int(value1), JValue::Int(value2)) => frame.operand_stack.push(JValue::Int(value1 % value2)),
            (other1, other2) => panic!("irem expected both values to be int, received {:?} % {:?}", other1, other2)
        }

        Ok(())
    }

    fn lrem(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Long(value1), JValue::Long(value2)) => frame.operand_stack.push(JValue::Long(value1 % value2)),
            (other1, other2) => panic!("lrem expected both values to be long, received {:?} % {:?}", other1, other2)
        }

        Ok(())
    }

    fn frem(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Float(value1), JValue::Float(value2)) => frame.operand_stack.push(JValue::Float(value1 % value2)),
            (other1, other2) => panic!("frem expected both values to be float, received {:?} % {:?}", other1, other2)
        }

        Ok(())
    }

    fn drem(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Double(value1), JValue::Double(value2)) => frame.operand_stack.push(JValue::Double(value1 % value2)),
            (other1, other2) => panic!("drem expected both values to be int, received {:?} % {:?}", other1, other2)
        }

        Ok(())
    }

    fn ineg(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value = frame.operand_stack.pop().unwrap();
        match value {
            JValue::Int(value) => frame.operand_stack.push(JValue::Int(-value)),
            other => panic!("ineg expected int, received {:?}", other)
        }

        Ok(())
    }

    fn lneg(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value = frame.operand_stack.pop().unwrap();
        match value {
            JValue::Long(value) => frame.operand_stack.push(JValue::Long(-value)),
            other => panic!("lneg expected long, received {:?}", other)
        }

        Ok(())
    }

    fn fneg(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value = frame.operand_stack.pop().unwrap();
        match value {
            JValue::Float(value) => frame.operand_stack.push(JValue::Float(-value)),
            other => panic!("fneg expected float, received {:?}", other)
        }

        Ok(())
    }

    fn dneg(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value = frame.operand_stack.pop().unwrap();
        match value {
            JValue::Double(value) => frame.operand_stack.push(JValue::Double(-value)),
            other => panic!("dneg expected double, received {:?}", other)
        }

        Ok(())
    }

    fn ishl(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Int(value1), JValue::Int(value2)) => {
                let result = value1 << (value2 & 0b11111);
                frame.operand_stack.push(JValue::Int(result));
            }
            (other1, other2) => panic!("ishl expected both values to be int, received {:?} << {:?}", other1, other2)
        }

        Ok(())
    }

    fn lshl(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Long(value1), JValue::Int(value2)) => {
                let result = value1 << (value2 & 0b111111);
                frame.operand_stack.push(JValue::Long(result));
            }
            (other1, other2) => panic!("lshl expected value1 to be long, value2 to be int, received {:?} << {:?}", other1, other2)
        }

        Ok(())
    }

    fn ishr(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Int(value1), JValue::Int(value2)) => {
                let result = value1 >> (value2 & 0b11111);
                frame.operand_stack.push(JValue::Int(result));
            }
            (other1, other2) => panic!("ishr expected both values to be int, received {:?} >> {:?}", other1, other2)
        }

        Ok(())
    }

    fn lshr(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Long(value1), JValue::Int(value2)) => {
                let result = value1 >> (value2 & 0b111111);
                frame.operand_stack.push(JValue::Long(result));
            }
            (other1, other2) => panic!("rshr expected value1 to be long, value2 to be int, received {:?} >> {:?}", other1, other2)
        }

        Ok(())
    }

    fn iushr(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Int(value1), JValue::Int(value2)) => {
                let shift = (value2 & 0b11111) as u32;
                let result = (value1 as u32) >> shift;
                frame.operand_stack.push(JValue::Int(result as i32));
            }
            (other1, other2) => panic!("iushr expected both values to be int, received {:?} >> {:?}", other1, other2)
        }

        Ok(())
    }

    fn lushr(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Long(value1), JValue::Int(value2)) => {
                let shift = (value2 & 0b111111) as u32;
                let result = (value1 as u32) >> shift;
                frame.operand_stack.push(JValue::Int(result as i32));
            }
            (other1, other2) => panic!("lushr expected value1 to be long, value2 to be int, received {:?} >> {:?}", other1, other2)
        }

        Ok(())
    }

    fn iand(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();

        match (value1, value2) {
            (JValue::Int(value1), JValue::Int(value2)) => frame.operand_stack.push(JValue::Int(value1 & value2)),
            (other1, other2) => panic!("iand expected both values to be int, received {:?} & {:?}", other1, other2)
        }

        Ok(())
    }

    fn land(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();

        match (value1, value2) {
            (JValue::Long(value1), JValue::Long(value2)) => frame.operand_stack.push(JValue::Long(value1 & value2)),
            (other1, other2) => panic!("land expected both values to be long, received {:?} & {:?}", other1, other2)
        }

        Ok(())
    }

    fn ior(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();

        match (value1, value2) {
            (JValue::Int(value1), JValue::Int(value2)) => frame.operand_stack.push(JValue::Int(value1 | value2)),
            (other1, other2) => panic!("ior expected both values to be int, received {:?} | {:?}", other1, other2)
        }

        Ok(())
    }

    fn lor(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();

        match (value1, value2) {
            (JValue::Long(value1), JValue::Long(value2)) => frame.operand_stack.push(JValue::Long(value1 | value2)),
            (other1, other2) => panic!("lor expected both values to be long, received {:?} | {:?}", other1, other2)
        }

        Ok(())
    }

    fn ixor(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();

        match (value1, value2) {
            (JValue::Int(value1), JValue::Int(value2)) => frame.operand_stack.push(JValue::Int(value1 ^ value2)),
            (other1, other2) => panic!("ixor expected both values to be int, received {:?} ^ {:?}", other1, other2)
        }

        Ok(())
    }

    fn lxor(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();

        match (value1, value2) {
            (JValue::Long(value1), JValue::Long(value2)) => frame.operand_stack.push(JValue::Long(value1 ^ value2)),
            (other1, other2) => panic!("lxor expected both values to be long, received {:?} ^ {:?}", other1, other2)
        }

        Ok(())
    }

    fn iinc(&mut self, frame: &mut StackFrame, index: u8, constant: i8) -> JVMResult {
        match &mut frame.locals[index as usize] {
            JValue::Int(value) => *value += constant as i32,
            other => panic!("iinc expected int local, found {:?}", other)
        }

        Ok(())
    }

    /* End Math */

    /* Stack */

    fn pop(&mut self, frame: &mut StackFrame) -> JVMResult {
        frame.operand_stack.pop().unwrap();
        Ok(())
    }

    fn pop2(&mut self, frame: &mut StackFrame) -> JVMResult {
        if frame.operand_stack.last().unwrap().is_category2() {
            frame.operand_stack.pop().unwrap();
        } else {
            frame.operand_stack.pop().unwrap();
            frame.operand_stack.pop().unwrap();
        }

        Ok(())
    }

    fn dup(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value = frame.operand_stack.last().unwrap().clone();
        frame.operand_stack.push(value);
        Ok(())
    }

    fn dup_x1(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value1 = frame.operand_stack.pop().unwrap();
        let value2 = frame.operand_stack.pop().unwrap();
        frame.operand_stack.push(value1.clone());
        frame.operand_stack.push(value2);
        frame.operand_stack.push(value1);
        Ok(())
    }

    fn dup_x2(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value1 = frame.operand_stack.pop().unwrap();
        let value2 = frame.operand_stack.pop().unwrap();
        if value2.is_category2() {
            frame.operand_stack.push(value1.clone());
            frame.operand_stack.push(value2);
            frame.operand_stack.push(value1);
        } else {
            let value3 = frame.operand_stack.pop().unwrap();
            frame.operand_stack.push(value1.clone());
            frame.operand_stack.push(value3);
            frame.operand_stack.push(value2);
            frame.operand_stack.push(value1);
        }

        Ok(())
    }

    fn dup2(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value1 = frame.operand_stack.pop().unwrap();

        if value1.is_category2() {
            frame.operand_stack.push(value1.clone());
            frame.operand_stack.push(value1);
        } else {
            let value2 = frame.operand_stack.pop().unwrap();
            frame.operand_stack.push(value2.clone());
            frame.operand_stack.push(value1.clone());
            frame.operand_stack.push(value2);
            frame.operand_stack.push(value1);
        }

        Ok(())
    }

    fn dup2_x1(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value1 = frame.operand_stack.pop().unwrap();
        if value1.is_category2() {
            let value2 = frame.operand_stack.pop().unwrap();
            frame.operand_stack.push(value1.clone());
            frame.operand_stack.push(value2);
            frame.operand_stack.push(value1);
        } else {
            let value2 = frame.operand_stack.pop().unwrap();
            let value3 = frame.operand_stack.pop().unwrap();
            frame.operand_stack.push(value2.clone());
            frame.operand_stack.push(value1.clone());
            frame.operand_stack.push(value3);
            frame.operand_stack.push(value2);
            frame.operand_stack.push(value1);
        }

        Ok(())
    }

    fn dup2_x2(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value1 = frame.operand_stack.pop().unwrap();
        let value2 = frame.operand_stack.pop().unwrap();
        if value1.is_category2() && value2.is_category2() {
            frame.operand_stack.push(value1.clone());
            frame.operand_stack.push(value2);
            frame.operand_stack.push(value1);
        } else {
            let value3 = frame.operand_stack.pop().unwrap();
            if value3.is_category2() {
                if value2.is_category2() {
                    frame.operand_stack.push(value1.clone());
                    frame.operand_stack.push(value3);
                    frame.operand_stack.push(value2);
                    frame.operand_stack.push(value1);
                } else {
                    frame.operand_stack.push(value2.clone());
                    frame.operand_stack.push(value1.clone());
                    frame.operand_stack.push(value3);
                    frame.operand_stack.push(value2);
                    frame.operand_stack.push(value1);
                }
            } else {
                let value4 = frame.operand_stack.pop().unwrap();
                frame.operand_stack.push(value2.clone());
                frame.operand_stack.push(value1.clone());
                frame.operand_stack.push(value4);
                frame.operand_stack.push(value3);
                frame.operand_stack.push(value2);
                frame.operand_stack.push(value1);
            }
        }

        Ok(())
    }

    fn swap(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value1 = frame.operand_stack.pop().unwrap();
        let value2 = frame.operand_stack.pop().unwrap();
        frame.operand_stack.push(value1);
        frame.operand_stack.push(value2);
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

    fn iastore(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value = match frame.operand_stack.pop().unwrap() {
            JValue::Int(value) => value,
            other => panic!("iastore instruction expected int, received {:?}", other)
        };

        let index = match frame.operand_stack.pop().unwrap() {
            JValue::Int(value) => value,
            other => panic!("iastore instruction expected index of type int, received {:?}", other)
        };

        match frame.operand_stack.pop().unwrap() {
            JValue::Reference(obj) => {
                let mut obj = obj.borrow_mut();
                match &mut obj.kind {
                    JObjectKind::ArrayInt(array) => array[index as usize] = value,
                    other => panic!("iastore instruction expected int array, received {:?}", other)
                }
            }
            other => panic!("iastore instruction expected reference, received {:?}", other)
        }

        Ok(())
    }

    fn lastore(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value = match frame.operand_stack.pop().unwrap() {
            JValue::Long(value) => value,
            other => panic!("lastore instruction expected long, received {:?}", other)
        };

        let index = match frame.operand_stack.pop().unwrap() {
            JValue::Int(value) => value,
            other => panic!("lastore instruction expected index of type int, received {:?}", other)
        };

        match frame.operand_stack.pop().unwrap() {
            JValue::Reference(obj) => {
                let mut obj = obj.borrow_mut();
                match &mut obj.kind {
                    JObjectKind::ArrayLong(array) => array[index as usize] = value,
                    other => panic!("lastore instruction expected long array, received {:?}", other)
                }
            }
            other => panic!("lastore instruction expected reference, received {:?}", other)
        }

        Ok(())
    }

    fn fastore(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value = match frame.operand_stack.pop().unwrap() {
            JValue::Float(value) => value,
            other => panic!("fastore instruction expected float, received {:?}", other)
        };

        let index = match frame.operand_stack.pop().unwrap() {
            JValue::Int(value) => value,
            other => panic!("fastore instruction expected index of type int, received {:?}", other)
        };

        match frame.operand_stack.pop().unwrap() {
            JValue::Reference(obj) => {
                let mut obj = obj.borrow_mut();
                match &mut obj.kind {
                    JObjectKind::ArrayFloat(array) => array[index as usize] = value,
                    other => panic!("fastore instruction expected float array, received {:?}", other)
                }
            }
            other => panic!("fastore instruction expected reference, received {:?}", other)
        }

        Ok(())
    }

    fn dastore(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value = match frame.operand_stack.pop().unwrap() {
            JValue::Double(value) => value,
            other => panic!("dastore instruction expected double, received {:?}", other)
        };

        let index = match frame.operand_stack.pop().unwrap() {
            JValue::Int(value) => value,
            other => panic!("dastore instruction expected index of type int, received {:?}", other)
        };

        match frame.operand_stack.pop().unwrap() {
            JValue::Reference(obj) => {
                let mut obj = obj.borrow_mut();
                match &mut obj.kind {
                    JObjectKind::ArrayDouble(array) => array[index as usize] = value,
                    other => panic!("dastore instruction expected double array, received {:?}", other)
                }
            }
            other => panic!("dastore instruction expected reference, received {:?}", other)
        }

        Ok(())
    }

    fn aastore(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value = match frame.operand_stack.pop().unwrap() {
            JValue::Reference(value) => value,
            other => panic!("aastore instruction expected reference, received {:?}", other)
        };

        let index = match frame.operand_stack.pop().unwrap() {
            JValue::Int(value) => value,
            other => panic!("aastore instruction expected index of type int, received {:?}", other)
        };

        match frame.operand_stack.pop().unwrap() {
            JValue::Reference(obj) => {
                let mut obj = obj.borrow_mut();
                match &mut obj.kind {
                    JObjectKind::ArrayRef(array) => array[index as usize] = value,
                    other => panic!("aastore instruction expected reference array, received {:?}", other)
                }
            }
            other => panic!("aastore instruction expected reference, received {:?}", other)
        }

        Ok(())
    }

    fn bastore(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value = match frame.operand_stack.pop().unwrap() {
            JValue::Int(value) => value,
            other => panic!("bastore instruction expected int, received {:?}", other)
        };

        let index = match frame.operand_stack.pop().unwrap() {
            JValue::Int(value) => value,
            other => panic!("bastore instruction expected index of type int, received {:?}", other)
        };

        match frame.operand_stack.pop().unwrap() {
            JValue::Reference(obj) => {
                let mut obj = obj.borrow_mut();
                match &mut obj.kind {
                    JObjectKind::ArrayBoolean(array) => {
                        array[index as usize] = value & 1 == 1;
                    },
                    JObjectKind::ArrayByte(array) => {
                        array[index as usize] = value as i8;
                    }
                    other => panic!("bastore instruction expected byte or boolean array, received {:?}", other)
                }
            }
            other => panic!("bastore instruction expected reference, received {:?}", other)
        }

        Ok(())
    }

    fn castore(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value = match frame.operand_stack.pop().unwrap() {
            JValue::Int(value) => value,
            other => panic!("castore instruction expected int, received {:?}", other)
        };

        let index = match frame.operand_stack.pop().unwrap() {
            JValue::Int(value) => value,
            other => panic!("castore instruction expected index of type int, received {:?}", other)
        };

        match frame.operand_stack.pop().unwrap() {
            JValue::Reference(obj) => {
                let mut obj = obj.borrow_mut();
                match &mut obj.kind {
                    JObjectKind::ArrayChar(array) => array[index as usize] = value as u16,
                    other => panic!("castore instruction expected char array, received {:?}", other)
                }
            }
            other => panic!("castore instruction expected reference, received {:?}", other)
        }

        Ok(())
    }

    fn sastore(&mut self, frame: &mut StackFrame) -> JVMResult {
        let value = match frame.operand_stack.pop().unwrap() {
            JValue::Int(value) => value,
            other => panic!("sastore instruction expected int, received {:?}", other)
        };

        let index = match frame.operand_stack.pop().unwrap() {
            JValue::Int(value) => value,
            other => panic!("sastore instruction expected index of type int, received {:?}", other)
        };

        match frame.operand_stack.pop().unwrap() {
            JValue::Reference(obj) => {
                let mut obj = obj.borrow_mut();
                match &mut obj.kind {
                    JObjectKind::ArrayShort(array) => array[index as usize] = value as i16,
                    other => panic!("sastore instruction expected short array, received {:?}", other)
                }
            }
            other => panic!("sastore instruction expected reference, received {:?}", other)
        }

        Ok(())
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

    fn iaload(&mut self, frame: &mut StackFrame) -> JVMResult {
        let index = match frame.operand_stack.pop().unwrap() {
            JValue::Int(i) => i,
            other => panic!("iaload expected int, got {:?}", other)
        };

        match frame.operand_stack.pop().unwrap() {
            JValue::Reference(object) => {
                let object = object.borrow();
                match &object.kind {
                    JObjectKind::ArrayInt(array) => frame.operand_stack.push(JValue::Int(array[index as usize])),
                    other => panic!("iaload expected int array, got {:?}", other)
                }
            }
            other => panic!("iaload expected reference, got {:?}", other),
        };

        Ok(())
    }

    fn laload(&mut self, frame: &mut StackFrame) -> JVMResult {
        let index = match frame.operand_stack.pop().unwrap() {
            JValue::Int(i) => i,
            other => panic!("laload expected int, got {:?}", other)
        };

        match frame.operand_stack.pop().unwrap() {
            JValue::Reference(object) => {
                let object = object.borrow();
                match &object.kind {
                    JObjectKind::ArrayLong(array) => frame.operand_stack.push(JValue::Long(array[index as usize])),
                    other => panic!("laload expected long array, got {:?}", other)
                }
            }
            other => panic!("laload expected reference, got {:?}", other),
        };

        Ok(())
    }

    fn faload(&mut self, frame: &mut StackFrame) -> JVMResult {
        let index = match frame.operand_stack.pop().unwrap() {
            JValue::Int(i) => i,
            other => panic!("faload expected int, got {:?}", other)
        };

        match frame.operand_stack.pop().unwrap() {
            JValue::Reference(object) => {
                let object = object.borrow();
                match &object.kind {
                    JObjectKind::ArrayFloat(array) => frame.operand_stack.push(JValue::Float(array[index as usize])),
                    other => panic!("faload expected float array, got {:?}", other)
                }
            }
            other => panic!("faload expected reference, got {:?}", other),
        };

        Ok(())
    }

    fn daload(&mut self, frame: &mut StackFrame) -> JVMResult {
        let index = match frame.operand_stack.pop().unwrap() {
            JValue::Int(i) => i,
            other => panic!("daload expected int, got {:?}", other)
        };

        match frame.operand_stack.pop().unwrap() {
            JValue::Reference(object) => {
                let object = object.borrow();
                match &object.kind {
                    JObjectKind::ArrayDouble(array) => frame.operand_stack.push(JValue::Double(array[index as usize])),
                    other => panic!("daload expected double array, got {:?}", other)
                }
            }
            other => panic!("daload expected reference, got {:?}", other),
        };

        Ok(())
    }

    fn aaload(&mut self, frame: &mut StackFrame) -> JVMResult {
        let index = match frame.operand_stack.pop().unwrap() {
            JValue::Int(i) => i,
            other => panic!("aaload expected int, got {:?}", other)
        };

        match frame.operand_stack.pop().unwrap() {
            JValue::Reference(object) => {
                let object = object.borrow();
                match &object.kind {
                    JObjectKind::ArrayRef(array) => frame.operand_stack.push(JValue::Reference(array[index as usize].clone())),
                    other => panic!("aaload expected object array, got {:?}", other)
                }
            }
            other => panic!("aaload expected reference, got {:?}", other),
        };

        Ok(())
    }

    fn baload(&mut self, frame: &mut StackFrame) -> JVMResult {
        let index = match frame.operand_stack.pop().unwrap() {
            JValue::Int(i) => i,
            other => panic!("baload expected int, got {:?}", other)
        };

        match frame.operand_stack.pop().unwrap() {
            JValue::Reference(object) => {
                let object = object.borrow();
                match &object.kind {
                    JObjectKind::ArrayBoolean(array) => frame.operand_stack.push(JValue::Int(array[index as usize] as i32)),
                    JObjectKind::ArrayByte(array) => frame.operand_stack.push(JValue::Int(array[index as usize] as i32)),
                    other => panic!("baload expected boolean or byte array, got {:?}", other)
                }
            }
            other => panic!("baload expected reference, got {:?}", other),
        };

        Ok(())
    }

    fn caload(&mut self, frame: &mut StackFrame) -> JVMResult {
        let index = match frame.operand_stack.pop().unwrap() {
            JValue::Int(i) => i,
            other => panic!("caload expected int, got {:?}", other)
        };

        match frame.operand_stack.pop().unwrap() {
            JValue::Reference(object) => {
                let object = object.borrow();
                match &object.kind {
                    JObjectKind::ArrayChar(array) => frame.operand_stack.push(JValue::Int(array[index as usize] as i32)),
                    other => panic!("caload expected char array, got {:?}", other)
                }
            }
            other => panic!("caload expected reference, got {:?}", other),
        };

        Ok(())
    }

    fn saload(&mut self, frame: &mut StackFrame) -> JVMResult {
        let index = match frame.operand_stack.pop().unwrap() {
            JValue::Int(i) => i,
            other => panic!("saload expected int, got {:?}", other)
        };

        match frame.operand_stack.pop().unwrap() {
            JValue::Reference(object) => {
                let object = object.borrow();
                match &object.kind {
                    JObjectKind::ArrayShort(array) => frame.operand_stack.push(JValue::Int(array[index as usize] as i32)),
                    other => panic!("saload expected short array, got {:?}", other)
                }
            }
            other => panic!("saload expected reference, got {:?}", other),
        };

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