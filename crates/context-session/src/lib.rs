// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Process-local, consumer-scoped immutable context packet references."]

use std::{collections::BTreeMap, fmt};

use context_core::{ContextPacket, packet_bytes, validate_packet};
use serde::{Deserialize, Serialize};

const CONTRACT_VERSION: &str = "1.0.0";

/// Hard process-local session ceilings selected by the embedding consumer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionPolicy {
    /// Maximum simultaneously open sessions.
    pub max_sessions: usize,
    /// Maximum packet references per session.
    pub max_packets_per_session: usize,
    /// Maximum canonical packet bytes held per session.
    pub max_packet_bytes_per_session: usize,
}

impl SessionPolicy {
    /// Creates bounded session policy.
    ///
    /// # Errors
    ///
    /// Fails when a limit is zero or exceeds the conservative local ceiling.
    pub fn new(
        max_sessions: usize,
        max_packets_per_session: usize,
        max_packet_bytes_per_session: usize,
    ) -> Result<Self, SessionError> {
        if max_sessions == 0
            || max_sessions > 1024
            || max_packets_per_session == 0
            || max_packets_per_session > 10_000
            || max_packet_bytes_per_session == 0
            || max_packet_bytes_per_session > 256 * 1024 * 1024
        {
            return Err(SessionError::ResourceLimit);
        }
        Ok(Self {
            max_sessions,
            max_packets_per_session,
            max_packet_bytes_per_session,
        })
    }
}

/// Public immutable reference metadata; it grants no workspace authority.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SessionPacketReference {
    /// Schema discriminator.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Consumer-selected process-local session identifier.
    pub session_id: String,
    /// Exact packet identity.
    pub packet_id: String,
    /// Exact workspace identity represented by the packet.
    pub workspace_identity: String,
    /// Exact workspace snapshot represented by the packet.
    pub workspace_snapshot: String,
    /// Declared packet purpose.
    pub purpose: String,
    /// Canonical bytes retained in memory.
    pub packet_bytes: String,
    /// Always false: a reference carries no workspace or execution authority.
    pub authority_added: bool,
}

#[derive(Debug)]
struct Session {
    consumer_id: String,
    packet_bytes: usize,
    packets: BTreeMap<String, (SessionPacketReference, ContextPacket)>,
}

/// In-memory session store. Dropping the store closes every reference.
#[derive(Debug)]
pub struct SessionStore {
    policy: SessionPolicy,
    sessions: BTreeMap<String, Session>,
}

impl SessionStore {
    /// Creates an empty process-local store.
    #[must_use]
    pub const fn new(policy: SessionPolicy) -> Self {
        Self {
            policy,
            sessions: BTreeMap::new(),
        }
    }

    /// Opens one consumer-scoped session.
    ///
    /// # Errors
    ///
    /// Fails for malformed/duplicate identifiers or session limit exhaustion.
    pub fn open(&mut self, session_id: &str, consumer_id: &str) -> Result<(), SessionError> {
        if !valid_identifier(session_id) || !valid_identifier(consumer_id) {
            return Err(SessionError::InvalidInput);
        }
        if self.sessions.contains_key(session_id) {
            return Err(SessionError::AlreadyExists);
        }
        if self.sessions.len() >= self.policy.max_sessions {
            return Err(SessionError::ResourceLimit);
        }
        self.sessions.insert(
            session_id.into(),
            Session {
                consumer_id: consumer_id.into(),
                packet_bytes: 0,
                packets: BTreeMap::new(),
            },
        );
        Ok(())
    }

    /// Attaches one already-valid immutable packet to its exact consumer session.
    ///
    /// # Errors
    ///
    /// Fails for wrong consumer, invalid/corrupt packet, duplicate identity, or limits.
    pub fn attach(
        &mut self,
        session_id: &str,
        consumer_id: &str,
        packet: &ContextPacket,
    ) -> Result<SessionPacketReference, SessionError> {
        validate_packet(packet).map_err(|_| SessionError::IntegrityFailure)?;
        let bytes = packet_bytes(packet).map_err(|_| SessionError::IntegrityFailure)?;
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or(SessionError::NotFound)?;
        if session.consumer_id != consumer_id {
            return Err(SessionError::Denied);
        }
        if session.packets.contains_key(&packet.packet_id) {
            return Err(SessionError::AlreadyExists);
        }
        if session.packets.len() >= self.policy.max_packets_per_session
            || session.packet_bytes.saturating_add(bytes.len())
                > self.policy.max_packet_bytes_per_session
        {
            return Err(SessionError::ResourceLimit);
        }
        let reference = SessionPacketReference {
            schema_name: "session-packet-reference".into(),
            schema_version: CONTRACT_VERSION.into(),
            session_id: session_id.into(),
            packet_id: packet.packet_id.clone(),
            workspace_identity: packet.workspace_identity.clone(),
            workspace_snapshot: packet.workspace_snapshot.clone(),
            purpose: packet.purpose.clone(),
            packet_bytes: bytes.len().to_string(),
            authority_added: false,
        };
        session.packet_bytes += bytes.len();
        session.packets.insert(
            packet.packet_id.clone(),
            (reference.clone(), packet.clone()),
        );
        Ok(reference)
    }

