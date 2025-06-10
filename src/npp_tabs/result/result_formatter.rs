use calc::parameters::{
    CalcFWParameters, CalcHESParameters, CalcRHXParameters, CalcResultParamters,
};

/// 格式化f64值为字符串，保留4位小数
fn fmt_f64(val: f64) -> String {
    format!("{:.4}", val) // Format to 4 decimal places
}

/// 格式化`CalcFWParameters`数据
fn format_fw_params(params: &[CalcFWParameters], title: &str, markdown: &mut String) {
    if !params.is_empty() {
        markdown.push_str(&format!("#### {}\n\n", title));
        for (i, p) in params.iter().enumerate() {
            markdown.push_str(&format!("  - **参数组 {}**\n", i + 1));
            markdown.push_str(&format!(
                "    - 进口给水压力 (p_fwxi): {}\n",
                fmt_f64(p.p_fwxi)
            ));
            markdown.push_str(&format!(
                "    - 进口给水比焓 (h_fwxi): {}\n",
                fmt_f64(p.h_fwxi)
            ));
            markdown.push_str(&format!(
                "    - 进口给水温度 (T_fwxi): {}\n",
                fmt_f64(p.t_fwxi)
            ));
            markdown.push_str(&format!(
                "    - 出口给水压力 (p_fwxo): {}\n",
                fmt_f64(p.p_fwxo)
            ));
            markdown.push_str(&format!(
                "    - 出口给水比焓 (h_fwxo): {}\n",
                fmt_f64(p.h_fwxo)
            ));
            markdown.push_str(&format!(
                "    - 出口给水温度 (T_fwxo): {}\n",
                fmt_f64(p.t_fwxo)
            ));
            markdown.push_str(&format!(
                "    - 汽侧疏水温度 (T_roxk): {}\n",
                fmt_f64(p.t_roxk)
            ));
            markdown.push_str(&format!(
                "    - 汽侧疏水比焓 (h_roxk): {}\n\n",
                fmt_f64(p.h_roxk)
            ));
            markdown.push_str(&format!(
                "    - 汽侧疏水压力 (p_roxk): {}\n\n",
                fmt_f64(p.p_roxk)
            ));
        }
    }
}

/// 格式化`CalcHESParameters`数据
fn format_hes_params(params: &[CalcHESParameters], title: &str, markdown: &mut String) {
    if !params.is_empty() {
        markdown.push_str(&format!("#### {}\n\n", title));
        for (i, p) in params.iter().enumerate() {
            markdown.push_str(&format!("  - **参数组 {}**\n", i + 1));
            markdown.push_str(&format!("    - 抽汽温度 (p_hesx): {}\n", fmt_f64(p.t_hesx)));
            markdown.push_str(&format!("    - 抽汽压力 (p_hesx): {}\n", fmt_f64(p.p_hesx)));
            markdown.push_str(&format!("    - 抽汽干度 (X_hesx): {}\n", fmt_f64(p.x_hesx)));
            markdown.push_str(&format!(
                "    - 抽汽理想比焓 (h_hesxs): {}\n",
                fmt_f64(p.h_hesxs)
            ));
            markdown.push_str(&format!(
                "    - 抽汽比焓 (h_hesx): {}\n\n",
                fmt_f64(p.h_hesx)
            ));
        }
    }
}

/// 格式化`CalcRHXParameters`数据
fn format_rhx_params(params: &[CalcRHXParameters], title: &str, markdown: &mut String) {
    if !params.is_empty() {
        markdown.push_str(&format!("#### {}\n\n", title));
        for (i, p) in params.iter().enumerate() {
            markdown.push_str(&format!("  - **参数组 {}**\n", i + 1));
            markdown.push_str(&format!(
                "    - 加热蒸汽进口压力 (p_rhx): {}\n",
                fmt_f64(p.p_rhx)
            ));
            markdown.push_str(&format!(
                "    - 加热蒸汽进口干度 (X_rhx): {}\n",
                fmt_f64(p.x_rhx)
            ));
            markdown.push_str(&format!(
                "    - 加热蒸汽进口温度 (T_rhx): {}\n",
                fmt_f64(p.t_rhx)
            ));
            markdown.push_str(&format!(
                "    - 加热蒸汽进口比焓 (h_rhx): {}\n",
                fmt_f64(p.h_rhx)
            ));
            markdown.push_str(&format!(
                "    - 再热器疏水比焓 (h_zsx): {}\n\n",
                fmt_f64(p.h_zsx)
            ));
        }
    }
}

