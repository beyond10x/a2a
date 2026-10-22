// generated from a2a v1
// model digest 19f81099dbb59817313f63273ff10776f7d98070eb5de7dda53e5d67514ef5bf
// contract digest b81ca6ca77a699d33385c2f8b48b31221205b1879fc0f96d44d1a695ab4bff58
// do not edit: regenerate with `ess synthesize --layout crate`

//! Discovery — `a2a.discovery`.
//!
//! The Agent Card an A2A agent publishes about itself: identity, interfaces, capabilities, skills and security schemes (A2A v1.0.1 section 4.4, proto messages AgentCard and below).
//!
//! Everything this bounded context declares that the synthesis plan marks generated.

/// APIKeySecurityScheme — `a2a.discovery.APIKeySecurityScheme`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct APIKeySecurityScheme {
    /// `description` — `Optional<String>`.
    pub description: Option<String>,
    /// `location` — `String`.
    pub location: String,
    /// `name` — `String`.
    pub name: String,
}

/// AgentCapabilities — `a2a.discovery.AgentCapabilities`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentCapabilities {
    /// `streaming` — `Optional<Boolean>`.
    pub streaming: Option<bool>,
    /// `push_notifications` — `Optional<Boolean>`.
    pub push_notifications: Option<bool>,
    /// `extensions` — `Optional<List<a2a.discovery.AgentExtension>>`.
    pub extensions: Option<Vec<AgentExtension>>,
    /// `extended_agent_card` — `Optional<Boolean>`.
    pub extended_agent_card: Option<bool>,
}

/// AgentCard — `a2a.discovery.AgentCard`.
///
/// Self-describing manifest of an agent (A2A v1.0.1 section 4.4.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentCard {
    /// `name` — `String`.
    pub name: String,
    /// `description` — `String`.
    pub description: String,
    /// `supported_interfaces` — `List<a2a.discovery.AgentInterface>`.
    pub supported_interfaces: Vec<AgentInterface>,
    /// `provider` — `Optional<a2a.discovery.AgentProvider>`.
    pub provider: Option<AgentProvider>,
    /// `version` — `String`.
    pub version: String,
    /// `documentation_url` — `Optional<String>`.
    pub documentation_url: Option<String>,
    /// `capabilities` — `a2a.discovery.AgentCapabilities`.
    pub capabilities: AgentCapabilities,
    /// `security_schemes` — `Optional<Map<String, a2a.discovery.SecurityScheme>>`.
    pub security_schemes: Option<std::collections::BTreeMap<String, SecurityScheme>>,
    /// `security_requirements` — `Optional<List<a2a.discovery.SecurityRequirement>>`.
    pub security_requirements: Option<Vec<SecurityRequirement>>,
    /// `default_input_modes` — `List<String>`.
    pub default_input_modes: Vec<String>,
    /// `default_output_modes` — `List<String>`.
    pub default_output_modes: Vec<String>,
    /// `skills` — `List<a2a.discovery.AgentSkill>`.
    pub skills: Vec<AgentSkill>,
    /// `signatures` — `Optional<List<a2a.discovery.AgentCardSignature>>`.
    pub signatures: Option<Vec<AgentCardSignature>>,
    /// `icon_url` — `Optional<String>`.
    pub icon_url: Option<String>,
}

/// AgentCardSignature — `a2a.discovery.AgentCardSignature`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentCardSignature {
    /// `protected` — `String`.
    pub protected: String,
    /// `signature` — `String`.
    pub signature: String,
    /// `header` — `Optional<Json>`.
    pub header: Option<crate::json::Value>,
}

/// AgentExtension — `a2a.discovery.AgentExtension`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentExtension {
    /// `uri` — `Optional<String>`.
    pub uri: Option<String>,
    /// `description` — `Optional<String>`.
    pub description: Option<String>,
    /// `required` — `Optional<Boolean>`.
    pub required: Option<bool>,
    /// `params` — `Optional<Json>`.
    pub params: Option<crate::json::Value>,
}

/// AgentInterface — `a2a.discovery.AgentInterface`.
///
/// A URL and protocol binding the agent is reached at (proto AgentInterface).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentInterface {
    /// `url` — `String`.
    pub url: String,
    /// `protocol_binding` — `String`.
    pub protocol_binding: String,
    /// `tenant` — `Optional<String>`.
    pub tenant: Option<String>,
    /// `protocol_version` — `String`.
    pub protocol_version: String,
}

