use crate::{
    simplex::{
        signing_scheme::Scheme,
        types::{Activity, Context},
    },
    types::Epoch,
    Automaton, Relay, Reporter,
};
use commonware_cryptography::{Digest, PublicKey};
use commonware_p2p::Blocker;
use std::time::Duration;

pub struct Config<
    P: PublicKey,
    S: Scheme,
    B: Blocker<PublicKey = P>,
    D: Digest,
    A: Automaton<Context = Context<D, P>>,
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

// TODO(0x00101010): Add configuration validation functions here.
