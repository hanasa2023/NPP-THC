use crate::common::theme::background_button_style;
use iced::{
    alignment, padding,
    widget::{button, center, container, mouse_area, opaque, row, stack, text, text_input},
    Element, Length, Renderer, Theme,
};

pub fn input_field<'a, F, M>(
    label: &'a str,
    placeholder: &'a str,
    value: &'a str,
    on_input_message_creator: F,
) -> Element<'a, M>
where
    F: Fn(String) -> M + 'a,
    M: Clone + 'a,
{
    row![
        container(text(label).align_x(alignment::Horizontal::Right)) // 标签右对齐
            .width(Length::Fixed(250.0))
            .padding(padding::right(4)),
        text_input(placeholder, value)
            .on_input(on_input_message_creator)
            .width(Length::Fill)
    ]
    .spacing(10)
    .align_y(alignment::Vertical::Center)
    .into()
}

pub fn labeled_button<'a, M>(label: &'a str, msg: M) -> button::Button<'a, M, Theme, Renderer>
where
    M: Clone + 'a,
{
    button(text(label).center())
        .style(background_button_style)
        .on_press(msg)
}

pub fn modal<'a, Message>(
    base: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
    on_blur: Message,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    stack![
        base.into(),
        opaque(mouse_area(center(opaque(content))).on_press(on_blur))
    ]
    .into()
}
