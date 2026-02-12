use std::collections::HashMap;
use std::env::current_dir;
use std::fs;
use std::net::IpAddr;
use std::path::PathBuf;

use clap::Parser;

use crate::cmd::runner::CommandOptionsValue;
use crate::utils;
use tracing_subscriber::filter::LevelFilter;

#[derive(Debug, Clone, Parser)]
#[command(about, version, author)]
pub struct CommandLine {
    /// HTTP server listen address.
    #[arg(long, default_value = "127.0.0.1", env = "MCPD_HTTP_HOST", value_parser = parse_ip_addr)]
    pub http_host: String,

    /// HTTP server listen port number.
    #[arg(long, default_value = "1995", env = "MCPD_HTTP_PORT")]
    pub http_port: u16,

    /// HTTP server base path. Currently not used!
    #[arg(long, default_value = "/", env = "MCPD_HTTP_BASE_PATH", value_parser = parse_http_base_path)]
    pub http_base_path: String,

    /// HTTP server TLS certificate file.
    ///
    /// If you configure this along with `--http-tls-key-file` option, mcpd
    /// serves everything over HTTPS.
    #[arg(long, env = "MCPD_HTTP_TLS_CERT_FILE", value_parser = parse_tls_file)]
    pub http_tls_cert_file: Option<PathBuf>,

    /// HTTP server TLS private-key file.
    ///
    /// If you configure this along with `--http-tls-cert-file` option, mcpd
    /// serves everything over HTTPS.
    #[arg(long, env = "MCPD_HTTP_TLS_KEY_FILE", value_parser = parse_tls_file)]
    pub http_tls_key_file: Option<PathBuf>,

    /// Read timeout for client connections in seconds.
    ///
    /// If a client doesn't send data within this period, the connection will be dropped.
    /// The default value is 5 seconds.
    #[arg(long, default_value = "5", env = "MCPD_HTTP_READ_TIMEOUT")]
    pub http_read_timeout_secs: u64,

    /// Write timeout for client connections in seconds.
    ///
    /// If data cannot be written to a client within this period, the connection will be dropped.
    /// The default value is 5 seconds.
    #[arg(long, default_value = "5", env = "MCPD_HTTP_WRITE_TIMEOUT")]
    pub http_write_timeout_secs: u64,

    /// HTTP server basic authentication username.
    ///
    /// You can use this `username` and configured password to get a new bearer token.
    /// If the value is empty and no password is configured, then no authentication
    /// is needed for anything. If the value is empty and password is configured, the
    /// username will be `admin`.
    #[arg(long, default_value = "", env = "MCPD_HTTP_AUTH_USERNAME")]
    pub http_auth_username: String,

    /// A file containing bcrypt(password: sha256(plain_password) of your user password.
    ///
    /// By configuring this you are able to change the password in runtime via REST API.
    /// Make sure that mcpd process has appropriate permissions to write to the file.
    /// Empty value means this option should be discarded and if one of `--http-auth-password-file`
    /// and `--http-auth-password-sha256-bcrypt` is not configured, You can call every REST API endpoint without
    /// authentication.
    #[arg(long, env = "MCPD_HTTP_AUTH_PASSWORD_FILE", value_parser = parse_password_file)]
    pub http_auth_password_file: Option<PathBuf>,

    /// bcrypt(password: sha256(plain_password), const: 12) of your user password.
    ///
    /// If `--http-auth-password-file` is configured, this is discarded.
    /// Note that by configuring this, You can not change the password via REST API or in
    /// web dashboard.
    /// Empty value means this option should be discarded and if one of `--http-auth-password-file`
    /// and `--http-auth-password-sha256-bcrypt` is not configured, You can call every REST API endpoint without
    /// authentication.
    #[arg(long, env = "MCPD_HTTP_AUTH_PASSWORD_SHA256_BCRYPT")]
    pub http_auth_password_sha256_bcrypt: Option<String>,

    /// Enable/Disable CAPTCHA.
    #[arg(long, env = "MCPD_HTTP_AUTH_CAPTCHA")]
    pub http_auth_captcha: bool,

    /// Make CAPTCHA case-sensitive
    #[arg(long, env = "MCPD_HTTP_AUTH_CAPTCHA_CASE_SENSITIVE")]
    pub http_auth_captcha_case_sensitive: bool,

