use std::path::PathBuf;

use math::{tan, Matrix, Size, Vector2, Vector3, Vector4};
use crate::buffer::Buffer;
use crate::layout::cast_slice;
use crate::color::Rgb;
use crate::app::CONTEXT;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub uv: Vector2<f32>,
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: size_of::<Vector3<f32>>() as wgpu::BufferAddress,
                    shader_location: 1,
                },
            ],
        }
    }
}

impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
            && self.uv == other.uv
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Transform {
    mat: Matrix<Vector4<f32>, 4>,
}

impl std::fmt::Debug for Transform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.mat)
    }
}

impl Transform {
    const IDENTITY: Self = Self { mat: Matrix::IDENTITIY };

    pub fn transform(&mut self, t: Vector2<f32>, s: Size<f32>) {
        self.mat.transform(t.x, t.y, s.width, s.height)
    }

    pub fn translate(&mut self, t: Vector2<f32>) {
        self.mat.translate(t.x, t.y);
    }

    pub fn as_slice(&self) -> &[u8] {
        cast_slice(self.mat.data()).unwrap()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeKind {
    FilledTriangle,
    FilledRectangle,
    TexturedRectangle,
}

impl ShapeKind {
    pub fn elements(&self) -> usize {
        match self {
            ShapeKind::FilledTriangle => 3,
            ShapeKind::FilledRectangle => 4,
            ShapeKind::TexturedRectangle => 4,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl From<ShapeKind> for Mesh {
    fn from(kind: ShapeKind) -> Self {
        match kind {
            ShapeKind::FilledTriangle => Self::triangle(),
            ShapeKind::FilledRectangle => Self::rectangle(),
            ShapeKind::TexturedRectangle => Self::rectangle(),
        }
    }
}

impl Mesh {
    fn rectangle() -> Self {
        Self {
            vertices: [
                Vertex { position: Vector3 { x: -1.0, y:  1.0, z: 1.0 }, uv: Vector2 { x: 0.0, y: 0.0 } },
                Vertex { position: Vector3 { x: -1.0, y: -1.0, z: 1.0 }, uv: Vector2 { x: 0.0, y: 1.0 } },
                Vertex { position: Vector3 { x:  1.0, y: -1.0, z: 1.0 }, uv: Vector2 { x: 1.0, y: 1.0 } },
                Vertex { position: Vector3 { x:  1.0, y:  1.0, z: 1.0 }, uv: Vector2 { x: 1.0, y: 0.0 } },
                ].to_vec(),
            indices: [0, 1, 2, 2, 3, 0].to_vec(),
        }
    }

    fn triangle() -> Self {
        Self {
            vertices: [
                Vertex { position: Vector3 { x:  0.0, y:  1.0, z: 1.0 }, uv: Vector2 { x: 0.5, y: 0.0 } },
                Vertex { position: Vector3 { x: -1.0, y: -1.0, z: 1.0 }, uv: Vector2 { x: 0.0, y: 1.0 } },
                Vertex { position: Vector3 { x:  1.0, y: -1.0, z: 1.0 }, uv: Vector2 { x: 1.0, y: 1.0 } },
                ].to_vec(),
            indices: [0, 1, 2].to_vec(),
        }
    }
}

// originaly, every shape is rooted to the center of the screen where center is [0, 0]
// going top    -> [ 0,  y ],
// going left   -> [-x,  0 ],
// going bottom -> [ 0, -y ],
// going right  -> [ x,  0 ],
//
//
// a normal square with (width, height) would have
// top left     [x - width/2, y + height/2],
// bottom left  [x - width/2, y - height/2],
// bottom right [x + width/2, y - height/2],
// top right    [x + width/2, y + height/2],
// where (width, height) is normalized to window's inner_size

#[derive(Debug, Clone)]
pub struct Shape {
    pub dimensions: Size<u32>,
    pub cached_color: Rgb<u8>,
    pub src: Option<PathBuf>,
    pub kind: ShapeKind,
    pub transform: Transform,
}

impl Shape {
    pub fn filled(color: Rgb<u8>, kind : ShapeKind) -> Self {
        Self {
            dimensions: Size::new(500, 500),
            cached_color: color,
            src: None,
            kind,
            transform: Transform::IDENTITY,
        }
    }

    pub fn textured(src: PathBuf, kind: ShapeKind) -> Self {
        Self {
            dimensions: Size::new(500, 500),
            cached_color: Rgb::WHITE,
            src: Some(src),
            kind,
            transform: Transform::IDENTITY,
        }
    }

    pub fn set_transform(&mut self, t: Vector2<f32>, s: Size<f32>) {
        self.transform.transform(t, s)
    }

    fn set_translate(&mut self, t: Vector2<f32>) {
        self.transform.translate(t);
    }

    pub fn vertices(&self,device: &wgpu::Device) -> Buffer<Vertex> {
        let vertices = Mesh::from(self.kind).vertices;
        Buffer::new(device, wgpu::BufferUsages::VERTEX, cast_slice(&vertices).unwrap(), vertices.len())
    }

    pub fn indices(&self, device: &wgpu::Device) -> Buffer<u32> {
        let indices = Mesh::from(self.kind).indices;
        Buffer::new(device, wgpu::BufferUsages::INDEX, cast_slice(&indices).unwrap(), indices.len())
    }

    pub fn uniform_buffer(&self, device: &wgpu::Device) -> Buffer<Transform> {
        Buffer::new(device, wgpu::BufferUsages::UNIFORM, self.transform.as_slice(), 0)
    }

    // for now, i think the dimension will always be constant due to scaling transform
    // but still, i need better calculation later
    fn dimension(&self) -> Size<f32> {
        let window_size = CONTEXT.with_borrow(|ctx| ctx.window_size);
        let width = self.dimensions.width as f32 / window_size.width as f32;
        let height = -(self.dimensions.height as f32 / window_size.height as f32);
        Size { width, height }
    }

    pub fn pos(&self) -> Vector2<f32> {
        let mut vertices = Mesh::from(self.kind).vertices;
        vertices.iter_mut().for_each(|vert| {
            let v4 = Vector4::from(vert.position);
            let v4 = self.transform.mat * v4;
            vert.position = Vector3 { x: v4.x, y: v4.y, z: v4.z };
        });
        Vector2 {
            x: vertices[1].position.x,
            y: vertices[0].position.y,
        }
    }

    pub fn is_hovered(&self) -> bool {
        let (cursor, window_size) = CONTEXT.with_borrow(|ctx| (ctx.cursor, ctx.window_size));
        let x_cursor = ((cursor.hover.pos.x / window_size.width as f32) - 0.5) * 2.0;
        let y_cursor = (0.5 - (cursor.hover.pos.y / window_size.height as f32)) * 2.0;

        let Size { width, height } = self.dimension();
        let Vector2 { x, y } = self.pos();

        let angled = if self.kind.elements() == 3 {
            let x_center = width / 2.0;
            let cursor_tan = tan(x + x_center - x_cursor, y - y_cursor);
            let triangle_tan = tan(x_center, height);
            cursor_tan >= triangle_tan
        } else { true };

        (y + height..y).contains(&y_cursor)
            && (x..x + width).contains(&x_cursor)
            && angled
    }

    pub fn set_color<F: FnOnce(&mut Rgb<u8>)>(&mut self, f: F) {
        f(&mut self.cached_color);
    }

    // pub fn revert_color(&mut self) {
    //     if let Some(ref c) = self.cached_color {
    //         self.uv_data = c.clone().into();
    //     }
    // }

    pub fn set_position(&mut self) {
        let (cursor, window_size) = CONTEXT.with_borrow(|ctx| (ctx.cursor, Size::<f32>::from(ctx.window_size)));
        let t = (cursor.hover.pos - cursor.click.pos) * 2.0;

        self.set_translate(
            Vector2 {
                x: t.x / window_size.width,
                y: -t.y / window_size.height
            }
        );

        CONTEXT.with_borrow_mut(|ctx| {
            ctx.cursor.click.pos = cursor.hover.pos;
        });
    }
}

