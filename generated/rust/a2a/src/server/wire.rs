// generated from a2a v1
// model digest d4cf404c98ec99676cdd8fe8e411c38768d3acb9560e07676018a3ce6a70e28d
// contract digest f51f3ef221113fd930cbd423a63039558132d9f28674116bc714692bd0a5ad44
// do not edit: regenerate with `ess synthesize --layout crate`
//! Every generated declaration, as JSON, in the renderings the published wire contracts fix.
//!
//! Generated from the model beside the types it crosses, so a field renamed in the specification
//! is renamed here in the same regeneration. An absent optional field is omitted rather
//! than sent as `null`, which is what the `required` list of the published schema says.

use crate::server::json;

/// Writes `a2a.discovery.APIKeySecurityScheme` as JSON.
pub fn encode_a2a_discovery_a_p_i_key_security_scheme(value: &crate::discovery::APIKeySecurityScheme, out: &mut String) {
    out.push('{');
    if let Some(held0) = &value.description {
        json::member(out, "description");
        json::push_text(out, &*held0);
    }
    json::member(out, "location");
    json::push_text(out, &value.location);
    json::member(out, "name");
    json::push_text(out, &value.name);
    out.push('}');
}

/// Reads `a2a.discovery.APIKeySecurityScheme` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_discovery_a_p_i_key_security_scheme(value: &json::Value, at: &str) -> Result<crate::discovery::APIKeySecurityScheme, json::DecodeError> {
    Ok(crate::discovery::APIKeySecurityScheme {
        description: match value.member("description") {
            None | Some(json::Value::Null) => None,
            Some(member0) => {
                let at0 = json::nested(at, "description");
                Some(json::text_at(member0, &at0, "a string")?.to_owned())
            }
        },
        location: {
            let at1 = json::nested(at, "location");
            let member1 = json::member_at(value, at, "location")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
        name: {
            let at2 = json::nested(at, "name");
            let member2 = json::member_at(value, at, "name")?;
            json::text_at(member2, &at2, "a string")?.to_owned()
        },
    })
}

/// Writes `a2a.discovery.AgentCapabilities` as JSON.
pub fn encode_a2a_discovery_agent_capabilities(value: &crate::discovery::AgentCapabilities, out: &mut String) {
    out.push('{');
    if let Some(held0) = &value.streaming {
        json::member(out, "streaming");
        json::push_bool(out, *held0);
    }
    if let Some(held0) = &value.push_notifications {
        json::member(out, "pushNotifications");
        json::push_bool(out, *held0);
    }
    if let Some(held0) = &value.extensions {
        json::member(out, "extensions");
        out.push('[');
        for (index1, item1) in (*held0).iter().enumerate() {
            if index1 > 0 {
                out.push(',');
            }
            encode_a2a_discovery_agent_extension(&*item1, out);
        }
        out.push(']');
    }
    if let Some(held0) = &value.extended_agent_card {
        json::member(out, "extendedAgentCard");
        json::push_bool(out, *held0);
    }
    out.push('}');
}

/// Reads `a2a.discovery.AgentCapabilities` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_discovery_agent_capabilities(value: &json::Value, at: &str) -> Result<crate::discovery::AgentCapabilities, json::DecodeError> {
    Ok(crate::discovery::AgentCapabilities {
        streaming: match value.member("streaming") {
            None | Some(json::Value::Null) => None,
            Some(member0) => {
                let at0 = json::nested(at, "streaming");
                Some(json::bool_at(member0, &at0, "a boolean")?)
            }
        },
        push_notifications: match value.member("pushNotifications") {
            None | Some(json::Value::Null) => None,
            Some(member1) => {
                let at1 = json::nested(at, "pushNotifications");
                Some(json::bool_at(member1, &at1, "a boolean")?)
            }
        },
        extensions: match value.member("extensions") {
            None | Some(json::Value::Null) => None,
            Some(member2) => {
                let at2 = json::nested(at, "extensions");
                Some({
                let mut items2 = Vec::new();
                for (index2, element2) in json::items_at(member2, &at2, "an array")?.iter().enumerate() {
                    let nested2 = json::nested(&at2, &index2.to_string());
                    items2.push(decode_a2a_discovery_agent_extension(element2, &nested2)?);
                }
                items2
            })
            }
        },
        extended_agent_card: match value.member("extendedAgentCard") {
            None | Some(json::Value::Null) => None,
            Some(member3) => {
                let at3 = json::nested(at, "extendedAgentCard");
                Some(json::bool_at(member3, &at3, "a boolean")?)
            }
        },
    })
}

/// Writes `a2a.discovery.AgentCard` as JSON.
pub fn encode_a2a_discovery_agent_card(value: &crate::discovery::AgentCard, out: &mut String) {
    out.push('{');
    json::member(out, "name");
    json::push_text(out, &value.name);
    json::member(out, "description");
    json::push_text(out, &value.description);
    json::member(out, "supportedInterfaces");
    out.push('[');
    for (index0, item0) in value.supported_interfaces.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        encode_a2a_discovery_agent_interface(&*item0, out);
    }
    out.push(']');
    if let Some(held0) = &value.provider {
        json::member(out, "provider");
        encode_a2a_discovery_agent_provider(&*held0, out);
    }
    json::member(out, "version");
    json::push_text(out, &value.version);
    if let Some(held0) = &value.documentation_url {
        json::member(out, "documentationUrl");
        json::push_text(out, &*held0);
    }
    json::member(out, "capabilities");
    encode_a2a_discovery_agent_capabilities(&value.capabilities, out);
    if let Some(held0) = &value.security_schemes {
        json::member(out, "securitySchemes");
        out.push('{');
        for (index1, (key1, item1)) in (*held0).iter().enumerate() {
            if index1 > 0 {
                out.push(',');
            }
            json::push_text(out, key1);
            out.push(':');
            encode_a2a_discovery_security_scheme(&*item1, out);
        }
        out.push('}');
    }
    if let Some(held0) = &value.security_requirements {
        json::member(out, "securityRequirements");
        out.push('[');
        for (index1, item1) in (*held0).iter().enumerate() {
            if index1 > 0 {
                out.push(',');
            }
            encode_a2a_discovery_security_requirement(&*item1, out);
        }
        out.push(']');
    }
    json::member(out, "defaultInputModes");
    out.push('[');
    for (index0, item0) in value.default_input_modes.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        json::push_text(out, &*item0);
    }
    out.push(']');
    json::member(out, "defaultOutputModes");
    out.push('[');
    for (index0, item0) in value.default_output_modes.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        json::push_text(out, &*item0);
    }
    out.push(']');
    json::member(out, "skills");
    out.push('[');
    for (index0, item0) in value.skills.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        encode_a2a_discovery_agent_skill(&*item0, out);
    }
    out.push(']');
    if let Some(held0) = &value.signatures {
        json::member(out, "signatures");
        out.push('[');
        for (index1, item1) in (*held0).iter().enumerate() {
            if index1 > 0 {
                out.push(',');
            }
            encode_a2a_discovery_agent_card_signature(&*item1, out);
        }
        out.push(']');
    }
    if let Some(held0) = &value.icon_url {
        json::member(out, "iconUrl");
        json::push_text(out, &*held0);
    }
    out.push('}');
}

/// Reads `a2a.discovery.AgentCard` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_discovery_agent_card(value: &json::Value, at: &str) -> Result<crate::discovery::AgentCard, json::DecodeError> {
    Ok(crate::discovery::AgentCard {
        name: {
            let at0 = json::nested(at, "name");
            let member0 = json::member_at(value, at, "name")?;
            json::text_at(member0, &at0, "a string")?.to_owned()
        },
        description: {
            let at1 = json::nested(at, "description");
            let member1 = json::member_at(value, at, "description")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
        supported_interfaces: {
            let at2 = json::nested(at, "supportedInterfaces");
            let member2 = json::member_at(value, at, "supportedInterfaces")?;
            {
                let mut items2 = Vec::new();
                for (index2, element2) in json::items_at(member2, &at2, "an array")?.iter().enumerate() {
                    let nested2 = json::nested(&at2, &index2.to_string());
                    items2.push(decode_a2a_discovery_agent_interface(element2, &nested2)?);
                }
                items2
            }
        },
        provider: match value.member("provider") {
            None | Some(json::Value::Null) => None,
            Some(member3) => {
                let at3 = json::nested(at, "provider");
                Some(decode_a2a_discovery_agent_provider(member3, &at3)?)
            }
        },
        version: {
            let at4 = json::nested(at, "version");
            let member4 = json::member_at(value, at, "version")?;
            json::text_at(member4, &at4, "a string")?.to_owned()
        },
        documentation_url: match value.member("documentationUrl") {
            None | Some(json::Value::Null) => None,
            Some(member5) => {
                let at5 = json::nested(at, "documentationUrl");
                Some(json::text_at(member5, &at5, "a string")?.to_owned())
            }
        },
        capabilities: {
            let at6 = json::nested(at, "capabilities");
            let member6 = json::member_at(value, at, "capabilities")?;
            decode_a2a_discovery_agent_capabilities(member6, &at6)?
        },
        security_schemes: match value.member("securitySchemes") {
            None | Some(json::Value::Null) => None,
            Some(member7) => {
                let at7 = json::nested(at, "securitySchemes");
                Some({
                let mut entries7 = std::collections::BTreeMap::new();
                for (key7, element7) in json::members_at(member7, &at7, "an object")? {
                    let nested7 = json::nested(&at7, key7);
                    entries7.insert(key7.clone(), decode_a2a_discovery_security_scheme(element7, &nested7)?);
                }
                entries7
            })
            }
        },
        security_requirements: match value.member("securityRequirements") {
            None | Some(json::Value::Null) => None,
            Some(member8) => {
                let at8 = json::nested(at, "securityRequirements");
                Some({
                let mut items8 = Vec::new();
                for (index8, element8) in json::items_at(member8, &at8, "an array")?.iter().enumerate() {
                    let nested8 = json::nested(&at8, &index8.to_string());
                    items8.push(decode_a2a_discovery_security_requirement(element8, &nested8)?);
                }
                items8
            })
            }
        },
        default_input_modes: {
            let at9 = json::nested(at, "defaultInputModes");
            let member9 = json::member_at(value, at, "defaultInputModes")?;
            {
                let mut items9 = Vec::new();
                for (index9, element9) in json::items_at(member9, &at9, "an array")?.iter().enumerate() {
                    let nested9 = json::nested(&at9, &index9.to_string());
                    items9.push(json::text_at(element9, &nested9, "a string")?.to_owned());
                }
                items9
            }
        },
        default_output_modes: {
            let at10 = json::nested(at, "defaultOutputModes");
            let member10 = json::member_at(value, at, "defaultOutputModes")?;
            {
                let mut items10 = Vec::new();
                for (index10, element10) in json::items_at(member10, &at10, "an array")?.iter().enumerate() {
                    let nested10 = json::nested(&at10, &index10.to_string());
                    items10.push(json::text_at(element10, &nested10, "a string")?.to_owned());
                }
                items10
            }
        },
        skills: {
            let at11 = json::nested(at, "skills");
            let member11 = json::member_at(value, at, "skills")?;
            {
                let mut items11 = Vec::new();
                for (index11, element11) in json::items_at(member11, &at11, "an array")?.iter().enumerate() {
                    let nested11 = json::nested(&at11, &index11.to_string());
                    items11.push(decode_a2a_discovery_agent_skill(element11, &nested11)?);
                }
                items11
            }
        },
        signatures: match value.member("signatures") {
            None | Some(json::Value::Null) => None,
            Some(member12) => {
                let at12 = json::nested(at, "signatures");
                Some({
                let mut items12 = Vec::new();
                for (index12, element12) in json::items_at(member12, &at12, "an array")?.iter().enumerate() {
                    let nested12 = json::nested(&at12, &index12.to_string());
                    items12.push(decode_a2a_discovery_agent_card_signature(element12, &nested12)?);
                }
                items12
            })
            }
        },
        icon_url: match value.member("iconUrl") {
            None | Some(json::Value::Null) => None,
            Some(member13) => {
                let at13 = json::nested(at, "iconUrl");
                Some(json::text_at(member13, &at13, "a string")?.to_owned())
            }
        },
    })
}

/// Writes `a2a.discovery.AgentCardSignature` as JSON.
pub fn encode_a2a_discovery_agent_card_signature(value: &crate::discovery::AgentCardSignature, out: &mut String) {
    out.push('{');
    json::member(out, "protected");
    json::push_text(out, &value.protected);
    json::member(out, "signature");
    json::push_text(out, &value.signature);
    if let Some(held0) = &value.header {
        json::member(out, "header");
        json::push_value(out, &*held0);
    }
    out.push('}');
}

/// Reads `a2a.discovery.AgentCardSignature` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_discovery_agent_card_signature(value: &json::Value, at: &str) -> Result<crate::discovery::AgentCardSignature, json::DecodeError> {
    Ok(crate::discovery::AgentCardSignature {
        protected: {
            let at0 = json::nested(at, "protected");
            let member0 = json::member_at(value, at, "protected")?;
            json::text_at(member0, &at0, "a string")?.to_owned()
        },
        signature: {
            let at1 = json::nested(at, "signature");
            let member1 = json::member_at(value, at, "signature")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
        header: match value.member("header") {
            None | Some(json::Value::Null) => None,
            Some(member2) => {
                let at2 = json::nested(at, "header");
                Some(json::value_at(member2, &at2))
            }
        },
    })
}

/// Writes `a2a.discovery.AgentExtension` as JSON.
pub fn encode_a2a_discovery_agent_extension(value: &crate::discovery::AgentExtension, out: &mut String) {
    out.push('{');
    if let Some(held0) = &value.uri {
        json::member(out, "uri");
        json::push_text(out, &*held0);
    }
    if let Some(held0) = &value.description {
        json::member(out, "description");
        json::push_text(out, &*held0);
    }
    if let Some(held0) = &value.required {
        json::member(out, "required");
        json::push_bool(out, *held0);
    }
    if let Some(held0) = &value.params {
        json::member(out, "params");
        json::push_value(out, &*held0);
    }
    out.push('}');
}

/// Reads `a2a.discovery.AgentExtension` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_discovery_agent_extension(value: &json::Value, at: &str) -> Result<crate::discovery::AgentExtension, json::DecodeError> {
    Ok(crate::discovery::AgentExtension {
        uri: match value.member("uri") {
            None | Some(json::Value::Null) => None,
            Some(member0) => {
                let at0 = json::nested(at, "uri");
                Some(json::text_at(member0, &at0, "a string")?.to_owned())
            }
        },
        description: match value.member("description") {
            None | Some(json::Value::Null) => None,
            Some(member1) => {
                let at1 = json::nested(at, "description");
                Some(json::text_at(member1, &at1, "a string")?.to_owned())
            }
        },
        required: match value.member("required") {
            None | Some(json::Value::Null) => None,
            Some(member2) => {
                let at2 = json::nested(at, "required");
                Some(json::bool_at(member2, &at2, "a boolean")?)
            }
        },
        params: match value.member("params") {
            None | Some(json::Value::Null) => None,
            Some(member3) => {
                let at3 = json::nested(at, "params");
                Some(json::value_at(member3, &at3))
            }
        },
    })
}

/// Writes `a2a.discovery.AgentInterface` as JSON.
pub fn encode_a2a_discovery_agent_interface(value: &crate::discovery::AgentInterface, out: &mut String) {
    out.push('{');
    json::member(out, "url");
    json::push_text(out, &value.url);
    json::member(out, "protocolBinding");
    json::push_text(out, &value.protocol_binding);
    if let Some(held0) = &value.tenant {
        json::member(out, "tenant");
        json::push_text(out, &*held0);
    }
    json::member(out, "protocolVersion");
    json::push_text(out, &value.protocol_version);
    out.push('}');
}

/// Reads `a2a.discovery.AgentInterface` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_discovery_agent_interface(value: &json::Value, at: &str) -> Result<crate::discovery::AgentInterface, json::DecodeError> {
    Ok(crate::discovery::AgentInterface {
        url: {
            let at0 = json::nested(at, "url");
            let member0 = json::member_at(value, at, "url")?;
            json::text_at(member0, &at0, "a string")?.to_owned()
        },
        protocol_binding: {
            let at1 = json::nested(at, "protocolBinding");
            let member1 = json::member_at(value, at, "protocolBinding")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
        tenant: match value.member("tenant") {
            None | Some(json::Value::Null) => None,
            Some(member2) => {
                let at2 = json::nested(at, "tenant");
                Some(json::text_at(member2, &at2, "a string")?.to_owned())
            }
        },
        protocol_version: {
            let at3 = json::nested(at, "protocolVersion");
            let member3 = json::member_at(value, at, "protocolVersion")?;
            json::text_at(member3, &at3, "a string")?.to_owned()
        },
    })
}

/// Writes `a2a.discovery.AgentProvider` as JSON.
pub fn encode_a2a_discovery_agent_provider(value: &crate::discovery::AgentProvider, out: &mut String) {
    out.push('{');
    json::member(out, "url");
    json::push_text(out, &value.url);
    json::member(out, "organization");
    json::push_text(out, &value.organization);
    out.push('}');
}

/// Reads `a2a.discovery.AgentProvider` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_discovery_agent_provider(value: &json::Value, at: &str) -> Result<crate::discovery::AgentProvider, json::DecodeError> {
    Ok(crate::discovery::AgentProvider {
        url: {
            let at0 = json::nested(at, "url");
            let member0 = json::member_at(value, at, "url")?;
            json::text_at(member0, &at0, "a string")?.to_owned()
        },
        organization: {
            let at1 = json::nested(at, "organization");
            let member1 = json::member_at(value, at, "organization")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
    })
}

