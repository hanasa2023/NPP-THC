pub mod parameters;

use std::fs::File;
use std::io::BufWriter;
use std::{error::Error, io::Write};

use parameters::{
    CalcFWParameters, CalcHESParameters, CalcInputParameters, CalcRHXParameters, CalcResult1,
    CalcResult2, CalcResultParamters,
};
use seuif97::*;

#[derive(Default)]
pub struct Calculator {
    pub params: CalcInputParameters,
    pub results: CalcResultParamters,
}

impl Calculator {
    pub fn new(params: CalcInputParameters) -> Self {
        Self {
            params,
            results: CalcResultParamters::default(),
        }
    }

    pub fn set_input_params(&mut self, params: CalcInputParameters) {
        self.params = params;
    }

    /// 计算核电厂的热力学参数
    pub fn calculate(&mut self) -> Result<(), Box<dyn Error>> {
        // 一回路冷却剂参数
        let t_cs = px(self.params.p_c, 0.0, OT); // 工作压力对应饱和温度（冷却剂压力对应饱和温度）
        let t_co = t_cs - self.params.dt_sub; // 反应堆出口冷却剂温度
        let t_ci = t_co - self.params.dt_c; // 反应堆进口冷却剂温度

        // 蒸汽初参数
        let t_s = px(self.params.p_s, 1.0, OT); // 对应的饱和温度
        let t_fh = px(self.params.p_s, self.params.x_fh, OT); // 新蒸汽温度（蒸汽发生器饱和蒸汽温度）
        let h_fh = tx(t_fh, self.params.x_fh, OH); // 新蒸汽比焓
        let s_fh = px(self.params.p_s, self.params.x_fh, OS); // 新蒸汽比熵
        let dt_m = (t_co - t_ci) / f64::ln((t_co - t_s) / (t_ci - t_s)); // 对数平均传热温差

        // 蒸汽终参数
        let t_cd = self.params.t_sw1 + self.params.dt_sw + self.params.dt; // 冷凝器凝结水饱和温度
        let p_cd = tx(t_cd, 0.0, OP); // 凝结水压力（冷凝器的运行压力 Mpa）

        // 高压缸参数
        let dp_fh = self.params.dp_fh * self.params.p_s; // 新蒸汽压损
        let p_hi = self.params.p_s - dp_fh; // 高压缸进口蒸汽压力(MPa)
        let h_hi = px(p_hi, 1.0, OH); // 高压缸进口蒸汽比焓
        let x_hi = ph(p_hi, h_hi, OX); // 进口蒸汽干度
        let s_hi = ph(p_hi, h_hi, OS); // 进口蒸汽比熵
        let p_hz = self.params.dp_hz * p_hi; // 排气压力
        let h_hzs = ps(p_hz, s_hi, OH); // 高压缸排气理想比焓
        let h_hz = h_hi - self.params.n_hi * (h_hi - h_hzs); // 高压缸排气实际比焓
        let x_hz = ph(p_hz, h_hz, OX); // 排气干度

        // 蒸汽中间再热参数
        // 在汽水分离器再热器中的总压降为高压缸排汽压力的3%左右。
        // 为计算方便，假设高压缸排汽经过汽水分离再热系统时各设备的压降相同，
        // 均为总压降的1/3。参照大亚湾的蒸汽参数，汽水分离器除去蒸汽中98%的水
        let dp_rh = self.params.dp_rh * p_hz; // 再热蒸汽压损
        let p_spi = p_hz; // 汽水分离器进口蒸汽压力
        let x_spi = x_hz; // 汽水分离器进口蒸汽干度
        let _h_spi = px(p_hz, 0.0, OH); // 汽水分离器入口焓值
        let p_uw = 0.99 * p_hz; // 汽水分离器出口疏水压力，考虑汽水分离器进出口有1%的压降
        let h_uw = px(p_uw, 0.0, OH); // 汽水分离器出口疏水比焓

        // 一级再热器
        let p_rh1i = 0.99 * p_hz; // 一级再热器进口蒸汽压力
        let x_rh1i = x_spi / (1.0 - 0.98 * (1.0 - x_spi)); // 一级再热器进口蒸汽干度，0.98为汽水分离器效率
        let h_rh1i = px(p_rh1i, x_rh1i, OH); // 一级再热器进口蒸汽比焓

        // 二级再热器
        let p_rh2i = 0.98 * p_hz; // 再热蒸汽进口压力
        let p_rh2z = 0.97 * p_hz; //二级再热器出口压力
        let t_rh2z = t_fh - self.params.t_rh2z; // 二级再热器出口温度
        let h_rh2z = pt(p_rh2z, t_rh2z, OH); // 二级再热器出口蒸汽比焓
        let dh_rh = (h_rh2z - h_rh1i) / 2.0; // 每级再热器平均焓升
        let h_rh1z = h_rh1i + dh_rh; // 一级再热器出口蒸汽比焓
        let h_rh2i = h_rh1z; // 二级再热器进口蒸汽比焓
        let t_rh2i = ph(p_rh2i, h_rh2i, OT); // 二级再热器进口蒸汽温度
        let p_rh2hs = p_hi; // 加热（新）蒸汽进口压力
        let x_rh2hs = x_hi; // 加热（新）蒸汽进口干度

        // 低压缸参数
        // 考虑低压缸的进汽损失占再热器出口压力的dp_f
        let p_li = (1.0 - self.params.dp_f) * p_rh2z; // 低压缸进气压力，考虑损失
        let h_li = h_rh2z; // 低压缸进口进气比焓，定焓过程
        let t_li = ph(p_li, h_li, OT); // 进口蒸汽温度
        let dp_cd = (1.0 / (1.0 - self.params.dp_cd) - 1.0) * p_cd; // 低压缸排气压损
        let p_lz = p_cd + dp_cd; // 低压缸排气压力
        let s_li = ph(p_li, h_li, OS); // 进口蒸汽比焓
        let s_lz = s_li; // 考虑定熵过程
        let h_lzs = ps(p_lz, s_lz, OH); // 低压缸排气理想比焓
        let h_lz = h_li - self.params.n_li * (h_li - h_lzs); // 排气实际比焓
        let x_lz = ph(p_lz, h_lz, OX); // 排气干度

        // 给水的焓升分配
        let h_s = px(self.params.p_s, 0.0, OH); // GS工作压力下的饱和水焓
        let h_cd = tx(t_cd, 0.0, OH); // 冷凝器出口凝结水比焓
        let dh_fwop = (h_s - h_cd) / (self.params.z + 1.0); // 理论给水焓升
        let h_fwop = h_cd + self.params.z * dh_fwop; // GS最佳给水比焓
        let t_fwop = ph(self.params.p_s, h_fwop, OT); // 最佳给水温度
        let t_fw = self.params.dt_fw * t_fwop; // 实际给水温度
        let h_fw = pt(self.params.p_s, t_fw, OH); // 实际给水比焓
        let dh_fw = (h_fw - h_cd) / self.params.z; // 每一级加热器内实际给水焓升

        // 除氧器
        let p_dea = 0.99 * p_hz; // 除氧器运行压力，略低于高压缸排汽压力
        let t_deao = px(p_dea, 0.0, OT); // 除氧器出口温度
        let h_deao = tx(t_deao, 0.0, OH); // 除氧器出口对应饱和水比焓
        let dh_fwh = (h_fw - h_deao) / self.params.z_h; // 高压给水加热器每一级给水焓升
        let dh_fwl = (h_deao - h_cd) / (self.params.z_l + 1.0); // 除氧器及低压加热器每一级给水焓升

        // 给水回路系统中的压力选择
        let p_cwp = self.params.dp_cwp * p_dea; // 取凝水泵出口压力为除氧器运行压力的dp_cwp倍
        let h_cwp = h_cd; // 凝水泵出口给水比焓
        let t_cwp = ph(p_cwp, h_cwp, OT); // 凝水泵出口给水温度
        let dp_cws = p_cwp - p_dea; // 凝水泵出口至除氧器的阻力压降
        let dp_fi = dp_cws / (self.params.z_l + 1.0); // 每级低压加热器及除氧器的平均压降

        // 一级低压给水加热器
        let (p_fw1i, h_fw1i, t_fw1i, p_fw1o, h_fw1o, t_fw1o, t_ro1k, h_ro1k, p_ro1k) =
            self.calc_fwxl(p_cwp, h_cwp, t_cwp, dp_fi, dh_fwl);
        // 二级低压给水加热器
        let (p_fw2i, h_fw2i, t_fw2i, p_fw2o, h_fw2o, t_fw2o, t_ro2k, h_ro2k, p_ro2k) =
            self.calc_fwxl(p_fw1o, h_fw1o, t_fw1o, dp_fi, dh_fwl);
        // 三级低压给水加热器
        let (p_fw3i, h_fw3i, t_fw3i, p_fw3o, h_fw3o, t_fw3o, t_ro3k, h_ro3k, p_ro3k) =
            self.calc_fwxl(p_fw2o, h_fw2o, t_fw2o, dp_fi, dh_fwl);
        // 四级低压给水加热器
        let (p_fw4i, h_fw4i, t_fw4i, p_fw4o, h_fw4o, t_fw4o, t_ro4k, h_ro4k, p_ro4k) =
            self.calc_fwxl(p_fw3o, h_fw3o, t_fw3o, dp_fi, dh_fwl);

        // 除氧器
        let h_deai = h_fw4o; // 进口给水比焓
        // let h_deao = h_deai + dh_fw; // 出口给水比焓
        // let t_deao = hx(h_deao, 0.0, OT); // 出口给水温度
        let p_fwpo = self.params.dp_fwpo * self.params.p_s; // 给水泵出口压力
        let h_fwpo = h_deao; // 给水泵出口流体比焓
        let t_fwpo = ph(p_fwpo, h_fwpo, OT); // 给水泵出口水温
        let p_fwi = self.params.p_s + 0.1; // GS二次侧进口给水压力

        // 六级高压给水加热器
        let (p_fw6i, h_fw6i, t_fw6i, p_fw6o, h_fw6o, t_fw6o, t_ro6k, h_ro6k, p_ro6k) = self
            .calc_fwxh(
                p_fwpo,
                h_fwpo,
                t_fwpo,
                p_fwpo - (p_fwpo - p_fwi) / 2.0,
                dh_fwh,
            );
        // 七级高压给水加热器
        let (p_fw7i, h_fw7i, t_fw7i, p_fw7o, h_fw7o, t_fw7o, t_ro7k, h_ro7k, p_ro7k) =
            self.calc_fwxh(p_fw6o, h_fw6o, t_fw6o, p_fwi, dh_fwh);

        // 高压缸抽汽
        // 六级给水加热器抽气参数
        let (p_hes6, h_hes6s, h_hes6, x_hes6, t_hes6) =
            self.calc_esx(t_fw6o, p_ro6k, s_hi, h_hi, true);
        // 七级给水加热器抽气参数
        let (p_hes7, h_hes7s, h_hes7, x_hes7, t_hes7) =
            self.calc_esx(t_fw7o, p_ro7k, s_hi, h_hi, true);

        // 低压缸抽汽
        // 一级给水加热器抽汽参数
        let (p_les1, h_les1s, h_les1, x_les1, t_les1) =
            self.calc_esx(t_fw1o, p_ro1k, s_li, h_li, false);
        // 二级给水加热器抽汽参数
        let (p_les2, h_les2s, h_les2, x_les2, t_les2) =
            self.calc_esx(t_fw2o, p_ro2k, s_li, h_li, false);
        // 三级给水加热器抽汽参数
        let (p_les3, h_les3s, h_les3, x_les3, t_les3) =
            self.calc_esx(t_fw3o, p_ro3k, s_li, h_li, false);
        // 四级给水加热器抽汽参数
        let (p_les4, h_les4s, h_les4, x_les4, t_les4) =
            self.calc_esx(t_fw4o, p_ro4k, s_li, h_li, false);

        // 再热器抽汽
        // 一级再热器抽汽参数
        let (p_rh1, x_rh1, t_rh1, h_rh1, h_zs1) = self.calc_rhx(p_hes7, x_hes7);
        // 二级再热器抽汽参数
        let (p_rh2, x_rh2, t_rh2, h_rh2, h_zs2) = self.calc_rhx(p_hi, x_hi);
        // 蒸汽发生器总蒸汽产量的计算
        let h_a = h_hi - h_hz; // 给水泵汽轮机中蒸汽的绝热焓降
        loop {
            let mut q_r = self.params.ne / self.params.ne_npp; // 反应堆热功率(MW)
            let mut d_s = (1000.0 * q_r * self.params.n_1)
                / ((h_fh - h_s) + (1.0 + self.params.zeta_d) * (h_s - h_fw)); // GS蒸汽产量(kg/s)
            let mut g_fw = (1.0 + self.params.zeta_d) * d_s; // GS给水流量(kg/s)
            let h_fwp = p_fwpo - p_dea; // 给水泵扬程(MPa)
            let rho_fwp = 0.5 * (px(p_dea, 0.0, OD) + px(p_fwpo, 0.0, OD)); // 给水泵中水的密度，定为给水泵进出口密度平均值

            let (
                mut g_sh,
                mut g_sl,
                mut g_fwps,
                mut g_les4,
                mut g_les3,
                mut g_les2,
                mut g_les1,
                mut g_zc1,
                mut g_zc2,
                mut g_hes6,
                mut g_hes7,
                mut g_uw,
                mut g_sdea,
            );
            // loop {
            let n_fwpp = 1000.0 * g_fw * h_fwp / rho_fwp; // 给水泵有效输出功率(kW)
            let n_fwpt = n_fwpp
                / (self.params.n_fwpp
                    * self.params.n_fwpti
                    * self.params.n_fwptm
                    * self.params.n_fwptg); // 给水泵理论功率(kW)
            g_fwps = n_fwpt / h_a; // 给水泵汽轮机耗汽量(kg/s)
            // 低压给水加热器抽汽量
            g_les4 = self.params.g_cd * (h_fw4o - h_fw4i) / (self.params.n_h * (h_les4 - h_ro4k)); // 第四级抽汽量
            g_les3 = (self.params.g_cd * (h_fw3o - h_fw3i)
                - self.params.n_h * g_les4 * (h_ro4k - h_ro3k))
                / (self.params.n_h * (h_les3 - h_ro3k)); // 第三级抽汽量
            g_les2 = (self.params.g_cd * (h_fw2o - h_fw2i)
                - self.params.n_h * (g_les3 + g_les4) * (h_ro3k - h_ro2k))
                / (self.params.n_h * (h_les2 - h_ro2k)); // 第二级抽汽量
            g_les1 = (self.params.g_cd * (h_fw1o - h_fw1i)
                - self.params.n_h * (g_les2 + g_les3 + g_les4) * (h_ro2k - h_ro1k))
                / (self.params.n_h * (h_les1 - h_ro1k)); // 第一级抽汽量
            g_sl = self.params.g_cd - self.params.zeta_d * d_s - g_fwps; // 低压缸耗气量
            // g_sl = (0.6 * 1000.0 * self.params.ne / (self.params.n_m * self.params.n_ge)
            //     + g_les4 * (h_les4 - h_lz)
            //     + g_les3 * (h_les3 - h_lz)
            //     + g_les2 * (h_les2 - h_lz)
            //     + g_les1 * (h_les1 - h_lz))
            //     / (h_li - h_lz); // 低压缸耗气量(kg/s)
            // 再热器加热蒸汽量
            g_zc1 = g_sl * dh_rh / (self.params.n_h * (h_rh1 - h_zs1));
            g_zc2 = g_sl * dh_rh / (self.params.n_h * (h_rh2 - h_zs2));
            // 高压给水加热器抽汽量
            g_hes7 = (g_fw * (h_fw7o - h_fw7i) - self.params.n_h * g_zc2 * (h_zs2 - h_ro7k))
                / (self.params.n_h * (h_hes7 - h_ro7k));
            g_hes6 = (g_fw * (h_fw6o - h_fw6i)
                - self.params.n_h * g_zc1 * (h_zs1 - h_ro6k)
                - self.params.n_h * (g_zc2 + g_hes7) * (h_ro7k - h_ro6k))
                / (self.params.n_h * (h_hes6 - h_ro6k));
            g_uw = g_sl * (x_rh1i - x_spi) / x_spi; // 汽水分离器疏水流量(kg/s)
            // let g_h1 = g_sl + g_uw;
            // 除氧器耗汽量
            g_sdea = (g_fw * h_deao
                    - g_uw * h_uw // h_psi???
                    - self.params.g_cd * h_fw4o
                    - (g_zc1 + g_zc2 + g_hes6 + g_hes7) * h_ro6k)
                / h_hz;
            // let g_t = g_sdea + g_sl * x_rh1i / x_hz; // 高压缸出口排气总流量
            // 高压缸耗汽量
            g_sh = (0.4 * 1000.0 * self.params.ne / (self.params.n_m * self.params.n_ge)
                + g_hes7 * (h_hes7 - h_hz)
                + g_hes6 * (h_hes6 - h_hz)
                + g_zc1 * (h_rh1 - h_hz))
                / (h_hi - h_hz);
            // g_sh = ((0.4 * self.params.ne / (self.params.n_m * self.params.n_ge)
            //     - g_hes6 * (h_hi - h_hes6)
            //     - g_hes7 * (h_hi - h_hes7)
            //     - g_zc1 * (h_hi - h_rh1))
            //     / (h_hi - h_hz))
            //     + g_hes6
            //     + g_hes7
            //     + g_zc1;
            // 对假设冷凝水流量验证
            d_s = g_fwps + g_zc2 + g_sh; // 新蒸汽耗量
            let g_fw1 = (1.0 + self.params.zeta_d) * d_s; // 给水流量
            let g_cd1 = g_fw1 - g_sdea - g_uw - (g_hes6 + g_hes7 + g_zc1 + g_zc2);
            // if (g_cd1 - self.params.g_cd).abs() / self.params.g_cd < 1e-2 {
            //     break;
            // } else {
            //     self.params.g_cd = g_cd1;
            //     g_fw = g_fw1;
            // }
            // }
            q_r = (d_s * (h_fh - h_fw) + self.params.zeta_d * d_s * (h_s - h_fw))
                / (1000.0 * self.params.n_1); // 新反应堆热功率(MW)
            let n_ennp1 = self.params.ne / q_r;
            self.results.result1.push(CalcResult1 {
                eta_enpp: self.params.ne_npp,
                g_shp: g_sh,
                g_slp: g_sl,
                g_srh1: g_zc1,
                g_srh2: g_zc2,
                g_sfwp: g_fwps,
                g_cd: self.params.g_cd,
                g_sdea,
                q_r: q_r / 1000.0, // kW -> MW
                d_s,
                g_fw,
                h_fwp,
                g_hes7,
                g_hes6,
                g_les4,
                g_les3,
                g_les2,
                g_les1,
                g_uw,
                g_zc1,
                g_zc2,
            });
            if (g_cd1 - self.params.g_cd).abs() / self.params.g_cd < 1e-2
                && (n_ennp1 - self.params.ne_npp).abs() < 1e-3
            {
                break;
            } else {
                self.params.ne_npp = n_ennp1;
                self.params.g_cd = g_cd1;
            }
        }

        // 存储附表结果
        self.results.result2 = CalcResult2 {
            ne: self.params.ne,
            eta_1: self.params.n_1,
            x_fh: self.params.x_fh,
            zeta_d: self.params.zeta_d,
            eta_hi: self.params.n_hi,
            eta_li: self.params.n_hi,
            eta_m: self.params.n_m,
            eta_ge: self.params.n_ge,
            dp_fh: self.params.dp_fh,
            dp_rh,
            dp_ej: self.params.dp_ej,
            dp_cd: self.params.dp_cd,
            theta_hu: self.params.theta_hu,
            theta_lu: self.params.theta_lu,
            eta_h: self.params.n_h,
            eta_fwpp: self.params.n_fwpp,
            eta_fwpti: self.params.n_fwpti,
            eta_fwptm: self.params.n_fwptm,
            eta_fwptg: self.params.n_fwptg,
            t_sw1: self.params.t_sw1,
            p_c: self.params.p_c,
            t_cs,
            dt_sub: self.params.dt_sub,
            t_co,
            dt_c: self.params.dt_c,
            t_ci,
            p_s: self.params.p_s,
            t_fh,
            dt_m,
            dt_sw: self.params.dt_sw,
            dt: self.params.dt,
            t_cd,
            p_cd,
            p_hi,
            x_hi,
            h_fh,
            s_fh,
            s_hi,
            p_hz,
            x_hz,
            h_hi,
            h_hzs,
            h_hz,
            p_spi,
            x_spi,
            p_uw,
            h_uw,
            p_rh1i,
            x_rh1i,
            h_rh1i,
            p_rh1hs: p_rh1,
            x_rh1hs: x_rh1,
            p_rh2i,
            t_rh2i,
            p_rh2z,
            t_rh2z,
            h_rh2z,
            dh_rh,
            h_rh1z,
            h_rh2i,
            p_rh2hs,
            x_rh2hs,
            p_li,
            t_li,
            p_lz,
            x_lz,
            s_li,
            h_li,
            h_lzs,
            h_lz,
            z: self.params.z,
            z_l: self.params.z_l,
            z_h: self.params.z_h,
            dh_fw,
            h_s,
            h_cd,
            dh_fwop,
            h_fwop,
            t_fwop,
            t_fw,
            h_fw,
            dh_fwh,
            p_dea,
            h_deao,
            dh_fwl,
            p_cwp,
            h_cwp,
            dp_cws,
            dp_fi,
            lfwx: vec![
                CalcFWParameters {
                    p_fwxi: p_fw1i,
                    h_fwxi: h_fw1i,
                    t_fwxi: t_fw1i,
                    p_fwxo: p_fw1o,
                    h_fwxo: h_fw1o,
                    t_fwxo: t_fw1o,
                    t_roxk: t_ro1k,
                    h_roxk: h_ro1k,
                },
                CalcFWParameters {
                    p_fwxi: p_fw2i,
                    h_fwxi: h_fw2i,
                    t_fwxi: t_fw2i,
                    p_fwxo: p_fw2o,
                    h_fwxo: h_fw2o,
                    t_fwxo: t_fw2o,
                    t_roxk: t_ro2k,
                    h_roxk: h_ro2k,
                },
                CalcFWParameters {
                    p_fwxi: p_fw3i,
                    h_fwxi: h_fw3i,
                    t_fwxi: t_fw3i,
                    p_fwxo: p_fw3o,
                    h_fwxo: h_fw3o,
                    t_fwxo: t_fw3o,
                    t_roxk: t_ro3k,
                    h_roxk: h_ro3k,
                },
                CalcFWParameters {
                    p_fwxi: p_fw4i,
                    h_fwxi: h_fw4i,
                    t_fwxi: t_fw4i,
                    p_fwxo: p_fw4o,
                    h_fwxo: h_fw4o,
                    t_fwxo: t_fw4o,
                    t_roxk: t_ro4k,
                    h_roxk: h_ro4k,
                },
            ],
            h_deai,
            h_deao1: h_deao,
            t_dea: t_deao,
            p_dea1: p_dea,
            p_fwpo,
            h_fwpo,
            p_fwi,
            hfwx: vec![
                CalcFWParameters {
                    p_fwxi: p_fw6i,
                    h_fwxi: h_fw6i,
                    t_fwxi: t_fw6i,
                    p_fwxo: p_fw6o,
                    h_fwxo: h_fw6o,
                    t_fwxo: t_fw6o,
                    t_roxk: t_ro6k,
                    h_roxk: h_ro6k,
                },
                CalcFWParameters {
                    p_fwxi: p_fw7i,
                    h_fwxi: h_fw7i,
                    t_fwxi: t_fw7i,
                    p_fwxo: p_fw7o,
                    h_fwxo: h_fw7o,
                    t_fwxo: t_fw7o,
                    t_roxk: t_ro7k,
                    h_roxk: h_ro7k,
                },
            ],
            s_hi1: s_hi,
            h_hi1: h_hi,
            hhes: vec![
                CalcHESParameters {
                    p_hesx: p_hes6,
                    x_hesx: x_hes6,
                    h_hesxs: h_hes6s,
                    h_hesx: h_hes6,
                    t_hesx: t_hes6,
                },
                CalcHESParameters {
                    p_hesx: p_hes7,
                    x_hesx: x_hes7,
                    h_hesxs: h_hes7s,
                    h_hesx: h_hes7,
                    t_hesx: t_hes7,
                },
            ],
            s_li1: s_li,
            h_li1: h_li,
            lhes: vec![
                CalcHESParameters {
                    p_hesx: p_les1,
                    x_hesx: x_les1,
                    h_hesxs: h_les1s,
                    h_hesx: h_les1,
                    t_hesx: t_les1,
                },
                CalcHESParameters {
                    p_hesx: p_les2,
                    x_hesx: x_les2,
                    h_hesxs: h_les2s,
                    h_hesx: h_les2,
                    t_hesx: t_les2,
                },
                CalcHESParameters {
                    p_hesx: p_les3,
                    x_hesx: x_les3,
                    h_hesxs: h_les3s,
                    h_hesx: h_les3,
                    t_hesx: t_les3,
                },
                CalcHESParameters {
                    p_hesx: p_les4,
                    x_hesx: x_les4,
                    h_hesxs: h_les4s,
                    h_hesx: h_les4,
                    t_hesx: t_les4,
                },
            ],
            rhx: vec![
                CalcRHXParameters {
                    p_rhx: p_rh1,
                    x_rhx: x_rh1,
                    t_rhx: t_rh1,
                    h_rhx: h_rh1,
                    h_zsx: h_zs1,
                },
                CalcRHXParameters {
                    p_rhx: p_rh2,
                    x_rhx: x_rh2,
                    t_rhx: t_rh2,
                    h_rhx: h_rh2,
                    h_zsx: h_zs2,
                },
            ],
        };

        Ok(())
    }

