use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use zip::read::ZipArchive;
use cfg_if::cfg_if;
use sysinfo::System;
use colored::Colorize;

cfg_if!(
    if #[cfg(target_os = "macos")] {
        const APP_NAME: &str = "Ani";
        const EXT: &str = ".dmg";
        if #[cfg(target_arch = "x86_64")] {
            const OS: &str = "macos-x86_64";
        } else if #[cfg(target_arch = "aarch64")] {
            const OS: &str = "macos-aarch64";
        }
    } else if #[cfg(target_os = "windows")] {
        const APP_NAME: &str = "Ani.exe";
    }
);

/// The default directory to extract the downloaded file
const DEFAULT_DIR: &str = ".";

macro_rules! info { 
    ($($arg:tt)*) => {
        println!("{} {}", "[INFO]".bright_cyan(), format!($($arg)*).bright_cyan());
    }
}

macro_rules! success {
    ($($arg:tt)*) => {
        println!("{} {}", "[SUCCESS]".bright_green(), format!($($arg)*).bright_green());
    };
}

macro_rules! warn {
    ($($arg:tt)*) => {
        println!("{} {}", "[WARN]".bright_yellow(), format!($($arg)*).bright_yellow());
    };
}

macro_rules! error {
    ($($arg:tt)*) => {
        eprintln!("{} {}", "[ERROR]".bright_red(), format!($($arg)*).bright_red());
    }
}

#[cfg(any(
    target_os = "windows",
    // target_os = "linux",
    target_os = "macos",
))]
fn main() -> Result<(), String> {
    std::panic::set_hook(Box::new(|panic_info| {
        error!("An error occurred: {}", panic_info);
    }));

    info!("Ani app updater started");

    let args = std::env::args().collect::<Vec<String>>();

    if args.len() < 2 {
        info!("Usage: {} <archive> [extract_dir]", args[0]);
        return Ok(());
    }

    // The .zip / .dmg file path
    let archive_path = PathBuf::from(&args[1]);

    // The directory to extract the .zip / .dmg file
    let extract_path = if args.len() > 2 {
        if args.len() > 3 {
            warn!("Too many arguments, ignore `{}`", args[3..].join(" "));
        }
        PathBuf::from(&args[2])
    } else {
        PathBuf::from(DEFAULT_DIR)
    };

    info!("Wait for Ani app to exit");

    if wait_app_exit().is_ok() {
        info!("Ani app is closed");
    }

    info!("Extracting {} to: {}", archive_path.display(), extract_path.display());

    // remove the old Ani app
    let app = extract_path.join(APP_NAME);
    if app.exists() {
        if let Err(e) = std::fs::remove_file(&app).map_err(|e| e.to_string()) {
            error!("Failed to remove the old Ani app: {}", e);
            return Ok(());
        }
    }
    if PathBuf::from("Ani.ico").exists() {
        if let Err(e) = std::fs::remove_file("Ani.ico").map_err(|e| e.to_string()) {
            error!("Failed to remove the old Ani icon: {}", e);
            return Ok(());
        }
    }
    if PathBuf::from("app").exists() {
        if let Err(e) = std::fs::remove_dir_all("app").map_err(|e| e.to_string()) {
            error!("Failed to remove the old Ani app: {}", e);
            return Ok(());
        }
    }
    if PathBuf::from("runtime").exists() {
        if let Err(e) = std::fs::remove_dir_all("runtime").map_err(|e| e.to_string()) {
            error!("Failed to remove the old Ani runtime: {}", e);
            return Ok(());
        }
    }

    let files = match extract(archive_path) {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to extract : {}", e);
            return Ok(());
        }
    };

    for (name, content) in files {
        let path = extract_path.join(name);
        // Create path if not exist
        info!("Extracting: {}", path.display());
        
        // path is a file
        if path.exists() && path.is_file() {
            if let Err(e) = std::fs::remove_file(&path).map_err(|e| e.to_string()) {
                error!("Failed to remove the old file: {}", e);
                return Ok(());
            }
        }

        if !path.exists() {
            if let Err(e) = std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string()) {
                error!("Failed to create the directory: {}", e);
                return Ok(());
            }
        }

        let mut file = match File::create(&path).map_err(|e| e.to_string()) {
            Ok(f) => f,
            Err(e) => {
                error!("Failed to create the file: {}", e);
                return Ok(());
            }
        };
        
        if let Err(e) = file.write_all(&content).map_err(|e| e.to_string()) {
            error!("Failed to write the file: {}", e);
            return Ok(());
        }
    }

    success!("Now you can close this window and run the Ani app");

    Ok(())
}

fn wait_app_exit() -> Result<(), String> {
    let mut sys = System::new_all();

    loop {
        sys.refresh_all();

        let mut ok = true;

        for proc in sys.processes_by_name("Ani") {
            if proc.name() == APP_NAME {
                ok = false;
                break;
            }
        }

        if ok {
            break;
        }
    }

    Ok(())
}

#[cfg(any(
    target_os = "windows",
    // target_os = "linux",
    target_os = "macos",
))]
fn extract(path: PathBuf) -> Result<HashMap<String, Vec<u8>>, String> {
    use std::path::{MAIN_SEPARATOR, MAIN_SEPARATOR_STR};

    let mut archive = ZipArchive::new(File::open(path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let mut files = HashMap::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        if file.is_dir() {
            continue;
        }
        let mut content = Vec::new();
        file.read_to_end(&mut content).map_err(|e| e.to_string())?;
        files.insert(file.name().trim_start_matches("Ani/").replace('/', MAIN_SEPARATOR_STR).trim_end_matches(MAIN_SEPARATOR).to_string(), content);
    }

    Ok(files)
}
