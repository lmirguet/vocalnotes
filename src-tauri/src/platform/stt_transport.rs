//! Allowlisted diagnostics only. Never display an error, response body or headers.
use tokio_tungstenite::tungstenite::{
    Error, client::IntoClientRequest, error::ProtocolError, http::HeaderValue,
};

pub const URL: &str = "wss://api.mistral.ai/v1/audio/transcriptions/realtime?model=voxtral-mini-transcribe-realtime-2602";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Failure {
    Timeout,
    Network,
    Tls,
    TlsCertificate,
    Authentication,
    Access,
    QuotaOrRateLimit,
    HttpClient,
    HttpServer,
    HttpOther,
    HandshakeIncomplete,
    HttpVersion,
    Upgrade,
    Protocol,
    Configuration,
    Capacity,
    Closed,
}

impl Failure {
    pub fn message(self) -> &'static str {
        match self {
            Self::Timeout => "STT [timeout] : connexion expirée.",
            Self::Network => "STT [network] : connexion réseau ou entrée/sortie impossible.",
            Self::Tls => "STT [tls] : négociation TLS refusée ; aucun contournement.",
            Self::TlsCertificate => {
                "STT [tls-certificate] : certificat non accepté ; aucun contournement."
            }
            Self::Authentication => "STT [http-401] : authentification refusée.",
            Self::Access => "STT [http-403] : accès refusé.",
            Self::QuotaOrRateLimit => {
                "STT [http-429] : quota ou limite de requêtes ; aucun nouvel essai automatique."
            }
            Self::HttpClient => "STT [http-4xx] : requête ou ressource refusée.",
            Self::HttpServer => "STT [http-5xx] : service indisponible.",
            Self::HttpOther => "STT [http-other] : réponse HTTP incompatible avec le handshake.",
            Self::HandshakeIncomplete => {
                "STT [ws-incomplete] : connexion fermée avant la fin du handshake."
            }
            Self::HttpVersion => {
                "STT [ws-http-version] : version HTTP incompatible avec le handshake WebSocket."
            }
            Self::Upgrade => "STT [ws-upgrade] : handshake WebSocket invalide.",
            Self::Protocol => "STT [ws-protocol] : protocole WebSocket invalide.",
            Self::Configuration => "STT [configuration] : URL ou en-tête invalide.",
            Self::Capacity => "STT [capacity] : limite de transport dépassée.",
            Self::Closed => "STT [closed] : connexion interrompue.",
        }
    }
}

pub fn request(
    secret: Option<&str>,
) -> Result<tokio_tungstenite::tungstenite::handshake::client::Request, Failure> {
    let mut request = URL
        .into_client_request()
        .map_err(|_| Failure::Configuration)?;
    if let Some(secret) = secret {
        let mut auth: HeaderValue = format!("Bearer {secret}")
            .parse()
            .map_err(|_| Failure::Configuration)?;
        auth.set_sensitive(true);
        request.headers_mut().insert("Authorization", auth);
    }
    Ok(request)
}

fn tls(error: &rustls::Error) -> Failure {
    match error {
        rustls::Error::InvalidCertificate(_) | rustls::Error::NoCertificatesPresented => {
            Failure::TlsCertificate
        }
        _ => Failure::Tls,
    }
}