    /// hardcoded HTTP bearer token that does not expire.
    ///
    /// You can use this value in your application(s) then you do not have to pass
    /// CAPTCHA each time the previous token has expired to get a new one.
    #[arg(long, env = "MCPD_HTTP_AUTH_API_TOKEN")]
    pub http_auth_api_token: Option<String>,

    /// Timeout for dynamically generated HTTP bearer tokens in seconds.
    ///
    /// The default value is 1 week.
    #[arg(long, default_value = "604800", env = "MCPD_HTTP_AUTH_TOKEN_TIMEOUT")]
    pub http_auth_token_timeout: usize,

    /// Root directory to load command files and directories and their information files.
    ///
    /// This option is required. The directory must exist and be readable.
    #[arg(long, env = "MCPD_SCRIPT_ROOT_DIRECTORY", value_parser = parse_script_root_directory)]
    pub script_root_directory: PathBuf,

    /// Configuration key/values for scripts in KEY=VALUE format (can be specified multiple times).
    ///
    /// Values must be valid JSON. These are passed to scripts via environment variables.
    #[arg(long, value_name = "KEY=VALUE", value_parser = parse_command_key_value)]
    pub script_config: Vec<(String, crate::cmd::tree::CommandOptionValue)>,

    /// Your scripts will receive below configuration key/values directly from env or stdin.
    #[arg(skip)]
    pub configuration: CommandOptionsValue,

    /// Enable trace level logging (shows target and location).
    #[arg(long)]
    pub trace: bool,

    /// Enable debug level logging (shows target).
    #[arg(long)]
    pub debug: bool,

    /// Disable all logging.
    #[arg(long)]
    pub quiet: bool,

    /// Disable the web dashboard.
    ///
    /// Default is enabled (false).
    #[arg(long, default_value = "false", env = "MCPD_WWW_UI_DISABLE")]
    pub www_ui_disable: bool,

    /// Configuration key/values for www in KEY=VALUE format (can be specified multiple times).
    ///
    /// These are accessible via the `/api/public/configuration` endpoint.
    /// Supported/Used keys:
    /// - title: Title of the web dashboard (default: mcpd)
    /// - banner-title: Title of the web dashboard banner (default: MCP Daemon)
    /// - banner-text: Text of the web dashboard banner (default: {{title}} exposes your scripts as MCP tools and resources)
    /// - footer: Footer text (default: Hosted on <a href="https://github.com/pouriya/mcpd" target="_blank"><b>GitHub</b></a>)
    #[arg(long, value_name = "KEY=VALUE", value_parser = parse_key_value)]
    pub www_config: Vec<(String, String)>,

    /// You can access below configuration key/values from REST-API `/public/configuration` endpoint.
    #[arg(skip)]
    pub www_configuration_map: HashMap<String, String>,
}

fn parse_ip_addr(s: &str) -> Result<String, String> {
    s.parse::<IpAddr>()
        .map_err(|e| format!("Could not parse hostname {:?}: {}", s, e))?;
    Ok(s.to_string())
}

fn parse_http_base_path(s: &str) -> Result<String, String> {
    if !s.starts_with("/") {
        return Err(format!(
            "Invalid HTTP base-path {:?}: HTTP base path must start with '/'",
            s
        ));
    }
    if !s.ends_with("/") {
        return Err(format!(
            "Invalid HTTP base-path {:?}: should contain '/' at the end",
            s
        ));
    }
    Ok(s.to_string())
}

fn parse_password_file(s: &str) -> Result<PathBuf, String> {
    let mut path = PathBuf::from(s);
    if path.is_relative() {
        path = current_dir()
            .map_err(|e| format!("Could not get current directory: {}", e))?
            .join(path);
    }
    // Note: We don't check if file exists here because it might be created later
    Ok(path)
}

fn parse_tls_file(s: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(s);
    if !path.is_file() {
        return Err(format!("TLS file {:?} is not found", path));
    }
    Ok(path)
}

