use std::{cell::RefCell, collections::HashMap, io::Cursor, rc::Rc};

use byteorder::{BigEndian, ReadBytesExt};
use num_enum::TryFromPrimitive;

use crate::vm::{class::Class, class_loader::ClassLoader, constant_pool::ResolvedConstant, jobject::{JObject, JObjectKind}, jthread::JThread, jvalue::JValue, method::Method, opcode::{AType, Opcode, WideInstruction}, stack_frame::StackFrame};

pub struct JVM {
    class_loader: ClassLoader,
    classes: HashMap<String, Rc<Class>>,
    interned_strings: HashMap<String, Rc<RefCell<JObject>>>,
    threads: Vec<JThread>
}

impl JVM {
    pub fn new() -> Self {
        Self {
            class_loader: ClassLoader::new(),
            classes: HashMap::new(),
            interned_strings: HashMap::new(),
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

        let frame = StackFrame::new(class.clone(), main_method);

        let mut thread = JThread { stack: vec![frame] };

        loop {
            let (should_pop, opcode_info) = {
                match thread.stack.last() {
                    None => break,
                    Some(frame) => {
                        if frame.pc >= frame.method.code.len() {
                            (true, None)
                        } else {
                            let opcode_info = Self::parse_opcode(&frame.method.code, frame.pc).unwrap();
                            (false, Some((opcode_info, frame.pc)))
                        }
                    }
                }
            };
            
            if should_pop {
                println!("Popping frame: {:#?}", thread.stack.last().unwrap());
                thread.stack.pop();
                continue;
            }
            
            if let Some(((opcode, len), start_pc)) = opcode_info {
                self.execute_opcode(&mut thread, opcode)?;
                if let Some(frame) = thread.stack.last_mut() {
                    if frame.pc == start_pc {
                        frame.pc += len;
                    }
                }
            }
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
                Opcode::IfEq(offset) => self.ifeq(frame, offset)?,
                Opcode::IfNe(offset) => self.ifne(frame, offset)?,
                Opcode::IfLt(offset) => self.iflt(frame, offset)?,
                Opcode::IfGe(offset) => self.ifge(frame, offset)?,
                Opcode::IfGt(offset) => self.ifgt(frame, offset)?,
                Opcode::IfLe(offset) => self.ifle(frame, offset)?,
                Opcode::IfICmpEq(offset) => self.if_icmpeq(frame, offset)?,
                Opcode::IfICmpNe(offset) => self.if_icmpne(frame, offset)?,
                Opcode::IfICmpLt(offset) => self.if_icmplt(frame, offset)?,
                Opcode::IfICmpGe(offset) => self.if_icmpgt(frame, offset)?,
                Opcode::IfICmpGt(offset) => self.if_icmpgt(frame, offset)?,
                Opcode::IfICmpLe(offset) => self.if_icmple(frame, offset)?,
                Opcode::IfACmpEq(offset) => self.if_acmpeq(frame, offset)?,
                Opcode::IfACmpNe(offset) => self.if_acmpne(frame, offset)?,

                // Control
                Opcode::Goto(offset) => self.goto(frame, offset)?,
                Opcode::Jsr(_) => unimplemented!("The jsr instruction is not supported by this JVM. Please recompile with Java 6+"),
                Opcode::Ret(_) => unimplemented!("The ret instruction is not supported by this JVM. Please recompile with Java 6+"),
                Opcode::TableSwitch { default_offset, low, high, jump_offsets } => self.tableswitch(frame, default_offset, low, high, jump_offsets)?,
                Opcode::LookupSwitch { default_offset, npairs, match_offsets } => self.lookupswitch(frame, default_offset, npairs, match_offsets)?,
                Opcode::IReturn => self.ireturn(thread)?,
                Opcode::LReturn => self.lreturn(thread)?,
                Opcode::FReturn => self.freturn(thread)?,
                Opcode::DReturn => self.dreturn(thread)?,
                Opcode::AReturn => self.areturn(thread)?,
                Opcode::Return => self.return_op(thread)?,

                // References
                Opcode::GetField(index) => self.getfield(frame, index)?,
                Opcode::PutField(index) => self.putfield(frame, index)?,
                Opcode::InvokeSpecial(index) => self.invokespecial(thread, index)?,
                Opcode::New(index) => self.new_op(frame, index)?,
                Opcode::NewArray(ty) => self.newarray(frame, ty)?,
                Opcode::ANewArray(index) => self.anewarray(frame, index)?,
                Opcode::ArrayLength => self.arraylength(frame)?,
                // Extended

                // Reserved
                _ => {}
            }
        }

        Ok(())
        
    }

