use std::fs;

use neat::parameters::Parameters;

fn _range_lerp( value: f32 , istart: f32, istop: f32, ostart: f32, ostop: f32) -> f32
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