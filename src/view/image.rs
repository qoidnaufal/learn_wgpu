use std::path::{Path, PathBuf};

use crate::context::{Alignment, LayoutCtx};
use crate::shapes::{Shape, ShapeKind};
use crate::callback::CALLBACKS;
use super::{AnyView, IntoView, NodeId, View};

pub fn image<P: AsRef<Path>>(src: P) -> Image {
    Image::new(src)
}

pub struct Image {
    id: NodeId,
    src: PathBuf,
}

impl Image {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let id = NodeId::new();
        Self { id, src: path.as_ref().to_path_buf() }
    }

    fn id(&self) -> NodeId {
        self.id
    }

    fn shape(&self) -> Shape {
        Shape::textured(ShapeKind::TexturedRectangle)
    }

    // pub fn on_hover<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
    //     CALLBACKS.with_borrow_mut(|cbs| cbs.on_hover.insert(self.id(), f.into()));
    //     self
    // }

    // pub fn on_click<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
    //     CALLBACKS.with_borrow_mut(|cbs| cbs.on_click.insert(self.id(), f.into()));
    //     self
    // }

    pub fn on_drag<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_drag.insert(self.id(), f.into()));
        self
    }
}

impl View for Image {
    fn id(&self) -> NodeId { self.id() }

    fn children(&self) -> Option<&[AnyView]> { None }

    fn shape(&self) -> Shape { self.shape() }

    fn img_src(&self) -> Option<&PathBuf> { Some(&self.src) }

    fn layout(&self, cx: &mut LayoutCtx, shape: &mut Shape) {
        cx.assign_position(shape);
    }

    fn padding(&self) -> u32 { 0 }

    fn spacing(&self) -> u32 { 0 }

    fn alignment(&self) -> Alignment { Alignment::Vertical }
}

impl IntoView for Image {
    type V = Self;
    fn into_view(self) -> Self::V {
        self
    }
}