    /* Extended */

    /* End Extended */

    /* References */

    fn getfield(&mut self, frame: &mut StackFrame, index: u16) -> JVMResult {
        let reference = frame.pop_ref();
        let field_ref = frame.class.constant_pool.resolve_constant(index, &mut self.class_loader);
        let field_name = match field_ref {
            ResolvedConstant::FieldRef { field } => format!("{}:{}", field.name, field.descriptor),
            other => panic!("Expected constant entry to be FieldRef, received {:?}", other)
        };

        let value = reference.borrow().get_field(&field_name);
        frame.operand_stack.push(value);
        Ok(())
    }

    fn putfield(&mut self, frame: &mut StackFrame, index: u16) -> JVMResult {
        let value = frame.operand_stack.pop().unwrap();
        let reference = frame.pop_ref();
        let field_ref = frame.class.constant_pool.resolve_constant(index, &mut self.class_loader);
        let field_name = match field_ref {
            ResolvedConstant::FieldRef { field } => format!("{}:{}", field.name, field.descriptor),
            other => panic!("Expected constant entry to be FieldRef, received {:?}", other)
        };

        reference.borrow_mut().set_field(&field_name, value);
        Ok(())
    }

    fn invokevirtual(&mut self, frame: &mut StackFrame, index: u16) -> JVMResult {
        Ok(())
    }

    fn invokespecial(&mut self, thread: &mut JThread, index: u16) -> JVMResult {
        let frame = thread.stack.last_mut().unwrap();
        let method_ref = frame.class.constant_pool.resolve_constant(index, &mut self.class_loader);
        match method_ref {
            ResolvedConstant::MethodRef { method, class } => {
                let (args, ret_ty) = Method::parse_method_descriptor(&method.descriptor);
                let mut arg_values = Vec::with_capacity(args.len());
                for _ in 0..args.len() {
                    arg_values.push(frame.operand_stack.pop().unwrap());
                }
                arg_values.reverse();
                let object = frame.pop_ref();

                let mut new_frame = StackFrame::new(class, method);
                new_frame.locals[0] = JValue::Reference(object);
                for (i, arg) in arg_values.into_iter().enumerate() {
                    new_frame.locals[i + 1] = arg;
                }

                frame.pc += 3;
                thread.stack.push(new_frame);
            }
            other => panic!("Expected constant entry to be MethodRef, received {:?}", other)
        }

        Ok(())
    }

    fn new_op(&mut self, frame: &mut StackFrame, index: u16) -> JVMResult {
        let class_name = frame.class.constant_pool.get_class_name(index);
        let class = self.class_loader.load_class(&class_name).unwrap();
        let reference = JObject::new(class);
        frame.push_ref(reference);
        Ok(())
    }
    
    fn newarray(&mut self, frame: &mut StackFrame, ty: AType) -> JVMResult {
        let count = frame.pop_int();
        let reference = JObject::new_primitive_array(ty, count);
        frame.push_ref(reference);
        Ok(())
    }

    fn anewarray(&mut self, frame: &mut StackFrame, index: u16) -> JVMResult {
        let count = frame.pop_int();
        let class_name = frame.class.constant_pool.get_class_name(index);
        let class = Class::reference_array_class(&class_name);
        let reference = JObject::new_reference_array(class, count);
        frame.push_ref(reference);
        Ok(())
    }

    fn arraylength(&mut self, frame: &mut StackFrame) -> JVMResult {
        let reference = frame.pop_ref();
        let length = reference.borrow().array_length();
        frame.push_int(length);
        Ok(())
    }

    /* End References */

    /* Control */

    fn goto(&mut self, frame: &mut StackFrame, offset: i16) -> JVMResult {
        frame.pc = frame.pc.checked_add_signed(offset as isize).unwrap();
        Ok(())
    }

    fn tableswitch(&mut self, frame: &mut StackFrame, default_offset: i32, low: i32, high: i32, jump_offsets: Vec<i32>) -> JVMResult {
        let index = match frame.operand_stack.pop().unwrap() {
            JValue::Int(value) => value,
            other => panic!("tableswitch instruction expected int, received {:?}", other)
        };

        if index < low || index > high {
            frame.pc = frame.pc.checked_add_signed(default_offset as isize).unwrap();
        } else {
            let value = jump_offsets[(index - low) as usize];
            frame.pc = frame.pc.checked_add_signed(value as isize).unwrap();
        }

        Ok(())
    }

