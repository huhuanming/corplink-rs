use std::collections::HashMap;
#[cfg(target_os = "macos")]
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use reqwest::Url;
use serde::Serialize;

use crate::config::Config;
use crate::template::Template;

pub const URL_GET_COMPANY: &str = "https://corplink.volcengine.cn/api/match";
pub(crate) const CORPLINK_APP_VERSION: &str = "3.2.16";
pub(crate) const CORPLINK_BUILD_NUMBER: &str = "12116";

const URL_GET_LOGIN_METHOD: &str = "{{url}}/api/login/setting?app_version={{app_version}}&brand={{brand}}&build_number={{build_number}}&client_source={{client_source}}&language={{language}}&model={{model}}&os={{os}}&os_release={{os_release}}&os_version={{version}}&soc={{soc}}&timestamp={{timestamp}}";
const URL_GET_TPS_LOGIN_METHOD: &str = "{{url}}/api/tpslogin/link?os={{os}}&os_version={{version}}";
const URL_GET_TPS_TOKEN_CHECK: &str =
    "{{url}}/api/tpslogin/token/check?os={{os}}&os_version={{version}}";
const URL_GET_CORPLINK_LOGIN_METHOD: &str = "{{url}}/api/lookup?os={{os}}&os_version={{version}}";
const URL_REQUEST_CODE: &str = "{{url}}/api/login/code/send?os={{os}}&os_version={{version}}";
const URL_VERIFY_CODE: &str = "{{url}}/api/login/code/verify?os={{os}}&os_version={{version}}";
const URL_REQUEST_CODE_V1: &str = "{{url}}/api/v1/login/send?os={{os}}&os_version={{version}}";
const URL_VERIFY_CODE_V1: &str = "{{url}}/api/v1/login/verify?os={{os}}&os_version={{version}}";
const URL_QR_TOKEN: &str = "{{url}}/api/login/token?app_version={{app_version}}&brand={{brand}}&build_number={{build_number}}&client_source={{client_source}}&language={{language}}&model={{model}}&os={{os}}&os_release={{os_release}}&os_version={{version}}&soc={{soc}}&timestamp={{timestamp}}";
const URL_AGREEMENT_SIGN: &str = "{{url}}/api/v2/agreement/sign?app_version={{app_version}}&brand={{brand}}&build_number={{build_number}}&client_source={{client_source}}&language={{language}}&model={{model}}&os={{os}}&os_release={{os_release}}&os_version={{version}}&soc={{soc}}&timestamp={{timestamp}}";
const URL_VPN_MFA_TYPE: &str = "{{url}}/api/mfa/type?app_version={{app_version}}&brand={{brand}}&build_number={{build_number}}&client_source={{client_source}}&language={{language}}&model={{model}}&os={{os}}&os_release={{os_release}}&os_version={{version}}&soc={{soc}}&timestamp={{timestamp}}";
const URL_VPN_MFA_SEND: &str = "{{url}}/api/mfa/code/send?app_version={{app_version}}&brand={{brand}}&build_number={{build_number}}&client_source={{client_source}}&language={{language}}&model={{model}}&os={{os}}&os_release={{os_release}}&os_version={{version}}&soc={{soc}}&timestamp={{timestamp}}";
const URL_VPN_MFA_VERIFY: &str = "{{url}}/api/mfa/code/verify?app_version={{app_version}}&brand={{brand}}&build_number={{build_number}}&client_source={{client_source}}&language={{language}}&model={{model}}&os={{os}}&os_release={{os_release}}&os_version={{version}}&soc={{soc}}&timestamp={{timestamp}}";
const URL_VPN_MFA_PUSH: &str = "{{url}}/api/v1/mfa/send?app_version={{app_version}}&brand={{brand}}&build_number={{build_number}}&client_source={{client_source}}&language={{language}}&model={{model}}&os={{os}}&os_release={{os_release}}&os_version={{version}}&soc={{soc}}&timestamp={{timestamp}}";
const URL_VPN_MFA_REVOKE: &str = "{{url}}/api/v1/mfa/revoke?app_version={{app_version}}&brand={{brand}}&build_number={{build_number}}&client_source={{client_source}}&language={{language}}&model={{model}}&os={{os}}&os_release={{os_release}}&os_version={{version}}&soc={{soc}}&timestamp={{timestamp}}";
const URL_LOGIN_PASSWORD: &str = "{{url}}/api/login?os={{os}}&os_version={{version}}";
const URL_LOGIN_PASSWORD_V1: &str =
    "{{url}}/api/v1/login?os={{os}}&os_version={{version}}&client_source=FeiLian";
