use proj_sys::{
    PJ_CONTEXT, PJ_COORD, PJ_DIRECTION_PJ_FWD, PJ_XYZT, PJconsts, proj_context_create,
    proj_context_destroy, proj_context_errno, proj_create, proj_create_crs_to_crs, proj_destroy,
    proj_errno, proj_errno_reset, proj_errno_string, proj_normalize_for_visualization, proj_trans,
};
use std::ffi::{CStr, CString};
use std::ptr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProjError {
    #[error("failed to create PROJ context")]
    ContextCreation,
    #[error("input string contains an embedded NUL byte")]
    InvalidInput(#[from] std::ffi::NulError),
    #[error("failed to create PROJ object: {0}")]
    Creation(String),
    #[error("failed to normalize CRS transform: {0}")]
    Normalization(String),
    #[error("PROJ transform failed: {0}")]
    Transform(String),
}

pub struct Proj {
    ctx: *mut PJ_CONTEXT,
    proj: *mut PJconsts,
}

impl Proj {
    pub fn new(definition: &str) -> Result<Self, ProjError> {
        let ctx = create_context()?;
        let c_def = CString::new(definition)?;
        let proj = unsafe { proj_create(ctx, c_def.as_ptr()) };
        if proj.is_null() {
            let err = context_error_message(ctx);
            unsafe { proj_context_destroy(ctx) };
            return Err(ProjError::Creation(err));
        }

        Ok(Self { ctx, proj })
    }

    pub fn new_known_crs(from: &str, to: &str) -> Result<Self, ProjError> {
        let ctx = create_context()?;
        let from_c = CString::new(from)?;
        let to_c = CString::new(to)?;
        let raw =
            unsafe { proj_create_crs_to_crs(ctx, from_c.as_ptr(), to_c.as_ptr(), ptr::null()) };
        if raw.is_null() {
            let err = context_error_message(ctx);
            unsafe { proj_context_destroy(ctx) };
            return Err(ProjError::Creation(err));
        }

        let normalized = unsafe { proj_normalize_for_visualization(ctx, raw) };
        unsafe { proj_destroy(raw) };
        if normalized.is_null() {
            let err = context_error_message(ctx);
            unsafe { proj_context_destroy(ctx) };
            return Err(ProjError::Normalization(err));
        }

        Ok(Self {
            ctx,
            proj: normalized,
        })
    }

    pub fn transform(&self, point: (f64, f64)) -> Result<(f64, f64), ProjError> {
        let coord = PJ_COORD {
            xyzt: PJ_XYZT {
                x: point.0,
                y: point.1,
                z: 0.0,
                t: f64::INFINITY,
            },
        };

        let transformed = unsafe {
            proj_errno_reset(self.proj);
            proj_trans(self.proj, PJ_DIRECTION_PJ_FWD, coord)
        };
        let err = unsafe { proj_errno(self.proj) };
        if err != 0 {
            return Err(ProjError::Transform(proj_error_message(err)));
        }

        let xy = unsafe { transformed.xy };
        Ok((xy.x, xy.y))
    }

    pub fn transform3(&self, point: (f64, f64, f64)) -> Result<(f64, f64, f64), ProjError> {
        let coord = PJ_COORD {
            xyzt: PJ_XYZT {
                x: point.0,
                y: point.1,
                z: point.2,
                t: f64::INFINITY,
            },
        };

        let transformed = unsafe {
            proj_errno_reset(self.proj);
            proj_trans(self.proj, PJ_DIRECTION_PJ_FWD, coord)
        };
        let err = unsafe { proj_errno(self.proj) };
        if err != 0 {
            return Err(ProjError::Transform(proj_error_message(err)));
        }

        let xyz = unsafe { transformed.xyzt };
        Ok((xyz.x, xyz.y, xyz.z))
    }
}

impl Drop for Proj {
    fn drop(&mut self) {
        unsafe {
            if !self.proj.is_null() {
                proj_destroy(self.proj);
            }
            if !self.ctx.is_null() {
                proj_context_destroy(self.ctx);
            }
        }
    }
}

fn create_context() -> Result<*mut PJ_CONTEXT, ProjError> {
    let ctx = unsafe { proj_context_create() };
    if ctx.is_null() {
        return Err(ProjError::ContextCreation);
    }
    Ok(ctx)
}

fn context_error_message(ctx: *mut PJ_CONTEXT) -> String {
    let err = unsafe { proj_context_errno(ctx) };
    proj_error_message(err)
}

fn proj_error_message(err: i32) -> String {
    if err == 0 {
        return "unknown error".to_string();
    }
    let ptr = unsafe { proj_errno_string(err) };
    if ptr.is_null() {
        return format!("PROJ errno={err}");
    }
    let text = unsafe { CStr::from_ptr(ptr) };
    text.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::Proj;

    #[test]
    fn converts_2d_known_crs() {
        let tf = Proj::new_known_crs("EPSG:2230", "EPSG:26946").expect("create transformer");
        let out = tf
            .transform((4_760_096.421_921, 3_744_293.729_449))
            .expect("transform point");

        assert!((out.0 - 1_450_880.29).abs() < 1e-2);
        assert!((out.1 - 1_141_263.01).abs() < 1e-2);
    }

    #[test]
    fn supports_3d_transform() {
        let tf = Proj::new_known_crs("EPSG:4326", "EPSG:4979").expect("create transformer");
        let out = tf
            .transform3((-122.4194, 37.7749, 10.0))
            .expect("transform point");

        assert!(out.0.is_finite());
        assert!(out.1.is_finite());
        assert!(out.2.is_finite());
    }
}
