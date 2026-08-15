//! Root request/effect ordering.

use std::collections::VecDeque;

use super::outcome::Outcome;

/// Collects slice output while the root processes coordination requests.
pub(crate) struct Router<Effect, Request> {
    effects: Vec<Effect>,
    requests: VecDeque<Request>,
}

impl<Effect, Request> Router<Effect, Request> {
    /// Start routing one reducer outcome.
    pub(crate) fn new(outcome: Outcome<Effect, Request>) -> Self {
        Self {
            effects: outcome.effects,
            requests: outcome.requests.into(),
        }
    }

    /// Return the next root request in FIFO order.
    pub(crate) fn pop_request(&mut self) -> Option<Request> {
        self.requests.pop_front()
    }

    /// Finish routing and return effects in collection order.
    pub(crate) fn into_effects(self) -> Vec<Effect> {
        debug_assert!(self.requests.is_empty());
        self.effects
    }
}

#[cfg(test)]
mod tests {
    use super::Router;
    use crate::app::{navigation, outcome::Outcome};

    #[test]
    fn requests_are_processed_fifo_before_effects_are_taken() {
        let mut router = Router::new(Outcome {
            effects: vec!["effect one"],
            requests: vec!["request one", "request two"],
        });
        assert_eq!(router.pop_request(), Some("request one"));
        assert_eq!(router.pop_request(), Some("request two"));
        assert_eq!(router.pop_request(), None);
        assert_eq!(router.into_effects(), ["effect one"]);
    }

    #[test]
    fn navigation_selection_work_reaches_root_in_causal_order() {
        let mut navigation = navigation::State::new();
        navigation.set_zones(vec!["public".to_string()]);
        let zone_id = navigation.zone_id("public").expect("zone is materialized");
        let outcome = navigation::update(
            &mut navigation,
            navigation::Message::Select(zone_id),
            navigation::Context,
        );
        let mut router = Router::new(outcome);

        assert_eq!(
            router.pop_request(),
            Some(navigation::Request::LoadZone("public".into()))
        );
        assert_eq!(
            router.pop_request(),
            Some(navigation::Request::RefreshTitle)
        );
        assert!(router.pop_request().is_none());
        assert!(router.into_effects().is_empty());
    }

    #[test]
    fn ipset_selection_reaches_root_before_title_refresh() {
        let mut navigation = navigation::State::new();
        let ipset_id = navigation.nav_model().active();
        let outcome = navigation::update(
            &mut navigation,
            navigation::Message::Select(ipset_id),
            navigation::Context,
        );
        let mut router = Router::new(outcome);

        assert_eq!(router.pop_request(), Some(navigation::Request::LoadIpSets));
        assert_eq!(
            router.pop_request(),
            Some(navigation::Request::RefreshTitle)
        );
        assert!(router.pop_request().is_none());
        assert!(router.into_effects().is_empty());
    }
}
