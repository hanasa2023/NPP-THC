use iced::{
    highlighter,
    widget::{container, row, text, text_editor},
    Element, Fill,
    Length::FillPortion,
};

use crate::{Message, Tab};

#[derive(Debug, Clone)]
pub enum CalcCodeTabMessage {
    UpdatePyCode(String),
    UpdateRsCode(String),
    PyActionPerformed(text_editor::Action),
    RsActionPerformed(text_editor::Action),
}

#[derive(Default)]
pub struct CalcCodeTab {
    content_py: text_editor::Content,
    content_rs: text_editor::Content,
    is_dark: bool,
}

impl CalcCodeTab {
    pub fn new(is_dark: bool) -> Self {
        Self {
            content_py: text_editor::Content::default(),
            content_rs: text_editor::Content::default(),
            is_dark,
        }
    }

    pub fn update(&mut self, message: CalcCodeTabMessage) {
        match message {
            CalcCodeTabMessage::PyActionPerformed(action) => {
                self.content_py.perform(action);
            }
            CalcCodeTabMessage::UpdatePyCode(code) => {
                self.content_py = text_editor::Content::with_text(&code)
            }
            CalcCodeTabMessage::RsActionPerformed(action) => {
                self.content_rs.perform(action);
            }
            CalcCodeTabMessage::UpdateRsCode(code) => {
                self.content_rs = text_editor::Content::with_text(&code)
            }
        }
    }
}

impl Tab for CalcCodeTab {
    type Message = Message;

    fn tab_label(&self) -> iced_aw::TabLabel {
        let label = "计算代码".to_string();
        iced_aw::TabLabel::Text(label)
    }

    fn content(&self) -> iced::Element<Self::Message> {
        let editor_py: Element<CalcCodeTabMessage> = text_editor(&self.content_py)
            .height(Fill)
            .placeholder("未计算，无计算代码")
            .on_action(CalcCodeTabMessage::PyActionPerformed)
            .wrapping(text::Wrapping::Word)
            .highlight(
                "py",
                if self.is_dark {
                    highlighter::Theme::Base16Mocha
                } else {
                    highlighter::Theme::InspiredGitHub
                },
            )
            .into();

        let editor_rs: Element<CalcCodeTabMessage> = text_editor(&self.content_rs)
            .height(Fill)
            .placeholder("未计算，无计算代码")
            .on_action(CalcCodeTabMessage::RsActionPerformed)
            .wrapping(text::Wrapping::Word)
            .highlight(
                "rs",
                if self.is_dark {
                    highlighter::Theme::Base16Mocha
                } else {
                    highlighter::Theme::InspiredGitHub
                },
            )
            .into();

        let content_layout: Element<CalcCodeTabMessage> = row![
            container(editor_py).width(FillPortion(1)).height(Fill),
            container(editor_rs).width(FillPortion(1)).height(Fill),
        ]
        .spacing(10)
        .into();

        content_layout.map(Message::CalcCodeTab)
    }
}
