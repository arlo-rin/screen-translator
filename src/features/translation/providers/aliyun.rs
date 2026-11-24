//! Alibaba Cloud Machine Translation (the traditional AccessKey product).
//!
//! This is *not* the Qwen / Model Studio OpenAI-compatible API. It signs
//! ordinary RPC requests with an AccessKey ID and Secret, and talks to the
//! official Machine Translation endpoints documented for product `alimt`
//! version `2018-10-12`.
//!
//! Multi-paragraph input is sent through `GetBatchTranslate` with a stable
//! id per piece. The reply is reassembled by those ids, so a missing or
//! merged line cannot silently shift later paragraphs — the way a model
//! that is only asked to "keep the same number of lines" sometimes does.
//!
//! Official limits, as of the public documentation: 50 items and 8000
//! characters per request, 1000 characters per item, 50 requests per second.
//! Batches are sent one after another, well inside that rate.

use std::collections::BTreeMap;

use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;
use hmac::{Hmac, Mac};
use rand::Rng;
use serde_json::Value;
use sha1::Sha1;
use tracing::debug;

use super::{LinePlan, TranslateRequest};
use crate::entities::language::Language;
use crate::entities::settings::{AliyunConfig, AliyunEndpoint};
use crate::features::translation::client::HTTP;
use crate::shared::error::AppError;
use crate::shared::i18n::t;

type HmacSha1 = Hmac<Sha1>;

const API_VERSION: &str = "2018-10-12";
const ACTION_BATCH: &str = "GetBatchTranslate";
/// General-edition scene and API type, as named in the official docs.
const SCENE: &str = "general";
const API_TYPE: &str = "translate_standard";
const FORMAT_TYPE: &str = "text";

/// Official per-item and per-request caps for `GetBatchTranslate`.
const MAX_ITEM_CHARS: usize = 1000;
const MAX_ITEMS: usize = 50;
const MAX_BATCH_CHARS: usize = 8000;

/// Builds the RFC 3986 / Alibaba Cloud percent-encoding of `s`.
///
/// Unreserved characters (`A–Z a–z 0–9 - _ . ~`) are left as they are;
/// everything else becomes `%` plus uppercase hex. A space is `%20`, not
/// `+`. This is the encoding the RPC signature is computed over.
pub fn percent_encode(s: &str) -> String {
    let mut out = String::new();
    for &b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                out.push_str(&format!("%{b:02X}"));
            }
        }
    }
    out
}

/// Canonical query string and `StringToSign` for an RPC request, plus the
/// HMAC-SHA1 signature (unencoded). Parameters must not include `Signature`.
///
/// The signing key is `AccessKeySecret` with a trailing `&`, as specified
/// by the Alibaba Cloud RPC signature documentation.
pub fn sign_rpc(
    method: &str,
    params: &BTreeMap<String, String>,
    access_key_secret: &str,
) -> (String, String, String) {
    let canonical = params
        .iter()
        .map(|(k, v)| format!("{}={}", percent_encode(k), percent_encode(v)))
        .collect::<Vec<_>>()
        .join("&");
    let string_to_sign = format!(
        "{method}&{}&{}",
        percent_encode("/"),
        percent_encode(&canonical)
    );
    let mut mac = HmacSha1::new_from_slice(format!("{access_key_secret}&").as_bytes())
        .expect("HMAC-SHA1 accepts a key of any length");
    mac.update(string_to_sign.as_bytes());
    let signature = STANDARD.encode(mac.finalize().into_bytes());
    (canonical, string_to_sign, signature)
}

fn utc_timestamp(now: time::OffsetDateTime) -> String {
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        now.year(),
        now.month() as u8,
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    )
}

fn signature_nonce() -> String {
    let mut rng = rand::thread_rng();
    let a: u64 = rng.gen();
    let b: u64 = rng.gen();
    format!("{a:016x}{b:016x}")
}

/// Language tag the official Machine Translation list accepts, or `None`
/// when this app language is not on that list (and must not be guessed at).
///
/// Serbian (`sr`) has no entry there — the closest official tag is
/// Serbo-Croatian `hbs`, which is not the same language and is not used.
/// Automatic detection (`auto`) is only valid as a source.
pub fn language_code(lang: Language, as_source: bool) -> Option<&'static str> {
    match lang {
        Language::Auto if as_source => Some("auto"),
        Language::Auto => None,
        Language::Sr => None,
        other => Some(other.code()),
    }
}

