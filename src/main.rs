use crate::vm::jvm::{JVMError, JVM};

mod reader;
mod vm;
fn main() -> Result<(), JVMError> {
    let mut jvm = JVM::new();
    jvm.run_class("Main")
}
