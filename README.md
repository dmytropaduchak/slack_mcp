# 🇺🇦 HELP UKRAINE

We fight for democratic values, for freedom, for our future. We need your support. 
Solidarity with the Ukrainian people against the Russian invasion [Find out how you can help.](https://war.ukraine.ua/support-ukraine/).


# SLACK MCP

MCP server that reads Slack data via CDP and the Slack Web API.

## Setup

```bash
# 1. Start Slack with remote debugging (one-time)
open -a Slack --args --remote-debugging-port=9222

# 2. Run
cargo run

# 3. Configure your MCP client
```

```json
{
  "slack_mcp": {
    "command": "<PATCH>/slack_mcp/target/debug/slack_mcp"
  }
}
```

## Tools

| Tool | Description |
|---|---|
| `slack_mcp_messages` | Read messages from a channel |
| `slack_mcp_channels` | List visible channels and DMs from the sidebar |
| `slack_mcp_channel` | Get channel details: name, topic, purpose, members |
| `slack_mcp_search` | Search messages across the workspace |
| `slack_mcp_profile` | Get user profile info |
| `slack_mcp_unread` | List channels with unread badges |
| `slack_mcp_send` | Send a message to a channel |
| `slack_mcp_reply` | Reply to a message thread |
| `slack_mcp_edit` | Edit an existing message |
| `slack_mcp_delete` | Delete a message |
| `slack_mcp_eval` | Run raw JavaScript in Slack's page context |

## Environment

| Variable | Default | Description |
|---|---|---|
| `SLACK_CDP_PORT` | `9222` | Slack DevTools debugging port |

> made by [OPENCODE](https://opencode.ai/go?ref=D0438CSYT3)
