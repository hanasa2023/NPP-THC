use super::errors;
use rfd::AsyncFileDialog;

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
        .map_err(|error| errors::Error::IoError(error.kind()))?;
    let input_params =
        serde_json::from_str(&contents).map_err(|_| errors::Error::JsonParseError)?;

    Ok(input_params)
}