    /// 计算低压加热器参数
    /// # Arguments
    ///
    /// * `p_fwxi` - 进口给水压力
    ///
    /// * `h_fwxi` - 进口给水比焓
    ///
    /// * `t_fwxi` - 进口给水温度
    ///
    /// * `dp_fi` - 每级加热器及除氧器的平均压降
    ///
    /// * `dh_fw` - 每级给水焓升
    ///
    fn calc_fwxl(
        &self,
        p_fwxi: f64,
        h_fwxi: f64,
        t_fwxi: f64,
        dp_fi: f64,
        dh_fw: f64,
    ) -> (f64, f64, f64, f64, f64, f64, f64, f64, f64) {
        let p_fwxo = p_fwxi - dp_fi; // 出口给水压力
        let h_fwxo = h_fwxi + dh_fw; // 出口给水比焓
        // println!("h_fwxi: {}, h_fwxo: {}, dh_fw: {}", h_fwxi, h_fwxo, dh_fw);
        let t_fwxo = ph(p_fwxo, h_fwxo, OT); // 出口给水温度
        let t_roxk = t_fwxo + self.params.theta_lu; // 出口疏水温度
        let h_roxk = tx(t_roxk, 0.0, OH); // 出口疏水比焓
        let p_roxk = tx(t_roxk, 0.0, OP); // 出口疏水压力
        (
            p_fwxi, h_fwxi, t_fwxi, p_fwxo, h_fwxo, t_fwxo, t_roxk, h_roxk, p_roxk,
        )
    }