/// Writes `a2a.discovery.AgentSkill` as JSON.
pub fn encode_a2a_discovery_agent_skill(value: &crate::discovery::AgentSkill, out: &mut String) {
    out.push('{');
    json::member(out, "id");
    json::push_text(out, &value.id);
    json::member(out, "name");
    json::push_text(out, &value.name);
    json::member(out, "description");
    json::push_text(out, &value.description);
    json::member(out, "tags");
    out.push('[');
    for (index0, item0) in value.tags.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        json::push_text(out, &*item0);
    }
    out.push(']');
    if let Some(held0) = &value.examples {
        json::member(out, "examples");
        out.push('[');
        for (index1, item1) in (*held0).iter().enumerate() {
            if index1 > 0 {
                out.push(',');
            }
            json::push_text(out, &*item1);
        }
        out.push(']');
    }
    if let Some(held0) = &value.input_modes {
        json::member(out, "inputModes");
        out.push('[');
        for (index1, item1) in (*held0).iter().enumerate() {
            if index1 > 0 {
                out.push(',');
            }
            json::push_text(out, &*item1);
        }
        out.push(']');
    }
    if let Some(held0) = &value.output_modes {
        json::member(out, "outputModes");
        out.push('[');
        for (index1, item1) in (*held0).iter().enumerate() {
            if index1 > 0 {
                out.push(',');
            }
            json::push_text(out, &*item1);
        }
        out.push(']');
    }
    if let Some(held0) = &value.security_requirements {
        json::member(out, "securityRequirements");
        out.push('[');
        for (index1, item1) in (*held0).iter().enumerate() {
            if index1 > 0 {
                out.push(',');
            }
            encode_a2a_discovery_security_requirement(&*item1, out);
        }
        out.push(']');
    }
    out.push('}');
}

/// Reads `a2a.discovery.AgentSkill` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_discovery_agent_skill(value: &json::Value, at: &str) -> Result<crate::discovery::AgentSkill, json::DecodeError> {
    Ok(crate::discovery::AgentSkill {
        id: {
            let at0 = json::nested(at, "id");
            let member0 = json::member_at(value, at, "id")?;
            json::text_at(member0, &at0, "a string")?.to_owned()
        },
        name: {
            let at1 = json::nested(at, "name");
            let member1 = json::member_at(value, at, "name")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
        description: {
            let at2 = json::nested(at, "description");
            let member2 = json::member_at(value, at, "description")?;
            json::text_at(member2, &at2, "a string")?.to_owned()
        },
        tags: {
            let at3 = json::nested(at, "tags");
            let member3 = json::member_at(value, at, "tags")?;
            {
                let mut items3 = Vec::new();
                for (index3, element3) in json::items_at(member3, &at3, "an array")?.iter().enumerate() {
                    let nested3 = json::nested(&at3, &index3.to_string());
                    items3.push(json::text_at(element3, &nested3, "a string")?.to_owned());
                }
                items3
            }
        },
        examples: match value.member("examples") {
            None | Some(json::Value::Null) => None,
            Some(member4) => {
                let at4 = json::nested(at, "examples");
                Some({
                let mut items4 = Vec::new();
                for (index4, element4) in json::items_at(member4, &at4, "an array")?.iter().enumerate() {
                    let nested4 = json::nested(&at4, &index4.to_string());
                    items4.push(json::text_at(element4, &nested4, "a string")?.to_owned());
                }
                items4
            })
            }
        },
        input_modes: match value.member("inputModes") {
            None | Some(json::Value::Null) => None,
            Some(member5) => {
                let at5 = json::nested(at, "inputModes");
                Some({
                let mut items5 = Vec::new();
                for (index5, element5) in json::items_at(member5, &at5, "an array")?.iter().enumerate() {
                    let nested5 = json::nested(&at5, &index5.to_string());
                    items5.push(json::text_at(element5, &nested5, "a string")?.to_owned());
                }
                items5
            })
            }
        },
        output_modes: match value.member("outputModes") {
            None | Some(json::Value::Null) => None,
            Some(member6) => {
                let at6 = json::nested(at, "outputModes");
                Some({
                let mut items6 = Vec::new();
                for (index6, element6) in json::items_at(member6, &at6, "an array")?.iter().enumerate() {
                    let nested6 = json::nested(&at6, &index6.to_string());
                    items6.push(json::text_at(element6, &nested6, "a string")?.to_owned());
                }
                items6
            })
            }
        },
        security_requirements: match value.member("securityRequirements") {
            None | Some(json::Value::Null) => None,
            Some(member7) => {
                let at7 = json::nested(at, "securityRequirements");
                Some({
                let mut items7 = Vec::new();
                for (index7, element7) in json::items_at(member7, &at7, "an array")?.iter().enumerate() {
                    let nested7 = json::nested(&at7, &index7.to_string());
                    items7.push(decode_a2a_discovery_security_requirement(element7, &nested7)?);
                }
                items7
            })
            }
        },
    })
}

/// Writes `a2a.discovery.AuthorizationCodeOAuthFlow` as JSON.
pub fn encode_a2a_discovery_authorization_code_o_auth_flow(value: &crate::discovery::AuthorizationCodeOAuthFlow, out: &mut String) {
    out.push('{');
    json::member(out, "authorizationUrl");
    json::push_text(out, &value.authorization_url);
    json::member(out, "tokenUrl");
    json::push_text(out, &value.token_url);
    if let Some(held0) = &value.refresh_url {
        json::member(out, "refreshUrl");
        json::push_text(out, &*held0);
    }
    json::member(out, "scopes");
    out.push('{');
    for (index0, (key0, item0)) in value.scopes.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        json::push_text(out, key0);
        out.push(':');
        json::push_text(out, &*item0);
    }
    out.push('}');
    if let Some(held0) = &value.pkce_required {
        json::member(out, "pkceRequired");
        json::push_bool(out, *held0);
    }
    out.push('}');
}

/// Reads `a2a.discovery.AuthorizationCodeOAuthFlow` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_discovery_authorization_code_o_auth_flow(value: &json::Value, at: &str) -> Result<crate::discovery::AuthorizationCodeOAuthFlow, json::DecodeError> {
    Ok(crate::discovery::AuthorizationCodeOAuthFlow {
        authorization_url: {
            let at0 = json::nested(at, "authorizationUrl");
            let member0 = json::member_at(value, at, "authorizationUrl")?;
            json::text_at(member0, &at0, "a string")?.to_owned()
        },
        token_url: {
            let at1 = json::nested(at, "tokenUrl");
            let member1 = json::member_at(value, at, "tokenUrl")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
        refresh_url: match value.member("refreshUrl") {
            None | Some(json::Value::Null) => None,
            Some(member2) => {
                let at2 = json::nested(at, "refreshUrl");
                Some(json::text_at(member2, &at2, "a string")?.to_owned())
            }
        },
        scopes: {
            let at3 = json::nested(at, "scopes");
            let member3 = json::member_at(value, at, "scopes")?;
            {
                let mut entries3 = std::collections::BTreeMap::new();
                for (key3, element3) in json::members_at(member3, &at3, "an object")? {
                    let nested3 = json::nested(&at3, key3);
                    entries3.insert(key3.clone(), json::text_at(element3, &nested3, "a string")?.to_owned());
                }
                entries3
            }
        },
        pkce_required: match value.member("pkceRequired") {
            None | Some(json::Value::Null) => None,
            Some(member4) => {
                let at4 = json::nested(at, "pkceRequired");
                Some(json::bool_at(member4, &at4, "a boolean")?)
            }
        },
    })
}

/// Writes `a2a.discovery.ClientCredentialsOAuthFlow` as JSON.
pub fn encode_a2a_discovery_client_credentials_o_auth_flow(value: &crate::discovery::ClientCredentialsOAuthFlow, out: &mut String) {
    out.push('{');
    json::member(out, "tokenUrl");
    json::push_text(out, &value.token_url);
    if let Some(held0) = &value.refresh_url {
        json::member(out, "refreshUrl");
        json::push_text(out, &*held0);
    }
    json::member(out, "scopes");
    out.push('{');
    for (index0, (key0, item0)) in value.scopes.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        json::push_text(out, key0);
        out.push(':');
        json::push_text(out, &*item0);
    }
    out.push('}');
    out.push('}');
}

/// Reads `a2a.discovery.ClientCredentialsOAuthFlow` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_discovery_client_credentials_o_auth_flow(value: &json::Value, at: &str) -> Result<crate::discovery::ClientCredentialsOAuthFlow, json::DecodeError> {
    Ok(crate::discovery::ClientCredentialsOAuthFlow {
        token_url: {
            let at0 = json::nested(at, "tokenUrl");
            let member0 = json::member_at(value, at, "tokenUrl")?;
            json::text_at(member0, &at0, "a string")?.to_owned()
        },
        refresh_url: match value.member("refreshUrl") {
            None | Some(json::Value::Null) => None,
            Some(member1) => {
                let at1 = json::nested(at, "refreshUrl");
                Some(json::text_at(member1, &at1, "a string")?.to_owned())
            }
        },
        scopes: {
            let at2 = json::nested(at, "scopes");
            let member2 = json::member_at(value, at, "scopes")?;
            {
                let mut entries2 = std::collections::BTreeMap::new();
                for (key2, element2) in json::members_at(member2, &at2, "an object")? {
                    let nested2 = json::nested(&at2, key2);
                    entries2.insert(key2.clone(), json::text_at(element2, &nested2, "a string")?.to_owned());
                }
                entries2
            }
        },
    })
}

/// Writes `a2a.discovery.DeviceCodeOAuthFlow` as JSON.
pub fn encode_a2a_discovery_device_code_o_auth_flow(value: &crate::discovery::DeviceCodeOAuthFlow, out: &mut String) {
    out.push('{');
    json::member(out, "deviceAuthorizationUrl");
    json::push_text(out, &value.device_authorization_url);
    json::member(out, "tokenUrl");
    json::push_text(out, &value.token_url);
    if let Some(held0) = &value.refresh_url {
        json::member(out, "refreshUrl");
        json::push_text(out, &*held0);
    }
    json::member(out, "scopes");
    out.push('{');
    for (index0, (key0, item0)) in value.scopes.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        json::push_text(out, key0);
        out.push(':');
        json::push_text(out, &*item0);
    }
    out.push('}');
    out.push('}');
}

/// Reads `a2a.discovery.DeviceCodeOAuthFlow` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_discovery_device_code_o_auth_flow(value: &json::Value, at: &str) -> Result<crate::discovery::DeviceCodeOAuthFlow, json::DecodeError> {
    Ok(crate::discovery::DeviceCodeOAuthFlow {
        device_authorization_url: {
            let at0 = json::nested(at, "deviceAuthorizationUrl");
            let member0 = json::member_at(value, at, "deviceAuthorizationUrl")?;
            json::text_at(member0, &at0, "a string")?.to_owned()
        },
        token_url: {
            let at1 = json::nested(at, "tokenUrl");
            let member1 = json::member_at(value, at, "tokenUrl")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
        refresh_url: match value.member("refreshUrl") {
            None | Some(json::Value::Null) => None,
            Some(member2) => {
                let at2 = json::nested(at, "refreshUrl");
                Some(json::text_at(member2, &at2, "a string")?.to_owned())
            }
        },
        scopes: {
            let at3 = json::nested(at, "scopes");
            let member3 = json::member_at(value, at, "scopes")?;
            {
                let mut entries3 = std::collections::BTreeMap::new();
                for (key3, element3) in json::members_at(member3, &at3, "an object")? {
                    let nested3 = json::nested(&at3, key3);
                    entries3.insert(key3.clone(), json::text_at(element3, &nested3, "a string")?.to_owned());
                }
                entries3
            }
        },
    })
}

/// Writes `a2a.discovery.HTTPAuthSecurityScheme` as JSON.
pub fn encode_a2a_discovery_h_t_t_p_auth_security_scheme(value: &crate::discovery::HTTPAuthSecurityScheme, out: &mut String) {
    out.push('{');
    if let Some(held0) = &value.description {
        json::member(out, "description");
        json::push_text(out, &*held0);
    }
    json::member(out, "scheme");
    json::push_text(out, &value.scheme);
    if let Some(held0) = &value.bearer_format {
        json::member(out, "bearerFormat");
        json::push_text(out, &*held0);
    }
    out.push('}');
}

/// Reads `a2a.discovery.HTTPAuthSecurityScheme` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_discovery_h_t_t_p_auth_security_scheme(value: &json::Value, at: &str) -> Result<crate::discovery::HTTPAuthSecurityScheme, json::DecodeError> {
    Ok(crate::discovery::HTTPAuthSecurityScheme {
        description: match value.member("description") {
            None | Some(json::Value::Null) => None,
            Some(member0) => {
                let at0 = json::nested(at, "description");
                Some(json::text_at(member0, &at0, "a string")?.to_owned())
            }
        },
        scheme: {
            let at1 = json::nested(at, "scheme");
            let member1 = json::member_at(value, at, "scheme")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
        bearer_format: match value.member("bearerFormat") {
            None | Some(json::Value::Null) => None,
            Some(member2) => {
                let at2 = json::nested(at, "bearerFormat");
                Some(json::text_at(member2, &at2, "a string")?.to_owned())
            }
        },
    })
}

/// Writes `a2a.discovery.ImplicitOAuthFlow` as JSON.
pub fn encode_a2a_discovery_implicit_o_auth_flow(value: &crate::discovery::ImplicitOAuthFlow, out: &mut String) {
    out.push('{');
    if let Some(held0) = &value.authorization_url {
        json::member(out, "authorizationUrl");
        json::push_text(out, &*held0);
    }
    if let Some(held0) = &value.refresh_url {
        json::member(out, "refreshUrl");
        json::push_text(out, &*held0);
    }
    if let Some(held0) = &value.scopes {
        json::member(out, "scopes");
        out.push('{');
        for (index1, (key1, item1)) in (*held0).iter().enumerate() {
            if index1 > 0 {
                out.push(',');
            }
            json::push_text(out, key1);
            out.push(':');
            json::push_text(out, &*item1);
        }
        out.push('}');
    }
    out.push('}');
}

/// Reads `a2a.discovery.ImplicitOAuthFlow` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_discovery_implicit_o_auth_flow(value: &json::Value, at: &str) -> Result<crate::discovery::ImplicitOAuthFlow, json::DecodeError> {
    Ok(crate::discovery::ImplicitOAuthFlow {
        authorization_url: match value.member("authorizationUrl") {
            None | Some(json::Value::Null) => None,
            Some(member0) => {
                let at0 = json::nested(at, "authorizationUrl");
                Some(json::text_at(member0, &at0, "a string")?.to_owned())
            }
        },
        refresh_url: match value.member("refreshUrl") {
            None | Some(json::Value::Null) => None,
            Some(member1) => {
                let at1 = json::nested(at, "refreshUrl");
                Some(json::text_at(member1, &at1, "a string")?.to_owned())
            }
        },
        scopes: match value.member("scopes") {
            None | Some(json::Value::Null) => None,
            Some(member2) => {
                let at2 = json::nested(at, "scopes");
                Some({
                let mut entries2 = std::collections::BTreeMap::new();
                for (key2, element2) in json::members_at(member2, &at2, "an object")? {
                    let nested2 = json::nested(&at2, key2);
                    entries2.insert(key2.clone(), json::text_at(element2, &nested2, "a string")?.to_owned());
                }
                entries2
            })
            }
        },
    })
}

/// Writes `a2a.discovery.MutualTlsSecurityScheme` as JSON.
pub fn encode_a2a_discovery_mutual_tls_security_scheme(value: &crate::discovery::MutualTlsSecurityScheme, out: &mut String) {
    out.push('{');
    if let Some(held0) = &value.description {
        json::member(out, "description");
        json::push_text(out, &*held0);
    }
    out.push('}');
}

/// Reads `a2a.discovery.MutualTlsSecurityScheme` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_discovery_mutual_tls_security_scheme(value: &json::Value, at: &str) -> Result<crate::discovery::MutualTlsSecurityScheme, json::DecodeError> {
    Ok(crate::discovery::MutualTlsSecurityScheme {
        description: match value.member("description") {
            None | Some(json::Value::Null) => None,
            Some(member0) => {
                let at0 = json::nested(at, "description");
                Some(json::text_at(member0, &at0, "a string")?.to_owned())
            }
        },
    })
}

/// Writes `a2a.discovery.OAuth2SecurityScheme` as JSON.
pub fn encode_a2a_discovery_o_auth2_security_scheme(value: &crate::discovery::OAuth2SecurityScheme, out: &mut String) {
    out.push('{');
    if let Some(held0) = &value.description {
        json::member(out, "description");
        json::push_text(out, &*held0);
    }
    json::member(out, "flows");
    encode_a2a_discovery_o_auth_flows(&value.flows, out);
    if let Some(held0) = &value.oauth2_metadata_url {
        json::member(out, "oauth2MetadataUrl");
        json::push_text(out, &*held0);
    }
    out.push('}');
}

/// Reads `a2a.discovery.OAuth2SecurityScheme` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_discovery_o_auth2_security_scheme(value: &json::Value, at: &str) -> Result<crate::discovery::OAuth2SecurityScheme, json::DecodeError> {
    Ok(crate::discovery::OAuth2SecurityScheme {
        description: match value.member("description") {
            None | Some(json::Value::Null) => None,
            Some(member0) => {
                let at0 = json::nested(at, "description");
                Some(json::text_at(member0, &at0, "a string")?.to_owned())
            }
        },
        flows: {
            let at1 = json::nested(at, "flows");
            let member1 = json::member_at(value, at, "flows")?;
            decode_a2a_discovery_o_auth_flows(member1, &at1)?
        },
        oauth2_metadata_url: match value.member("oauth2MetadataUrl") {
            None | Some(json::Value::Null) => None,
            Some(member2) => {
                let at2 = json::nested(at, "oauth2MetadataUrl");
                Some(json::text_at(member2, &at2, "a string")?.to_owned())
            }
        },
    })
}

/// Writes `a2a.discovery.OAuthFlows` as JSON.
pub fn encode_a2a_discovery_o_auth_flows(value: &crate::discovery::OAuthFlows, out: &mut String) {
    out.push('{');
    if let Some(held0) = &value.authorization_code {
        json::member(out, "authorizationCode");
        encode_a2a_discovery_authorization_code_o_auth_flow(&*held0, out);
    }
    if let Some(held0) = &value.client_credentials {
        json::member(out, "clientCredentials");
        encode_a2a_discovery_client_credentials_o_auth_flow(&*held0, out);
    }
    if let Some(held0) = &value.implicit {
        json::member(out, "implicit");
        encode_a2a_discovery_implicit_o_auth_flow(&*held0, out);
    }
    if let Some(held0) = &value.password {
        json::member(out, "password");
        encode_a2a_discovery_password_o_auth_flow(&*held0, out);
    }
    if let Some(held0) = &value.device_code {
        json::member(out, "deviceCode");
        encode_a2a_discovery_device_code_o_auth_flow(&*held0, out);
    }
    out.push('}');
}

