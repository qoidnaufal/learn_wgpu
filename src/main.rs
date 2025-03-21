mod app;
mod callback;
mod color;
mod error;
mod context;
mod renderer;
mod shapes;
mod signal;
mod storage;
mod view;

use app::launch;
use color::*;
use shapes::Shape;
use signal::Signal;
use view::*;

use error::Error;

fn root() -> impl IntoView {
    let counter = Signal::new(0i32);
    eprintln!("init {}", counter.get());

    let c1 = counter.clone();
    let inc = move |shape: &mut Shape| {
        c1.set(|num| *num += 1);
        eprintln!("inc1 {}", c1.get());
        shape.set_color(|color| color.r += 150);
    };

    let c2 = counter.clone();
    let shift_left = move |shape: &mut Shape| {
        c2.set(|num| *num <<= 1);
        eprintln!("shift left {}", c2.get());
        shape.set_color(|color| color.r += 150);
    };

    let c3 = counter.clone();
    let dec = move |shape: &mut Shape| {
        c3.set(|num| *num -= 1);
        eprintln!("dec {}", c3.get());
        shape.set_color(|color| color.r += 150);
    };

    let c4 = counter.clone();
    let shift_right = move |shape: &mut Shape| {
        c4.set(|num| *num >>= 1);
        eprintln!("shift right {}", c4.get());
        shape.set_color(|color| color.r += 150);
    };

    let hover = move |shape: &mut Shape| { shape.set_color(|color| *color = Rgb::BLUE) };
    let drag = move |shape: &mut Shape| {
        shape.set_color(|color| *color = Rgb::GREEN);
    };

    vstack(
        [
            hstack(
                [
                    image("assets/image1.jpg").into_any(),
                    image("assets/image2.jpg").on_drag(drag).into_any(),
                    TestTriangleWidget::new().on_hover(hover).into_any(),
                ]
            ).on_drag(drag).into_any(),
            hstack(
                [
                    vstack(
                        [
                            button()
                                .style(|shape| shape.set_radius(0.5))
                                .on_click(shift_right)
                                .on_hover(hover)
                                .into_any(),
                            button()
                                .style(|shape| shape.set_radius(0.5))
                                .on_click(shift_left)
                                .on_hover(hover)
                                .into_any(),
                            button()
                                .style(|shape| shape.set_radius(0.5))
                                .on_click(dec)
                                .on_hover(hover)
                                .into_any(),
                            button()
                                .style(|shape| shape.set_radius(0.5))
                                .on_click(inc)
                                .on_hover(hover)
                                .into_any(),
                        ]
                    ).on_drag(drag).into_any(),
                    TestCircleWidget::new().on_hover(hover).into_any(),
                ]
            ).into_any(),
            TestCircleWidget::new()
                .on_drag(drag)
                .on_hover(hover)
                .into_any(),
        ]
    )
}

fn dummy() -> impl IntoView {
    let hover = move |shape: &mut Shape| { shape.set_color(|color| *color = Rgb::BLUE) };
    let drag = move |shape: &mut Shape| {
        shape.set_color(|color| *color = Rgb::GREEN);
    };

    TestCircleWidget::new().on_hover(hover).on_drag(drag)
}

fn main() -> Result<(), Error> {
    let mut args = std::env::args();
    match args.nth(1) {
        Some(arg) if arg == "dummy" => launch(dummy),
        _ => launch(root)
    }
}