const URL_LIST_VPN: &str = "{{url}}/api/vpn/list?app_version={{app_version}}&brand={{brand}}&build_number={{build_number}}&client_source={{client_source}}&language={{language}}&model={{model}}&os={{os}}&os_release={{os_release}}&os_version={{version}}&soc={{soc}}&timestamp={{timestamp}}";

const URL_PING_VPN_HOST: &str = "{{url}}/vpn/ping?app_version={{app_version}}&brand={{brand}}&build_number={{build_number}}&client_source={{client_source}}&language={{language}}&model={{model}}&os={{os}}&os_release={{os_release}}&os_version={{version}}&soc={{soc}}&timestamp={{timestamp}}";
const URL_FETCH_PEER_INFO: &str = "{{url}}/vpn/conn?app_version={{app_version}}&brand={{brand}}&build_number={{build_number}}&client_source={{client_source}}&language={{language}}&model={{model}}&os={{os}}&os_release={{os_release}}&os_version={{version}}&soc={{soc}}&timestamp={{timestamp}}";
const URL_OPERATE_VPN: &str = "{{url}}/vpn/report?os={{os}}&os_version={{version}}";
const URL_OTP: &str = "{{url}}/api/v2/p/otp?os={{os}}&os_version={{version}}";
// log out the current terminal so it frees the server-side session/terminal
// quota. logout_all=false only signs out this device. responds with a 302.
const URL_LOGOUT: &str = "{{url}}/api/logout?os={{os}}&os_version={{version}}&logout_all=false";

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub enum ApiName {
    LoginMethod,
    TpsLoginMethod,
    TpsTokenCheck,
    CorplinkLoginMethod,
    RequestEmailCode,
    RequestEmailCodeV1,
    LoginPassword,
    LoginPasswordV1,
    LoginEmail,
    LoginEmailV1,
    LoginQrToken,
    LoginQrCheck,
    AgreementSign,
    VpnMfaType,
    VpnMfaSend,
    VpnMfaVerify,
    VpnMfaPush,
    VpnMfaRevoke,
    ListVPN,

    PingVPN,
    ConnectVPN,
    KeepAliveVPN,
    DisconnectVPN,
    Otp,
    Logout,
}

#[derive(Clone, Serialize)]
struct UserUrlParam {
    url: String,
    os: String,
    version: String,
    app_version: String,
}

#[derive(Clone, Serialize)]
struct ListVpnUrlParam {
    url: String,
    app_version: String,
    brand: String,
    build_number: String,
    client_source: String,
    language: String,
    model: String,
    os: String,
    os_release: String,
    version: String,
    soc: String,
    timestamp: String,
}

#[derive(Clone, Serialize)]
pub struct VpnUrlParam {
    pub url: String,
    os: String,
    version: String,
}

#[derive(Clone)]
pub struct ApiUrl {
    user_param: UserUrlParam,
    list_vpn_param: ListVpnUrlParam,
    pub vpn_param: VpnUrlParam,
    api_template: HashMap<ApiName, Template>,
}

fn unix_timestamp_seconds() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default()
        .to_string()
}

#[cfg(target_os = "macos")]
fn command_output(program: &str, args: &[&str]) -> String {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_default()
}

#[derive(Clone)]
struct SystemParameters {
    brand: String,
    language: String,
    model: String,
    os: String,
    os_release: String,
    os_version: String,
    soc: String,
}