/// Reads `a2a.discovery.OAuthFlows` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_discovery_o_auth_flows(value: &json::Value, at: &str) -> Result<crate::discovery::OAuthFlows, json::DecodeError> {
    Ok(crate::discovery::OAuthFlows {
        authorization_code: match value.member("authorizationCode") {
            None | Some(json::Value::Null) => None,
            Some(member0) => {
                let at0 = json::nested(at, "authorizationCode");
                Some(decode_a2a_discovery_authorization_code_o_auth_flow(member0, &at0)?)
            }
        },
        client_credentials: match value.member("clientCredentials") {
            None | Some(json::Value::Null) => None,
            Some(member1) => {
                let at1 = json::nested(at, "clientCredentials");
                Some(decode_a2a_discovery_client_credentials_o_auth_flow(member1, &at1)?)
            }
        },
        implicit: match value.member("implicit") {
            None | Some(json::Value::Null) => None,
            Some(member2) => {
                let at2 = json::nested(at, "implicit");
                Some(decode_a2a_discovery_implicit_o_auth_flow(member2, &at2)?)
            }
        },
        password: match value.member("password") {
            None | Some(json::Value::Null) => None,
            Some(member3) => {
                let at3 = json::nested(at, "password");
                Some(decode_a2a_discovery_password_o_auth_flow(member3, &at3)?)
            }
        },
        device_code: match value.member("deviceCode") {
            None | Some(json::Value::Null) => None,
            Some(member4) => {
                let at4 = json::nested(at, "deviceCode");
                Some(decode_a2a_discovery_device_code_o_auth_flow(member4, &at4)?)
            }
        },
    })
}

/// Writes `a2a.discovery.OpenIdConnectSecurityScheme` as JSON.
pub fn encode_a2a_discovery_open_id_connect_security_scheme(value: &crate::discovery::OpenIdConnectSecurityScheme, out: &mut String) {
    out.push('{');
    if let Some(held0) = &value.description {
        json::member(out, "description");
        json::push_text(out, &*held0);
    }
    json::member(out, "openIdConnectUrl");
    json::push_text(out, &value.open_id_connect_url);
    out.push('}');
}

/// Reads `a2a.discovery.OpenIdConnectSecurityScheme` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_discovery_open_id_connect_security_scheme(value: &json::Value, at: &str) -> Result<crate::discovery::OpenIdConnectSecurityScheme, json::DecodeError> {
    Ok(crate::discovery::OpenIdConnectSecurityScheme {
        description: match value.member("description") {
            None | Some(json::Value::Null) => None,
            Some(member0) => {
                let at0 = json::nested(at, "description");
                Some(json::text_at(member0, &at0, "a string")?.to_owned())
            }
        },
        open_id_connect_url: {
            let at1 = json::nested(at, "openIdConnectUrl");
            let member1 = json::member_at(value, at, "openIdConnectUrl")?;
            json::text_at(member1, &at1, "a string")?.to_owned()
        },
    })
}

/// Writes `a2a.discovery.PasswordOAuthFlow` as JSON.
pub fn encode_a2a_discovery_password_o_auth_flow(value: &crate::discovery::PasswordOAuthFlow, out: &mut String) {
    out.push('{');
    if let Some(held0) = &value.token_url {
        json::member(out, "tokenUrl");
        json::push_text(out, &*held0);
    }
    if let Some(held0) = &value.refresh_url {
        json::member(out, "refreshUrl");
        json::push_text(out, &*held0);
    }
    if let Some(held0) = &value.scopes {
        json::member(out, "scopes");
        out.push('{');
        for (index1, (key1, item1)) in (*held0).iter().enumerate() {
            if index1 > 0 {
                out.push(',');
            }
            json::push_text(out, key1);
            out.push(':');
            json::push_text(out, &*item1);
        }
        out.push('}');
    }
    out.push('}');
}

/// Reads `a2a.discovery.PasswordOAuthFlow` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_discovery_password_o_auth_flow(value: &json::Value, at: &str) -> Result<crate::discovery::PasswordOAuthFlow, json::DecodeError> {
    Ok(crate::discovery::PasswordOAuthFlow {
        token_url: match value.member("tokenUrl") {
            None | Some(json::Value::Null) => None,
            Some(member0) => {
                let at0 = json::nested(at, "tokenUrl");
                Some(json::text_at(member0, &at0, "a string")?.to_owned())
            }
        },
        refresh_url: match value.member("refreshUrl") {
            None | Some(json::Value::Null) => None,
            Some(member1) => {
                let at1 = json::nested(at, "refreshUrl");
                Some(json::text_at(member1, &at1, "a string")?.to_owned())
            }
        },
        scopes: match value.member("scopes") {
            None | Some(json::Value::Null) => None,
            Some(member2) => {
                let at2 = json::nested(at, "scopes");
                Some({
                let mut entries2 = std::collections::BTreeMap::new();
                for (key2, element2) in json::members_at(member2, &at2, "an object")? {
                    let nested2 = json::nested(&at2, key2);
                    entries2.insert(key2.clone(), json::text_at(element2, &nested2, "a string")?.to_owned());
                }
                entries2
            })
            }
        },
    })
}

/// Writes `a2a.discovery.SecurityRequirement` as JSON.
pub fn encode_a2a_discovery_security_requirement(value: &crate::discovery::SecurityRequirement, out: &mut String) {
    out.push('{');
    json::member(out, "schemes");
    out.push('{');
    for (index0, (key0, item0)) in value.schemes.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        json::push_text(out, key0);
        out.push(':');
        encode_a2a_discovery_string_list(&*item0, out);
    }
    out.push('}');
    out.push('}');
}

/// Reads `a2a.discovery.SecurityRequirement` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_discovery_security_requirement(value: &json::Value, at: &str) -> Result<crate::discovery::SecurityRequirement, json::DecodeError> {
    Ok(crate::discovery::SecurityRequirement {
        schemes: {
            let at0 = json::nested(at, "schemes");
            let member0 = json::member_at(value, at, "schemes")?;
            {
                let mut entries0 = std::collections::BTreeMap::new();
                for (key0, element0) in json::members_at(member0, &at0, "an object")? {
                    let nested0 = json::nested(&at0, key0);
                    entries0.insert(key0.clone(), decode_a2a_discovery_string_list(element0, &nested0)?);
                }
                entries0
            }
        },
    })
}

/// Writes `a2a.discovery.SecurityScheme` as JSON.
pub fn encode_a2a_discovery_security_scheme(value: &crate::discovery::SecurityScheme, out: &mut String) {
    out.push('{');
    if let Some(held0) = &value.api_key_security_scheme {
        json::member(out, "apiKeySecurityScheme");
        encode_a2a_discovery_a_p_i_key_security_scheme(&*held0, out);
    }
    if let Some(held0) = &value.http_auth_security_scheme {
        json::member(out, "httpAuthSecurityScheme");
        encode_a2a_discovery_h_t_t_p_auth_security_scheme(&*held0, out);
    }
    if let Some(held0) = &value.oauth2_security_scheme {
        json::member(out, "oauth2SecurityScheme");
        encode_a2a_discovery_o_auth2_security_scheme(&*held0, out);
    }
    if let Some(held0) = &value.open_id_connect_security_scheme {
        json::member(out, "openIdConnectSecurityScheme");
        encode_a2a_discovery_open_id_connect_security_scheme(&*held0, out);
    }
    if let Some(held0) = &value.mtls_security_scheme {
        json::member(out, "mtlsSecurityScheme");
        encode_a2a_discovery_mutual_tls_security_scheme(&*held0, out);
    }
    out.push('}');
}

/// Reads `a2a.discovery.SecurityScheme` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_discovery_security_scheme(value: &json::Value, at: &str) -> Result<crate::discovery::SecurityScheme, json::DecodeError> {
    Ok(crate::discovery::SecurityScheme {
        api_key_security_scheme: match value.member("apiKeySecurityScheme") {
            None | Some(json::Value::Null) => None,
            Some(member0) => {
                let at0 = json::nested(at, "apiKeySecurityScheme");
                Some(decode_a2a_discovery_a_p_i_key_security_scheme(member0, &at0)?)
            }
        },
        http_auth_security_scheme: match value.member("httpAuthSecurityScheme") {
            None | Some(json::Value::Null) => None,
            Some(member1) => {
                let at1 = json::nested(at, "httpAuthSecurityScheme");
                Some(decode_a2a_discovery_h_t_t_p_auth_security_scheme(member1, &at1)?)
            }
        },
        oauth2_security_scheme: match value.member("oauth2SecurityScheme") {
            None | Some(json::Value::Null) => None,
            Some(member2) => {
                let at2 = json::nested(at, "oauth2SecurityScheme");
                Some(decode_a2a_discovery_o_auth2_security_scheme(member2, &at2)?)
            }
        },
        open_id_connect_security_scheme: match value.member("openIdConnectSecurityScheme") {
            None | Some(json::Value::Null) => None,
            Some(member3) => {
                let at3 = json::nested(at, "openIdConnectSecurityScheme");
                Some(decode_a2a_discovery_open_id_connect_security_scheme(member3, &at3)?)
            }
        },
        mtls_security_scheme: match value.member("mtlsSecurityScheme") {
            None | Some(json::Value::Null) => None,
            Some(member4) => {
                let at4 = json::nested(at, "mtlsSecurityScheme");
                Some(decode_a2a_discovery_mutual_tls_security_scheme(member4, &at4)?)
            }
        },
    })
}

/// Writes `a2a.discovery.StringList` as JSON.
pub fn encode_a2a_discovery_string_list(value: &crate::discovery::StringList, out: &mut String) {
    out.push('{');
    json::member(out, "list");
    out.push('[');
    for (index0, item0) in value.list.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        json::push_text(out, &*item0);
    }
    out.push(']');
    out.push('}');
}

/// Reads `a2a.discovery.StringList` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_discovery_string_list(value: &json::Value, at: &str) -> Result<crate::discovery::StringList, json::DecodeError> {
    Ok(crate::discovery::StringList {
        list: {
            let at0 = json::nested(at, "list");
            let member0 = json::member_at(value, at, "list")?;
            {
                let mut items0 = Vec::new();
                for (index0, element0) in json::items_at(member0, &at0, "an array")?.iter().enumerate() {
                    let nested0 = json::nested(&at0, &index0.to_string());
                    items0.push(json::text_at(element0, &nested0, "a string")?.to_owned());
                }
                items0
            }
        },
    })
}

/// Writes `a2a.tasks.Artifact` as JSON.
pub fn encode_a2a_tasks_artifact(value: &crate::tasks::Artifact, out: &mut String) {
    out.push('{');
    json::member(out, "artifactId");
    json::push_text(out, &value.artifact_id);
    if let Some(held0) = &value.name {
        json::member(out, "name");
        json::push_text(out, &*held0);
    }
    if let Some(held0) = &value.description {
        json::member(out, "description");
        json::push_text(out, &*held0);
    }
    json::member(out, "parts");
    out.push('[');
    for (index0, item0) in value.parts.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        encode_a2a_tasks_part(&*item0, out);
    }
    out.push(']');
    if let Some(held0) = &value.metadata {
        json::member(out, "metadata");
        json::push_value(out, &*held0);
    }
    if let Some(held0) = &value.extensions {
        json::member(out, "extensions");
        out.push('[');
        for (index1, item1) in (*held0).iter().enumerate() {
            if index1 > 0 {
                out.push(',');
            }
            json::push_text(out, &*item1);
        }
        out.push(']');
    }
    out.push('}');
}

/// Reads `a2a.tasks.Artifact` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_tasks_artifact(value: &json::Value, at: &str) -> Result<crate::tasks::Artifact, json::DecodeError> {
    Ok(crate::tasks::Artifact {
        artifact_id: {
            let at0 = json::nested(at, "artifactId");
            let member0 = json::member_at(value, at, "artifactId")?;
            json::text_at(member0, &at0, "a string")?.to_owned()
        },
        name: match value.member("name") {
            None | Some(json::Value::Null) => None,
            Some(member1) => {
                let at1 = json::nested(at, "name");
                Some(json::text_at(member1, &at1, "a string")?.to_owned())
            }
        },
        description: match value.member("description") {
            None | Some(json::Value::Null) => None,
            Some(member2) => {
                let at2 = json::nested(at, "description");
                Some(json::text_at(member2, &at2, "a string")?.to_owned())
            }
        },
        parts: {
            let at3 = json::nested(at, "parts");
            let member3 = json::member_at(value, at, "parts")?;
            {
                let mut items3 = Vec::new();
                for (index3, element3) in json::items_at(member3, &at3, "an array")?.iter().enumerate() {
                    let nested3 = json::nested(&at3, &index3.to_string());
                    items3.push(decode_a2a_tasks_part(element3, &nested3)?);
                }
                items3
            }
        },
        metadata: match value.member("metadata") {
            None | Some(json::Value::Null) => None,
            Some(member4) => {
                let at4 = json::nested(at, "metadata");
                Some(json::value_at(member4, &at4))
            }
        },
        extensions: match value.member("extensions") {
            None | Some(json::Value::Null) => None,
            Some(member5) => {
                let at5 = json::nested(at, "extensions");
                Some({
                let mut items5 = Vec::new();
                for (index5, element5) in json::items_at(member5, &at5, "an array")?.iter().enumerate() {
                    let nested5 = json::nested(&at5, &index5.to_string());
                    items5.push(json::text_at(element5, &nested5, "a string")?.to_owned());
                }
                items5
            })
            }
        },
    })
}

/// Writes `a2a.tasks.AuthenticationInfo` as JSON.
pub fn encode_a2a_tasks_authentication_info(value: &crate::tasks::AuthenticationInfo, out: &mut String) {
    out.push('{');
    json::member(out, "scheme");
    json::push_text(out, &value.scheme);
    if let Some(held0) = &value.credentials {
        json::member(out, "credentials");
        json::push_text(out, &*held0);
    }
    out.push('}');
}

/// Reads `a2a.tasks.AuthenticationInfo` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_tasks_authentication_info(value: &json::Value, at: &str) -> Result<crate::tasks::AuthenticationInfo, json::DecodeError> {
    Ok(crate::tasks::AuthenticationInfo {
        scheme: {
            let at0 = json::nested(at, "scheme");
            let member0 = json::member_at(value, at, "scheme")?;
            json::text_at(member0, &at0, "a string")?.to_owned()
        },
        credentials: match value.member("credentials") {
            None | Some(json::Value::Null) => None,
            Some(member1) => {
                let at1 = json::nested(at, "credentials");
                Some(json::text_at(member1, &at1, "a string")?.to_owned())
            }
        },
    })
}

/// Writes `a2a.tasks.ContextId` as JSON.
pub fn encode_a2a_tasks_context_id(value: &crate::tasks::ContextId, out: &mut String) {
    json::push_text(out, &value.0);
}

/// Reads `a2a.tasks.ContextId` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_tasks_context_id(value: &json::Value, at: &str) -> Result<crate::tasks::ContextId, json::DecodeError> {
    Ok(crate::tasks::ContextId(json::text_at(value, at, "a string")?.to_owned()))
}

/// Writes `a2a.tasks.Message` as JSON.
pub fn encode_a2a_tasks_message(value: &crate::tasks::Message, out: &mut String) {
    out.push('{');
    json::member(out, "messageId");
    json::push_text(out, &value.message_id);
    if let Some(held0) = &value.context_id {
        json::member(out, "contextId");
        encode_a2a_tasks_context_id(&*held0, out);
    }
    if let Some(held0) = &value.task_id {
        json::member(out, "taskId");
        encode_a2a_tasks_task_id(&*held0, out);
    }
    json::member(out, "role");
    encode_a2a_tasks_role(&value.role, out);
    json::member(out, "parts");
    out.push('[');
    for (index0, item0) in value.parts.iter().enumerate() {
        if index0 > 0 {
            out.push(',');
        }
        encode_a2a_tasks_part(&*item0, out);
    }
    out.push(']');
    if let Some(held0) = &value.metadata {
        json::member(out, "metadata");
        json::push_value(out, &*held0);
    }
    if let Some(held0) = &value.extensions {
        json::member(out, "extensions");
        out.push('[');
        for (index1, item1) in (*held0).iter().enumerate() {
            if index1 > 0 {
                out.push(',');
            }
            json::push_text(out, &*item1);
        }
        out.push(']');
    }
    if let Some(held0) = &value.reference_task_ids {
        json::member(out, "referenceTaskIds");
        out.push('[');
        for (index1, item1) in (*held0).iter().enumerate() {
            if index1 > 0 {
                out.push(',');
            }
            encode_a2a_tasks_task_id(&*item1, out);
        }
        out.push(']');
    }
    out.push('}');
}

/// Reads `a2a.tasks.Message` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_tasks_message(value: &json::Value, at: &str) -> Result<crate::tasks::Message, json::DecodeError> {
    Ok(crate::tasks::Message {
        message_id: {
            let at0 = json::nested(at, "messageId");
            let member0 = json::member_at(value, at, "messageId")?;
            json::text_at(member0, &at0, "a string")?.to_owned()
        },
        context_id: match value.member("contextId") {
            None | Some(json::Value::Null) => None,
            Some(member1) => {
                let at1 = json::nested(at, "contextId");
                Some(decode_a2a_tasks_context_id(member1, &at1)?)
            }
        },
        task_id: match value.member("taskId") {
            None | Some(json::Value::Null) => None,
            Some(member2) => {
                let at2 = json::nested(at, "taskId");
                Some(decode_a2a_tasks_task_id(member2, &at2)?)
            }
        },
        role: {
            let at3 = json::nested(at, "role");
            let member3 = json::member_at(value, at, "role")?;
            decode_a2a_tasks_role(member3, &at3)?
        },
        parts: {
            let at4 = json::nested(at, "parts");
            let member4 = json::member_at(value, at, "parts")?;
            {
                let mut items4 = Vec::new();
                for (index4, element4) in json::items_at(member4, &at4, "an array")?.iter().enumerate() {
                    let nested4 = json::nested(&at4, &index4.to_string());
                    items4.push(decode_a2a_tasks_part(element4, &nested4)?);
                }
                items4
            }
        },
        metadata: match value.member("metadata") {
            None | Some(json::Value::Null) => None,
            Some(member5) => {
                let at5 = json::nested(at, "metadata");
                Some(json::value_at(member5, &at5))
            }
        },
        extensions: match value.member("extensions") {
            None | Some(json::Value::Null) => None,
            Some(member6) => {
                let at6 = json::nested(at, "extensions");
                Some({
                let mut items6 = Vec::new();
                for (index6, element6) in json::items_at(member6, &at6, "an array")?.iter().enumerate() {
                    let nested6 = json::nested(&at6, &index6.to_string());
                    items6.push(json::text_at(element6, &nested6, "a string")?.to_owned());
                }
                items6
            })
            }
        },
        reference_task_ids: match value.member("referenceTaskIds") {
            None | Some(json::Value::Null) => None,
            Some(member7) => {
                let at7 = json::nested(at, "referenceTaskIds");
                Some({
                let mut items7 = Vec::new();
                for (index7, element7) in json::items_at(member7, &at7, "an array")?.iter().enumerate() {
                    let nested7 = json::nested(&at7, &index7.to_string());
                    items7.push(decode_a2a_tasks_task_id(element7, &nested7)?);
                }
                items7
            })
            }
        },
    })
}

