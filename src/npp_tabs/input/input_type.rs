use calc::parameters;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum InputParameters {
    Ne(String),
    N1(String),
    Xfh(String),
    Zetad(String),
    Nhi(String),
    Nli(String),
    Nm(String),
    Nge(String),
    DPfh(String),
    DPrh(String),
    DPej(String),
    DPcd(String),
    DPf(String),
    ThetaHu(String),
    ThetaLu(String),
    Nh(String),
    Nfwpp(String),
    Nwpti(String),
    Nfwptm(String),
    Nfwptg(String),
    Tsw1(String),
    Nenpp(String),
    Gcd(String),
    Pc(String),
    DTsub(String),
    DTc(String),
    Ps(String),
    DTsw(String),
    DT(String),
    DPhz(String),
    Trh2z(String),
    Z(String),
    Zl(String),
    Zh(String),
    DTfw(String),
    DPfwpo(String),
    DPcwp(String),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InputParameterString {
    // 已知条件和给定参数
    /// 核电厂输出电功率, 给定，1000(MW)
    pub ne: String,
    /// 一回路能量利用系数, 99 ~ 100%
    pub n_1: String,
    /// 蒸汽发生器出口蒸汽干度，给定，99.75%
    pub x_fh: String,
    /// 蒸汽发生器排污率，给定，1.05%
    pub zeta_d: String,
    /// 高压缸内效率，给定，82.07%
    pub n_hi: String,
    /// 低压缸内效率，给定，83.59%
    pub n_li: String,
    /// 汽轮机组机械效率，98 ~ 99%
    pub n_m: String,
    /// 发电机效率，98 ~ 99%
    pub n_ge: String,
    /// 新蒸汽压损，(3% ~ 7%)P_fh(MPa)
    pub dp_fh: String,
    /// 再热蒸汽压损，dP_rh <= 10% P_hz(MPa)
    pub dp_rh: String,
    /// 回热抽汽压损，(3% ~ 5%)P_cj(MPa)
    pub dp_ej: String,
    /// 低压缸排汽压损，给定，5%P_cd(kPa)
    pub dp_cd: String,
    /// 流动损失（%入口压力），默认1
    pub dp_f: String,
    /// 高压给水加热器出口端差, 给定，3(℃)
    pub theta_hu: String,
    /// 低压给水加热器出口端差, 给定，2(℃)
    pub theta_lu: String,
    /// 加热器效率，97 ~ 99%
    pub n_h: String,
    /// 给水泵效率，给定，58%
    pub n_fwpp: String,
    /// 给水泵汽轮机内效率，78 ~ 82%
    pub n_fwpti: String,
    /// 给水泵汽轮机机械效率，给定，90%
    pub n_fwptm: String,
    /// 给水泵汽轮机减速器效率，给定，98%
    pub n_fwptg: String,
    /// 循环冷却水进口温度，给定，24(℃)
    pub t_sw1: String,
    /// 假定核电厂效率(%)
    pub ne_npp: String,
    /// 假定冷凝器凝水量
    pub g_cd: String,
    // 确定的主要热力参数
    /// 反应堆冷却剂系统运行压力，15 ~ 16(MPa)
    pub p_c: String,
    /// 反应堆出口冷却剂过冷度，15 ~ 20(℃)
    pub dt_sub: String,
    /// 反应堆进出口冷却剂温升，30 ~ 40(℃)
    pub dt_c: String,
    /// 蒸汽发生器饱和蒸汽压力，5.0 ~ 7.0(MPa)
    pub p_s: String,
    /// 冷凝器中循环冷却水温升，6 ~ 8(℃)
    pub dt_sw: String,
    /// 冷凝器传热端差，3 ~ 10(℃)
    pub dt: String,
    /// 高压缸排汽压力与进口蒸汽压力之比(%)，最佳分压：12 ~ 14%
    pub dp_hz: String,
    /// 第二级再热器再热蒸汽出口温度与新蒸汽温度之差13 ~ 15℃
    pub t_rh2z: String,
    /// 回热级数，7
    pub z: String,
    /// 低压给水加热器级数，4
    pub z_l: String,
    /// 高压给水加热器级数，2
    pub z_h: String,
    /// 实际给水温度/最佳给水温度，85 ~ 90%
    pub dt_fw: String,
    /// 给水泵出口压力(x倍GS二次侧蒸汽压力，MPa)，1.15 ~ 1.25
    pub dp_fwpo: String,
    /// 凝水泵出口压力(x倍除氧器运行压力，MPa)，3 ~ 3.2
    pub dp_cwp: String,
}

impl From<parameters::CalcInputParameters> for InputParameterString {
    fn from(params: parameters::CalcInputParameters) -> Self {
        Self {
            ne: params.ne.to_string(),
            n_1: (params.n_1 * 100.0).to_string(),
            x_fh: (params.x_fh * 100.0).to_string(),
            zeta_d: (params.zeta_d * 100.0).to_string(),
            n_hi: (params.n_hi * 100.0).to_string(),
            n_li: (params.n_li * 100.0).to_string(),
            n_m: (params.n_m * 100.0).to_string(),
            n_ge: (params.n_ge * 100.0).to_string(),
            dp_fh: (params.dp_fh * 100.0).to_string(),
            dp_rh: (params.dp_rh * 100.0).to_string(),
            dp_ej: (params.dp_ej * 100.0).to_string(),
            dp_cd: (params.dp_cd * 100.0).to_string(),
            dp_f: (params.dp_f * 100.0).to_string(),
            theta_hu: params.theta_hu.to_string(),
            theta_lu: params.theta_lu.to_string(),
            n_h: (params.n_h * 100.0).to_string(),
            n_fwpp: (params.n_fwpp * 100.0).to_string(),
            n_fwpti: (params.n_fwpti * 100.0).to_string(),
            n_fwptm: (params.n_fwptm * 100.0).to_string(),
            n_fwptg: (params.n_fwptg * 100.0).to_string(),
            t_sw1: params.t_sw1.to_string(),
            ne_npp: (params.ne_npp * 100.0).to_string(),
            g_cd: params.g_cd.to_string(),
            p_c: params.p_c.to_string(),
            dt_sub: params.dt_sub.to_string(),
            dt_c: params.dt_c.to_string(),
            p_s: params.p_s.to_string(),
            dt_sw: params.dt_sw.to_string(),
            dt: params.dt.to_string(),
            dp_hz: (params.dp_hz * 100.0).to_string(),
            t_rh2z: params.t_rh2z.to_string(),
            z: params.z.to_string(),
            z_l: params.z_l.to_string(),
            z_h: params.z_h.to_string(),
            dt_fw: (params.dt_fw * 100.0).to_string(),
            dp_fwpo: params.dp_fwpo.to_string(),
            dp_cwp: params.dp_cwp.to_string(),
        }
    }
}

impl From<InputParameterString> for parameters::CalcInputParameters {
    fn from(params_string: InputParameterString) -> Self {
        Self {
            ne: params_string.ne.parse::<f64>().unwrap_or(0.0),
            n_1: params_string.n_1.parse::<f64>().unwrap_or(0.0) / 100.0,
            x_fh: params_string.x_fh.parse::<f64>().unwrap_or(0.0) / 100.0,
            zeta_d: params_string.zeta_d.parse::<f64>().unwrap_or(0.0) / 100.0,
            n_hi: params_string.n_hi.parse::<f64>().unwrap_or(0.0) / 100.0,
            n_li: params_string.n_li.parse::<f64>().unwrap_or(0.0) / 100.0,
            n_m: params_string.n_m.parse::<f64>().unwrap_or(0.0) / 100.0,
            n_ge: params_string.n_ge.parse::<f64>().unwrap_or(0.0) / 100.0,
            dp_fh: params_string.dp_fh.parse::<f64>().unwrap_or(0.0) / 100.0,
            dp_rh: params_string.dp_rh.parse::<f64>().unwrap_or(0.0) / 100.0,
            dp_ej: params_string.dp_ej.parse::<f64>().unwrap_or(0.0) / 100.0,
            dp_cd: params_string.dp_cd.parse::<f64>().unwrap_or(0.0) / 100.0,
            dp_f: params_string.dp_f.parse::<f64>().unwrap_or(0.0) / 100.0,
            theta_hu: params_string.theta_hu.parse::<f64>().unwrap_or(0.0),
            theta_lu: params_string.theta_lu.parse::<f64>().unwrap_or(0.0),
            n_h: params_string.n_h.parse::<f64>().unwrap_or(0.0) / 100.0,
            n_fwpp: params_string.n_fwpp.parse::<f64>().unwrap_or(0.0) / 100.0,
            n_fwpti: params_string.n_fwpti.parse::<f64>().unwrap_or(0.0) / 100.0,
            n_fwptm: params_string.n_fwptm.parse::<f64>().unwrap_or(0.0) / 100.0,
            n_fwptg: params_string.n_fwptg.parse::<f64>().unwrap_or(0.0) / 100.0,
            t_sw1: params_string.t_sw1.parse::<f64>().unwrap_or(0.0),
            ne_npp: params_string.ne_npp.parse::<f64>().unwrap_or(0.0) / 100.0,
            g_cd: params_string.g_cd.parse::<f64>().unwrap_or(0.0),
            p_c: params_string.p_c.parse::<f64>().unwrap_or(0.0),
            dt_sub: params_string.dt_sub.parse::<f64>().unwrap_or(0.0),
            dt_c: params_string.dt_c.parse::<f64>().unwrap_or(0.0),
            p_s: params_string.p_s.parse::<f64>().unwrap_or(0.0),
            dt_sw: params_string.dt_sw.parse::<f64>().unwrap_or(0.0),
            dt: params_string.dt.parse::<f64>().unwrap_or(0.0),
            dp_hz: params_string.dp_hz.parse::<f64>().unwrap_or(0.0) / 100.0,
            t_rh2z: params_string.t_rh2z.parse::<f64>().unwrap_or(0.0),
            z: params_string.z.parse::<f64>().unwrap_or(0.0),
            z_l: params_string.z_l.parse::<f64>().unwrap_or(0.0),
            z_h: params_string.z_h.parse::<f64>().unwrap_or(0.0),
            dt_fw: params_string.dt_fw.parse::<f64>().unwrap_or(0.0) / 100.0,
            dp_fwpo: params_string.dp_fwpo.parse::<f64>().unwrap_or(0.0),
            dp_cwp: params_string.dp_cwp.parse::<f64>().unwrap_or(0.0),
        }
    }
}