fn common_params(
    action: &str,
    access_key_id: &str,
    timestamp: &str,
    nonce: &str,
) -> BTreeMap<String, String> {
    let mut p = BTreeMap::new();
    p.insert("Action".into(), action.into());
    p.insert("Version".into(), API_VERSION.into());
    p.insert("Format".into(), "JSON".into());
    p.insert("AccessKeyId".into(), access_key_id.into());
    p.insert("SignatureMethod".into(), "HMAC-SHA1".into());
    p.insert("SignatureVersion".into(), "1.0".into());
    p.insert("SignatureNonce".into(), nonce.into());
    p.insert("Timestamp".into(), timestamp.into());
    p
}

/// Form body for a signed POST. The signature is in the body, not the URL,
/// so a failed request cannot leak it through the URL of an HTTP error.
fn signed_form(params: &BTreeMap<String, String>, secret: &str) -> String {
    let (canonical, _, signature) = sign_rpc("POST", params, secret);
    format!("{canonical}&Signature={}", percent_encode(&signature))
}

pub async fn translate(req: &TranslateRequest, cfg: &AliyunConfig) -> Result<String, AppError> {
    if !cfg.is_configured() {
        return Err(AppError::Other(
            t("Missing AccessKey ID or Secret").to_string(),
        ));
    }
    let source = language_code(req.source, true).ok_or_else(|| {
        AppError::Other(
            t("Alibaba Cloud does not support {language}")
                .replace("{language}", &req.source.to_string()),
        )
    })?;
    let target = language_code(req.target, false).ok_or_else(|| {
        AppError::Other(
            t("Alibaba Cloud does not support {language}")
                .replace("{language}", &req.target.to_string()),
        )
    })?;

    let plan = LinePlan::new(&req.text, MAX_ITEM_CHARS);
    if plan.pieces.is_empty() {
        return plan.assemble(&[]).ok_or_else(line_mismatch);
    }

    let mismatch = line_mismatch;
    let mut translated: Vec<String> = Vec::with_capacity(plan.pieces.len());
    // One request at a time: well under the official 50 QPS, and a later
    // batch is not sent if an earlier one already failed.
    for batch in plan.batches(MAX_BATCH_CHARS, MAX_ITEMS) {
        let mut map = serde_json::Map::new();
        for (offset, piece) in plan.pieces[batch.clone()].iter().enumerate() {
            map.insert(offset.to_string(), Value::String(piece.clone()));
        }
        let source_text = Value::Object(map).to_string();
        let raw = rpc(
            cfg,
            ACTION_BATCH,
            &[
                ("FormatType", FORMAT_TYPE),
                ("SourceLanguage", source),
                ("TargetLanguage", target),
                ("Scene", SCENE),
                ("ApiType", API_TYPE),
                ("SourceText", &source_text),
            ],
        )
        .await?;
        let pieces = parse_batch(&raw)?;
        if pieces.len() != batch.len() {
            return Err(mismatch());
        }
        translated.extend(pieces);
    }

    plan.assemble(&translated).ok_or_else(mismatch)
}

/// A short, user-initiated connection check. Not called automatically.
pub async fn test_connection(cfg: &AliyunConfig, target: Language) -> Result<String, AppError> {
    let target = if target == Language::En {
        Language::Zh
    } else {
        target
    };
    translate(
        &TranslateRequest {
            text: "Hello".into(),
            source: Language::En,
            target,
        },
        cfg,
    )
    .await
}