/// Writes `a2a.tasks.Part` as JSON.
pub fn encode_a2a_tasks_part(value: &crate::tasks::Part, out: &mut String) {
    out.push('{');
    if let Some(held0) = &value.text {
        json::member(out, "text");
        json::push_text(out, &*held0);
    }
    if let Some(held0) = &value.raw {
        json::member(out, "raw");
        json::push_base64(out, &*held0);
    }
    if let Some(held0) = &value.url {
        json::member(out, "url");
        json::push_text(out, &*held0);
    }
    if let Some(held0) = &value.data {
        json::member(out, "data");
        json::push_value(out, &*held0);
    }
    if let Some(held0) = &value.metadata {
        json::member(out, "metadata");
        json::push_value(out, &*held0);
    }
    if let Some(held0) = &value.filename {
        json::member(out, "filename");
        json::push_text(out, &*held0);
    }
    if let Some(held0) = &value.media_type {
        json::member(out, "mediaType");
        json::push_text(out, &*held0);
    }
    out.push('}');
}

/// Reads `a2a.tasks.Part` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_tasks_part(value: &json::Value, at: &str) -> Result<crate::tasks::Part, json::DecodeError> {
    Ok(crate::tasks::Part {
        text: match value.member("text") {
            None | Some(json::Value::Null) => None,
            Some(member0) => {
                let at0 = json::nested(at, "text");
                Some(json::text_at(member0, &at0, "a string")?.to_owned())
            }
        },
        raw: match value.member("raw") {
            None | Some(json::Value::Null) => None,
            Some(member1) => {
                let at1 = json::nested(at, "raw");
                Some(json::bytes_at(member1, &at1, "base64-encoded bytes")?)
            }
        },
        url: match value.member("url") {
            None | Some(json::Value::Null) => None,
            Some(member2) => {
                let at2 = json::nested(at, "url");
                Some(json::text_at(member2, &at2, "a string")?.to_owned())
            }
        },
        data: match value.member("data") {
            None | Some(json::Value::Null) => None,
            Some(member3) => {
                let at3 = json::nested(at, "data");
                Some(json::value_at(member3, &at3))
            }
        },
        metadata: match value.member("metadata") {
            None | Some(json::Value::Null) => None,
            Some(member4) => {
                let at4 = json::nested(at, "metadata");
                Some(json::value_at(member4, &at4))
            }
        },
        filename: match value.member("filename") {
            None | Some(json::Value::Null) => None,
            Some(member5) => {
                let at5 = json::nested(at, "filename");
                Some(json::text_at(member5, &at5, "a string")?.to_owned())
            }
        },
        media_type: match value.member("mediaType") {
            None | Some(json::Value::Null) => None,
            Some(member6) => {
                let at6 = json::nested(at, "mediaType");
                Some(json::text_at(member6, &at6, "a string")?.to_owned())
            }
        },
    })
}

/// Writes `a2a.tasks.PushNotificationConfig.State` as JSON.
pub fn encode_a2a_tasks_push_notification_config_state(value: &crate::tasks::PushNotificationConfigState, out: &mut String) {
    match value {
        crate::tasks::PushNotificationConfigState::Active => json::push_text(out, "Active"),
    }
}

/// Reads `a2a.tasks.PushNotificationConfig.State` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_tasks_push_notification_config_state(value: &json::Value, at: &str) -> Result<crate::tasks::PushNotificationConfigState, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `Active`")? {
        "Active" => crate::tasks::PushNotificationConfigState::Active,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `Active`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `a2a.tasks.PushNotificationConfigId` as JSON.
pub fn encode_a2a_tasks_push_notification_config_id(value: &crate::tasks::PushNotificationConfigId, out: &mut String) {
    json::push_text(out, &value.0);
}

/// Reads `a2a.tasks.PushNotificationConfigId` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_tasks_push_notification_config_id(value: &json::Value, at: &str) -> Result<crate::tasks::PushNotificationConfigId, json::DecodeError> {
    Ok(crate::tasks::PushNotificationConfigId(json::text_at(value, at, "a string")?.to_owned()))
}

/// Writes `a2a.tasks.PushNotificationConfigRow` as JSON.
pub fn encode_a2a_tasks_push_notification_config_row(value: &crate::tasks::PushNotificationConfigRow, out: &mut String) {
    json::push_text(out, &value.0);
}

/// Reads `a2a.tasks.PushNotificationConfigRow` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_tasks_push_notification_config_row(value: &json::Value, at: &str) -> Result<crate::tasks::PushNotificationConfigRow, json::DecodeError> {
    Ok(crate::tasks::PushNotificationConfigRow(json::text_at(value, at, "a string")?.to_owned()))
}

/// Writes `a2a.tasks.Role` as JSON.
pub fn encode_a2a_tasks_role(value: &crate::tasks::Role, out: &mut String) {
    match value {
        crate::tasks::Role::ROLEUSER => json::push_text(out, "ROLE_USER"),
        crate::tasks::Role::ROLEAGENT => json::push_text(out, "ROLE_AGENT"),
    }
}

/// Reads `a2a.tasks.Role` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_tasks_role(value: &json::Value, at: &str) -> Result<crate::tasks::Role, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `ROLE_USER`, `ROLE_AGENT`")? {
        "ROLE_USER" => crate::tasks::Role::ROLEUSER,
        "ROLE_AGENT" => crate::tasks::Role::ROLEAGENT,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `ROLE_USER`, `ROLE_AGENT`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `a2a.tasks.SendMessageConfiguration` as JSON.
pub fn encode_a2a_tasks_send_message_configuration(value: &crate::tasks::SendMessageConfiguration, out: &mut String) {
    out.push('{');
    if let Some(held0) = &value.accepted_output_modes {
        json::member(out, "acceptedOutputModes");
        out.push('[');
        for (index1, item1) in (*held0).iter().enumerate() {
            if index1 > 0 {
                out.push(',');
            }
            json::push_text(out, &*item1);
        }
        out.push(']');
    }
    if let Some(held0) = &value.task_push_notification_config {
        json::member(out, "taskPushNotificationConfig");
        encode_a2a_tasks_task_push_notification_config(&*held0, out);
    }
    if let Some(held0) = &value.history_length {
        json::member(out, "historyLength");
        json::push_integer(out, *held0);
    }
    if let Some(held0) = &value.return_immediately {
        json::member(out, "returnImmediately");
        json::push_bool(out, *held0);
    }
    out.push('}');
}

/// Reads `a2a.tasks.SendMessageConfiguration` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_tasks_send_message_configuration(value: &json::Value, at: &str) -> Result<crate::tasks::SendMessageConfiguration, json::DecodeError> {
    Ok(crate::tasks::SendMessageConfiguration {
        accepted_output_modes: match value.member("acceptedOutputModes") {
            None | Some(json::Value::Null) => None,
            Some(member0) => {
                let at0 = json::nested(at, "acceptedOutputModes");
                Some({
                let mut items0 = Vec::new();
                for (index0, element0) in json::items_at(member0, &at0, "an array")?.iter().enumerate() {
                    let nested0 = json::nested(&at0, &index0.to_string());
                    items0.push(json::text_at(element0, &nested0, "a string")?.to_owned());
                }
                items0
            })
            }
        },
        task_push_notification_config: match value.member("taskPushNotificationConfig") {
            None | Some(json::Value::Null) => None,
            Some(member1) => {
                let at1 = json::nested(at, "taskPushNotificationConfig");
                Some(decode_a2a_tasks_task_push_notification_config(member1, &at1)?)
            }
        },
        history_length: match value.member("historyLength") {
            None | Some(json::Value::Null) => None,
            Some(member2) => {
                let at2 = json::nested(at, "historyLength");
                Some(json::integer_at(member2, &at2, "an integer")?)
            }
        },
        return_immediately: match value.member("returnImmediately") {
            None | Some(json::Value::Null) => None,
            Some(member3) => {
                let at3 = json::nested(at, "returnImmediately");
                Some(json::bool_at(member3, &at3, "a boolean")?)
            }
        },
    })
}

/// Writes `a2a.tasks.Task.State` as JSON.
pub fn encode_a2a_tasks_task_state(value: &crate::tasks::TaskState, out: &mut String) {
    match value {
        crate::tasks::TaskState::AuthRequired => json::push_text(out, "AuthRequired"),
        crate::tasks::TaskState::Canceled => json::push_text(out, "Canceled"),
        crate::tasks::TaskState::Completed => json::push_text(out, "Completed"),
        crate::tasks::TaskState::Failed => json::push_text(out, "Failed"),
        crate::tasks::TaskState::InputRequired => json::push_text(out, "InputRequired"),
        crate::tasks::TaskState::Rejected => json::push_text(out, "Rejected"),
        crate::tasks::TaskState::Submitted => json::push_text(out, "Submitted"),
        crate::tasks::TaskState::Working => json::push_text(out, "Working"),
    }
}

/// Reads `a2a.tasks.Task.State` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_tasks_task_state(value: &json::Value, at: &str) -> Result<crate::tasks::TaskState, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `AuthRequired`, `Canceled`, `Completed`, `Failed`, `InputRequired`, `Rejected`, `Submitted`, `Working`")? {
        "AuthRequired" => crate::tasks::TaskState::AuthRequired,
        "Canceled" => crate::tasks::TaskState::Canceled,
        "Completed" => crate::tasks::TaskState::Completed,
        "Failed" => crate::tasks::TaskState::Failed,
        "InputRequired" => crate::tasks::TaskState::InputRequired,
        "Rejected" => crate::tasks::TaskState::Rejected,
        "Submitted" => crate::tasks::TaskState::Submitted,
        "Working" => crate::tasks::TaskState::Working,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `AuthRequired`, `Canceled`, `Completed`, `Failed`, `InputRequired`, `Rejected`, `Submitted`, `Working`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `a2a.tasks.TaskId` as JSON.
pub fn encode_a2a_tasks_task_id(value: &crate::tasks::TaskId, out: &mut String) {
    json::push_text(out, &value.0);
}

/// Reads `a2a.tasks.TaskId` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_tasks_task_id(value: &json::Value, at: &str) -> Result<crate::tasks::TaskId, json::DecodeError> {
    Ok(crate::tasks::TaskId(json::text_at(value, at, "a string")?.to_owned()))
}

/// Writes `a2a.tasks.TaskPushNotificationConfig` as JSON.
pub fn encode_a2a_tasks_task_push_notification_config(value: &crate::tasks::TaskPushNotificationConfig, out: &mut String) {
    out.push('{');
    if let Some(held0) = &value.tenant {
        json::member(out, "tenant");
        json::push_text(out, &*held0);
    }
    if let Some(held0) = &value.id {
        json::member(out, "id");
        encode_a2a_tasks_push_notification_config_id(&*held0, out);
    }
    if let Some(held0) = &value.task_id {
        json::member(out, "taskId");
        encode_a2a_tasks_task_id(&*held0, out);
    }
    json::member(out, "url");
    json::push_text(out, &value.url);
    if let Some(held0) = &value.token {
        json::member(out, "token");
        json::push_text(out, &*held0);
    }
    if let Some(held0) = &value.authentication {
        json::member(out, "authentication");
        encode_a2a_tasks_authentication_info(&*held0, out);
    }
    out.push('}');
}

/// Reads `a2a.tasks.TaskPushNotificationConfig` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_tasks_task_push_notification_config(value: &json::Value, at: &str) -> Result<crate::tasks::TaskPushNotificationConfig, json::DecodeError> {
    Ok(crate::tasks::TaskPushNotificationConfig {
        tenant: match value.member("tenant") {
            None | Some(json::Value::Null) => None,
            Some(member0) => {
                let at0 = json::nested(at, "tenant");
                Some(json::text_at(member0, &at0, "a string")?.to_owned())
            }
        },
        id: match value.member("id") {
            None | Some(json::Value::Null) => None,
            Some(member1) => {
                let at1 = json::nested(at, "id");
                Some(decode_a2a_tasks_push_notification_config_id(member1, &at1)?)
            }
        },
        task_id: match value.member("taskId") {
            None | Some(json::Value::Null) => None,
            Some(member2) => {
                let at2 = json::nested(at, "taskId");
                Some(decode_a2a_tasks_task_id(member2, &at2)?)
            }
        },
        url: {
            let at3 = json::nested(at, "url");
            let member3 = json::member_at(value, at, "url")?;
            json::text_at(member3, &at3, "a string")?.to_owned()
        },
        token: match value.member("token") {
            None | Some(json::Value::Null) => None,
            Some(member4) => {
                let at4 = json::nested(at, "token");
                Some(json::text_at(member4, &at4, "a string")?.to_owned())
            }
        },
        authentication: match value.member("authentication") {
            None | Some(json::Value::Null) => None,
            Some(member5) => {
                let at5 = json::nested(at, "authentication");
                Some(decode_a2a_tasks_authentication_info(member5, &at5)?)
            }
        },
    })
}

/// Writes `a2a.tasks.TaskSnapshot` as JSON.
pub fn encode_a2a_tasks_task_snapshot(value: &crate::tasks::TaskSnapshot, out: &mut String) {
    out.push('{');
    json::member(out, "id");
    encode_a2a_tasks_task_id(&value.id, out);
    json::member(out, "contextId");
    encode_a2a_tasks_context_id(&value.context_id, out);
    json::member(out, "status");
    encode_a2a_tasks_task_status(&value.status, out);
    if let Some(held0) = &value.artifacts {
        json::member(out, "artifacts");
        out.push('[');
        for (index1, item1) in (*held0).iter().enumerate() {
            if index1 > 0 {
                out.push(',');
            }
            encode_a2a_tasks_artifact(&*item1, out);
        }
        out.push(']');
    }
    if let Some(held0) = &value.history {
        json::member(out, "history");
        out.push('[');
        for (index1, item1) in (*held0).iter().enumerate() {
            if index1 > 0 {
                out.push(',');
            }
            encode_a2a_tasks_message(&*item1, out);
        }
        out.push(']');
    }
    if let Some(held0) = &value.metadata {
        json::member(out, "metadata");
        json::push_value(out, &*held0);
    }
    out.push('}');
}

/// Reads `a2a.tasks.TaskSnapshot` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_tasks_task_snapshot(value: &json::Value, at: &str) -> Result<crate::tasks::TaskSnapshot, json::DecodeError> {
    Ok(crate::tasks::TaskSnapshot {
        id: {
            let at0 = json::nested(at, "id");
            let member0 = json::member_at(value, at, "id")?;
            decode_a2a_tasks_task_id(member0, &at0)?
        },
        context_id: {
            let at1 = json::nested(at, "contextId");
            let member1 = json::member_at(value, at, "contextId")?;
            decode_a2a_tasks_context_id(member1, &at1)?
        },
        status: {
            let at2 = json::nested(at, "status");
            let member2 = json::member_at(value, at, "status")?;
            decode_a2a_tasks_task_status(member2, &at2)?
        },
        artifacts: match value.member("artifacts") {
            None | Some(json::Value::Null) => None,
            Some(member3) => {
                let at3 = json::nested(at, "artifacts");
                Some({
                let mut items3 = Vec::new();
                for (index3, element3) in json::items_at(member3, &at3, "an array")?.iter().enumerate() {
                    let nested3 = json::nested(&at3, &index3.to_string());
                    items3.push(decode_a2a_tasks_artifact(element3, &nested3)?);
                }
                items3
            })
            }
        },
        history: match value.member("history") {
            None | Some(json::Value::Null) => None,
            Some(member4) => {
                let at4 = json::nested(at, "history");
                Some({
                let mut items4 = Vec::new();
                for (index4, element4) in json::items_at(member4, &at4, "an array")?.iter().enumerate() {
                    let nested4 = json::nested(&at4, &index4.to_string());
                    items4.push(decode_a2a_tasks_message(element4, &nested4)?);
                }
                items4
            })
            }
        },
        metadata: match value.member("metadata") {
            None | Some(json::Value::Null) => None,
            Some(member5) => {
                let at5 = json::nested(at, "metadata");
                Some(json::value_at(member5, &at5))
            }
        },
    })
}

/// Writes `a2a.tasks.TaskState` as JSON.
pub fn encode_a2a_tasks_task_state_2(value: &crate::tasks::WireTaskState, out: &mut String) {
    match value {
        crate::tasks::WireTaskState::TASKSTATESUBMITTED => json::push_text(out, "TASK_STATE_SUBMITTED"),
        crate::tasks::WireTaskState::TASKSTATEWORKING => json::push_text(out, "TASK_STATE_WORKING"),
        crate::tasks::WireTaskState::TASKSTATECOMPLETED => json::push_text(out, "TASK_STATE_COMPLETED"),
        crate::tasks::WireTaskState::TASKSTATEFAILED => json::push_text(out, "TASK_STATE_FAILED"),
        crate::tasks::WireTaskState::TASKSTATECANCELED => json::push_text(out, "TASK_STATE_CANCELED"),
        crate::tasks::WireTaskState::TASKSTATEINPUTREQUIRED => json::push_text(out, "TASK_STATE_INPUT_REQUIRED"),
        crate::tasks::WireTaskState::TASKSTATEREJECTED => json::push_text(out, "TASK_STATE_REJECTED"),
        crate::tasks::WireTaskState::TASKSTATEAUTHREQUIRED => json::push_text(out, "TASK_STATE_AUTH_REQUIRED"),
    }
}

