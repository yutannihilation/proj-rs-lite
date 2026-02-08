#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

extern crate libsqlite3_sys;
extern crate link_cplusplus;

use core::ffi::{c_char, c_double, c_int, c_void};

#[repr(C)]
pub struct PJ_CONTEXT {
    _private: [u8; 0],
}

#[repr(C)]
pub struct PJconsts {
    _private: [u8; 0],
}

pub type PJ_DIRECTION = c_int;
pub const PJ_DIRECTION_PJ_INV: PJ_DIRECTION = -1;
pub const PJ_DIRECTION_PJ_IDENT: PJ_DIRECTION = 0;
pub const PJ_DIRECTION_PJ_FWD: PJ_DIRECTION = 1;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PJ_XY {
    pub x: c_double,
    pub y: c_double,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PJ_XYZT {
    pub x: c_double,
    pub y: c_double,
    pub z: c_double,
    pub t: c_double,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PJ_LPZT {
    pub lam: c_double,
    pub phi: c_double,
    pub z: c_double,
    pub t: c_double,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union PJ_COORD {
    pub xy: PJ_XY,
    pub xyzt: PJ_XYZT,
    pub lpzt: PJ_LPZT,
    pub v: [c_double; 4],
}

unsafe extern "C" {
    pub fn proj_context_create() -> *mut PJ_CONTEXT;
    pub fn proj_context_destroy(ctx: *mut PJ_CONTEXT);
    pub fn proj_context_errno(ctx: *const PJ_CONTEXT) -> c_int;

    pub fn proj_create(ctx: *mut PJ_CONTEXT, definition: *const c_char) -> *mut PJconsts;
    pub fn proj_create_crs_to_crs(
        ctx: *mut PJ_CONTEXT,
        source_crs: *const c_char,
        target_crs: *const c_char,
        area: *const c_void,
    ) -> *mut PJconsts;
    pub fn proj_normalize_for_visualization(
        ctx: *mut PJ_CONTEXT,
        obj: *mut PJconsts,
    ) -> *mut PJconsts;
    pub fn proj_destroy(obj: *mut PJconsts);

    pub fn proj_errno_reset(obj: *mut PJconsts);
    pub fn proj_errno(obj: *const PJconsts) -> c_int;
    pub fn proj_errno_string(err: c_int) -> *const c_char;

    pub fn proj_trans(obj: *mut PJconsts, direction: PJ_DIRECTION, coord: PJ_COORD) -> PJ_COORD;
}
