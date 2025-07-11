use aplite_types::Rgba;

use crate::texture::AtlasId;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Element {
    pub(crate) fill_color: Rgba<f32>,
    pub(crate) stroke_color: Rgba<f32>,
    pub(crate) corners: CornerRadius,
    pub(crate) shape: Shape,
    pub(crate) rotation: f32,
    pub(crate) stroke_width: f32,
    pub(crate) atlas_id: AtlasId,
    pub(crate) transform_id: u32,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    Circle = 0,
    Rect = 1,
    RoundedRect = 2,
    Triangle = 3,
}

// impl Shape {
//     pub(crate) fn is_triangle(&self) -> bool { matches!(self, Self::Triangle) }

//     pub(crate) fn is_rounded_rect(&self) -> bool { matches!(self, Self::RoundedRect) }
// }

#[repr(C, align(16))]
#[derive(Default, Debug, Clone, Copy)]
pub struct CornerRadius {
    tl: f32,
    bl: f32,
    br: f32,
    tr: f32,
}

impl From<u32> for CornerRadius {
    fn from(val: u32) -> Self {
        Self {
            tl: val as _,
            bl: val as _,
            br: val as _,
            tr: val as _,
        }
    }
}

impl CornerRadius {
    /// It's recommended the value is between 0-100, where 0 means fully square and 100 means fully rounded
    /// Doesn't necessarily mean that you can't put a value more than 100
    pub const fn new(tl: u32, bl: u32, br: u32, tr: u32) -> Self {
        Self {
            tl: tl as _,
            bl: bl as _,
            br: br as _,
            tr: tr as _,
        }
    }

    /// It's recommended the value is between 0-100, where 0 means fully square and 100 means fully rounded
    /// Doesn't necessarily mean that you can't put a value more than 100
    pub const fn homogen(r: u32) -> Self {
        Self {
            tl: r as _,
            bl: r as _,
            br: r as _,
            tr: r as _,
        }
    }

    /// It's recommended the value is between 0-100, where 0 means fully square and 100 means fully rounded
    /// Doesn't necessarily mean that you can't put a value more than 100
    pub fn set_each(&mut self, tl: u32, bl: u32, br: u32, tr: u32) {
        self.tl = tl as _;
        self.bl = bl as _;
        self.br = br as _;
        self.tr = tr as _;
    }

    /// It's recommended the value is between 0-100, where 0 means fully square and 100 means fully rounded
    /// Doesn't necessarily mean that you can't put a value more than 100
    pub fn set_all(&mut self, val: u32) {
        self.tl = val as _;
        self.bl = val as _;
        self.br = val as _;
        self.tr = val as _;
    }

    /// It's recommended the value is between 0-100, where 0 means fully square and 100 means fully rounded
    /// Doesn't necessarily mean that you can't put a value more than 100
    pub fn set_top_left(&mut self, val: u32) {
        self.tl = val as _;
    }

    /// It's recommended the value is between 0-100, where 0 means fully square and 100 means fully rounded
    /// Doesn't necessarily mean that you can't put a value more than 100
    pub fn set_bot_left(&mut self, val: u32) {
        self.bl = val as _;
    }

    /// It's recommended the value is between 0-100, where 0 means fully square and 100 means fully rounded
    /// Doesn't necessarily mean that you can't put a value more than 100
    pub fn set_bot_right(&mut self, val: u32) {
        self.br = val as _;
    }

    /// It's recommended the value is between 0-100, where 0 means fully square and 100 means fully rounded
    /// Doesn't necessarily mean that you can't put a value more than 100
    pub fn set_top_right(&mut self, val: u32) {
        self.tr = val as _;
    }
}

impl Element {
    pub const fn new() -> Self {
        Self {
            fill_color: Rgba::new(1., 0., 0., 1.),
            stroke_color: Rgba::new(1., 1., 1., 1.),
            corners: CornerRadius::homogen(25),
            shape: Shape::RoundedRect,
            rotation: 0.0,
            stroke_width: 0.0,
            atlas_id: AtlasId::new(-1),
            transform_id: 0,
        }
    }

    pub(crate) fn atlas_id(&self) -> AtlasId {
        self.atlas_id
    }

    pub fn rotation(&self) -> f32 {
        self.rotation
    }

    pub fn with_fill_color(mut self, color: Rgba<u8>) -> Self {
        self.fill_color = color.into();
        self
    }

    pub fn with_stroke_color(mut self, color: Rgba<u8>) -> Self {
        self.stroke_color = color.into();
        self
    }

    pub fn with_corner_radius(mut self, corner_radius: CornerRadius) -> Self {
        self.corners = corner_radius;
        self
    }

    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = width;
        self
    }

    // pub(crate) fn with_transform_id(mut self, id: u32) -> Self {
    //     self.transform_id = id;
    //     self
    // }

    // pub(crate) fn with_atlas_id(mut self, id: i32) -> Self {
    //     self.atlas_id = id;
    //     self
    // }

    // pub(crate) fn with_image_id(mut self, id: i32) -> Self {
    //     self.image_id = id;
    //     self
    // }

    pub fn set_fill_color(&mut self, color: Rgba<u8>) {
        self.fill_color = color.into();
    }

    pub fn set_stroke_color(&mut self, color: Rgba<u8>) {
        self.stroke_color = color.into();
    }

    pub fn set_stroke_width(&mut self, val: u32) {
        self.stroke_width = val as _;
    }

    pub fn set_rotation(&mut self, val: f32) {
        self.rotation = val;
    }

    pub fn set_corner_radius(&mut self, val: CornerRadius) {
        self.corners = val;
    }

    pub fn set_shape(&mut self, shape: Shape) {
        self.shape = shape;
    }

    pub fn set_transform_id(&mut self, val: u32) {
        self.transform_id = val;
    }

    pub fn set_atlas_id(&mut self, id: AtlasId) {
        self.atlas_id = id;
    }

    pub fn fill_color(&self) -> Rgba<u8> {
        self.fill_color.u8()
    }
}
