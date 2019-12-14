extern crate libc;
use crate::{CResult, IntoFFi};
use std::fmt::Display;
use std::os::raw::{c_char, c_void};
use std::marker::PhantomData;
use std::mem;

pub type FFiResult<T> = Result<T, FFiError>;
pub struct FFiResultWrap<T>(FFiResult<T>);

impl<T> From<FFiResult<T>> for FFiResultWrap<T> {
    fn from(v: FFiResult<T>) -> Self {
        Self(v)
    }
}


impl<T> Into<*mut CResult<T, *mut c_char>> for FFiResultWrap<T> {
    fn into(self) -> *mut CResult<T, *mut c_char> {
        unsafe {
            let v = match self.0 {
                Ok(v) => {
                    let obj_size = mem::size_of_val(&v);
                    let ptr: *mut T = libc::malloc(obj_size) as *mut T;
                    *ptr = v;
                    CResult {
                        is_err: false,
                        value: ptr as *mut c_void,
                        t: PhantomData,
                        e: PhantomData,
                    }
                },
                Err(e) => {
                    let v = String::into_ffi(e.content).unwrap();
                    let obj_size = mem::size_of_val(&v);
                    let ptr: *mut *mut c_char = libc::malloc(obj_size) as *mut *mut c_char;
                    *ptr = v;
                    CResult {
                        is_err: true,
                        value: ptr as *mut c_void,
                        t: PhantomData,
                        e: PhantomData,
                    }
                }
            };

            let obj_size = mem::size_of_val(&v);
            let ptr = libc::malloc(obj_size) as *mut CResult<T, *mut c_char>;
            *ptr = v;

            ptr
        }
    }
}

#[derive(Debug)]
pub struct FFiError {
    pub content: String
}

impl<T> From<T> for FFiError 
where
    T: Display
{
    fn from(v: T) -> FFiError {
        println!("Creating error {}", v);
        FFiError {
            content: format!("{}", v)
        }
    }
}
