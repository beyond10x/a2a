//! What a call can fail with, and the `google.rpc.Status` error answers of the binding decoded.

use std::fmt;

use a2a::json::Value;

/// Why a call did not answer the operation's value.
#[derive(Debug)]
pub enum Error {
    /// The base URL's scheme is not `http`: this client speaks HTTP/1.1 over plain TCP only.
    /// TLS (`https://`) is a later milestone.
    UnsupportedScheme(String),
    /// The request cannot be written: a base URL that is not `http://host[:port][/path]`, or a
    /// header value holding a control character. Nothing was sent.
    InvalidRequest(String),
    /// The connection could not be made, or failed or timed out while writing or reading.
    Transport(std::io::Error),
    /// The agent answered with an error (A2A v1.0.1 section 11.6): a `google.rpc.Status`.
    Status {
        /// The HTTP status of the answer.
        http: u16,
        /// `error.code`.
        code: i64,
        /// `error.status`, the `google.rpc.Code` name such as `NOT_FOUND`; empty where absent.
        status: String,
        /// `error.message`; empty where absent.
        message: String,
        /// The `reason` of the first `google.rpc.ErrorInfo` in `error.details`; `None` where
        /// there is none, as for an error that is not an A2A one.
        reason: Option<Reason>,
    },
    /// The answer is not the binding's: not HTTP/1.x, a body that is not JSON or not the
    /// operation's type, an error body that is no `google.rpc.Status`, a body or stream that
    /// ends early.
    Protocol(String),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedScheme(scheme) => {
                write!(formatter, "unsupported URL scheme `{scheme}`: only http is")
            }
            Self::InvalidRequest(detail) => write!(formatter, "invalid request: {detail}"),
            Self::Transport(error) => write!(formatter, "transport: {error}"),
            Self::Status {
                http,
                status,
                message,
                reason,
                ..
            } => {
                write!(formatter, "{http} {status}: {message}")?;
                match reason {
                    Some(reason) => write!(formatter, " ({})", reason.as_str()),
                    None => Ok(()),
                }
            }
            Self::Protocol(detail) => write!(formatter, "not the A2A HTTP+JSON binding: {detail}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Transport(error) => Some(error),
            _ => None,
        }
    }
}

/// The `google.rpc.ErrorInfo` reason of an A2A error answer (A2A v1.0.1 section 11.6): each A2A
/// error of section 3.3.2, mapped as the table of section 5.4 does, plus the reasons `a2a-server`
/// answers (`crates/a2a-server/src/router.rs`, `declared_error`) and any other as it was sent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reason {
    /// `TASK_NOT_FOUND`: section 5.4, "`TaskNotFoundError` | `-32001` | `NOT_FOUND` |
    /// `404 Not Found`".
    TaskNotFound,
    /// `TASK_NOT_CANCELABLE`: section 5.4, "`TaskNotCancelableError` | `-32002` |
    /// `FAILED_PRECONDITION` | `400 Bad Request`".
    TaskNotCancelable,
    /// `PUSH_NOTIFICATION_NOT_SUPPORTED`: section 5.4, "`PushNotificationNotSupportedError` |
    /// `-32003` | `FAILED_PRECONDITION` | `400 Bad Request`".
    PushNotificationNotSupported,
    /// `UNSUPPORTED_OPERATION`: section 5.4, "`UnsupportedOperationError` | `-32004` |
    /// `FAILED_PRECONDITION` | `400 Bad Request`".
    UnsupportedOperation,
    /// `CONTENT_TYPE_NOT_SUPPORTED`: section 5.4, "`ContentTypeNotSupportedError` | `-32005` |
    /// `INVALID_ARGUMENT` | `400 Bad Request`".
    ContentTypeNotSupported,
    /// `INVALID_AGENT_RESPONSE`: section 5.4, "`InvalidAgentResponseError` | `-32006` |
    /// `INTERNAL` | `500 Internal Server Error`".
    InvalidAgentResponse,
    /// `EXTENDED_AGENT_CARD_NOT_CONFIGURED`: section 5.4,
    /// "`ExtendedAgentCardNotConfiguredError` | `-32007` | `FAILED_PRECONDITION` |
    /// `400 Bad Request`".
    ExtendedAgentCardNotConfigured,
    /// `EXTENSION_SUPPORT_REQUIRED`: section 5.4, "`ExtensionSupportRequiredError` | `-32008` |
    /// `FAILED_PRECONDITION` | `400 Bad Request`".
    ExtensionSupportRequired,
    /// `VERSION_NOT_SUPPORTED`: section 5.4, "`VersionNotSupportedError` | `-32009` |
    /// `FAILED_PRECONDITION` | `400 Bad Request`".
    VersionNotSupported,
    /// `CONTEXT_MISMATCH`: not in section 3.3.2's table; `a2a-server` answers it (400
    /// `INVALID_ARGUMENT`) for a message whose `contextId` is not its task's
    /// (`ess/domains/tasks.yaml`, error `ContextMismatch`).
    ContextMismatch,
    /// `CONTEXT_NOT_ACCEPTED`: not in section 3.3.2's table; `a2a-server` answers it (400
    /// `INVALID_ARGUMENT`) for a `contextId` the agent cannot accept (`ess/domains/tasks.yaml`,
    /// error `ContextNotAccepted`).
    ContextNotAccepted,
    /// Any other reason, as it was sent.
    Other(String),
}

