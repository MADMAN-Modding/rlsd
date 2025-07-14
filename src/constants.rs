use directories::ProjectDirs;
use once_cell::sync::OnceCell;

static PROJ_DIRS: OnceCell<ProjectDirs> = OnceCell::new();

pub fn setup() {
    PROJ_DIRS
        .set(
            ProjectDirs::from(
                "com",
                "MADMAN-Modding",
                "RLSD",
            )
            .expect("Failed to create ProjectDirs"),
        )
        .unwrap();

    let _ = std::fs::create_dir_all(
        PROJ_DIRS
            .get()
            .expect("Failed to make config dir")
            .config_dir(),
    );
}


pub fn get_config_dir() -> String {
    let proj_dir = PROJ_DIRS.get().expect("ProjectDirs is not initialized :(");

    proj_dir.config_dir();

    let config_dir = ProjectDirs::config_dir(&proj_dir).to_str().unwrap();

    return config_dir.to_string();
}

pub fn get_config_json_path() -> String {
    format!("{}/config.json", get_config_dir())
}

/// A module for different conversions
pub mod conversions {
    /// This module is to be used when dividing from bytes to another unit such as mebibytes or gibibytes
    pub mod byte {
        /// B -> KiB (1024^1) 
        pub const KIBIBYTE: f64 = 1024.0;
        /// B -> MiB (1024^2) 
        pub const MEBIBYTE: f64 = 1024.0 * 1024.0;
        /// B -> GiB (1024^3)
        pub const GIBIBYTE: f64 = 1024.0 * 1024.0 * 1024.0;
        /// B -> TiB (1024^4)
        pub const TEBIBYTE: f64 = 1024.0 * 1024.0 * 1024.0 * 1024.0;
    }
}