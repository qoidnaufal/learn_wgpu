# Learn Wgpu
Learn the inner process of a gui, especially the gpu-rendered one.<br>
Highly inspired by:
- [`leptos`](https://github.com/leptos-rs/leptos)
- [`cushy`](https://github.com/khonsulabs/cushy)
- [`floem`](https://github.com/lapce/floem)
- [`kludgine`](https://github.com/khonsulabs/kludgine)
- [`vger-rs`](https://github.com/audulus/vger-rs)

### Example
This is an example from the current (incomplete) works I've accomplished so far:

```rust
use learn_wgpu::prelude::*;

fn root() -> impl IntoView {
    let counter = signal(0i32);
    eprintln!("{}", counter.get());

    let c1 = counter.clone();
    let inc = move |el: &mut Element| {
        c1.set(|num| *num += 1);
        eprintln!("inc {}", c1.get());
        el.set_fill_color(|color| color.r += 150);
    };

    let c2 = counter.clone();
    let dec = move |el: &mut Element| {
        c2.set(|num| *num -= 1);
        eprintln!("dec {}", c2.get());
        el.set_fill_color(|color| color.r += 150);
    };

    let hover = |el: &mut Element| {
        el.set_fill_color(|color| *color = Rgba::BLUE);
    };
    let drag = |el: &mut Element| {
        el.set_fill_color(|color| *color = Rgba::GREEN);
    };

    stack([
        stack([
            image("assets/image1.jpg").into_any(),
            image("assets/image2.jpg").on_drag(drag).into_any(),
            TestTriangleWidget::new()
                .style(|style| {
                    if counter.get() % 2 == 0 {
                        style.set_fill_color(Rgba::BLACK);
                    } else {
                        style.set_fill_color(Rgba::RED);
                    };
                })
                .on_hover(hover)
                .into_any(),
        ])
        .style(|style| {
            style.set_orientation(Orientation::Horizontal);
            style.set_fill_color(Rgba::YELLOW);
            style.set_padding(20);
            style.set_spacing(20);
        })
        .on_drag(drag)
        .into_any(),
        stack([
            stack([
                button()
                    .style(|style| {
                        style.set_stroke_width(10.);
                        style.set_corners(|corners| {
                            corners.set_top_left(0.025);
                            corners.set_bot_left(0.025);
                            corners.set_bot_right(0.0);
                            corners.set_top_right(0.0);
                        });
                    })
                    .on_click(inc)
                    .on_hover(hover)
                    .into_any(),
                button()
                    .style(|style| {
                        style.set_stroke_width(5.);
                        style.set_corners(|r| r.set_each(0.039));
                    })
                    .on_click(dec)
                    .on_hover(hover)
                    .into_any(),
                button()
                    .style(|style| {
                        style.set_stroke_width(5.);
                        style.set_corners(|corners| {
                            corners.set_top_left(0.);
                            corners.set_bot_left(0.03);
                            corners.set_bot_right(0.);
                            corners.set_top_right(0.03);
                        });
                    })
                    .on_hover(hover)
                    .into_any(),
                button()
                    .style(|style| {
                        style.set_corners(|corners| corners.set_each(0.04));
                        style.set_fill_color(Rgba::new(69, 172, 23, 255));
                    })
                    .on_hover(hover)
                    .into_any(),
            ])
            .on_drag(drag)
            .style(|style| {
                style.set_fill_color(Rgba::new(111, 72, 234, 255));
                style.set_padding(20);
                style.set_spacing(20);
            })
            .into_any(),
            TestCircleWidget::new()
                .style(|style| {
                    style.set_stroke_width(10.);
                    style.set_fill_color(Rgba::BLACK);
                    style.set_stroke_color(Rgba::RED);
                })
                .on_hover(hover)
                .into_any(),
        ])
        .style(|style| {
            style.set_fill_color(Rgba::new(69, 69, 69, 255));
            style.set_orientation(Orientation::Horizontal);
            style.set_padding(20);
            style.set_spacing(20);
        })
        .on_drag(drag)
        .into_any(),
        TestCircleWidget::new()
            .style(|style| {
                style.set_fill_color(Rgba::new(169, 72, 43, 255));
                style.set_stroke_width(5.);
                style.set_stroke_color(Rgba::WHITE);
            })
            .on_drag(drag)
            .on_hover(hover)
            .into_any(),
    ])
    .style(|style| {
        style.set_padding(20);
        style.set_spacing(20);
    })
}

fn main() -> Result<(), learn_wgpu::Error> {
    launch(root)
}
```
