use crate::{Message, Tab};
use calc::parameters;

#[derive(Debug, Clone)]
pub enum InputMessage {
    ValueChanged(f64),
}

#[derive(Default)]
pub struct InputTab {
    pub input: parameters::CalcInputParameters,
}

impl InputTab {
    fn new() -> Self {
        Self {
            input: parameters::CalcInputParameters::default(),
        }
    }

    fn update(&mut self, message: InputMessage) {
        match message {
            InputMessage::ValueChanged(value) => {}
        }
    }
}

impl Tab for InputTab {
    type Message = Message;

    fn title(&self) -> String {
        todo!()
    }

    fn tab_label(&self) -> iced_aw::TabLabel {
        todo!()
    }

    fn content(&self) -> iced::Element<Self::Message> {
        todo!()
    }
}