fn system_parameters() -> SystemParameters {
    #[cfg(target_os = "macos")]
    {
        return SystemParameters {
            brand: "Apple".to_string(),
            language: "zh".to_string(),
            model: command_output("/usr/sbin/sysctl", &["-n", "hw.model"]),
            os: "Mac".to_string(),
            os_release: String::new(),
            os_version: command_output("/usr/bin/sw_vers", &["-productVersion"]),
            soc: command_output("/usr/sbin/sysctl", &["-n", "machdep.cpu.brand_string"]),
        };
    }

    #[cfg(target_os = "linux")]
    {
        return SystemParameters {
            brand: String::new(),
            language: "en".to_string(),
            model: String::new(),
            os: "Linux".to_string(),
            os_release: linux_os_release(),
            os_version: linux_os_version(),
            soc: cpu_soc(),
        };
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    SystemParameters {
        brand: String::new(),
        language: "en".to_string(),
        model: String::new(),
        os: std::env::consts::OS.to_string(),
        os_release: std::env::consts::OS.to_string(),
        os_version: std::env::consts::OS.to_string(),
        soc: cpu_soc(),
    }
}

pub(crate) fn corplink_user_agent() -> String {
    let system = system_parameters();
    let goos = if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        std::env::consts::OS
    };
    format!(
        "CorpLink/{} ({}; {} {}; {})",
        CORPLINK_APP_VERSION, goos, system.os, system.os_release, system.language
    )
}

#[cfg(target_os = "linux")]
fn linux_os_release() -> String {
    if let Ok(release) = std::fs::read_to_string("/etc/os-release") {
        for line in release.lines() {
            if let Some(value) = line.strip_prefix("ID=") {
                return value.trim_matches('"').to_string();
            }
        }
    }
    "linux".to_string()
}

#[cfg(target_os = "linux")]
fn linux_os_version() -> String {
    if let Ok(release) = std::fs::read_to_string("/etc/os-release") {
        for line in release.lines() {
            if let Some(value) = line.strip_prefix("PRETTY_NAME=") {
                return value.trim_matches('"').to_string();
            }
        }
    }
    "Linux".to_string()
}

#[cfg(not(target_os = "macos"))]
fn cpu_soc() -> String {
    match std::env::consts::ARCH {
        "aarch64" => "aarch64".to_string(),
        "x86_64" => "x86_64".to_string(),
        arch => arch.to_string(),
    }
}

impl ApiUrl {
    pub fn new(conf: &Config) -> Result<ApiUrl> {
        let os = "Android".to_string();
        let version = "2".to_string();
        let system = system_parameters();
        let server_url = conf
            .server
            .clone()
            .context("server url missing in config")?;
        let mut api_template = HashMap::new();

        api_template.insert(ApiName::LoginMethod, Template::new(URL_GET_LOGIN_METHOD));
        api_template.insert(
            ApiName::TpsLoginMethod,
            Template::new(URL_GET_TPS_LOGIN_METHOD),
        );
        api_template.insert(
            ApiName::TpsTokenCheck,
            Template::new(URL_GET_TPS_TOKEN_CHECK),
        );
        api_template.insert(
            ApiName::CorplinkLoginMethod,
            Template::new(URL_GET_CORPLINK_LOGIN_METHOD),
        );
        api_template.insert(ApiName::RequestEmailCode, Template::new(URL_REQUEST_CODE));
        api_template.insert(
            ApiName::RequestEmailCodeV1,
            Template::new(URL_REQUEST_CODE_V1),
        );
        api_template.insert(ApiName::LoginEmail, Template::new(URL_VERIFY_CODE));
        api_template.insert(ApiName::LoginEmailV1, Template::new(URL_VERIFY_CODE_V1));
        api_template.insert(ApiName::LoginQrToken, Template::new(URL_QR_TOKEN));
        api_template.insert(ApiName::AgreementSign, Template::new(URL_AGREEMENT_SIGN));
        api_template.insert(ApiName::VpnMfaType, Template::new(URL_VPN_MFA_TYPE));
        api_template.insert(ApiName::VpnMfaSend, Template::new(URL_VPN_MFA_SEND));
        api_template.insert(ApiName::VpnMfaVerify, Template::new(URL_VPN_MFA_VERIFY));
        api_template.insert(ApiName::VpnMfaPush, Template::new(URL_VPN_MFA_PUSH));
        api_template.insert(ApiName::VpnMfaRevoke, Template::new(URL_VPN_MFA_REVOKE));
        api_template.insert(ApiName::LoginPassword, Template::new(URL_LOGIN_PASSWORD));
        api_template.insert(
            ApiName::LoginPasswordV1,
            Template::new(URL_LOGIN_PASSWORD_V1),
        );
        api_template.insert(ApiName::ListVPN, Template::new(URL_LIST_VPN));
        api_template.insert(ApiName::PingVPN, Template::new(URL_PING_VPN_HOST));
        api_template.insert(ApiName::ConnectVPN, Template::new(URL_FETCH_PEER_INFO));
        api_template.insert(ApiName::KeepAliveVPN, Template::new(URL_OPERATE_VPN));
        api_template.insert(ApiName::DisconnectVPN, Template::new(URL_OPERATE_VPN));
        api_template.insert(ApiName::Otp, Template::new(URL_OTP));
        api_template.insert(ApiName::Logout, Template::new(URL_LOGOUT));

        Ok(ApiUrl {
            user_param: UserUrlParam {
                url: server_url.clone(),
                os: os.clone(),
                version: version.clone(),
                app_version: CORPLINK_APP_VERSION.to_string(),
            },
            list_vpn_param: ListVpnUrlParam {
                url: server_url,
                app_version: CORPLINK_APP_VERSION.to_string(),
                brand: system.brand,
                build_number: CORPLINK_BUILD_NUMBER.to_string(),
                client_source: "FeiLian".to_string(),
                language: system.language,
                model: system.model,
                os: system.os,
                os_release: system.os_release,
                version: system.os_version,
                soc: system.soc,
                timestamp: unix_timestamp_seconds(),
            },
            vpn_param: VpnUrlParam {
                url: "".to_string(),
                os,
                version,
            },
            api_template,
        })
    }