fn parse_command_key_value(
    s: &str,
) -> Result<(String, crate::cmd::tree::CommandOptionValue), String> {
    if let Some(equal_pos) = s.find('=') {
        let key = s[..equal_pos].to_string();
        let value_str = s[equal_pos + 1..].to_string();
        let value: crate::cmd::tree::CommandOptionValue = serde_json::from_str(&value_str)
            .map_err(|e| format!("Invalid JSON value for key {}: {}", key, e))?;
        Ok((key, value))
    } else {
        Err(format!("Invalid KEY=VALUE format: {}", s))
    }
}

fn parse_key_value(s: &str) -> Result<(String, String), String> {
    if let Some(equal_pos) = s.find('=') {
        let key = s[..equal_pos].to_string();
        let value = s[equal_pos + 1..].to_string();
        Ok((key, value))
    } else {
        Err(format!("Invalid KEY=VALUE format: {}", s))
    }
}

fn parse_script_root_directory(s: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(s);
    if !path.exists() {
        return Err(format!("Script root directory {:?} does not exist", path));
    }
    if !path.is_dir() {
        return Err(format!(
            "Script root directory {:?} is not a directory",
            path
        ));
    }
    Ok(path)
}

impl CommandLine {
    pub fn logging_level(&self) -> LevelFilter {
        if self.quiet {
            LevelFilter::OFF
        } else if self.trace {
            LevelFilter::TRACE
        } else if self.debug {
            LevelFilter::DEBUG
        } else {
            LevelFilter::INFO
        }
    }

    pub fn after_parse(&mut self) -> Result<(), String> {
        // Handle password_file reading
        if let Some(password_file) = &self.http_auth_password_file {
            let password = fs::read(password_file)
                .map_err(|e| format!("Could not read password file {:?}: {}", password_file, e))?;
            let password = String::from_utf8(password)
                .map_err(|e| {
                    format!(
                        "Could not decode password file {:?} content to UTF-8: {}",
                        password_file, e
                    )
                })?
                .trim()
                .to_string();
            if password.is_empty() {
                return Err(format!("Password file {:?} is empty!", password_file));
            }
            if self.http_auth_password_sha256_bcrypt.is_some() {
                tracing::warn!(
                    msg = "Both password and password_file fields are set, ignoring password field",
                );
            }
            self.http_auth_password_sha256_bcrypt = Some(password);
        } else if let Some(ref mut password) = &mut self.http_auth_password_sha256_bcrypt {
            *password =
                utils::hash_bcrypt(utils::to_sha256(password.as_str()).as_str(), 12).unwrap();
        }

        // Handle username/password validation
        match (
            !self.http_auth_username.is_empty(),
            self.http_auth_password_sha256_bcrypt.is_some(),
            self.http_auth_password_file.is_some(),
        ) {
            (true, false, false) => {
                return Err("Configuration contains `--http-auth-username` but `--http-auth-password-file` or `--http-auth-password-sha256-bcrypt` field is not set".to_string())
            }
            (false, true, _) => {
                tracing::warn!(
                    msg = "Configuration contains password but username field is not set, using 'admin' as default username",
                );
                self.http_auth_username = "admin".to_string();
            }
            (false, _, true) => {
                tracing::warn!(
                    msg = "Configuration contains password_file but username field is not set, using 'admin' as default username",
                );
                self.http_auth_username = "admin".to_string();
            }
            _ => (),
        }

        // Handle TLS file validation
        match (
            self.http_tls_cert_file.is_some(),
            self.http_tls_key_file.is_some(),
        ) {
            (true, false) => {
                return Err("TLS cert file is set but TLS key file is not set".to_string())
            }
            (false, true) => {
                return Err("TLS key file is set but TLS cert file is not set".to_string())
            }
            _ => (),
        }

        // Convert parsed script key-value pairs into HashMap
        let mut config = HashMap::new();
        for (key, value) in &self.script_config {
            config.insert(key.clone(), value.clone());
        }
        self.configuration = config;

        // Convert parsed www key-value pairs into HashMap
        let mut www_config = HashMap::new();
        for (key, value) in &self.www_config {
            www_config.insert(key.clone(), value.clone());
        }
        self.www_configuration_map = www_config;

        Ok(())
    }
}

pub fn try_setup() -> Result<CommandLine, String> {
    let mut value = CommandLine::parse();
    value.after_parse()?;
    Ok(value)
}
