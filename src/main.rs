mod config;

use calc::Calculator;
use config::Config;
use rfd::FileDialog;
use slint::{ModelRc, PlatformError, SharedString};
use std::fs;
use std::io::Write;
use std::path::Path;

slint::include_modules!();

fn main() -> Result<(), PlatformError> {
    let app = App::new().unwrap();
    let app_handle = app.as_weak();

    // 尝试加载配置
    let config_path = "config.json";
    let config = if let Ok(config_str) = fs::read_to_string(config_path) {
        serde_json::from_str(&config_str).unwrap_or_default()
    } else {
        Config::default()
    };

    // 设置初始输出目录
    if !config.output_directory.is_empty() {
        app.set_output_directory(SharedString::from(config.output_directory.clone()));
    }

    // 选择输出目录回调
    let directory_handle = app_handle.clone();
    app.on_select_output_directory(move || {
        if let Some(app) = directory_handle.upgrade() {
            if let Some(folder) = FileDialog::new().pick_folder() {
                let folder_str = folder.to_string_lossy().to_string();
                app.set_output_directory(SharedString::from(folder_str.clone()));

                // 更新配置
                let mut config = if let Ok(config_str) = fs::read_to_string(config_path) {
                    serde_json::from_str(&config_str).unwrap_or_default()
                } else {
                    Config::default()
                };
                config.output_directory = folder_str;
                if let Ok(config_json) = serde_json::to_string_pretty(&config) {
                    if let Ok(mut file) = fs::File::create(config_path) {
                        let _ = file.write_all(config_json.as_bytes());
                    }
                }

                app.set_status(SharedString::from("已选择输出目录"));
            }
        }
    });

    // 文件对话框操作
    let file_dialog_handle = app_handle.clone();
    app.on_open_file_dialog(move |action| {
        if let Some(app) = file_dialog_handle.upgrade() {
            match action.as_str() {
                "load" => {
                    if let Some(file) = FileDialog::new()
                        .add_filter("JSON", &["json"])
                        .set_title("加载参数文件")
                        .pick_file()
                    {
                        let file_str = file.to_string_lossy().to_string();
                        app.set_loaded_file_path(SharedString::from(file_str.clone()));

                        // 更新配置
                        let mut config = if let Ok(config_str) = fs::read_to_string(config_path) {
                            serde_json::from_str(&config_str).unwrap_or_default()
                        } else {
                            Config::default()
                        };
                        config.last_file = file_str.clone();
                        if let Ok(config_json) = serde_json::to_string_pretty(&config) {
                            if let Ok(mut file) = fs::File::create(config_path) {
                                let _ = file.write_all(config_json.as_bytes());
                            }
                        }

                        // 加载参数
                        if let Ok(content) = fs::read_to_string(&file) {
                            if let Ok(params) = serde_json::from_str::<
                                calc::parameters::CalcInputParameters,
                            >(&content)
                            {
                                // 将参数设置到UI中
                                let mut input_params = app.get_input_parameters();
                                input_params.n_e = params.ne as f32;
                                input_params.n_1 = params.n_1 as f32;
                                input_params.x_fh = params.x_fh as f32;
                                input_params.zeta_d = params.zeta_d as f32;
                                input_params.n_hi = params.n_hi as f32;
                                input_params.n_li = params.n_li as f32;
                                input_params.n_m = params.n_m as f32;
                                input_params.n_ge = params.n_ge as f32;
                                input_params.dP_fh = params.dp_fh as f32;
                                input_params.dP_rh = params.dp_rh as f32;
                                input_params.dP_ej = params.dp_ej as f32;
                                input_params.dP_cd = params.dp_cd as f32;
                                input_params.dP_f = params.dp_f as f32;
                                input_params.theta_hu = params.theta_hu as f32;
                                input_params.theta_lu = params.theta_lu as f32;
                                input_params.n_h = params.n_h as f32;
                                input_params.n_fwpp = params.n_fwpp as f32;
                                input_params.n_fwpti = params.n_fwpti as f32;
                                input_params.n_fwptm = params.n_fwptm as f32;
                                input_params.n_fwptg = params.n_fwptg as f32;
                                input_params.T_sw1 = params.t_sw1 as f32;
                                input_params.P_c = params.p_c as f32;
                                input_params.dT_sub = params.dt_sub as f32;
                                input_params.dT_c = params.dt_c as f32;
                                input_params.P_s = params.p_s as f32;
                                input_params.dT_sw = params.dt_sw as f32;
                                input_params.dT = params.dt as f32;
                                input_params.dP_hz = params.dp_hz as f32;
                                input_params.T_rh2z = params.t_rh2z as f32;
                                input_params.Z = params.z as f32;
                                input_params.Z_l = params.z_l as f32;
                                input_params.Z_h = params.z_h as f32;
                                input_params.dT_fw = params.dt_fw as f32;
                                input_params.n_eNPP = params.ne_npp as f32;
                                input_params.dP_fwpo = params.dp_fwpo as f32;
                                input_params.dP_cwp = params.dp_cwp as f32;
                                input_params.g_cd = params.g_cd as f32;

                                app.set_input_parameters(input_params);
                                app.set_status(SharedString::from("已加载参数文件"));
                            } else {
                                app.set_status(SharedString::from("参数文件格式错误"));
                            }
                        } else {
                            app.set_status(SharedString::from("无法读取文件"));
                        }
                    }
                }
                _ => {}
            }
        }
    });

    // 保存参数
    let save_params_handle = app_handle.clone();
    app.on_save_parameters(move || {
        if let Some(app) = save_params_handle.upgrade() {
            let output_dir = app.get_output_directory().to_string();
            if output_dir.is_empty() {
                app.set_status(SharedString::from("请先选择输出目录"));
                return;
            }

            let path = Path::new(&output_dir);
            if !path.exists() {
                if let Err(_) = fs::create_dir_all(path) {
                    app.set_status(SharedString::from("无法创建输出目录"));
                    return;
                }
            }

            // 将UI参数转换成计算库需要的参数结构体
            let input_params = app.get_input_parameters();
            let calc_params = calc::parameters::CalcInputParameters {
                ne: input_params.n_e as f64,
                n_1: input_params.n_1 as f64,
                x_fh: input_params.x_fh as f64,
                zeta_d: input_params.zeta_d as f64,
                n_hi: input_params.n_hi as f64,
                n_li: input_params.n_li as f64,
                n_m: input_params.n_m as f64,
                n_ge: input_params.n_ge as f64,
                dp_fh: input_params.dP_fh as f64,
                dp_rh: input_params.dP_rh as f64,
                dp_ej: input_params.dP_ej as f64,
                dp_cd: input_params.dP_cd as f64,
                dp_f: input_params.dP_f as f64,
                theta_hu: input_params.theta_hu as f64,
                theta_lu: input_params.theta_lu as f64,
                n_h: input_params.n_h as f64,
                n_fwpp: input_params.n_fwpp as f64,
                n_fwpti: input_params.n_fwpti as f64,
                n_fwptm: input_params.n_fwptm as f64,
                n_fwptg: input_params.n_fwptg as f64,
                t_sw1: input_params.T_sw1 as f64,
                p_c: input_params.P_c as f64,
                dt_sub: input_params.dT_sub as f64,
                dt_c: input_params.dT_c as f64,
                p_s: input_params.P_s as f64,
                dt_sw: input_params.dT_sw as f64,
                dt: input_params.dT as f64,
                dp_hz: input_params.dP_hz as f64,
                t_rh2z: input_params.T_rh2z as f64,
                z: input_params.Z as f64,
                z_l: input_params.Z_l as f64,
                z_h: input_params.Z_h as f64,
                dt_fw: input_params.dT_fw as f64,
                ne_npp: input_params.n_eNPP as f64,
                dp_fwpo: input_params.dP_fwpo as f64,
                dp_cwp: input_params.dP_cwp as f64,
                g_cd: input_params.g_cd as f64,
            };

            // 创建一个计算器实例，只用于保存参数
            let calculator = Calculator::new(calc_params);

            // 保存参数到文件
            match calculator.save_parameters_to_file(&output_dir) {
                Ok(_) => {
                    app.set_status(SharedString::from("参数已保存"));
                }
                Err(_) => {
                    app.set_status(SharedString::from("保存参数失败"));
                }
            }
        }
    });

    // 加载默认值
    let load_defaults_handle = app_handle.clone();
    app.on_load_defaults(move || {
        if let Some(app) = load_defaults_handle.upgrade() {
            // 默认参数值
            let default_params = InputParameters {
                P_c: 15.0,
                P_s: 5.0,
                T_rh2z: 15.0,
                T_sw1: 24.0,
                Z: 7.0,
                Z_h: 4.0,
                Z_l: 2.0,
                dP_cd: 5.0,
                dP_cwp: 3.1,
                dP_ej: 4.0,
                dP_f: 1.0,
                dP_fh: 5.0,
                dP_fwpo: 1.2,
                dP_hz: 13.0,
                dP_rh: 8.0,
                dT: 5.0,
                dT_c: 35.0,
                dT_fw: 85.0,
                dT_sub: 15.0,
                dT_sw: 7.0,
                g_cd: 1200.0,
                n_1: 99.6,
                n_e: 1000.0,
                n_eNPP: 100.0,
                n_fwpp: 58.0,
                n_fwptg: 98.0,
                n_fwpti: 80.0,
                n_fwptm: 90.0,
                n_ge: 99.0,
                n_h: 98.0,
                n_hi: 82.07,
                n_li: 83.59,
                n_m: 98.5,
                theta_hu: 3.0,
                theta_lu: 2.0,
                x_fh: 99.75,
                zeta_d: 1.05,
            };

            app.set_input_parameters(default_params);
            app.set_status(SharedString::from("已加载默认参数"));
        }
    });

    // 计算功能
    let calculate_handle = app_handle.clone();
    app.on_calculate(move || {
        if let Some(app) = calculate_handle.upgrade() {
            let output_dir = app.get_output_directory().to_string();
            if output_dir.is_empty() {
                app.set_status(SharedString::from("请先选择输出目录"));
                return;
            }

            // 将UI中的输入参数转换成计算库需要的参数结构体
            let input_params = app.get_input_parameters();
            let calc_params = calc::parameters::CalcInputParameters {
                ne: input_params.n_e as f64,
                n_1: (input_params.n_1 / 100.0) as f64,
                x_fh: (input_params.x_fh / 100.0) as f64,
                zeta_d: (input_params.zeta_d / 100.0) as f64,
                n_hi: (input_params.n_hi / 100.0) as f64,
                n_li: (input_params.n_li / 100.0) as f64,
                n_m: (input_params.n_m / 100.0) as f64,
                n_ge: (input_params.n_ge / 100.0) as f64,
                dp_fh: (input_params.dP_fh / 100.0) as f64,
                dp_rh: (input_params.dP_rh / 100.0) as f64,
                dp_ej: (input_params.dP_ej / 100.0) as f64,
                dp_cd: (input_params.dP_cd / 100.0) as f64,
                dp_f: (input_params.dP_f / 100.0) as f64,
                theta_hu: input_params.theta_hu as f64,
                theta_lu: input_params.theta_lu as f64,
                n_h: (input_params.n_h / 100.0) as f64,
                n_fwpp: (input_params.n_fwpp / 100.0) as f64,
                n_fwpti: (input_params.n_fwpti / 100.0) as f64,
                n_fwptm: (input_params.n_fwptm / 100.0) as f64,
                n_fwptg: (input_params.n_fwptg / 100.0) as f64,
                t_sw1: input_params.T_sw1 as f64,
                p_c: input_params.P_c as f64,
                dt_sub: input_params.dT_sub as f64,
                dt_c: input_params.dT_c as f64,
                p_s: input_params.P_s as f64,
                dt_sw: input_params.dT_sw as f64,
                dt: input_params.dT as f64,
                dp_hz: (input_params.dP_hz / 100.0) as f64,
                t_rh2z: input_params.T_rh2z as f64,
                z: input_params.Z as f64,
                z_l: input_params.Z_l as f64,
                z_h: input_params.Z_h as f64,
                dt_fw: (input_params.dT_fw / 100.0) as f64,
                ne_npp: (input_params.n_eNPP / 100.0) as f64,
                dp_fwpo: input_params.dP_fwpo as f64,
                dp_cwp: input_params.dP_cwp as f64,
                g_cd: input_params.g_cd as f64,
            };

            app.set_status(SharedString::from("计算中..."));

            // 创建一个计算器实例
            let mut calculator = Calculator::new(calc_params);

            // 执行计算
            match calculator.calculate() {
                Ok(_) => {
                    // 保存结果
                    match calculator.save_results_to_file(&output_dir) {
                        Ok(_) => {
                            // 保存参数
                            match calculator.save_parameters_to_file(&output_dir) {
                                Ok(_) => {
                                    // 计算成功后更新UI显示的结果数据
                                    let result = if let Some(rest) = calculator.get_results() {
                                        rest
                                    } else {
                                        app.set_status(SharedString::from(
                                            "计算完成，但无结果数据",
                                        ));
                                        return;
                                    };

                                    let result1_vec: Vec<Result1> = result
                                        .result1
                                        .iter()
                                        .map(|r| Result1 {
                                            eta_enpp: r.eta_enpp as f32,
                                            q_r: r.q_r as f32,
                                            d_s: r.d_s as f32,
                                            g_shp: r.g_shp as f32,
                                            g_slp: r.g_slp as f32,
                                            g_srh1: r.g_srh1 as f32,
                                            g_srh2: r.g_srh2 as f32,
                                            g_sdea: r.g_sdea as f32,
                                            g_sfwp: r.g_sfwp as f32,
                                            g_fw: r.g_fw as f32,
                                            h_fwp: r.h_fwp as f32,
                                            g_hes7: r.g_hes7 as f32,
                                            g_hes6: r.g_hes6 as f32,
                                            g_les4: r.g_les4 as f32,
                                            g_les3: r.g_les3 as f32,
                                            g_les2: r.g_les2 as f32,
                                            g_les1: r.g_les1 as f32,
                                            g_cd: r.g_cd as f32,
                                            g_uw: r.g_uw as f32,
                                            g_zc1: r.g_zc1 as f32,
                                            g_zc2: r.g_zc2 as f32,
                                        })
                                        .collect();

                                    // 转换为 Slint 的 ModelRc
                                    let result1_vec_model = slint::VecModel::from(result1_vec);
                                    let result1_model = ModelRc::new(result1_vec_model);

                                    // 同样处理 result2
                                    // 这里需要转换 result.result2 的所有字段...
                                    let result2 = convert_calc_result2_to_slint(&result.result2);

                                    // 更新结果
                                    let result_params = MyResult {
                                        result1: result1_model,
                                        result2: result2,
                                    };

                                    app.set_results(result_params);
                                    app.set_status(SharedString::from("计算完成，结果已保存"));
                                }
                                Err(_) => {
                                    app.set_status(SharedString::from("计算完成，但保存参数失败"));
                                }
                            }
                        }
                        Err(_) => {
                            app.set_status(SharedString::from("计算完成，但保存结果失败"));
                        }
                    }
                }
                Err(e) => {
                    app.set_status(SharedString::from(format!("计算失败: {}", e)));
                }
            }
        }
    });

    app.run()
}

