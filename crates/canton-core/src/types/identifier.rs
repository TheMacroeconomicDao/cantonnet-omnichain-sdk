//! Identifier types: PartyId, ContractId, Identifier.
//! See research/08 §3.3; Party ID format partyHint::fingerprint (research/06, 09).

use std::fmt;
use std::str::FromStr;
use thiserror::Error;

/// Template/type identifier (package:module.entity).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub package_id: String,
    pub module_name: String,
    pub entity_name: String,
}

impl Identifier {
    pub fn new(
        package_id: impl Into<String>,
        module_name: impl Into<String>,
        entity_name: impl Into<String>,
    ) -> Self {
        Self {
            package_id: package_id.into(),
            module_name: module_name.into(),
            entity_name: entity_name.into(),
        }
    }

    pub fn qualified_name(&self) -> String {
        format!("{}:{}.{}", self.package_id, self.module_name, self.entity_name)
    }
}

impl FromStr for Identifier {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(ParseError::InvalidFormat("expected package:module.entity".into()));
        }
        let package_id = parts[0];
        let module_entity: Vec<&str> = parts[1].rsplitn(2, '.').collect();
        if module_entity.len() != 2 {
            return Err(ParseError::InvalidFormat("expected module.entity".into()));
        }
        Ok(Self {
            package_id: package_id.to_string(),
            module_name: module_entity[1].to_string(),
            entity_name: module_entity[0].to_string(),
        })
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.qualified_name())
    }
}

/// Party identifier. Canton external party format: partyHint::fingerprint (research/06, 09).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PartyId(pub String);

impl PartyId {
    pub fn new(id: impl Into<String>) -> Result<Self, ValidationError> {
        let id = id.into();
        Self::validate(&id)?;
        Ok(Self(id))
    }

    pub fn new_unchecked(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    fn validate(id: &str) -> Result<(), ValidationError> {
        if id.is_empty() {
            return Err(ValidationError::Empty("party_id"));
        }
        if id.len() > 256 {
            return Err(ValidationError::TooLong("party_id", 256));
        }
        if !id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ':') {
            return Err(ValidationError::InvalidCharacters("party_id"));
        }
        Ok(())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PartyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for PartyId {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

/// Contract identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContractId(pub String);

impl ContractId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ContractId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ContractId {
    fn from(s: String) -> Self { Self(s) }
}
impl From<&str> for ContractId {
    fn from(s: &str) -> Self { Self(s.to_string()) }
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("{0} cannot be empty")]
    Empty(&'static str),
    #[error("{0} exceeds maximum length of {1}")]
    TooLong(&'static str, usize),
    #[error("{0} contains invalid characters")]
    InvalidCharacters(&'static str),
}