/// Every named reason with its wire form.
const NAMED: [(Reason, &str); 11] = [
    (Reason::TaskNotFound, "TASK_NOT_FOUND"),
    (Reason::TaskNotCancelable, "TASK_NOT_CANCELABLE"),
    (
        Reason::PushNotificationNotSupported,
        "PUSH_NOTIFICATION_NOT_SUPPORTED",
    ),
    (Reason::UnsupportedOperation, "UNSUPPORTED_OPERATION"),
    (
        Reason::ContentTypeNotSupported,
        "CONTENT_TYPE_NOT_SUPPORTED",
    ),
    (Reason::InvalidAgentResponse, "INVALID_AGENT_RESPONSE"),
    (
        Reason::ExtendedAgentCardNotConfigured,
        "EXTENDED_AGENT_CARD_NOT_CONFIGURED",
    ),
    (
        Reason::ExtensionSupportRequired,
        "EXTENSION_SUPPORT_REQUIRED",
    ),
    (Reason::VersionNotSupported, "VERSION_NOT_SUPPORTED"),
    (Reason::ContextMismatch, "CONTEXT_MISMATCH"),
    (Reason::ContextNotAccepted, "CONTEXT_NOT_ACCEPTED"),
];

impl Reason {
    /// The reason as it is sent.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Other(reason) => reason,
            named => NAMED
                .iter()
                .find(|(reason, _)| reason == named)
                .map_or("", |(_, wire)| wire),
        }
    }
}

impl From<&str> for Reason {
    fn from(wire: &str) -> Self {
        NAMED.iter().find(|(_, name)| *name == wire).map_or_else(
            || Self::Other(wire.to_owned()),
            |(reason, _)| reason.clone(),
        )
    }
}

/// The error an answer of status `http` with this body is: `Error::Status` where the body is a
/// `google.rpc.Status`, `Error::Protocol` where it is not.
pub(crate) fn status_error(http: u16, body: &str) -> Error {
    let Ok(value) = a2a::json::parse(body) else {
        return Error::Protocol(format!(
            "the {http} answer's body is not JSON: {}",
            excerpt(body)
        ));
    };
    let Some(error @ Value::Object(_)) = value.member("error") else {
        return Error::Protocol(format!(
            "the {http} answer is not a google.rpc.Status: {}",
            excerpt(body)
        ));
    };
    let code = match error.member("code") {
        Some(Value::Number(digits)) => digits.parse::<i64>().ok(),
        _ => None,
    };
    let Some(code) = code else {
        return Error::Protocol(format!(
            "the {http} answer's error.code is not an integer: {}",
            excerpt(body)
        ));
    };
    let text = |name: &str| match error.member(name) {
        None | Some(Value::Null) => Ok(String::new()),
        Some(Value::Text(text)) => Ok(text.clone()),
        Some(_) => Err(Error::Protocol(format!(
            "the {http} answer's error.{name} is not a string"
        ))),
    };
    let status = match text("status") {
        Ok(status) => status,
        Err(error) => return error,
    };
    let message = match text("message") {
        Ok(message) => message,
        Err(error) => return error,
    };
    let reason = match error.member("details") {
        Some(Value::Array(details)) => details.iter().find_map(|detail| {
            let info = matches!(
                detail.member("@type"),
                Some(Value::Text(kind)) if kind.ends_with("google.rpc.ErrorInfo")
            );
            match detail.member("reason") {
                Some(Value::Text(reason)) if info => Some(Reason::from(reason.as_str())),
                _ => None,
            }
        }),
        _ => None,
    };
    Error::Status {
        http,
        code,
        status,
        message,
        reason,
    }
}

/// The start of a body, for a message about it.
pub(crate) fn excerpt(body: &str) -> String {
    const SHOWN: usize = 200;
    match body.char_indices().nth(SHOWN) {
        Some((at, _)) => format!("{}…", &body[..at]),
        None => body.to_owned(),
    }
}
