use std::error;

use tempfile::NamedTempFile;

use super::data::IRIS_DATASET_LINK;
use std::io::Write;

pub struct ContentFilePair(pub String, pub NamedTempFile);

pub async fn get_iris_content() -> Result<ContentFilePair, Box<dyn error::Error>> {
    let tmp_file = NamedTempFile::new()?;
    let response = reqwest::get(IRIS_DATASET_LINK).await?;
    let content = response.text().await?;
    writeln!(&tmp_file, "{}", &content)?;

    Ok(ContentFilePair(content, tmp_file))
}
