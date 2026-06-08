mod cdp;
mod mcp;
mod tools;

use std::env;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_ansi(false)
        .init();

    info!("slack_mcp server starting");

    let cdp_port: u16 = env::var("SLACK_CDP_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(9222);

    let hint = if cdp_port == 9222 {
        "   open -a Slack --args --remote-debugging-port=9222".to_string()
    } else {
        format!("   open -a Slack --args --remote-debugging-port={}", cdp_port)
    };

    let automation = cdp::CdpAutomation::new(cdp_port).await.map_err(|e| {
        anyhow::anyhow!(
            "Cannot connect to Slack via CDP on port {}.\n\
             Error: {}\n\n\
             Make sure Slack is running with the remote debugging flag:\n\
             {}\n\n\
             Or set SLACK_CDP_PORT to a different port.",
            cdp_port, e, hint
        )
    })?;

    info!("Connected to Slack via CDP on port {}", cdp_port);

    let server = mcp::McpServer::new(Box::new(automation));
    server.run().await
}
