mod vector2;
mod vector3;
mod size;
mod matrix;

pub use vector2::Vector2;
pub use vector3::Vector3;
pub use size::Size;
pub use matrix::{Matrix, Vector4};

pub fn tan(x: f32, y: f32) -> f32 {
    (y / x).abs()
}

pub fn cos(x: f32, y: f32) -> f32 {
    let hyp = (x*x + y*y).sqrt();
    (x / hyp).abs()
}
