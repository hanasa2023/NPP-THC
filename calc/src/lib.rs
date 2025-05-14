pub mod parameters;

use std::fs::File;
use std::io::BufWriter;
use std::{error::Error, io::Write};

use parameters::{
    CalcFWParameters, CalcHESParameters, CalcInputParameters, CalcRHXParameters, CalcResult1,
    CalcResult2, CalcResultParamters,
};
use seuif97::*;

pub struct Calculator {
    params: CalcInputParameters,
    results: CalcResultParamters,
}

impl Default for Calculator {
    fn default() -> Self {
        Self {
            params: CalcInputParameters::default(),
            results: CalcResultParamters::default(),
        }
    }
}

impl Calculator {
    pub fn new(params: CalcInputParameters) -> Self {
        Self {
            params,
            results: CalcResultParamters::default(),
        }
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
        let (p_hes6, h_hes6s, h_hes6, x_hes6) = self.calc_esx(p_ro6k, s_hi, h_hi, true);
        // 七级给水加热器抽气参数
        let (p_hes7, h_hes7s, h_hes7, x_hes7) = self.calc_esx(p_ro7k, s_hi, h_hi, true);

        // 低压缸抽汽
        // 一级给水加热器抽汽参数
        let (p_les1, h_les1s, h_les1, x_les1) = self.calc_esx(p_ro1k, s_li, h_li, false);
        // 二级给水加热器抽汽参数
        let (p_les2, h_les2s, h_les2, x_les2) = self.calc_esx(p_ro2k, s_li, h_li, false);
        // 三级给水加热器抽汽参数
        let (p_les3, h_les3s, h_les3, x_les3) = self.calc_esx(p_ro3k, s_li, h_li, false);
        // 四级给水加热器抽汽参数
        let (p_les4, h_les4s, h_les4, x_les4) = self.calc_esx(p_ro4k, s_li, h_li, false);

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
            loop {
                let n_fwpp = 1000.0 * g_fw * h_fwp / rho_fwp; // 给水泵有效输出功率(kW)
                let n_fwpt = n_fwpp
                    / (self.params.n_fwpp
                        * self.params.n_fwpti
                        * self.params.n_fwptm
                        * self.params.n_fwptg); // 给水泵理论功率(kW)
                g_fwps = n_fwpt / h_a; // 给水泵汽轮机耗汽量(kg/s)
                // 低压给水加热器抽汽量
                g_les4 =
                    self.params.g_cd * (h_fw4o - h_fw4i) / (self.params.n_h * (h_les4 - h_ro4k)); // 第四级抽汽量
                g_les3 = (self.params.g_cd * (h_fw3o - h_fw3i)
                    - self.params.n_h * g_les4 * (h_ro4k - h_ro3k))
                    / (self.params.n_h * (h_les3 - h_ro3k)); // 第三级抽汽量
                g_les2 = (self.params.g_cd * (h_fw2o - h_fw2i)
                    - self.params.n_h * (g_les3 + g_les4) * (h_ro3k - h_ro2k))
                    / (self.params.n_h * (h_les2 - h_ro2k)); // 第二级抽汽量
                g_les1 = (self.params.g_cd * (h_fw1o - h_fw1i)
                    - self.params.n_h * (g_les2 + g_les3 + g_les4) * (h_ro2k - h_ro1k))
                    / (self.params.n_h * (h_les1 - h_ro1k)); // 第一级抽汽量
                // g_sl = self.params.g_cd - self.params.zeta_d * d_s - g_fwps; // 低压缸耗气量
                g_sl = (0.6 * 1000.0 * self.params.ne / (self.params.n_m * self.params.n_ge)
                    + g_les4 * (h_les4 - h_lz)
                    + g_les3 * (h_les3 - h_lz)
                    + g_les2 * (h_les2 - h_lz)
                    + g_les1 * (h_les1 - h_lz))
                    / (h_li - h_lz); // 低压缸耗气量(kg/s)
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
                if (g_cd1 - self.params.g_cd).abs() / self.params.g_cd < 1e-2 {
                    break;
                } else {
                    self.params.g_cd = g_cd1;
                    g_fw = g_fw1;
                }
            }
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
            if (n_ennp1 - self.params.ne_npp).abs() < 1e-3 {
                break;
            } else {
                self.params.ne_npp = n_ennp1;
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
                },
                CalcHESParameters {
                    p_hesx: p_hes7,
                    x_hesx: x_hes7,
                    h_hesxs: h_hes7s,
                    h_hesx: h_hes7,
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
                },
                CalcHESParameters {
                    p_hesx: p_les2,
                    x_hesx: x_les2,
                    h_hesxs: h_les2s,
                    h_hesx: h_les2,
                },
                CalcHESParameters {
                    p_hesx: p_les3,
                    x_hesx: x_les3,
                    h_hesxs: h_les3s,
                    h_hesx: h_les3,
                },
                CalcHESParameters {
                    p_hesx: p_les4,
                    x_hesx: x_les4,
                    h_hesxs: h_les4s,
                    h_hesx: h_les4,
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

        return Ok(());
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
    /// * `p_roxk` - 汽侧疏水压力
    ///
    /// * `s_i` - 进口蒸汽比熵
    ///
    /// * `h_i` - 进口进气比焓
    fn calc_esx(&self, p_roxk: f64, s_i: f64, h_i: f64, is_h: bool) -> (f64, f64, f64, f64) {
        // TODO: 验证合理性
        let p_esx = p_roxk / (1.0 - self.params.dp_ej); // 抽气压力
        let h_esxs = ps(p_esx, s_i, OH); // 抽气理想比焓
        let h_esx = h_i
            - if is_h {
                self.params.n_hi
            } else {
                self.params.n_li
            } * (h_i - h_esxs); // 抽气比焓
        let x_esx = ph(p_esx, h_esx, OX); // 抽气干度
        (p_esx, h_esxs, h_esx, x_esx)
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
        let file = File::create(&format!("{}/parameters.json", base_path))?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self.params)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(())
    }

    /// 将计算结果保存到json文件
    pub fn save_results_to_file(&self, base_path: &str) -> std::io::Result<()> {
        let file = File::create(&format!("{}/results.json", base_path))?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self.results)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(())
    }

