//! Environment variable `LANG` is used to determine
//! the language of the program.

pub enum Language {
    En,
    Zh,
}

impl Language {
    pub fn get() -> Self {
        match std::env::var("LANG") {
            Ok(lang) if lang.starts_with("zh") => Language::Zh,
            _ => Language::En,
        }
    }
}

macro_rules! invoke {
    ($($id:ident => $en:expr, $zh:expr),*$(,)?) => {
        pub enum Lang {
            $($id),*
        }

        impl Lang {
            pub fn get(&self) -> &str {
                match *self {
                    $(Lang::$id => match Language::get() {
                        Language::En => $en,
                        Language::Zh => $zh,
                    })*
                }
            }
        }
    }
}

invoke! {
    ErrorOccurred => "An error occurred", "发生了一个错误",
    UpdaterStarted => "Ani app updater started", "Ani 程序更新器已启动",
    Usage => "Usage: {} <archive> [extract_dir]", "用法: {} <压缩档案> [提取目录]",
    TooManyArguments => "Too many arguments, ignore", "参数过多，忽略",
    FailedToGetCurrentDir => "Failed to get the current directory", "无法获取当前目录",
    WaitAniAppExit => "Wait for Ani app to exit", "等待 Ani 程序退出",
    AniAppIsClosed => "Ani app is closed", "Ani 程序已关闭",
    ExtractTo => "Extracting {} to {}", "正在提取 {} 到 {}",
    ExtractFailed => "Failed to extract the archive due to {}", "无法提取压缩档案由于 {}",
    Extracting => "Extracting {}", "正在提取 {}",
    WaitBlock => "Wait for the blocked files to be released: {}", "等待被锁定的文件被释放: {}",
    AniRemoved => "Ani app has been removed", "Ani 程序已被移除",
    RemoveFailed => "Failed to remove the file", "无法移除文件",
    CreateFileFailed => "Failed to create the file", "无法创建文件",
    CreateDirFailed => "Failed to create the directory", "无法创建目录",
    WriteFailed => "Failed to write the file", "无法写入文件",
    UpdateCompleted => "Ani app update completed! :D", "Ani 程序更新完成! :D",
    StartAniApp => "Wait for Ani app to start", "等待 Ani 程序启动",
    StartAniFailed => "Failed to start the Ani app", "启动 Ani 程序失败",
    End => "Press <Enter>/<Return> to exit...", "按 <Enter> 或 <Return> 键关闭窗口...",
    PermissionDenied => "Permission denied, try to change the file permission", "访问被拒绝，尝试修改文件权限",
}

/// A helper macro to get the language string.
#[macro_export]
macro_rules! lang {
    ($id:ident) => {
        Lang::$id.get()
    };
}