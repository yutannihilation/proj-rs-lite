use proj_lite::Proj;
use wasm_bindgen::prelude::*;

fn to_js_err(err: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&err.to_string())
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
    let out = proj.transform3((x, y, z)).map_err(to_js_err)?;
    Ok(vec![out.0, out.1, out.2])
}
