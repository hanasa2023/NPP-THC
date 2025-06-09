#![windows_subsystem = "windows"]

mod common;
mod components;
mod config;
mod npp_tabs;

use npp_tabs::{
    input::{InputTab, InputTabMessage},
    result::{ResultMessage, ResultTab},
};

use common::{
    errors, helpers,
    theme::{MISANS_FONT, TAB_PADDING},
};

use components::{labeled_button, modal};

use config::AppConfig;

use calc::parameters;

use iced::{
    widget::{column as col, container, horizontal_space, row, text},
    window, Alignment, Element, Length, Padding, Settings, Task, Theme,
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
    let window_icon = window::icon::from_file("../assets/logo.png").ok();
    iced::application(App::get_title, App::update, App::view)
        .theme(App::get_theme)
        .settings(settings)
        .window(window::Settings {
            icon: window_icon,
            ..window::Settings::default()
        })
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
    config: AppConfig,
    status: String,
    pending_action: Option<PendingAction>,
    caculator: calc::Calculator,
    active_tab: TabId,
    input_tab: InputTab,
    result_tab: ResultTab,
    show_help_dialog: bool,
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
    Calculate,
    ThemeSelect(Theme),
    OpenHelpDialog,
    HideHelpDialog,
    // Tab消息
    TabSelected(TabId),
    InputTab(InputTabMessage),
    ResultTab(ResultMessage),
}

#[derive(Debug, Clone)]
enum PendingAction {
    InputParams,
    Result,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let config = AppConfig::from_file("config.json").unwrap_or_default();
        let theme = match config.theme.as_str() {
            "CatppuccinLatte" => Theme::CatppuccinLatte,
            "TokyoNightLight" => Theme::TokyoNightLight,
            "CatppuccinMocha" => Theme::CatppuccinMocha,
            "TokyoNightStorm" => Theme::TokyoNightStorm,
            _ => Theme::CatppuccinMocha,
        };
        let app = Self {
            app_name: String::from("核电厂热力计算程序"),
            theme,
            config,
            status: String::new(),
            pending_action: None,
            caculator: calc::Calculator::default(),
            active_tab: TabId::Input,
            input_tab: InputTab::default(),
            result_tab: ResultTab::default(),
            show_help_dialog: false,
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
                if self.config.output_path.is_empty() {
                    self.pending_action = Some(PendingAction::InputParams);
                    return Task::perform(helpers::select_output_dir(), Message::SelectedOutputDir);
                }
                match self
                    .caculator
                    .save_parameters_to_file(&self.config.output_path)
                {
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
                        self.config.output_path = path.clone();
                        self.config.to_file("config.json").unwrap();
                        if let Some(action) = self.pending_action.clone() {
                            match action {
                                PendingAction::InputParams => {
                                    self.pending_action = None;
                                    match self
                                        .caculator
                                        .save_parameters_to_file(&self.config.output_path)
                                    {
                                        Ok(_) => self.status = "保存输入参数成功".to_string(),
                                        Err(error) => {
                                            self.status = format!("保存输入参数失败{error}")
                                        }
                                    }
                                }
                                PendingAction::Result => {
                                    self.pending_action = None;
                                    match self
                                        .caculator
                                        .save_results_to_file(&self.config.output_path)
                                    {
                                        Ok(_) => self.status = "保存计算结果成功".to_string(),
                                        Err(error) => {
                                            self.status = format!("保存计算结果失败{error}")
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
                if self.config.output_path.is_empty() {
                    self.pending_action = Some(PendingAction::Result);
                    return Task::perform(helpers::select_output_dir(), Message::SelectedOutputDir);
                }
                match self
                    .caculator
                    .save_results_to_file(&self.config.output_path)
                {
                    Ok(_) => self.status = "保存计算结果成功".to_string(),
                    Err(error) => self.status = format!("保存计算结果失败{error}"),
                }
                Task::none()
            }
            Message::Calculate => {
                match self.caculator.calculate() {
                    Ok(_) => {
                        self.status = String::from("计算成功");
                        self.result_tab.update(ResultMessage::UpdateResult(Box::new(
                            self.caculator.results.clone(),
                        )));
                    }
                    Err(err) => self.status = format!("计算失败: {err}"),
                }
                Task::none()
            }
            Message::ThemeSelect(theme) => {
                self.config.theme = match theme.clone() {
                    Theme::CatppuccinLatte => "CatppuccinLatte".to_string(),
                    Theme::TokyoNightLight => "TokyoNightLight".to_string(),
                    Theme::CatppuccinMocha => "CatppuccinMocha".to_string(),
                    Theme::TokyoNightStorm => "TokyoNightStorm".to_string(),
                    _ => "CatppuccinMocha".to_string(),
                };
                self.theme = theme;
                self.config.to_file("config.json").unwrap();
                Task::none()
            }
            Message::OpenHelpDialog => {
                self.show_help_dialog = true;
                Task::none()
            }
            Message::HideHelpDialog => {
                self.show_help_dialog = false;
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
            .height(Length::Fill)
            .set_active_tab(&self.active_tab)
            .tab_bar_position(TabBarPosition::Top);

        let content = container(ts).center(Length::Fill);

        let output_dir_status = if self.config.output_path.is_empty() {
            "未选择".to_string()
        } else {
            self.config.output_path.clone()
        };

        let status = row![
            text(format!("状态：{}", &self.status)),
            horizontal_space(),
            text(format!("输出目录：{}", output_dir_status))
        ]
        .padding(8);

        let v = col![menubar, content, status];

        if self.show_help_dialog {
            let app_name_text = text(&self.app_name).size(24);
            let version_text = text(format!("版本: {}", env!("CARGO_PKG_VERSION"))).size(16); // 从 Cargo.toml 获取版本
            let description_text = text(
                "本项目是一个基于 Rust 和 Iced 构建的核电厂热力循环计算桌面应用程序。 \
                它允许用户输入详细的运行参数，执行热力计算，查看计算结果，并能生成相应的计算过程代码。"
            )
            .width(Length::Fill); // 允许文本换行

            let features_title = text("主要功能:").size(18);
            let features_list = col![
                text("- 参数输入、加载、保存与清空"),
                text("- 热力循环计算"),
                text("- 计算结果展示与保存"),
                text("- 计算过程代码生成 (Rust & Python) 与保存"),
                text("- 多主题选择"),
                text("- 输出目录选择与配置保存")
            ]
            .spacing(5);

            let usage_title = text("简要说明:").size(18);
            let usage_list = col![
                text("1. 在“输入参数”页输入或加载参数。"),
                text("2. 通过“文件”菜单选择输出目录。"),
                text("3. 点击“计算”菜单中的“开始计算”。"),
                text("4. 在“计算结果”和“计算代码”页查看详情。"),
                text("5. 使用菜单栏进行参数、结果、代码的保存。")
            ]
            .spacing(5);

            let author_text = text("灵感来源: euaurora/curriculum-design2").size(14);
            let github_link_text = text("项目地址: https://github.com/hanasa2023/NPP-THC") // 替换为您的实际链接
                .size(14);

            let help_column = col![
                app_name_text,
                version_text,
                description_text,
                features_title,
                features_list,
                usage_title,
                usage_list,
                author_text,
                github_link_text,
            ]
            .spacing(15)
            .align_x(Alignment::Start) // 内容左对齐
            .padding(20);

            let help_content_styled = container(help_column)
                .width(450) // 调整宽度以适应更多内容
                .style(container::rounded_box);
            modal(v, help_content_styled, Message::HideHelpDialog)
        } else {
            v.into()
        }
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
