#[cfg(genproto)]
extern crate prost_build;
#[cfg(genproto)]
use std::{env, fs, path::Path};

/// To generate updated proto objects:
/// 1. Place `vss.proto` file in `src/proto/`
/// 2. run `RUSTFLAGS="--cfg genproto" cargo build`
fn main() {
	#[cfg(genproto)]
	generate_protos();
}

#[cfg(genproto)]
fn generate_protos() {
	prost_build::compile_protos(&["src/proto/vss.proto"], &["src/"]).unwrap();
	let from_path = Path::new(&env::var("OUT_DIR").unwrap()).join("vss.rs");
	fs::copy(from_path, "src/types.rs").unwrap();
}
