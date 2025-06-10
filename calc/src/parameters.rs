use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CalcInputParameters {
    // 已知条件和给定参数
    /// 核电厂输出电功率, 给定，1000(MW)
    pub ne: f64,
    /// 一回路能量利用系数, 99 ~ 100%
    pub n_1: f64,
    /// 蒸汽发生器出口蒸汽干度，给定，99.75%
    pub x_fh: f64,
    /// 蒸汽发生器排污率，给定，1.05%
    pub zeta_d: f64,
    /// 高压缸内效率，给定，82.07%
    pub n_hi: f64,
    /// 低压缸内效率，给定，83.59%
    pub n_li: f64,
    /// 汽轮机组机械效率，98 ~ 99%
    pub n_m: f64,
    /// 发电机效率，98 ~ 99%
    pub n_ge: f64,
    /// 新蒸汽压损，(3% ~ 7%)P_fh(MPa)
    pub dp_fh: f64,
    /// 再热蒸汽压损，dP_rh <= 10% P_hz(MPa)
    pub dp_rh: f64,
    /// 回热抽汽压损，(3% ~ 5%)P_cj(MPa)
    pub dp_ej: f64,
    /// 低压缸排汽压损，给定，5%P_cd(kPa)
    pub dp_cd: f64,
    /// 流动损失（%入口压力），默认1
    pub dp_f: f64,
    /// 高压给水加热器出口端差, 给定，3(℃)
    pub theta_hu: f64,
    /// 低压给水加热器出口端差, 给定，2(℃)
    pub theta_lu: f64,
    /// 加热器效率，97 ~ 99%
    pub n_h: f64,
    /// 给水泵效率，给定，58%
    pub n_fwpp: f64,
    /// 给水泵汽轮机内效率，78 ~ 82%
    pub n_fwpti: f64,
    /// 给水泵汽轮机机械效率，给定，90%
    pub n_fwptm: f64,
    /// 给水泵汽轮机减速器效率，给定，98%
    pub n_fwptg: f64,
    /// 循环冷却水进口温度，给定，24(℃)
    pub t_sw1: f64,
    /// 假定核电厂效率(%)
    pub ne_npp: f64,
    /// 假定冷凝器凝水量
    pub g_cd: f64,
    // 确定的主要热力参数
    /// 反应堆冷却剂系统运行压力，15 ~ 16(MPa)
    pub p_c: f64,
    /// 反应堆出口冷却剂过冷度，15 ~ 20(℃)
    pub dt_sub: f64,
    /// 反应堆进出口冷却剂温升，30 ~ 40(℃)
    pub dt_c: f64,
    /// 蒸汽发生器饱和蒸汽压力，5.0 ~ 7.0(MPa)
    pub p_s: f64,
    /// 冷凝器中循环冷却水温升，6 ~ 8(℃)
    pub dt_sw: f64,
    /// 冷凝器传热端差，3 ~ 10(℃)
    pub dt: f64,
    /// 高压缸排汽压力与进口蒸汽压力之比(%)，最佳分压：12 ~ 14%
    pub dp_hz: f64,
    /// 第二级再热器再热蒸汽出口温度与新蒸汽温度之差13 ~ 15℃
    pub t_rh2z: f64,
    /// 回热级数，7
    pub z: f64,
    /// 低压给水加热器级数，4
    pub z_l: f64,
    /// 高压给水加热器级数，2
    pub z_h: f64,
    /// 实际给水温度/最佳给水温度，85 ~ 90%
    pub dt_fw: f64,
    /// 给水泵出口压力(x倍GS二次侧蒸汽压力，MPa)，1.15 ~ 1.25
    pub dp_fwpo: f64,
    /// 凝水泵出口压力(x倍除氧器运行压力，MPa)，3 ~ 3.2
    pub dp_cwp: f64,
}