/// AgentProvider — `a2a.discovery.AgentProvider`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentProvider {
    /// `url` — `String`.
    pub url: String,
    /// `organization` — `String`.
    pub organization: String,
}

/// AgentSkill — `a2a.discovery.AgentSkill`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentSkill {
    /// `id` — `String`.
    pub id: String,
    /// `name` — `String`.
    pub name: String,
    /// `description` — `String`.
    pub description: String,
    /// `tags` — `List<String>`.
    pub tags: Vec<String>,
    /// `examples` — `Optional<List<String>>`.
    pub examples: Option<Vec<String>>,
    /// `input_modes` — `Optional<List<String>>`.
    pub input_modes: Option<Vec<String>>,
    /// `output_modes` — `Optional<List<String>>`.
    pub output_modes: Option<Vec<String>>,
    /// `security_requirements` — `Optional<List<a2a.discovery.SecurityRequirement>>`.
    pub security_requirements: Option<Vec<SecurityRequirement>>,
}

/// AuthorizationCodeOAuthFlow — `a2a.discovery.AuthorizationCodeOAuthFlow`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizationCodeOAuthFlow {
    /// `authorization_url` — `String`.
    pub authorization_url: String,
    /// `token_url` — `String`.
    pub token_url: String,
    /// `refresh_url` — `Optional<String>`.
    pub refresh_url: Option<String>,
    /// `scopes` — `Map<String, String>`.
    pub scopes: std::collections::BTreeMap<String, String>,
    /// `pkce_required` — `Optional<Boolean>`.
    pub pkce_required: Option<bool>,
}

/// ClientCredentialsOAuthFlow — `a2a.discovery.ClientCredentialsOAuthFlow`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientCredentialsOAuthFlow {
    /// `token_url` — `String`.
    pub token_url: String,
    /// `refresh_url` — `Optional<String>`.
    pub refresh_url: Option<String>,
    /// `scopes` — `Map<String, String>`.
    pub scopes: std::collections::BTreeMap<String, String>,
}

/// DeviceCodeOAuthFlow — `a2a.discovery.DeviceCodeOAuthFlow`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceCodeOAuthFlow {
    /// `device_authorization_url` — `String`.
    pub device_authorization_url: String,
    /// `token_url` — `String`.
    pub token_url: String,
    /// `refresh_url` — `Optional<String>`.
    pub refresh_url: Option<String>,
    /// `scopes` — `Map<String, String>`.
    pub scopes: std::collections::BTreeMap<String, String>,
}

/// HTTPAuthSecurityScheme — `a2a.discovery.HTTPAuthSecurityScheme`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HTTPAuthSecurityScheme {
    /// `description` — `Optional<String>`.
    pub description: Option<String>,
    /// `scheme` — `String`.
    pub scheme: String,
    /// `bearer_format` — `Optional<String>`.
    pub bearer_format: Option<String>,
}

/// ImplicitOAuthFlow — `a2a.discovery.ImplicitOAuthFlow`.
///
/// Deprecated in A2A v1.0.1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplicitOAuthFlow {
    /// `authorization_url` — `Optional<String>`.
    pub authorization_url: Option<String>,
    /// `refresh_url` — `Optional<String>`.
    pub refresh_url: Option<String>,
    /// `scopes` — `Optional<Map<String, String>>`.
    pub scopes: Option<std::collections::BTreeMap<String, String>>,
}

/// MutualTlsSecurityScheme — `a2a.discovery.MutualTlsSecurityScheme`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MutualTlsSecurityScheme {
    /// `description` — `Optional<String>`.
    pub description: Option<String>,
}

/// OAuth2SecurityScheme — `a2a.discovery.OAuth2SecurityScheme`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OAuth2SecurityScheme {
    /// `description` — `Optional<String>`.
    pub description: Option<String>,
    /// `flows` — `a2a.discovery.OAuthFlows`.
    pub flows: OAuthFlows,
    /// `oauth2_metadata_url` — `Optional<String>`.
    pub oauth2_metadata_url: Option<String>,
}

