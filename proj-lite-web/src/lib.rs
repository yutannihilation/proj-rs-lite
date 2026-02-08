use geo_traits::{CoordTrait, Dimensions};
use proj_lite::Proj;
use wasm_bindgen::prelude::*;

fn to_js_err(err: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&err.to_string())
}

struct Coord3 {
    x: f64,
    y: f64,
    z: f64,
}

impl CoordTrait for Coord3 {
    type T = f64;

    fn dim(&self) -> Dimensions {
        Dimensions::Xyz
    }

    fn nth_or_panic(&self, n: usize) -> Self::T {
        match n {
            0 => self.x,
            1 => self.y,
            2 => self.z,
            _ => panic!("Coord3 supports only 3 dimensions"),
        }
    }

    fn x(&self) -> Self::T {
        self.x
    }

    fn y(&self) -> Self::T {
        self.y
    }
}

#[wasm_bindgen]
pub fn transform2_known_crs(
    from_crs: &str,
    to_crs: &str,
    x: f64,
    y: f64,
) -> Result<Vec<f64>, JsValue> {
    let proj = Proj::new_known_crs(from_crs, to_crs).map_err(to_js_err)?;
    let out = proj.transform2((x, y)).map_err(to_js_err)?;
    Ok(vec![out.0, out.1])
}

#[wasm_bindgen]
pub fn transform3_known_crs(
    from_crs: &str,
    to_crs: &str,
    x: f64,
    y: f64,
    z: f64,
) -> Result<Vec<f64>, JsValue> {
    let proj = Proj::new_known_crs(from_crs, to_crs).map_err(to_js_err)?;
    let out = proj.transform3(Coord3 { x, y, z }).map_err(to_js_err)?;
    Ok(vec![out.0, out.1, out.2])
}
