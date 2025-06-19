use std::sync::LazyLock;

pub const JSON_RPC_URL: &str = "http://api.devnet.gorbagana.com";

pub static CONFIG_FILE: LazyLock<Option<String>> = LazyLock::new(|| {
    dirs_next::home_dir().map(|mut path| {
        path.extend([".config", "gorbagana", "install", "config.yml"]);
        path.to_str().unwrap().to_string()
    })
});

pub static USER_KEYPAIR: LazyLock<Option<String>> = LazyLock::new(|| {
    dirs_next::home_dir().map(|mut path| {
        path.extend([".config", "gorbagana", "id.json"]);
        path.to_str().unwrap().to_string()
    })
});

pub static DATA_DIR: LazyLock<Option<String>> = LazyLock::new(|| {
    dirs_next::home_dir().map(|mut path| {
        path.extend([".local", "share", "gorbagana", "install"]);
        path.to_str().unwrap().to_string()
    })
});