/// OAuthFlows — `a2a.discovery.OAuthFlows`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OAuthFlows {
    /// `authorization_code` — `Optional<a2a.discovery.AuthorizationCodeOAuthFlow>`.
    pub authorization_code: Option<AuthorizationCodeOAuthFlow>,
    /// `client_credentials` — `Optional<a2a.discovery.ClientCredentialsOAuthFlow>`.
    pub client_credentials: Option<ClientCredentialsOAuthFlow>,
    /// `implicit` — `Optional<a2a.discovery.ImplicitOAuthFlow>`.
    pub implicit: Option<ImplicitOAuthFlow>,
    /// `password` — `Optional<a2a.discovery.PasswordOAuthFlow>`.
    pub password: Option<PasswordOAuthFlow>,
    /// `device_code` — `Optional<a2a.discovery.DeviceCodeOAuthFlow>`.
    pub device_code: Option<DeviceCodeOAuthFlow>,
}

/// OpenIdConnectSecurityScheme — `a2a.discovery.OpenIdConnectSecurityScheme`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenIdConnectSecurityScheme {
    /// `description` — `Optional<String>`.
    pub description: Option<String>,
    /// `open_id_connect_url` — `String`.
    pub open_id_connect_url: String,
}

/// PasswordOAuthFlow — `a2a.discovery.PasswordOAuthFlow`.
///
/// Deprecated in A2A v1.0.1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasswordOAuthFlow {
    /// `token_url` — `Optional<String>`.
    pub token_url: Option<String>,
    /// `refresh_url` — `Optional<String>`.
    pub refresh_url: Option<String>,
    /// `scopes` — `Optional<Map<String, String>>`.
    pub scopes: Option<std::collections::BTreeMap<String, String>>,
}

/// SecurityRequirement — `a2a.discovery.SecurityRequirement`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityRequirement {
    /// `schemes` — `Map<String, a2a.discovery.StringList>`.
    pub schemes: std::collections::BTreeMap<String, StringList>,
}

/// SecurityScheme — `a2a.discovery.SecurityScheme`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityScheme {
    /// `api_key_security_scheme` — `Optional<a2a.discovery.APIKeySecurityScheme>`.
    pub api_key_security_scheme: Option<APIKeySecurityScheme>,
    /// `http_auth_security_scheme` — `Optional<a2a.discovery.HTTPAuthSecurityScheme>`.
    pub http_auth_security_scheme: Option<HTTPAuthSecurityScheme>,
    /// `oauth2_security_scheme` — `Optional<a2a.discovery.OAuth2SecurityScheme>`.
    pub oauth2_security_scheme: Option<OAuth2SecurityScheme>,
    /// `open_id_connect_security_scheme` — `Optional<a2a.discovery.OpenIdConnectSecurityScheme>`.
    pub open_id_connect_security_scheme: Option<OpenIdConnectSecurityScheme>,
    /// `mtls_security_scheme` — `Optional<a2a.discovery.MutualTlsSecurityScheme>`.
    pub mtls_security_scheme: Option<MutualTlsSecurityScheme>,
}

/// StringList — `a2a.discovery.StringList`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringList {
    /// `list` — `List<String>`.
    pub list: Vec<String>,
}

/// Get extended agent card — the input of `a2a.discovery.GetExtendedAgentCard`.
///
/// Return the Agent Card an authenticated client may see, possibly with more skills than the public card (A2A v1.0.1 section 3.1.11; GET /extendedAgentCard).
///
/// Everything it can result in is [`GetExtendedAgentCardOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetExtendedAgentCard {
    /// `tenant` — `Optional<String>`.
    pub tenant: Option<String>,
}

/// Actual typed response of `a2a.discovery.GetExtendedAgentCard`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetExtendedAgentCardResponse {
    /// `card` — `a2a.discovery.AgentCard`.
    pub card: AgentCard,
}

