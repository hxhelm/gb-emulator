use std::fs::{self, File};
use std::io::{copy, BufWriter};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Once;

const DOWNLOAD_URL: &str =
    "https://github.com/c-sp/game-boy-test-roms/releases/download/v7.0/game-boy-test-roms-v7.0.zip";
const DOWNLOAD_PATH: &str = "data/test-roms";

static INIT: Once = Once::new();

fn download_test_roms() {
    if fs::exists("data/test-roms").unwrap() {
        println!("Test roms already present, skipping Download...");
        return;
    }

    let data_dir = PathBuf::from_str("data").unwrap();
    if !data_dir.exists() {
        fs::create_dir(data_dir).unwrap();
    }

    println!("Downloading test roms...");

    let mut response = ureq::get(DOWNLOAD_URL).call().unwrap();

    let zip_path = format!("{}.zip", DOWNLOAD_PATH);
    let mut output = BufWriter::new(fs::File::create(&zip_path).unwrap());

    copy(&mut response.body_mut().as_reader(), &mut output).unwrap();

    let zip_file = fs::File::open(zip_path).unwrap();

    extract_zip_archive(zip_file);
}

fn extract_zip_archive(file: File) {
    let mut archive = zip::ZipArchive::new(file).unwrap();

    let target_dir = PathBuf::from_str(DOWNLOAD_PATH).unwrap();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => target_dir.join(path),
            None => continue,
        };

        if file.is_dir() {
            fs::create_dir_all(&outpath).unwrap();
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).unwrap();
                }
            }
            let mut outfile = fs::File::create(&outpath).unwrap();
            copy(&mut file, &mut outfile).unwrap();
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).unwrap();
            }
        }
    }
}

pub fn initialize() {
    INIT.call_once(|| {
        download_test_roms();
    });
}