pub fn classify(error: &Error) -> Failure {
    match error {
        Error::Io(error) => {
            // tokio-tungstenite 0.28 wraps rustls handshake failures in Error::Io.
            // Downcast the typed payload, NEVER classify by its potentially private text.
            match error
                .get_ref()
                .and_then(|e| e.downcast_ref::<rustls::Error>())
            {
                Some(error) => tls(error),
                None if error.kind() == std::io::ErrorKind::TimedOut => Failure::Timeout,
                None => Failure::Network,
            }
        }
        Error::Tls(tokio_tungstenite::tungstenite::error::TlsError::Rustls(e)) => tls(e),
        Error::Tls(_) => Failure::Tls,
        Error::Http(response) => match response.status().as_u16() {
            401 => Failure::Authentication,
            403 => Failure::Access,
            429 => Failure::QuotaOrRateLimit,
            400..=499 => Failure::HttpClient,
            500..=599 => Failure::HttpServer,
            _ => Failure::HttpOther,
        },
        Error::Protocol(ProtocolError::HandshakeIncomplete) => Failure::HandshakeIncomplete,
        Error::Protocol(ProtocolError::WrongHttpVersion) => Failure::HttpVersion,
        Error::Protocol(
            ProtocolError::MissingConnectionUpgradeHeader
            | ProtocolError::MissingUpgradeWebSocketHeader
            | ProtocolError::SecWebSocketAcceptKeyMismatch
            | ProtocolError::SecWebSocketSubProtocolError(_),
        ) => Failure::Upgrade,
        Error::Url(_) | Error::HttpFormat(_) => Failure::Configuration,
        Error::Capacity(_) | Error::WriteBufferFull(_) => Failure::Capacity,
        Error::ConnectionClosed | Error::AlreadyClosed => Failure::Closed,
        _ => Failure::Protocol,
    }
}