    fn lookupswitch(&mut self, frame: &mut StackFrame, default_offset: i32, npairs: i32, match_offsets: Vec<(i32, i32)>) -> JVMResult {
        let key = match frame.operand_stack.pop().unwrap() {
            JValue::Int(value) => value,
            other => panic!("lookupswitch instruction expected int, received {:?}", other)
        };

        for (value, offset) in match_offsets {
            if key == value {
                frame.pc = frame.pc.checked_add_signed(offset as isize).unwrap();
                return Ok(());
            }
        }

        frame.pc = frame.pc.checked_add_signed(default_offset as isize).unwrap();

        Ok(())
    }

    fn ireturn(&mut self, thread: &mut JThread) -> JVMResult {
        let mut frame = thread.stack.pop().unwrap();

        let value = match frame.operand_stack.pop().unwrap() {
            JValue::Int(value) => value,
            other => panic!("ireturn expected int, received {:?}", other)
        };

        if let Some(caller) = thread.stack.last_mut() {
            caller.operand_stack.push(JValue::Int(value));
        }

        Ok(())
    }

    fn lreturn(&mut self, thread: &mut JThread) -> JVMResult {
        let mut frame = thread.stack.pop().unwrap();

        let value = match frame.operand_stack.pop().unwrap() {
            JValue::Long(value) => value,
            other => panic!("lreturn expected long, received {:?}", other)
        };

        if let Some(caller) = thread.stack.last_mut() {
            caller.operand_stack.push(JValue::Long(value));
        }

        Ok(())
    }

    fn freturn(&mut self, thread: &mut JThread) -> JVMResult {
        let mut frame = thread.stack.pop().unwrap();

        let value = match frame.operand_stack.pop().unwrap() {
            JValue::Float(value) => value,
            other => panic!("freturn expected float, received {:?}", other)
        };

        if let Some(caller) = thread.stack.last_mut() {
            caller.operand_stack.push(JValue::Float(value));
        }

        Ok(())
    }

    fn dreturn(&mut self, thread: &mut JThread) -> JVMResult {
        let mut frame = thread.stack.pop().unwrap();

        let value = match frame.operand_stack.pop().unwrap() {
            JValue::Double(value) => value,
            other => panic!("dreturn expected double, received {:?}", other)
        };

        if let Some(caller) = thread.stack.last_mut() {
            caller.operand_stack.push(JValue::Double(value));
        }

        Ok(())
    }

    fn areturn(&mut self, thread: &mut JThread) -> JVMResult {
        let mut frame = thread.stack.pop().unwrap();

        let value = match frame.operand_stack.pop().unwrap() {
            JValue::Reference(value) => value,
            other => panic!("areturn expected reference, received {:?}", other)
        };

        if let Some(caller) = thread.stack.last_mut() {
            caller.operand_stack.push(JValue::Reference(value));
        }

        Ok(())
    }

