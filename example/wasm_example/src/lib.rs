use std::ffi::{c_char, CStr, CString};

extern "C" {
    fn success(ptr: *const u8, len: usize) -> u32;
    fn error(ptr: *const u8, len: usize) -> u32;
}

#[no_mangle]
pub extern "C" fn handle(ptr: *mut c_char, len: u32) -> u32 {
    unsafe {
        let slice = std::slice::from_raw_parts(ptr, len as usize);
        let cs = CStr::from_ptr(slice.as_ptr());
        let cs = CString::from(cs);
        let s = cs.to_str().unwrap();
        let s = format!(r#"{{"input":{}}}"#, s);

        success(s.as_ptr(), s.len());
    }
    1u32
}

// #[no_mangle]
// pub extern "C" fn handle(ptr: *mut c_char, len: u32) -> u32 {
//     unsafe {
//         let slice = std::slice::from_raw_parts(ptr, len as usize);
//         let cs = CStr::from_ptr(slice.as_ptr());
//         let cs = cs.to_str().unwrap();
//         if cs.trim_matches('"') == "true" {
//             let s = format!(r#"{{"result":"success"}}"#);
//             success(s.as_ptr(), s.len());
//         }else{
//             let s = format!(r#"input[{}] make a error"#,cs);
//             error(s.as_ptr(), s.len());
//         }
//     }
//     1u32
// }
