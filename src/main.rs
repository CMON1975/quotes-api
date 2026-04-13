mod auth;
mod config;
mod db;
mod errors;
mod models;
mod routes;

fn main() {
    let config = config::Config::load();
    println!("Loaded config for {}:{}", config.host, config.port);
}