    /// Resolves one packet only for the consumer that owns the session.
    ///
    /// # Errors
    ///
    /// Fails without disclosure for missing sessions/references or wrong consumers.
    pub fn resolve(
        &self,
        session_id: &str,
        consumer_id: &str,
        packet_id: &str,
    ) -> Result<(&SessionPacketReference, &ContextPacket), SessionError> {
        let session = self
            .sessions
            .get(session_id)
            .ok_or(SessionError::NotFound)?;
        if session.consumer_id != consumer_id {
            return Err(SessionError::Denied);
        }
        let (reference, packet) = session
            .packets
            .get(packet_id)
            .ok_or(SessionError::NotFound)?;
        validate_packet(packet).map_err(|_| SessionError::IntegrityFailure)?;
        Ok((reference, packet))
    }

    /// Closes one exact session and invalidates all of its references.
    ///
    /// # Errors
    ///
    /// Fails for a missing session or wrong consumer.
    pub fn close(&mut self, session_id: &str, consumer_id: &str) -> Result<(), SessionError> {
        let session = self
            .sessions
            .get(session_id)
            .ok_or(SessionError::NotFound)?;
        if session.consumer_id != consumer_id {
            return Err(SessionError::Denied);
        }
        self.sessions.remove(session_id);
        Ok(())
    }
}

fn valid_identifier(value: &str) -> bool {
    (8..=128).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

/// Stable process-local session failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionError {
    /// An identifier or policy value is malformed.
    InvalidInput,
    /// The session or reference already exists.
    AlreadyExists,
    /// The session or reference is unavailable.
    NotFound,
    /// The consumer does not own this session.
    Denied,
    /// A hard session ceiling was reached.
    ResourceLimit,
    /// Packet integrity validation failed.
    IntegrityFailure,
}

impl fmt::Display for SessionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidInput => "invalid session input",
            Self::AlreadyExists => "session object already exists",
            Self::NotFound => "session object is unavailable",
            Self::Denied => "session access denied",
            Self::ResourceLimit => "session resource limit exceeded",
            Self::IntegrityFailure => "session packet integrity failed",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use context_core::{PacketDraft, PolicySubject, ResourceBudget, build_packet, decide};

    fn packet() -> ContextPacket {
        let workspace = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let snapshot = "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        let subject = PolicySubject {
            caller_id: "consumer_12345678".into(),
            role: "client".into(),
            purpose: "handoff".into(),
        };
        let budget = ResourceBudget::conservative(4096, 10, 10, 128, 100, 32, 30_000, 1_048_576)
            .expect("budget");
        let decision = decide(
            "req_decision_session01",
            &subject,
            Some(workspace),
            context_core::Capability::ContextBuild,
            Some(budget.clone()),
            "2026-08-22T00:00:00Z",
        )
        .expect("decision");
        build_packet(PacketDraft {
            workspace_identity: workspace.into(),
            workspace_snapshot: snapshot.into(),
            request_id: "req_session01".into(),
            purpose: "handoff".into(),
            created_at: "2026-08-22T00:00:00Z".into(),
            policy_decision: decision.decision_id,
            budget,
            evidence: Vec::new(),
            assumptions: Vec::new(),
            conflicts: Vec::new(),
            unknowns: Vec::new(),
            redactions: Vec::new(),
        })
        .expect("packet")
    }

    #[test]
    fn references_are_consumer_scoped_immutable_and_close_with_session() {
        let policy = SessionPolicy::new(2, 2, 65_536).expect("policy");
        let mut store = SessionStore::new(policy);
        store
            .open("session_alpha01", "consumer_alpha01")
            .expect("open");
        let packet = packet();
        let reference = store
            .attach("session_alpha01", "consumer_alpha01", &packet)
            .expect("attach");
        assert_eq!(reference.packet_id, packet.packet_id);
        assert!(!reference.authority_added);
        assert_eq!(
            store
                .resolve("session_alpha01", "consumer_alpha01", &packet.packet_id)
                .expect("resolve")
                .1,
            &packet
        );
        assert_eq!(
            store.resolve("session_alpha01", "consumer_other01", &packet.packet_id),
            Err(SessionError::Denied)
        );
        store
            .close("session_alpha01", "consumer_alpha01")
            .expect("close");
        assert_eq!(
            store.resolve("session_alpha01", "consumer_alpha01", &packet.packet_id),
            Err(SessionError::NotFound)
        );
    }
}
