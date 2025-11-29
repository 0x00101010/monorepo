pub mod actor;

use crate::{
    simplex::{signing_scheme::Scheme, types::Activity},
    types::Epoch,
    Automaton, Relay, Reporter,
};
use commonware_cryptography::Digest;
use commonware_p2p::Blocker;
use std::time::Duration;

pub struct Config<
    S: Scheme,
    B: Blocker,
    D: Digest,
    A: Automaton,
    R: Relay<Digest = D>,
    F: Reporter<Activity = Activity<S, D>>,
> {
    pub scheme: S,
    pub blocker: B,
    pub automaton: A,
    pub relay: R,
    pub reporter: F,

    pub epoch: Epoch,
    pub namespace: Vec<u8>,
    pub leader_timeout: Duration,
    pub notarization_timeout: Duration,
}
