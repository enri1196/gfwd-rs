//! Synchronous feature-update output.

/// Work emitted by a feature reducer.
///
/// Reducers collect root-coordination requests separately from asynchronous
/// effects so the root router can apply every request before any effect starts.
#[derive(Debug)]
pub(crate) struct Outcome<Effect, Request> {
    /// Feature-owned asynchronous work.
    pub(crate) effects: Vec<Effect>,
    /// Root-owned coordination work, in emission order.
    pub(crate) requests: Vec<Request>,
}

impl<Effect, Request> Default for Outcome<Effect, Request> {
    fn default() -> Self {
        Self {
            effects: Vec::new(),
            requests: Vec::new(),
        }
    }
}

impl<Effect, Request> Outcome<Effect, Request> {
    /// Return an outcome containing one asynchronous effect.
    pub(crate) fn effect(effect: Effect) -> Self {
        Self {
            effects: vec![effect],
            requests: Vec::new(),
        }
    }

    /// Return an outcome containing one root request.
    pub(crate) fn request(request: Request) -> Self {
        Self {
            effects: Vec::new(),
            requests: vec![request],
        }
    }

    /// Append another outcome while preserving emission order.
    pub(crate) fn append(&mut self, mut other: Self) {
        self.effects.append(&mut other.effects);
        self.requests.append(&mut other.requests);
    }
}

#[cfg(test)]
mod tests {
    use super::Outcome;

    #[test]
    fn append_preserves_effect_and_request_order() {
        let mut outcome = Outcome::request("first request");
        outcome.append(Outcome::effect("first effect"));
        outcome.append(Outcome {
            effects: vec!["second effect"],
            requests: vec!["second request"],
        });

        assert_eq!(outcome.requests, ["first request", "second request"]);
        assert_eq!(outcome.effects, ["first effect", "second effect"]);
    }
}