    /// 将计算代码保存到文件
    pub fn save_code_to_file(&self, base_path: &str) -> std::io::Result<()> {
        let rs_code = self.generate_calc_code_rs();
        let file = File::create(&format!("{}/calc.rs", base_path))?;
        let mut writer = BufWriter::new(file);
        writer.write_all(rs_code.as_bytes())?;
        let py_code = self.generate_calc_code_py();
        let file = File::create(&format!("{}/calc.py", base_path))?;
        let mut writer = BufWriter::new(file);
        writer.write_all(py_code.as_bytes())?;
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

    /// 生成计算代码
    pub fn generate_calc_code_rs(&self) -> String {
        let mut code = String::new();
        // 使用 self.params 来获取输入参数的实际值
        let params = &self.params;

        code.push_str("fn main () {\n");
        code.push_str("\t// 根据 Calculator::calculate 方法生成的计算过程代码\n");

        code.push_str("\t// --- 计算开始 ---\n\n");

        code.push_str("\t// 一回路冷却剂参数\n");
        let t_cs = px(params.p_c, 0.0, OT);
        code.push_str(&format!(
            "\tlet t_cs = {:.4}; // 工作压力对应饱和温度(℃)\n",
            t_cs
        ));
        let t_co = t_cs - params.dt_sub;
        code.push_str(&format!(
            "\tlet t_co = {:.4}; // 反应堆出口冷却剂温度(℃)\n",
            t_co
        ));
        let t_ci = t_co - params.dt_c;
        code.push_str(&format!(
            "\tlet t_ci = {:.4}; // 反应堆进口冷却剂温度(℃)\n\n",
            t_ci
        ));

        code.push_str("\t// 蒸汽初参数\n");
        let t_s = px(params.p_s, 1.0, OT);
        code.push_str(&format!("\tlet t_s = {:.4}; // 对应的饱和温度(℃)\n", t_s));
        let t_fh = px(params.p_s, params.x_fh, OT);
        code.push_str(&format!("\tlet t_fh = {:.4}; // 新蒸汽温度(℃)\n", t_fh));
        let h_fh = tx(t_fh, params.x_fh, OH);
        code.push_str(&format!("\tlet h_fh = {:.4}; // 新蒸汽比焓(℃)\n", h_fh));
        let s_fh = px(params.p_s, params.x_fh, OS);
        code.push_str(&format!(
            "\tlet s_fh = {:.4}; // 新蒸汽比熵(kJ/(kg * K))\n",
            s_fh
        ));
        let dt_m = (t_co - t_ci) / f64::ln((t_co - t_s) / (t_ci - t_s));
        code.push_str(&format!(
            "\tlet dt_m = {:.4}; // 对数平均传热温差(℃)\n\n",
            dt_m
        ));

        code.push_str("\t// 蒸汽终参数\n");
        let t_cd = params.t_sw1 + params.dt_sw + params.dt;
        code.push_str(&format!(
            "\tlet t_cd = {:.4}; // 冷凝器凝结水饱和温度(℃)\n",
            t_cd
        ));
        let p_cd = tx(t_cd, 0.0, OP);
        code.push_str(&format!("\tlet p_cd = {:.4}; // 凝结水压力(MPa)\n\n", p_cd));

        code.push_str("\t// 高压缸参数\n");
        let dp_fh_calc = params.dp_fh * params.p_s;
        code.push_str(&format!(
            "\tlet dp_fh_calc = {:.4}; // 新蒸汽压损(MPa)\n",
            dp_fh_calc
        ));
        let p_hi = params.p_s - dp_fh_calc;
        code.push_str(&format!(
            "\tlet p_hi = {:.4}; // 高压缸进口蒸汽压力(MPa)\n",
            p_hi
        ));
        let h_hi = px(p_hi, 1.0, OH);
        code.push_str(&format!(
            "\tlet h_hi = {:.4}; // 高压缸进口蒸汽比焓(kJ/kg)\n",
            h_hi
        ));
        let x_hi = ph(p_hi, h_hi, OX);
        code.push_str(&format!("\tlet x_hi = {:.4}; // 进口蒸汽干度(%)\n", x_hi));
        let s_hi = ph(p_hi, h_hi, OS);
        code.push_str(&format!(
            "\tlet s_hi = {:.4}; // 进口蒸汽比熵(kJ/(kg * K))\n",
            s_hi
        ));
        let p_hz = params.dp_hz * p_hi;
        code.push_str(&format!("\tlet p_hz = {:.4}; // 排气压力(MPa)\n", p_hz));
        let h_hzs = ps(p_hz, s_hi, OH);
        code.push_str(&format!(
            "\tlet h_hzs = {:.4}; // 高压缸排气理想比焓(kJ/kg)\n",
            h_hzs
        ));
        let h_hz = h_hi - params.n_hi * (h_hi - h_hzs);
        code.push_str(&format!(
            "\tlet h_hz = {:.4}; // 高压缸排气实际比焓(kJ/kg)\n",
            h_hz
        ));
        let x_hz = ph(p_hz, h_hz, OX);
        code.push_str(&format!("\tlet x_hz = {:.4}; // 排气干度 (%)\n\n", x_hz));

        code.push_str("\t// 蒸汽中间再热参数\n");
        let dp_rh_calc = params.dp_rh * p_hz;
        code.push_str(&format!(
            "\tlet dp_rh_calc = {:.4}; // 再热蒸汽压损 (MPa)\n",
            dp_rh_calc
        ));
        let p_spi = p_hz;
        code.push_str(&format!(
            "\tlet p_spi = {:.4}; // 汽水分离器进口蒸汽压力 (MPa)\n",
            p_spi
        ));
        let x_spi = x_hz;
        code.push_str(&format!(
            "\tlet x_spi = {:.4}; // 汽水分离器进口蒸汽干度 (%))\n",
            x_spi
        ));
        let _h_spi = px(p_hz, 0.0, OH);
        code.push_str(&format!(
            "\tlet _h_spi = {:.4}; // 汽水分离器入口焓值 (kJ/kg)\n",
            _h_spi
        ));
        let p_uw = 0.99 * p_hz;
        code.push_str(&format!(
            "\tlet p_uw = {:.4}; // 汽水分离器出口疏水压力 (MPa)\n",
            p_uw
        ));
        let h_uw = px(p_uw, 0.0, OH);
        code.push_str(&format!(
            "\tlet h_uw = {:.4}; // 汽水分离器出口疏水比焓 (kJ/kg)\n\n",
            h_uw
        ));

        code.push_str("\t// 一级再热器\n");
        let p_rh1i = 0.99 * p_hz;
        code.push_str(&format!(
            "\tlet p_rh1i = {:.4}; // 一级再热器进口蒸汽压力 (MPa)\n",
            p_rh1i
        ));
        let x_rh1i = x_spi / (1.0 - 0.98 * (1.0 - x_spi));
        code.push_str(&format!(
            "\tlet x_rh1i = {:.4}; // 一级再热器进口蒸汽干度 (%)\n",
            x_rh1i
        ));
        let h_rh1i = px(p_rh1i, x_rh1i, OH);
        code.push_str(&format!(
            "\tlet h_rh1i = {:.4}; // 一级再热器进口蒸汽比焓 (kJ/kg)\n\n",
            h_rh1i
        ));

        code.push_str("\t// 二级再热器\n");
        let p_rh2i = 0.98 * p_hz;
        code.push_str(&format!(
            "\tlet p_rh2i = {:.4}; // 二级再热器进口蒸汽压力 (MPa)\n",
            p_rh2i
        ));
        let p_rh2z = 0.97 * p_hz;
        code.push_str(&format!(
            "\tlet p_rh2z = {:.4}; // 二级再热器出口压力 (MPa)\n",
            p_rh2z
        ));
        let t_rh2z = t_fh - params.t_rh2z;
        code.push_str(&format!(
            "\tlet t_rh2z = {:.4}; // 二级再热器出口温度 (℃)\n",
            t_rh2z
        ));
        let h_rh2z = pt(p_rh2z, t_rh2z, OH);
        code.push_str(&format!(
            "\tlet h_rh2z = {:.4}; // 二级再热器出口蒸汽比焓 (kJ/kg)\n",
            h_rh2z
        ));
        let dh_rh = (h_rh2z - h_rh1i) / 2.0;
        code.push_str(&format!(
            "\tlet dh_rh = {:.4}; // 每级再热器平均焓升 (kJ/kg)\n",
            dh_rh
        ));
        let h_rh1z = h_rh1i + dh_rh;
        code.push_str(&format!(
            "\tlet h_rh1z = {:.4}; // 一级再热器出口蒸汽比焓 (kJ/kg)\n",
            h_rh1z
        ));
        let h_rh2i = h_rh1z;
        code.push_str(&format!(
            "\tlet h_rh2i = {:.4}; // 二级再热器进口蒸汽比焓 (kJ/kg)\n",
            h_rh2i
        ));
        let t_rh2i = ph(p_rh2i, h_rh2i, OT);
        code.push_str(&format!(
            "\tlet t_rh2i = {:.4}; // 二级再热器进口蒸汽温度 (℃)\n",
            t_rh2i
        ));
        let p_rh2hs = p_hi;
        code.push_str(&format!(
            "\tlet p_rh2hs = {:.4}; // 加热(新)蒸汽进口压力 (MPa)\n",
            p_rh2hs
        ));
        let x_rh2hs = x_hi;
        code.push_str(&format!(
            "\tlet x_rh2hs = {:.4}; // 加热(新)蒸汽进口干度 (%)\n\n",
            x_rh2hs
        ));

        code.push_str("\t// 低压缸参数\n");
        let p_li = (1.0 - params.dp_f) * p_rh2z;
        code.push_str(&format!(
            "\tlet p_li = {:.4}; // 低压缸进气压力 (MPa)\n",
            p_li
        ));
        let h_li = h_rh2z;
        code.push_str(&format!(
            "\tlet h_li = {:.4}; // 低压缸进口进气比焓 (kJ/kg)\n",
            h_li
        ));
        let t_li = ph(p_li, h_li, OT);
        code.push_str(&format!("\tlet t_li = {:.4}; // 进口蒸汽温度 ()\n", t_li));
        let dp_cd_calc = (1.0 / (1.0 - params.dp_cd) - 1.0) * p_cd;
        code.push_str(&format!(
            "\tlet dp_cd_calc = {:.4}; // 低压缸排气压损 (MPa)\n",
            dp_cd_calc
        ));
        let p_lz = p_cd + dp_cd_calc;
        code.push_str(&format!(
            "\tlet p_lz = {:.4}; // 低压缸排气压力 (MPa)\n",
            p_lz
        ));
        let s_li = ph(p_li, h_li, OS);
        code.push_str(&format!("\tlet s_li = {:.4}; // 进口蒸汽比熵 ()\n", s_li));
        let s_lz = s_li;
        code.push_str(&format!(
            "\tlet s_lz = {:.4}; // 定熵过程，排气比熵 ()\n",
            s_lz
        ));
        let h_lzs = ps(p_lz, s_lz, OH);
        code.push_str(&format!(
            "\tlet h_lzs = {:.4}; // 低压缸排气理想比焓 ()\n",
            h_lzs
        ));
        let h_lz = h_li - params.n_li * (h_li - h_lzs);
        code.push_str(&format!("\tlet h_lz = {:.4}; // 排气实际比焓 ()\n", h_lz));
        let x_lz = ph(p_lz, h_lz, OX);
        code.push_str(&format!("\tlet x_lz = {:.4}; // 排气干度 ()\n\n", x_lz));

        code.push_str("\t// 给水的焓升分配\n");
        let h_s_calc = px(params.p_s, 0.0, OH);
        code.push_str(&format!(
            "\tlet h_s_calc = {:.4}; // GS工作压力下的饱和水焓 ()\n",
            h_s_calc
        ));
        let h_cd_val = tx(t_cd, 0.0, OH);
        code.push_str(&format!(
            "\tlet h_cd_val = {:.4}; // 冷凝器出口凝结水比焓 ()\n",
            h_cd_val
        ));
        let dh_fwop = (h_s_calc - h_cd_val) / (params.z + 1.0);
        code.push_str(&format!(
            "\tlet dh_fwop = {:.4}; // 理论给水焓升 ()\n",
            dh_fwop
        ));
        let h_fwop = h_cd_val + params.z * dh_fwop;
        code.push_str(&format!(
            "\tlet h_fwop = {:.4}; // GS最佳给水比焓 ()\n",
            h_fwop
        ));
        let t_fwop = ph(params.p_s, h_fwop, OT);
        code.push_str(&format!(
            "\tlet t_fwop = {:.4}; // 最佳给水温度 ()\n",
            t_fwop
        ));
        let t_fw_calc = params.dt_fw * t_fwop;
        code.push_str(&format!(
            "\tlet t_fw_calc = {:.4}; // 实际给水温度 ()\n",
            t_fw_calc
        ));
        let h_fw_calc = pt(params.p_s, t_fw_calc, OH);
        code.push_str(&format!(
            "\tlet h_fw_calc = {:.4}; // 实际给水比焓 ()\n",
            h_fw_calc
        ));
        let dh_fw_calc = (h_fw_calc - h_cd_val) / params.z;
        code.push_str(&format!(
            "\tlet dh_fw_calc = {:.4}; // 每一级加热器内实际给水焓升 ()\n\n",
            dh_fw_calc
        ));

        code.push_str("\t// 除氧器\n");
        let p_dea = 0.99 * p_hz;
        code.push_str(&format!(
            "\tlet p_dea = {:.4}; // 除氧器运行压力 ()\n",
            p_dea
        ));
        let t_deao = px(p_dea, 0.0, OT);
        code.push_str(&format!(
            "\tlet t_deao = {:.4}; // 除氧器出口温度 ()\n",
            t_deao
        ));
        let h_deao = tx(t_deao, 0.0, OH);
        code.push_str(&format!(
            "\tlet h_deao = {:.4}; // 除氧器出口对应饱和水比焓 ()\n",
            h_deao
        ));
        let dh_fwh = (h_fw_calc - h_deao) / params.z_h;
        code.push_str(&format!(
            "\tlet dh_fwh = {:.4}; // 高压给水加热器每一级给水焓升 ()\n",
            dh_fwh
        ));
        let dh_fwl = (h_deao - h_cd_val) / (params.z_l + 1.0);
        code.push_str(&format!(
            "\tlet dh_fwl = {:.4}; // 除氧器及低压加热器每一级给水焓升 ()\n\n",
            dh_fwl
        ));

        code.push_str("\t// 给水回路系统中的压力选择\n");
        let p_cwp = params.dp_cwp * p_dea;
        code.push_str(&format!(
            "\tlet p_cwp = {:.4}; // 凝水泵出口压力 ()\n",
            p_cwp
        ));
        let h_cwp = h_cd_val;
        code.push_str(&format!(
            "\tlet h_cwp = {:.4}; // 凝水泵出口给水比焓 ()\n",
            h_cwp
        ));
        let t_cwp = ph(p_cwp, h_cwp, OT);
        code.push_str(&format!(
            "\tlet t_cwp = {:.4}; // 凝水泵出口给水温度 ()\n",
            t_cwp
        ));
        let dp_cws = p_cwp - p_dea;
        code.push_str(&format!(
            "\tlet dp_cws = {:.4}; // 凝水泵出口至除氧器的阻力压降 ()\n",
            dp_cws
        ));
        let dp_fi = dp_cws / (params.z_l + 1.0);
        code.push_str(&format!(
            "\tlet dp_fi = {:.4}; // 每级低压加热器及除氧器的平均压降 ()\n\n",
            dp_fi
        ));

        code.push_str("\t// 低压给水加热器 \n");
        let (p_fw1i, h_fw1i, t_fw1i, p_fw1o, h_fw1o, t_fw1o, t_ro1k, h_ro1k, p_ro1k) =
            self.calc_fwxl(p_cwp, h_cwp, t_cwp, dp_fi, dh_fwl);
        code.push_str(&format!("\tlet (p_fw1i, h_fw1i, t_fw1i, p_fw1o, h_fw1o, t_fw1o, t_ro1k, h_ro1k, p_ro1k) = ({:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4});\n",
        p_fw1i, h_fw1i, t_fw1i, p_fw1o, h_fw1o, t_fw1o, t_ro1k, h_ro1k, p_ro1k));
        let (p_fw2i, h_fw2i, t_fw2i, p_fw2o, h_fw2o, t_fw2o, t_ro2k, h_ro2k, p_ro2k) =
            self.calc_fwxl(p_fw1o, h_fw1o, t_fw1o, dp_fi, dh_fwl);
        code.push_str(&format!("\tlet (p_fw2i, h_fw2i, t_fw2i, p_fw2o, h_fw2o, t_fw2o, t_ro2k, h_ro2k, p_ro2k) = ({:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4});\n",
        p_fw2i, h_fw2i, t_fw2i, p_fw2o, h_fw2o, t_fw2o, t_ro2k, h_ro2k, p_ro2k));
        let (p_fw3i, h_fw3i, t_fw3i, p_fw3o, h_fw3o, t_fw3o, t_ro3k, h_ro3k, p_ro3k) =
            self.calc_fwxl(p_fw2o, h_fw2o, t_fw2o, dp_fi, dh_fwl);
        code.push_str(&format!("\tlet (p_fw3i, h_fw3i, t_fw3i, p_fw3o, h_fw3o, t_fw3o, t_ro3k, h_ro3k, p_ro3k) = ({:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4});\n",
        p_fw3i, h_fw3i, t_fw3i, p_fw3o, h_fw3o, t_fw3o, t_ro3k, h_ro3k, p_ro3k));
        let (p_fw4i, h_fw4i, t_fw4i, p_fw4o, h_fw4o, t_fw4o, t_ro4k, h_ro4k, p_ro4k) =
            self.calc_fwxl(p_fw3o, h_fw3o, t_fw3o, dp_fi, dh_fwl);
        code.push_str(&format!("\tlet (p_fw4i, h_fw4i, t_fw4i, p_fw4o, h_fw4o, t_fw4o, t_ro4k, h_ro4k, p_ro4k) = ({:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4});\n",
        p_fw4i, h_fw4i, t_fw4i, p_fw4o, h_fw4o, t_fw4o, t_ro4k, h_ro4k, p_ro4k));

        code.push_str("\t// 除氧器 (续)\n");
        let h_deai = h_fw4o;
        code.push_str(&format!(
            "\tlet h_deai = {:.4}; // 进口给水比焓 ()\n",
            h_deai
        ));
        let p_fwpo = params.dp_fwpo * params.p_s;
        code.push_str(&format!(
            "\tlet p_fwpo = {:.4}; // 给水泵出口压力 ()\n",
            p_fwpo
        ));
        let h_fwpo_calc = h_deao;
        code.push_str(&format!(
            "\tlet h_fwpo_calc = {:.4}; // 给水泵出口流体比焓 ()\n",
            h_fwpo_calc
        ));
        let t_fwpo = ph(p_fwpo, h_fwpo_calc, OT);
        code.push_str(&format!(
            "\tlet t_fwpo = {:.4}; // 给水泵出口水温 ()\n",
            t_fwpo
        ));
        let p_fwi = params.p_s + 0.1;
        code.push_str(&format!(
            "\tlet p_fwi = {:.4}; // GS二次侧进口给水压力 ()\n\n",
            p_fwi
        ));

        code.push_str("\t// 高压给水加热器\n");
        let (p_fw6i, h_fw6i, t_fw6i, p_fw6o, h_fw6o, t_fw6o, t_ro6k, h_ro6k, p_ro6k) = self
            .calc_fwxh(
                p_fwpo,
                h_fwpo_calc,
                t_fwpo,
                p_fwpo - (p_fwpo - p_fwi) / 2.0,
                dh_fwh,
            );
        code.push_str(&format!("\tlet (p_fw6i, h_fw6i, t_fw6i, p_fw6o, h_fw6o, t_fw6o, t_ro6k, h_ro6k, p_ro6k) = ({:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4});\n",
        p_fw6i, h_fw6i, t_fw6i, p_fw6o, h_fw6o, t_fw6o, t_ro6k, h_ro6k, p_ro6k));
        let (p_fw7i, h_fw7i, t_fw7i, p_fw7o, h_fw7o, t_fw7o, t_ro7k, h_ro7k, p_ro7k) =
            self.calc_fwxh(p_fw6o, h_fw6o, t_fw6o, p_fwi, dh_fwh);
        code.push_str(&format!("\tlet (p_fw7i, h_fw7i, t_fw7i, p_fw7o, h_fw7o, t_fw7o, t_ro7k, h_ro7k, p_ro7k) = ({:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4});\n",
        p_fw7i, h_fw7i, t_fw7i, p_fw7o, h_fw7o, t_fw7o, t_ro7k, h_ro7k, p_ro7k));

        code.push_str("\t// 高压缸抽汽\n");
        let (p_hes6, h_hes6s, h_hes6, x_hes6) = self.calc_esx(p_ro6k, s_hi, h_hi, true);
        code.push_str(&format!(
            "\tlet (p_hes6, h_hes6s, h_hes6, x_hes6) = ({:.4}, {:.4}, {:.4}, {:.4});\n",
            p_hes6, h_hes6s, h_hes6, x_hes6
        ));
        let (p_hes7, h_hes7s, h_hes7, x_hes7) = self.calc_esx(p_ro7k, s_hi, h_hi, true);
        code.push_str(&format!(
            "\tlet (p_hes7, h_hes7s, h_hes7, x_hes7) = ({:.4}, {:.4}, {:.4}, {:.4});\n",
            p_hes7, h_hes7s, h_hes7, x_hes7
        ));

        code.push_str("\t// 低压缸抽汽\n");
        let (p_les1, h_les1s, h_les1, x_les1) = self.calc_esx(p_ro1k, s_li, h_li, false);
        code.push_str(&format!(
            "\tlet (p_les1, h_les1s, h_les1, x_les1) = ({:.4}, {:.4}, {:.4}, {:.4});\n",
            p_les1, h_les1s, h_les1, x_les1
        ));
        let (p_les2, h_les2s, h_les2, x_les2) = self.calc_esx(p_ro2k, s_li, h_li, false);
        code.push_str(&format!(
            "\tlet (p_les2, h_les2s, h_les2, x_les2) = ({:.4}, {:.4}, {:.4}, {:.4});\n",
            p_les2, h_les2s, h_les2, x_les2
        ));
        let (p_les3, h_les3s, h_les3, x_les3) = self.calc_esx(p_ro3k, s_li, h_li, false);
        code.push_str(&format!(
            "\tlet (p_les3, h_les3s, h_les3, x_les3) = ({:.4}, {:.4}, {:.4}, {:.4});\n",
            p_les3, h_les3s, h_les3, x_les3
        ));
        let (p_les4, h_les4s, h_les4, x_les4) = self.calc_esx(p_ro4k, s_li, h_li, false);
        code.push_str(&format!(
            "\tlet (p_les4, h_les4s, h_les4, x_les4) = ({:.4}, {:.4}, {:.4}, {:.4});\n",
            p_les4, h_les4s, h_les4, x_les4
        ));

        code.push_str("\t// 再热器抽汽\n");
        let (p_rh1_calc, x_rh1_calc, t_rh1_calc, h_rh1_calc, h_zs1_calc) =
            self.calc_rhx(p_hes7, x_hes7);
        code.push_str(&format!("\tlet (p_rh1_calc, x_rh1_calc, t_rh1_calc, h_rh1_calc, h_zs1_calc) = ({:.4}, {:.4}, {:.4}, {:.4}, {:.4});\n",
        p_rh1_calc, x_rh1_calc, t_rh1_calc, h_rh1_calc, h_zs1_calc));
        let (p_rh2_calc, x_rh2_calc, t_rh2_calc, h_rh2_calc, h_zs2_calc) =
            self.calc_rhx(p_hi, x_hi);
        code.push_str(&format!("\tlet (p_rh2_calc, x_rh2_calc, t_rh2_calc, h_rh2_calc, h_zs2_calc) = ({:.4}, {:.4}, {:.4}, {:.4}, {:.4});\n",
        p_rh2_calc, x_rh2_calc, t_rh2_calc, h_rh2_calc, h_zs2_calc));
        code.push_str("\t// 蒸汽发生器总蒸汽产量的计算 (迭代循环)\n");
        let h_a = h_hi - h_hz;
        code.push_str(&format!(
            "\tlet h_a = {:.4}; // 给水泵汽轮机中蒸汽的绝热焓降\n",
            h_a
        ));
        code.push_str(&format!(
            "\tlet mut mutable_params_g_cd = {:.4}; // 迭代变量，初始值为 g_cd\n",
            params.g_cd,
        ));
        code.push_str(&format!(
            "\tlet ne = {:.4}; // 迭代变量，初始值为 ne\n",
            params.ne,
        ));
        code.push_str(&format!(
            "\tlet mut mutable_params_ne_npp = {:.4}; // 迭代变量，初始值为 ne_npp\n",
            params.ne_npp,
        ));
        code.push_str("\tlet mut d_s_loop = 0.0;\n");
        code.push_str("\tlet mut g_fw_loop = 0.0;\n");
        code.push_str("\tlet mut q_r_loop = 0.0;\n");
        code.push_str(
            "\tlet mut g_sh_loop = 0.0; let mut g_sl_loop = 0.0; let mut g_fwps_loop = 0.0;\n",
        );
        code.push_str("\tlet mut g_les4_loop = 0.0; let mut g_les3_loop = 0.0; let mut g_les2_loop = 0.0; let mut g_les1_loop = 0.0;\n");
        code.push_str("\tlet mut g_zc1_loop = 0.0; let mut g_zc2_loop = 0.0; let mut g_hes6_loop = 0.0; let mut g_hes7_loop = 0.0;\n");
        code.push_str("\tlet mut g_uw_loop = 0.0; let mut g_sdea_loop = 0.0;\n");
        code.push_str("\tlet mut iteration_count = 0;\n\n");

        code.push_str("\tloop { // 外层循环: 迭代优化核电厂效率 (ne_npp)\n");
        code.push_str("\t\tq_r_loop = ne / mutable_params_ne_npp; // 反应堆热功率(MW)\n");
        code.push_str(&format!("\t\td_s_loop = (1000.0 * q_r_loop * {:.4}) / ((h_fh - h_s_calc) + (1.0 + {:.4}) * (h_s_calc - h_fw_calc)); // GS蒸汽产量(kg/s)\n", params.n_1, params.zeta_d));
        code.push_str(&format!(
            "\t\tg_fw_loop = (1.0 + {:.4}) * d_s_loop; // GS给水流量(kg/s)\n",
            params.zeta_d
        ));
        code.push_str("\t\tlet h_fwp_loop = p_fwpo - p_dea; // 给水泵扬程(MPa)\n");
        code.push_str(&format!(
            "\t\tlet rho_fwp_loop = 0.5 * {} + {}; // 给水泵中水的密度\n\n",
            px(p_dea, 0.0, OD),
            px(p_fwpo, 0.0, OD)
        ));

        code.push_str("\t\tloop { // 内层循环: 迭代优化冷凝器凝结水量 (g_cd)\n");
        code.push_str("\t\t\tlet n_fwpp_loop = 1000.0 * g_fw_loop * h_fwp_loop / rho_fwp_loop; // 给水泵有效输出功率(kW)\n");
        code.push_str(&format!("\t\t\tlet n_fwpt_loop = n_fwpp_loop / ({:.4} * {:.4} * {:.4} * {:.4}); // 给水泵理论功率(kW)\n", params.n_fwpp,
            params.n_fwpti, params.n_fwptg, params.n_fwptm));
        code.push_str(&format!(
            "\t\t\tg_fwps_loop = n_fwpt_loop / h_a; // 给水泵汽轮机耗汽量(kg/s)\n\n"
        ));

        code.push_str("\t\t\t// 低压给水加热器抽汽量\n");
        code.push_str(&format!("\t\t\tg_les4_loop = mutable_params_g_cd * (h_fw4o - h_fw4i) / ({:.4} * (h_les4 - h_ro4k));\n", params.n_h));
        code.push_str(&format!("\t\t\tg_les3_loop = (mutable_params_g_cd * (h_fw3o - h_fw3i) - {:.4} * g_les4_loop * (h_ro4k - h_ro3k)) / ({:.4} * (h_les3 - h_ro3k));\n", params.n_h, params.n_h));
        code.push_str(&format!("\t\t\tg_les2_loop = (mutable_params_g_cd * (h_fw2o - h_fw2i) - {:.4} * (g_les3_loop + g_les4_loop) * (h_ro3k - h_ro2k)) / ({:.4} * (h_les2 - h_ro2k));\n", params.n_h, params.n_h));
        code.push_str(&format!("\t\t\tg_les1_loop = (mutable_params_g_cd * (h_fw1o - h_fw1i) - {:.4} * (g_les2_loop + g_les3_loop + g_les4_loop) * (h_ro2k - h_ro1k)) / ({:.4} * (h_les1 - h_ro1k));\n\n", params.n_h, params.n_h));

        code.push_str("\t\t\t// 低压缸耗气量(kg/s)\n");
        code.push_str(&format!("\t\t\tg_sl_loop = (0.6 * 1000.0 * {:.4} / ({:.4} * {:.4}) + g_les4_loop * (h_les4 - h_lz) + g_les3_loop * (h_les3 - h_lz) + g_les2_loop * (h_les2 - h_lz) + g_les1_loop * (h_les1 - h_lz)) / (h_li - h_lz);\n\n",
            params.ne, params.n_m, params.n_ge));

        code.push_str("\t\t\t// 再热器加热蒸汽量\n");
        code.push_str(&format!(
            "\t\t\tg_zc1_loop = g_sl_loop * dh_rh / ({:.4} * (h_rh1_calc - h_zs1_calc));\n",
            params.n_h
        ));
        code.push_str(&format!(
            "\t\t\tg_zc2_loop = g_sl_loop * dh_rh / ({:.4} * (h_rh2_calc - h_zs2_calc));\n\n",
            params.n_h
        ));

        code.push_str("\t\t\t// 高压给水加热器抽汽量\n");
        code.push_str(&format!("\t\t\tg_hes7_loop = (g_fw_loop * (h_fw7o - h_fw7i) - {:.4} * g_zc2_loop * (h_zs2_calc - h_ro7k)) / ({:.4} * (h_hes7 - h_ro7k));\n", params.n_h, params.n_h));
        code.push_str(&format!("\t\t\tg_hes6_loop = (g_fw_loop * (h_fw6o - h_fw6i) - {:.4} * g_zc1_loop * (h_zs1_calc - h_ro6k) - {:.4} * (g_zc2_loop + g_hes7_loop) * (h_ro7k - h_ro6k)) / ({:.4} * (h_hes6 - h_ro6k));\n\n", params.n_h, params.n_h, params.n_h));

        code.push_str(&format!(
            "\t\t\tg_uw_loop = g_sl_loop * (x_rh1i - x_spi) / x_spi; // 汽水分离器疏水流量(kg/s)\n\n"
        ));

        code.push_str("\t\t\t// 除氧器耗汽量\n");
        code.push_str(&format!("\t\t\tg_sdea_loop = (g_fw_loop * h_deao - g_uw_loop * h_uw - mutable_params_g_cd * h_fw4o - (g_zc1_loop + g_zc2_loop + g_hes6_loop + g_hes7_loop) * h_ro6k) / h_hz;\n\n"));

        code.push_str("\t\t\t// 高压缸耗汽量\n");
        code.push_str(&format!("\t\t\tg_sh_loop = (0.4 * 1000.0 * ne / ({:.4} * {:.4}) + g_hes7_loop * (h_hes7 - h_hz) + g_hes6_loop * (h_hes6 - h_hz) + g_zc1_loop * (h_rh1_calc - h_hz)) / (h_hi - h_hz);\n\n", params.n_m, params.n_ge));

        code.push_str("\t\t\t// 对假设冷凝水流量验证\n");
        code.push_str("\t\t\td_s_loop = g_fwps_loop + g_zc2_loop + g_sh_loop; // 新蒸汽耗量 (根据新的流量重新评估 d_s_loop)\n");
        code.push_str(&format!(
            "\t\t\tlet g_fw1_loop = (1.0 + {:.4}) * d_s_loop; // 给水流量\n",
            params.zeta_d
        ));
        code.push_str(
            "\t\t\tlet g_cd1_loop = g_fw1_loop - g_sdea_loop - g_uw_loop - (g_hes6_loop + g_hes7_loop + g_zc1_loop + g_zc2_loop);\n",
        );

        code.push_str(
            "\t\t\tif ((g_cd1_loop - mutable_params_g_cd) as f64).abs() / mutable_params_g_cd < 1e-2 {\n",
        );
        code.push_str("\t\t\t\tbreak; // 内层循环中断\n");
        code.push_str("\t\t\t} else {\n");
        code.push_str("\t\t\t\tmutable_params_g_cd = g_cd1_loop;\n");
        code.push_str("\t\t\t\tg_fw_loop = g_fw1_loop; // 为下一次内层迭代更新 g_fw_loop\n");
        code.push_str("\t\t\t}\n");
        code.push_str("\t\t} // 内层循环结束\n\n");

        code.push_str(&format!("\t\tq_r_loop = (d_s_loop * (h_fh - h_fw_calc) + {:.4} * d_s_loop * (h_s_calc - h_fw_calc)) / (1000.0 * {:.4}); // 新反应堆热功率(MW)\n", params.zeta_d, params.n_1));
        code.push_str("\t\tlet n_ennp1_loop = ne / q_r_loop;\n\n");

        // 打印迭代结果 (CalcResult1)
        code.push_str("\t\titeration_count += 1;\n");
        code.push_str("\t\tprintln!(\"\\n--- Iteration: {} ---\", iteration_count);\n");
        code.push_str("\t\tprintln!(\"  核电厂热效率η-enpp: {:.4}\", mutable_params_ne_npp);\n");
        code.push_str("\t\tprintln!(\"  反应堆热功率Qʀ(MW): {:.4}\", q_r_loop);\n");
        code.push_str("\t\tprintln!(\"  新蒸汽流量D(kg/s): {:.4}\", d_s_loop);\n");
        code.push_str("\t\tprintln!(\"  高压缸耗汽量G-shp(kg/s): {:.4}\", g_sh_loop);\n");
        code.push_str("\t\tprintln!(\"  低压缸耗汽量G-slp(kg/s): {:.4}\", g_sl_loop);\n");
        code.push_str(
            "\t\tprintln!(\"  一级再热耗汽量G-srh1(kg/s) (g_zc1_loop): {:.4}\", g_zc1_loop);\n",
        );
        code.push_str(
            "\t\tprintln!(\"  二级再热耗汽量G-srh2(kg/s) (g_zc2_loop): {:.4}\", g_zc2_loop);\n",
        );
        code.push_str("\t\tprintln!(\"  除氧器耗汽量G-sdea(kg/s): {:.4}\", g_sdea_loop);\n");
        code.push_str("\t\tprintln!(\"  给水泵汽轮机耗汽量G-sfwp(kg/s): {:.4}\", g_fwps_loop);\n");
        code.push_str("\t\tprintln!(\"  总给水流量G-fw(kg/s): {:.4}\", g_fw_loop);\n");
        code.push_str("\t\tprintln!(\"  给水泵扬程Δh-fwp(MPa): {:.4}\", h_fwp_loop);\n");
        code.push_str("\t\tprintln!(\"  七级回热抽汽量G-hes7(kg/s): {:.4}\", g_hes7_loop);\n");
        code.push_str("\t\tprintln!(\"  六级回热抽汽量G-hes6(kg/s): {:.4}\", g_hes6_loop);\n");
        code.push_str("\t\tprintln!(\"  四级回热抽汽量G-les4(kg/s): {:.4}\", g_les4_loop);\n");
        code.push_str("\t\tprintln!(\"  三级回热抽汽量G-les3(kg/s): {:.4}\", g_les3_loop);\n");
        code.push_str("\t\tprintln!(\"  二级回热抽汽量G-les2(kg/s): {:.4}\", g_les2_loop);\n");
        code.push_str("\t\tprintln!(\"  一级回热抽汽量G-les1(kg/s): {:.4}\", g_les1_loop);\n");
        code.push_str(
            "\t\tprintln!(\"  冷凝器凝结水量G-cd(kg/s): {:.4}\", mutable_params_g_cd);\n",
        );
        code.push_str("\t\tprintln!(\"  汽水分离器疏水量G-uw(kg/s): {:.4}\", g_uw_loop);\n");

        code.push_str("\t\tif ((n_ennp1_loop - mutable_params_ne_npp) as f64).abs() < 1e-3 {\n");
        code.push_str("\t\t\tbreak; // 外层循环中断\n");
        code.push_str("\t\t} else {\n");
        code.push_str("\t\t\tmutable_params_ne_npp = n_ennp1_loop;\n");
        code.push_str("\t\t}\n");
        code.push_str("\t} // 外层循环结束\n\n");

        // 打印最终结果 (CalcResult2)
        code.push_str("\tprintln!(\"\\n--- Final Calculation Results ---\");\n");

        code.push_str("\tprintln!(\"\\n--- 附表一 (部分为输入参数) ---\");\n");
        code.push_str("\tprintln!(\"  1.核电厂输出功率N-e(MW): {:.4}\", ne);\n");
        code.push_str(&format!(
            "\tprintln!(\"  2.一回路能量利用系数η-1: {:.4}\");\n",
            params.n_1
        ));
        code.push_str(&format!(
            "\tprintln!(\"  3.蒸汽发生器出口蒸汽干度X-fh: {:.4}\");\n",
            params.x_fh
        ));
        code.push_str(&format!(
            "\tprintln!(\"  4.蒸汽发生器排污率ξ-d: {:.4}\");\n",
            params.zeta_d
        ));
        code.push_str(&format!(
            "\tprintln!(\"  5.高压缸内效率η-hi: {:.4}\");\n",
            params.n_hi
        ));
        code.push_str(&format!(
            "\tprintln!(\"  6.低压缸内效率η-li: {:.4}\");\n",
            params.n_li
        ));
        code.push_str(&format!(
            "\tprintln!(\"  7.汽轮机组机械效率η-m: {:.4}\");\n",
            params.n_m
        ));
        code.push_str(&format!(
            "\tprintln!(\"  8.发电机效率η-ge: {:.4}\");\n",
            params.n_ge
        ));
        code.push_str("\tprintln!(\"  9.新蒸汽压损Δp-fh(MPa): {:.4}\", dp_fh_calc);\n");
        code.push_str("\tprintln!(\"  10.再热蒸汽压损Δp-rh (MPa): {:.4}\", dp_rh_calc);\n");
        code.push_str(&format!(
            "\tprintln!(\"  11.回热抽汽压损Δp-ej (%): {:.4}\");\n",
            params.dp_ej
        ));
        code.push_str("\tprintln!(\"  12.低压缸排气压损Δp-cd (MPa): {:.4}\", dp_cd_calc);\n");
        code.push_str(&format!(
            "\tprintln!(\"  13.高压给水加热器出口端差θ-hu(°C): {:.4}\");\n",
            params.theta_hu
        ));
        code.push_str(&format!(
            "\tprintln!(\"  14.低压给水加热器出口端差θ-lu(°C): {:.4}\");\n",
            params.theta_lu
        ));
        code.push_str(&format!(
            "\tprintln!(\"  15.加热器效率η-h: {:.4}\");\n",
            params.n_h
        ));
        code.push_str(&format!(
            "\tprintln!(\"  16.给水泵效率η-fwpp: {:.4}\");\n",
            params.n_fwpp
        ));
        code.push_str(&format!(
            "\tprintln!(\"  17.给水泵汽轮机内效率η-fwpti: {:.4}\");\n",
            params.n_fwpti
        ));
        code.push_str(&format!(
            "\tprintln!(\"  18.给水泵汽轮机机械效率η-fwptm: {:.4}\");\n",
            params.n_fwptm
        ));
        code.push_str(&format!(
            "\tprintln!(\"  19.给水泵汽轮机减速器效率η-fwptg: {:.4}\");\n",
            params.n_fwptg
        ));
        code.push_str(&format!(
            "\tprintln!(\"  20.循环冷却水进口温度T-sw1(°C): {:.4}\");\n",
            params.t_sw1
        ));

        code.push_str("\tprintln!(\"\\n--- 附表二 (计算结果) ---\");\n");
        code.push_str(&format!(
            "\tprintln!(\"  1.反应堆冷却剂系统运行压力p-c(MPa): {:.4}\");\n",
            params.p_c
        ));
        code.push_str("\tprintln!(\"  2.冷却剂压力对应的饱和温度T-cs(°C): {:.4}\", t_cs);\n");
        code.push_str(&format!(
            "\tprintln!(\"  3.反应堆出口冷却剂过冷度ΔT-sub(°C): {:.4}\");\n",
            params.dt_sub
        ));
        code.push_str("\tprintln!(\"  4.反应堆出口冷却剂温度T-co(°C): {:.4}\", t_co);\n");
        code.push_str(&format!(
            "\tprintln!(\"  5.反应堆进出口冷却剂温升ΔT-c(°C): {:.4}\");\n",
            params.dt_c
        ));
        code.push_str("\tprintln!(\"  6.反应堆进口冷却剂温度T-ci(°C): {:.4}\", t_ci);\n");
        code.push_str(&format!(
            "\tprintln!(\"  7.蒸汽发生器饱和蒸汽压力p-s(MPa): {:.4}\");\n",
            params.p_s
        ));
        code.push_str("\tprintln!(\"  8.蒸汽发生器饱和蒸汽温度T-fh(°C): {:.4}\", t_fh);\n");
        code.push_str("\tprintln!(\"  9.一、二次侧对数平均温差ΔT-m(°C): {:.4}\", dt_m);\n");
        code.push_str(&format!(
            "\tprintln!(\"  10.冷凝器中循环冷却水温升ΔT-sw(°C): {:.4}\");\n",
            params.dt_sw
        ));
        code.push_str(&format!(
            "\tprintln!(\"  11.冷凝器传热端差δT(°C): {:.4}\");\n",
            params.dt
        ));
        code.push_str("\tprintln!(\"  12.冷凝器凝结水饱和温度T-cd(°C): {:.4}\", t_cd);\n");
        code.push_str("\tprintln!(\"  13.冷凝器的运行压力p-cd(MPa): {:.4}\", p_cd);\n");
        code.push_str("\tprintln!(\"  14.高压缸进口的蒸汽压力p-hi(MPa): {:.4}\", p_hi);\n");
        code.push_str("\tprintln!(\"  15.高压缸进口蒸汽干度X-hi: {:.4}\", x_hi);\n");
        code.push_str("\tprintln!(\"  15.1.蒸汽发生器出口蒸汽比焓h-fh(kJ/kg): {:.4}\", h_fh);\n");
        code.push_str("\tprintln!(\"  15.2.蒸汽发生器出口蒸汽比熵s-fh: {:.4}\", s_fh);\n");
        code.push_str("\tprintln!(\"  15.3.高压缸进口蒸汽比熵s-hi: {:.4}\", s_hi);\n");
        code.push_str("\tprintln!(\"  16.高压缸排气压力p-hz(MPa): {:.4}\", p_hz);\n");
        code.push_str("\tprintln!(\"  17.高压缸排气干度X-hz: {:.4}\", x_hz);\n");
        code.push_str("\tprintln!(\"  17.1.高压缸进口蒸汽比焓h-hi(kJ/kg): {:.4}\", h_hi);\n");
        code.push_str("\tprintln!(\"  17.2.高压缸出口理想比焓h-hzs(kJ/kg): {:.4}\", h_hzs);\n");
        code.push_str("\tprintln!(\"  17.3.高压缸出口蒸汽比焓h-hz(kJ/kg): {:.4}\", h_hz);\n");
        code.push_str("\tprintln!(\"  18.汽水分离器进口蒸汽压力p-spi(MPa): {:.4}\", p_spi);\n");
        code.push_str("\tprintln!(\"  19.汽水分离器进口蒸汽干度X-spi: {:.4}\", x_spi);\n");
        code.push_str("\tprintln!(\"  19.1.汽水分离器出口疏水压力p-uw(MPa): {:.4}\", p_uw);\n");
        code.push_str("\tprintln!(\"  19.2.汽水分离器出口疏水比焓h-uw(kJ/kg): {:.4}\", h_uw);\n");
        code.push_str("\tprintln!(\"  20.再热蒸汽进口压力p-rh1i(MPa): {:.4}\", p_rh1i);\n");
        code.push_str("\tprintln!(\"  21.再热蒸汽进口干度X-rh1i: {:.4}\", x_rh1i);\n");
        code.push_str(
            "\tprintln!(\"  21.1.一级再热器进口蒸汽比焓h-rh1i(kJ/kg): {:.4}\", h_rh1i);\n",
        );

        code.push_str("\tprintln!(\"\\n--- 附表二 (Result2) - 续 ---\");\n");
        code.push_str(
            "\tprintln!(\"  22.加热蒸汽进口压力p-rh1hs(MPa) (p_rh1_calc): {:.4}\", p_rh1_calc);\n",
        );
        code.push_str(
            "\tprintln!(\"  23.加热蒸汽进口干度X-rh1hs (x_rh1_calc): {:.4}\", x_rh1_calc);\n",
        );
        code.push_str("\tprintln!(\"  24.再热蒸汽进口压力p-rh2i(MPa): {:.4}\", p_rh2i);\n");
        code.push_str("\tprintln!(\"  25.再热蒸汽进口温度T-rh2i(°C): {:.4}\", t_rh2i);\n");
        code.push_str("\tprintln!(\"  26.再热蒸汽出口压力p-rh2z(MPa): {:.4}\", p_rh2z);\n");
        code.push_str("\tprintln!(\"  27.再热蒸汽出口温度T-rh2z(°C): {:.4}\", t_rh2z);\n");
        code.push_str("\tprintln!(\"  27.1.二级再热器出口比焓h-rh2z(kJ/kg): {:.4}\", h_rh2z);\n");
        code.push_str("\tprintln!(\"  27.2.每级再热器平均焓升Δh-rh(kJ/kg): {:.4}\", dh_rh);\n");
        code.push_str(
            "\tprintln!(\"  27.3.一级再热器出口蒸汽比焓h-rh1z(kJ/kg): {:.4}\", h_rh1z);\n",
        );
        code.push_str("\tprintln!(\"  27.4.二级再热器进口蒸汽比焓h-rh2i(kJ/kg) (h_rh2i_calc): {:.4}\", h_rh2_calc);\n");
        code.push_str(
            "\tprintln!(\"  28.加热蒸汽进口压力p-rh2hs(MPa) (p_rh2_calc): {:.4}\", p_rh2_calc);\n",
        );
        code.push_str(
            "\tprintln!(\"  29.加热蒸汽进口干度X-rh2hs (x_rh2_calc): {:.4}\", x_rh2_calc);\n",
        );
        code.push_str("\tprintln!(\"  30.进口蒸汽压力p-li(MPa): {:.4}\", p_li);\n");
        code.push_str("\tprintln!(\"  31.进口蒸汽温度T-li(°C): {:.4}\", t_li);\n");
        code.push_str("\tprintln!(\"  32.排汽压力p-lz(MPa): {:.4}\", p_lz);\n");
        code.push_str("\tprintln!(\"  33.排汽干度X-lz: {:.4}\", x_lz);\n");
        code.push_str("\tprintln!(\"  33.1.低压缸进口蒸汽比熵s-li: {:.4}\", s_li);\n");
        code.push_str("\tprintln!(\"  33.2.低压缸进口蒸汽比焓h-li(kJ/kg): {:.4}\", h_li);\n");
        code.push_str("\tprintln!(\"  33.3.低压缸出口理想比焓h-lzs(kJ/kg): {:.4}\", h_lzs);\n");
        code.push_str("\tprintln!(\"  33.4.低压缸出口蒸汽比焓h-lz(kJ/kg): {:.4}\", h_lz);\n");
        code.push_str(&format!(
            "\tprintln!(\"  34.回热级数Z: {:.4}\");\n",
            params.z
        ));
        code.push_str(&format!(
            "\tprintln!(\"  35.低压给水加热器级数Z-l: {:.4}\");\n",
            params.z_l
        ));
        code.push_str(&format!(
            "\tprintln!(\"  36.高压给水加热器级数Z-h: {:.4}\");\n",
            params.z_h
        ));
        code.push_str("\tprintln!(\"  37.第一次给水回热分配Δh-fw: {:.4}\", dh_fw_calc);\n");
        code.push_str(
            "\tprintln!(\"  37.1.蒸汽发生器运行压力饱和水比焓h-s(kJ/kg): {:.4}\", h_s_calc);\n",
        );
        code.push_str("\tprintln!(\"  37.2.冷凝器出口凝结水比焓h-cd(kJ/kg): {:.4}\", h_cd_val);\n");
        code.push_str(
            "\tprintln!(\"  37.3.每级加热器理论给水焓升Δh-fwop(kJ/kg): {:.4}\", dh_fwop);\n",
        );
        code.push_str("\tprintln!(\"  37.4.最佳给水比焓h-fwop(kJ/kg): {:.4}\", h_fwop);\n");
        code.push_str("\tprintln!(\"  37.5.最佳给水温度T-fwop(°C): {:.4}\", t_fwop);\n");
        code.push_str("\tprintln!(\"  37.6.实际给水温度T-fw(°C): {:.4}\", t_fw_calc);\n");
        code.push_str("\tprintln!(\"  37.7.实际给水比焓h-fw(kJ/kg): {:.4}\", h_fw_calc);\n");

        code.push_str("\tprintln!(\"\\n--- 给水及除氧器参数 ---\");\n");
        code.push_str("\tprintln!(\"  38.高压加热器给水焓升Δh-fwh(kJ/kg): {:.4}\", dh_fwh);\n");
        code.push_str("\tprintln!(\"  38.1.除氧器运行压力p-dea(MPa): {:.4}\", p_dea);\n");
        code.push_str("\tprintln!(\"  38.2.除氧器出口饱和水比焓h-deao(kJ/kg): {:.4}\", h_deao);\n");
        code.push_str(
            "\tprintln!(\"  39.除氧器及低压加热器给水焓升Δh-fwl(kJ/kg): {:.4}\", dh_fwl);\n",
        );
        code.push_str("\tprintln!(\"  39.1.凝水泵出口给水压力p-cwp(MPa): {:.4}\", p_cwp);\n");
        code.push_str("\tprintln!(\"  39.2.凝水泵出口给水比焓h-cwp(kJ/kg): {:.4}\", h_cwp);\n");
        code.push_str(
            "\tprintln!(\"  39.3.凝水泵出口至除氧器出口阻力压降Δp-cws(MPa): {:.4}\", dp_cws);\n",
        );
        code.push_str(
            "\tprintln!(\"  39.4.每级低压加热器及除氧器阻力压降Δp-fi(MPa): {:.4}\", dp_fi);\n",
        );

        code.push_str("\tprintln!(\"\\n  --- 低压给水加热器 (lfwx) ---\");\n");
        code.push_str("\tlet lfwx_data = [\n");
        code.push_str(
            "\t\t(1, p_fw1i, h_fw1i, t_fw1i, p_fw1o, h_fw1o, t_fw1o, t_ro1k, h_ro1k, p_ro1k),\n",
        );
        code.push_str(
            "\t\t(2, p_fw2i, h_fw2i, t_fw2i, p_fw2o, h_fw2o, t_fw2o, t_ro2k, h_ro2k, p_ro2k),\n",
        );
        code.push_str(
            "\t\t(3, p_fw3i, h_fw3i, t_fw3i, p_fw3o, h_fw3o, t_fw3o, t_ro3k, h_ro3k, p_ro3k),\n",
        );
        code.push_str(
            "\t\t(4, p_fw4i, h_fw4i, t_fw4i, p_fw4o, h_fw4o, t_fw4o, t_ro4k, h_ro4k, p_ro4k),\n",
        );
        code.push_str("\t];\n");
        code.push_str(
            "\tfor (level, p_i, h_i, t_i, p_o, h_o, t_o, t_r, h_r, _p_r) in lfwx_data.iter() {\n",
        );
        code.push_str("\t\tprintln!(\"    --- {}级低压加热器给水参数 ---\", level);\n");
        code.push_str("\t\tprintln!(\"      进口给水压力p-fwxi(MPa): {:.4}\", p_i);\n");
        code.push_str("\t\tprintln!(\"      进口给水比焓h-fwxi(kJ/kg): {:.4}\", h_i);\n");
        code.push_str("\t\tprintln!(\"      进口给水温度T-fwxi(°C): {:.4}\", t_i);\n");
        code.push_str("\t\tprintln!(\"      出口给水压力p-fwxo(MPa): {:.4}\", p_o);\n");
        code.push_str("\t\tprintln!(\"      出口给水比焓h-fwxo(kJ/kg): {:.4}\", h_o);\n");
        code.push_str("\t\tprintln!(\"      出口给水温度T-fwxo(°C): {:.4}\", t_o);\n");
        code.push_str("\t\tprintln!(\"      疏水温度T-roxk(°C): {:.4}\", t_r);\n");
        code.push_str("\t\tprintln!(\"      疏水比焓h-roxk(kJ/kg): {:.4}\", h_r);\n");
        code.push_str("\t}\n");

        code.push_str("\tprintln!(\"  41.进口给水比焓h-deai(kJ/kg) (除氧器): {:.4}\", h_deai);\n");
        code.push_str(
            "\tprintln!(\"  42.出口给水比焓h-deao(kJ/kg) (除氧器, h_deao): {:.4}\", h_deao);\n",
        );
        code.push_str(
            "\tprintln!(\"  43.出口给水温度T-dea(°C) (除氧器, t_deao): {:.4}\", t_deao);\n",
        );
        code.push_str("\tprintln!(\"  44.运行压力p-dea(MPa) (除氧器, p_dea): {:.4}\", p_dea);\n");
        code.push_str("\tprintln!(\"  44.1.给水泵出口压力p-fwpo(MPa): {:.4}\", p_fwpo);\n");
        code.push_str(
            "\tprintln!(\"  44.2.给水泵出口流体比焓h-fwpo(kJ/kg): {:.4}\", h_fwpo_calc);\n",
        );
        code.push_str("\tprintln!(\"  44.3.蒸汽发生器进口给水压力p-fwi(MPa): {:.4}\", p_fwi);\n");

        code.push_str("\tprintln!(\"\\n  --- 高压给水加热器 (hfwx) ---\");\n");
        code.push_str("\tlet hfwx_data = [\n");
        code.push_str(
            "\t\t(6, p_fw6i, h_fw6i, t_fw6i, p_fw6o, h_fw6o, t_fw6o, t_ro6k, h_ro6k, p_ro6k),\n",
        );
        code.push_str(
            "\t\t(7, p_fw7i, h_fw7i, t_fw7i, p_fw7o, h_fw7o, t_fw7o, t_ro7k, h_ro7k, p_ro7k)\n",
        );
        code.push_str("\t];\n");
        code.push_str(
            "\tfor (level, p_i, h_i, t_i, p_o, h_o, t_o, t_r, h_r, _p_r) in hfwx_data.iter() {\n",
        );
        code.push_str("\t\tprintln!(\"    --- {}级高压加热器给水参数 ---\", level);\n");
        code.push_str("\t\tprintln!(\"      进口给水压力p-fwxi(MPa): {:.4}\", p_i);\n");
        code.push_str("\t\tprintln!(\"      进口给水比焓h-fwxi(kJ/kg): {:.4}\", h_i);\n");
        code.push_str("\t\tprintln!(\"      进口给水温度T-fwxi(°C): {:.4}\", t_i);\n");
        code.push_str("\t\tprintln!(\"      出口给水压力p-fwxo(MPa): {:.4}\", p_o);\n");
        code.push_str("\t\tprintln!(\"      出口给水比焓h-fwxo(kJ/kg): {:.4}\", h_o);\n");
        code.push_str("\t\tprintln!(\"      出口给水温度T-fwxo(°C): {:.4}\", t_o);\n");
        code.push_str("\t\tprintln!(\"      疏水温度T-roxk(°C): {:.4}\", t_r);\n");
        code.push_str("\t\tprintln!(\"      疏水比焓h-roxk(kJ/kg): {:.4}\", h_r);\n");
        code.push_str("\t}\n");

        code.push_str("\tprintln!(\"\\n--- 46.高压缸抽气 ---\");\n");
        code.push_str("\tprintln!(\"  46.1.高压缸进口蒸汽比熵s-hi: {:.4}\", s_hi);\n");
        code.push_str("\tprintln!(\"  46.2.高压缸进口蒸汽比焓h-hi(kJ/kg): {:.4}\", h_hi);\n");
        code.push_str("\tlet hhes_data = [\n");
        code.push_str("\t\t(6, p_hes6, h_hes6s, h_hes6, x_hes6),\n");
        code.push_str("\t\t(7, p_hes7, h_hes7s, h_hes7, x_hes7),\n");
        code.push_str("\t];\n");
        code.push_str("\tfor (level, p_ex, h_exs, h_ex, x_ex) in hhes_data.iter() {\n");
        code.push_str("\t\tprintln!(\"    --- {}号高压抽汽参数 ---\", level);\n");
        code.push_str("\t\tprintln!(\"      抽汽压力p-hesx(MPa): {:.4}\", p_ex);\n");
        code.push_str("\t\tprintln!(\"      抽汽干度X-hesx: {:.4}\", x_ex);\n");
        code.push_str("\t\tprintln!(\"      抽汽理想比焓h-hesxs(kJ/kg): {:.4}\", h_exs);\n");
        code.push_str("\t\tprintln!(\"      抽汽比焓h-hesx(kJ/kg): {:.4}\", h_ex);\n");
        code.push_str("\t}\n");

        code.push_str("\tprintln!(\"\\n--- 47.低压缸抽气 ---\");\n");
        code.push_str("\tprintln!(\"  47.1.低压缸进口蒸汽比熵s-li: {:.4}\", s_li);\n");
        code.push_str("\tprintln!(\"  47.2.低压缸进口蒸汽比焓h-li(kJ/kg): {:.4}\", h_li);\n");
        code.push_str("\tlet lhes_data = [\n");
        code.push_str("\t\t(1, p_les1, h_les1s, h_les1, x_les1),\n");
        code.push_str("\t\t(2, p_les2, h_les2s, h_les2, x_les2),\n");
        code.push_str("\t\t(3, p_les3, h_les3s, h_les3, x_les3),\n");
        code.push_str("\t\t(4, p_les4, h_les4s, h_les4, x_les4),\n");
        code.push_str("\t];\n");
        code.push_str("\tfor (level, p_ex, h_exs, h_ex, x_ex) in lhes_data.iter() {\n");
        code.push_str("\t\tprintln!(\"    --- {}号低压抽汽参数 ---\", level);\n");
        code.push_str("\t\tprintln!(\"      抽汽压力p-lesx(MPa): {:.4}\", p_ex);\n");
        code.push_str("\t\tprintln!(\"      抽汽干度X-lesx: {:.4}\", x_ex);\n");
        code.push_str("\t\tprintln!(\"      抽汽理想比焓h-lesxs(kJ/kg): {:.4}\", h_exs);\n");
        code.push_str("\t\tprintln!(\"      抽汽比焓h-lesx(kJ/kg): {:.4}\", h_ex);\n");
        code.push_str("\t}\n");

        code.push_str("\tprintln!(\"\\n--- 48.再热器抽气 ---\");\n");
        code.push_str("\tlet rhx_data = [\n");
        code.push_str("\t\t(1, p_rh1_calc, x_rh1_calc, t_rh1_calc, h_rh1_calc, h_zs1_calc),\n");
        code.push_str("\t\t(2, p_rh2_calc, x_rh2_calc, t_rh2_calc, h_rh2_calc, h_zs2_calc),\n");
        code.push_str("\t];\n");
        code.push_str(
            "\tfor (level, p_rhx_v, x_rhx_v, t_rhx_v, h_rhx_v, h_zsx_v) in rhx_data.iter() {\n",
        );
        code.push_str("\t\tprintln!(\"    --- {}级再热器参数 ---\", level);\n");
        code.push_str("\t\tprintln!(\"      加热蒸汽进口压力p-rhx(MPa): {:.4}\", p_rhx_v);\n");
        code.push_str("\t\tprintln!(\"      加热蒸汽进口干度X-rhx: {:.4}\", x_rhx_v);\n");
        code.push_str("\t\tprintln!(\"      加热蒸汽进口温度T-rhx(°C): {:.4}\", t_rhx_v);\n");
        code.push_str("\t\tprintln!(\"      加热蒸汽进口比焓h-rhx(kJ/kg): {:.4}\", h_rhx_v);\n");
        code.push_str("\t\tprintln!(\"      疏水比焓h-zsx(kJ/kg): {:.4}\", h_zsx_v);\n");
        code.push_str("\t}\n");

        code.push_str("\n// --- 计算结束 ---\n\n");
        code.push_str("}\n");

        code
    }

    pub fn generate_calc_code_py(&self) -> String {
        let mut code = String::new();
        // 使用 self.params 来获取输入参数的实际值
        let params = &self.params;

        code.push_str("#!/usr/bin/env python3\n");
        code.push_str("# -*- coding: utf-8 -*-\n");
        code.push_str("# @Time    : 2025/05/14 19:00\n");
        code.push_str("# @Author  : hanasaki\n");
        code.push_str("# @Email   : hanasakayui2022@gmail.com\n");
        code.push_str("# @File    : calc.py\n");
        code.push_str("# @Software: Zed\n");
        code.push_str("# @Description: 核电厂热力计算程序\n");

        code.push_str("def main ():\n");
        code.push_str("# 根据 Calculator::calculate 方法生成的计算过程代码\n");

        code.push_str("\t# --- 计算开始 ---\n\n");

        code.push_str("\t# 一回路冷却剂参数\n");
        let t_cs = px(params.p_c, 0.0, OT);
        code.push_str(&format!("\tt_cs = {:.4} # 工作压力对应饱和温度(℃)\n", t_cs));
        let t_co = t_cs - params.dt_sub;
        code.push_str(&format!("\tt_co = {:.4} # 反应堆出口冷却剂温度(℃)\n", t_co));
        let t_ci = t_co - params.dt_c;
        code.push_str(&format!(
            "\tt_ci = {:.4} # 反应堆进口冷却剂温度(℃)\n\n",
            t_ci
        ));

        code.push_str("\t# 蒸汽初参数\n");
        let t_s = px(params.p_s, 1.0, OT);
        code.push_str(&format!("\tt_s = {:.4} # 对应的饱和温度(℃)\n", t_s));
        let t_fh = px(params.p_s, params.x_fh, OT);
        code.push_str(&format!("\tt_fh = {:.4} # 新蒸汽温度(℃)\n", t_fh));
        let h_fh = tx(t_fh, params.x_fh, OH);
        code.push_str(&format!("\th_fh = {:.4} # 新蒸汽比焓(℃)\n", h_fh));
        let s_fh = px(params.p_s, params.x_fh, OS);
        code.push_str(&format!("\ts_fh = {:.4} # 新蒸汽比熵(kJ/(kg * K))\n", s_fh));
        let dt_m = (t_co - t_ci) / f64::ln((t_co - t_s) / (t_ci - t_s));
        code.push_str(&format!("\tdt_m = {:.4} # 对数平均传热温差(℃)\n\n", dt_m));

        code.push_str("\t# 蒸汽终参数\n");
        let t_cd = params.t_sw1 + params.dt_sw + params.dt;
        code.push_str(&format!("\tt_cd = {:.4} # 冷凝器凝结水饱和温度(℃)\n", t_cd));
        let p_cd = tx(t_cd, 0.0, OP);
        code.push_str(&format!("\tp_cd = {:.4} # 凝结水压力(MPa)\n\n", p_cd));

        code.push_str("\t# 高压缸参数\n");
        let dp_fh_calc = params.dp_fh * params.p_s;
        code.push_str(&format!(
            "\tdp_fh_calc = {:.4} # 新蒸汽压损(MPa)\n",
            dp_fh_calc
        ));
        let p_hi = params.p_s - dp_fh_calc;
        code.push_str(&format!("\tp_hi = {:.4} # 高压缸进口蒸汽压力(MPa)\n", p_hi));
        let h_hi = px(p_hi, 1.0, OH);
        code.push_str(&format!(
            "\th_hi = {:.4} # 高压缸进口蒸汽比焓(kJ/kg)\n",
            h_hi
        ));
        let x_hi = ph(p_hi, h_hi, OX);
        code.push_str(&format!("\tx_hi = {:.4} # 进口蒸汽干度(%)\n", x_hi));
        let s_hi = ph(p_hi, h_hi, OS);
        code.push_str(&format!(
            "\ts_hi = {:.4} # 进口蒸汽比熵(kJ/(kg * K))\n",
            s_hi
        ));
        let p_hz = params.dp_hz * p_hi;
        code.push_str(&format!("\tp_hz = {:.4} # 排气压力(MPa)\n", p_hz));
        let h_hzs = ps(p_hz, s_hi, OH);
        code.push_str(&format!(
            "\th_hzs = {:.4} # 高压缸排气理想比焓(kJ/kg)\n",
            h_hzs
        ));
        let h_hz = h_hi - params.n_hi * (h_hi - h_hzs);
        code.push_str(&format!(
            "\th_hz = {:.4} # 高压缸排气实际比焓(kJ/kg)\n",
            h_hz
        ));
        let x_hz = ph(p_hz, h_hz, OX);
        code.push_str(&format!("\tx_hz = {:.4} # 排气干度 (%)\n\n", x_hz));

        code.push_str("\t# 蒸汽中间再热参数\n");
        let dp_rh_calc = params.dp_rh * p_hz;
        code.push_str(&format!(
            "\tdp_rh_calc = {:.4} # 再热蒸汽压损 (MPa)\n",
            dp_rh_calc
        ));
        let p_spi = p_hz;
        code.push_str(&format!(
            "\tp_spi = {:.4} # 汽水分离器进口蒸汽压力 (MPa)\n",
            p_spi
        ));
        let x_spi = x_hz;
        code.push_str(&format!(
            "\tx_spi = {:.4} # 汽水分离器进口蒸汽干度 (%))\n",
            x_spi
        ));
        let _h_spi = px(p_hz, 0.0, OH);
        code.push_str(&format!(
            "\t_h_spi = {:.4} # 汽水分离器入口焓值 (kJ/kg)\n",
            _h_spi
        ));
        let p_uw = 0.99 * p_hz;
        code.push_str(&format!(
            "\tp_uw = {:.4} # 汽水分离器出口疏水压力 (MPa)\n",
            p_uw
        ));
        let h_uw = px(p_uw, 0.0, OH);
        code.push_str(&format!(
            "\th_uw = {:.4} # 汽水分离器出口疏水比焓 (kJ/kg)\n\n",
            h_uw
        ));

        code.push_str("\t# 一级再热器\n");
        let p_rh1i = 0.99 * p_hz;
        code.push_str(&format!(
            "\tp_rh1i = {:.4} # 一级再热器进口蒸汽压力 (MPa)\n",
            p_rh1i
        ));
        let x_rh1i = x_spi / (1.0 - 0.98 * (1.0 - x_spi));
        code.push_str(&format!(
            "\tx_rh1i = {:.4} # 一级再热器进口蒸汽干度 (%)\n",
            x_rh1i
        ));
        let h_rh1i = px(p_rh1i, x_rh1i, OH);
        code.push_str(&format!(
            "\th_rh1i = {:.4} # 一级再热器进口蒸汽比焓 (kJ/kg)\n\n",
            h_rh1i
        ));

        code.push_str("\t# 二级再热器\n");
        let p_rh2i = 0.98 * p_hz;
        code.push_str(&format!(
            "\tp_rh2i = {:.4} # 二级再热器进口蒸汽压力 (MPa)\n",
            p_rh2i
        ));
        let p_rh2z = 0.97 * p_hz;
        code.push_str(&format!(
            "\tp_rh2z = {:.4} # 二级再热器出口压力 (MPa)\n",
            p_rh2z
        ));
        let t_rh2z = t_fh - params.t_rh2z;
        code.push_str(&format!(
            "\tt_rh2z = {:.4} # 二级再热器出口温度 (℃)\n",
            t_rh2z
        ));
        let h_rh2z = pt(p_rh2z, t_rh2z, OH);
        code.push_str(&format!(
            "\th_rh2z = {:.4} # 二级再热器出口蒸汽比焓 (kJ/kg)\n",
            h_rh2z
        ));
        let dh_rh = (h_rh2z - h_rh1i) / 2.0;
        code.push_str(&format!(
            "\tdh_rh = {:.4} # 每级再热器平均焓升 (kJ/kg)\n",
            dh_rh
        ));
        let h_rh1z = h_rh1i + dh_rh;
        code.push_str(&format!(
            "\th_rh1z = {:.4} # 一级再热器出口蒸汽比焓 (kJ/kg)\n",
            h_rh1z
        ));
        let h_rh2i = h_rh1z;
        code.push_str(&format!(
            "\th_rh2i = {:.4} # 二级再热器进口蒸汽比焓 (kJ/kg)\n",
            h_rh2i
        ));
        let t_rh2i = ph(p_rh2i, h_rh2i, OT);
        code.push_str(&format!(
            "\tt_rh2i = {:.4} # 二级再热器进口蒸汽温度 (℃)\n",
            t_rh2i
        ));
        let p_rh2hs = p_hi;
        code.push_str(&format!(
            "\tp_rh2hs = {:.4} # 加热(新)蒸汽进口压力 (MPa)\n",
            p_rh2hs
        ));
        let x_rh2hs = x_hi;
        code.push_str(&format!(
            "\tx_rh2hs = {:.4} # 加热(新)蒸汽进口干度 (%)\n\n",
            x_rh2hs
        ));

        code.push_str("\t# 低压缸参数\n");
        let p_li = (1.0 - params.dp_f) * p_rh2z;
        code.push_str(&format!("\tp_li = {:.4} # 低压缸进气压力 (MPa)\n", p_li));
        let h_li = h_rh2z;
        code.push_str(&format!(
            "\th_li = {:.4} # 低压缸进口进气比焓 (kJ/kg)\n",
            h_li
        ));
        let t_li = ph(p_li, h_li, OT);
        code.push_str(&format!("\tt_li = {:.4} # 进口蒸汽温度 ()\n", t_li));
        let dp_cd_calc = (1.0 / (1.0 - params.dp_cd) - 1.0) * p_cd;
        code.push_str(&format!(
            "\tdp_cd_calc = {:.4} # 低压缸排气压损 (MPa)\n",
            dp_cd_calc
        ));
        let p_lz = p_cd + dp_cd_calc;
        code.push_str(&format!("\tp_lz = {:.4} # 低压缸排气压力 (MPa)\n", p_lz));
        let s_li = ph(p_li, h_li, OS);
        code.push_str(&format!("\ts_li = {:.4} # 进口蒸汽比熵 ()\n", s_li));
        let s_lz = s_li;
        code.push_str(&format!("\ts_lz = {:.4} # 定熵过程，排气比熵 ()\n", s_lz));
        let h_lzs = ps(p_lz, s_lz, OH);
        code.push_str(&format!("\th_lzs = {:.4} # 低压缸排气理想比焓 ()\n", h_lzs));
        let h_lz = h_li - params.n_li * (h_li - h_lzs);
        code.push_str(&format!("\th_lz = {:.4} # 排气实际比焓 ()\n", h_lz));
        let x_lz = ph(p_lz, h_lz, OX);
        code.push_str(&format!("\tx_lz = {:.4} # 排气干度 ()\n\n", x_lz));

        code.push_str("\t# 给水的焓升分配\n");
        let h_s_calc = px(params.p_s, 0.0, OH);
        code.push_str(&format!(
            "\th_s_calc = {:.4} # GS工作压力下的饱和水焓 ()\n",
            h_s_calc
        ));
        let h_cd_val = tx(t_cd, 0.0, OH);
        code.push_str(&format!(
            "\th_cd_val = {:.4} # 冷凝器出口凝结水比焓 ()\n",
            h_cd_val
        ));
        let dh_fwop = (h_s_calc - h_cd_val) / (params.z + 1.0);
        code.push_str(&format!("\tdh_fwop = {:.4} # 理论给水焓升 ()\n", dh_fwop));
        let h_fwop = h_cd_val + params.z * dh_fwop;
        code.push_str(&format!("\th_fwop = {:.4} # GS最佳给水比焓 ()\n", h_fwop));
        let t_fwop = ph(params.p_s, h_fwop, OT);
        code.push_str(&format!("\tt_fwop = {:.4} # 最佳给水温度 ()\n", t_fwop));
        let t_fw_calc = params.dt_fw * t_fwop;
        code.push_str(&format!(
            "\tt_fw_calc = {:.4} # 实际给水温度 ()\n",
            t_fw_calc
        ));
        let h_fw_calc = pt(params.p_s, t_fw_calc, OH);
        code.push_str(&format!(
            "\th_fw_calc = {:.4} # 实际给水比焓 ()\n",
            h_fw_calc
        ));
        let dh_fw_calc = (h_fw_calc - h_cd_val) / params.z;
        code.push_str(&format!(
            "\tdh_fw_calc = {:.4} # 每一级加热器内实际给水焓升 ()\n\n",
            dh_fw_calc
        ));

        code.push_str("\t# 除氧器\n");
        let p_dea = 0.99 * p_hz;
        code.push_str(&format!("\tp_dea = {:.4} # 除氧器运行压力 ()\n", p_dea));
        let t_deao = px(p_dea, 0.0, OT);
        code.push_str(&format!("\tt_deao = {:.4} # 除氧器出口温度 ()\n", t_deao));
        let h_deao = tx(t_deao, 0.0, OH);
        code.push_str(&format!(
            "\th_deao = {:.4} # 除氧器出口对应饱和水比焓 ()\n",
            h_deao
        ));
        let dh_fwh = (h_fw_calc - h_deao) / params.z_h;
        code.push_str(&format!(
            "\tdh_fwh = {:.4} # 高压给水加热器每一级给水焓升 ()\n",
            dh_fwh
        ));
        let dh_fwl = (h_deao - h_cd_val) / (params.z_l + 1.0);
        code.push_str(&format!(
            "\tdh_fwl = {:.4} # 除氧器及低压加热器每一级给水焓升 ()\n\n",
            dh_fwl
        ));

        code.push_str("\t# 给水回路系统中的压力选择\n");
        let p_cwp = params.dp_cwp * p_dea;
        code.push_str(&format!("\tp_cwp = {:.4} # 凝水泵出口压力 ()\n", p_cwp));
        let h_cwp = h_cd_val;
        code.push_str(&format!("\th_cwp = {:.4} # 凝水泵出口给水比焓 ()\n", h_cwp));
        let t_cwp = ph(p_cwp, h_cwp, OT);
        code.push_str(&format!("\tt_cwp = {:.4} # 凝水泵出口给水温度 ()\n", t_cwp));
        let dp_cws = p_cwp - p_dea;
        code.push_str(&format!(
            "\tdp_cws = {:.4} # 凝水泵出口至除氧器的阻力压降 ()\n",
            dp_cws
        ));
        let dp_fi = dp_cws / (params.z_l + 1.0);
        code.push_str(&format!(
            "\tdp_fi = {:.4} # 每级低压加热器及除氧器的平均压降 ()\n\n",
            dp_fi
        ));

        code.push_str("\t# 低压给水加热器 \n");
        let (p_fw1i, h_fw1i, t_fw1i, p_fw1o, h_fw1o, t_fw1o, t_ro1k, h_ro1k, p_ro1k) =
            self.calc_fwxl(p_cwp, h_cwp, t_cwp, dp_fi, dh_fwl);
        code.push_str(&format!("\t(p_fw1i, h_fw1i, t_fw1i, p_fw1o, h_fw1o, t_fw1o, t_ro1k, h_ro1k, p_ro1k) = ({:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4})\n",
        p_fw1i, h_fw1i, t_fw1i, p_fw1o, h_fw1o, t_fw1o, t_ro1k, h_ro1k, p_ro1k));
        let (p_fw2i, h_fw2i, t_fw2i, p_fw2o, h_fw2o, t_fw2o, t_ro2k, h_ro2k, p_ro2k) =
            self.calc_fwxl(p_fw1o, h_fw1o, t_fw1o, dp_fi, dh_fwl);
        code.push_str(&format!("\t(p_fw2i, h_fw2i, t_fw2i, p_fw2o, h_fw2o, t_fw2o, t_ro2k, h_ro2k, p_ro2k) = ({:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4})\n",
        p_fw2i, h_fw2i, t_fw2i, p_fw2o, h_fw2o, t_fw2o, t_ro2k, h_ro2k, p_ro2k));
        let (p_fw3i, h_fw3i, t_fw3i, p_fw3o, h_fw3o, t_fw3o, t_ro3k, h_ro3k, p_ro3k) =
            self.calc_fwxl(p_fw2o, h_fw2o, t_fw2o, dp_fi, dh_fwl);
        code.push_str(&format!("\t(p_fw3i, h_fw3i, t_fw3i, p_fw3o, h_fw3o, t_fw3o, t_ro3k, h_ro3k, p_ro3k) = ({:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4})\n",
        p_fw3i, h_fw3i, t_fw3i, p_fw3o, h_fw3o, t_fw3o, t_ro3k, h_ro3k, p_ro3k));
        let (p_fw4i, h_fw4i, t_fw4i, p_fw4o, h_fw4o, t_fw4o, t_ro4k, h_ro4k, p_ro4k) =
            self.calc_fwxl(p_fw3o, h_fw3o, t_fw3o, dp_fi, dh_fwl);
        code.push_str(&format!("\t(p_fw4i, h_fw4i, t_fw4i, p_fw4o, h_fw4o, t_fw4o, t_ro4k, h_ro4k, p_ro4k) = ({:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4})\n",
        p_fw4i, h_fw4i, t_fw4i, p_fw4o, h_fw4o, t_fw4o, t_ro4k, h_ro4k, p_ro4k));

        code.push_str("\t# 除氧器 (续)\n");
        let h_deai = h_fw4o;
        code.push_str(&format!("\th_deai = {:.4} # 进口给水比焓 ()\n", h_deai));
        let p_fwpo = params.dp_fwpo * params.p_s;
        code.push_str(&format!("\tp_fwpo = {:.4} # 给水泵出口压力 ()\n", p_fwpo));
        let h_fwpo_calc = h_deao;
        code.push_str(&format!(
            "\th_fwpo_calc = {:.4} # 给水泵出口流体比焓 ()\n",
            h_fwpo_calc
        ));
        let t_fwpo = ph(p_fwpo, h_fwpo_calc, OT);
        code.push_str(&format!("\tt_fwpo = {:.4} # 给水泵出口水温 ()\n", t_fwpo));
        let p_fwi = params.p_s + 0.1;
        code.push_str(&format!(
            "\tp_fwi = {:.4} # GS二次侧进口给水压力 ()\n\n",
            p_fwi
        ));

        code.push_str("\t# 高压给水加热器\n");
        let (p_fw6i, h_fw6i, t_fw6i, p_fw6o, h_fw6o, t_fw6o, t_ro6k, h_ro6k, p_ro6k) = self
            .calc_fwxh(
                p_fwpo,
                h_fwpo_calc,
                t_fwpo,
                p_fwpo - (p_fwpo - p_fwi) / 2.0,
                dh_fwh,
            );
        code.push_str(&format!("\t(p_fw6i, h_fw6i, t_fw6i, p_fw6o, h_fw6o, t_fw6o, t_ro6k, h_ro6k, p_ro6k) = ({:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4})\n",
        p_fw6i, h_fw6i, t_fw6i, p_fw6o, h_fw6o, t_fw6o, t_ro6k, h_ro6k, p_ro6k));
        let (p_fw7i, h_fw7i, t_fw7i, p_fw7o, h_fw7o, t_fw7o, t_ro7k, h_ro7k, p_ro7k) =
            self.calc_fwxh(p_fw6o, h_fw6o, t_fw6o, p_fwi, dh_fwh);
        code.push_str(&format!("\t(p_fw7i, h_fw7i, t_fw7i, p_fw7o, h_fw7o, t_fw7o, t_ro7k, h_ro7k, p_ro7k) = ({:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4}, {:.4})\n",
        p_fw7i, h_fw7i, t_fw7i, p_fw7o, h_fw7o, t_fw7o, t_ro7k, h_ro7k, p_ro7k));

        code.push_str("\t# 高压缸抽汽\n");
        let (p_hes6, h_hes6s, h_hes6, x_hes6) = self.calc_esx(p_ro6k, s_hi, h_hi, true);
        code.push_str(&format!(
            "\t(p_hes6, h_hes6s, h_hes6, x_hes6) = ({:.4}, {:.4}, {:.4}, {:.4})\n",
            p_hes6, h_hes6s, h_hes6, x_hes6
        ));
        let (p_hes7, h_hes7s, h_hes7, x_hes7) = self.calc_esx(p_ro7k, s_hi, h_hi, true);
        code.push_str(&format!(
            "\t(p_hes7, h_hes7s, h_hes7, x_hes7) = ({:.4}, {:.4}, {:.4}, {:.4})\n",
            p_hes7, h_hes7s, h_hes7, x_hes7
        ));

        code.push_str("\t# 低压缸抽汽\n");
        let (p_les1, h_les1s, h_les1, x_les1) = self.calc_esx(p_ro1k, s_li, h_li, false);
        code.push_str(&format!(
            "\t(p_les1, h_les1s, h_les1, x_les1) = ({:.4}, {:.4}, {:.4}, {:.4})\n",
            p_les1, h_les1s, h_les1, x_les1
        ));
        let (p_les2, h_les2s, h_les2, x_les2) = self.calc_esx(p_ro2k, s_li, h_li, false);
        code.push_str(&format!(
            "\t(p_les2, h_les2s, h_les2, x_les2) = ({:.4}, {:.4}, {:.4}, {:.4})\n",
            p_les2, h_les2s, h_les2, x_les2
        ));
        let (p_les3, h_les3s, h_les3, x_les3) = self.calc_esx(p_ro3k, s_li, h_li, false);
        code.push_str(&format!(
            "\t(p_les3, h_les3s, h_les3, x_les3) = ({:.4}, {:.4}, {:.4}, {:.4})\n",
            p_les3, h_les3s, h_les3, x_les3
        ));
        let (p_les4, h_les4s, h_les4, x_les4) = self.calc_esx(p_ro4k, s_li, h_li, false);
        code.push_str(&format!(
            "\t(p_les4, h_les4s, h_les4, x_les4) = ({:.4}, {:.4}, {:.4}, {:.4})\n",
            p_les4, h_les4s, h_les4, x_les4
        ));

        code.push_str("\t# 再热器抽汽\n");
        let (p_rh1_calc, x_rh1_calc, t_rh1_calc, h_rh1_calc, h_zs1_calc) =
            self.calc_rhx(p_hes7, x_hes7);
        code.push_str(&format!("\t(p_rh1_calc, x_rh1_calc, t_rh1_calc, h_rh1_calc, h_zs1_calc) = ({:.4}, {:.4}, {:.4}, {:.4}, {:.4})\n",
        p_rh1_calc, x_rh1_calc, t_rh1_calc, h_rh1_calc, h_zs1_calc));
        let (p_rh2_calc, x_rh2_calc, t_rh2_calc, h_rh2_calc, h_zs2_calc) =
            self.calc_rhx(p_hi, x_hi);
        code.push_str(&format!("\t(p_rh2_calc, x_rh2_calc, t_rh2_calc, h_rh2_calc, h_zs2_calc) = ({:.4}, {:.4}, {:.4}, {:.4}, {:.4})\n",
        p_rh2_calc, x_rh2_calc, t_rh2_calc, h_rh2_calc, h_zs2_calc));
        code.push_str("\t# 蒸汽发生器总蒸汽产量的计算 (迭代循环)\n");
        let h_a = h_hi - h_hz;
        code.push_str(&format!(
            "\th_a = {:.4} # 给水泵汽轮机中蒸汽的绝热焓降\n",
            h_a
        ));
        code.push_str(&format!(
            "\tmutable_params_g_cd = {:.4} # 迭代变量，初始值为 g_cd\n",
            params.g_cd,
        ));
        code.push_str(&format!(
            "\tne = {:.4} # 迭代变量，初始值为 ne\n",
            params.ne,
        ));
        code.push_str(&format!(
            "\tmutable_params_ne_npp = {:.4} # 迭代变量，初始值为 ne_npp\n",
            params.ne_npp,
        ));
        code.push_str("\td_s_loop = 0.0\n");
        code.push_str("\tg_fw_loop = 0.0\n");
        code.push_str("\tq_r_loop = 0.0\n");
        code.push_str("\tg_sh_loop = 0.0\n\tg_sl_loop = 0.0\n\tg_fwps_loop = 0.0\n");
        code.push_str(
            "\tg_les4_loop = 0.0\n\tg_les3_loop = 0.0\n\tg_les2_loop = 0.0\n\tg_les1_loop = 0.0\n",
        );
        code.push_str(
            "\tg_zc1_loop = 0.0\n\tg_zc2_loop = 0.0\n\tg_hes6_loop = 0.0\n\tg_hes7_loop = 0.0\n",
        );
        code.push_str("\tg_uw_loop = 0.0\n\tg_sdea_loop = 0.0\n");
        code.push_str("\titeration_count = 0;\n\n");

        code.push_str("\twhile True: # 外层循环: 迭代优化核电厂效率 (ne_npp)\n");
        code.push_str("\t\tq_r_loop = ne / mutable_params_ne_npp # 反应堆热功率(MW)\n");
        code.push_str(&format!("\t\td_s_loop = (1000.0 * q_r_loop * {:.4}) / ((h_fh - h_s_calc) + (1.0 + {:.4}) * (h_s_calc - h_fw_calc)) # GS蒸汽产量(kg/s)\n", params.n_1, params.zeta_d));
        code.push_str(&format!(
            "\t\tg_fw_loop = (1.0 + {:.4}) * d_s_loop # GS给水流量(kg/s)\n",
            params.zeta_d
        ));
        code.push_str("\t\th_fwp_loop = p_fwpo - p_dea # 给水泵扬程(MPa)\n");
        code.push_str(&format!(
            "\t\trho_fwp_loop = 0.5 * {} + {} # 给水泵中水的密度\n\n",
            px(p_dea, 0.0, OD),
            px(p_fwpo, 0.0, OD)
        ));

        code.push_str("\t\twhile True:# 内层循环: 迭代优化冷凝器凝结水量 (g_cd)\n");
        code.push_str("\t\t\tn_fwpp_loop = 1000.0 * g_fw_loop * h_fwp_loop / rho_fwp_loop # 给水泵有效输出功率(kW)\n");
        code.push_str(&format!(
        "\t\t\tn_fwpt_loop = n_fwpp_loop / ({:.4} * {:.4} * {:.4} * {:.4}) # 给水泵理论功率(kW)\n",
        params.n_fwpp, params.n_fwpti, params.n_fwptg, params.n_fwptm
    ));
        code.push_str(&format!(
            "\t\t\tg_fwps_loop = n_fwpt_loop / h_a# 给水泵汽轮机耗汽量(kg/s)\n\n"
        ));

        code.push_str("\t\t# 低压给水加热器抽汽量\n");
        code.push_str(&format!("\t\t\tg_les4_loop = mutable_params_g_cd * (h_fw4o - h_fw4i) / ({:.4} * (h_les4 - h_ro4k));\n", params.n_h));
        code.push_str(&format!("\t\t\tg_les3_loop = (mutable_params_g_cd * (h_fw3o - h_fw3i) - {:.4} * g_les4_loop * (h_ro4k - h_ro3k)) / ({:.4} * (h_les3 - h_ro3k))\n", params.n_h, params.n_h));
        code.push_str(&format!("\t\t\tg_les2_loop = (mutable_params_g_cd * (h_fw2o - h_fw2i) - {:.4} * (g_les3_loop + g_les4_loop) * (h_ro3k - h_ro2k)) / ({:.4} * (h_les2 - h_ro2k))\n", params.n_h, params.n_h));
        code.push_str(&format!("\t\t\tg_les1_loop = (mutable_params_g_cd * (h_fw1o - h_fw1i) - {:.4} * (g_les2_loop + g_les3_loop + g_les4_loop) * (h_ro2k - h_ro1k)) / ({:.4} * (h_les1 - h_ro1k))\n\n", params.n_h, params.n_h));

        code.push_str("\t\t# 低压缸耗气量(kg/s)\n");
        code.push_str(&format!("\t\t\tg_sl_loop = (0.6 * 1000.0 * {:.4} / ({:.4} * {:.4}) + g_les4_loop * (h_les4 - h_lz) + g_les3_loop * (h_les3 - h_lz) + g_les2_loop * (h_les2 - h_lz) + g_les1_loop * (h_les1 - h_lz)) / (h_li - h_lz)\n\n",
            params.ne, params.n_m, params.n_ge));

        code.push_str("\t\t# 再热器加热蒸汽量\n");
        code.push_str(&format!(
            "\t\t\tg_zc1_loop = g_sl_loop * dh_rh / ({:.4} * (h_rh1_calc - h_zs1_calc))\n",
            params.n_h
        ));
        code.push_str(&format!(
            "\t\t\tg_zc2_loop = g_sl_loop * dh_rh / ({:.4} * (h_rh2_calc - h_zs2_calc))\n\n",
            params.n_h
        ));

        code.push_str("\t\t# 高压给水加热器抽汽量\n");
        code.push_str(&format!("\t\t\tg_hes7_loop = (g_fw_loop * (h_fw7o - h_fw7i) - {:.4} * g_zc2_loop * (h_zs2_calc - h_ro7k)) / ({:.4} * (h_hes7 - h_ro7k))\n", params.n_h, params.n_h));
        code.push_str(&format!("\t\t\tg_hes6_loop = (g_fw_loop * (h_fw6o - h_fw6i) - {:.4} * g_zc1_loop * (h_zs1_calc - h_ro6k) - {:.4} * (g_zc2_loop + g_hes7_loop) * (h_ro7k - h_ro6k)) / ({:.4} * (h_hes6 - h_ro6k))\n\n", params.n_h, params.n_h, params.n_h));

        code.push_str(&format!(
            "\t\t\tg_uw_loop = g_sl_loop * (x_rh1i - x_spi) / x_spi# 汽水分离器疏水流量(kg/s)\n\n"
        ));

        code.push_str("\t\t# 除氧器耗汽量\n");
        code.push_str(&format!("\t\t\tg_sdea_loop = (g_fw_loop * h_deao - g_uw_loop * h_uw - mutable_params_g_cd * h_fw4o - (g_zc1_loop + g_zc2_loop + g_hes6_loop + g_hes7_loop) * h_ro6k) / h_hz\n\n"));

        code.push_str("\t\t# 高压缸耗汽量\n");
        code.push_str(&format!("\t\t\tg_sh_loop = (0.4 * 1000.0 * ne / ({:.4} * {:.4}) + g_hes7_loop * (h_hes7 - h_hz) + g_hes6_loop * (h_hes6 - h_hz) + g_zc1_loop * (h_rh1_calc - h_hz)) / (h_hi - h_hz)\n\n", params.n_m, params.n_ge));

        code.push_str("\t\t# 对假设冷凝水流量验证\n");
        code.push_str("\t\t\td_s_loop = g_fwps_loop + g_zc2_loop + g_sh_loop# 新蒸汽耗量 (根据新的流量重新评估 d_s_loop)\n");
        code.push_str(&format!(
            "\t\t\tg_fw1_loop = (1.0 + {:.4}) * d_s_loop# 给水流量\n",
            params.zeta_d
        ));
        code.push_str(
            "\t\t\tg_cd1_loop = g_fw1_loop - g_sdea_loop - g_uw_loop - (g_hes6_loop + g_hes7_loop + g_zc1_loop + g_zc2_loop)\n",
        );

        code.push_str(
            "\t\t\tif abs(g_cd1_loop - mutable_params_g_cd) / mutable_params_g_cd < 1e-2:\n",
        );
        code.push_str("\t\t\t\tbreak# 内层循环中断\n");
        code.push_str("\t\t\telse:\n");
        code.push_str("\t\t\t\tmutable_params_g_cd = g_cd1_loop\n");
        code.push_str("\t\t\t\tg_fw_loop = g_fw1_loop# 为下一次内层迭代更新 g_fw_loop\n");
        code.push_str("\t# 内层循环结束\n\n");

        code.push_str(&format!("\t\tq_r_loop = (d_s_loop * (h_fh - h_fw_calc) + {:.4} * d_s_loop * (h_s_calc - h_fw_calc)) / (1000.0 * {:.4})# 新反应堆热功率(MW)\n", params.zeta_d, params.n_1));
        code.push_str("\t\tn_ennp1_loop = ne / q_r_loop\n\n");

        // 打印迭代结果 (CalcResult1)
        code.push_str("\t\titeration_count += 1;\n");
        code.push_str("\t\tprint(\"\\n--- Iteration: {} ---\", iteration_count)\n");
        code.push_str("\t\tprint(f\"  核电厂热效率η-enpp: {mutable_params_ne_npp:.4}\")\n");
        code.push_str("\t\tprint(f\"  反应堆热功率Qʀ(MW): {q_r_loop:.4}\")\n");
        code.push_str("\t\tprint(f\"  新蒸汽流量D(kg/s): {d_s_loop:.4}\")\n");
        code.push_str("\t\tprint(f\"  高压缸耗汽量G-shp(kg/s): {g_sh_loop:.4}\")\n");
        code.push_str("\t\tprint(f\"  低压缸耗汽量G-slp(kg/s): {g_sl_loop:.4}\")\n");
        code.push_str(
            "\t\tprint(f\"  一级再热耗汽量G-srh1(kg/s) (g_zc1_loop): {g_zc1_loop:.4}\")\n",
        );
        code.push_str(
            "\t\tprint(f\"  二级再热耗汽量G-srh2(kg/s) (g_zc2_loop): {g_zc2_loop:.4}\")\n",
        );
        code.push_str("\t\tprint(f\"  除氧器耗汽量G-sdea(kg/s): {g_sdea_loop:.4}\")\n");
        code.push_str("\t\tprint(f\"  给水泵汽轮机耗汽量G-sfwp(kg/s): {g_fwps_loop:.4}\")\n");
        code.push_str("\t\tprint(f\"  总给水流量G-fw(kg/s): {g_fw_loop:.4}\")\n");
        code.push_str("\t\tprint(f\"  给水泵扬程Δh-fwp(MPa): {h_fwp_loop:.4}\")\n");
        code.push_str("\t\tprint(f\"  七级回热抽汽量G-hes7(kg/s): {g_hes7_loop:.4}\")\n");
        code.push_str("\t\tprint(f\"  六级回热抽汽量G-hes6(kg/s): {g_hes6_loop:.4}\")\n");
        code.push_str("\t\tprint(f\"  四级回热抽汽量G-les4(kg/s): {g_les4_loop:.4}\")\n");
        code.push_str("\t\tprint(f\"  三级回热抽汽量G-les3(kg/s): {g_les3_loop:.4}\")\n");
        code.push_str("\t\tprint(f\"  二级回热抽汽量G-les2(kg/s): {g_les2_loop:.4}\")\n");
        code.push_str("\t\tprint(f\"  一级回热抽汽量G-les1(kg/s): {g_les1_loop:.4}\")\n");
        code.push_str("\t\tprint(f\"  冷凝器凝结水量G-cd(kg/s): {mutable_params_g_cd:.4}\")\n");
        code.push_str("\t\tprint(f\"  汽水分离器疏水量G-uw(kg/s): {g_uw_loop:.4}\")\n");

        code.push_str("\t\tif abs(n_ennp1_loop - mutable_params_ne_npp) < 1e-3:\n");
        code.push_str("\t\t\tbreak# 外层循环中断\n");
        code.push_str("\t\telse:\n");
        code.push_str("\t\t\tmutable_params_ne_npp = n_ennp1_loop\n");
        code.push_str("# 外层循环结束\n\n");

        // 打印最终结果 (CalcResult2)
        code.push_str("\tprint(\"\\n--- Final Calculation Results ---\")\n");

        code.push_str("\tprint(\"\\n--- 附表一 (部分为输入参数) ---\")\n");
        code.push_str("\tprint(\"  1.核电厂输出功率N-e(MW): {ne:.4}\")\n");
        code.push_str(&format!(
            "\tprint(\"  2.一回路能量利用系数η-1: {:.4}\")\n",
            params.n_1
        ));
        code.push_str(&format!(
            "\tprint(\"  3.蒸汽发生器出口蒸汽干度X-fh: {:.4}\")\n",
            params.x_fh
        ));
        code.push_str(&format!(
            "\tprint(\"  4.蒸汽发生器排污率ξ-d: {:.4}\")\n",
            params.zeta_d
        ));
        code.push_str(&format!(
            "\tprint(\"  5.高压缸内效率η-hi: {:.4}\")\n",
            params.n_hi
        ));
        code.push_str(&format!(
            "\tprint(\"  6.低压缸内效率η-li: {:.4}\")\n",
            params.n_li
        ));
        code.push_str(&format!(
            "\tprint(\"  7.汽轮机组机械效率η-m: {:.4}\")\n",
            params.n_m
        ));
        code.push_str(&format!(
            "\tprint(\"  8.发电机效率η-ge: {:.4}\")\n",
            params.n_ge
        ));
        code.push_str("\tprint(f\"  9.新蒸汽压损Δp-fh(MPa): {dp_fh_calc:.4}\")\n");
        code.push_str("\tprint(f\"  10.再热蒸汽压损Δp-rh (MPa): {dp_rh_calc:.4}\")\n");
        code.push_str(&format!(
            "\tprint(\"  11.回热抽汽压损Δp-ej (%): {:.4}\")\n",
            params.dp_ej
        ));
        code.push_str("\tprint(\"  12.低压缸排气压损Δp-cd (MPa): {:.4}\", dp_cd_calc)\n");
        code.push_str(&format!(
            "\tprint(\"  13.高压给水加热器出口端差θ-hu(°C): {:.4}\")\n",
            params.theta_hu
        ));
        code.push_str(&format!(
            "\tprint(\"  14.低压给水加热器出口端差θ-lu(°C): {:.4}\");\n",
            params.theta_lu
        ));
        code.push_str(&format!(
            "\tprint(\"  15.加热器效率η-h: {:.4}\")\n",
            params.n_h
        ));
        code.push_str(&format!(
            "\tprint(\"  16.给水泵效率η-fwpp: {:.4}\")\n",
            params.n_fwpp
        ));
        code.push_str(&format!(
            "\tprint(\"  17.给水泵汽轮机内效率η-fwpti: {:.4}\")\n",
            params.n_fwpti
        ));
        code.push_str(&format!(
            "\tprint(\"  18.给水泵汽轮机机械效率η-fwptm: {:.4}\")\n",
            params.n_fwptm
        ));
        code.push_str(&format!(
            "\tprint(\"  19.给水泵汽轮机减速器效率η-fwptg: {:.4}\")\n",
            params.n_fwptg
        ));
        code.push_str(&format!(
            "\tprint(\"  20.循环冷却水进口温度T-sw1(°C): {:.4}\")\n",
            params.t_sw1
        ));

        code.push_str("\tprint(\"\\n--- 附表二 (计算结果) ---\")\n");
        code.push_str(&format!(
            "\tprint(\"  1.反应堆冷却剂系统运行压力p-c(MPa): {:.4}\")\n",
            params.p_c
        ));
        code.push_str("\tprint(f\"  2.冷却剂压力对应的饱和温度T-cs(°C): {t_cs:.4}\")\n");
        code.push_str(&format!(
            "\tprint(\"  3.反应堆出口冷却剂过冷度ΔT-sub(°C): {:.4}\")\n",
            params.dt_sub
        ));
        code.push_str("\tprint(f\"  4.反应堆出口冷却剂温度T-co(°C): {t_co:.4}\")\n");
        code.push_str(&format!(
            "\tprint(\"  5.反应堆进出口冷却剂温升ΔT-c(°C): {:.4}\")\n",
            params.dt_c
        ));
        code.push_str("\tprint(f\"  6.反应堆进口冷却剂温度T-ci(°C): {t_ci:.4}\")\n");
        code.push_str(&format!(
            "\tprint(\"  7.蒸汽发生器饱和蒸汽压力p-s(MPa): {:.4}\")\n",
            params.p_s
        ));
        code.push_str("\tprint(f\"  8.蒸汽发生器饱和蒸汽温度T-fh(°C): {t_fh:.4}\")\n");
        code.push_str("\tprint(f\"  9.一、二次侧对数平均温差ΔT-m(°C): {dt_m:.4}\")\n");
        code.push_str(&format!(
            "\tprint(\"  10.冷凝器中循环冷却水温升ΔT-sw(°C): {:.4}\")\n",
            params.dt_sw
        ));
        code.push_str(&format!(
            "\tprint(\"  11.冷凝器传热端差δT(°C): {:.4}\")\n",
            params.dt
        ));
        code.push_str("\tprint(f\"  12.冷凝器凝结水饱和温度T-cd(°C): {t_cd:.4}\")\n");
        code.push_str("\tprint(f\"  13.冷凝器的运行压力p-cd(MPa): {p_cd:.4}\")\n");
        code.push_str("\tprint(f\"  14.高压缸进口的蒸汽压力p-hi(MPa): {p_hi:.4}\")\n");
        code.push_str("\tprint(f\"  15.高压缸进口蒸汽干度X-hi: {x_hi:.4}\")\n");
        code.push_str("\tprint(f\"  15.1.蒸汽发生器出口蒸汽比焓h-fh(kJ/kg): {h_fh:.4}\")\n");
        code.push_str("\tprint(f\"  15.2.蒸汽发生器出口蒸汽比熵s-fh: {s_fh:.4}\")\n");
        code.push_str("\tprint(f\"  15.3.高压缸进口蒸汽比熵s-hi: {s_hi:.4}\")\n");
        code.push_str("\tprint(f\"  16.高压缸排气压力p-hz(MPa): {p_hz:.4}\")\n");
        code.push_str("\tprint(f\"  17.高压缸排气干度X-hz: {x_hz:.4}\");\n");
        code.push_str("\tprint(f\"  17.1.高压缸进口蒸汽比焓h-hi(kJ/kg): {h_hi:.4}\")\n");
        code.push_str("\tprint(f\"  17.2.高压缸出口理想比焓h-hzs(kJ/kg): {h_hzs:.4}\")\n");
        code.push_str("\tprint(f\"  17.3.高压缸出口蒸汽比焓h-hz(kJ/kg): {h_hz:.4}\")\n");
        code.push_str("\tprint(f\"  18.汽水分离器进口蒸汽压力p-spi(MPa): {p_spi:.4}\")\n");
        code.push_str("\tprint(f\"  19.汽水分离器进口蒸汽干度X-spi: {x_spi:.4}\")\n");
        code.push_str("\tprint(f\"  19.1.汽水分离器出口疏水压力p-uw(MPa): {p_uw:.4}\")\n");
        code.push_str("\tprint(f\"  19.2.汽水分离器出口疏水比焓h-uw(kJ/kg): {h_uw:.4}\")\n");
        code.push_str("\tprint(f\"  20.再热蒸汽进口压力p-rh1i(MPa): {p_rh1i:.4}\")\n");
        code.push_str("\tprint(f\"  21.再热蒸汽进口干度X-rh1i: {x_rh1i:.4}\")\n");
        code.push_str("\tprint(f\"  21.1.一级再热器进口蒸汽比焓h-rh1i(kJ/kg): {h_rh1i:.4}\")\n");

        code.push_str("\tprint(\"\\n--- 附表二 (Result2) - 续 ---\");\n");
        code.push_str(
            "\tprint(f\"  22.加热蒸汽进口压力p-rh1hs(MPa) (p_rh1_calc): {p_rh1_calc:.4}\")\n",
        );
        code.push_str("\tprint(f\"  23.加热蒸汽进口干度X-rh1hs (x_rh1_calc): {x_rh1_calc:.4}\")\n");
        code.push_str("\tprint(f\"  24.再热蒸汽进口压力p-rh2i(MPa): {p_rh2i:.4}\")\n");
        code.push_str("\tprint(f\"  25.再热蒸汽进口温度T-rh2i(°C): {t_rh2i:.4}\")\n");
        code.push_str("\tprint(f\"  26.再热蒸汽出口压力p-rh2z(MPa): {p_rh2z:.4}\")\n");
        code.push_str("\tprint(f\"  27.再热蒸汽出口温度T-rh2z(°C): {t_rh2z:.4}\")\n");
        code.push_str("\tprint(f\"  27.1.二级再热器出口比焓h-rh2z(kJ/kg): {h_rh2z:.4}\")\n");
        code.push_str("\tprint(f\"  27.2.每级再热器平均焓升Δh-rh(kJ/kg): {dh_rh:.4}\")\n");
        code.push_str("\tprint(f\"  27.3.一级再热器出口蒸汽比焓h-rh1z(kJ/kg): {h_rh1z:.4}\")\n");
        code.push_str(
        "\tprint(f\"  27.4.二级再热器进口蒸汽比焓h-rh2i(kJ/kg) (h_rh2i_calc): {h_rh2_calc:.4}\")\n",
    );
        code.push_str(
            "\tprint(f\"  28.加热蒸汽进口压力p-rh2hs(MPa) (p_rh2_calc): {p_rh2_calc:.4}\")\n",
        );
        code.push_str("\tprint(f\"  29.加热蒸汽进口干度X-rh2hs (x_rh2_calc): {x_rh2_calc:.4}\")\n");
        code.push_str("\tprint(f\"  30.进口蒸汽压力p-li(MPa): {p_li:.4}\")\n");
        code.push_str("\tprint(f\"  31.进口蒸汽温度T-li(°C): {t_li:.4}\")\n");
        code.push_str("\tprint(f\"  32.排汽压力p-lz(MPa): {p_lz:.4}\")\n");
        code.push_str("\tprint(f\"  33.排汽干度X-lz: {x_lz:.4}\")\n");
        code.push_str("\tprint(f\"  33.1.低压缸进口蒸汽比熵s-li: {s_li:.4}\")\n");
        code.push_str("\tprint(f\"  33.2.低压缸进口蒸汽比焓h-li(kJ/kg): {h_li:.4}\")\n");
        code.push_str("\tprint(f\"  33.3.低压缸出口理想比焓h-lzs(kJ/kg): {h_lzs:.4}\")\n");
        code.push_str("\tprint(f\"  33.4.低压缸出口蒸汽比焓h-lz(kJ/kg): {h_lz:.4}\")\n");
        code.push_str(&format!("\tprint(\"  34.回热级数Z: {:.4}\")\n", params.z));
        code.push_str(&format!(
            "\tprint(\"  35.低压给水加热器级数Z-l: {:.4}\")\n",
            params.z_l
        ));
        code.push_str(&format!(
            "\tprint(\"  36.高压给水加热器级数Z-h: {:.4}\")\n",
            params.z_h
        ));
        code.push_str("\tprint(f\"  37.第一次给水回热分配Δh-fw: {dh_fw_calc:.4}\")\n");
        code.push_str(
            "\tprint(f\"  37.1.蒸汽发生器运行压力饱和水比焓h-s(kJ/kg): {h_s_calc:.4}\")\n",
        );
        code.push_str("\tprint(f\"  37.2.冷凝器出口凝结水比焓h-cd(kJ/kg): {h_cd_val:.4}\")\n");
        code.push_str("\tprint(f\"  37.3.每级加热器理论给水焓升Δh-fwop(kJ/kg): {dh_fwop:.4}\")\n");
        code.push_str("\tprint(f\"  37.4.最佳给水比焓h-fwop(kJ/kg): {h_fwop:.4}\")\n");
        code.push_str("\tprint(f\"  37.5.最佳给水温度T-fwop(°C): {t_fwop:.4}\")\n");
        code.push_str("\tprint(f\"  37.6.实际给水温度T-fw(°C): {t_fw_calc:.4}\")\n");
        code.push_str("\tprint(f\"  37.7.实际给水比焓h-fw(kJ/kg): {h_fw_calc:.4}\")\n");

        code.push_str("\tprint(f\"\\n--- 给水及除氧器参数 ---\")\n");
        code.push_str("\tprint(f\"  38.高压加热器给水焓升Δh-fwh(kJ/kg): {dh_fwh:.4}\")\n");
        code.push_str("\tprint(f\"  38.1.除氧器运行压力p-dea(MPa): {p_dea:.4}\")\n");
        code.push_str("\tprint(f\"  38.2.除氧器出口饱和水比焓h-deao(kJ/kg): {h_deao:.4}\")\n");
        code.push_str("\tprint(f\"  39.除氧器及低压加热器给水焓升Δh-fwl(kJ/kg): {dh_fwl:.4}\")\n");
        code.push_str("\tprint(f\"  39.1.凝水泵出口给水压力p-cwp(MPa): {p_cwp:.4}\")\n");
        code.push_str("\tprint(f\"  39.2.凝水泵出口给水比焓h-cwp(kJ/kg): {h_cwp:.4}\")\n");
        code.push_str(
            "\tprint(f\"  39.3.凝水泵出口至除氧器出口阻力压降Δp-cws(MPa): {dp_cws:.4}\")\n",
        );
        code.push_str(
            "\tprint(f\"  39.4.每级低压加热器及除氧器阻力压降Δp-fi(MPa): {dp_fi:.4}\")\n",
        );

        code.push_str("\tprint(\"\\n  --- 低压给水加热器 (lfwx) ---\")\n");
        code.push_str("\tlfwx_data = [\n");
        code.push_str(
            "\t\t(1, p_fw1i, h_fw1i, t_fw1i, p_fw1o, h_fw1o, t_fw1o, t_ro1k, h_ro1k, p_ro1k),\n",
        );
        code.push_str(
            "\t\t(2, p_fw2i, h_fw2i, t_fw2i, p_fw2o, h_fw2o, t_fw2o, t_ro2k, h_ro2k, p_ro2k),\n",
        );
        code.push_str(
            "\t\t(3, p_fw3i, h_fw3i, t_fw3i, p_fw3o, h_fw3o, t_fw3o, t_ro3k, h_ro3k, p_ro3k),\n",
        );
        code.push_str(
            "\t\t(4, p_fw4i, h_fw4i, t_fw4i, p_fw4o, h_fw4o, t_fw4o, t_ro4k, h_ro4k, p_ro4k),\n",
        );
        code.push_str("\t]\n");
        code.push_str(
            "\tfor (level, p_i, h_i, t_i, p_o, h_o, t_o, t_r, h_r, _p_r) in lfwx_data:\n",
        );
        code.push_str("\t\tprint(f\"      进口给水压力p-fwxi(MPa): {p_i:.4}\")\n");
        code.push_str("\t\tprint(f\"      进口给水比焓h-fwxi(kJ/kg): {h_i:.4}\")\n");
        code.push_str("\t\tprint(f\"      进口给水温度T-fwxi(°C): {t_i:.4}\")\n");
        code.push_str("\t\tprint(f\"      出口给水压力p-fwxo(MPa): {p_o:.4}\")\n");
        code.push_str("\t\tprint(f\"      出口给水比焓h-fwxo(kJ/kg): {h_o:.4}\")\n");
        code.push_str("\t\tprint(f\"      出口给水温度T-fwxo(°C): {t_o:.4}\")\n");
        code.push_str("\t\tprint(f\"      疏水温度T-roxk(°C): {t_r:.4}\")\n");
        code.push_str("\t\tprint(f\"      疏水比焓h-roxk(kJ/kg): {h_r:.4}\")\n\n");

        code.push_str("\tprint(f\"  41.进口给水比焓h-deai(kJ/kg) (除氧器): {h_deai:.4}\")\n");
        code.push_str(
            "\tprint(f\"  42.出口给水比焓h-deao(kJ/kg) (除氧器, h_deao): {h_deao:.4}\")\n",
        );
        code.push_str("\tprint(f\"  43.出口给水温度T-dea(°C) (除氧器, t_deao): {t_deao:.4}\")\n");
        code.push_str("\tprint(f\"  44.运行压力p-dea(MPa) (除氧器, p_dea): {p_dea:.4}\")\n");
        code.push_str("\tprint(f\"  44.1.给水泵出口压力p-fwpo(MPa): {p_fwpo:.4}\")\n");
        code.push_str("\tprint(f\"  44.2.给水泵出口流体比焓h-fwpo(kJ/kg): {h_fwpo_calc:.4}\")\n");
        code.push_str("\tprint(f\"  44.3.蒸汽发生器进口给水压力p-fwi(MPa): {p_fwi:.4}\")\n");

        code.push_str("\tprint(\"\\n  --- 高压给水加热器 (hfwx) ---\")\n");
        code.push_str("\thfwx_data = [\n");
        code.push_str(
            "\t\t(6, p_fw6i, h_fw6i, t_fw6i, p_fw6o, h_fw6o, t_fw6o, t_ro6k, h_ro6k, p_ro6k),\n",
        );
        code.push_str(
            "\t\t(7, p_fw7i, h_fw7i, t_fw7i, p_fw7o, h_fw7o, t_fw7o, t_ro7k, h_ro7k, p_ro7k)\n",
        );
        code.push_str("\t]\n");
        code.push_str(
            "\tfor (level, p_i, h_i, t_i, p_o, h_o, t_o, t_r, h_r, _p_r) in hfwx_data:\n",
        );
        code.push_str("\t\tprint(f\"    --- {level}级高压加热器给水参数 ---\")\n");
        code.push_str("\t\tprint(f\"      进口给水压力p-fwxi(MPa): {p_i:.4}\")\n");
        code.push_str("\t\tprint(f\"      进口给水比焓h-fwxi(kJ/kg): {h_i:.4}\")\n");
        code.push_str("\t\tprint(f\"      进口给水温度T-fwxi(°C): {t_i:.4}\")\n");
        code.push_str("\t\tprint(f\"      出口给水压力p-fwxo(MPa): {p_o:.4}\")\n");
        code.push_str("\t\tprint(f\"      出口给水比焓h-fwxo(kJ/kg): {h_o:.4}\")\n");
        code.push_str("\t\tprint(f\"      出口给水温度T-fwxo(°C): {t_o:.4}\")\n");
        code.push_str("\t\tprint(f\"      疏水温度T-roxk(°C): {t_r:.4}\")\n");
        code.push_str("\t\tprint(f\"      疏水比焓h-roxk(kJ/kg): {h_r:.4}\")\n");
        code.push_str("\n");

        code.push_str("\tprint(\"\\n--- 46.高压缸抽气 ---\")\n");
        code.push_str("\tprint(f\"  46.1.高压缸进口蒸汽比熵s-hi: {s_hi:.4}\")\n");
        code.push_str("\tprint(f\"  46.2.高压缸进口蒸汽比焓h-hi(kJ/kg): {h_hi:.4}\")\n");
        code.push_str("\thhes_data = [\n");
        code.push_str("\t\t(6, p_hes6, h_hes6s, h_hes6, x_hes6),\n");
        code.push_str("\t\t(7, p_hes7, h_hes7s, h_hes7, x_hes7),\n");
        code.push_str("\t]\n");
        code.push_str("\tfor (level, p_ex, h_exs, h_ex, x_ex) in hhes_data:\n");
        code.push_str("\t\tprint(\"    --- {}号高压抽汽参数 ---\", level)\n");
        code.push_str("\t\tprint(f\"      抽汽压力p-hesx(MPa): {p_ex:.4}\")\n");
        code.push_str("\t\tprint(f\"      抽汽干度X-hesx: {x_ex:.4}\")\n");
        code.push_str("\t\tprint(f\"      抽汽理想比焓h-hesxs(kJ/kg): {h_exs:.4}\")\n");
        code.push_str("\t\tprint(f\"      抽汽比焓h-hesx(kJ/kg): {h_ex:.4}\")\n");
        code.push_str("\n");

        code.push_str("\tprint(\"\\n--- 47.低压缸抽气 ---\")\n");
        code.push_str("\tprint(f\"  47.1.低压缸进口蒸汽比熵s-li: {s_li:.4}\")\n");
        code.push_str("\tprint(f\"  47.2.低压缸进口蒸汽比焓h-li(kJ/kg): {h_li:.4}\")\n");
        code.push_str("\tlhes_data = [\n");
        code.push_str("\t\t(1, p_les1, h_les1s, h_les1, x_les1),\n");
        code.push_str("\t\t(2, p_les2, h_les2s, h_les2, x_les2),\n");
        code.push_str("\t\t(3, p_les3, h_les3s, h_les3, x_les3),\n");
        code.push_str("\t\t(4, p_les4, h_les4s, h_les4, x_les4),\n");
        code.push_str("\t]\n");
        code.push_str("\tfor (level, p_ex, h_exs, h_ex, x_ex) in lhes_data:\n");
        code.push_str("\t\tprint(\"    --- {}号低压抽汽参数 ---\", level)\n");
        code.push_str("\t\tprint(f\"      抽汽压力p-lesx(MPa): {p_ex:.4}\")\n");
        code.push_str("\t\tprint(f\"      抽汽干度X-lesx: {x_ex:.4}\")\n");
        code.push_str("\t\tprint(f\"      抽汽理想比焓h-lesxs(kJ/kg): {h_exs:.4}\")\n");
        code.push_str("\t\tprint(f\"      抽汽比焓h-lesx(kJ/kg): {h_ex:.4}\")\n");
        code.push_str("\n");

        code.push_str("\tprint(\"\\n--- 48.再热器抽气 ---\")\n");
        code.push_str("\trhx_data = [\n");
        code.push_str("\t\t(1, p_rh1_calc, x_rh1_calc, t_rh1_calc, h_rh1_calc, h_zs1_calc),\n");
        code.push_str("\t\t(2, p_rh2_calc, x_rh2_calc, t_rh2_calc, h_rh2_calc, h_zs2_calc),\n");
        code.push_str("\t]\n");
        code.push_str("\tfor (level, p_rhx_v, x_rhx_v, t_rhx_v, h_rhx_v, h_zsx_v) in rhx_data:\n");
        code.push_str("\t\tprint(\"    --- {}级再热器参数 ---\", level)\n");
        code.push_str("\t\tprint(f\"      加热蒸汽进口压力p-rhx(MPa): {p_rhx_v:.4}\")\n");
        code.push_str("\t\tprint(f\"      加热蒸汽进口干度X-rhx: {x_rhx_v:.4}\")\n");
        code.push_str("\t\tprint(f\"      加热蒸汽进口温度T-rhx(°C): {t_rhx_v:.4}\")\n");
        code.push_str("\t\tprint(f\"      加热蒸汽进口比焓h-rhx(kJ/kg): {h_rhx_v:.4}\")\n");
        code.push_str("\t\tprint(f\"      疏水比焓h-zsx(kJ/kg): {h_zsx_v:.4}\")\n");
        code.push_str("\n");

        code.push_str("\n# --- 计算结束 ---\n\n");
        code.push_str("\n");
        code.push_str("if __name__ == \"__main__\":\n");
        code.push_str("\tmain()\n");

        code
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
