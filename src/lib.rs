mod callback;
mod renderer;
mod cursor;

pub mod app;
pub mod color;
pub mod element;
pub mod error;
pub mod layout;
pub mod reactive;
pub mod style;
pub mod tree;
pub mod view;

pub mod prelude {
    use crate::error::Error;
    use crate::app::App;

    pub use crate::reactive::{signal, Get, Set};
    pub use crate::color::Rgba;
    pub use crate::element::Element;
    pub use crate::style::Orientation;
    pub use crate::view::{
        IntoView,
        TestCircleWidget,
        TestTriangleWidget,
        stack,
        button,
        image
    };

    pub type AppResult = Result<(), Error>;

    pub fn launch<F, IV>(f: F) -> Result<(), Error>
    where
        F: Fn() -> IV + 'static,
        IV: IntoView + 'static,
    {
        App::new(f).run()
    }
}