    /// 计算高压加热器参数
    /// # Arguments
    ///
    /// * `p_fwxi` - 进口给水压力
    ///
    /// * `h_fwxi` - 进口给水比焓
    ///
    /// * `t_fwxi` - 进口给水温度
    ///
    /// * `p_fwxo` - 出口给水温度
    ///
    /// * `dh_fw` - 每级给水焓升
    ///
    fn calc_fwxh(
        &self,
        p_fwxi: f64,
        h_fwxi: f64,
        t_fwxi: f64,
        p_fwxo: f64,
        dh_fw: f64,
    ) -> (f64, f64, f64, f64, f64, f64, f64, f64, f64) {
        let h_fwxo = h_fwxi + dh_fw; // 出口给水比焓
        let t_fwxo = ph(p_fwxo, h_fwxo, OT); // 出口给水温度
        let t_roxk = t_fwxo + self.params.theta_hu; // 出口疏水温度
        let p_roxk = tx(t_roxk, 0.0, OP); // 出口疏水压力
        let h_roxk = px(p_roxk, 0.0, OH); // 出口疏水比焓
        (
            p_fwxi, h_fwxi, t_fwxi, p_fwxo, h_fwxo, t_fwxo, t_roxk, h_roxk, p_roxk,
        )
    }

    /// 计算加热器抽汽参数
    ///
    /// # Arguments
    ///
    /// * `t_fwxo` - 出口给水温度
    ///
    /// * `p_roxk` - 汽侧疏水压力
    ///
    /// * `s_i` - 进口蒸汽比熵
    ///
    /// * `h_i` - 进口进气比焓
    // TODO: 移除`_p_roxk`
    fn calc_esx(
        &self,
        t_fwxo: f64,
        _p_roxk: f64,
        s_i: f64,
        h_i: f64,
        is_h: bool,
    ) -> (f64, f64, f64, f64, f64) {
        let t_esx = t_fwxo
            + if is_h {
                self.params.theta_hu
            } else {
                self.params.theta_lu
            }; // 抽汽温度
        let p_esx = tx(t_esx, 1.0, OP) / (1.0 - self.params.dp_ej);
        let h_esxs = ps(p_esx, s_i, OH); // 抽气理想比焓
        let h_esx = h_i
            - if is_h {
                self.params.n_hi
            } else {
                self.params.n_li
            } * (h_i - h_esxs); // 抽气比焓
        let x_esx = ph(p_esx, h_esx, OX); // 抽气干度
        (p_esx, h_esxs, h_esx, x_esx, t_esx)
    }