fn convert_calc_result2_to_slint(r: &calc::parameters::CalcResult2) -> Result2 {
    // 处理 FW 参数数组
    let fw_params = r
        .lfwx
        .iter()
        .map(|fw| FWParameters {
            p_fwxi: fw.p_fwxi as f32,
            h_fwxi: fw.h_fwxi as f32,
            t_fwxi: fw.t_fwxi as f32,
            p_fwxo: fw.p_fwxo as f32,
            h_fwxo: fw.h_fwxo as f32,
            t_fwxo: fw.t_fwxo as f32,
            t_roxk: fw.t_roxk as f32,
            h_roxk: fw.h_roxk as f32,
        })
        .collect::<Vec<FWParameters>>();

    // 转换为 VecModel
    let lfwx = slint::VecModel::from(fw_params);

    // 同样处理 hfwx，hhes，lhes 和 rhx
    let hfwx = slint::VecModel::from(
        r.hfwx
            .iter()
            .map(|fw| FWParameters {
                // 映射字段
                p_fwxi: fw.p_fwxi as f32,
                h_fwxi: fw.h_fwxi as f32,
                t_fwxi: fw.t_fwxi as f32,
                p_fwxo: fw.p_fwxo as f32,
                h_fwxo: fw.h_fwxo as f32,
                t_fwxo: fw.t_fwxo as f32,
                t_roxk: fw.t_roxk as f32,
                h_roxk: fw.h_roxk as f32,
            })
            .collect::<Vec<FWParameters>>(),
    );

    // 以相同的方式处理其他数组
    let hhes = slint::VecModel::from(
        r.hhes
            .iter()
            .map(|h| HESParameters {
                // 映射所有 HESParameters 字段
                h_hesx: h.h_hesx as f32,
                h_hesxs: h.h_hesxs as f32,
                p_hesx: h.p_hesx as f32,
                x_hesx: h.x_hesx as f32,
            })
            .collect::<Vec<HESParameters>>(),
    );

    let lhes = slint::VecModel::from(
        r.lhes
            .iter()
            .map(|h| HESParameters {
                // 映射所有 HESParameters 字段
                h_hesx: h.h_hesx as f32,
                h_hesxs: h.h_hesxs as f32,
                p_hesx: h.p_hesx as f32,
                x_hesx: h.x_hesx as f32,
            })
            .collect::<Vec<HESParameters>>(),
    );

    let rhx = slint::VecModel::from(
        r.rhx
            .iter()
            .map(|h| RHXParameters {
                // 映射所有 RHXParameters 字段
                h_rhx: h.h_rhx as f32,
                h_zsx: h.h_zsx as f32,
                p_rhx: h.p_rhx as f32,
                x_rhx: h.x_rhx as f32,
                t_rhx: h.t_rhx as f32,
            })
            .collect::<Vec<RHXParameters>>(),
    );

    // 返回带有所有字段的 Result2 结构体
    Result2 {
        // 所有标量字段
        ne: r.ne as f32,
        eta_1: r.eta_1 as f32,
        x_fh: r.x_fh as f32,
        zeta_d: r.zeta_d as f32,
        eta_hi: r.eta_hi as f32,
        eta_li: r.eta_li as f32,
        eta_m: r.eta_m as f32,
        eta_ge: r.eta_ge as f32,
        dp_fh: r.dp_fh as f32,
        dp_rh: r.dp_rh as f32,
        dp_ej: r.dp_ej as f32,
        dp_cd: r.dp_cd as f32,
        theta_hu: r.theta_hu as f32,
        theta_lu: r.theta_lu as f32,
        eta_h: r.eta_h as f32,
        eta_fwpp: r.eta_fwpp as f32,
        eta_fwpti: r.eta_fwpti as f32,
        eta_fwptm: r.eta_fwptm as f32,
        eta_fwptg: r.eta_fwptg as f32,
        t_sw1: r.t_sw1 as f32,
        p_c: r.p_c as f32,
        t_cs: r.t_cs as f32,
        dt_sub: r.dt_sub as f32,
        t_co: r.t_co as f32,
        dt_c: r.dt_c as f32,
        t_ci: r.t_ci as f32,
        p_s: r.p_s as f32,
        t_fh: r.t_fh as f32,
        dt_m: r.dt_m as f32,
        dt_sw: r.dt_sw as f32,
        dt: r.dt as f32,
        t_cd: r.t_cd as f32,
        p_cd: r.p_cd as f32,
        p_hi: r.p_hi as f32,
        x_hi: r.x_hi as f32,
        h_fh: r.h_fh as f32,
        s_fh: r.s_fh as f32,
        s_hi: r.s_hi as f32,
        p_hz: r.p_hz as f32,
        x_hz: r.x_hz as f32,
        h_hi: r.h_hi as f32,
        h_hzs: r.h_hzs as f32,
        h_hz: r.h_hz as f32,
        p_spi: r.p_spi as f32,
        x_spi: r.x_spi as f32,
        p_uw: r.p_uw as f32,
        h_uw: r.h_uw as f32,
        p_rh1i: r.p_rh1i as f32,
        x_rh1i: r.x_rh1i as f32,
        p_rh1hs: r.p_rh1hs as f32,
        x_rh1hs: r.x_rh1hs as f32,
        p_rh2i: r.p_rh2i as f32,
        t_rh2i: r.t_rh2i as f32,
        p_rh2z: r.p_rh2z as f32,
        t_rh2z: r.t_rh2z as f32,
        h_rh2z: r.h_rh2z as f32,
        dh_rh: r.dh_rh as f32,
        h_rh1z: r.h_rh1z as f32,
        h_rh2i: r.h_rh2i as f32,
        p_rh2hs: r.p_rh2hs as f32,
        x_rh2hs: r.x_rh2hs as f32,
        p_li: r.p_li as f32,
        t_li: r.t_li as f32,
        p_lz: r.p_lz as f32,
        x_lz: r.x_lz as f32,
        s_li: r.s_li as f32,
        h_li: r.h_li as f32,
        h_lzs: r.h_lzs as f32,
        h_lz: r.h_lz as f32,
        z_h: r.z_h as f32,
        z_l: r.z_l as f32,
        dh_fw: r.dh_fw as f32,
        h_s: r.h_s as f32,
        h_cd: r.h_cd as f32,
        h_hi1: r.h_hi1 as f32,
        h_li1: r.h_li1 as f32,
        h_rh1i: r.h_rh1i as f32,
        z: r.z as f32,
        dh_fwop: r.dh_fwop as f32,
        h_fwop: r.h_fwop as f32,
        t_fwop: r.t_fwop as f32,
        t_fw: r.t_fw as f32,
        h_fw: r.h_fw as f32,
        dh_fwh: r.dh_fwh as f32,
        p_dea: r.p_dea as f32,
        h_deao: r.h_deao as f32,
        dh_fwl: r.dh_fwl as f32,
        p_cwp: r.p_cwp as f32,
        h_cwp: r.h_cwp as f32,
        dp_cws: r.dp_cws as f32,
        dp_fi: r.dp_fi as f32,
        h_deai: r.h_deai as f32,
        h_deao1: r.h_deao1 as f32,
        t_dea: r.t_dea as f32,
        p_dea1: r.p_dea1 as f32,
        p_fwpo: r.p_fwpo as f32,
        h_fwpo: r.h_fwpo as f32,
        p_fwi: r.p_fwi as f32,
        s_hi1: r.s_hi1 as f32,
        s_li1: r.s_li1 as f32,
        // 使用已转换为 VecModel 的数组字段
        lfwx: slint::ModelRc::new(lfwx),
        hfwx: slint::ModelRc::new(hfwx),
        hhes: slint::ModelRc::new(hhes),
        lhes: slint::ModelRc::new(lhes),
        rhx: slint::ModelRc::new(rhx),
    }
}