/// Variant names only; never format the protocol payload.
pub fn protocol_code(error: &ProtocolError) -> &'static str {
    match error {
        ProtocolError::WrongHttpMethod => "wrong-http-method",
        ProtocolError::WrongHttpVersion => "wrong-http-version",
        ProtocolError::MissingConnectionUpgradeHeader => "missing-connection-upgrade",
        ProtocolError::MissingUpgradeWebSocketHeader => "missing-websocket-upgrade",
        ProtocolError::MissingSecWebSocketVersionHeader => "missing-websocket-version",
        ProtocolError::MissingSecWebSocketKey => "missing-websocket-key",
        ProtocolError::SecWebSocketAcceptKeyMismatch => "websocket-accept-mismatch",
        ProtocolError::SecWebSocketSubProtocolError(_) => "websocket-subprotocol",
        ProtocolError::JunkAfterRequest => "junk-after-request",
        ProtocolError::CustomResponseSuccessful => "custom-response-successful",
        ProtocolError::InvalidHeader(_) => "invalid-header",
        ProtocolError::HandshakeIncomplete => "handshake-incomplete",
        ProtocolError::HttparseError(_) => "http-parser",
        ProtocolError::SendAfterClosing => "send-after-closing",
        ProtocolError::ReceivedAfterClosing => "received-after-closing",
        ProtocolError::NonZeroReservedBits => "reserved-bits",
        ProtocolError::UnmaskedFrameFromClient => "unmasked-frame",
        ProtocolError::MaskedFrameFromServer => "masked-server-frame",
        ProtocolError::FragmentedControlFrame => "fragmented-control-frame",
        ProtocolError::ControlFrameTooBig => "control-frame-size",
        ProtocolError::UnknownControlFrameType(_) => "control-frame-type",
        ProtocolError::UnknownDataFrameType(_) => "data-frame-type",
        ProtocolError::UnexpectedContinueFrame => "unexpected-continue",
        ProtocolError::ExpectedFragment(_) => "expected-fragment",
        ProtocolError::ResetWithoutClosingHandshake => "reset-without-close",
        ProtocolError::InvalidOpcode(_) => "invalid-opcode",
        ProtocolError::InvalidCloseSequence => "invalid-close-sequence",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use tokio_tungstenite::tungstenite::{Error, error::ProtocolError, http::Response};

    #[test]
    fn request_has_standard_upgrade_and_sensitive_bearer_header() {
        let r = request(Some("synthetic-test-not-a-credential")).unwrap();
        assert_eq!(
            r.version(),
            tokio_tungstenite::tungstenite::http::Version::HTTP_11
        );
        assert_eq!(r.uri().scheme_str(), Some("wss"));
        assert_eq!(r.uri().host(), Some("api.mistral.ai"));
        assert_eq!(r.uri().path(), "/v1/audio/transcriptions/realtime");
        assert_eq!(
            r.uri().query(),
            Some("model=voxtral-mini-transcribe-realtime-2602")
        );
        assert_eq!(r.headers()["upgrade"], "websocket");
        assert_eq!(r.headers()["connection"], "Upgrade");
        assert_eq!(r.headers()["sec-websocket-version"], "13");
        let auth = &r.headers()["authorization"];
        assert!(auth.is_sensitive());
        assert_eq!(
            auth.to_str().unwrap(),
            "Bearer synthetic-test-not-a-credential"
        );
        assert!(!format!("{:?}", r.headers()).contains("synthetic-test"));
        assert!(
            !request(None)
                .unwrap()
                .headers()
                .contains_key("authorization")
        );
        assert_eq!(
            request(Some("synthetic\r\ninjection")).unwrap_err(),
            Failure::Configuration
        );
    }

    #[test]
    fn http_classification_ignores_all_remote_content() {
        for (status, expected) in [
            (401, Failure::Authentication),
            (403, Failure::Access),
            (429, Failure::QuotaOrRateLimit),
            (400, Failure::HttpClient),
            (404, Failure::HttpClient),
            (500, Failure::HttpServer),
            (302, Failure::HttpOther),
        ] {
            let response = Response::builder()
                .status(status)
                .header("x-private", "synthetic-private-header")
                .body(Some(b"synthetic-private-body".to_vec()))
                .unwrap();
            let category = classify(&Error::Http(Box::new(response)));
            assert_eq!(category, expected);
            assert!(!category.message().contains("synthetic"));
        }
    }

    #[test]
    fn rustls_handshake_errors_wrapped_as_io_are_tls_not_network() {
        let e = Error::Io(io::Error::new(
            io::ErrorKind::InvalidData,
            rustls::Error::InvalidCertificate(rustls::CertificateError::UnknownIssuer),
        ));
        assert_eq!(classify(&e), Failure::TlsCertificate);
        let e = Error::Io(io::Error::new(
            io::ErrorKind::InvalidData,
            rustls::Error::General("synthetic-private-detail".into()),
        ));
        assert_eq!(classify(&e), Failure::Tls);
        let e = Error::Io(io::Error::new(
            io::ErrorKind::ConnectionRefused,
            "synthetic-private-network-detail",
        ));
        assert_eq!(classify(&e), Failure::Network);
        assert!(!classify(&e).message().contains("synthetic"));
    }

    #[test]
    fn protocol_and_configuration_errors_are_not_account_errors() {
        assert_eq!(
            classify(&Error::Protocol(ProtocolError::WrongHttpVersion)),
            Failure::HttpVersion
        );
        assert_eq!(
            classify(&Error::Protocol(ProtocolError::HandshakeIncomplete)),
            Failure::HandshakeIncomplete
        );
        assert_eq!(
            classify(&Error::Protocol(
                ProtocolError::MissingUpgradeWebSocketHeader
            )),
            Failure::Upgrade
        );
        assert_eq!(
            classify(&Error::Protocol(
                ProtocolError::SecWebSocketAcceptKeyMismatch
            )),
            Failure::Upgrade
        );
        assert_eq!(
            classify(&Error::Utf8("synthetic-private-data".into())),
            Failure::Protocol
        );
    }

    #[test]
    fn http10_response_masks_status_before_auth_classification() {
        use std::io::{Cursor, Read, Write};
        struct Replay(Cursor<Vec<u8>>);
        impl Read for Replay {
            fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
                self.0.read(bytes)
            }
        }
        impl Write for Replay {
            fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
                Ok(bytes.len())
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        for (version, expected) in [
            ("1.0", Failure::HttpVersion),
            ("1.1", Failure::Authentication),
        ] {
            let response = format!("HTTP/{version} 401 Unauthorized\r\nContent-Length: 0\r\n\r\n");
            let stream = Replay(Cursor::new(response.into_bytes()));
            let result = tokio_tungstenite::tungstenite::client(request(None).unwrap(), stream);
            let Err(tokio_tungstenite::tungstenite::HandshakeError::Failure(error)) = result else {
                panic!("expected synthetic handshake failure");
            };
            assert_eq!(classify(&error), expected);
        }
    }
}
