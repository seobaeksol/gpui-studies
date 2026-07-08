use gpui::{
    div, prelude::*, px, rgb, size, uniform_list, App, Bounds, Context, Entity, Half, Hsla, Pixels,
    Point, SharedString, Window, WindowBounds, WindowOptions,
};
use gpui_platform::application;

struct RootView {
    text: SharedString,
    drag_drop: Entity<DragDrop>,
}

impl Render for RootView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .bg(rgb(0x505050))
            .w_full()
            .h_full()
            .justify_center()
            .items_center()
            .shadow_lg()
            .border_1()
            .border_color(rgb(0x0000ff))
            .text_xl()
            .text_color(rgb(0xffffff))
            .child(format!("Hello, {}!", &self.text))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        div()
                            .size_8()
                            .bg(gpui::red())
                            .border_1()
                            .border_dashed()
                            .rounded_md()
                            .border_color(gpui::white()),
                    )
                    .child(
                        div()
                            .size_8()
                            .bg(gpui::green())
                            .border_1()
                            .border_dashed()
                            .rounded_md()
                            .border_color(gpui::white()),
                    )
                    .child(
                        div()
                            .size_8()
                            .bg(gpui::blue())
                            .border_1()
                            .border_dashed()
                            .rounded_md()
                            .border_color(gpui::white()),
                    )
                    .child(
                        div()
                            .size_8()
                            .bg(gpui::yellow())
                            .border_1()
                            .border_dashed()
                            .rounded_md()
                            .border_color(gpui::white()),
                    )
                    .child(
                        div()
                            .size_8()
                            .bg(gpui::black())
                            .border_1()
                            .border_dashed()
                            .rounded_md()
                            .border_color(gpui::white()),
                    )
                    .child(
                        div()
                            .size_8()
                            .bg(gpui::white())
                            .border_1()
                            .border_dashed()
                            .rounded_md()
                            .border_color(gpui::black()),
                    ),
            )
            .child(
                div().size_full().bg(gpui::white()).child(
                    uniform_list(
                        "entries",
                        50,
                        cx.processor(|_this, range, _window, _cx| {
                            let mut items = Vec::new();
                            for ix in range {
                                let item = ix + 1;

                                items.push(
                                    div()
                                        .id(ix)
                                        .px_2()
                                        .cursor_pointer()
                                        .text_color(gpui::black())
                                        .on_click(move |_event, _window, _cx| {
                                            println!("clicked Item {item:?}");
                                        })
                                        .child(format!("Item {item}")),
                                );
                            }

                            items
                        }),
                    )
                    .h_full(),
                ),
            )
            .child(self.drag_drop.clone())
    }
}

#[derive(Clone, Copy)]
struct DragInfo {
    ix: usize,
    color: Hsla,
    position: Point<Pixels>,
}

impl DragInfo {
    fn new(ix: usize, color: Hsla) -> Self {
        Self {
            ix,
            color,
            position: Point::default(),
        }
    }

    fn position(mut self, pos: Point<Pixels>) -> Self {
        self.position = pos;
        self
    }
}

impl Render for DragInfo {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let size = gpui::size(px(120.), px(50.));

        div()
            .pl(self.position.x - size.width.half())
            .pt(self.position.y - size.height.half())
            .child(
                div()
                    .flex()
                    .justify_center()
                    .items_center()
                    .w(size.width)
                    .h(size.height)
                    .bg(self.color.opacity(0.5))
                    .text_color(gpui::white())
                    .text_xs()
                    .shadow_md()
                    .child(format!("Item {}", self.ix)),
            )
    }
}

struct DragDrop {
    drop_on: Option<DragInfo>,
}

impl DragDrop {
    fn new() -> Self {
        Self { drop_on: None }
    }
}

impl Render for DragDrop {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let items = [gpui::blue(), gpui::red(), gpui::green()];

        div()
            .size_full()
            .h_full()
            .bg(gpui::black())
            .flex()
            .flex_col()
            .gap_5()
            .justify_center()
            .items_center()
            .text_color(rgb(0x333333))
            .child(div().text_xl().text_center().child("Drag & Drop"))
            .child(
                div()
                    .w_full()
                    .mb_10()
                    .justify_center()
                    .flex()
                    .flex_row()
                    .gap_4()
                    .items_center()
                    .children(items.into_iter().enumerate().map(|(ix, color)| {
                        let drag_info = DragInfo::new(ix, color);

                        div()
                            .id(("item", ix))
                            .size_32()
                            .flex()
                            .justify_center()
                            .items_center()
                            .border_2()
                            .border_color(color)
                            .text_color(color)
                            .cursor_move()
                            .hover(|this| this.bg(color.opacity(0.2)))
                            .child(format!("Item ({})", ix))
                            .on_drag(drag_info, |info, position, _, cx| {
                                println!("On drag...");
                                cx.new(|_| info.position(position))
                            })
                    })),
            )
            .child(
                div()
                    .id("drop-target")
                    .w_128()
                    .h_32()
                    .flex()
                    .justify_center()
                    .items_center()
                    .border_3()
                    .border_color(self.drop_on.map(|info| info.color).unwrap_or(gpui::white()))
                    .when_some(self.drop_on, |this, info| this.bg(info.color.opacity(0.2)))
                    .on_drop(cx.listener(|this, info: &DragInfo, _, _| {
                        println!("On drop...!");
                        this.drop_on = Some(*info);
                    }))
                    .child("Drop items here"),
            )
    }
}

fn main() {
    application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(800.), px(800.0)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                let drag_drop = cx.new(|_| DragDrop::new());
                cx.new(|_| RootView {
                    text: "World".into(),
                    drag_drop,
                })
            },
        )
        .unwrap();

        cx.activate(true);
    });
}
