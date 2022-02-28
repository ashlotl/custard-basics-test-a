use custard_macros::attach_datachunk;
use custard_use::{
	dylib_management::safe_library::load_types::{DatachunkLoadFn, FFIResult, FFISafeString},
	user_types::datachunk::Datachunk,
};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TestDatachunkA {
	field_a: bool,
	field_b: u32,
	field_c: String,
}

impl Datachunk for TestDatachunkA {}

attach_datachunk!(TestDatachunkA);
