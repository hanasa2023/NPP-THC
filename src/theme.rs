use iced::{border, widget::button, Background, Font, Theme};

pub const MISANS_FONT: Font = Font::with_name("MiSans VF");
pub const HEADER_SIZE: u16 = 32;
pub const TAB_PADDING: u16 = 16;

pub fn background_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();

    let base = button::Style {
        background: Some(Background::Color(palette.background.base.color)),
        text_color: palette.background.base.text,
        border: border::rounded(2),
        ..button::Style::default()
    };

    let hovered = button::Style {
        background: Some(Background::Color(palette.background.weak.color)),
        ..base
    };

    let disabled = button::Style {
        background: base
            .background
            .map(|background| background.scale_alpha(0.5)),
        text_color: base.text_color.scale_alpha(0.5),
        ..base
    };

    match status {
        button::Status::Active | button::Status::Pressed => base,
        button::Status::Hovered => hovered,
        button::Status::Disabled => disabled,
    }
}
