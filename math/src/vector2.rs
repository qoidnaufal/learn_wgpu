use crate::Vector3;

#[derive(Clone, Copy)]
pub struct Vector2<T> {
    pub x: T,
    pub y: T,
}

impl<T: Default> Default for Vector2<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Default> Vector2<T> {
    pub fn new() -> Self {
        Self { x: T::default(), y: T::default() }
    }
}

impl<T: std::fmt::Display> std::fmt::Debug for Vector2<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Vector2 {{ x: {:0.3}, y: {:0.3} }}", self.x, self.y)
    }
}

impl<T> std::ops::Mul<T> for Vector2<T>
where T:
    std::ops::Mul<T, Output = T>
    + Copy
{
    type Output = Self;
    fn mul(self, rhs: T) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl<T> std::ops::Add<Self> for Vector2<T>
where T:
    std::ops::Add<T, Output = T>
    + Copy
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y
        }
    }
}

impl<T> std::ops::AddAssign<Self> for Vector2<T>
where T:
    std::ops::Add<T, Output = T>
    + std::ops::AddAssign
    + Copy
{
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl<T> std::ops::Sub<Self> for Vector2<T>
where T:
    std::ops::Sub<T, Output = T>
    + Copy
{
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y
        }
    }
}

impl<T> std::ops::SubAssign<Self> for Vector2<T>
where T:
    std::ops::Sub<T, Output = T>
    + std::ops::SubAssign
    + Copy
{
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl From<Vector2<u32>> for Vector2<f32> {
    fn from(val: Vector2<u32>) -> Self {
        Self {
            x: val.x as _,
            y: val.y as _,
        }
    }
}

impl From<Vector2<f32>> for Vector2<u32> {
    fn from(val: Vector2<f32>) -> Self {
        Self {
            x: val.x as _,
            y: val.y as _,
        }
    }
}

impl<T> From<Vector3<T>> for Vector2<T> {
    fn from(val: Vector3<T>) -> Self {
        Self {
            x: val.x,
            y: val.y
        }
    }
}

impl<T> From<(T, T)> for Vector2<T> {
    fn from(value: (T, T)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

impl<T> PartialEq for Vector2<T>
where T:
    PartialEq<T>
{
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl<T: PartialEq + Eq> Eq for Vector2<T> {}
