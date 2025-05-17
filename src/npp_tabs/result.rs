mod result_formatter;
use result_formatter::format_result_to_markdown;

use calc::parameters;
use iced::{
    widget::markdown,
    widget::{container, scrollable},
    Element, Length, Theme,
};

use crate::{Message, Tab};

#[derive(Debug, Clone)]
pub enum ResultMessage {
    UpdateResult(Box<parameters::CalcResultParamters>),
    LinkClicked(markdown::Url),
}

#[derive(Default)]
pub struct ResultTab {
    theme: Theme,
    result_markdown: Vec<markdown::Item>,
}

impl ResultTab {
    pub fn update(&mut self, message: ResultMessage) {
        match message {
            ResultMessage::UpdateResult(result) => {
                let result_markdown_string = format_result_to_markdown(&result);
                self.result_markdown = markdown::parse(&result_markdown_string).collect();
            }
            ResultMessage::LinkClicked(url) => {
                log::info!("点击链接: {}", url);
            }
        }
    }
}

impl Tab for ResultTab {
    type Message = Message;

    fn tab_label(&self) -> iced_aw::TabLabel {
        iced_aw::TabLabel::Text("计算结果".to_string())
    }

    fn content(&self) -> iced::Element<Self::Message> {
        let md_view = markdown::view(
            &self.result_markdown,
            markdown::Settings::default(),
            markdown::Style::from_palette(self.theme.palette()),
        )
        .map(ResultMessage::LinkClicked);

        let scrollable_content: Element<ResultMessage> =
            scrollable(container(md_view).width(Length::Fill).padding(10))
                .width(Length::Fill)
                .height(Length::Fill)
                .into();

        scrollable_content.map(Message::ResultTab)
    }
}
