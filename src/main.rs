mod common;
mod components;
mod npp_tabs;

use npp_tabs::{
    calc_code::{CalcCodeTab, CalcCodeTabMessage},
    input::{InputTab, InputTabMessage},
    result::{ResultMessage, ResultTab},
};

use common::{
    errors, helpers,
    theme::{MISANS_FONT, TAB_PADDING},
};

use components::labeled_button;

use calc::parameters;

use iced::{
    widget::{column as col, container, horizontal_space, row, text},
    Alignment, Element, Length, Padding, Settings, Task, Theme,
};
use iced_aw::{
    menu::{Item, Menu},
    menu_bar, menu_items, TabBarPosition, TabLabel, Tabs,
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

#[derive(Clone, PartialEq, Eq, Debug, Default)]
enum TabId {
    #[default]
    Input,
    Result,
    CalcCode,
}

struct App {
    app_name: String,
    theme: Theme,
    status: String,
    pending_action: Option<PendingAction>,
    output_dir: String,
    caculator: calc::Calculator,
    active_tab: TabId,
    input_tab: InputTab,
    result_tab: ResultTab,
    calc_code_tab: CalcCodeTab,
}

#[derive(Debug, Clone)]
enum Message {
    FontLoaded(Result<(), iced::font::Error>),
    OpenSubMenu,
    LoadParamsFromFile,
    LoadedParamsFromFile(Result<Box<parameters::CalcInputParameters>, errors::Error>),
    SaveInputParams,
    SelectOutputDir,
    SelectedOutputDir(Result<String, errors::Error>),
    LoadDefaultParams,
    ClearInputParams,
    SaveResult,
    SaveCalcCode,
    Calculate,
    ThemeSelect(Theme),
    OpenHelpDialog,
    // Tab消息
    TabSelected(TabId),
    InputTab(InputTabMessage),
    ResultTab(ResultMessage),
    CalcCodeTab(CalcCodeTabMessage),
}

#[derive(Debug, Clone)]
enum PendingAction {
    InputParams,
    Result,
    CalcCode,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let app = Self {
            app_name: String::from("核电厂热力计算程序"),
            theme: Theme::CatppuccinMocha,
            status: String::new(),
            pending_action: None,
            output_dir: String::new(),
            caculator: calc::Calculator::default(),
            active_tab: TabId::Input,
            input_tab: InputTab::default(),
            result_tab: ResultTab::default(),
            calc_code_tab: CalcCodeTab::new(true),
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
                    self.input_tab
                        .update(InputTabMessage::UpdateParams(Box::new(
                            self.caculator.params.clone(),
                        )));
                    self.status = String::from("加载参数成功");
                } else {
                    self.status = String::from("加载参数失败");
                }

                Task::none()
            }
            Message::SaveInputParams => {
                if self.output_dir.is_empty() {
                    self.pending_action = Some(PendingAction::InputParams);
                    return Task::perform(helpers::select_output_dir(), Message::SelectedOutputDir);
                }
                match self.caculator.save_parameters_to_file(&self.output_dir) {
                    Ok(_) => self.status = "保存输入参数成功".to_string(),
                    Err(error) => self.status = format!("保存输入参数失败{error}"),
                }
                Task::none()
            }
            Message::SelectOutputDir => {
                Task::perform(helpers::select_output_dir(), Message::SelectedOutputDir)
            }
            Message::SelectedOutputDir(result) => {
                match result {
                    Ok(path) => {
                        self.output_dir = path.clone();
                        if let Some(action) = self.pending_action.clone() {
                            match action {
                                PendingAction::InputParams => {
                                    self.pending_action = None;
                                    match self.caculator.save_parameters_to_file(&self.output_dir) {
                                        Ok(_) => self.status = "保存输入参数成功".to_string(),
                                        Err(error) => {
                                            self.status = format!("保存输入参数失败{error}")
                                        }
                                    }
                                }
                                PendingAction::Result => {
                                    self.pending_action = None;
                                    match self.caculator.save_results_to_file(&self.output_dir) {
                                        Ok(_) => self.status = "保存计算结果成功".to_string(),
                                        Err(error) => {
                                            self.status = format!("保存计算结果失败{error}")
                                        }
                                    }
                                }
                                PendingAction::CalcCode => {
                                    self.pending_action = None;
                                    match self.caculator.save_code_to_file(&self.output_dir) {
                                        Ok(_) => self.status = "保存计算代码成功".to_string(),
                                        Err(error) => {
                                            self.status = format!("保存计算代码失败{error}")
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => {
                        self.status = "选择输出目录失败".to_string();
                        self.pending_action = None;
                    }
                }
                Task::none()
            }
            Message::LoadDefaultParams => {
                self.caculator.params = parameters::CalcInputParameters::from_default();
                self.input_tab
                    .update(InputTabMessage::UpdateParams(Box::new(
                        self.caculator.params.clone(),
                    )));
                self.status = "加载默认参数成功".to_string();
                Task::none()
            }
            Message::ClearInputParams => {
                self.caculator.params = parameters::CalcInputParameters::default();
                self.input_tab.update(InputTabMessage::ClearParams);
                self.status = "清除输入参数成功".to_string();
                Task::none()
            }
            Message::SaveResult => {
                if self.output_dir.is_empty() {
                    self.pending_action = Some(PendingAction::Result);
                    return Task::perform(helpers::select_output_dir(), Message::SelectedOutputDir);
                }
                match self.caculator.save_results_to_file(&self.output_dir) {
                    Ok(_) => self.status = "保存计算结果成功".to_string(),
                    Err(error) => self.status = format!("保存计算结果失败{error}"),
                }
                Task::none()
            }
            Message::SaveCalcCode => {
                if self.output_dir.is_empty() {
                    self.pending_action = Some(PendingAction::CalcCode);
                    return Task::perform(helpers::select_output_dir(), Message::SelectedOutputDir);
                }
                match self.caculator.save_code_to_file(&self.output_dir) {
                    Ok(_) => self.status = String::from("保存计算代码成功"),
                    Err(error) => self.status = format!("保存计算代码失败{error}"),
                }

                Task::none()
            }
            Message::Calculate => {
                match self.caculator.calculate() {
                    Ok(_) => {
                        self.status = String::from("计算成功");
                        self.calc_code_tab.update(CalcCodeTabMessage::UpdatePyCode(
                            self.caculator.calc_code_py.clone(),
                        ));
                        self.calc_code_tab.update(CalcCodeTabMessage::UpdateRsCode(
                            self.caculator.calc_code_rs.clone(),
                        ));
                        self.result_tab.update(ResultMessage::UpdateResult(Box::new(
                            self.caculator.results.clone(),
                        )));
                    }
                    Err(err) => self.status = format!("计算失败: {err}"),
                }
                Task::none()
            }
            Message::ThemeSelect(theme) => {
                self.theme = theme;
                Task::none()
            }
            Message::OpenHelpDialog => {
                // TODO: 打开帮助对话框
                Task::none()
            }
            // Tab消息
            Message::TabSelected(selected) => {
                self.active_tab = selected;
                Task::none()
            }
            Message::InputTab(msg) => {
                self.input_tab.update(msg.clone());
                if let InputTabMessage::ValueChanged(_) = msg {
                    self.caculator.params = self.input_tab.input_strings.clone().into()
                }
                Task::none()
            }
            Message::ResultTab(msg) => {
                self.result_tab.update(msg);
                Task::none()
            }
            Message::CalcCodeTab(msg) => {
                self.calc_code_tab.update(msg);
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        #[rustfmt::skip]
        let menubar = menu_bar!(
            (labeled_button("文件", Message::OpenSubMenu).width(Length::Shrink), {
                Menu::new(menu_items!(
                    (labeled_button("加载参数", Message::LoadParamsFromFile).width(Length::Fill))
                    (labeled_button("保存输入参数", Message::SaveInputParams).width(Length::Fill))
                    (labeled_button("选择输出目录", Message::SelectOutputDir).width(Length::Fill))
                )).max_width(180.0)
            })
            (labeled_button("计算", Message::OpenSubMenu).width(Length::Shrink), {
                Menu::new(menu_items!(
                    (labeled_button("加载默认参数", Message::LoadDefaultParams).width(Length::Fill))
                    (labeled_button("清空输入参数", Message::ClearInputParams).width(Length::Fill))
                    (labeled_button("保存计算结果", Message::SaveResult).width(Length::Fill))
                    (labeled_button("保存计算代码", Message::SaveCalcCode).width(Length::Fill))
                    (labeled_button("开始计算", Message::Calculate).width(Length::Fill))
                )).max_width(180.0)
            })
            (labeled_button("主题", Message::OpenSubMenu).width(Length::Shrink), {
                Menu::new(menu_items!(
                    (labeled_button("CatppuccinLatte", Message::ThemeSelect(Theme::CatppuccinLatte)).width(Length::Fill))
                    (labeled_button("TokyoNightLight", Message::ThemeSelect(Theme::TokyoNightLight)).width(Length::Fill))
                    (labeled_button("CatppuccinMocha", Message::ThemeSelect(Theme::CatppuccinMocha)).width(Length::Fill))
                    (labeled_button("TokyoNightStorm", Message::ThemeSelect(Theme::TokyoNightStorm)).width(Length::Fill))
                )).max_width(180.0)
            })
            (labeled_button("帮助", Message::OpenSubMenu).width(Length::Shrink), {
                Menu::new(menu_items!(
                    (labeled_button("关于", Message::OpenHelpDialog).width(Length::Fill))
                )).max_width(180.0)
            })
        ).padding(Padding::from([8.0, 0.0]));

        let ts = Tabs::new(Message::TabSelected)
            .push(
                TabId::Input,
                self.input_tab.tab_label(),
                self.input_tab.view(),
            )
            .push(
                TabId::Result,
                self.result_tab.tab_label(),
                self.result_tab.view(),
            )
            .push(
                TabId::CalcCode,
                self.calc_code_tab.tab_label(),
                self.calc_code_tab.view(),
            )
            .height(Length::Fill)
            .set_active_tab(&self.active_tab)
            .tab_bar_position(TabBarPosition::Top);

        let content = container(ts).center(Length::Fill);

        let output_dir_status = if self.output_dir.is_empty() {
            "未选择".to_string()
        } else {
            self.output_dir.clone()
        };

        let status = row![
            text(format!("状态：{}", &self.status)),
            horizontal_space(),
            text(format!("输出目录：{}", output_dir_status))
        ]
        .padding(8);

        let v = col![menubar, content, status];

        v.into()
    }

    fn get_theme(&self) -> Theme {
        self.theme.clone()
    }

    fn get_title(&self) -> String {
        self.app_name.clone()
    }
}

trait Tab {
    type Message;

    fn tab_label(&self) -> TabLabel;

    fn view(&self) -> Element<Self::Message> {
        container(self.content())
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .padding(TAB_PADDING)
            .into()
    }

    fn content(&self) -> Element<Self::Message>;
}
