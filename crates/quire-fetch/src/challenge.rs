//! Recognising "are you human" pages (`.knowledge/metadata-sources.md`, challenge pages):
//! Cloudflare marks them with the `cf-mitigated: challenge` header; others are recognised by
//! markers in the HTML (interstitial titles, CAPTCHA widgets, DDoS-Guard, Imperva).
//! Seen live 2026-10-19: ScienceDirect and Oxford Academic answer 403 + `cf-mitigated`,
//! ResearchGate 403 with a CAPTCHA page, PMC's PDF links 200 with a proof-of-work page
//! ("Preparing to download ..."). Cloudflare's `/cdn-cgi/challenge-platform/` script is
//! not a marker: it is also loaded by normal pages and error pages (doi.org's 404).

/// Text found in challenge pages and not in normal article pages.
const MARKERS: &[&str] = &[
    "<title>Just a moment...</title>",
    "<title>Preparing to download ...</title>",
    "challenge-captcha",
    "cf-turnstile",
    "g-recaptcha",
    "hcaptcha.com",
    "px-captcha",
    "_Incapsula_Resource",
    "ddos-guard",
    "Checking your browser before accessing",
    "Verify you are human",
];

/// Markers that only an interstitial page has; for a successful response only these count
/// (a normal article page may embed a CAPTCHA widget, e.g. in a comment form).
const INTERSTITIAL: &[&str] = &[
    "<title>Just a moment...</title>",
    "Checking your browser before accessing",
    "<title>Preparing to download ...</title>",
];

/// Whether a response is a challenge: the Cloudflare header; for an error status any marker in
/// the body; for a successful one only an interstitial marker.
pub(crate) fn is_challenge(cf_mitigated: Option<&str>, body: &str, success: bool) -> bool {
    let markers = if success { INTERSTITIAL } else { MARKERS };
    cf_mitigated.is_some_and(|v| v.eq_ignore_ascii_case("challenge")) || markers.iter().any(|m| body.contains(m))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markers() {
        assert!(is_challenge(Some("challenge"), "", false));
        assert!(is_challenge(None, "<html><head><title>Just a moment...</title>", true));
        assert!(!is_challenge(None, "<html><head><title>Attention Is All You Need</title>", false));
        let normal = r#"<title>A paper</title><script src="/cdn-cgi/challenge-platform/scripts/jsd/main.js">"#;
        assert!(!is_challenge(None, normal, true), "bot scripts on a normal page");
        assert!(!is_challenge(None, normal, false), "bot scripts on an error page");
        assert!(!is_challenge(None, r#"<div class="g-recaptcha"></div>"#, true), "comment form");
        assert!(is_challenge(None, r#"<div class="g-recaptcha"></div>"#, false));
    }
}