/// Reads `a2a.tasks.TaskState` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_tasks_task_state_2(value: &json::Value, at: &str) -> Result<crate::tasks::WireTaskState, json::DecodeError> {
    Ok(match json::text_at(value, at, "one of `TASK_STATE_SUBMITTED`, `TASK_STATE_WORKING`, `TASK_STATE_COMPLETED`, `TASK_STATE_FAILED`, `TASK_STATE_CANCELED`, `TASK_STATE_INPUT_REQUIRED`, `TASK_STATE_REJECTED`, `TASK_STATE_AUTH_REQUIRED`")? {
        "TASK_STATE_SUBMITTED" => crate::tasks::WireTaskState::TASKSTATESUBMITTED,
        "TASK_STATE_WORKING" => crate::tasks::WireTaskState::TASKSTATEWORKING,
        "TASK_STATE_COMPLETED" => crate::tasks::WireTaskState::TASKSTATECOMPLETED,
        "TASK_STATE_FAILED" => crate::tasks::WireTaskState::TASKSTATEFAILED,
        "TASK_STATE_CANCELED" => crate::tasks::WireTaskState::TASKSTATECANCELED,
        "TASK_STATE_INPUT_REQUIRED" => crate::tasks::WireTaskState::TASKSTATEINPUTREQUIRED,
        "TASK_STATE_REJECTED" => crate::tasks::WireTaskState::TASKSTATEREJECTED,
        "TASK_STATE_AUTH_REQUIRED" => crate::tasks::WireTaskState::TASKSTATEAUTHREQUIRED,
        other => return Err(json::DecodeError { at: at.to_owned(), expected: "one of `TASK_STATE_SUBMITTED`, `TASK_STATE_WORKING`, `TASK_STATE_COMPLETED`, `TASK_STATE_FAILED`, `TASK_STATE_CANCELED`, `TASK_STATE_INPUT_REQUIRED`, `TASK_STATE_REJECTED`, `TASK_STATE_AUTH_REQUIRED`".to_owned(), found: format!("`{other}`") }),
    })
}

/// Writes `a2a.tasks.TaskStatus` as JSON.
pub fn encode_a2a_tasks_task_status(value: &crate::tasks::TaskStatus, out: &mut String) {
    out.push('{');
    json::member(out, "state");
    encode_a2a_tasks_task_state_2(&value.state, out);
    if let Some(held0) = &value.message {
        json::member(out, "message");
        encode_a2a_tasks_message(&*held0, out);
    }
    if let Some(held0) = &value.timestamp {
        json::member(out, "timestamp");
        json::push_text(out, &(*held0).0);
    }
    out.push('}');
}

/// Reads `a2a.tasks.TaskStatus` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_a2a_tasks_task_status(value: &json::Value, at: &str) -> Result<crate::tasks::TaskStatus, json::DecodeError> {
    Ok(crate::tasks::TaskStatus {
        state: {
            let at0 = json::nested(at, "state");
            let member0 = json::member_at(value, at, "state")?;
            decode_a2a_tasks_task_state_2(member0, &at0)?
        },
        message: match value.member("message") {
            None | Some(json::Value::Null) => None,
            Some(member1) => {
                let at1 = json::nested(at, "message");
                Some(decode_a2a_tasks_message(member1, &at1)?)
            }
        },
        timestamp: match value.member("timestamp") {
            None | Some(json::Value::Null) => None,
            Some(member2) => {
                let at2 = json::nested(at, "timestamp");
                Some(crate::primitives::Timestamp(json::text_at(member2, &at2, "an RFC 3339 instant")?.to_owned()))
            }
        },
    })
}

/// Writes the event `a2a.tasks.PushNotificationConfigCreated` as JSON.
pub fn encode_event_a2a_tasks_push_notification_config_created(value: &crate::tasks::PushNotificationConfigCreated, out: &mut String) {
    out.push('{');
    json::member(out, "row");
    encode_a2a_tasks_push_notification_config_row(&value.row, out);
    json::member(out, "config_id");
    encode_a2a_tasks_push_notification_config_id(&value.config_id, out);
    json::member(out, "task_id");
    encode_a2a_tasks_task_id(&value.task_id, out);
    out.push('}');
}

/// Writes the event `a2a.tasks.PushNotificationConfigDeleted` as JSON.
pub fn encode_event_a2a_tasks_push_notification_config_deleted(value: &crate::tasks::PushNotificationConfigDeleted, out: &mut String) {
    out.push('{');
    json::member(out, "config_id");
    encode_a2a_tasks_push_notification_config_id(&value.config_id, out);
    json::member(out, "task_id");
    encode_a2a_tasks_task_id(&value.task_id, out);
    out.push('}');
}

/// Writes the event `a2a.tasks.TaskArtifactUpdated` as JSON.
pub fn encode_event_a2a_tasks_task_artifact_updated(value: &crate::tasks::TaskArtifactUpdated, out: &mut String) {
    out.push('{');
    json::member(out, "taskId");
    encode_a2a_tasks_task_id(&value.task_id, out);
    json::member(out, "contextId");
    encode_a2a_tasks_context_id(&value.context_id, out);
    json::member(out, "artifact");
    encode_a2a_tasks_artifact(&value.artifact, out);
    if let Some(held0) = &value.append {
        json::member(out, "append");
        json::push_bool(out, *held0);
    }
    if let Some(held0) = &value.last_chunk {
        json::member(out, "lastChunk");
        json::push_bool(out, *held0);
    }
    out.push('}');
}

/// Writes the event `a2a.tasks.TaskCreated` as JSON.
pub fn encode_event_a2a_tasks_task_created(value: &crate::tasks::TaskCreated, out: &mut String) {
    out.push('{');
    json::member(out, "task_id");
    encode_a2a_tasks_task_id(&value.task_id, out);
    out.push('}');
}

/// Writes the event `a2a.tasks.TaskStatusUpdated` as JSON.
pub fn encode_event_a2a_tasks_task_status_updated(value: &crate::tasks::TaskStatusUpdated, out: &mut String) {
    out.push('{');
    json::member(out, "task_id");
    encode_a2a_tasks_task_id(&value.task_id, out);
    json::member(out, "context_id");
    encode_a2a_tasks_context_id(&value.context_id, out);
    json::member(out, "status");
    encode_a2a_tasks_task_status(&value.status, out);
    out.push('}');
}

/// Writes the declared error `a2a.discovery.ExtendedAgentCardNotConfigured` as JSON.
pub fn encode_error_a2a_discovery_extended_agent_card_not_configured(_value: &crate::discovery::ExtendedAgentCardNotConfigured, out: &mut String) {
    out.push('{');
    out.push('}');
}

/// Writes the declared error `a2a.discovery.ExtensionSupportRequired` as JSON.
pub fn encode_error_a2a_discovery_extension_support_required(_value: &crate::discovery::ExtensionSupportRequired, out: &mut String) {
    out.push('{');
    out.push('}');
}

/// Writes the declared error `a2a.discovery.UnsupportedOperation` as JSON.
pub fn encode_error_a2a_discovery_unsupported_operation(_value: &crate::discovery::UnsupportedOperation, out: &mut String) {
    out.push('{');
    out.push('}');
}

/// Writes the declared error `a2a.discovery.VersionNotSupported` as JSON.
pub fn encode_error_a2a_discovery_version_not_supported(_value: &crate::discovery::VersionNotSupported, out: &mut String) {
    out.push('{');
    out.push('}');
}

/// Writes the declared error `a2a.tasks.ContentTypeNotSupported` as JSON.
pub fn encode_error_a2a_tasks_content_type_not_supported(_value: &crate::tasks::ContentTypeNotSupported, out: &mut String) {
    out.push('{');
    out.push('}');
}

/// Writes the declared error `a2a.tasks.ContextMismatch` as JSON.
pub fn encode_error_a2a_tasks_context_mismatch(_value: &crate::tasks::ContextMismatch, out: &mut String) {
    out.push('{');
    out.push('}');
}

/// Writes the declared error `a2a.tasks.ContextNotAccepted` as JSON.
pub fn encode_error_a2a_tasks_context_not_accepted(_value: &crate::tasks::ContextNotAccepted, out: &mut String) {
    out.push('{');
    out.push('}');
}

/// Writes the declared error `a2a.tasks.ExtensionSupportRequired` as JSON.
pub fn encode_error_a2a_tasks_extension_support_required(_value: &crate::tasks::ExtensionSupportRequired, out: &mut String) {
    out.push('{');
    out.push('}');
}

/// Writes the declared error `a2a.tasks.PushNotificationNotSupported` as JSON.
pub fn encode_error_a2a_tasks_push_notification_not_supported(_value: &crate::tasks::PushNotificationNotSupported, out: &mut String) {
    out.push('{');
    out.push('}');
}

/// Writes the declared error `a2a.tasks.TaskNotCancelable` as JSON.
pub fn encode_error_a2a_tasks_task_not_cancelable(_value: &crate::tasks::TaskNotCancelable, out: &mut String) {
    out.push('{');
    out.push('}');
}

/// Writes the declared error `a2a.tasks.TaskNotFound` as JSON.
pub fn encode_error_a2a_tasks_task_not_found(_value: &crate::tasks::TaskNotFound, out: &mut String) {
    out.push('{');
    out.push('}');
}

/// Writes the declared error `a2a.tasks.TaskStateConflict` as JSON.
pub fn encode_error_a2a_tasks_task_state_conflict(value: &crate::tasks::TaskStateConflict, out: &mut String) {
    out.push('{');
    json::member(out, "state");
    encode_a2a_tasks_task_state(&value.state, out);
    out.push('}');
}

/// Writes the declared error `a2a.tasks.UnsupportedOperation` as JSON.
pub fn encode_error_a2a_tasks_unsupported_operation(_value: &crate::tasks::UnsupportedOperation, out: &mut String) {
    out.push('{');
    out.push('}');
}

/// Writes the declared error `a2a.tasks.VersionNotSupported` as JSON.
pub fn encode_error_a2a_tasks_version_not_supported(_value: &crate::tasks::VersionNotSupported, out: &mut String) {
    out.push('{');
    out.push('}');
}

/// Writes one row of the view `a2a.tasks.AllTasks` as JSON.
pub fn encode_view_a2a_tasks_all_tasks(value: &crate::tasks::AllTasks, out: &mut String) {
    out.push('{');
    json::member(out, "id");
    encode_a2a_tasks_task_id(&value.id, out);
    json::member(out, "context_id");
    encode_a2a_tasks_context_id(&value.context_id, out);
    json::member(out, "state");
    encode_a2a_tasks_task_state(&value.state, out);
    if let Some(held0) = &value.status_message {
        json::member(out, "status_message");
        encode_a2a_tasks_message(&*held0, out);
    }
    if let Some(held0) = &value.status_timestamp {
        json::member(out, "status_timestamp");
        json::push_text(out, &(*held0).0);
    }
    if let Some(held0) = &value.artifacts {
        json::member(out, "artifacts");
        out.push('[');
        for (index1, item1) in (*held0).iter().enumerate() {
            if index1 > 0 {
                out.push(',');
            }
            encode_a2a_tasks_artifact(&*item1, out);
        }
        out.push(']');
    }
    if let Some(held0) = &value.history {
        json::member(out, "history");
        out.push('[');
        for (index1, item1) in (*held0).iter().enumerate() {
            if index1 > 0 {
                out.push(',');
            }
            encode_a2a_tasks_message(&*item1, out);
        }
        out.push(']');
    }
    if let Some(held0) = &value.metadata {
        json::member(out, "metadata");
        json::push_value(out, &*held0);
    }
    out.push('}');
}

/// Writes one row of the view `a2a.tasks.PushNotificationConfigs` as JSON.
pub fn encode_view_a2a_tasks_push_notification_configs(value: &crate::tasks::PushNotificationConfigs, out: &mut String) {
    out.push('{');
    json::member(out, "row");
    encode_a2a_tasks_push_notification_config_row(&value.row, out);
    json::member(out, "config_id");
    encode_a2a_tasks_push_notification_config_id(&value.config_id, out);
    json::member(out, "state");
    encode_a2a_tasks_push_notification_config_state(&value.state, out);
    json::member(out, "task_id");
    encode_a2a_tasks_task_id(&value.task_id, out);
    if let Some(held0) = &value.tenant {
        json::member(out, "tenant");
        json::push_text(out, &*held0);
    }
    json::member(out, "url");
    json::push_text(out, &value.url);
    if let Some(held0) = &value.token {
        json::member(out, "token");
        json::push_text(out, &*held0);
    }
    if let Some(held0) = &value.authentication {
        json::member(out, "authentication");
        encode_a2a_tasks_authentication_info(&*held0, out);
    }
    out.push('}');
}

/// Writes one row of the view `a2a.tasks.TaskPushNotificationConfigs` as JSON.
pub fn encode_view_a2a_tasks_task_push_notification_configs(value: &crate::tasks::TaskPushNotificationConfigs, out: &mut String) {
    out.push('{');
    json::member(out, "config_id");
    encode_a2a_tasks_push_notification_config_id(&value.config_id, out);
    json::member(out, "task_id");
    encode_a2a_tasks_task_id(&value.task_id, out);
    if let Some(held0) = &value.tenant {
        json::member(out, "tenant");
        json::push_text(out, &*held0);
    }
    json::member(out, "url");
    json::push_text(out, &value.url);
    if let Some(held0) = &value.token {
        json::member(out, "token");
        json::push_text(out, &*held0);
    }
    if let Some(held0) = &value.authentication {
        json::member(out, "authentication");
        encode_a2a_tasks_authentication_info(&*held0, out);
    }
    out.push('}');
}

/// Reads the parameters of `a2a.tasks.TaskPushNotificationConfigs` from an object keyed by their wire names.
///
/// # Errors
///
/// [`json::DecodeError`] naming the parameter and what the declaration says belongs there.
pub fn decode_params_a2a_tasks_task_push_notification_configs(value: &json::Value, at: &str) -> Result<(crate::tasks::TaskId,), json::DecodeError> {
    Ok((
        {
            let at0 = json::nested(at, "task_id");
            let member0 = json::member_at(value, at, "task_id")?;
            decode_a2a_tasks_task_id(member0, &at0)?
        },
    ))
}

/// Writes one row of the view `a2a.tasks.Tasks` as JSON.
pub fn encode_view_a2a_tasks_tasks(value: &crate::tasks::Tasks, out: &mut String) {
    out.push('{');
    json::member(out, "id");
    encode_a2a_tasks_task_id(&value.id, out);
    json::member(out, "context_id");
    encode_a2a_tasks_context_id(&value.context_id, out);
    json::member(out, "state");
    encode_a2a_tasks_task_state(&value.state, out);
    if let Some(held0) = &value.status_message {
        json::member(out, "status_message");
        encode_a2a_tasks_message(&*held0, out);
    }
    if let Some(held0) = &value.status_timestamp {
        json::member(out, "status_timestamp");
        json::push_text(out, &(*held0).0);
    }
    if let Some(held0) = &value.artifacts {
        json::member(out, "artifacts");
        out.push('[');
        for (index1, item1) in (*held0).iter().enumerate() {
            if index1 > 0 {
                out.push(',');
            }
            encode_a2a_tasks_artifact(&*item1, out);
        }
        out.push(']');
    }
    if let Some(held0) = &value.history {
        json::member(out, "history");
        out.push('[');
        for (index1, item1) in (*held0).iter().enumerate() {
            if index1 > 0 {
                out.push(',');
            }
            encode_a2a_tasks_message(&*item1, out);
        }
        out.push(']');
    }
    if let Some(held0) = &value.metadata {
        json::member(out, "metadata");
        json::push_value(out, &*held0);
    }
    out.push('}');
}

/// Reads the parameters of `a2a.tasks.Tasks` from an object keyed by their wire names.
///
/// # Errors
///
/// [`json::DecodeError`] naming the parameter and what the declaration says belongs there.
pub fn decode_params_a2a_tasks_tasks(value: &json::Value, at: &str) -> Result<(Option<crate::tasks::ContextId>, Option<crate::tasks::TaskState>, Option<crate::primitives::Timestamp>,), json::DecodeError> {
    Ok((
        match value.member("context_id") {
            None | Some(json::Value::Null) => None,
            Some(member0) => {
                let at0 = json::nested(at, "context_id");
                Some(decode_a2a_tasks_context_id(member0, &at0)?)
            }
        },
        match value.member("status") {
            None | Some(json::Value::Null) => None,
            Some(member1) => {
                let at1 = json::nested(at, "status");
                Some(decode_a2a_tasks_task_state(member1, &at1)?)
            }
        },
        match value.member("status_timestamp_after") {
            None | Some(json::Value::Null) => None,
            Some(member2) => {
                let at2 = json::nested(at, "status_timestamp_after");
                Some(crate::primitives::Timestamp(json::text_at(member2, &at2, "an RFC 3339 instant")?.to_owned()))
            }
        },
    ))
}

/// Writes the input of `a2a.discovery.GetExtendedAgentCard` as JSON.
pub fn encode_command_a2a_discovery_get_extended_agent_card(value: &crate::discovery::GetExtendedAgentCard, out: &mut String) {
    out.push('{');
    if let Some(held0) = &value.tenant {
        json::member(out, "tenant");
        json::push_text(out, &*held0);
    }
    out.push('}');
}

/// Reads the input of `a2a.discovery.GetExtendedAgentCard` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_a2a_discovery_get_extended_agent_card(value: &json::Value, at: &str) -> Result<crate::discovery::GetExtendedAgentCard, json::DecodeError> {
    Ok(crate::discovery::GetExtendedAgentCard {
        tenant: match value.member("tenant") {
            None | Some(json::Value::Null) => None,
            Some(member0) => {
                let at0 = json::nested(at, "tenant");
                Some(json::text_at(member0, &at0, "a string")?.to_owned())
            }
        },
    })
}

/// Writes the response of `a2a.discovery.GetExtendedAgentCard` as JSON.
pub fn encode_response_a2a_discovery_get_extended_agent_card(value: &crate::discovery::GetExtendedAgentCardResponse, out: &mut String) {
    out.push('{');
    json::member(out, "card");
    encode_a2a_discovery_agent_card(&value.card, out);
    out.push('}');
}

