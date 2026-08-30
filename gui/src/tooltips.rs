use imgui_app::dear_imgui_rs::{ItemHoveredFlags, Ui};

use crate::ui_extensions::UiExt;

static mut SHOW_TOOLTIPS: bool = false;

pub fn tooltips_are_enabled() -> bool {
    unsafe { SHOW_TOOLTIPS }
}

pub fn set_tooltips_enabled(enabled: bool) {
    unsafe { SHOW_TOOLTIPS = enabled; }
}

pub fn toggle_tooltips() {
    unsafe {
        SHOW_TOOLTIPS = !SHOW_TOOLTIPS;
    }
}

pub fn tooltip(ui: &Ui, text: &str) {
    if unsafe { !SHOW_TOOLTIPS } || !ui.is_item_hovered_with_flags(ItemHoveredFlags::ALLOW_WHEN_DISABLED) {
        return;
    }

    let style = unsafe { ui.style() };
    let text_max_width = f32::min(600.0, {
        ui.window_viewport().size()[0]
            - style.display_safe_area_padding()[0] * 2.0
            - style.window_padding()[0] * 2.0
            - style.window_border_size() * 2.0
    });
    let text_width_unwrapped = ui.calc_text_size(text)[0];
    
    if text_width_unwrapped > text_max_width {
        let text_width_wrapped = ui.calc_text_size_with_opts(text, false, text_max_width)[0];
        ui.set_next_window_content_size([text_width_wrapped, 0.0]);
        ui.tooltip(|| {
            ui.text_wrapped(text);
        });
    }
    else {
        ui.tooltip(|| {
            ui.text(text);
        });
    }
}
