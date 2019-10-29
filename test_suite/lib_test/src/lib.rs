mod serde_test;
mod base_test;
mod impl_test;

pub use std::os::raw::{c_char, c_void};
pub use std::ffi::CString;

#[link(name = "test_lib.dll")]
extern "C" {
    // lib.rs
    pub fn free_string(ptr: *mut c_char);
    pub fn last_error() -> *mut c_char;

    // base_test.rs
    pub fn hello_world() -> *mut c_char;
    pub fn add(a: i32, b: i32) -> i32;
    pub fn throw_err(is_err: bool) -> *mut c_char;
    pub fn return_option(is_opt: bool) -> *mut c_char;

    // serde_test.rs
    pub fn new_serde_struct() -> *mut c_char;

    //impl_test.rs
    pub fn new_boxed_struct() -> *mut c_void; 
    pub fn inc_boxed_struct(this: *mut c_void);
    pub fn get_boxed_struct(this: *const c_void) -> u32;
    pub fn free_boxed_struct(ptr: *mut c_void);
}

pub fn assert_cstr(expected: &str, actual: *mut c_char) {
    unsafe {
        let msg = CString::from_raw(actual);
        assert_eq!(expected, msg.to_str().unwrap());
    }
}