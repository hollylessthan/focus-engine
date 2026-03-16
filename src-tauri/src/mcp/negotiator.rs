/// Priority Buffer Negotiator — stub.
///
/// Intercepts incoming MCP messages and places them in a priority queue.
/// The queue is surfaced to the user at the next natural breakpoint.
///
/// Auto-response template:
///   "User is in a {load}%-complexity task. Est. refocus cost: {minutes} min.
///    Your message has been queued for their next downtime block."
pub struct Negotiator;

impl Negotiator {
    pub fn new() -> Self {
        Negotiator
    }

    /// Queue an incoming interruption for later review.
    pub fn enqueue(&self, _source: &str, _preview: &str, _priority: u8) -> Result<(), String> {
        // TODO: insert into DB interruptions table
        Ok(())
    }

    /// Generate the auto-response message body.
    pub fn compose_response(&self, cognitive_load_score: f32) -> String {
        let pct = (cognitive_load_score * 100.0).round() as u32;
        let minutes = (cognitive_load_score * 23.25).round() as u32;
        format!(
            "User is in a {pct}%-complexity task. \
             Estimated refocus cost: {minutes} min. \
             Your message has been queued in the Priority Buffer for their next downtime block."
        )
    }
}

impl Default for Negotiator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_contains_load_percentage() {
        let neg = Negotiator::new();
        let msg = neg.compose_response(0.72);
        assert!(msg.contains("72%"));
        assert!(msg.contains("min"));
    }
}
