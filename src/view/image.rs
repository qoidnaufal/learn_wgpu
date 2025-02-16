use std::path::{Path, PathBuf};

use crate::storage::WidgetStorage;
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

    pub fn _on_hover<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_hover.insert(self.id(), f.into()));
        self
    }

    // pub fn on_click<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
    //     CALLBACKS.with_borrow_mut(|cbs| cbs.on_click.insert(self.id(), f.into()));
    //     self
    // }

    pub fn _on_drag<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_drag.insert(self.id(), f.into()));
        self
    }
}

impl View for Image {
    fn id(&self) -> NodeId { self.id() }

    fn children(&self) -> Option<&[AnyView]> { None }

    fn shape(&self) -> Shape { self.shape() }

    fn img_src(&self) -> Option<&PathBuf> { Some(&self.src) }

    fn layout(&self, cx: &mut WidgetStorage, shape: &mut Shape) {
        cx.layout.assign_position(shape);
        // let half = self.shape().dimensions / 2;
        // let current_pos = if cx.get_parent(&self.id()).is_some() {
        //     cx.layout.next_child_pos()
        // } else {
        //     cx.layout.next_pos()
        // };
        // shape.pos = current_pos + half;
    }

    fn padding(&self) -> u32 { 0 }

    fn spacing(&self) -> u32 { 0 }
}

impl IntoView for Image {
    type V = Self;
    fn into_view(self) -> Self::V {
        self
    }
}