    fn return_op(&mut self, thread: &mut JThread) -> JVMResult {
        println!("POPPING {:#?}", thread.stack.last());
        thread.stack.pop().unwrap();
        Ok(())
    }

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
                    frame.pc = frame.pc.checked_add_signed(offset as isize).unwrap();
                }
            }
            other => panic!("ifeq expected int, received {:?}", other)
        }

        Ok(())
    }

    fn ifne(&mut self, frame: &mut StackFrame, offset: i16) -> JVMResult {
        match frame.operand_stack.pop().unwrap() {
            JValue::Int(value) => {
                if value != 0 {
                    frame.pc = frame.pc.checked_add_signed(offset as isize).unwrap();
                }
            }
            other => panic!("ifne expected int, received {:?}", other)
        }

        Ok(())
    }

    fn iflt(&mut self, frame: &mut StackFrame, offset: i16) -> JVMResult {
        match frame.operand_stack.pop().unwrap() {
            JValue::Int(value) => {
                if value < 0 {
                    frame.pc = frame.pc.checked_add_signed(offset as isize).unwrap();
                }
            }
            other => panic!("iflt expected int, received {:?}", other)
        }

        Ok(())
    }

    fn ifge(&mut self, frame: &mut StackFrame, offset: i16) -> JVMResult {
        match frame.operand_stack.pop().unwrap() {
            JValue::Int(value) => {
                if value >= 0 {
                    frame.pc = frame.pc.checked_add_signed(offset as isize).unwrap();
                }
            }
            other => panic!("ifge expected int, received {:?}", other)
        }

        Ok(())
    }

    fn ifgt(&mut self, frame: &mut StackFrame, offset: i16) -> JVMResult {
        match frame.operand_stack.pop().unwrap() {
            JValue::Int(value) => {
                if value > 0 {
                    frame.pc = frame.pc.checked_add_signed(offset as isize).unwrap();
                }
            }
            other => panic!("ifgt expected int, received {:?}", other)
        }

        Ok(())
    }

    fn ifle(&mut self, frame: &mut StackFrame, offset: i16) -> JVMResult {
        match frame.operand_stack.pop().unwrap() {
            JValue::Int(value) => {
                if value <= 0 {
                    frame.pc = frame.pc.checked_add_signed(offset as isize).unwrap();
                }
            }
            other => panic!("ifle expected int, received {:?}", other)
        }

        Ok(())
    }    

    fn if_icmpeq(&mut self, frame: &mut StackFrame, offset: i16) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Int(value1), JValue::Int(value2)) => {
                if value1 == value2 {
                    frame.pc = frame.pc.checked_add_signed(offset as isize).unwrap();
                }
            },
            (other1, other2) => panic!("if_icmpeq expected both values to be int, received {:?} and {:?}", other1, other2)
        }

        Ok(())
    }

    fn if_icmpne(&mut self, frame: &mut StackFrame, offset: i16) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Int(value1), JValue::Int(value2)) => {
                if value1 != value2 {
                    frame.pc = frame.pc.checked_add_signed(offset as isize).unwrap();
                }
            },
            (other1, other2) => panic!("if_icmpne expected both values to be int, received {:?} and {:?}", other1, other2)
        }

        Ok(())
    }

    fn if_icmplt(&mut self, frame: &mut StackFrame, offset: i16) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Int(value1), JValue::Int(value2)) => {
                if value1 < value2 {
                    frame.pc = frame.pc.checked_add_signed(offset as isize).unwrap();
                }
            },
            (other1, other2) => panic!("if_icmplt expected both values to be int, received {:?} and {:?}", other1, other2)
        }

        Ok(())
    }

    fn if_icmpge(&mut self, frame: &mut StackFrame, offset: i16) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Int(value1), JValue::Int(value2)) => {
                if value1 >= value2 {
                    frame.pc = frame.pc.checked_add_signed(offset as isize).unwrap();
                }
            },
            (other1, other2) => panic!("if_icmpge expected both values to be int, received {:?} and {:?}", other1, other2)
        }

        Ok(())
    }

    fn if_icmpgt(&mut self, frame: &mut StackFrame, offset: i16) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Int(value1), JValue::Int(value2)) => {
                if value1 > value2 {
                    frame.pc = frame.pc.checked_add_signed(offset as isize).unwrap();
                }
            },
            (other1, other2) => panic!("if_icmpgt expected both values to be int, received {:?} and {:?}", other1, other2)
        }

        Ok(())
    }

    fn if_icmple(&mut self, frame: &mut StackFrame, offset: i16) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Int(value1), JValue::Int(value2)) => {
                if value1 <= value2 {
                    frame.pc = frame.pc.checked_add_signed(offset as isize).unwrap();
                }
            },
            (other1, other2) => panic!("if_icmple expected both values to be int, received {:?} and {:?}", other1, other2)
        }

        Ok(())
    }    

    fn if_acmpeq(&mut self, frame: &mut StackFrame, offset: i16) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Reference(value1), JValue::Reference(value2)) => {
                if Rc::ptr_eq(&value1, &value2) {
                    frame.pc = frame.pc.checked_add_signed(offset as isize).unwrap();
                }
            },
            (other1, other2) => panic!("if_acmpeq expected both values to be reference, received {:?} and {:?}", other1, other2)
        }

        Ok(())
    }

    fn if_acmpne(&mut self, frame: &mut StackFrame, offset: i16) -> JVMResult {
        let value2 = frame.operand_stack.pop().unwrap();
        let value1 = frame.operand_stack.pop().unwrap();
        match (value1, value2) {
            (JValue::Reference(value1), JValue::Reference(value2)) => {
                if !Rc::ptr_eq(&value1, &value2) {
                    frame.pc = frame.pc.checked_add_signed(offset as isize).unwrap();
                }
            },
            (other1, other2) => panic!("if_acmpne expected both values to be reference, received {:?} and {:?}", other1, other2)
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
                    JObjectKind::ArrayRef(array) => array[index as usize] = Some(value),
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
                    JObjectKind::ArrayRef(array) => frame.operand_stack.push(JValue::Reference(array[index as usize].clone().unwrap())),
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
        let entry = match frame.class.constant_pool.resolve_constant(index, &mut self.class_loader) {
            ResolvedConstant::Integer(value) => JValue::Int(value),
            ResolvedConstant::Float(value) => JValue::Float(value),
            ResolvedConstant::String(value) => JValue::Reference(self.make_java_string(value)),
            ResolvedConstant::Class(value) => unimplemented!("Class ldc not implemented yet"),
            other => panic!("Invalid ldc entry received: {:?}", other)
        };
        frame.operand_stack.push(entry);
        Ok(())
    }

    /* End Constants */

    fn make_java_string(&mut self, s: String) -> Rc<RefCell<JObject>> {
        if let Some(ptr) = self.interned_strings.get(&s) {
            return ptr.clone();
        }

        let string_class = self.class_loader.load_class("java/lang/String").unwrap();
        let str_object = JObject::new(string_class);
        let chars = s.encode_utf16().collect::<Vec<u16>>();
        let char_array_class = self.class_loader.load_class("[C").unwrap();
        let char_array_object = JObject::new_kind(
            char_array_class,
            JObjectKind::ArrayChar(chars)
        );

        str_object.borrow_mut().set_field(
            "value:[C",
            JValue::Reference(char_array_object) 
        );

        self.interned_strings.insert(s.to_string(), str_object.clone());

        str_object
    }

    fn parse_opcode(code: &[u8], pc: usize) -> std::io::Result<(Opcode, usize)> {
        let mut reader = Cursor::new(&code[pc..]);
        let mut len = 0_usize;
        let code = reader.read_u8()?;
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
                len += 1;
                Opcode::BIPush(idx)
            }
            0x11 => {
                let idx = reader.read_i16::<BigEndian>()?;
                len += 2;
                Opcode::SIPush(idx)
            }
            0x12 => {
                let idx = reader.read_u8()?; 
                len += 1;
                Opcode::Ldc(idx) 
            }
            0x13 => {
                let idx = reader.read_u16::<BigEndian>()?;
                len += 2;
                Opcode::LdcW(idx)
            }
            0x14 => {
                let idx = reader.read_u16::<BigEndian>()?;
                len += 2;
                Opcode::Ldc2W(idx)
            }

            // Loads
            0x15 => {
                let idx = reader.read_u8()?;
                len += 1;
                Opcode::ILoad(idx)
            }
            0x16 => {
                let idx = reader.read_u8()?;
                len += 1;
                Opcode::LLoad(idx)
            }
            0x17 => {
                let idx = reader.read_u8()?;
                len += 1;
                Opcode::FLoad(idx)
            }
            0x18 => {
                let idx = reader.read_u8()?;
                len += 1;
                Opcode::DLoad(idx)
            }
            0x19 => {
                let idx = reader.read_u8()?;
                len += 1;
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
                len += 1;
                Opcode::IStore(idx)
            }
            0x37 => { /* LSTORE */ 
                let idx = reader.read_u8()?;
                len += 1;
                Opcode::LStore(idx)
            }
            0x38 => { /* FSTORE */
                let idx = reader.read_u8()?;
                len += 1;
                Opcode::FStore(idx)
            }
            0x39 => { /* DSTORE */
                let idx = reader.read_u8()?;
                len += 1;
                Opcode::DStore(idx)
            }
            0x3A => { /* ASTORE */
                let idx = reader.read_u8()?;
                len += 1;
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
                len += 2;
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
                len += 2;
                Opcode::IfEq(offset)
            }
            0x9A => {
                let offset = reader.read_i16::<BigEndian>()?;
                len += 2;
                Opcode::IfNe(offset)
            }
            0x9B => {
                let offset = reader.read_i16::<BigEndian>()?;
                len += 2;
                Opcode::IfLt(offset)
            }
            0x9C => {
                let offset = reader.read_i16::<BigEndian>()?;
                len += 2;
                Opcode::IfGe(offset)
            }
            0x9D => {
                let offset = reader.read_i16::<BigEndian>()?;
                len += 2;
                Opcode::IfGt(offset)
            }
            0x9E => {
                let offset = reader.read_i16::<BigEndian>()?;
                len += 2;
                Opcode::IfLe(offset)
            }
            0x9F => {
                let offset = reader.read_i16::<BigEndian>()?;
                len += 2;
                Opcode::IfICmpEq(offset)
            }
            0xA0 => {
                let offset = reader.read_i16::<BigEndian>()?;
                len += 2;
                Opcode::IfICmpNe(offset)
            }
            0xA1 => {
                let offset = reader.read_i16::<BigEndian>()?;
                len += 2;
                Opcode::IfICmpLt(offset)
            }
            0xA2 => {
                let offset = reader.read_i16::<BigEndian>()?;
                len += 2;
                Opcode::IfICmpGe(offset)
            }
            0xA3 => {
                let offset = reader.read_i16::<BigEndian>()?;
                len += 2;
                Opcode::IfICmpGt(offset)
            }
            0xA4 => {
                let offset = reader.read_i16::<BigEndian>()?;
                len += 2;
                Opcode::IfICmpLe(offset)
            }
            0xA5 => {
                let offset = reader.read_i16::<BigEndian>()?;
                len += 2;
                Opcode::IfACmpEq(offset)
            }
            0xA6 => {
                let offset = reader.read_i16::<BigEndian>()?;
                len += 2;
                Opcode::IfACmpNe(offset)
            }

            /* Control */
            0xA7 => {
                let offset = reader.read_i16::<BigEndian>()?;
                len += 2;
                Opcode::Goto(offset)
            }
            0xA8 => {
                let offset = reader.read_i16::<BigEndian>()?;
                len += 2;
                Opcode::Jsr(offset)
            }
            0xA9 => {
                let idx = reader.read_u8()?;
                len += 1;
                Opcode::Ret(idx)
            }
            0xAA => {
                let padding = (4 - (pc + 1) % 4) % 4;
                for _ in 0..padding {
                    reader.read_u8()?;
                    len += 1;
                }

                let default = reader.read_i32::<BigEndian>()?;
                let low = reader.read_i32::<BigEndian>()?;
                let high = reader.read_i32::<BigEndian>()?;

                len += 12;
                let mut offsets = Vec::with_capacity((high - low + 1) as usize);
                for _ in low..=high {
                    offsets.push(reader.read_i32::<BigEndian>()?);
                    len += 4;
                }

                Opcode::TableSwitch { default_offset: default, low, high, jump_offsets: offsets }
            }
            0xAB => {
                let padding = (4 - (pc + 1) % 4) % 4;
                for _ in 0..padding {
                    reader.read_u8()?;
                    len += 1;
                }

                let default = reader.read_i32::<BigEndian>()?;
                let npairs = reader.read_i32::<BigEndian>()?;
                len += 8;

                let mut pairs = Vec::with_capacity(npairs as usize);
                for _ in 0..npairs {
                    let match_val = reader.read_i32::<BigEndian>()?;
                    let offset = reader.read_i32::<BigEndian>()?;
                    pairs.push((match_val, offset));
                    len += 8;
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
                len += 2;
                Opcode::GetStatic(index)
            }
            0xB3 => {
                let index = reader.read_u16::<BigEndian>()?;
                len += 2;
                Opcode::PutStatic(index)
            }
            0xB4 => {
                let index = reader.read_u16::<BigEndian>()?;
                len += 2;
                Opcode::GetField(index)
            }
            0xB5 => {
                let index = reader.read_u16::<BigEndian>()?;
                len += 2;
                Opcode::PutField(index)
            }
            0xB6 => {
                let idx = reader.read_u16::<BigEndian>()?;
                len += 2;
                Opcode::InvokeVirtual(idx)
            }
            0xB7 => {
                let idx = reader.read_u16::<BigEndian>()?;
                len += 2;
                Opcode::InvokeSpecial(idx)
            }
            0xB8 => {
                let idx = reader.read_u16::<BigEndian>()?;
                len += 2;
                Opcode::InvokeStatic(idx)
            }
            0xB9 => {
                let idx = reader.read_u16::<BigEndian>()?;
                let count = reader.read_u8()?;
                if reader.read_u8()? == 0 {
                    panic!("Expected 0 after invokeinterface count");
                }
                len += 4;
                Opcode::InvokeInterface(idx, count)
            }
            0xBA => {
                let idx = reader.read_u16::<BigEndian>()?;
                len += 2;
                Opcode::InvokeDynamic(idx)
            }
            0xBB => {
                let index = reader.read_u16::<BigEndian>()?;
                len += 2;
                Opcode::New(index)
            }
            0xBC => {
                let atype = reader.read_u8()?;
                let atype = AType::try_from_primitive(atype).expect("AType after newarray instruction was invalid.");
                len += 1;
                Opcode::NewArray(atype)
            }
            0xBD => {
                let index = reader.read_u16::<BigEndian>()?;
                len += 2;
                Opcode::ANewArray(index)
            }
            0xBE => Opcode::ArrayLength,
            0xBF => Opcode::AThrow,
            0xC0 => {
                let index = reader.read_u16::<BigEndian>()?;
                len += 2;
                Opcode::CheckCast(index)
            }
            0xC1 => {
                let index = reader.read_u16::<BigEndian>()?;
                len += 2;
                Opcode::InstanceOf(index)
            }
            0xC2 => Opcode::MonitorEnter,
            0xC3 => Opcode::MonitorExit,

            /* Extended */
            0xC4 => {
                let wide_opcode = reader.read_u8()?;
                len += 1;

                let instr = match wide_opcode {
                    // Loads
                    0x15 => {
                        let index = reader.read_u16::<BigEndian>()?;
                        len += 2;
                        WideInstruction::ILoad(index)
                    }
                    0x16 => {
                        let index = reader.read_u16::<BigEndian>()?;
                        len += 2;
                        WideInstruction::LLoad(index)
                    }
                    0x17 => {
                        let index = reader.read_u16::<BigEndian>()?;
                        len += 2;
                        WideInstruction::FLoad(index)
                    }
                    0x18 => {
                        let index = reader.read_u16::<BigEndian>()?;
                        len += 2;
                        WideInstruction::DLoad(index)
                    }
                    0x19 => {
                        let index = reader.read_u16::<BigEndian>()?;
                        len += 2;
                        WideInstruction::ALoad(index)
                    }

                    // Stores
                    0x36 => {
                        let index = reader.read_u16::<BigEndian>()?;
                        len += 2;
                        WideInstruction::IStore(index)
                    }
                    0x37 => {
                        let index = reader.read_u16::<BigEndian>()?;
                        len += 2;
                        WideInstruction::LStore(index)
                    }
                    0x38 => {
                        let index = reader.read_u16::<BigEndian>()?;
                        len += 2;
                        WideInstruction::FStore(index)
                    }
                    0x39 => {
                        let index = reader.read_u16::<BigEndian>()?;
                        len += 2;
                        WideInstruction::DStore(index)
                    }
                    0x3A => {
                        let index = reader.read_u16::<BigEndian>()?;
                        len += 2;
                        WideInstruction::AStore(index)
                    }

                    // Increment
                    0x84 => {
                        let index = reader.read_u16::<BigEndian>()?;
                        let constant = reader.read_i16::<BigEndian>()?;
                        len += 4;
                        WideInstruction::IInc(index, constant)
                    }

                    _ => panic!("Unsupported opcode after WIDE: {}", wide_opcode),
                };

                Opcode::Wide(instr)
            }
            0xC5 => {
                let index = reader.read_u16::<BigEndian>()?;
                let dimensions = reader.read_u8()?;
                len += 3;
                Opcode::MultiANewArray(index, dimensions)
            }
            0xC6 => {
                let offset = reader.read_i16::<BigEndian>()?;
                len += 2;
                Opcode::IfNull(offset)
            }
            0xC7 => {
                let offset = reader.read_i16::<BigEndian>()?;
                len += 2;
                Opcode::IfNonNull(offset)
            }
            0xC8 => {
                let offset = reader.read_i32::<BigEndian>()?;
                len += 4;
                Opcode::GotoW(offset)
            }
            0xC9 => {
                let offset = reader.read_i32::<BigEndian>()?;
                len += 4;
                Opcode::JsrW(offset)
            }
            0xCA => Opcode::Breakpoint,

            0xCB..=0xFF => panic!("Unknown opcode {:X}", code),
        };

        len += 1;

        Ok((op, len))
    }
}

type JVMResult = Result<(), JVMError>;

#[derive(Debug)]
pub enum JVMError {

}