/// Everything `a2a.discovery.GetExtendedAgentCard` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GetExtendedAgentCardOutcome {
    /// `not-supported` — externally decided (the public agent card does not declare capabilities.extendedAgentCard).
    NotSupported {
        /// Why it was refused: `a2a.discovery.UnsupportedOperation`.
        error: UnsupportedOperation,
    },
    /// `not-configured` — externally decided (the agent declares the capability but has no extended card configured).
    NotConfigured {
        /// Why it was refused: `a2a.discovery.ExtendedAgentCardNotConfigured`.
        error: ExtendedAgentCardNotConfigured,
    },
    /// `returned` — otherwise.
    ///
    /// The extended Agent Card.
    Returned {
        /// Actual response returned by this branch.
        response: GetExtendedAgentCardResponse,
    },
    /// `version-not-supported` — externally decided (the A2A-Version service parameter names a version the agent does not support).
    VersionNotSupported {
        /// Why it was refused: `a2a.discovery.VersionNotSupported`.
        error: VersionNotSupported,
    },
    /// `extension-required` — externally decided (the agent card marks an extension required and the request does not declare it).
    ExtensionRequired {
        /// Why it was refused: `a2a.discovery.ExtensionSupportRequired`.
        error: ExtensionSupportRequired,
    },
}

/// The declared error `a2a.discovery.ExtendedAgentCardNotConfigured`.
///
/// ExtendedAgentCardNotConfiguredError (JSON-RPC -32007, HTTP 400): the agent declares the capability but has no extended card configured.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtendedAgentCardNotConfigured;

/// The declared error `a2a.discovery.ExtensionSupportRequired`.
///
/// ExtensionSupportRequiredError (JSON-RPC -32008, gRPC FAILED_PRECONDITION, HTTP 400).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionSupportRequired;

/// The declared error `a2a.discovery.UnsupportedOperation`.
///
/// UnsupportedOperationError (JSON-RPC -32004, HTTP 400): the agent card does not declare capabilities.extendedAgentCard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsupportedOperation;

/// The declared error `a2a.discovery.VersionNotSupported`.
///
/// VersionNotSupportedError (JSON-RPC -32009, gRPC FAILED_PRECONDITION, HTTP 400).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionNotSupported;

/// What this bounded context owes its implementor, as typed seams.
///
/// One trait per obligation in the synthesis plan, each carrying the plan's own contract.
/// [`Unimplemented`](obligations::Unimplemented) satisfies every trait by refusing in the type system, so the workspace builds —
/// and says exactly what it cannot yet do — before a line is hand-written.
pub mod obligations {
    /// The behaviour `a2a.discovery.GetExtendedAgentCard` — an implementation obligation.
    ///
    /// Why it is not generated: kept an obligation by a typed response (`response:`).
    ///
    /// Contract: given `a2a.discovery.GetExtendedAgentCard` input, decide and enact exactly one outcome. Declared outcomes (declaration order, not selection precedence): `not-supported` externally decided (the public agent card does not declare capabilities.extendedAgentCard), error `a2a.discovery.UnsupportedOperation`; `not-configured` externally decided (the agent declares the capability but has no extended card configured), error `a2a.discovery.ExtendedAgentCardNotConfigured`; `returned` otherwise; `version-not-supported` externally decided (the A2A-Version service parameter names a version the agent does not support), error `a2a.discovery.VersionNotSupported`; `extension-required` externally decided (the agent card marks an extension required and the request does not declare it), error `a2a.discovery.ExtensionSupportRequired`.
    pub trait GetExtendedAgentCardBehavior {
        /// Decides and enacts exactly one declared outcome of `a2a.discovery.GetExtendedAgentCard`.
        ///
        /// `Err` is the typed refusal of an obligation nothing has satisfied; a satisfying
        /// implementation never returns it.
        fn get_extended_agent_card(&mut self, input: super::GetExtendedAgentCard) -> Result<super::GetExtendedAgentCardOutcome, crate::obligation::UnmetObligation>;
    }

    /// Every obligation of this bounded context, refused in the type system.
    ///
    /// Each method returns the typed refusal naming what is owed — never a panic, never a guessed
    /// value — so a workspace built on this stub compiles and reports its own gaps.
    pub struct Unimplemented;

    impl GetExtendedAgentCardBehavior for Unimplemented {
        fn get_extended_agent_card(&mut self, _input: super::GetExtendedAgentCard) -> Result<super::GetExtendedAgentCardOutcome, crate::obligation::UnmetObligation> {
            Err(crate::obligation::UnmetObligation { capability: "command behaviour", source: "a2a.discovery.GetExtendedAgentCard" })
        }
    }
}