impl Display for CalcInputParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl CalcInputParameters {
    pub fn from_default() -> Self {
        Self {
            p_c: 15.5,
            p_s: 6.0,
            t_rh2z: 15.0,
            t_sw1: 24.0,
            z: 7.0,
            z_h: 2.0,
            z_l: 4.0,
            dp_cd: 5.0 / 100.0,
            dp_cwp: 3.1,
            dp_ej: 4.0 / 100.0,
            dp_f: 1.0 / 100.0,
            dp_fh: 5.0 / 100.0,
            dp_fwpo: 1.2,
            dp_hz: 13.0 / 100.0,
            dp_rh: 8.0 / 100.0,
            dt: 5.0,
            dt_c: 35.0,
            dt_fw: 85.0 / 100.0,
            dt_sub: 15.0,
            dt_sw: 7.0,
            g_cd: 1200.0,
            n_1: 99.6 / 100.0,
            ne: 1000.0,
            ne_npp: 1.0,
            n_fwpp: 58.0 / 100.0,
            n_fwptg: 98.0 / 100.0,
            n_fwpti: 80.0 / 100.0,
            n_fwptm: 90.0 / 100.0,
            n_ge: 99.0 / 100.0,
            n_h: 98.0 / 100.0,
            n_hi: 82.07 / 100.0,
            n_li: 83.59 / 100.0,
            n_m: 98.5 / 100.0,
            theta_hu: 3.0,
            theta_lu: 2.0,
            x_fh: 99.75 / 100.0,
            zeta_d: 1.05 / 100.0,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CalcResultParamters {
    /// 热平衡计算结果
    pub result1: Vec<CalcResult1>,
    /// 附表
    pub result2: CalcResult2,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CalcResult1 {
    /// 1.核电厂效率η_eNPP
    pub eta_enpp: f64,
    /// 2.反应堆热功率Q_R
    pub q_r: f64,
    /// 3.蒸汽发生器总蒸汽产量Ds
    pub d_s: f64,
    /// 4.汽轮机高压缸耗气量G_shp
    pub g_shp: f64,
    /// 5.汽轮机低压缸耗气量G_slp
    pub g_slp: f64,
    /// 6.第一级再热器耗气量G_srh1
    pub g_srh1: f64,
    /// 7.第二级再热器耗气量G_srh2
    pub g_srh2: f64,
    /// 8.除氧器耗气量G_sdea
    pub g_sdea: f64,
    /// 9.给水泵汽轮机耗气量G_sfwp
    pub g_sfwp: f64,
    /// 10.给水泵给水量G_fw
    pub g_fw: f64,
    /// 11.给水泵扬程H_fwp
    pub h_fwp: f64,
    /// 12.1.第七级抽汽量G_hes7
    pub g_hes7: f64,
    /// 12.2.第六级抽汽量G_hes6
    pub g_hes6: f64,
    /// 13.1.第四级抽汽量G_les4
    pub g_les4: f64,
    /// 13.2.第三级抽汽量G_les3
    pub g_les3: f64,
    /// 13.3.第二级抽汽量G_les2
    pub g_les2: f64,
    /// 13.4.第一级抽汽量G_les1
    pub g_les1: f64,
    /// 14.凝结水量G_cd
    pub g_cd: f64,
    /// 15.汽水分离器疏水量G_uw
    pub g_uw: f64,
    /// 16.一级再热器加热蒸汽量G_zc1
    pub g_zc1: f64,
    /// 17.二级再热器加热蒸汽量G_zc2
    pub g_zc2: f64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CalcResult2 {
    // 附表一
    /// 1.核电厂输出功率N_e
    pub ne: f64,
    /// 2.一回路能量利用系数η_1
    pub eta_1: f64,
    /// 3.蒸汽发生器出口蒸汽干度X_fh
    pub x_fh: f64,
    /// 4.蒸汽发生器排污率ξ_d
    pub zeta_d: f64,
    /// 5.高压缸内效率η_hi
    pub eta_hi: f64,
    /// 6.低压缸内效率η_li
    pub eta_li: f64,
    /// 7.汽轮机组机械效率η_m
    pub eta_m: f64,
    /// 8.发电机效率η_ge
    pub eta_ge: f64,
    /// 9.新蒸汽压损Δp_fh
    pub dp_fh: f64,
    /// 10.再热蒸汽压损Δp_rh
    pub dp_rh: f64,
    /// 11.回热蒸汽压损Δp_ej
    pub dp_ej: f64,
    /// 12.低压缸排气压损Δp_cd
    pub dp_cd: f64,
    /// 13.高压给水加热器出口端差θ_hu
    pub theta_hu: f64,
    /// 14.低压给水加热器出口端差θ_hu
    pub theta_lu: f64,
    /// 15.加热器效率η_h
    pub eta_h: f64,
    /// 16.给水泵效率η_fwpp
    pub eta_fwpp: f64,
    /// 17.给水泵汽轮机内效率η_fwpti
    pub eta_fwpti: f64,
    /// 18.给水泵汽轮机机械效率η_fwptm
    pub eta_fwptm: f64,
    /// 19.给水泵汽轮机减速器效率η_fwptg
    pub eta_fwptg: f64,
    /// 20.循环冷却水进口温度T_sw1
    pub t_sw1: f64,
    // 附表二
    /// 1.反应堆冷却剂系统运行压力p_c
    pub p_c: f64,
    /// 2.冷却剂压力对应的饱和温度T_cs
    pub t_cs: f64,
    /// 3.反应堆出口冷却剂过冷度ΔT_sub
    pub dt_sub: f64,
    /// 4.反应堆出口冷却剂温度T_co
    pub t_co: f64,
    /// 5.反应堆进出口冷却剂温升ΔT_c
    pub dt_c: f64,
    /// 6.反应堆进口冷却剂温度T_ci
    pub t_ci: f64,
    /// 7.蒸汽发生器饱和蒸汽压力p_s
    pub p_s: f64,
    /// 8.蒸汽发生器饱和蒸汽温度T_fh
    pub t_fh: f64,
    /// 9.一、二次侧对数平均温差ΔT_m
    pub dt_m: f64,
    /// 10.冷凝器中循环冷却水温升ΔT_sw
    pub dt_sw: f64,
    /// 11.冷凝器传热端差δT
    pub dt: f64,
    /// 12.冷凝器凝结水饱和温度T_cd
    pub t_cd: f64,
    /// 13.冷凝器的运行压力p_cd
    pub p_cd: f64,
    /// 14.高压缸进口的蒸汽压力p_hi
    pub p_hi: f64,
    /// 15.高压缸进口蒸汽干度X_hi
    pub x_hi: f64,
    /// 15.1.蒸汽发生器出口蒸汽比焓h_fh
    pub h_fh: f64,
    /// 15.2.蒸汽发生器出口蒸汽比熵s_fh
    pub s_fh: f64,
    /// 15.3.高压缸进口蒸汽比熵s_hi
    pub s_hi: f64,
    /// 16.高压缸排气压力p_hz
    pub p_hz: f64,
    /// 17.高压缸排气干度X_hz
    pub x_hz: f64,
    /// 17.1.高压缸进口蒸汽比焓h_hi
    pub h_hi: f64,
    /// 17.2.高压缸出口理想比焓h_hzs
    pub h_hzs: f64,
    /// 17.3.高压缸出口蒸汽比焓h_hz
    pub h_hz: f64,
    /// 18.汽水分离器进口蒸汽压力p_spi
    pub p_spi: f64,
    /// 19.汽水分离器进口蒸汽干度X_spi
    pub x_spi: f64,
    /// 19.1.汽水分离器出口疏水压力p_uw
    pub p_uw: f64,
    /// 19.2.汽水分离器出口疏水比焓h_uw
    pub h_uw: f64,
    // 第一级再热器
    /// 20.再热蒸汽进口压力p_rh1i
    pub p_rh1i: f64,
    /// 21.再热蒸汽进口干度X_rh1i
    pub x_rh1i: f64,
    /// 21.1.一级再热器进口蒸汽比焓h_rh1i
    pub h_rh1i: f64,
    /// 22.加热蒸汽进口压力p_rh1hs
    pub p_rh1hs: f64,
    /// 23.加热蒸汽进口干度X_rh1hs
    pub x_rh1hs: f64,
    // 第二级再热器
    /// 24.再热蒸汽进口压力p_rh2i
    pub p_rh2i: f64,
    /// 25.再热蒸汽进口温度T_rh2i
    pub t_rh2i: f64,
    /// 26.再热蒸汽出口压力p_rh2z
    pub p_rh2z: f64,
    /// 27.再热蒸汽出口温度T_rh2z
    pub t_rh2z: f64,
    /// 27.1.二级再热器出口比焓h_rh2z
    pub h_rh2z: f64,
    /// 27.2.每级再热器平均焓升Δh_rh
    pub dh_rh: f64,
    /// 27.3.一级再热器出口蒸汽比焓h_rh1z
    pub h_rh1z: f64,
    /// 27.4.二级再热器进口蒸汽比焓h_rh2i
    pub h_rh2i: f64,
    /// 28.加热蒸汽进口压力p_rh2hs
    pub p_rh2hs: f64,
    /// 29.加热蒸汽进口干度X_rh2hs
    pub x_rh2hs: f64,
    // 低压缸
    /// 30.进口蒸汽压力p_li
    pub p_li: f64,
    /// 31.进口蒸汽温度T_li
    pub t_li: f64,
    /// 32.排汽压力p_lz
    pub p_lz: f64,
    /// 33.排汽干度X_lz
    pub x_lz: f64,
    /// 33.1.低压缸进口蒸汽比熵s_li
    pub s_li: f64,
    /// 33.2.低压缸进口蒸汽比焓h_li
    pub h_li: f64,
    /// 33.3.低压缸出口理想比焓h_lzs
    pub h_lzs: f64,
    /// 33.4.低压缸出口蒸汽比焓h_lz
    pub h_lz: f64,
    /// 34.回热级数Z
    pub z: f64,
    /// 35.低压给水加热器级数Z_l
    pub z_l: f64,
    /// 36.高压给水加热器级数Z_h
    pub z_h: f64,
    /// 37.第一次给水回热分配Δh_fw
    pub dh_fw: f64,
    /// 37.1.蒸汽发生器运行压力饱和水比焓h_s
    pub h_s: f64,
    /// 37.2.冷凝器出口凝结水比焓h_cd
    pub h_cd: f64,
    /// 37.3.每级加热器理论给水焓升Δh_fwop
    pub dh_fwop: f64,
    /// 37.4.最佳给水比焓h_fwop
    pub h_fwop: f64,
    /// 37.5.最佳给水温度T_fwop
    pub t_fwop: f64,
    /// 37.6.实际给水温度T_fw
    pub t_fw: f64,
    /// 37.7.实际给水比焓h_fw
    pub h_fw: f64,
    /// 38.高压加热器给水焓升Δh_fwh
    pub dh_fwh: f64,
    /// 38.1.除氧器运行压力p_dea
    pub p_dea: f64,
    /// 38.2.除氧器出口饱和水比焓h_deao
    pub h_deao: f64,
    /// 39.除氧器及低压加热器给水焓升Δh_fwl
    pub dh_fwl: f64,
    /// 39.1.凝水泵出口给水压力p_cwp
    pub p_cwp: f64,
    /// 39.2.凝水泵出口给水比焓h_cwp
    pub h_cwp: f64,
    /// 39.3.凝水泵出口至除氧器出口的阻力压降Δp_cws
    pub dp_cws: f64,
    /// 39.4.每级低压加热器及除氧器的阻力压降Δp_fi
    pub dp_fi: f64,
    /// 40.低压加热器给水参数(1 ~ 4级)
    pub lfwx: Vec<CalcFWParameters>,
    /// 41.进口给水比焓h_deai
    pub h_deai: f64,
    /// 42.出口给水比焓h_deao
    pub h_deao1: f64,
    /// 43.出口给水温度T_dea
    pub t_dea: f64,
    /// 44.运行压力p_dea
    pub p_dea1: f64,
    /// 44.1.给水泵出口压力p_fwpo
    pub p_fwpo: f64,
    /// 44.2.给水泵出口流体比焓h_fwpo
    pub h_fwpo: f64,
    /// 44.3.蒸汽发生器进口给水压力p_fwi
    pub p_fwi: f64,
    /// 45.高压加热器给水参数(6 ~ 7级)
    pub hfwx: Vec<CalcFWParameters>,
    // 46.高压缸抽汽
    /// 46.1.高压缸进口蒸汽比熵s_hi
    pub s_hi1: f64,
    /// 46.2.高压缸进口蒸汽比焓h_hi
    pub h_hi1: f64,
    /// 第六、七级给水加热器抽汽参数
    pub hhes: Vec<CalcHESParameters>,
    // 47.低压缸抽汽
    /// 47.1.低压缸进口蒸汽比熵s_li
    pub s_li1: f64,
    /// 47.2.低压缸进口蒸汽比焓h_li
    pub h_li1: f64,
    /// 第一至四级给水加热器抽汽参数
    pub lhes: Vec<CalcHESParameters>,
    /// 48.再热器抽汽(第一、二级再热器抽汽参数)
    pub rhx: Vec<CalcRHXParameters>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CalcFWParameters {
    /// 进口给水压力p_fwxi
    pub p_fwxi: f64,
    /// 进口给水比焓h_fwxi
    pub h_fwxi: f64,
    /// 进口给水温度T_fwxi
    pub t_fwxi: f64,
    /// 出口给水压力p_fwxo
    pub p_fwxo: f64,
    /// 出口给水比焓h_fwxo
    pub h_fwxo: f64,
    /// 出口给水温度T_fwxo
    pub t_fwxo: f64,
    /// 汽侧疏水温度T_roxk
    pub t_roxk: f64,
    /// 汽侧疏水比焓h_roxk
    pub h_roxk: f64,
    /// 汽测疏水压力p_roxk
    pub p_roxk: f64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CalcHESParameters {
    /// 抽汽温度T_hesx
    pub t_hesx: f64,
    /// 抽汽压力p_hesx
    pub p_hesx: f64,
    /// 抽汽干度X_hesx
    pub x_hesx: f64,
    /// 抽汽理想比焓h_hesxs
    pub h_hesxs: f64,
    /// 抽汽比焓h_hesx
    pub h_hesx: f64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CalcRHXParameters {
    /// 加热蒸汽进口压力p_rhx
    pub p_rhx: f64,
    /// 加热蒸汽进口干度X_rhx
    pub x_rhx: f64,
    /// 加热蒸汽进口温度T_rhx
    pub t_rhx: f64,
    /// 加热蒸汽进口比焓h_rhx
    pub h_rhx: f64,
    /// 再热器疏水比焓h_zsx
    pub h_zsx: f64,
}
