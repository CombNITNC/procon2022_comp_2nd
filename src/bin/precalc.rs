use std::{
    fs::{create_dir_all, OpenOptions},
    path::PathBuf,
};

use log::info;
use procon2022_comp_2nd::{load_all_jk, precalculate};

fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    env_logger::init();

    let all_jk = load_all_jk()?;
    let database = precalculate(&all_jk);
    let database_dir: PathBuf = ["assets", "database"].into_iter().collect();
    create_dir_all(database_dir)?;

    for (using_voice, dict) in database {
        let path: PathBuf = [
            "assets".into(),
            "database".into(),
            format!("{}.json", using_voice),
        ]
        .into_iter()
        .collect();
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)?;
        serde_json::to_writer(file, &dict)?;
        info!("written to: {}", path.display());
    }
    Ok(())
}
