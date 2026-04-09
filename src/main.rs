mod config;
mod entries;
mod render;

fn main() {
    let config = config::Config::load("config.toml").expect("failed to load config.toml");
    println!("listening on {}", config.listen);
}
