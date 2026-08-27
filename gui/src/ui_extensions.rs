use imgui_app::ImguiExt;
use imgui_app::dear_imgui_rs::{GroupToken, StyleVar, Ui};

pub trait UiExt {
    fn text_aligned_right<S: AsRef<str>>(&self, text: S);
    fn text_aligned_center<S: AsRef<str>>(&self, text: S);
    fn text_aligned_center_center<S: AsRef<str>>(&self, text: S);
    fn checkbox_small<S: AsRef<str>>(&self, label: S, checked: &mut bool) -> bool;
    fn calc_text_width<S: AsRef<str>>(&self, text: S) -> f32;
    fn calc_button_size<S: AsRef<str>>(&self, label: S) -> [f32; 2];
    fn calc_button_width<S: AsRef<str>>(&self, label: S) -> f32;
    fn calc_checkbox_size<S: AsRef<str>>(&self, label: S) -> [f32; 2];
    fn calc_checkbox_width<S: AsRef<str>>(&self, label: S) -> f32;
    fn widget_group_begin(&self) -> GroupToken<'_>;
    fn widget_group_label<S: AsRef<str>>(&self, label: S);
}

impl UiExt for Ui {
    fn text_aligned_right<S: AsRef<str>>(&self, text: S) {
        let [width, _] = self.calc_text_size(text.as_ref());
        self.align_next_item_right(width);
        self.text(text);
    }
    
    fn text_aligned_center<S: AsRef<str>>(&self, text: S) {
        let [width, _] = self.calc_text_size(text.as_ref());
        self.align_next_item_center(width);
        self.text(text);
    }
    
    fn text_aligned_center_center<S: AsRef<str>>(&self, text: S) {
        let [width, height] = self.calc_text_size(text.as_ref());
        self.align_next_item_center(width);
        
        let height_avail = self.content_region_avail_height();
        if height_avail > height {
            let delta_y = f32::round((height_avail - height) / 2.0);
            self.set_cursor_pos_y(self.cursor_pos_y() + delta_y);
        }
        
        self.text(text);
    }
    
    fn checkbox_small<S: AsRef<str>>(&self, label: S, checked: &mut bool) -> bool {
        self.set_cursor_pos_y(self.cursor_pos_y() + unsafe { self.style().frame_padding()[1] });
        let _padding = self.push_style_var(StyleVar::FramePadding([0.0, 0.0]));
        self.checkbox(label, checked)
    }
    
    fn calc_text_width<S: AsRef<str>>(&self, text: S) -> f32 {
        self.calc_text_size(text.as_ref())[0]
    }
    
        fn calc_button_size<S: AsRef<str>>(&self, label: S) -> [f32; 2] {
        let [label_width, label_height] = self.calc_text_size(label.as_ref());
        let [padding_x, padding_y] = unsafe { self.style().frame_padding() };
        return [
            label_width + padding_x * 2.0,
            label_height + padding_y * 2.0
        ];
    }
    
    fn calc_button_width<S: AsRef<str>>(&self, label: S) -> f32 {
        return self.calc_button_size(label)[0];
    }
    
    fn calc_checkbox_size<S: AsRef<str>>(&self, label: S) -> [f32; 2] {
        let style = unsafe { self.style() };
        let padding_y = style.frame_padding()[1];
        let frame_height = self.current_font_size() + padding_y * 2.0;
        let spacing_x = style.item_inner_spacing()[0];
        let label_width = self.calc_text_size(label.as_ref())[0];
        return [
            frame_height + spacing_x + label_width,
            frame_height,
        ];
    }
    
    fn calc_checkbox_width<S: AsRef<str>>(&self, label: S) -> f32 {
        return self.calc_checkbox_size(label)[0];
    }
    
    fn widget_group_begin(&self) -> GroupToken<'_> {
        let widget_col_width = f32::round(self.window_width() * 0.65);
        let widget_col_x = self.cursor_pos_x() + self.content_region_avail_width() - widget_col_width;
        self.set_cursor_pos_x(widget_col_x);
        self.begin_group()
    }
    
    fn widget_group_label<S: AsRef<str>>(&self, label: S) {
        let label_width = self.calc_text_width(&label);
        let inner_spacing_x = unsafe { self.style().item_inner_spacing()[0] };
        self.set_cursor_pos_x(self.cursor_pos_x() - inner_spacing_x - label_width);
        self.align_text_to_frame_padding();
        self.text(label);
        self.same_line_with_spacing(0.0, inner_spacing_x);
        self.set_next_item_width(-1.0);
    }
}