    /// 计算再热器抽汽参数
    fn calc_rhx(&self, p_rhx: f64, x_rhx: f64) -> (f64, f64, f64, f64, f64) {
        let t_rhx = px(p_rhx, x_rhx, OT); // 加热蒸汽进口温度
        let h_rhx = px(p_rhx, x_rhx, OH); // 加热蒸汽进口比焓
        let h_zsx = px(p_rhx, 0.0, OH); // 再热器疏水比焓
        (p_rhx, x_rhx, t_rhx, h_rhx, h_zsx)
    }

    /// 将计算参数保存到json文件
    pub fn save_parameters_to_file(&self, base_path: &str) -> std::io::Result<()> {
        let file = File::create(format!("{}/parameters.json", base_path))?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self.params)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(())
    }

    /// 将计算结果保存到json文件
    pub fn save_results_to_file(&self, base_path: &str) -> std::io::Result<()> {
        let file = File::create(format!("{}/results.json", base_path))?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self.results)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(())
    }

    /// 获取计算结果
    pub fn get_results(&self) -> Option<&CalcResultParamters> {
        if self.results.result1.is_empty() {
            None
        } else {
            Some(&self.results)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculator() {
        let mut calculator = Calculator::default();
        calculator.calculate().unwrap();
        let result = calculator.get_results();
        assert!(result.is_some(), "Expect result is Some(...)");
    }
}
