mod input_type;
use input_type::{InputParameterString, InputParameters};

use iced::{
    alignment,
    widget::{column, container, horizontal_rule, scrollable, text},
    Length,
};

use crate::{components::input_field, Message, Tab};
use calc::parameters;

#[derive(Debug, Clone)]
pub enum InputTabMessage {
    UpdateParams(Box<parameters::CalcInputParameters>),
    ClearParams,
    ValueChanged(InputParameters),
}

#[derive(Default)]
pub struct InputTab {
    pub input_strings: InputParameterString,
}

impl InputTab {
    pub fn update(&mut self, message: InputTabMessage) {
        match message {
            InputTabMessage::UpdateParams(params) => self.input_strings = (*params).into(),
            InputTabMessage::ClearParams => self.input_strings = InputParameterString::default(),
            InputTabMessage::ValueChanged(input_params) => match input_params {
                InputParameters::Ne(value) => self.input_strings.ne = value,
                InputParameters::N1(value) => self.input_strings.n_1 = value,
                InputParameters::Xfh(value) => self.input_strings.x_fh = value,
                InputParameters::Zetad(value) => self.input_strings.zeta_d = value,
                InputParameters::Nhi(value) => self.input_strings.n_hi = value,
                InputParameters::Nli(value) => self.input_strings.n_li = value,
                InputParameters::Nm(value) => self.input_strings.n_m = value,
                InputParameters::Nge(value) => self.input_strings.n_ge = value,
                InputParameters::DPfh(value) => self.input_strings.dp_fh = value,
                InputParameters::DPrh(value) => self.input_strings.dp_rh = value,
                InputParameters::DPej(value) => self.input_strings.dp_ej = value,
                InputParameters::DPcd(value) => self.input_strings.dp_cd = value,
                InputParameters::DPf(value) => self.input_strings.dp_f = value,
                InputParameters::ThetaHu(value) => self.input_strings.theta_hu = value,
                InputParameters::ThetaLu(value) => self.input_strings.theta_lu = value,
                InputParameters::Nh(value) => self.input_strings.n_h = value,
                InputParameters::Nfwpp(value) => self.input_strings.n_fwpp = value,
                InputParameters::Nwpti(value) => self.input_strings.n_fwpti = value, // Corrected enum variant name if it was Nfwpti
                InputParameters::Nfwptm(value) => self.input_strings.n_fwptm = value,
                InputParameters::Nfwptg(value) => self.input_strings.n_fwptg = value,
                InputParameters::Tsw1(value) => self.input_strings.t_sw1 = value,
                InputParameters::Nenpp(value) => self.input_strings.ne_npp = value,
                InputParameters::Gcd(value) => self.input_strings.g_cd = value,
                InputParameters::Pc(value) => self.input_strings.p_c = value,
                InputParameters::DTsub(value) => self.input_strings.dt_sub = value,
                InputParameters::DTc(value) => self.input_strings.dt_c = value,
                InputParameters::Ps(value) => self.input_strings.p_s = value,
                InputParameters::DTsw(value) => self.input_strings.dt_sw = value,
                InputParameters::DT(value) => self.input_strings.dt = value,
                InputParameters::DPhz(value) => self.input_strings.dp_hz = value,
                InputParameters::Trh2z(value) => self.input_strings.t_rh2z = value,
                InputParameters::Z(value) => self.input_strings.z = value,
                InputParameters::Zl(value) => self.input_strings.z_l = value,
                InputParameters::Zh(value) => self.input_strings.z_h = value,
                InputParameters::DTfw(value) => self.input_strings.dt_fw = value,
                InputParameters::DPfwpo(value) => self.input_strings.dp_fwpo = value,
                InputParameters::DPcwp(value) => self.input_strings.dp_cwp = value,
            },
        }
    }
}

impl Tab for InputTab {
    type Message = Message;

    fn tab_label(&self) -> iced_aw::TabLabel {
        iced_aw::TabLabel::Text(String::from("输入参数"))
    }

