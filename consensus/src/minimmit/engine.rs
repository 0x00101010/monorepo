use super::actors::voter;
use crate::{
    minimmit::config::Config,
    simplex::{
        signing_scheme::Scheme,
        types::{Activity, Context},
    },
    Automaton, Relay, Reporter,
};
use commonware_cryptography::{Digest, PublicKey};
use commonware_macros::select;
use commonware_p2p::{Blocker, Receiver, Sender};
use commonware_runtime::{spawn_cell, Clock, ContextCell, Handle, Metrics, Spawner, Storage};
use governor::clock::Clock as GClock;
use rand::{CryptoRng, Rng};
use tracing::*;

/// Instance of `minimmit` consensus engine.
pub struct Engine<
    E: Clock + GClock + Rng + CryptoRng + Spawner + Storage + Metrics,
    P: PublicKey,
    S: Scheme<PublicKey = P>,
    B: Blocker<PublicKey = P>,
    D: Digest,
    A: Automaton<Context = Context<D, P>, Digest = D>,
    R: Relay<Digest = D>,
    F: Reporter<Activity = Activity<S, D>>,
> {
    context: ContextCell<E>,

    voter: voter::actor::Actor<E, P, S, B, D, A, R, F>,
}

impl<
        E: Clock + GClock + Rng + CryptoRng + Spawner + Storage + Metrics,
        P: PublicKey,
        S: Scheme<PublicKey = P>,
        B: Blocker<PublicKey = P>,
        D: Digest,
        A: Automaton<Context = Context<D, P>, Digest = D>,
        R: Relay<Digest = D>,
        F: Reporter<Activity = Activity<S, D>>,
    > Engine<E, P, S, B, D, A, R, F>
{
    pub fn new(context: E, cfg: Config<P, S, B, D, A, R, F>) -> Self {
        let voter = voter::actor::Actor::new(
            context.with_label("voter"),
            voter::Config {
                scheme: cfg.scheme.clone(),
                blocker: cfg.blocker.clone(),
                automaton: cfg.automaton,
                relay: cfg.relay,
                reporter: cfg.reporter,
                epoch: cfg.epoch,
                namespace: cfg.namespace.clone(),
                leader_timeout: cfg.leader_timeout,
                notarization_timeout: cfg.notarization_timeout,
            },
        );

        Self {
            context: ContextCell::new(context),
            voter,
        }
    }

    pub fn start(
        mut self,
        network: (impl Sender<PublicKey = P>, impl Receiver<PublicKey = P>),
    ) -> Handle<()> {
        spawn_cell!(self.context, self.run(network).await,)
    }

    async fn run(self, network: (impl Sender<PublicKey = P>, impl Receiver<PublicKey = P>)) {
        let (sender, _) = network;
        let mut voter_task = self.voter.start(sender);

        let mut shutdown = self.context.stopped();
        select! {
            _ = &mut shutdown => {
                debug!("context shutdown, stopping engine");
            },
            _ = &mut voter_task => {
                panic!("voter should not finish");
            }
        }
    }
}
