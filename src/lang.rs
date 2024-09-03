//! Environment variable `LANG` is used to determine
//! the language of the program.

use cfg_if::cfg_if;
use ani_update_macro::invoke;

pub enum Language {
    En,
    Zh,
}

cfg_if!(
    if #[cfg(target_os = "windows")] {
        extern "C" {
            fn GetSystemDefaultUILanguage() -> u16;
        }

        fn get_env_lang() -> Option<String> {
            let lang_id = unsafe { GetSystemDefaultUILanguage() };
            match lang_id {
                0x0804 => Some("zh-CN".to_string()),
                0x0409 => Some("en-US".to_string()),
                _ => None,
            }
        }
    } else {
        fn get_env_lang() -> Option<String> {
            std::env::var("LANG").ok()
        }
    }
);

impl Language {
    pub fn get() -> Self {
        match get_env_lang() {
            Some(lang) if lang.starts_with("zh") => Language::Zh,
            _ => Language::En,
        }
    }
}


#[macro_export]
macro_rules! generate_macro {
    ($($id:ident => $en:expr, $zh:expr),*) => {
        $(
            invoke!($id => $en, $zh);
        )*
    }
}

generate_macro! {
    error_occurred => "An error occurred with {}", "发生了一个错误由于 {}",
    updater_started => "Ani app updater started", "Ani 程序更新器已启动",
    usage => "Usage: {} <archive> [extract_dir]", "用法: {} <压缩档案> [提取目录]",
    too_many_args => "Too many arguments, ignore {}", "参数过多，忽略 {}",
    failed_get_current_dir => "Failed to get the current directory due to: {}", "无法获取当前目录由于: {}",
    wait_app_exit => "Wait for Ani app to exit", "等待 Ani 程序退出",
    ani_closed => "Ani app is closed", "Ani 程序已关闭",
    extract_to => "Extracting {} to {}", "正在提取 {} 到 {}",
    extract_failed => "Failed to extract the archive due to {}", "无法提取压缩档案由于 {}",
    extracting => "Extracting {}", "正在提取 {}",
    wait_blocked => "Wait for the blocked files to be released: {}", "等待被锁定的文件被释放: {}",
    ani_removed => "Ani app has been removed", "Ani 程序已被移除",
    remove_failed => "Failed to remove the file due to {}", "无法移除文件由于 {}",
    create_file_failed => "Failed to create the file due to {}", "无法创建文件由于 {}",
    create_dir_failed => "Failed to create the directory due to {}", "无法创建目录由于 {}",
    write_failed => "Failed to write the file due to {}", "无法写入文件由于 {}",
    update_finish => "Ani app update completed! :D", "Ani 程序更新完成! :D",
    start_ani => "Wait for Ani app to start", "等待 Ani 程序启动",
    start_ani_failed => "Failed to start the Ani app due to {}", "启动 Ani 程序失败由于 {}",
    end => "Press <Enter>/<Return> to exit...", "按 <Enter> 或 <Return> 键关闭窗口...",
    permission_denied => "Permission denied by {}, try to change the file permission", "访问被拒绝由于 {}，尝试修改文件权限"
}