    fn content(&self) -> iced::Element<Self::Message> {
        let section_title = |title: String| {
            container(text(title).size(20)) // 加大标题字号
                .width(Length::Fill)
                .padding(5)
                .align_x(alignment::Horizontal::Center) // 标题居中
        };

        let known_conditions_section = column![
            section_title("已知条件和给定参数".to_string()),
            horizontal_rule(1), // 分隔线
            input_field(
                "核电站电功率(MW)",
                "1000",
                &self.input_strings.ne,
                |text| InputTabMessage::ValueChanged(InputParameters::Ne(text))
            ),
            input_field(
                "一回路能量利用系数(%)",
                "99 ~ 100",
                &self.input_strings.n_1,
                |text| InputTabMessage::ValueChanged(InputParameters::N1(text))
            ),
            input_field(
                "蒸汽发生器出口蒸汽干度(%)",
                "99.75",
                &self.input_strings.x_fh,
                |text| InputTabMessage::ValueChanged(InputParameters::Xfh(text))
            ),
            input_field(
                "蒸汽发生器排污率(%)",
                "1.05",
                &self.input_strings.zeta_d,
                |text| InputTabMessage::ValueChanged(InputParameters::Zetad(text))
            ),
        ]
        .spacing(10);

        let efficiencies_section = column![
            section_title("效率参数".to_string()),
            horizontal_rule(1),
            input_field(
                "高压缸内效率(%)",
                "82.07",
                &self.input_strings.n_hi,
                |text| InputTabMessage::ValueChanged(InputParameters::Nhi(text))
            ),
            input_field(
                "低压缸内效率(%)",
                "83.59",
                &self.input_strings.n_li,
                |text| InputTabMessage::ValueChanged(InputParameters::Nli(text))
            ),
            input_field(
                "汽轮机组机械效率(%)",
                "98 ~ 99",
                &self.input_strings.n_m,
                |text| InputTabMessage::ValueChanged(InputParameters::Nm(text))
            ),
            input_field(
                "发电机效率(%)",
                "98 ~ 99",
                &self.input_strings.n_ge,
                |text| InputTabMessage::ValueChanged(InputParameters::Nge(text))
            ),
            input_field(
                "加热器效率(%)",
                "97 ~ 99",
                &self.input_strings.n_h,
                |text| InputTabMessage::ValueChanged(InputParameters::Nh(text))
            ),
            input_field(
                "给水泵效率(%)",
                "58.0",
                &self.input_strings.n_fwpp,
                |text| InputTabMessage::ValueChanged(InputParameters::Nfwpp(text))
            ),
            input_field(
                "给水泵汽轮机内效率(%)",
                "78 ~ 82",
                &self.input_strings.n_fwpti,
                |text| InputTabMessage::ValueChanged(InputParameters::Nwpti(text))
            ),
            input_field(
                "给水泵汽轮机机械效率(%)",
                "90",
                &self.input_strings.n_fwptm,
                |text| InputTabMessage::ValueChanged(InputParameters::Nfwptm(text))
            ),
            input_field(
                "给水泵汽轮机减速器效率(%)",
                "98",
                &self.input_strings.n_fwptg,
                |text| InputTabMessage::ValueChanged(InputParameters::Nfwptg(text))
            ),
            input_field(
                "假定核电厂效率(%)",
                "0 ~ 100",
                &self.input_strings.ne_npp,
                |text| InputTabMessage::ValueChanged(InputParameters::Nenpp(text))
            ),
        ]
        .spacing(10);

        let pressure_loss_section = column![
            section_title("压损参数".to_string()),
            horizontal_rule(1),
            input_field(
                "新蒸汽压损(%P_fh)",
                "3 ~ 7",
                &self.input_strings.dp_fh,
                |text| InputTabMessage::ValueChanged(InputParameters::DPfh(text))
            ),
            input_field(
                "再热蒸汽压损(%P_hz)",
                "0 ~ 10",
                &self.input_strings.dp_rh,
                |text| InputTabMessage::ValueChanged(InputParameters::DPrh(text))
            ),
            input_field(
                "回热抽汽压损(%P_cj)",
                "3 ~ 5",
                &self.input_strings.dp_ej,
                |text| InputTabMessage::ValueChanged(InputParameters::DPej(text))
            ),
            input_field(
                "低压缸排汽压损(%P_cd)",
                "5",
                &self.input_strings.dp_cd,
                |text| InputTabMessage::ValueChanged(InputParameters::DPcd(text))
            ),
            input_field(
                "流动损失(%入口压力)默认为1",
                "0 ~ 10",
                &self.input_strings.dp_f,
                |text| InputTabMessage::ValueChanged(InputParameters::DPf(text))
            ),
        ]
        .spacing(10);

        let temperature_diff_section = column![
            section_title("温差与温度参数".to_string()),
            horizontal_rule(1),
            input_field(
                "高压给水加热器出口端差(℃)",
                "3",
                &self.input_strings.theta_hu,
                |text| InputTabMessage::ValueChanged(InputParameters::ThetaHu(text))
            ),
            input_field(
                "低压给水加热器出口端差(℃)",
                "2",
                &self.input_strings.theta_lu,
                |text| InputTabMessage::ValueChanged(InputParameters::ThetaLu(text))
            ),
            input_field(
                "循环冷却水进口温度(℃)",
                "24",
                &self.input_strings.t_sw1,
                |text| InputTabMessage::ValueChanged(InputParameters::Tsw1(text))
            ),
            input_field(
                "反应堆出口冷却剂过冷度(℃)",
                "15 ~ 20",
                &self.input_strings.dt_sub,
                |text| InputTabMessage::ValueChanged(InputParameters::DTsub(text))
            ),
            input_field(
                "反应堆进出口冷却剂温升(℃)",
                "30 ~ 40",
                &self.input_strings.dt_c,
                |text| InputTabMessage::ValueChanged(InputParameters::DTc(text))
            ),
            input_field(
                "冷凝器中循环冷却水温升(℃)",
                "6 ~ 8",
                &self.input_strings.dt_sw,
                |text| InputTabMessage::ValueChanged(InputParameters::DTsw(text))
            ),
            input_field(
                "冷凝器传热端差(℃)",
                "3 ~ 10",
                &self.input_strings.dt,
                |text| InputTabMessage::ValueChanged(InputParameters::DT(text))
            ),
            input_field(
                "二级再热出口与新蒸汽温差(℃)",
                "13 ~ 15",
                &self.input_strings.t_rh2z,
                |text| InputTabMessage::ValueChanged(InputParameters::Trh2z(text))
            ),
            input_field(
                "实际/最佳给水温度比(%)",
                "85 ~ 90",
                &self.input_strings.dt_fw,
                |text| InputTabMessage::ValueChanged(InputParameters::DTfw(text))
            ),
        ]
        .spacing(10);

        let main_thermal_params_section = column![
            section_title("主要热力参数".to_string()),
            horizontal_rule(1),
            input_field(
                "假定冷凝器凝水量(kg/s)",
                "1500",
                &self.input_strings.g_cd,
                |text| InputTabMessage::ValueChanged(InputParameters::Gcd(text))
            ),
            input_field(
                "反应堆冷却剂系统运行压力(MPa)",
                "15 ~ 16",
                &self.input_strings.p_c,
                |text| InputTabMessage::ValueChanged(InputParameters::Pc(text))
            ),
            input_field(
                "蒸汽发生器饱和蒸汽压力(MPa)",
                "5 ~ 7",
                &self.input_strings.p_s,
                |text| InputTabMessage::ValueChanged(InputParameters::Ps(text))
            ),
            input_field(
                "高压缸排汽/进口压力比(%)",
                "12 ~ 14",
                &self.input_strings.dp_hz,
                |text| InputTabMessage::ValueChanged(InputParameters::DPhz(text))
            ),
            input_field(
                "给水泵出口压力(x倍P_s)",
                "1.15 ~ 1.25",
                &self.input_strings.dp_fwpo,
                |text| InputTabMessage::ValueChanged(InputParameters::DPfwpo(text))
            ),
            input_field(
                "凝水泵出口压力(x倍P_da)",
                "3 ~ 3.2",
                &self.input_strings.dp_cwp,
                |text| InputTabMessage::ValueChanged(InputParameters::DPcwp(text))
            ),
        ]
        .spacing(10);

        let stage_params_section = column![
            section_title("级数参数".to_string()),
            horizontal_rule(1),
            input_field("回热级数", "7", &self.input_strings.z, |text| {
                InputTabMessage::ValueChanged(InputParameters::Z(text))
            }),
            input_field(
                "低压给水加热器级数",
                "4",
                &self.input_strings.z_l,
                |text| InputTabMessage::ValueChanged(InputParameters::Zl(text))
            ),
            input_field(
                "高压给水加热器级数",
                "2",
                &self.input_strings.z_h,
                |text| InputTabMessage::ValueChanged(InputParameters::Zh(text))
            ),
        ]
        .spacing(10);

        let col_content = column![
            known_conditions_section,
            efficiencies_section,
            pressure_loss_section,
            temperature_diff_section,
            main_thermal_params_section,
            stage_params_section,
        ]
        .padding(15)
        .spacing(20);

        let scroll: iced::Element<InputTabMessage> = scrollable(col_content).into();
        scroll.map(Message::InputTab)
    }
}
