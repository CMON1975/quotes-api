use dotenvy::dotenv;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub api_key: String,
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn load() -> Self {
        dotenv().ok();

        Self {
            database_url: required("DATABASE_URL"),
            api_key: required("API_KEY"),
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .expect("PORT must be a valid number"),
        }
    }
}

fn required(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("{key} must be set in environment"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn clear_env() {
        unsafe {
            env::remove_var("DATABASE_URL");
            env::remove_var("API_KEY");
            env::remove_var("HOST");
            env::remove_var("PORT");
        }
    }

    #[test]
    fn loads_all_required_fields() {
        clear_env();
        unsafe {
            env::set_var("DATABASE_URL", "sqlite:test.db");
            env::set_var("API_KEY", "testkey");
            env::set_var("HOST", "0.0.0.0");
            env::set_var("PORT", "8080");
        }

        let config = Config::load();

        assert_eq!(config.database_url, "sqlite:test.db");
        assert_eq!(config.api_key, "testkey");
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn uses_default_host_when_absent() {
        clear_env();
        unsafe {
            env::set_var("DATABASE_URL", "sqlite:test.db");
            env::set_var("API_KEY", "testkey");
        }

        let config = Config::load();

        assert_eq!(config.host, "127.0.0.1");

        #[test]
        fn uses_default_port_when_absent() {
            clear_env();
            unsafe {
                env::set_var("DATABASE_URL", "sqlite:test.db");
                env::set_var("API_KEY", "testkey");
            }

            let config = Config::load();

            assert_eq!(config.port, 3000)
        }

        #[test]
        #[should_panic(expected = "DATABASE_URL must be set")]
        fn panics_when_database_url_missing() {
            clear_env();
            unsafe {
                env::set_var("API_KEY", "testkey");
            }

            Config::load();
        }

        #[test]
        #[should_panic(expected = "API_KEY must be set")]
        fn panics_when_api_key_missing() {
            clear_env();
            unsafe {
                env::set_var("DATABASE_URL", "sqlite:test.db");
            }

            Config::load();
        }
    }
}