/// Writes the outcome of `a2a.discovery.GetExtendedAgentCard` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_a2a_discovery_get_extended_agent_card(value: &crate::discovery::GetExtendedAgentCardOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::discovery::GetExtendedAgentCardOutcome::NotSupported { error } => {
            json::member(out, "outcome");
            json::push_text(out, "not-supported");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.discovery.UnsupportedOperation");
            json::member(out, "payload");
            encode_error_a2a_discovery_unsupported_operation(error, out);
            out.push('}');
        }
        crate::discovery::GetExtendedAgentCardOutcome::NotConfigured { error } => {
            json::member(out, "outcome");
            json::push_text(out, "not-configured");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.discovery.ExtendedAgentCardNotConfigured");
            json::member(out, "payload");
            encode_error_a2a_discovery_extended_agent_card_not_configured(error, out);
            out.push('}');
        }
        crate::discovery::GetExtendedAgentCardOutcome::Returned { .. } => {
            json::member(out, "outcome");
            json::push_text(out, "returned");
            json::member(out, "published");
            out.push('[');
            out.push(']');
        }
        crate::discovery::GetExtendedAgentCardOutcome::VersionNotSupported { error } => {
            json::member(out, "outcome");
            json::push_text(out, "version-not-supported");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.discovery.VersionNotSupported");
            json::member(out, "payload");
            encode_error_a2a_discovery_version_not_supported(error, out);
            out.push('}');
        }
        crate::discovery::GetExtendedAgentCardOutcome::ExtensionRequired { error } => {
            json::member(out, "outcome");
            json::push_text(out, "extension-required");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.discovery.ExtensionSupportRequired");
            json::member(out, "payload");
            encode_error_a2a_discovery_extension_support_required(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `a2a.tasks.CancelTask` as JSON.
pub fn encode_command_a2a_tasks_cancel_task(value: &crate::tasks::CancelTask, out: &mut String) {
    out.push('{');
    if let Some(held0) = &value.tenant {
        json::member(out, "tenant");
        json::push_text(out, &*held0);
    }
    json::member(out, "id");
    encode_a2a_tasks_task_id(&value.id, out);
    if let Some(held0) = &value.metadata {
        json::member(out, "metadata");
        json::push_value(out, &*held0);
    }
    out.push('}');
}

/// Reads the input of `a2a.tasks.CancelTask` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_a2a_tasks_cancel_task(value: &json::Value, at: &str) -> Result<crate::tasks::CancelTask, json::DecodeError> {
    Ok(crate::tasks::CancelTask {
        tenant: match value.member("tenant") {
            None | Some(json::Value::Null) => None,
            Some(member0) => {
                let at0 = json::nested(at, "tenant");
                Some(json::text_at(member0, &at0, "a string")?.to_owned())
            }
        },
        id: {
            let at1 = json::nested(at, "id");
            let member1 = json::member_at(value, at, "id")?;
            decode_a2a_tasks_task_id(member1, &at1)?
        },
        metadata: match value.member("metadata") {
            None | Some(json::Value::Null) => None,
            Some(member2) => {
                let at2 = json::nested(at, "metadata");
                Some(json::value_at(member2, &at2))
            }
        },
    })
}

/// Writes the response of `a2a.tasks.CancelTask` as JSON.
pub fn encode_response_a2a_tasks_cancel_task(value: &crate::tasks::CancelTaskResponse, out: &mut String) {
    out.push('{');
    json::member(out, "task");
    encode_a2a_tasks_task_snapshot(&value.task, out);
    out.push('}');
}

/// Writes the outcome of `a2a.tasks.CancelTask` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_a2a_tasks_cancel_task(value: &crate::tasks::CancelTaskOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::tasks::CancelTaskOutcome::NotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.TaskNotFound");
            json::member(out, "payload");
            encode_error_a2a_tasks_task_not_found(error, out);
            out.push('}');
        }
        crate::tasks::CancelTaskOutcome::NotCancelableNow { error } => {
            json::member(out, "outcome");
            json::push_text(out, "not-cancelable-now");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.TaskNotCancelable");
            json::member(out, "payload");
            encode_error_a2a_tasks_task_not_cancelable(error, out);
            out.push('}');
        }
        crate::tasks::CancelTaskOutcome::Canceled { task_status_updated, .. } => {
            json::member(out, "outcome");
            json::push_text(out, "canceled");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "a2a.tasks.TaskStatusUpdated");
            json::member(out, "payload");
            encode_event_a2a_tasks_task_status_updated(task_status_updated, out);
            out.push('}');
            out.push(']');
        }
        crate::tasks::CancelTaskOutcome::NotCancelable { error } => {
            json::member(out, "outcome");
            json::push_text(out, "not-cancelable");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.TaskNotCancelable");
            json::member(out, "payload");
            encode_error_a2a_tasks_task_not_cancelable(error, out);
            out.push('}');
        }
        crate::tasks::CancelTaskOutcome::VersionNotSupported { error } => {
            json::member(out, "outcome");
            json::push_text(out, "version-not-supported");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.VersionNotSupported");
            json::member(out, "payload");
            encode_error_a2a_tasks_version_not_supported(error, out);
            out.push('}');
        }
        crate::tasks::CancelTaskOutcome::ExtensionRequired { error } => {
            json::member(out, "outcome");
            json::push_text(out, "extension-required");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.ExtensionSupportRequired");
            json::member(out, "payload");
            encode_error_a2a_tasks_extension_support_required(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `a2a.tasks.CompleteTask` as JSON.
pub fn encode_command_a2a_tasks_complete_task(value: &crate::tasks::CompleteTask, out: &mut String) {
    out.push('{');
    json::member(out, "id");
    encode_a2a_tasks_task_id(&value.id, out);
    if let Some(held0) = &value.status_message {
        json::member(out, "status_message");
        encode_a2a_tasks_message(&*held0, out);
    }
    out.push('}');
}

/// Reads the input of `a2a.tasks.CompleteTask` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_a2a_tasks_complete_task(value: &json::Value, at: &str) -> Result<crate::tasks::CompleteTask, json::DecodeError> {
    Ok(crate::tasks::CompleteTask {
        id: {
            let at0 = json::nested(at, "id");
            let member0 = json::member_at(value, at, "id")?;
            decode_a2a_tasks_task_id(member0, &at0)?
        },
        status_message: match value.member("status_message") {
            None | Some(json::Value::Null) => None,
            Some(member1) => {
                let at1 = json::nested(at, "status_message");
                Some(decode_a2a_tasks_message(member1, &at1)?)
            }
        },
    })
}

/// Writes the outcome of `a2a.tasks.CompleteTask` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_a2a_tasks_complete_task(value: &crate::tasks::CompleteTaskOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::tasks::CompleteTaskOutcome::Moved { task_status_updated } => {
            json::member(out, "outcome");
            json::push_text(out, "moved");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "a2a.tasks.TaskStatusUpdated");
            json::member(out, "payload");
            encode_event_a2a_tasks_task_status_updated(task_status_updated, out);
            out.push('}');
            out.push(']');
        }
        crate::tasks::CompleteTaskOutcome::NotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.TaskNotFound");
            json::member(out, "payload");
            encode_error_a2a_tasks_task_not_found(error, out);
            out.push('}');
        }
        crate::tasks::CompleteTaskOutcome::WrongState { error } => {
            json::member(out, "outcome");
            json::push_text(out, "wrong-state");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.TaskStateConflict");
            json::member(out, "payload");
            encode_error_a2a_tasks_task_state_conflict(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `a2a.tasks.CreateTaskPushNotificationConfig` as JSON.
pub fn encode_command_a2a_tasks_create_task_push_notification_config(value: &crate::tasks::CreateTaskPushNotificationConfig, out: &mut String) {
    out.push('{');
    if let Some(held0) = &value.tenant {
        json::member(out, "tenant");
        json::push_text(out, &*held0);
    }
    if let Some(held0) = &value.id {
        json::member(out, "id");
        encode_a2a_tasks_push_notification_config_id(&*held0, out);
    }
    json::member(out, "taskId");
    encode_a2a_tasks_task_id(&value.task_id, out);
    json::member(out, "url");
    json::push_text(out, &value.url);
    if let Some(held0) = &value.token {
        json::member(out, "token");
        json::push_text(out, &*held0);
    }
    if let Some(held0) = &value.authentication {
        json::member(out, "authentication");
        encode_a2a_tasks_authentication_info(&*held0, out);
    }
    out.push('}');
}

/// Reads the input of `a2a.tasks.CreateTaskPushNotificationConfig` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_a2a_tasks_create_task_push_notification_config(value: &json::Value, at: &str) -> Result<crate::tasks::CreateTaskPushNotificationConfig, json::DecodeError> {
    Ok(crate::tasks::CreateTaskPushNotificationConfig {
        tenant: match value.member("tenant") {
            None | Some(json::Value::Null) => None,
            Some(member0) => {
                let at0 = json::nested(at, "tenant");
                Some(json::text_at(member0, &at0, "a string")?.to_owned())
            }
        },
        id: match value.member("id") {
            None | Some(json::Value::Null) => None,
            Some(member1) => {
                let at1 = json::nested(at, "id");
                Some(decode_a2a_tasks_push_notification_config_id(member1, &at1)?)
            }
        },
        task_id: {
            let at2 = json::nested(at, "taskId");
            let member2 = json::member_at(value, at, "taskId")?;
            decode_a2a_tasks_task_id(member2, &at2)?
        },
        url: {
            let at3 = json::nested(at, "url");
            let member3 = json::member_at(value, at, "url")?;
            json::text_at(member3, &at3, "a string")?.to_owned()
        },
        token: match value.member("token") {
            None | Some(json::Value::Null) => None,
            Some(member4) => {
                let at4 = json::nested(at, "token");
                Some(json::text_at(member4, &at4, "a string")?.to_owned())
            }
        },
        authentication: match value.member("authentication") {
            None | Some(json::Value::Null) => None,
            Some(member5) => {
                let at5 = json::nested(at, "authentication");
                Some(decode_a2a_tasks_authentication_info(member5, &at5)?)
            }
        },
    })
}

/// Writes the response of `a2a.tasks.CreateTaskPushNotificationConfig` as JSON.
pub fn encode_response_a2a_tasks_create_task_push_notification_config(value: &crate::tasks::CreateTaskPushNotificationConfigResponse, out: &mut String) {
    out.push('{');
    json::member(out, "config");
    encode_a2a_tasks_task_push_notification_config(&value.config, out);
    out.push('}');
}

/// Writes the outcome of `a2a.tasks.CreateTaskPushNotificationConfig` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_a2a_tasks_create_task_push_notification_config(value: &crate::tasks::CreateTaskPushNotificationConfigOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::tasks::CreateTaskPushNotificationConfigOutcome::TaskNotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "task-not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.TaskNotFound");
            json::member(out, "payload");
            encode_error_a2a_tasks_task_not_found(error, out);
            out.push('}');
        }
        crate::tasks::CreateTaskPushNotificationConfigOutcome::Created { push_notification_config_created, .. } => {
            json::member(out, "outcome");
            json::push_text(out, "created");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "a2a.tasks.PushNotificationConfigCreated");
            json::member(out, "payload");
            encode_event_a2a_tasks_push_notification_config_created(push_notification_config_created, out);
            out.push('}');
            out.push(']');
        }
        crate::tasks::CreateTaskPushNotificationConfigOutcome::Assigned { push_notification_config_created, .. } => {
            json::member(out, "outcome");
            json::push_text(out, "assigned");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "a2a.tasks.PushNotificationConfigCreated");
            json::member(out, "payload");
            encode_event_a2a_tasks_push_notification_config_created(push_notification_config_created, out);
            out.push('}');
            out.push(']');
        }
        crate::tasks::CreateTaskPushNotificationConfigOutcome::VersionNotSupported { error } => {
            json::member(out, "outcome");
            json::push_text(out, "version-not-supported");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.VersionNotSupported");
            json::member(out, "payload");
            encode_error_a2a_tasks_version_not_supported(error, out);
            out.push('}');
        }
        crate::tasks::CreateTaskPushNotificationConfigOutcome::ExtensionRequired { error } => {
            json::member(out, "outcome");
            json::push_text(out, "extension-required");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.ExtensionSupportRequired");
            json::member(out, "payload");
            encode_error_a2a_tasks_extension_support_required(error, out);
            out.push('}');
        }
        crate::tasks::CreateTaskPushNotificationConfigOutcome::PushNotSupported { error } => {
            json::member(out, "outcome");
            json::push_text(out, "push-not-supported");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.PushNotificationNotSupported");
            json::member(out, "payload");
            encode_error_a2a_tasks_push_notification_not_supported(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `a2a.tasks.DeleteTaskPushNotificationConfig` as JSON.
pub fn encode_command_a2a_tasks_delete_task_push_notification_config(value: &crate::tasks::DeleteTaskPushNotificationConfig, out: &mut String) {
    out.push('{');
    if let Some(held0) = &value.tenant {
        json::member(out, "tenant");
        json::push_text(out, &*held0);
    }
    json::member(out, "taskId");
    encode_a2a_tasks_task_id(&value.task_id, out);
    json::member(out, "id");
    encode_a2a_tasks_push_notification_config_id(&value.id, out);
    out.push('}');
}

/// Reads the input of `a2a.tasks.DeleteTaskPushNotificationConfig` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_a2a_tasks_delete_task_push_notification_config(value: &json::Value, at: &str) -> Result<crate::tasks::DeleteTaskPushNotificationConfig, json::DecodeError> {
    Ok(crate::tasks::DeleteTaskPushNotificationConfig {
        tenant: match value.member("tenant") {
            None | Some(json::Value::Null) => None,
            Some(member0) => {
                let at0 = json::nested(at, "tenant");
                Some(json::text_at(member0, &at0, "a string")?.to_owned())
            }
        },
        task_id: {
            let at1 = json::nested(at, "taskId");
            let member1 = json::member_at(value, at, "taskId")?;
            decode_a2a_tasks_task_id(member1, &at1)?
        },
        id: {
            let at2 = json::nested(at, "id");
            let member2 = json::member_at(value, at, "id")?;
            decode_a2a_tasks_push_notification_config_id(member2, &at2)?
        },
    })
}

/// Writes the outcome of `a2a.tasks.DeleteTaskPushNotificationConfig` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_a2a_tasks_delete_task_push_notification_config(value: &crate::tasks::DeleteTaskPushNotificationConfigOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::tasks::DeleteTaskPushNotificationConfigOutcome::TaskNotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "task-not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.TaskNotFound");
            json::member(out, "payload");
            encode_error_a2a_tasks_task_not_found(error, out);
            out.push('}');
        }
        crate::tasks::DeleteTaskPushNotificationConfigOutcome::Deleted { push_notification_config_deleted } => {
            json::member(out, "outcome");
            json::push_text(out, "deleted");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "a2a.tasks.PushNotificationConfigDeleted");
            json::member(out, "payload");
            encode_event_a2a_tasks_push_notification_config_deleted(push_notification_config_deleted, out);
            out.push('}');
            out.push(']');
        }
        crate::tasks::DeleteTaskPushNotificationConfigOutcome::VersionNotSupported { error } => {
            json::member(out, "outcome");
            json::push_text(out, "version-not-supported");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.VersionNotSupported");
            json::member(out, "payload");
            encode_error_a2a_tasks_version_not_supported(error, out);
            out.push('}');
        }
        crate::tasks::DeleteTaskPushNotificationConfigOutcome::ExtensionRequired { error } => {
            json::member(out, "outcome");
            json::push_text(out, "extension-required");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.ExtensionSupportRequired");
            json::member(out, "payload");
            encode_error_a2a_tasks_extension_support_required(error, out);
            out.push('}');
        }
        crate::tasks::DeleteTaskPushNotificationConfigOutcome::PushNotSupported { error } => {
            json::member(out, "outcome");
            json::push_text(out, "push-not-supported");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.PushNotificationNotSupported");
            json::member(out, "payload");
            encode_error_a2a_tasks_push_notification_not_supported(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `a2a.tasks.FailTask` as JSON.
pub fn encode_command_a2a_tasks_fail_task(value: &crate::tasks::FailTask, out: &mut String) {
    out.push('{');
    json::member(out, "id");
    encode_a2a_tasks_task_id(&value.id, out);
    if let Some(held0) = &value.status_message {
        json::member(out, "status_message");
        encode_a2a_tasks_message(&*held0, out);
    }
    out.push('}');
}

/// Reads the input of `a2a.tasks.FailTask` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_a2a_tasks_fail_task(value: &json::Value, at: &str) -> Result<crate::tasks::FailTask, json::DecodeError> {
    Ok(crate::tasks::FailTask {
        id: {
            let at0 = json::nested(at, "id");
            let member0 = json::member_at(value, at, "id")?;
            decode_a2a_tasks_task_id(member0, &at0)?
        },
        status_message: match value.member("status_message") {
            None | Some(json::Value::Null) => None,
            Some(member1) => {
                let at1 = json::nested(at, "status_message");
                Some(decode_a2a_tasks_message(member1, &at1)?)
            }
        },
    })
}

/// Writes the outcome of `a2a.tasks.FailTask` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_a2a_tasks_fail_task(value: &crate::tasks::FailTaskOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::tasks::FailTaskOutcome::Moved { task_status_updated } => {
            json::member(out, "outcome");
            json::push_text(out, "moved");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "a2a.tasks.TaskStatusUpdated");
            json::member(out, "payload");
            encode_event_a2a_tasks_task_status_updated(task_status_updated, out);
            out.push('}');
            out.push(']');
        }
        crate::tasks::FailTaskOutcome::NotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.TaskNotFound");
            json::member(out, "payload");
            encode_error_a2a_tasks_task_not_found(error, out);
            out.push('}');
        }
        crate::tasks::FailTaskOutcome::WrongState { error } => {
            json::member(out, "outcome");
            json::push_text(out, "wrong-state");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.TaskStateConflict");
            json::member(out, "payload");
            encode_error_a2a_tasks_task_state_conflict(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `a2a.tasks.GetTask` as JSON.
pub fn encode_command_a2a_tasks_get_task(value: &crate::tasks::GetTask, out: &mut String) {
    out.push('{');
    if let Some(held0) = &value.tenant {
        json::member(out, "tenant");
        json::push_text(out, &*held0);
    }
    json::member(out, "id");
    encode_a2a_tasks_task_id(&value.id, out);
    if let Some(held0) = &value.history_length {
        json::member(out, "historyLength");
        json::push_integer(out, *held0);
    }
    out.push('}');
}

/// Reads the input of `a2a.tasks.GetTask` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_a2a_tasks_get_task(value: &json::Value, at: &str) -> Result<crate::tasks::GetTask, json::DecodeError> {
    Ok(crate::tasks::GetTask {
        tenant: match value.member("tenant") {
            None | Some(json::Value::Null) => None,
            Some(member0) => {
                let at0 = json::nested(at, "tenant");
                Some(json::text_at(member0, &at0, "a string")?.to_owned())
            }
        },
        id: {
            let at1 = json::nested(at, "id");
            let member1 = json::member_at(value, at, "id")?;
            decode_a2a_tasks_task_id(member1, &at1)?
        },
        history_length: match value.member("historyLength") {
            None | Some(json::Value::Null) => None,
            Some(member2) => {
                let at2 = json::nested(at, "historyLength");
                Some(json::integer_at(member2, &at2, "an integer")?)
            }
        },
    })
}

/// Writes the response of `a2a.tasks.GetTask` as JSON.
pub fn encode_response_a2a_tasks_get_task(value: &crate::tasks::GetTaskResponse, out: &mut String) {
    out.push('{');
    json::member(out, "task");
    encode_a2a_tasks_task_snapshot(&value.task, out);
    out.push('}');
}

/// Writes the outcome of `a2a.tasks.GetTask` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_a2a_tasks_get_task(value: &crate::tasks::GetTaskOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::tasks::GetTaskOutcome::NotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.TaskNotFound");
            json::member(out, "payload");
            encode_error_a2a_tasks_task_not_found(error, out);
            out.push('}');
        }
        crate::tasks::GetTaskOutcome::Found { .. } => {
            json::member(out, "outcome");
            json::push_text(out, "found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
        }
        crate::tasks::GetTaskOutcome::VersionNotSupported { error } => {
            json::member(out, "outcome");
            json::push_text(out, "version-not-supported");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.VersionNotSupported");
            json::member(out, "payload");
            encode_error_a2a_tasks_version_not_supported(error, out);
            out.push('}');
        }
        crate::tasks::GetTaskOutcome::ExtensionRequired { error } => {
            json::member(out, "outcome");
            json::push_text(out, "extension-required");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.ExtensionSupportRequired");
            json::member(out, "payload");
            encode_error_a2a_tasks_extension_support_required(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `a2a.tasks.GetTaskPushNotificationConfig` as JSON.
pub fn encode_command_a2a_tasks_get_task_push_notification_config(value: &crate::tasks::GetTaskPushNotificationConfig, out: &mut String) {
    out.push('{');
    if let Some(held0) = &value.tenant {
        json::member(out, "tenant");
        json::push_text(out, &*held0);
    }
    json::member(out, "taskId");
    encode_a2a_tasks_task_id(&value.task_id, out);
    json::member(out, "id");
    encode_a2a_tasks_push_notification_config_id(&value.id, out);
    out.push('}');
}

/// Reads the input of `a2a.tasks.GetTaskPushNotificationConfig` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_a2a_tasks_get_task_push_notification_config(value: &json::Value, at: &str) -> Result<crate::tasks::GetTaskPushNotificationConfig, json::DecodeError> {
    Ok(crate::tasks::GetTaskPushNotificationConfig {
        tenant: match value.member("tenant") {
            None | Some(json::Value::Null) => None,
            Some(member0) => {
                let at0 = json::nested(at, "tenant");
                Some(json::text_at(member0, &at0, "a string")?.to_owned())
            }
        },
        task_id: {
            let at1 = json::nested(at, "taskId");
            let member1 = json::member_at(value, at, "taskId")?;
            decode_a2a_tasks_task_id(member1, &at1)?
        },
        id: {
            let at2 = json::nested(at, "id");
            let member2 = json::member_at(value, at, "id")?;
            decode_a2a_tasks_push_notification_config_id(member2, &at2)?
        },
    })
}

/// Writes the response of `a2a.tasks.GetTaskPushNotificationConfig` as JSON.
pub fn encode_response_a2a_tasks_get_task_push_notification_config(value: &crate::tasks::GetTaskPushNotificationConfigResponse, out: &mut String) {
    out.push('{');
    json::member(out, "config");
    encode_a2a_tasks_task_push_notification_config(&value.config, out);
    out.push('}');
}

/// Writes the outcome of `a2a.tasks.GetTaskPushNotificationConfig` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_a2a_tasks_get_task_push_notification_config(value: &crate::tasks::GetTaskPushNotificationConfigOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::tasks::GetTaskPushNotificationConfigOutcome::VersionNotSupported { error } => {
            json::member(out, "outcome");
            json::push_text(out, "version-not-supported");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.VersionNotSupported");
            json::member(out, "payload");
            encode_error_a2a_tasks_version_not_supported(error, out);
            out.push('}');
        }
        crate::tasks::GetTaskPushNotificationConfigOutcome::ExtensionRequired { error } => {
            json::member(out, "outcome");
            json::push_text(out, "extension-required");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.ExtensionSupportRequired");
            json::member(out, "payload");
            encode_error_a2a_tasks_extension_support_required(error, out);
            out.push('}');
        }
        crate::tasks::GetTaskPushNotificationConfigOutcome::PushNotSupported { error } => {
            json::member(out, "outcome");
            json::push_text(out, "push-not-supported");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.PushNotificationNotSupported");
            json::member(out, "payload");
            encode_error_a2a_tasks_push_notification_not_supported(error, out);
            out.push('}');
        }
        crate::tasks::GetTaskPushNotificationConfigOutcome::NotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.TaskNotFound");
            json::member(out, "payload");
            encode_error_a2a_tasks_task_not_found(error, out);
            out.push('}');
        }
        crate::tasks::GetTaskPushNotificationConfigOutcome::ConfigNotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "config-not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.TaskNotFound");
            json::member(out, "payload");
            encode_error_a2a_tasks_task_not_found(error, out);
            out.push('}');
        }
        crate::tasks::GetTaskPushNotificationConfigOutcome::Found { .. } => {
            json::member(out, "outcome");
            json::push_text(out, "found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
        }
    }
    out.push('}');
}

