use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use zip::read::ZipArchive;
use cfg_if::cfg_if;
use sysinfo::System;
use colored::Colorize;
use chrono::Local;

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

macro_rules! local_time {
    () => {
        Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
    };
}

macro_rules! info { 
    ($($arg:tt)*) => {
        println!("{} {}", format!("[INFO {}]", local_time!()).bright_cyan(), format!($($arg)*).bright_cyan());
    }
}

macro_rules! success {
    ($($arg:tt)*) => {
        println!("{} {}", format!("[SUCCESS {}]", local_time!()).bright_green(), format!($($arg)*).bright_green());
    };
}

macro_rules! warn {
    ($($arg:tt)*) => {
        println!("{} {}", format!("[WARN {}]", local_time!()).bright_yellow(), format!($($arg)*).bright_yellow());
    };
}

macro_rules! error {
    ($($arg:tt)*) => {
        eprintln!("{} {}", format!("[ERROR {}]", local_time!()).bright_red(), format!($($arg)*).bright_red());
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
        return end();
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
        match std::env::current_dir().map_err(|e| e.to_string()) {
            Ok(dir) => dir,
            Err(e) => {
                error!("Failed to get the current directory: {}", e);
                return end();
            }
        }
    };

    info!("Wait for Ani app to exit");

    if wait_app_exit().is_ok() {
        info!("Ani app is closed");
    }

    info!("Extracting {} to: {}", archive_path.display(), extract_path.display());

    let try_remove = |path: &PathBuf| {
        if path.exists() {
            for _ in 0..5 {
                if let Err(err) = if path.is_file() { 
                    // std::fs::remove_file(path) 
                    remove_file_force(path)
                } else { 
                    debug_assert!(path.is_dir());

                    // std::fs::remove_dir_all(path)
                    remove_dir_filter(
                        path, 
                        std::env::current_exe()
                            .ok()
                            .as_ref()
                            .map(|p| vec![p])
                            .as_deref()
                            .unwrap_or(&[])
                    )
                } {
                    error!("{}", err.to_string());
                    info!("Wait for blocked files: {}", path.display());
                    std::thread::sleep(std::time::Duration::from_secs(1));
                } else {
                    return Ok(());
                }
            }
            Err(())
        } else {
            Ok(())
        }
    };

    // remove the old Ani app
    let app = extract_path.join(APP_NAME);
    if try_remove(&app).is_err() {
        return end();
    }

    let ico = extract_path.join("Ani.ico");
    if try_remove(&ico).is_err() {
        return end();
    }

    let app_dir = extract_path.join("app");
    if try_remove(&app_dir).is_err() {
        return end();
    }

    let runtime_dir = extract_path.join("runtime");
    if try_remove(&runtime_dir).is_err() {
        return end();
    }

    success!("Old Ani app removed");

    let files = match extract(archive_path) {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to extract : {}", e);
            return end();
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
                return end();
            }
        }

        if !path.exists() {
            if let Err(e) = std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string()) {
                error!("Failed to create the directory: {}", e);
                return end();
            }
        }

        let mut file = match File::create(&path).map_err(|e| e.to_string()) {
            Ok(f) => f,
            Err(e) => {
                error!("Failed to create the file: {}", e);
                return end();
            }
        };
        
        if let Err(e) = file.write_all(&content).map_err(|e| e.to_string()) {
            error!("Failed to write the file: {}", e);
            return end();
        }
    }

    success!("Now you can close this window and run the Ani app");

    end()
}

fn end() -> Result<(), String> {
    info!("Press <Enter>/<Return> to exit...");

    let mut s = String::new();

    std::io::stdin().read_line(&mut s).map_err(|e| e.to_string())?;

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

fn remove_dir_filter(dir: impl AsRef<Path>, filter_paths: &[impl AsRef<Path>]) -> io::Result<()> {
    let dir = dir.as_ref();
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            if filter_paths.iter().any(|p| p.as_ref() == &path) {
                continue;
            }
            if path.is_dir() {
                remove_dir_filter(&path, filter_paths)?;
            } else {
                // fs::remove_file(&path)?;
                remove_file_force(&path)?;
            }
        }
    }

    Ok(())
}

fn remove_file_force(path: &Path) -> io::Result<()> {
    fs::remove_file(path).or_else(|err| {
        if err.kind() == io::ErrorKind::PermissionDenied {
            warn!("Permission denied, try to change the file permission: {}", path.display());
            let mut perms = fs::metadata(path)?.permissions();
            perms.set_readonly(false);
            fs::set_permissions(path, perms)?;
            fs::remove_file(path)
        } else {
            Err(err)
        }
    })
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


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn change_perm() {
        let path = PathBuf::from("E:\\Ani\\test\\Ani\\Ani.exe");

        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_readonly(false);
        fs::set_permissions(&path, perms).unwrap();

        println!("{:?}", fs::metadata(&path).unwrap().permissions());


    }
}