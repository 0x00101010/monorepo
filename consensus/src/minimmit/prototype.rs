use crate::{
    minimmit::types::{Proposal, Voter},
    types::Round,
};
use commonware_cryptography::{ed25519, Digest};
use commonware_runtime::{Clock, Spawner};
use futures::channel::mpsc;
use std::time::Duration;

pub struct Replica<D: Digest> {
    id: u32,
    keypair: ed25519::PrivateKey,

    // internal states to maintain
    round: Round,
    notarized: Option<Proposal<D>>,
    nullified: bool,
    timeout: Duration,
    messages: Vec<Voter<D>>,
    // proof:

    // external communication channels
    inbox: mpsc::UnboundedReceiver<Voter<D>>,
    peers: Vec<mpsc::UnboundedSender<Voter<D>>>,
}

impl<D> Replica<D>
where
    D: Digest,
{
    pub fn new(
        id: u32,
        keypair: ed25519::PrivateKey,
        timeout: Duration,
        inbox: mpsc::UnboundedReceiver<Voter<D>>,
        peers: Vec<mpsc::UnboundedSender<Voter<D>>>,
    ) -> Self {
        Self {
            id,
            keypair,
            round: Default::default(),
            notarized: None,
            nullified: false,
            timeout,
            messages: vec![],
            inbox,
            peers,
        }
    }

    pub async fn run(&self, context: impl Clock + Spawner) {
        println!("Replica {} started in round {}", self.id, self.round);

        loop {
            println!("Replica {} sleeping for {:?}...", self.id, self.timeout);
            context.sleep(self.timeout).await;
            println!("Replica {} woke up!", self.id);
        }
    }
}
