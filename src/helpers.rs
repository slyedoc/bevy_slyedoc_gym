fn _range_lerp( value: f32 , istart: f32, istop: f32, ostart: f32, ostop: f32) -> f32
{
    ostart + (ostop - ostart) * value / (istop - istart)
}