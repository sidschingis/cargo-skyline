use crate::game_paths::{get_subsdk_path, get_npdm_path, get_plugin_nro_path};
use crate::error::{Error, Result};
use crate::cargo_info;
use crate::build;
use std::io::{Read, Cursor, Write};
use std::result::Result as StdResult;
use zip::{ZipArchive, ZipWriter};
use std::fs;

pub struct Exefs {
    pub main_npdm: Vec<u8>,
    pub subsdk1: Vec<u8>,
}

// TODO: Cache exefs to disk, figure out some strategy for cache invalidation?
pub fn get_exefs(url: &str) -> Result<Exefs> {
    let zip_response = reqwest::blocking::get(url).map_err(|_| Error::DownloadError)?;
    let zip_reader = Cursor::new(Read::bytes(zip_response).collect::<StdResult<Vec<_>, _>>()?);

    let mut zip = ZipArchive::new(zip_reader).unwrap();

    let subsdk1 = zip.by_name("exefs/subsdk1")?.bytes().collect::<StdResult<_, _>>()?;
    let main_npdm = zip.by_name("exefs/main.npdm")?.bytes().collect::<StdResult<_, _>>()?;

    Ok(Exefs {
        main_npdm,
        subsdk1,
    })
}

pub fn package(skyline_url: &str, title_id: Option<&str>, out_path: &str) -> Result<()> {
    let args = vec![String::from("--release")];
    let nro_path = build::build_get_nro(args)?;
    let plugin_name = nro_path.file_name().unwrap().to_string_lossy();
    println!("Built {:?}!", plugin_name);

    let metadata = cargo_info::get_metadata()?;

    let title_id =
            title_id.or_else(|| metadata.title_id.as_deref())
                    .ok_or(Error::NoTitleId)?;

    println!("Downloading latest Skyline release...");
    let exefs = get_exefs(skyline_url)?;
    
    println!("Building Zip File...");
    let plugin_data = fs::read(&nro_path)?;
    
    let mut zip = ZipWriter::new(fs::File::create(out_path)?);

    zip.start_file(get_plugin_nro_path(title_id, plugin_name.as_ref()), Default::default())?;
    zip.write_all(&plugin_data)?;

    zip.start_file(get_npdm_path(title_id), Default::default())?;
    zip.write_all(&exefs.main_npdm)?;

    zip.start_file(get_subsdk_path(title_id), Default::default())?;
    zip.write_all(&exefs.subsdk1)?;

    Ok(())
}