    pub fn get_api_url(&self, name: &ApiName) -> String {
        let user_param = &self.user_param;
        let vpn_param = &self.vpn_param;
        match name {
            ApiName::LoginMethod
            | ApiName::LoginQrToken
            | ApiName::AgreementSign
            | ApiName::VpnMfaType
            | ApiName::VpnMfaSend
            | ApiName::VpnMfaVerify
            | ApiName::VpnMfaPush
            | ApiName::VpnMfaRevoke => {
                let mut params = self.list_vpn_param.clone();
                params.timestamp = unix_timestamp_seconds();
                self.api_template[name].render(&params)
            }
            ApiName::TpsLoginMethod => self.api_template[name].render(user_param),
            ApiName::TpsTokenCheck => self.api_template[name].render(user_param),
            ApiName::CorplinkLoginMethod => self.api_template[name].render(user_param),
            ApiName::RequestEmailCode => self.api_template[name].render(user_param),
            ApiName::RequestEmailCodeV1 => self.api_template[name].render(user_param),
            ApiName::LoginEmail => self.api_template[name].render(user_param),
            ApiName::LoginEmailV1 => self.api_template[name].render(user_param),
            ApiName::LoginQrCheck => unreachable!("QR check URL requires a token"),
            ApiName::LoginPassword => self.api_template[name].render(user_param),
            ApiName::LoginPasswordV1 => self.api_template[name].render(user_param),
            ApiName::ListVPN => {
                let mut list_vpn_param = self.list_vpn_param.clone();
                list_vpn_param.timestamp = unix_timestamp_seconds();
                self.api_template[name].render(&list_vpn_param)
            }
            ApiName::Otp => self.api_template[name].render(user_param),
            ApiName::Logout => self.api_template[name].render(user_param),

            ApiName::PingVPN | ApiName::ConnectVPN => {
                let mut param = self.list_vpn_param.clone();
                param.url = self.vpn_param.url.clone();
                param.timestamp = unix_timestamp_seconds();
                self.api_template[name].render(&param)
            }
            ApiName::KeepAliveVPN => self.api_template[name].render(vpn_param),
            ApiName::DisconnectVPN => self.api_template[name].render(vpn_param),
        }
    }

    pub fn get_qr_check_url(&self, token: &str) -> Result<String> {
        let mut url =
            Url::parse(&self.user_param.url).context("invalid server URL for QR login")?;
        url.set_path("/api/login/token/check");
        url.set_query(None);
        let params = &self.list_vpn_param;
        url.query_pairs_mut()
            .append_pair("app_version", &params.app_version)
            .append_pair("brand", &params.brand)
            .append_pair("build_number", &params.build_number)
            .append_pair("client_source", &params.client_source)
            .append_pair("language", &params.language)
            .append_pair("model", &params.model)
            .append_pair("os", &params.os)
            .append_pair("os_release", &params.os_release)
            .append_pair("os_version", &params.version)
            .append_pair("soc", &params.soc)
            .append_pair("timestamp", &unix_timestamp_seconds())
            .append_pair("token", token)
            .append_pair("login_scene", "feilian");
        Ok(url.into())
    }

