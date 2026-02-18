use std::path::PathBuf;

use gpui::Window;

use crate::scan;
use crate::state::FileBrowserState;

pub fn refresh(window: &mut Window, cx: &mut gpui::App) {
    let root = {
        let state = cx.global::<FileBrowserState>();
        state.root.lock().unwrap().clone()
    };

    scan::start_scan(root, window, cx);
}

pub fn set_root_and_rescan(new_root: PathBuf, window: &mut Window, cx: &mut gpui::App) {
    {
        let state = cx.global::<FileBrowserState>();
        *state.root.lock().unwrap() = new_root;
        *state.selected_path.lock().unwrap() = None;
        *state.context_path.lock().unwrap() = None;
        *state.menu_open.lock().unwrap() = false;
        *state.menu_position.lock().unwrap() = None;
    }

    refresh(window, cx);
}

pub fn prompt_for_root(window: &mut Window, cx: &mut gpui::App) {
    let receiver = cx.prompt_for_paths(gpui::PathPromptOptions {
        files: false,
        directories: true,
        multiple: false,
        prompt: Some("Select root directory".into()),
    });

    window
        .spawn(cx, async move |cx| {
            let result = receiver.await;
            cx.update(|window, cx| {
                let selected = match result {
                    Ok(Ok(Some(paths))) => paths.into_iter().next(),
                    _ => None,
                };

                if let Some(path) = selected {
                    set_root_and_rescan(path, window, cx);
                }
            })
            .ok();
        })
        .detach();
}