/// Writes the input of `a2a.tasks.ListTaskPushNotificationConfigs` as JSON.
pub fn encode_command_a2a_tasks_list_task_push_notification_configs(value: &crate::tasks::ListTaskPushNotificationConfigs, out: &mut String) {
    out.push('{');
    if let Some(held0) = &value.tenant {
        json::member(out, "tenant");
        json::push_text(out, &*held0);
    }
    json::member(out, "taskId");
    encode_a2a_tasks_task_id(&value.task_id, out);
    if let Some(held0) = &value.page_size {
        json::member(out, "pageSize");
        json::push_integer(out, *held0);
    }
    if let Some(held0) = &value.page_token {
        json::member(out, "pageToken");
        json::push_text(out, &*held0);
    }
    out.push('}');
}

/// Reads the input of `a2a.tasks.ListTaskPushNotificationConfigs` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_a2a_tasks_list_task_push_notification_configs(value: &json::Value, at: &str) -> Result<crate::tasks::ListTaskPushNotificationConfigs, json::DecodeError> {
    Ok(crate::tasks::ListTaskPushNotificationConfigs {
        tenant: match value.member("tenant") {
            None | Some(json::Value::Null) => None,
            Some(member0) => {
                let at0 = json::nested(at, "tenant");
                Some(json::text_at(member0, &at0, "a string")?.to_owned())
            }
        },
        task_id: {
            let at1 = json::nested(at, "taskId");
            let member1 = json::member_at(value, at, "taskId")?;
            decode_a2a_tasks_task_id(member1, &at1)?
        },
        page_size: match value.member("pageSize") {
            None | Some(json::Value::Null) => None,
            Some(member2) => {
                let at2 = json::nested(at, "pageSize");
                Some(json::integer_at(member2, &at2, "an integer")?)
            }
        },
        page_token: match value.member("pageToken") {
            None | Some(json::Value::Null) => None,
            Some(member3) => {
                let at3 = json::nested(at, "pageToken");
                Some(json::text_at(member3, &at3, "a string")?.to_owned())
            }
        },
    })
}

/// Writes the response of `a2a.tasks.ListTaskPushNotificationConfigs` as JSON.
pub fn encode_response_a2a_tasks_list_task_push_notification_configs(value: &crate::tasks::ListTaskPushNotificationConfigsResponse, out: &mut String) {
    out.push('{');
    if let Some(held0) = &value.configs {
        json::member(out, "configs");
        out.push('[');
        for (index1, item1) in (*held0).iter().enumerate() {
            if index1 > 0 {
                out.push(',');
            }
            encode_a2a_tasks_task_push_notification_config(&*item1, out);
        }
        out.push(']');
    }
    if let Some(held0) = &value.next_page_token {
        json::member(out, "nextPageToken");
        json::push_text(out, &*held0);
    }
    out.push('}');
}

