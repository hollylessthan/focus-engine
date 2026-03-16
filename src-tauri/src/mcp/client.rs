/// MCP Client — stub.
///
/// Focus Engine acts as an MCP client that connects to locally running
/// MCP servers (Slack, Jira, Email). All connections are to 127.0.0.1 only.
///
/// Zero-Knowledge constraint: screen OCR content NEVER leaves this process.
/// Only `cognitive_load_score` (a float) is included in auto-responses.
pub struct McpClient {
    pub endpoint: String,
}

impl McpClient {
    pub fn new(endpoint: impl Into<String>) -> Self {
        McpClient { endpoint: endpoint.into() }
    }

    /// Send an auto-response indicating the user is in a focus session.
    /// Only exposes the cognitive_load_score — never the raw OCR context.
    pub fn send_focus_response(
        &self,
        _cognitive_load_score: f32,
        _refocus_minutes: u32,
    ) -> Result<(), String> {
        // TODO: implement MCP protocol message send over local socket
        Ok(())
    }
}
