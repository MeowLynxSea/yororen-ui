use gpui::{Keystroke, Modifiers, SharedString};

pub fn format_keybinding_ui(keystroke: &Keystroke) -> SharedString {
    let mut out = String::new();
    push_modifiers(&mut out, &keystroke.modifiers);
    push_key(&mut out, &keystroke.key);
    out.into()
}

fn push_modifiers(out: &mut String, m: &Modifiers) {
    if m.control {
        out.push('⌃');
    }
    if m.alt {
        out.push('⌥');
    }
    if m.shift {
        out.push('⇧');
    }
    if m.platform {
        out.push('⌘');
    }
    if m.function {
        out.push_str("fn ");
    }
}

fn push_key(out: &mut String, key: &str) {
    let k = key.to_ascii_lowercase();
    let pretty = match k.as_str() {
        "escape" | "esc" => "⎋".to_string(),
        "enter" => "↩".to_string(),
        "tab" => "⇥".to_string(),
        "backspace" => "⌫".to_string(),
        "delete" => "⌦".to_string(),
        "space" => "␣".to_string(),
        "left" => "←".to_string(),
        "right" => "→".to_string(),
        "up" => "↑".to_string(),
        "down" => "↓".to_string(),
        other if other.len() == 1 => other.to_ascii_uppercase(),
        other => other.to_string(),
    };

    out.push_str(&pretty);
}
