use std::cmp;

use crate::{
    app::{AppData, Nibble},
    label::LabelHandler,
    screen::ScreenHandler,
};

use super::{adjust_offset, KeyHandler, Window};

/// The main windows that allow users to edit HEX and ASCII.
#[derive(PartialEq, Eq, Clone, Copy)]
pub(crate) enum Editor {
    Ascii,
    Hex,
}

impl KeyHandler for Editor {
    fn is_focusing(&self, window_type: Window) -> bool {
        match self {
            Self::Ascii => window_type == Window::Ascii,
            Self::Hex => window_type == Window::Hex,
        }
    }
    fn left(&mut self, app: &mut AppData, display: &mut ScreenHandler, labels: &mut LabelHandler) {
        match self {
            Self::Ascii => {
                app.offset = app.offset.saturating_sub(1);
                labels.update_all(&app.contents[app.offset..]);
                adjust_offset(app, display, labels);
            }
            Self::Hex => {
                if app.nibble == Nibble::Beginning {
                    app.offset = app.offset.saturating_sub(1);
                    labels.update_all(&app.contents[app.offset..]);
                    adjust_offset(app, display, labels);
                }
                app.nibble.toggle();
            }
        }
    }
    fn right(&mut self, app: &mut AppData, display: &mut ScreenHandler, labels: &mut LabelHandler) {
        match self {
            Self::Ascii => {
                app.offset = cmp::min(app.offset.saturating_add(1), app.contents.len() - 1);
                labels.update_all(&app.contents[app.offset..]);
                adjust_offset(app, display, labels);
            }
            Self::Hex => {
                if app.nibble == Nibble::End {
                    app.offset = cmp::min(app.offset.saturating_add(1), app.contents.len() - 1);
                    labels.update_all(&app.contents[app.offset..]);
                    adjust_offset(app, display, labels);
                }
                app.nibble.toggle();
            }
        }
    }
    fn up(&mut self, app: &mut AppData, display: &mut ScreenHandler, labels: &mut LabelHandler) {
        if let Some(new_offset) = app.offset.checked_sub(display.comp_layouts.bytes_per_line) {
            app.offset = new_offset;
            labels.update_all(&app.contents[app.offset..]);
            adjust_offset(app, display, labels);
        }
    }
    fn down(&mut self, app: &mut AppData, display: &mut ScreenHandler, labels: &mut LabelHandler) {
        if let Some(new_offset) = app.offset.checked_add(display.comp_layouts.bytes_per_line) {
            if new_offset < app.contents.len() {
                app.offset = new_offset;
                labels.update_all(&app.contents[app.offset..]);
                adjust_offset(app, display, labels);
            }
        }
    }
    fn home(&mut self, app: &mut AppData, display: &mut ScreenHandler, labels: &mut LabelHandler) {
        let bytes_per_line = display.comp_layouts.bytes_per_line;
        app.offset = app.offset / bytes_per_line * bytes_per_line;
        labels.update_all(&app.contents[app.offset..]);
        adjust_offset(app, display, labels);

        if self.is_focusing(Window::Hex) {
            app.nibble = Nibble::Beginning;
        }
    }
    fn end(&mut self, app: &mut AppData, display: &mut ScreenHandler, labels: &mut LabelHandler) {
        let bytes_per_line = display.comp_layouts.bytes_per_line;
        app.offset = cmp::min(
            app.offset + (bytes_per_line - 1 - app.offset % bytes_per_line),
            app.contents.len() - 1,
        );
        labels.update_all(&app.contents[app.offset..]);
        adjust_offset(app, display, labels);

        if self.is_focusing(Window::Hex) {
            app.nibble = Nibble::End;
        }
    }
    fn backspace(
        &mut self,
        app: &mut AppData,
        display: &mut ScreenHandler,
        labels: &mut LabelHandler,
    ) {
        if app.offset > 0 {
            app.contents.remove(app.offset - 1);
            app.offset = app.offset.saturating_sub(1);
            labels.update_all(&app.contents[app.offset..]);
            adjust_offset(app, display, labels);
        }
    }
    fn delete(
        &mut self,
        app: &mut AppData,
        display: &mut ScreenHandler,
        labels: &mut LabelHandler,
    ) {
        if app.contents.len() > 1 {
            app.contents.remove(app.offset);
            labels.update_all(&app.contents[app.offset..]);
            adjust_offset(app, display, labels);
        }
    }
    fn char(
        &mut self,
        app: &mut AppData,
        display: &mut ScreenHandler,
        labels: &mut LabelHandler,
        c: char,
    ) {
        match *self {
            Self::Ascii => {
                app.contents[app.offset] = c as u8;
                app.offset = cmp::min(app.offset.saturating_add(1), app.contents.len() - 1);
                labels.update_all(&app.contents[app.offset..]);
                adjust_offset(app, display, labels);
            }
            Self::Hex => {
                if c.is_ascii_hexdigit() {
                    // This can probably be optimized...
                    match app.nibble {
                        Nibble::Beginning => {
                            let mut src = c.to_string();
                            src.push(
                                format!("{:02X}", app.contents[app.offset]).chars().last().unwrap(),
                            );
                            let changed = u8::from_str_radix(src.as_str(), 16).unwrap();
                            app.contents[app.offset] = changed;
                        }
                        Nibble::End => {
                            let mut src = format!("{:02X}", app.contents[app.offset])
                                .chars()
                                .take(1)
                                .collect::<String>();
                            src.push(c);
                            let changed = u8::from_str_radix(src.as_str(), 16).unwrap();
                            app.contents[app.offset] = changed;

                            // Move to the next byte
                            app.offset =
                                cmp::min(app.offset.saturating_add(1), app.contents.len() - 1);
                            labels.update_all(&app.contents[app.offset..]);
                            adjust_offset(app, display, labels);
                        }
                    }
                    app.nibble.toggle();
                } else {
                    labels.notification = format!("Invalid Hex: {c}");
                }
            }
        }
    }

    fn enter(&mut self, _: &mut AppData, _: &mut ScreenHandler, _: &mut LabelHandler) {}
}
