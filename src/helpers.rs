use std::fs;

use neat::parameters::Parameters;

// Just used for const because Vec2 cant be
pub struct V2<T> {
    pub x: T,
    pub y: T,
}

#[allow(dead_code)]
pub fn range_lerp( value: f32 , istart: f32, istop: f32, ostart: f32, ostop: f32) -> f32
{
    ostart + (ostop - ostart) * value / (istop - istart)
}

pub fn read_parameters_file(path: &str) -> Parameters {
    let params_str;
    if let Ok(str) = fs::read_to_string(path) {
        params_str = str;
    } else {
        panic!("Couldn't read params file path: {}", path);
    }

    toml::from_str(&params_str).unwrap()
}