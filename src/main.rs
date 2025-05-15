mod errors;

mod helpers;

mod theme;
use theme::{HEADER_SIZE, MISANS_FONT, TAB_PADDING};

mod input;

use calc::parameters;

use iced::{
    widget::{button, column as col, container, text, Text},
    Alignment, Element, Length, Padding, Renderer, Settings, Task, Theme,
};
use iced_aw::{
    menu::{Item, Menu},
    menu_bar, menu_items, TabLabel,
};

fn main() -> iced::Result {
    let settings = Settings {
        default_font: MISANS_FONT,
        ..Settings::default()
    };
    iced::application(App::get_title, App::update, App::view)
        .theme(App::get_theme)
        .settings(settings)
        .run_with(App::new)
}

struct App {
    app_name: String,
    theme: Theme,
    status: String,
    output_dir: String,
    caculator: calc::Calculator,
}

#[derive(Debug, Clone)]
enum Message {
    FontLoaded(Result<(), iced::font::Error>),
    OpenSubMenu,
    LoadParamsFromFile,
    LoadedParamsFromFile(Result<Box<parameters::CalcInputParameters>, errors::Error>),
    SaveInputParams,
    SelectOutputDir,
    LoadDefaultParams,
    ClearInputParams,
    SaveCalcCode,
    Calculate,
    OpenHelpDialog,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let app = Self {
            app_name: String::from("核电厂热力计算程序"),
            theme: Theme::CatppuccinMocha,
            status: String::new(),
            output_dir: String::new(),
            caculator: calc::Calculator::default(),
        };
        let command = Task::batch(vec![iced::font::load(
            include_bytes!("../fonts/MiSans VF.ttf").as_slice(),
        )
        .map(Message::FontLoaded)]);
        (app, command)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FontLoaded(result) => {
                match result {
                    Ok(_) => self.status = String::from("字体加载成功"),
                    Err(_) => self.status = String::from("字体加载失败"),
                }
                Task::none()
            }
            Message::OpenSubMenu => Task::none(),
            Message::LoadParamsFromFile => {
                self.status = String::from("加载参数文件");
                Task::perform(
                    helpers::load_input_params_from_file(),
                    Message::LoadedParamsFromFile,
                )
            }
            Message::LoadedParamsFromFile(result) => {
                if let Ok(input_params) = result {
                    self.caculator.set_input_params(*input_params);
                    self.status = String::from("加载参数成功");
                } else {
                    self.status = String::from("加载参数失败");
                }

                Task::none()
            }
            Message::SaveInputParams => Task::none(),
            Message::SelectOutputDir => Task::none(),
            Message::LoadDefaultParams => {
                self.caculator
                    .set_input_params(parameters::CalcInputParameters::from_default());
                Task::none()
            }
            Message::ClearInputParams => {
                self.caculator
                    .set_input_params(parameters::CalcInputParameters::default());

                Task::none()
            }
            Message::SaveCalcCode => {
                match self.caculator.save_code_to_file(&self.output_dir) {
                    Ok(_) => self.status = String::from("保存计算代码成功"),
                    Err(err) => self.status = format!("保存计算代码失败: {err}"),
                }

                Task::none()
            }
            Message::Calculate => {
                match self.caculator.calculate() {
                    Ok(_) => self.status = String::from("计算成功"),
                    Err(err) => self.status = format!("计算失败: {err}"),
                }
                Task::none()
            }
            Message::OpenHelpDialog => Task::none(),
        }
    }

    fn view(&self) -> Element<Message> {
        #[rustfmt::skip]
        let menubar = menu_bar!(
            (labeled_button("文件", Message::OpenSubMenu).width(Length::Shrink), {
                Menu::new(menu_items!(
                    (labeled_button("加载参数", Message::LoadParamsFromFile).width(Length::Fill))
                    (labeled_button("保存参数", Message::SaveInputParams).width(Length::Fill))
                    (labeled_button("选择输出目录", Message::SelectOutputDir).width(Length::Fill))
                )).max_width(180.0)
            })
            (labeled_button("计算", Message::OpenSubMenu).width(Length::Shrink), {
                Menu::new(menu_items!(
                    (labeled_button("加载默认参数", Message::LoadDefaultParams).width(Length::Fill))
                    (labeled_button("清空输入参数", Message::ClearInputParams).width(Length::Fill))
                    (labeled_button("保存计算代码", Message::SaveCalcCode).width(Length::Fill))
                    (labeled_button("开始计算", Message::Calculate).width(Length::Fill))
                )).max_width(180.0)
            })
            (labeled_button("帮助", Message::OpenSubMenu).width(Length::Shrink), {
                Menu::new(menu_items!(
                    (labeled_button("关于", Message::OpenHelpDialog).width(Length::Fill))
                )).max_width(180.0)
            })
        ).padding(Padding::from([8.0, 0.0]));

        let content = container("content").center(Length::Fill);

        let v = col![menubar, content];

        v.into()
    }

    fn get_theme(&self) -> Theme {
        self.theme.clone()
    }

    fn get_title(&self) -> String {
        self.app_name.clone()
    }
}

fn labeled_button(label: &str, msg: Message) -> button::Button<Message, Theme, Renderer> {
    button(text(label).center())
        .style(theme::background_button_style)
        .on_press(msg)
}

trait Tab {
    type Message;

    fn title(&self) -> String;

    fn tab_label(&self) -> TabLabel;

    fn view(&self) -> Element<Self::Message> {
        let column = col![Text::new(self.title()).size(HEADER_SIZE), self.content()]
            .spacing(20)
            .align_x(Alignment::Center);

        container(column)
            .center(Length::Fill)
            .padding(TAB_PADDING)
            .into()
    }

    fn content(&self) -> Element<Self::Message>;
}
