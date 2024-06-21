//! Toplevel errors raised by the light client.

use std::{fmt::Debug, time::Duration};

use crossbeam_channel as crossbeam;
use flex_error::{define_error, DisplayError, TraceError};

// Re-export for backward compatibility
pub use crate::verifier::errors::ErrorExt;
use crate::{
    components::io::IoError,
    verifier::{
        errors::VerificationErrorDetail,
        operations::voting_power::VotingPowerTally,
        options::Options,
        types::{Hash, Height, LightBlock, PeerId, Status},
    },
};

#[cfg(feature = "sled")]
type SledError = TraceError<sled::Error>;

#[cfg(not(feature = "sled"))]
type SledError = flex_error::NoSource;

define_error! {
    #[derive(Debug)]
    Error {
        Io
            [ IoError ]
            | _ | { "io error" },

        NoPrimary
            | _ | { "no primary" },

        NoWitnesses
            | _ | { "no witnesses" },

        NoWitnessesLeft
            | _ | { "no witnesses left" },

        ForkDetected
            { peers: Vec<PeerId> }
            | e | {
                format_args!("fork detected peers={0:?}",
                    e.peers)
            },

        NoInitialTrustedState
            | _ | { "no initial trusted state" },

        NoTrustedState
            { status: Status }
            | e | {
                format_args!("no trusted state with status {:?}",
                    e.status)
            },

        TargetLowerThanTrustedState
            {
                target_height: Height,
                trusted_height: Height,
            }
            | e | {
                format_args!("target height ({0}) is lower than trusted state ({1})",
                    e.target_height, e.trusted_height)
            },

        HeightTooHigh
            {
                height: Height,
                latest_height: Height,
            }
            |e| {
                format_args!("height ({0}) is higher than latest height ({1})",
                    e.height, e.latest_height)
            },

        TrustedStateOutsideTrustingPeriod
            {
                trusted_state: Box<LightBlock>,
                options: Options,
            }
            | _ | {
                format_args!("trusted state outside of trusting period")
            },

        BisectionFailed
            {
                target_height: Height,
                trusted_height: Height
            }
            | e | {
                format_args!("bisection for target at height {0} failed when reached trusted state at height {1}",
                    e.target_height, e.trusted_height)
            },

        InvalidLightBlock
            [ DisplayError<VerificationErrorDetail> ]
            | _ | { "invalid light block" },

        InvalidAdjacentHeaders
            {
                hash1: Hash,
                hash2: Hash,
            }
            | e | {
                format_args!("hash mismatch between two adjacent headers: {0} != {1}",
                    e.hash1, e.hash2)
            },

        MissingLastBlockId
            { height: Height }
            | e | {
                format_args!("missing last_block_id for header at height {0}",
                    e.height)
            },

        ChannelDisconnected
            | _ | { "internal channel disconnected" },

        Sled
            [ SledError ]
            | _ | { "sled error" },

        SerdeCbor
            [ TraceError<serde_cbor::Error> ]
            | _ | { "serde cbor error" },

    }
}

impl ErrorExt for ErrorDetail {
    fn not_enough_trust(&self) -> Option<VotingPowerTally> {
        if let Self::InvalidLightBlock(e) = self {
            e.source.not_enough_trust()
        } else {
            None
        }
    }

    fn has_expired(&self) -> bool {
        if let Self::InvalidLightBlock(e) = self {
            e.source.has_expired()
        } else {
            false
        }
    }

    /// Whether this error means that a timeout occurred when querying a node.
    fn is_timeout(&self) -> Option<Duration> {
        if let Self::Io(e) = self {
            e.source.is_timeout()
        } else {
            None
        }
    }

    fn is_io(&self) -> bool {
        matches!(self, Self::Io(_))
    }

    fn is_height_too_high(&self) -> bool {
        matches!(self, Self::HeightTooHigh { .. })
    }
}

impl Error {
    pub fn send<T>(_e: crossbeam::SendError<T>) -> Error {
        Error::channel_disconnected()
    }

    pub fn recv(_e: crossbeam::RecvError) -> Error {
        Error::channel_disconnected()
    }
}