    pub fn get_websocket_url(&self) -> Result<String> {
        let mut url = Url::parse(&self.user_param.url)
            .context("invalid server URL for push confirmation WebSocket")?;
        let websocket_scheme = match url.scheme() {
            "https" => "wss",
            "http" => "ws",
            scheme => anyhow::bail!("unsupported WebSocket base URL scheme: {scheme}"),
        };
        url.set_scheme(websocket_scheme)
            .map_err(|_| anyhow::anyhow!("failed to set WebSocket URL scheme"))?;
        url.set_path("/api/ws/socket");
        url.set_query(None);

        let params = &self.list_vpn_param;
        url.query_pairs_mut()
            .append_pair("os", &params.os)
            .append_pair("model", &params.model)
            .append_pair("brand", &params.brand)
            .append_pair("client_source", &params.client_source)
            .append_pair("app_version", &params.app_version)
            .append_pair("build_number", &params.build_number)
            .append_pair("os_version", &params.version)
            .append_pair("os_release", &params.os_release)
            .append_pair("soc", &params.soc)
            .append_pair("language", &params.language)
            .append_pair("timestamp", &unix_timestamp_seconds());
        Ok(url.into())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn list_vpn_url_matches_official_platform_shape() {
        let conf: Config = serde_json::from_value(json!({
            "company_name": "test",
            "username": "test",
            "server": "https://vpn.example.com"
        }))
        .unwrap();

        let api_url = ApiUrl::new(&conf).unwrap();
        let url = api_url.get_api_url(&ApiName::ListVPN);

        assert!(url.starts_with("https://vpn.example.com/api/vpn/list?"));
        assert!(url.contains("app_version=3.2.16"));
        assert!(url.contains("build_number=12116"));
        assert!(url.contains("client_source=FeiLian"));
        assert!(url.contains(&format!("os={}", system_parameters().os)));
        assert!(url.contains("timestamp="));
    }

    #[test]
    fn connect_vpn_url_matches_official_platform_shape() {
        let conf: Config = serde_json::from_value(json!({
            "company_name": "test",
            "username": "test",
            "server": "https://vpn.example.com"
        }))
        .unwrap();

        let mut api_url = ApiUrl::new(&conf).unwrap();
        api_url.vpn_param.url = "https://vpn-node.example.com".to_string();
        let url = api_url.get_api_url(&ApiName::ConnectVPN);

        assert!(url.starts_with("https://vpn-node.example.com/vpn/conn?"));
        assert!(url.contains("app_version=3.2.16"));
        assert!(url.contains("build_number=12116"));
        assert!(url.contains("client_source=FeiLian"));
        assert!(url.contains(&format!("os={}", system_parameters().os)));
        assert!(url.contains("timestamp="));
    }

    #[test]
    fn connect_vpn_url_uses_the_compatible_gateway_shape() {
        let conf: Config = serde_json::from_value(json!({
            "company_name": "test",
            "username": "test",
            "server": "https://vpn.example.com"
        }))
        .unwrap();
        let mut api_url = ApiUrl::new(&conf).unwrap();
        api_url.vpn_param.url = "http://192.0.2.1:80".to_string();

        let url = Url::parse(&api_url.get_api_url(&ApiName::ConnectVPN)).unwrap();
        assert_eq!(url.scheme(), "http");
        assert_eq!(url.host_str(), Some("192.0.2.1"));
        assert_eq!(url.port_or_known_default(), Some(80));
        assert_eq!(url.path(), "/vpn/conn");
        assert_eq!(
            url.query_pairs()
                .find(|(key, _)| key == "os")
                .map(|(_, value)| value.into_owned()),
            Some(system_parameters().os)
        );
    }

    #[test]
    fn email_v1_urls_match_current_official_client_paths() {
        let conf: Config = serde_json::from_value(json!({
            "company_name": "test",
            "username": "test",
            "server": "https://vpn.example.com"
        }))
        .unwrap();

        let api_url = ApiUrl::new(&conf).unwrap();

        assert_eq!(
            api_url.get_api_url(&ApiName::RequestEmailCodeV1),
            "https://vpn.example.com/api/v1/login/send?os=Android&os_version=2"
        );
        assert_eq!(
            api_url.get_api_url(&ApiName::LoginEmailV1),
            "https://vpn.example.com/api/v1/login/verify?os=Android&os_version=2"
        );
    }

    #[test]
    fn qr_urls_match_current_official_client_paths_and_encode_token() {
        let conf: Config = serde_json::from_value(json!({
            "company_name": "test",
            "username": "test",
            "server": "https://vpn.example.com"
        }))
        .unwrap();

        let api_url = ApiUrl::new(&conf).unwrap();

        let token_url = Url::parse(&api_url.get_api_url(&ApiName::LoginQrToken)).unwrap();
        assert_eq!(token_url.path(), "/api/login/token");
        assert!(token_url.query_pairs().any(|(key, _)| key == "timestamp"));
        let check_url = Url::parse(&api_url.get_qr_check_url("a+b&c").unwrap()).unwrap();
        assert_eq!(check_url.path(), "/api/login/token/check");
        let query = check_url.query_pairs().collect::<HashMap<_, _>>();
        assert_eq!(
            query.get("token").map(|value| value.as_ref()),
            Some("a+b&c")
        );
        assert_eq!(
            query.get("login_scene").map(|value| value.as_ref()),
            Some("feilian")
        );
        assert!(query.contains_key("timestamp"));
    }

    #[test]
    fn vpn_mfa_urls_match_current_official_client_paths() {
        let conf: Config = serde_json::from_value(json!({
            "company_name": "test",
            "username": "test",
            "server": "https://vpn.example.com"
        }))
        .unwrap();

        let api_url = ApiUrl::new(&conf).unwrap();

        assert!(api_url
            .get_api_url(&ApiName::VpnMfaType)
            .starts_with("https://vpn.example.com/api/mfa/type?app_version=3.2.16"));
        assert!(api_url
            .get_api_url(&ApiName::VpnMfaSend)
            .starts_with("https://vpn.example.com/api/mfa/code/send?app_version=3.2.16"));
        assert!(api_url
            .get_api_url(&ApiName::VpnMfaVerify)
            .starts_with("https://vpn.example.com/api/mfa/code/verify?app_version=3.2.16"));
        let push_url = api_url.get_api_url(&ApiName::VpnMfaPush);
        assert!(push_url.starts_with("https://vpn.example.com/api/v1/mfa/send?app_version=3.2.16"));
        assert!(push_url.contains("&os_version="));
        assert!(push_url.contains("&soc="));
        assert!(push_url.contains("&timestamp="));
        let revoke_url = api_url.get_api_url(&ApiName::VpnMfaRevoke);
        assert!(
            revoke_url.starts_with("https://vpn.example.com/api/v1/mfa/revoke?app_version=3.2.16")
        );
        assert!(revoke_url.contains("&timestamp="));
    }

    #[test]
    fn websocket_url_matches_official_shape() {
        let conf: Config = serde_json::from_value(json!({
            "company_name": "test",
            "username": "test",
            "server": "https://vpn.example.com:10443/base"
        }))
        .unwrap();
        let api_url = ApiUrl::new(&conf).unwrap();
        let url = Url::parse(&api_url.get_websocket_url().unwrap()).unwrap();

        assert_eq!(url.scheme(), "wss");
        assert_eq!(url.host_str(), Some("vpn.example.com"));
        assert_eq!(url.port(), Some(10443));
        assert_eq!(url.path(), "/api/ws/socket");
        let query = url.query_pairs().collect::<HashMap<_, _>>();
        for key in [
            "os",
            "model",
            "brand",
            "client_source",
            "app_version",
            "build_number",
            "os_version",
            "os_release",
            "soc",
            "language",
            "timestamp",
        ] {
            assert!(query.contains_key(key), "missing query parameter {key}");
        }
        assert!(!query.contains_key("device_id"));
    }
}