async fn rpc(cfg: &AliyunConfig, action: &str, extra: &[(&str, &str)]) -> Result<String, AppError> {
    let timestamp = utc_timestamp(time::OffsetDateTime::now_utc());
    let nonce = signature_nonce();
    let mut params = common_params(action, cfg.access_key_id.trim(), &timestamp, &nonce);
    for (k, v) in extra {
        params.insert((*k).to_string(), (*v).to_string());
    }
    let body = signed_form(&params, cfg.access_key_secret.expose());
    let url = cfg.endpoint.url();

    let resp = HTTP
        .post(url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await?;

    let status = resp.status();
    let raw = resp.text().await?;
    // Status only — never the body, the signed form, or the URL.
    debug!(status = status.as_u16(), action, "Alibaba Cloud response");

    if !status.is_success() {
        return Err(AppError::Other(explain(&raw, status.as_u16())));
    }
    if let Some(message) = error_from_body(&raw) {
        return Err(AppError::Other(message));
    }
    Ok(raw)
}

fn line_mismatch() -> AppError {
    AppError::Other(t("The service answered with a different number of lines").to_string())
}

/// `GetBatchTranslate` success body. Items are taken by the `index` we sent,
/// not by list order; any missing, failed, or extra item is a hard error.
pub fn parse_batch(raw: &str) -> Result<Vec<String>, AppError> {
    let json: Value = serde_json::from_str(raw).map_err(|e| {
        AppError::Other(
            t("Unrecognized response from Alibaba Cloud: {error}")
                .replace("{error}", &e.to_string()),
        )
    })?;
    if let Some(message) = error_from_value(&json) {
        return Err(AppError::Other(message));
    }
    let list = json
        .get("TranslatedList")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            AppError::Other(
                t("Unrecognized response from Alibaba Cloud: {error}")
                    .replace("{error}", "no TranslatedList"),
            )
        })?;

    let mut by_index: BTreeMap<usize, String> = BTreeMap::new();
    for item in list {
        if !item_ok(item) {
            let code = item_code(item);
            return Err(AppError::Other(explain_code(code, "")));
        }
        let index = item
            .get("index")
            .and_then(|v| match v {
                Value::String(s) => s.parse::<usize>().ok(),
                Value::Number(n) => n.as_u64().map(|n| n as usize),
                _ => None,
            })
            .ok_or_else(line_mismatch)?;
        let text = item
            .get("translated")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if by_index.insert(index, text).is_some() {
            return Err(line_mismatch());
        }
    }

    // The ids we send are 0..n in batch order. Gaps or a surprise id mean
    // the reply cannot be lined up with the request.
    let n = by_index.len();
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        match by_index.remove(&i) {
            Some(text) => out.push(text),
            None => return Err(line_mismatch()),
        }
    }
    if !by_index.is_empty() {
        return Err(line_mismatch());
    }
    if out.is_empty() {
        return Err(AppError::Other(
            t("Alibaba Cloud returned an empty translation").to_string(),
        ));
    }
    Ok(out)
}

fn item_ok(item: &Value) -> bool {
    match item_code(item) {
        None => true,
        Some(code) => code == "200" || code.eq_ignore_ascii_case("success"),
    }
}

fn item_code(item: &Value) -> Option<String> {
    item.get("code").and_then(|v| match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    })
}

/// A top-level `Code` that is not success, if any.
pub fn error_from_body(raw: &str) -> Option<String> {
    serde_json::from_str::<Value>(raw)
        .ok()
        .and_then(|v| error_from_value(&v))
}

fn error_from_value(json: &Value) -> Option<String> {
    let code_owned = match json.get("Code") {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => n.to_string(),
        _ => return None,
    };
    if code_owned == "200" || code_owned.eq_ignore_ascii_case("success") {
        return None;
    }
    let message = json.get("Message").and_then(Value::as_str).unwrap_or("");
    Some(explain_code(Some(code_owned), message))
}

fn explain(raw: &str, http_status: u16) -> String {
    if let Some(message) = error_from_body(raw) {
        return message;
    }
    match http_status {
        401 | 403 => t("The AccessKey was rejected").to_string(),
        429 => t("Too many requests, try again later").to_string(),
        500..=599 => t("The service is having trouble — try again in a moment").to_string(),
        _ => t("The service refused the request").to_string(),
    }
}

fn explain_code(code: Option<String>, message: &str) -> String {
    let code = code.unwrap_or_default();
    let lower = code.to_ascii_lowercase();
    let numeric = match code.parse::<i32>() {
        // The public error tables list both a short status (101–113, 199)
        // and a five-digit code (10001–10013, 19999). Either may appear in
        // the JSON `Code` field.
        Ok(n) if (101..=113).contains(&n) => Some(n + 9900),
        Ok(199) => Some(19999),
        Ok(n) => Some(n),
        Err(_) => None,
    };
    let text = match (numeric, lower.as_str()) {
        (Some(10001), _) | (_, "system.requesttimeout") => t("The service did not answer in time"),
        (Some(10005), _) | (_, "system.languagenotsupported") => {
            t("This language pair is not supported")
        }
        (Some(10008), _) | (_, "system.stringtoolong") => {
            t("The string is longer than Alibaba Cloud allows")
        }
        (Some(10009), _) | (_, "system.subnotpermission") | (_, "forbidden.ram") => {
            t("The RAM user is not allowed to call Machine Translation")
        }
        (Some(10010), _) | (_, "system.accountnotactivated") => {
            t("The account has not activated Machine Translation")
        }
        (Some(10013), _) | (_, "system.accountnotactivatedorinarrears") => {
            t("The Alibaba Cloud account is in arrears or the service is not activated")
        }
        (_, "invalidaccesskeyid.notfound")
        | (_, "invalidaccesskeyid")
        | (_, "signaturedoesnotmatch")
        | (_, "incompleteSignature")
        | (_, "incompletesignature")
        | (_, "invalidsecuritytoken") => t("The AccessKey was rejected"),
        (Some(10004), _) | (_, "system.parametererror") => t("The service refused the request"),
        (Some(10002 | 10007 | 10011 | 10012 | 19999), _) => {
            t("The service is having trouble — try again in a moment")
        }
        _ if lower.contains("accesskey") || lower.contains("signature") => {
            t("The AccessKey was rejected")
        }
        _ => t("The service refused the request"),
    }
    .to_string();
    let _ = message;
    // The vendor message can echo request parameters. The mapped phrase is
    // enough; the numeric or symbolic code is safe to keep for diagnosis.
    if code.is_empty() {
        text
    } else {
        format!("{text} ({code})")
    }
}

