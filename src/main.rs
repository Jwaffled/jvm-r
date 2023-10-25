use reader::ClassFileReader;

mod reader;
mod vm;
fn main() {
    let mut reader = ClassFileReader::new("Main.class");
    println!("Classfile: {:#?}", reader.read().unwrap());
}
