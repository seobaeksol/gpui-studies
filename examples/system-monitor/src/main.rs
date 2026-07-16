use gpui::{App, Bounds, Context, Window, WindowBounds, WindowOptions, div, prelude::*, px, size};
use gpui_platform::application;

struct SystemMonitorView;

impl Render for SystemMonitorView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .justify_center()
            .items_center()
            .child("System Monitor")
    }
}

fn main() {
    application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(960.), px(640.)), cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            ..Default::default()
        };

        cx.open_window(options, |_window, cx| cx.new(|_| SystemMonitorView))
            .expect("failed to open system monitor window");
        cx.activate(true);
    });
}