/// 格式化计算结果为Markdown格式
pub fn format_result_to_markdown(result: &CalcResultParamters) -> String {
    let mut markdown = String::new();
    markdown.push_str("# 计算结果\n\n");

    // --- Result1 ---
    markdown.push_str("## 主要热平衡计算结果\n\n");
    if result.result1.is_empty() {
        markdown.push_str("无主要计算结果。\n\n");
    }
    for (index, r1) in result.result1.iter().enumerate() {
        if result.result1.len() > 1 {
            markdown.push_str(&format!("### 结果组 {}\n\n", index + 1));
        }
        markdown.push_str(&format!(
            "- 1. 核电厂效率 (η_eNPP): {}\n",
            fmt_f64(r1.eta_enpp)
        ));
        markdown.push_str(&format!("- 2. 反应堆热功率 (Q_R): {}\n", fmt_f64(r1.q_r)));
        markdown.push_str(&format!(
            "- 3. 蒸汽发生器总蒸汽产量 (Ds): {}\n",
            fmt_f64(r1.d_s)
        ));
        markdown.push_str(&format!(
            "- 4. 汽轮机高压缸耗气量 (G_shp): {}\n",
            fmt_f64(r1.g_shp)
        ));
        markdown.push_str(&format!(
            "- 5. 汽轮机低压缸耗气量 (G_slp): {}\n",
            fmt_f64(r1.g_slp)
        ));
        markdown.push_str(&format!(
            "- 6. 第一级再热器耗气量 (G_srh1): {}\n",
            fmt_f64(r1.g_srh1)
        ));
        markdown.push_str(&format!(
            "- 7. 第二级再热器耗气量 (G_srh2): {}\n",
            fmt_f64(r1.g_srh2)
        ));
        markdown.push_str(&format!(
            "- 8. 除氧器耗气量 (G_sdea): {}\n",
            fmt_f64(r1.g_sdea)
        ));
        markdown.push_str(&format!(
            "- 9. 给水泵汽轮机耗气量 (G_sfwp): {}\n",
            fmt_f64(r1.g_sfwp)
        ));
        markdown.push_str(&format!(
            "- 10. 给水泵给水量 (G_fw): {}\n",
            fmt_f64(r1.g_fw)
        ));
        markdown.push_str(&format!(
            "- 11. 给水泵扬程 (H_fwp): {}\n",
            fmt_f64(r1.h_fwp)
        ));
        markdown.push_str(&format!(
            "- 12.1. 第七级抽汽量 (G_hes7): {}\n",
            fmt_f64(r1.g_hes7)
        ));
        markdown.push_str(&format!(
            "- 12.2. 第六级抽汽量 (G_hes6): {}\n",
            fmt_f64(r1.g_hes6)
        ));
        markdown.push_str(&format!(
            "- 13.1. 第四级抽汽量 (G_les4): {}\n",
            fmt_f64(r1.g_les4)
        ));
        markdown.push_str(&format!(
            "- 13.2. 第三级抽汽量 (G_les3): {}\n",
            fmt_f64(r1.g_les3)
        ));
        markdown.push_str(&format!(
            "- 13.3. 第二级抽汽量 (G_les2): {}\n",
            fmt_f64(r1.g_les2)
        ));
        markdown.push_str(&format!(
            "- 13.4. 第一级抽汽量 (G_les1): {}\n",
            fmt_f64(r1.g_les1)
        ));
        markdown.push_str(&format!("- 14. 凝结水量 (G_cd): {}\n", fmt_f64(r1.g_cd)));
        markdown.push_str(&format!(
            "- 15. 汽水分离器疏水量 (G_uw): {}\n",
            fmt_f64(r1.g_uw)
        ));
        markdown.push_str(&format!(
            "- 16. 一级再热器加热蒸汽量 (G_zc1): {}\n",
            fmt_f64(r1.g_zc1)
        ));
        markdown.push_str(&format!(
            "- 17. 二级再热器加热蒸汽量 (G_zc2): {}\n\n",
            fmt_f64(r1.g_zc2)
        ));
    }

    // --- Result2 ---
    markdown.push_str("## 附表参数\n\n");
    let r2 = &result.result2;

    markdown.push_str("### 附表一 (输入参数回顾)\n\n");
    markdown.push_str(&format!("- 1. 核电厂输出功率 (N_e): {}\n", fmt_f64(r2.ne)));
    markdown.push_str(&format!(
        "- 2. 一回路能量利用系数 (η_1): {}\n",
        fmt_f64(r2.eta_1)
    ));
    markdown.push_str(&format!(
        "- 3. 蒸汽发生器出口蒸汽干度 (X_fh): {}\n",
        fmt_f64(r2.x_fh)
    ));
    markdown.push_str(&format!(
        "- 4. 蒸汽发生器排污率 (ξ_d): {}\n",
        fmt_f64(r2.zeta_d)
    ));
    markdown.push_str(&format!(
        "- 5. 高压缸内效率 (η_hi): {}\n",
        fmt_f64(r2.eta_hi)
    ));
    markdown.push_str(&format!(
        "- 6. 低压缸内效率 (η_li): {}\n",
        fmt_f64(r2.eta_li)
    ));
    markdown.push_str(&format!(
        "- 7. 汽轮机组机械效率 (η_m): {}\n",
        fmt_f64(r2.eta_m)
    ));
    markdown.push_str(&format!("- 8. 发电机效率 (η_ge): {}\n", fmt_f64(r2.eta_ge)));
    markdown.push_str(&format!("- 9. 新蒸汽压损 (Δp_fh): {}\n", fmt_f64(r2.dp_fh)));
    markdown.push_str(&format!(
        "- 10. 再热蒸汽压损 (Δp_rh): {}\n",
        fmt_f64(r2.dp_rh)
    ));
    markdown.push_str(&format!(
        "- 11. 回热蒸汽压损 (Δp_ej): {}\n",
        fmt_f64(r2.dp_ej)
    ));
    markdown.push_str(&format!(
        "- 12. 低压缸排气压损 (Δp_cd): {}\n",
        fmt_f64(r2.dp_cd)
    ));
    markdown.push_str(&format!(
        "- 13. 高压给水加热器出口端差 (θ_hu): {}\n",
        fmt_f64(r2.theta_hu)
    ));
    markdown.push_str(&format!(
        "- 14. 低压给水加热器出口端差 (θ_lu): {}\n",
        fmt_f64(r2.theta_lu)
    )); // Corrected from θ_hu
    markdown.push_str(&format!("- 15. 加热器效率 (η_h): {}\n", fmt_f64(r2.eta_h)));
    markdown.push_str(&format!(
        "- 16. 给水泵效率 (η_fwpp): {}\n",
        fmt_f64(r2.eta_fwpp)
    ));
    markdown.push_str(&format!(
        "- 17. 给水泵汽轮机内效率 (η_fwpti): {}\n",
        fmt_f64(r2.eta_fwpti)
    ));
    markdown.push_str(&format!(
        "- 18. 给水泵汽轮机机械效率 (η_fwptm): {}\n",
        fmt_f64(r2.eta_fwptm)
    ));
    markdown.push_str(&format!(
        "- 19. 给水泵汽轮机减速器效率 (η_fwptg): {}\n",
        fmt_f64(r2.eta_fwptg)
    ));
    markdown.push_str(&format!(
        "- 20. 循环冷却水进口温度 (T_sw1): {}\n\n",
        fmt_f64(r2.t_sw1)
    ));

    markdown.push_str("### 附表二 (详细热力参数)\n\n");
    markdown.push_str(&format!(
        "- 1. 反应堆冷却剂系统运行压力 (p_c): {}\n",
        fmt_f64(r2.p_c)
    ));
    markdown.push_str(&format!(
        "- 2. 冷却剂压力对应的饱和温度 (T_cs): {}\n",
        fmt_f64(r2.t_cs)
    ));
    markdown.push_str(&format!(
        "- 3. 反应堆出口冷却剂过冷度 (ΔT_sub): {}\n",
        fmt_f64(r2.dt_sub)
    ));
    markdown.push_str(&format!(
        "- 4. 反应堆出口冷却剂温度 (T_co): {}\n",
        fmt_f64(r2.t_co)
    ));
    markdown.push_str(&format!(
        "- 5. 反应堆进出口冷却剂温升 (ΔT_c): {}\n",
        fmt_f64(r2.dt_c)
    ));
    markdown.push_str(&format!(
        "- 6. 反应堆进口冷却剂温度 (T_ci): {}\n",
        fmt_f64(r2.t_ci)
    ));
    markdown.push_str(&format!(
        "- 7. 蒸汽发生器饱和蒸汽压力 (p_s): {}\n",
        fmt_f64(r2.p_s)
    ));
    markdown.push_str(&format!(
        "- 8. 蒸汽发生器饱和蒸汽温度 (T_fh): {}\n",
        fmt_f64(r2.t_fh)
    ));
    markdown.push_str(&format!(
        "- 9. 一、二次侧对数平均温差 (ΔT_m): {}\n",
        fmt_f64(r2.dt_m)
    ));
    markdown.push_str(&format!(
        "- 10. 冷凝器中循环冷却水温升 (ΔT_sw): {}\n",
        fmt_f64(r2.dt_sw)
    ));
    markdown.push_str(&format!("- 11. 冷凝器传热端差 (δT): {}\n", fmt_f64(r2.dt)));
    markdown.push_str(&format!(
        "- 12. 冷凝器凝结水饱和温度 (T_cd): {}\n",
        fmt_f64(r2.t_cd)
    ));
    markdown.push_str(&format!(
        "- 13. 冷凝器的运行压力 (p_cd): {}\n",
        fmt_f64(r2.p_cd)
    ));
    markdown.push_str(&format!(
        "- 14. 高压缸进口的蒸汽压力 (p_hi): {}\n",
        fmt_f64(r2.p_hi)
    ));
    markdown.push_str(&format!(
        "- 15. 高压缸进口蒸汽干度 (X_hi): {}\n",
        fmt_f64(r2.x_hi)
    ));
    markdown.push_str(&format!(
        "- 15.1. 蒸汽发生器出口蒸汽比焓 (h_fh): {}\n",
        fmt_f64(r2.h_fh)
    ));
    markdown.push_str(&format!(
        "- 15.2. 蒸汽发生器出口蒸汽比熵 (s_fh): {}\n",
        fmt_f64(r2.s_fh)
    ));
    markdown.push_str(&format!(
        "- 15.3. 高压缸进口蒸汽比熵 (s_hi): {}\n",
        fmt_f64(r2.s_hi)
    ));
    markdown.push_str(&format!(
        "- 16. 高压缸排气压力 (p_hz): {}\n",
        fmt_f64(r2.p_hz)
    ));
    markdown.push_str(&format!(
        "- 17. 高压缸排气干度 (X_hz): {}\n",
        fmt_f64(r2.x_hz)
    ));
    markdown.push_str(&format!(
        "- 17.1. 高压缸进口蒸汽比焓 (h_hi): {}\n",
        fmt_f64(r2.h_hi)
    )); // Note: duplicate field name from 15.1 in doc, assuming this is h_hi for context
    markdown.push_str(&format!(
        "- 17.2. 高压缸出口理想比焓 (h_hzs): {}\n",
        fmt_f64(r2.h_hzs)
    ));
    markdown.push_str(&format!(
        "- 17.3. 高压缸出口蒸汽比焓 (h_hz): {}\n",
        fmt_f64(r2.h_hz)
    ));
    markdown.push_str(&format!(
        "- 18. 汽水分离器进口蒸汽压力 (p_spi): {}\n",
        fmt_f64(r2.p_spi)
    ));
    markdown.push_str(&format!(
        "- 19. 汽水分离器进口蒸汽干度 (X_spi): {}\n",
        fmt_f64(r2.x_spi)
    ));
    markdown.push_str(&format!(
        "- 19.1. 汽水分离器出口疏水压力 (p_uw): {}\n",
        fmt_f64(r2.p_uw)
    ));
    markdown.push_str(&format!(
        "- 19.2. 汽水分离器出口疏水比焓 (h_uw): {}\n\n",
        fmt_f64(r2.h_uw)
    ));

    markdown.push_str("#### 第一级再热器\n\n");
    markdown.push_str(&format!(
        "- 20. 再热蒸汽进口压力 (p_rh1i): {}\n",
        fmt_f64(r2.p_rh1i)
    ));
    markdown.push_str(&format!(
        "- 21. 再热蒸汽进口干度 (X_rh1i): {}\n",
        fmt_f64(r2.x_rh1i)
    ));
    markdown.push_str(&format!(
        "- 21.1. 一级再热器进口蒸汽比焓 (h_rh1i): {}\n",
        fmt_f64(r2.h_rh1i)
    ));
    markdown.push_str(&format!(
        "- 22. 加热蒸汽进口压力 (p_rh1hs): {}\n",
        fmt_f64(r2.p_rh1hs)
    ));
    markdown.push_str(&format!(
        "- 23. 加热蒸汽进口干度 (X_rh1hs): {}\n\n",
        fmt_f64(r2.x_rh1hs)
    ));

    markdown.push_str("#### 第二级再热器\n\n");
    markdown.push_str(&format!(
        "- 24. 再热蒸汽进口压力 (p_rh2i): {}\n",
        fmt_f64(r2.p_rh2i)
    ));
    markdown.push_str(&format!(
        "- 25. 再热蒸汽进口温度 (T_rh2i): {}\n",
        fmt_f64(r2.t_rh2i)
    ));
    markdown.push_str(&format!(
        "- 26. 再热蒸汽出口压力 (p_rh2z): {}\n",
        fmt_f64(r2.p_rh2z)
    ));
    markdown.push_str(&format!(
        "- 27. 再热蒸汽出口温度 (T_rh2z): {}\n",
        fmt_f64(r2.t_rh2z)
    ));
    markdown.push_str(&format!(
        "- 27.1. 二级再热器出口比焓 (h_rh2z): {}\n",
        fmt_f64(r2.h_rh2z)
    ));
    markdown.push_str(&format!(
        "- 27.2. 每级再热器平均焓升 (Δh_rh): {}\n",
        fmt_f64(r2.dh_rh)
    ));
    markdown.push_str(&format!(
        "- 27.3. 一级再热器出口蒸汽比焓 (h_rh1z): {}\n",
        fmt_f64(r2.h_rh1z)
    ));
    markdown.push_str(&format!(
        "- 27.4. 二级再热器进口蒸汽比焓 (h_rh2i): {}\n",
        fmt_f64(r2.h_rh2i)
    )); // Note: duplicate field name from 27.1 in doc
    markdown.push_str(&format!(
        "- 28. 加热蒸汽进口压力 (p_rh2hs): {}\n",
        fmt_f64(r2.p_rh2hs)
    ));
    markdown.push_str(&format!(
        "- 29. 加热蒸汽进口干度 (X_rh2hs): {}\n\n",
        fmt_f64(r2.x_rh2hs)
    ));

    markdown.push_str("#### 低压缸\n\n");
    markdown.push_str(&format!(
        "- 30. 进口蒸汽压力 (p_li): {}\n",
        fmt_f64(r2.p_li)
    ));
    markdown.push_str(&format!(
        "- 31. 进口蒸汽温度 (T_li): {}\n",
        fmt_f64(r2.t_li)
    ));
    markdown.push_str(&format!("- 32. 排汽压力 (p_lz): {}\n", fmt_f64(r2.p_lz)));
    markdown.push_str(&format!("- 33. 排汽干度 (X_lz): {}\n", fmt_f64(r2.x_lz)));
    markdown.push_str(&format!(
        "- 33.1. 低压缸进口蒸汽比熵 (s_li): {}\n",
        fmt_f64(r2.s_li)
    ));
    markdown.push_str(&format!(
        "- 33.2. 低压缸进口蒸汽比焓 (h_li): {}\n",
        fmt_f64(r2.h_li)
    ));
    markdown.push_str(&format!(
        "- 33.3. 低压缸出口理想比焓 (h_lzs): {}\n",
        fmt_f64(r2.h_lzs)
    ));
    markdown.push_str(&format!(
        "- 33.4. 低压缸出口蒸汽比焓 (h_lz): {}\n\n",
        fmt_f64(r2.h_lz)
    ));

    markdown.push_str("#### 回热与给水系统\n\n");
    markdown.push_str(&format!("- 34. 回热级数 (Z): {}\n", fmt_f64(r2.z)));
    markdown.push_str(&format!(
        "- 35. 低压给水加热器级数 (Z_l): {}\n",
        fmt_f64(r2.z_l)
    ));
    markdown.push_str(&format!(
        "- 36. 高压给水加热器级数 (Z_h): {}\n",
        fmt_f64(r2.z_h)
    ));
    markdown.push_str(&format!(
        "- 37. 第一次给水回热分配 (Δh_fw): {}\n",
        fmt_f64(r2.dh_fw)
    ));
    markdown.push_str(&format!(
        "- 37.1. 蒸汽发生器运行压力饱和水比焓 (h_s): {}\n",
        fmt_f64(r2.h_s)
    ));
    markdown.push_str(&format!(
        "- 37.2. 冷凝器出口凝结水比焓 (h_cd): {}\n",
        fmt_f64(r2.h_cd)
    ));
    markdown.push_str(&format!(
        "- 37.3. 每级加热器理论给水焓升 (Δh_fwop): {}\n",
        fmt_f64(r2.dh_fwop)
    ));
    markdown.push_str(&format!(
        "- 37.4. 最佳给水比焓 (h_fwop): {}\n",
        fmt_f64(r2.h_fwop)
    ));
    markdown.push_str(&format!(
        "- 37.5. 最佳给水温度 (T_fwop): {}\n",
        fmt_f64(r2.t_fwop)
    ));
    markdown.push_str(&format!(
        "- 37.6. 实际给水温度 (T_fw): {}\n",
        fmt_f64(r2.t_fw)
    ));
    markdown.push_str(&format!(
        "- 37.7. 实际给水比焓 (h_fw): {}\n",
        fmt_f64(r2.h_fw)
    ));
    markdown.push_str(&format!(
        "- 38. 高压加热器给水焓升 (Δh_fwh): {}\n",
        fmt_f64(r2.dh_fwh)
    ));
    markdown.push_str(&format!(
        "- 38.1. 除氧器运行压力 (p_dea): {}\n",
        fmt_f64(r2.p_dea)
    ));
    markdown.push_str(&format!(
        "- 38.2. 除氧器出口饱和水比焓 (h_deao): {}\n",
        fmt_f64(r2.h_deao)
    ));
    markdown.push_str(&format!(
        "- 39. 除氧器及低压加热器给水焓升 (Δh_fwl): {}\n",
        fmt_f64(r2.dh_fwl)
    ));
    markdown.push_str(&format!(
        "- 39.1. 凝水泵出口给水压力 (p_cwp): {}\n",
        fmt_f64(r2.p_cwp)
    ));
    markdown.push_str(&format!(
        "- 39.2. 凝水泵出口给水比焓 (h_cwp): {}\n",
        fmt_f64(r2.h_cwp)
    ));
    markdown.push_str(&format!(
        "- 39.3. 凝水泵出口至除氧器出口的阻力压降 (Δp_cws): {}\n",
        fmt_f64(r2.dp_cws)
    ));
    markdown.push_str(&format!(
        "- 39.4. 每级低压加热器及除氧器的阻力压降 (Δp_fi): {}\n\n",
        fmt_f64(r2.dp_fi)
    ));

    format_fw_params(&r2.lfwx, "40. 低压加热器给水参数 (1 ~ 4级)", &mut markdown);

    markdown.push_str("#### 除氧器参数\n\n");
    markdown.push_str(&format!(
        "- 41. 进口给水比焓 (h_deai): {}\n",
        fmt_f64(r2.h_deai)
    ));
    markdown.push_str(&format!(
        "- 42. 出口给水比焓 (h_deao): {}\n",
        fmt_f64(r2.h_deao1)
    )); // Field name h_deao1
    markdown.push_str(&format!(
        "- 43. 出口给水温度 (T_dea): {}\n",
        fmt_f64(r2.t_dea)
    ));
    markdown.push_str(&format!(
        "- 44. 运行压力 (p_dea): {}\n\n",
        fmt_f64(r2.p_dea1)
    )); // Field name p_dea1

    markdown.push_str("#### 给水泵与高压给水系统\n\n");
    markdown.push_str(&format!(
        "- 44.1. 给水泵出口压力 (p_fwpo): {}\n",
        fmt_f64(r2.p_fwpo)
    ));
    markdown.push_str(&format!(
        "- 44.2. 给水泵出口流体比焓 (h_fwpo): {}\n",
        fmt_f64(r2.h_fwpo)
    ));
    markdown.push_str(&format!(
        "- 44.3. 蒸汽发生器进口给水压力 (p_fwi): {}\n\n",
        fmt_f64(r2.p_fwi)
    ));

    format_fw_params(&r2.hfwx, "45. 高压加热器给水参数 (6 ~ 7级)", &mut markdown);

    markdown.push_str("#### 高压缸抽汽\n\n");
    markdown.push_str(&format!(
        "- 46.1. 高压缸进口蒸汽比熵 (s_hi): {}\n",
        fmt_f64(r2.s_hi1)
    )); // Field name s_hi1
    markdown.push_str(&format!(
        "- 46.2. 高压缸进口蒸汽比焓 (h_hi): {}\n\n",
        fmt_f64(r2.h_hi1)
    )); // Field name h_hi1
    format_hes_params(&r2.hhes, "第六、七级给水加热器抽汽参数", &mut markdown);

    markdown.push_str("#### 低压缸抽汽\n\n");
    markdown.push_str(&format!(
        "- 47.1. 低压缸进口蒸汽比熵 (s_li): {}\n",
        fmt_f64(r2.s_li1)
    )); // Field name s_li1
    markdown.push_str(&format!(
        "- 47.2. 低压缸进口蒸汽比焓 (h_li): {}\n\n",
        fmt_f64(r2.h_li1)
    )); // Field name h_li1
    format_hes_params(&r2.lhes, "第一至四级给水加热器抽汽参数", &mut markdown);

    format_rhx_params(
        &r2.rhx,
        "48. 再热器抽汽 (第一、二级再热器抽汽参数)",
        &mut markdown,
    );

    markdown
}
