use std::env;
use tracing::info;

fn load_env_file_into_env(file_path: &str) {
    info!("loading env '{}'", file_path);
    std::fs::read_to_string(file_path)
        .expect("Something went wrong reading the file")
        .lines()
        .for_each(|line| {
            let mut split = line.split("=");
            let key = split.next().unwrap();
            let value = split.next().unwrap();
            env::set_var(key, value);
        });
}

#[cfg(debug_assertions)]
pub fn load_env() {
    load_env_file_into_env(".env.development");
}

#[cfg(not(debug_assertions))]
pub fn load_env() {
    load_env_file_into_env(".env.production");
}