use proj_lite::Proj;
use std::ptr;
use std::slice;
use std::str;
use wasm_bindgen::prelude::*;

static mut LAST_ERROR_BUF: [u8; 1024] = [0; 1024];
static mut LAST_ERROR_LEN: usize = 0;

fn set_last_error(msg: &str) {
    let bytes = msg.as_bytes();
    let n = bytes.len().min(1024);
    unsafe {
        LAST_ERROR_BUF[..n].copy_from_slice(&bytes[..n]);
        LAST_ERROR_LEN = n;
    }
}

fn clear_last_error() {
    unsafe {
        LAST_ERROR_LEN = 0;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn last_error_message_ptr() -> *const u8 {
    ptr::addr_of!(LAST_ERROR_BUF).cast::<u8>()
}

#[unsafe(no_mangle)]
pub extern "C" fn last_error_message_len() -> usize {
    unsafe { LAST_ERROR_LEN }
}

#[unsafe(no_mangle)]
pub extern "C" fn transform2_known_crs_raw(
    from_ptr: *const u8,
    from_len: usize,
    to_ptr: *const u8,
    to_len: usize,
    x: f64,
    y: f64,
    out_xy_ptr: *mut f64,
) -> i32 {
    if out_xy_ptr.is_null() || (from_len > 0 && from_ptr.is_null()) || (to_len > 0 && to_ptr.is_null()) {
        set_last_error("invalid pointer argument");
        return 1;
    }

    let from_bytes = unsafe { slice::from_raw_parts(from_ptr, from_len) };
    let to_bytes = unsafe { slice::from_raw_parts(to_ptr, to_len) };

    let from_crs = match str::from_utf8(from_bytes) {
        Ok(v) => v,
        Err(_) => {
            set_last_error("from_crs is not valid UTF-8");
            return 2;
        }
    };

    let to_crs = match str::from_utf8(to_bytes) {
        Ok(v) => v,
        Err(_) => {
            set_last_error("to_crs is not valid UTF-8");
            return 2;
        }
    };

    let proj = match Proj::new_known_crs(from_crs, to_crs) {
        Ok(v) => v,
        Err(e) => {
            set_last_error(&e.to_string());
            return 3;
        }
    };

    let out = match proj.transform2((x, y)) {
        Ok(v) => v,
        Err(e) => {
            set_last_error(&e.to_string());
            return 4;
        }
    };

    unsafe {
        *out_xy_ptr = out.0;
        *out_xy_ptr.add(1) = out.1;
    }
    clear_last_error();
    0
}

#[wasm_bindgen]
pub fn transform2_known_crs(
    from_crs: &str,
    to_crs: &str,
    x: f64,
    y: f64,
) -> Result<Vec<f64>, JsValue> {
    let proj = Proj::new_known_crs(from_crs, to_crs)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let out = proj
        .transform2((x, y))
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(vec![out.0, out.1])
}
