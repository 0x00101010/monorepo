use crate::{
    minimmit::actors::voter::Config,
    simplex::{
        signing_scheme::Scheme,
        types::{Activity, Context},
    },
    Automaton, Relay, Reporter,
};
use commonware_cryptography::{Digest, PublicKey};
use commonware_p2p::{Blocker, Sender};
use commonware_runtime::{spawn_cell, Clock, ContextCell, Handle, Metrics, Spawner, Storage};
use rand::{CryptoRng, Rng};
use std::time::Duration;
use tracing::info;

/// Actor responsible for driving participation in the consensus protocol.
pub struct Actor<
    E: Clock + Rng + CryptoRng + Spawner + Storage + Metrics,
    P: PublicKey,
    S: Scheme<PublicKey = P>,
    B: Blocker<PublicKey = P>,
    D: Digest,
    A: Automaton<Digest = D, Context = Context<D, P>>,
    R: Relay,
    F: Reporter<Activity = Activity<S, D>>,
> {
    context: ContextCell<E>,

    #[allow(dead_code)]
    blocker: B,
    #[allow(dead_code)]
    automaton: A,
    #[allow(dead_code)]
    relay: R,
    #[allow(dead_code)]
    reporter: F,
}

impl<
        E: Clock + Rng + CryptoRng + Spawner + Storage + Metrics,
        P: PublicKey,
        S: Scheme<PublicKey = P>,
        B: Blocker<PublicKey = P>,
        D: Digest,
        A: Automaton<Digest = D, Context = Context<D, P>>,
        R: Relay<Digest = D>,
        F: Reporter<Activity = Activity<S, D>>,
    > Actor<E, P, S, B, D, A, R, F>
{
    pub fn new(context: E, cfg: Config<S, B, D, A, R, F>) -> Self {
        info!("Actor initialized");
        Self {
            context: ContextCell::new(context),
            blocker: cfg.blocker,
            automaton: cfg.automaton,
            relay: cfg.relay,
            reporter: cfg.reporter,
        }
    }

    pub fn start(mut self, sender: impl Sender<PublicKey = P>) -> Handle<()> {
        spawn_cell!(self.context, self.run(sender).await,)
    }

    async fn run(self, _sender: impl Sender<PublicKey = P>) {
        loop {
            info!("Voter running...");
            self.context.sleep(Duration::from_secs(1)).await;
        }
    }
}
