mod todo;
mod todo_app;
mod state;
mod components;

use gpui::{AppContext, Application, App, WindowOptions, px, size};
use yororen_ui::assets::UiAsset;
use yororen_ui::component;
use yororen_ui::theme::GlobalTheme;

fn main() {
    let app = Application::new().with_assets(UiAsset);

    app.run(|cx: &mut App| {
        component::init(cx);
        cx.set_global(GlobalTheme::new(cx.window_appearance()));
        cx.set_global(state::TodoState::default());

        let options = WindowOptions {
            window_bounds: Some(gpui::WindowBounds::Windowed(gpui::Bounds::centered(
                None,
                size(px(800.0), px(600.0)),
                cx,
            ))),
            ..Default::default()
        };

        cx.open_window(options, |_, cx| {
            cx.new(|cx| todo_app::TodoApp::new(cx))
        }).unwrap();
    });
}