/// Writes the outcome of `a2a.tasks.ListTaskPushNotificationConfigs` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_a2a_tasks_list_task_push_notification_configs(value: &crate::tasks::ListTaskPushNotificationConfigsOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::tasks::ListTaskPushNotificationConfigsOutcome::TaskNotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "task-not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.TaskNotFound");
            json::member(out, "payload");
            encode_error_a2a_tasks_task_not_found(error, out);
            out.push('}');
        }
        crate::tasks::ListTaskPushNotificationConfigsOutcome::Listed { .. } => {
            json::member(out, "outcome");
            json::push_text(out, "listed");
            json::member(out, "published");
            out.push('[');
            out.push(']');
        }
        crate::tasks::ListTaskPushNotificationConfigsOutcome::VersionNotSupported { error } => {
            json::member(out, "outcome");
            json::push_text(out, "version-not-supported");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.VersionNotSupported");
            json::member(out, "payload");
            encode_error_a2a_tasks_version_not_supported(error, out);
            out.push('}');
        }
        crate::tasks::ListTaskPushNotificationConfigsOutcome::ExtensionRequired { error } => {
            json::member(out, "outcome");
            json::push_text(out, "extension-required");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.ExtensionSupportRequired");
            json::member(out, "payload");
            encode_error_a2a_tasks_extension_support_required(error, out);
            out.push('}');
        }
        crate::tasks::ListTaskPushNotificationConfigsOutcome::PushNotSupported { error } => {
            json::member(out, "outcome");
            json::push_text(out, "push-not-supported");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.PushNotificationNotSupported");
            json::member(out, "payload");
            encode_error_a2a_tasks_push_notification_not_supported(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `a2a.tasks.RejectTask` as JSON.
pub fn encode_command_a2a_tasks_reject_task(value: &crate::tasks::RejectTask, out: &mut String) {
    out.push('{');
    json::member(out, "id");
    encode_a2a_tasks_task_id(&value.id, out);
    if let Some(held0) = &value.status_message {
        json::member(out, "status_message");
        encode_a2a_tasks_message(&*held0, out);
    }
    out.push('}');
}

/// Reads the input of `a2a.tasks.RejectTask` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_a2a_tasks_reject_task(value: &json::Value, at: &str) -> Result<crate::tasks::RejectTask, json::DecodeError> {
    Ok(crate::tasks::RejectTask {
        id: {
            let at0 = json::nested(at, "id");
            let member0 = json::member_at(value, at, "id")?;
            decode_a2a_tasks_task_id(member0, &at0)?
        },
        status_message: match value.member("status_message") {
            None | Some(json::Value::Null) => None,
            Some(member1) => {
                let at1 = json::nested(at, "status_message");
                Some(decode_a2a_tasks_message(member1, &at1)?)
            }
        },
    })
}

/// Writes the outcome of `a2a.tasks.RejectTask` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_a2a_tasks_reject_task(value: &crate::tasks::RejectTaskOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::tasks::RejectTaskOutcome::Moved { task_status_updated } => {
            json::member(out, "outcome");
            json::push_text(out, "moved");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "a2a.tasks.TaskStatusUpdated");
            json::member(out, "payload");
            encode_event_a2a_tasks_task_status_updated(task_status_updated, out);
            out.push('}');
            out.push(']');
        }
        crate::tasks::RejectTaskOutcome::NotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.TaskNotFound");
            json::member(out, "payload");
            encode_error_a2a_tasks_task_not_found(error, out);
            out.push('}');
        }
        crate::tasks::RejectTaskOutcome::WrongState { error } => {
            json::member(out, "outcome");
            json::push_text(out, "wrong-state");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.TaskStateConflict");
            json::member(out, "payload");
            encode_error_a2a_tasks_task_state_conflict(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `a2a.tasks.RequireAuth` as JSON.
pub fn encode_command_a2a_tasks_require_auth(value: &crate::tasks::RequireAuth, out: &mut String) {
    out.push('{');
    json::member(out, "id");
    encode_a2a_tasks_task_id(&value.id, out);
    if let Some(held0) = &value.status_message {
        json::member(out, "status_message");
        encode_a2a_tasks_message(&*held0, out);
    }
    out.push('}');
}

/// Reads the input of `a2a.tasks.RequireAuth` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_a2a_tasks_require_auth(value: &json::Value, at: &str) -> Result<crate::tasks::RequireAuth, json::DecodeError> {
    Ok(crate::tasks::RequireAuth {
        id: {
            let at0 = json::nested(at, "id");
            let member0 = json::member_at(value, at, "id")?;
            decode_a2a_tasks_task_id(member0, &at0)?
        },
        status_message: match value.member("status_message") {
            None | Some(json::Value::Null) => None,
            Some(member1) => {
                let at1 = json::nested(at, "status_message");
                Some(decode_a2a_tasks_message(member1, &at1)?)
            }
        },
    })
}

/// Writes the outcome of `a2a.tasks.RequireAuth` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_a2a_tasks_require_auth(value: &crate::tasks::RequireAuthOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::tasks::RequireAuthOutcome::Moved { task_status_updated } => {
            json::member(out, "outcome");
            json::push_text(out, "moved");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "a2a.tasks.TaskStatusUpdated");
            json::member(out, "payload");
            encode_event_a2a_tasks_task_status_updated(task_status_updated, out);
            out.push('}');
            out.push(']');
        }
        crate::tasks::RequireAuthOutcome::NotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.TaskNotFound");
            json::member(out, "payload");
            encode_error_a2a_tasks_task_not_found(error, out);
            out.push('}');
        }
        crate::tasks::RequireAuthOutcome::WrongState { error } => {
            json::member(out, "outcome");
            json::push_text(out, "wrong-state");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.TaskStateConflict");
            json::member(out, "payload");
            encode_error_a2a_tasks_task_state_conflict(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `a2a.tasks.RequireInput` as JSON.
pub fn encode_command_a2a_tasks_require_input(value: &crate::tasks::RequireInput, out: &mut String) {
    out.push('{');
    json::member(out, "id");
    encode_a2a_tasks_task_id(&value.id, out);
    if let Some(held0) = &value.status_message {
        json::member(out, "status_message");
        encode_a2a_tasks_message(&*held0, out);
    }
    out.push('}');
}

/// Reads the input of `a2a.tasks.RequireInput` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_a2a_tasks_require_input(value: &json::Value, at: &str) -> Result<crate::tasks::RequireInput, json::DecodeError> {
    Ok(crate::tasks::RequireInput {
        id: {
            let at0 = json::nested(at, "id");
            let member0 = json::member_at(value, at, "id")?;
            decode_a2a_tasks_task_id(member0, &at0)?
        },
        status_message: match value.member("status_message") {
            None | Some(json::Value::Null) => None,
            Some(member1) => {
                let at1 = json::nested(at, "status_message");
                Some(decode_a2a_tasks_message(member1, &at1)?)
            }
        },
    })
}

/// Writes the outcome of `a2a.tasks.RequireInput` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_a2a_tasks_require_input(value: &crate::tasks::RequireInputOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::tasks::RequireInputOutcome::Moved { task_status_updated } => {
            json::member(out, "outcome");
            json::push_text(out, "moved");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "a2a.tasks.TaskStatusUpdated");
            json::member(out, "payload");
            encode_event_a2a_tasks_task_status_updated(task_status_updated, out);
            out.push('}');
            out.push(']');
        }
        crate::tasks::RequireInputOutcome::NotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.TaskNotFound");
            json::member(out, "payload");
            encode_error_a2a_tasks_task_not_found(error, out);
            out.push('}');
        }
        crate::tasks::RequireInputOutcome::WrongState { error } => {
            json::member(out, "outcome");
            json::push_text(out, "wrong-state");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.TaskStateConflict");
            json::member(out, "payload");
            encode_error_a2a_tasks_task_state_conflict(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `a2a.tasks.SendMessage` as JSON.
pub fn encode_command_a2a_tasks_send_message(value: &crate::tasks::SendMessage, out: &mut String) {
    out.push('{');
    if let Some(held0) = &value.tenant {
        json::member(out, "tenant");
        json::push_text(out, &*held0);
    }
    json::member(out, "message");
    encode_a2a_tasks_message(&value.message, out);
    if let Some(held0) = &value.configuration {
        json::member(out, "configuration");
        encode_a2a_tasks_send_message_configuration(&*held0, out);
    }
    if let Some(held0) = &value.metadata {
        json::member(out, "metadata");
        json::push_value(out, &*held0);
    }
    out.push('}');
}

/// Reads the input of `a2a.tasks.SendMessage` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_a2a_tasks_send_message(value: &json::Value, at: &str) -> Result<crate::tasks::SendMessage, json::DecodeError> {
    Ok(crate::tasks::SendMessage {
        tenant: match value.member("tenant") {
            None | Some(json::Value::Null) => None,
            Some(member0) => {
                let at0 = json::nested(at, "tenant");
                Some(json::text_at(member0, &at0, "a string")?.to_owned())
            }
        },
        message: {
            let at1 = json::nested(at, "message");
            let member1 = json::member_at(value, at, "message")?;
            decode_a2a_tasks_message(member1, &at1)?
        },
        configuration: match value.member("configuration") {
            None | Some(json::Value::Null) => None,
            Some(member2) => {
                let at2 = json::nested(at, "configuration");
                Some(decode_a2a_tasks_send_message_configuration(member2, &at2)?)
            }
        },
        metadata: match value.member("metadata") {
            None | Some(json::Value::Null) => None,
            Some(member3) => {
                let at3 = json::nested(at, "metadata");
                Some(json::value_at(member3, &at3))
            }
        },
    })
}

/// Writes the response of `a2a.tasks.SendMessage` as JSON.
pub fn encode_response_a2a_tasks_send_message(value: &crate::tasks::SendMessageResponse, out: &mut String) {
    out.push('{');
    if let Some(held0) = &value.task {
        json::member(out, "task");
        encode_a2a_tasks_task_snapshot(&*held0, out);
    }
    if let Some(held0) = &value.reply {
        json::member(out, "message");
        encode_a2a_tasks_message(&*held0, out);
    }
    out.push('}');
}

/// Writes the outcome of `a2a.tasks.SendMessage` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_a2a_tasks_send_message(value: &crate::tasks::SendMessageOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::tasks::SendMessageOutcome::TaskNotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "task-not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.TaskNotFound");
            json::member(out, "payload");
            encode_error_a2a_tasks_task_not_found(error, out);
            out.push('}');
        }
        crate::tasks::SendMessageOutcome::TaskTerminal { error } => {
            json::member(out, "outcome");
            json::push_text(out, "task-terminal");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.UnsupportedOperation");
            json::member(out, "payload");
            encode_error_a2a_tasks_unsupported_operation(error, out);
            out.push('}');
        }
        crate::tasks::SendMessageOutcome::ContextMismatch { error } => {
            json::member(out, "outcome");
            json::push_text(out, "context-mismatch");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.ContextMismatch");
            json::member(out, "payload");
            encode_error_a2a_tasks_context_mismatch(error, out);
            out.push('}');
        }
        crate::tasks::SendMessageOutcome::ContextNotAccepted { error } => {
            json::member(out, "outcome");
            json::push_text(out, "context-not-accepted");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.ContextNotAccepted");
            json::member(out, "payload");
            encode_error_a2a_tasks_context_not_accepted(error, out);
            out.push('}');
        }
        crate::tasks::SendMessageOutcome::ContentTypeNotSupported { error } => {
            json::member(out, "outcome");
            json::push_text(out, "content-type-not-supported");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.ContentTypeNotSupported");
            json::member(out, "payload");
            encode_error_a2a_tasks_content_type_not_supported(error, out);
            out.push('}');
        }
        crate::tasks::SendMessageOutcome::Continued { .. } => {
            json::member(out, "outcome");
            json::push_text(out, "continued");
            json::member(out, "published");
            out.push('[');
            out.push(']');
        }
        crate::tasks::SendMessageOutcome::DirectMessage { .. } => {
            json::member(out, "outcome");
            json::push_text(out, "direct-message");
            json::member(out, "published");
            out.push('[');
            out.push(']');
        }
        crate::tasks::SendMessageOutcome::TaskCreated { task_created, .. } => {
            json::member(out, "outcome");
            json::push_text(out, "task-created");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "a2a.tasks.TaskCreated");
            json::member(out, "payload");
            encode_event_a2a_tasks_task_created(task_created, out);
            out.push('}');
            out.push(']');
        }
        crate::tasks::SendMessageOutcome::VersionNotSupported { error } => {
            json::member(out, "outcome");
            json::push_text(out, "version-not-supported");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.VersionNotSupported");
            json::member(out, "payload");
            encode_error_a2a_tasks_version_not_supported(error, out);
            out.push('}');
        }
        crate::tasks::SendMessageOutcome::ExtensionRequired { error } => {
            json::member(out, "outcome");
            json::push_text(out, "extension-required");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.ExtensionSupportRequired");
            json::member(out, "payload");
            encode_error_a2a_tasks_extension_support_required(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `a2a.tasks.SendStreamingMessage` as JSON.
pub fn encode_command_a2a_tasks_send_streaming_message(value: &crate::tasks::SendStreamingMessage, out: &mut String) {
    out.push('{');
    if let Some(held0) = &value.tenant {
        json::member(out, "tenant");
        json::push_text(out, &*held0);
    }
    json::member(out, "message");
    encode_a2a_tasks_message(&value.message, out);
    if let Some(held0) = &value.configuration {
        json::member(out, "configuration");
        encode_a2a_tasks_send_message_configuration(&*held0, out);
    }
    if let Some(held0) = &value.metadata {
        json::member(out, "metadata");
        json::push_value(out, &*held0);
    }
    out.push('}');
}

/// Reads the input of `a2a.tasks.SendStreamingMessage` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_a2a_tasks_send_streaming_message(value: &json::Value, at: &str) -> Result<crate::tasks::SendStreamingMessage, json::DecodeError> {
    Ok(crate::tasks::SendStreamingMessage {
        tenant: match value.member("tenant") {
            None | Some(json::Value::Null) => None,
            Some(member0) => {
                let at0 = json::nested(at, "tenant");
                Some(json::text_at(member0, &at0, "a string")?.to_owned())
            }
        },
        message: {
            let at1 = json::nested(at, "message");
            let member1 = json::member_at(value, at, "message")?;
            decode_a2a_tasks_message(member1, &at1)?
        },
        configuration: match value.member("configuration") {
            None | Some(json::Value::Null) => None,
            Some(member2) => {
                let at2 = json::nested(at, "configuration");
                Some(decode_a2a_tasks_send_message_configuration(member2, &at2)?)
            }
        },
        metadata: match value.member("metadata") {
            None | Some(json::Value::Null) => None,
            Some(member3) => {
                let at3 = json::nested(at, "metadata");
                Some(json::value_at(member3, &at3))
            }
        },
    })
}

/// Writes the response of `a2a.tasks.SendStreamingMessage` as JSON.
pub fn encode_response_a2a_tasks_send_streaming_message(value: &crate::tasks::SendStreamingMessageResponse, out: &mut String) {
    out.push('{');
    if let Some(held0) = &value.task {
        json::member(out, "task");
        encode_a2a_tasks_task_snapshot(&*held0, out);
    }
    if let Some(held0) = &value.reply {
        json::member(out, "message");
        encode_a2a_tasks_message(&*held0, out);
    }
    out.push('}');
}

/// Writes the outcome of `a2a.tasks.SendStreamingMessage` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_a2a_tasks_send_streaming_message(value: &crate::tasks::SendStreamingMessageOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::tasks::SendStreamingMessageOutcome::TaskNotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "task-not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.TaskNotFound");
            json::member(out, "payload");
            encode_error_a2a_tasks_task_not_found(error, out);
            out.push('}');
        }
        crate::tasks::SendStreamingMessageOutcome::TaskTerminal { error } => {
            json::member(out, "outcome");
            json::push_text(out, "task-terminal");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.UnsupportedOperation");
            json::member(out, "payload");
            encode_error_a2a_tasks_unsupported_operation(error, out);
            out.push('}');
        }
        crate::tasks::SendStreamingMessageOutcome::ContextMismatch { error } => {
            json::member(out, "outcome");
            json::push_text(out, "context-mismatch");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.ContextMismatch");
            json::member(out, "payload");
            encode_error_a2a_tasks_context_mismatch(error, out);
            out.push('}');
        }
        crate::tasks::SendStreamingMessageOutcome::ContextNotAccepted { error } => {
            json::member(out, "outcome");
            json::push_text(out, "context-not-accepted");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.ContextNotAccepted");
            json::member(out, "payload");
            encode_error_a2a_tasks_context_not_accepted(error, out);
            out.push('}');
        }
        crate::tasks::SendStreamingMessageOutcome::ContentTypeNotSupported { error } => {
            json::member(out, "outcome");
            json::push_text(out, "content-type-not-supported");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.ContentTypeNotSupported");
            json::member(out, "payload");
            encode_error_a2a_tasks_content_type_not_supported(error, out);
            out.push('}');
        }
        crate::tasks::SendStreamingMessageOutcome::Continued { .. } => {
            json::member(out, "outcome");
            json::push_text(out, "continued");
            json::member(out, "published");
            out.push('[');
            out.push(']');
        }
        crate::tasks::SendStreamingMessageOutcome::DirectMessage { .. } => {
            json::member(out, "outcome");
            json::push_text(out, "direct-message");
            json::member(out, "published");
            out.push('[');
            out.push(']');
        }
        crate::tasks::SendStreamingMessageOutcome::TaskCreated { task_created, .. } => {
            json::member(out, "outcome");
            json::push_text(out, "task-created");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "a2a.tasks.TaskCreated");
            json::member(out, "payload");
            encode_event_a2a_tasks_task_created(task_created, out);
            out.push('}');
            out.push(']');
        }
        crate::tasks::SendStreamingMessageOutcome::VersionNotSupported { error } => {
            json::member(out, "outcome");
            json::push_text(out, "version-not-supported");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.VersionNotSupported");
            json::member(out, "payload");
            encode_error_a2a_tasks_version_not_supported(error, out);
            out.push('}');
        }
        crate::tasks::SendStreamingMessageOutcome::ExtensionRequired { error } => {
            json::member(out, "outcome");
            json::push_text(out, "extension-required");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.ExtensionSupportRequired");
            json::member(out, "payload");
            encode_error_a2a_tasks_extension_support_required(error, out);
            out.push('}');
        }
        crate::tasks::SendStreamingMessageOutcome::StreamingNotSupported { error } => {
            json::member(out, "outcome");
            json::push_text(out, "streaming-not-supported");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.UnsupportedOperation");
            json::member(out, "payload");
            encode_error_a2a_tasks_unsupported_operation(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `a2a.tasks.StartWorking` as JSON.
pub fn encode_command_a2a_tasks_start_working(value: &crate::tasks::StartWorking, out: &mut String) {
    out.push('{');
    json::member(out, "id");
    encode_a2a_tasks_task_id(&value.id, out);
    if let Some(held0) = &value.status_message {
        json::member(out, "status_message");
        encode_a2a_tasks_message(&*held0, out);
    }
    out.push('}');
}

/// Reads the input of `a2a.tasks.StartWorking` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_a2a_tasks_start_working(value: &json::Value, at: &str) -> Result<crate::tasks::StartWorking, json::DecodeError> {
    Ok(crate::tasks::StartWorking {
        id: {
            let at0 = json::nested(at, "id");
            let member0 = json::member_at(value, at, "id")?;
            decode_a2a_tasks_task_id(member0, &at0)?
        },
        status_message: match value.member("status_message") {
            None | Some(json::Value::Null) => None,
            Some(member1) => {
                let at1 = json::nested(at, "status_message");
                Some(decode_a2a_tasks_message(member1, &at1)?)
            }
        },
    })
}

/// Writes the outcome of `a2a.tasks.StartWorking` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_a2a_tasks_start_working(value: &crate::tasks::StartWorkingOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::tasks::StartWorkingOutcome::Moved { task_status_updated } => {
            json::member(out, "outcome");
            json::push_text(out, "moved");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "a2a.tasks.TaskStatusUpdated");
            json::member(out, "payload");
            encode_event_a2a_tasks_task_status_updated(task_status_updated, out);
            out.push('}');
            out.push(']');
        }
        crate::tasks::StartWorkingOutcome::NotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.TaskNotFound");
            json::member(out, "payload");
            encode_error_a2a_tasks_task_not_found(error, out);
            out.push('}');
        }
        crate::tasks::StartWorkingOutcome::WrongState { error } => {
            json::member(out, "outcome");
            json::push_text(out, "wrong-state");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.TaskStateConflict");
            json::member(out, "payload");
            encode_error_a2a_tasks_task_state_conflict(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `a2a.tasks.SubscribeToTask` as JSON.
pub fn encode_command_a2a_tasks_subscribe_to_task(value: &crate::tasks::SubscribeToTask, out: &mut String) {
    out.push('{');
    if let Some(held0) = &value.tenant {
        json::member(out, "tenant");
        json::push_text(out, &*held0);
    }
    json::member(out, "id");
    encode_a2a_tasks_task_id(&value.id, out);
    out.push('}');
}

/// Reads the input of `a2a.tasks.SubscribeToTask` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_a2a_tasks_subscribe_to_task(value: &json::Value, at: &str) -> Result<crate::tasks::SubscribeToTask, json::DecodeError> {
    Ok(crate::tasks::SubscribeToTask {
        tenant: match value.member("tenant") {
            None | Some(json::Value::Null) => None,
            Some(member0) => {
                let at0 = json::nested(at, "tenant");
                Some(json::text_at(member0, &at0, "a string")?.to_owned())
            }
        },
        id: {
            let at1 = json::nested(at, "id");
            let member1 = json::member_at(value, at, "id")?;
            decode_a2a_tasks_task_id(member1, &at1)?
        },
    })
}

/// Writes the response of `a2a.tasks.SubscribeToTask` as JSON.
pub fn encode_response_a2a_tasks_subscribe_to_task(value: &crate::tasks::SubscribeToTaskResponse, out: &mut String) {
    out.push('{');
    json::member(out, "task");
    encode_a2a_tasks_task_snapshot(&value.task, out);
    out.push('}');
}

/// Writes the outcome of `a2a.tasks.SubscribeToTask` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_a2a_tasks_subscribe_to_task(value: &crate::tasks::SubscribeToTaskOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::tasks::SubscribeToTaskOutcome::NotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.TaskNotFound");
            json::member(out, "payload");
            encode_error_a2a_tasks_task_not_found(error, out);
            out.push('}');
        }
        crate::tasks::SubscribeToTaskOutcome::TaskTerminal { error } => {
            json::member(out, "outcome");
            json::push_text(out, "task-terminal");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.UnsupportedOperation");
            json::member(out, "payload");
            encode_error_a2a_tasks_unsupported_operation(error, out);
            out.push('}');
        }
        crate::tasks::SubscribeToTaskOutcome::Subscribed { .. } => {
            json::member(out, "outcome");
            json::push_text(out, "subscribed");
            json::member(out, "published");
            out.push('[');
            out.push(']');
        }
        crate::tasks::SubscribeToTaskOutcome::VersionNotSupported { error } => {
            json::member(out, "outcome");
            json::push_text(out, "version-not-supported");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.VersionNotSupported");
            json::member(out, "payload");
            encode_error_a2a_tasks_version_not_supported(error, out);
            out.push('}');
        }
        crate::tasks::SubscribeToTaskOutcome::ExtensionRequired { error } => {
            json::member(out, "outcome");
            json::push_text(out, "extension-required");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.ExtensionSupportRequired");
            json::member(out, "payload");
            encode_error_a2a_tasks_extension_support_required(error, out);
            out.push('}');
        }
        crate::tasks::SubscribeToTaskOutcome::StreamingNotSupported { error } => {
            json::member(out, "outcome");
            json::push_text(out, "streaming-not-supported");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.UnsupportedOperation");
            json::member(out, "payload");
            encode_error_a2a_tasks_unsupported_operation(error, out);
            out.push('}');
        }
    }
    out.push('}');
}

/// Writes the input of `a2a.tasks.UpdateArtifact` as JSON.
pub fn encode_command_a2a_tasks_update_artifact(value: &crate::tasks::UpdateArtifact, out: &mut String) {
    out.push('{');
    json::member(out, "id");
    encode_a2a_tasks_task_id(&value.id, out);
    json::member(out, "artifact");
    encode_a2a_tasks_artifact(&value.artifact, out);
    json::member(out, "append");
    json::push_bool(out, value.append);
    json::member(out, "last_chunk");
    json::push_bool(out, value.last_chunk);
    out.push('}');
}

/// Reads the input of `a2a.tasks.UpdateArtifact` from JSON.
///
/// # Errors
///
/// [`json::DecodeError`] naming the path and what the declaration says belongs there.
pub fn decode_command_a2a_tasks_update_artifact(value: &json::Value, at: &str) -> Result<crate::tasks::UpdateArtifact, json::DecodeError> {
    Ok(crate::tasks::UpdateArtifact {
        id: {
            let at0 = json::nested(at, "id");
            let member0 = json::member_at(value, at, "id")?;
            decode_a2a_tasks_task_id(member0, &at0)?
        },
        artifact: {
            let at1 = json::nested(at, "artifact");
            let member1 = json::member_at(value, at, "artifact")?;
            decode_a2a_tasks_artifact(member1, &at1)?
        },
        append: {
            let at2 = json::nested(at, "append");
            let member2 = json::member_at(value, at, "append")?;
            json::bool_at(member2, &at2, "a boolean")?
        },
        last_chunk: {
            let at3 = json::nested(at, "last_chunk");
            let member3 = json::member_at(value, at, "last_chunk")?;
            json::bool_at(member3, &at3, "a boolean")?
        },
    })
}

/// Writes the outcome of `a2a.tasks.UpdateArtifact` as JSON: the branch taken, what it published, and the declared
/// refusal it carries where it carries one.
pub fn encode_outcome_a2a_tasks_update_artifact(value: &crate::tasks::UpdateArtifactOutcome, out: &mut String) {
    out.push('{');
    match value {
        crate::tasks::UpdateArtifactOutcome::NotFound { error } => {
            json::member(out, "outcome");
            json::push_text(out, "not-found");
            json::member(out, "published");
            out.push('[');
            out.push(']');
            json::member(out, "refusal");
            out.push('{');
            json::member(out, "error");
            json::push_text(out, "a2a.tasks.TaskNotFound");
            json::member(out, "payload");
            encode_error_a2a_tasks_task_not_found(error, out);
            out.push('}');
        }
        crate::tasks::UpdateArtifactOutcome::Updated { task_artifact_updated } => {
            json::member(out, "outcome");
            json::push_text(out, "updated");
            json::member(out, "published");
            out.push('[');
            out.push('{');
            json::member(out, "event");
            json::push_text(out, "a2a.tasks.TaskArtifactUpdated");
            json::member(out, "payload");
            encode_event_a2a_tasks_task_artifact_updated(task_artifact_updated, out);
            out.push('}');
            out.push(']');
        }
    }
    out.push('}');
}

/// Writes any event on the system's log as JSON: its qualified name and its payload,
/// `{"event": …, "payload": {…}}`, the envelope a command's answer lists it in.
pub fn encode_system_event(value: &crate::system::SystemEvent) -> String {
    let mut out = String::from("{");
    json::member(&mut out, "event");
    json::push_text(&mut out, value.name());
    json::member(&mut out, "payload");
    match value {
        crate::system::SystemEvent::PushNotificationConfigCreated(event) => encode_event_a2a_tasks_push_notification_config_created(event, &mut out),
        crate::system::SystemEvent::PushNotificationConfigDeleted(event) => encode_event_a2a_tasks_push_notification_config_deleted(event, &mut out),
        crate::system::SystemEvent::TaskArtifactUpdated(event) => encode_event_a2a_tasks_task_artifact_updated(event, &mut out),
        crate::system::SystemEvent::TaskCreated(event) => encode_event_a2a_tasks_task_created(event, &mut out),
        crate::system::SystemEvent::TaskStatusUpdated(event) => encode_event_a2a_tasks_task_status_updated(event, &mut out),
    }
    out.push('}');
    out
}
