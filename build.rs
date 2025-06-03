use std::path::Path;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    // 编译FlatBuffers schema
    flatc_rust::run(flatc_rust::Args {
        inputs: &[Path::new("schemas/trade_event.fbs")],
        out_dir: Path::new(&out_dir),
        ..Default::default()
    }).expect("Failed to compile FlatBuffers schema");
    
    println!("cargo:rerun-if-changed=schemas/trade_event.fbs");
}