impl AliyunEndpoint {
    pub fn url(self) -> &'static str {
        match self {
            AliyunEndpoint::Default => "https://mt.aliyuncs.com",
            AliyunEndpoint::Hangzhou => "https://mt.cn-hangzhou.aliyuncs.com",
            AliyunEndpoint::Singapore => "https://mt.ap-southeast-1.aliyuncs.com",
            AliyunEndpoint::Riyadh => "https://mt.me-central-1.aliyuncs.com",
            AliyunEndpoint::Dubai => "https://alimt.me-east-1.aliyuncs.com",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            AliyunEndpoint::Default => t("Default (mt.aliyuncs.com)"),
            AliyunEndpoint::Hangzhou => t("China (Hangzhou)"),
            AliyunEndpoint::Singapore => t("Singapore"),
            AliyunEndpoint::Riyadh => t("Saudi Arabia (Riyadh)"),
            AliyunEndpoint::Dubai => t("UAE (Dubai)"),
        }
    }

    pub fn all() -> [AliyunEndpoint; 5] {
        [
            AliyunEndpoint::Default,
            AliyunEndpoint::Hangzhou,
            AliyunEndpoint::Singapore,
            AliyunEndpoint::Riyadh,
            AliyunEndpoint::Dubai,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::secret::Secret;

    fn map(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect()
    }

    #[test]
    fn percent_encoding_follows_the_aliyun_rules() {
        assert_eq!(percent_encode("Hello"), "Hello");
        assert_eq!(percent_encode("a b"), "a%20b");
        assert_eq!(percent_encode("a*b"), "a%2Ab");
        assert_eq!(percent_encode("~ok~"), "~ok~");
        assert_eq!(percent_encode("你好"), "%E4%BD%A0%E5%A5%BD");
        assert_eq!(
            percent_encode("2026-01-02T03:04:05Z"),
            "2026-01-02T03%3A04%3A05Z"
        );
    }

    #[test]
    fn a_known_request_signs_to_the_documented_vector() {
        // Synthetic credentials only. The expected value was computed with
        // the official HMAC-SHA1 + percent-encode procedure, independently
        // of this crate.
        let params = map(&[
            ("AccessKeyId", "testid"),
            ("Action", "GetBatchTranslate"),
            ("ApiType", "translate_standard"),
            ("Format", "JSON"),
            ("FormatType", "text"),
            ("Scene", "general"),
            ("SignatureMethod", "HMAC-SHA1"),
            ("SignatureNonce", "nonce-1"),
            ("SignatureVersion", "1.0"),
            ("SourceLanguage", "en"),
            ("SourceText", r#"{"0":"Hello","1":"你好"}"#),
            ("TargetLanguage", "zh"),
            ("Timestamp", "2026-01-02T03:04:05Z"),
            ("Version", "2018-10-12"),
        ]);
        let (canonical, string_to_sign, signature) = sign_rpc("POST", &params, "testsecret");
        assert!(canonical.starts_with("AccessKeyId=testid&Action=GetBatchTranslate"));
        assert!(canonical.contains(
            "SourceText=%7B%220%22%3A%22Hello%22%2C%221%22%3A%22%E4%BD%A0%E5%A5%BD%22%7D"
        ));
        assert!(string_to_sign.starts_with("POST&%2F&"));
        assert_eq!(signature, "rgTdVTyuBpby/Jyuxw5ehR1Z/kA=");
        assert!(!canonical.contains("Signature="));
        assert!(!string_to_sign.contains("testsecret"));
    }

    #[test]
    fn english_and_chinese_are_accepted_both_ways() {
        assert_eq!(language_code(Language::En, true), Some("en"));
        assert_eq!(language_code(Language::Zh, false), Some("zh"));
        assert_eq!(language_code(Language::Auto, true), Some("auto"));
        assert_eq!(language_code(Language::Auto, false), None);
        assert_eq!(language_code(Language::Ja, true), Some("ja"));
        assert_eq!(language_code(Language::Sr, true), None);
    }

    #[test]
    fn official_endpoints_are_the_documented_hosts() {
        assert_eq!(AliyunEndpoint::Default.url(), "https://mt.aliyuncs.com");
        assert_eq!(
            AliyunEndpoint::Hangzhou.url(),
            "https://mt.cn-hangzhou.aliyuncs.com"
        );
        assert_eq!(
            AliyunEndpoint::Singapore.url(),
            "https://mt.ap-southeast-1.aliyuncs.com"
        );
    }

    #[test]
    fn a_batch_reply_is_reassembled_by_index_not_list_order() {
        let raw = r#"{
            "Code": 200,
            "Message": "success",
            "RequestId": "86D18195-D89C-4C8C-9DC4-5FCE789CE6D5",
            "TranslatedList": [
                {"code":"200","index":"1","translated":"第二段","wordCount":"4"},
                {"code":"200","index":"0","translated":"第一段","wordCount":"4"}
            ]
        }"#;
        assert_eq!(parse_batch(raw).unwrap(), vec!["第一段", "第二段"]);
    }

    #[test]
    fn a_failed_item_is_not_silently_dropped() {
        let raw = r#"{
            "Code": 200,
            "TranslatedList": [
                {"code":"200","index":"0","translated":"好"},
                {"code":"10007","index":"1","translated":""}
            ]
        }"#;
        let err = parse_batch(raw).unwrap_err().user_message();
        assert!(err.contains("10007"), "{err}");
        assert!(!err.contains("Signature"), "{err}");
    }

    #[test]
    fn a_missing_index_is_refused() {
        let raw = r#"{
            "Code": 200,
            "TranslatedList": [
                {"code":"200","index":"0","translated":"一"},
                {"code":"200","index":"2","translated":"三"}
            ]
        }"#;
        assert!(parse_batch(raw).is_err());
    }

    #[test]
    fn account_and_key_errors_are_named_without_echoing_the_body() {
        let raw = r#"{"Code":"InvalidAccessKeyId.NotFound","Message":"Specified access key is not found","AccessKeyId":"LTAI-not-a-real-key","RequestId":"abc"}"#;
        let msg = error_from_body(raw).expect("error");
        assert!(msg.contains("AccessKey"), "{msg}");
        assert!(!msg.contains("LTAI-not-a-real-key"), "{msg}");
        assert!(!msg.contains("not found"), "{msg}");

        let arrears = error_from_body(r#"{"Code":113,"Message":"account overdue"}"#).unwrap();
        assert!(
            arrears.contains("arrears") || arrears.contains("activated"),
            "{arrears}"
        );

        let unopened =
            error_from_body(r#"{"Code":"System.AccountNotActivated","Message":"no"}"#).unwrap();
        assert!(unopened.to_lowercase().contains("activated"), "{unopened}");

        let pair = error_from_body(r#"{"Code":10005,"Message":"lang"}"#).unwrap();
        assert!(pair.to_lowercase().contains("language"), "{pair}");
    }

    #[test]
    fn a_success_code_is_not_treated_as_an_error() {
        assert!(error_from_body(r#"{"Code":200,"Message":"success"}"#).is_none());
        assert!(error_from_body(r#"{"Code":"200","Message":"success"}"#).is_none());
    }

    #[test]
    fn signed_form_puts_the_signature_in_the_body_not_as_a_displayable_url() {
        let mut params = map(&[("AccessKeyId", "testid"), ("Action", "GetBatchTranslate")]);
        // Fill the rest so the helper can be used as the request does.
        params.extend(map(&[
            ("Version", "2018-10-12"),
            ("Format", "JSON"),
            ("SignatureMethod", "HMAC-SHA1"),
            ("SignatureVersion", "1.0"),
            ("SignatureNonce", "n"),
            ("Timestamp", "2026-01-02T03:04:05Z"),
        ]));
        let body = signed_form(&params, "testsecret");
        assert!(body.contains("Signature="));
        assert!(!body.contains("testsecret"));
        assert!(!AliyunEndpoint::Default.url().contains('?'));
    }

    #[test]
    fn synthetic_credentials_are_enough_to_be_configured() {
        let mut cfg = AliyunConfig::default();
        assert!(!cfg.is_configured());
        cfg.access_key_id = "testid".into();
        assert!(!cfg.is_configured());
        cfg.access_key_secret = Secret::new("testsecret");
        assert!(cfg.is_configured());
        assert!(format!("{cfg:?}").contains("Secret(***)"));
        assert!(!format!("{cfg:?}").contains("testsecret"));
    }
}
