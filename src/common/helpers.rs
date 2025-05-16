use super::errors;
use rfd::AsyncFileDialog;

pub async fn select_output_dir() -> Result<String, errors::Error> {
    let handle = AsyncFileDialog::new()
        .set_title("选择输出目录")
        .pick_folder()
        .await
        .ok_or(errors::Error::DialogClosed)?;

    Ok(handle.path().to_string_lossy().into_owned())
}

pub async fn load_input_params_from_file(
) -> Result<Box<calc::parameters::CalcInputParameters>, errors::Error> {
    let handle = AsyncFileDialog::new()
        .set_title("选择输入参数文件")
        .add_filter("JSON", &["json"])
        .pick_file()
        .await
        .ok_or(errors::Error::DialogClosed)?;

    let path = handle.path();

    let contents = tokio::fs::read_to_string(path)
        .await
        .map_err(|_| errors::Error::Io)?;
    let input_params = serde_json::from_str(&contents).map_err(|_| errors::Error::JsonParse)?;

    Ok(input_params)
}
