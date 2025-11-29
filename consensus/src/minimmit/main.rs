use clap::Parser;
use commonware_consensus::minimmit::{prototype::Replica, types::Voter};
use commonware_cryptography::{ed25519, sha256, PrivateKeyExt};
use commonware_runtime::{tokio as runtime, Runner};
use futures::channel::mpsc;
use futures::future;
use std::time::Duration;
use commonware_runtime::Spawner;

#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value = "6")]
    replicas: u32,
    #[arg(short, long, default_value = "1000")]
    timeout_ms: u64,
}

pub fn main() {
    let args = Args::parse();
    let n = args.replicas;
    let timeout = Duration::from_millis(args.timeout_ms);

    println!(
        "Setting up {} replicas with {}ms timeout",
        n, args.timeout_ms
    );

    let executor = runtime::Runner::default();
    executor.start(|context| async move {
        // Generate keypairs for each replica
        let mut rng = rand::thread_rng();
        let keypairs: Vec<ed25519::PrivateKey> = (0..n)
            .map(|_| ed25519::PrivateKey::from_rng(&mut rng))
            .collect();

        // Create communication channels - one inbox per replica
        let mut all_senders = vec![];
        let mut all_receivers = vec![];

        for _ in 0..n {
            let (sender, receiver) = mpsc::unbounded::<Voter<sha256::Digest>>();
            all_senders.push(sender);
            all_receivers.push(receiver);
        }

        // Create and start each replica
        for i in 0..n {
            let replica_id = i;
            let keypair = keypairs[i as usize].clone();
            let inbox = all_receivers.remove(0);

            // Give this replica senders to all other replicas
            let mut peers = Vec::new();
            for j in 0..n {
                if i != j {
                    peers.push(all_senders[j as usize].clone());
                }
            }

            // Spawn replica actor
            context.clone().spawn(move |ctx| async move {
                let replica = Replica::new(replica_id, keypair, timeout, inbox, peers);
                replica.run(ctx).await;
            });
        }

        // Wait indefinitely (Ctrl+C will kill the process)
        println!("Replicas running. Press Ctrl+C to terminate...");
        future::pending::<()>().await;
    });
}
