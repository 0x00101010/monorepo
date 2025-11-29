pub mod actors;
pub mod config;
pub mod engine;
pub mod types;

#[cfg(test)]
mod mocks;

#[cfg(test)]
mod tests {
    use crate::{
        minimmit::{config, engine::Engine},
        simplex::{
            mocks::fixtures::{ed25519, Fixture},
            signing_scheme::Scheme,
        },
        types::Epoch,
    };
    use commonware_cryptography::{ed25519, PublicKey, Sha256};
    use commonware_macros::test_traced;
    use commonware_p2p::simulated::{Config, Link, Network, Oracle, Receiver, Sender};
    use commonware_runtime::{deterministic, Clock, Metrics, Runner};
    use std::{collections::HashMap, sync::Arc, time::Duration};

    /// Register a validator with the oracle.
    async fn register_validator<P: PublicKey>(
        oracle: &mut Oracle<P>,
        validator: P,
    ) -> (Sender<P>, Receiver<P>) {
        let mut control = oracle.control(validator.clone());
        control.register(0).await.unwrap()
    }

    /// Registers all validators using the oracle.
    async fn register_validators<P: PublicKey>(
        oracle: &mut Oracle<P>,
        validators: &[P],
    ) -> HashMap<P, (Sender<P>, Receiver<P>)> {
        let mut registrations = HashMap::new();
        for validator in validators.iter() {
            let registration = register_validator(oracle, validator.clone()).await;
            registrations.insert(validator.clone(), registration);
        }
        registrations
    }

    /// Enum to describe the action to take when linking validators.
    enum Action {
        Link(Link),
        #[allow(dead_code)]
        Update(Link), // Unlink and then link
        #[allow(dead_code)]
        Unlink,
    }

    /// Links (or unlinks) validators using the oracle.
    ///
    /// The `action` parameter determines the action (e.g. link, unlink) to take.
    /// The `restrict_to` function can be used to restrict the linking to certain connections,
    /// otherwise all validators will be linked to all other validators.
    async fn link_validators<P: PublicKey>(
        oracle: &mut Oracle<P>,
        validators: &[P],
        action: Action,
        restrict_to: Option<fn(usize, usize, usize) -> bool>,
    ) {
        for (i1, v1) in validators.iter().enumerate() {
            for (i2, v2) in validators.iter().enumerate() {
                // Ignore self
                if v2 == v1 {
                    continue;
                }

                // Restrict to certain connections
                if let Some(f) = restrict_to {
                    if !f(validators.len(), i1, i2) {
                        continue;
                    }
                }

                // Do any unlinking first
                match action {
                    Action::Update(_) | Action::Unlink => {
                        oracle.remove_link(v1.clone(), v2.clone()).await.unwrap();
                    }
                    _ => {}
                }

                // Do any linking after
                match action {
                    Action::Link(ref link) | Action::Update(ref link) => {
                        oracle
                            .add_link(v1.clone(), v2.clone(), link.clone())
                            .await
                            .unwrap();
                    }
                    _ => {}
                }
            }
        }
    }

    fn all_online<S, F>(mut fixture: F)
    where
        S: Scheme<PublicKey = ed25519::PublicKey>,
        F: FnMut(&mut deterministic::Context, u32) -> Fixture<S>,
    {
        // Create context
        let n = 6; // 5f + 1
        let _quorum = 5;
        let namespace = b"minimmit".to_vec();
        let executor = deterministic::Runner::timed(Duration::from_secs(30));
        executor.start(|mut context| async move {
            // Create network
            let (network, mut oracle) = Network::new(
                context.with_label("network"),
                Config {
                    max_size: 1024 * 1024,
                    disconnect_on_block: true,
                    tracked_peer_sets: None,
                },
            );
            network.start();

            // Register participants
            let Fixture {
                participants,
                schemes,
                ..
            } = fixture(&mut context, n);
            let mut registrations = register_validators(&mut oracle, &participants).await;

            // Link all validators
            let link = Link {
                latency: Duration::from_millis(100),
                jitter: Duration::from_millis(1),
                success_rate: 1.0,
            };
            link_validators(&mut oracle, &participants, Action::Link(link), None).await;

            // Create engines
            let relay = Arc::new(crate::simplex::mocks::relay::Relay::new());
            let mut reporters = Vec::new();
            let mut engine_handles = Vec::new();
            for (idx, validator) in participants.iter().enumerate() {
                // Create scheme context
                let context = context.with_label(&format!("validator-{}", idx));

                // configure actor
                let reporter_config = crate::simplex::mocks::reporter::Config {
                    namespace: namespace.clone(),
                    participants: participants.clone().into(),
                    scheme: schemes[idx].clone(),
                };
                let reporter = crate::simplex::mocks::reporter::Reporter::new(
                    context.with_label("reporter"),
                    reporter_config,
                );
                reporters.push(reporter.clone());

                let application_cfg = crate::simplex::mocks::application::Config {
                    hasher: Sha256::default(),
                    relay: relay.clone(),
                    me: validator.clone(),
                    // TODO: ??
                    propose_latency: (10.0, 5.0),
                    verify_latency: (10.0, 5.0),
                };
                let (actor, application) = crate::simplex::mocks::application::Application::new(
                    context.with_label("application"),
                    application_cfg,
                );
                actor.start();

                let blocker = oracle.control(validator.clone());
                let cfg = config::Config {
                    scheme: schemes[idx].clone(),
                    blocker,
                    automaton: application.clone(),
                    relay: application.clone(),
                    reporter: reporter.clone(),
                    epoch: Epoch::new(0),
                    namespace: namespace.clone(),
                    leader_timeout: Duration::from_secs(1),
                    notarization_timeout: Duration::from_secs(2),
                };
                let engine = Engine::new(context, cfg);
                let network = registrations.remove(validator).unwrap();
                let engine_handle = engine.start(network);

                engine_handles.push(engine_handle);
            }
            
            context.sleep(Duration::from_secs(10)).await;
        });
    }

    #[test_traced]
    fn test_all_online() {
        all_online(ed25519);
    }
}
