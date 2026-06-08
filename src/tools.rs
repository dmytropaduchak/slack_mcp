use serde_json::Value;
use tracing::info;

use crate::cdp::SlackAutomation;

pub struct ToolDefinition {
    pub name: &'static str,
    pub description: &'static str,
    pub input_schema: Value,
}

pub fn tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "slack_mcp_messages",
            description: "Read messages from a Slack channel.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "channel_id": {
                        "type": "string",
                        "description": "Channel ID (e.g. C017ZMHM4GY). Use slack_mcp_channels to find IDs."
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Max messages (default 20)"
                    }
                },
                "required": ["channel_id"]
            }),
        },
        ToolDefinition {
            name: "slack_mcp_channels",
            description: "List visible channels and DMs from the sidebar with their IDs and unread status.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "slack_mcp_channel",
            description: "Get channel details: name, topic, purpose, member count.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "channel_id": {
                        "type": "string",
                        "description": "Channel ID (e.g. C017ZMHM4GY)"
                    }
                },
                "required": ["channel_id"]
            }),
        },
        ToolDefinition {
            name: "slack_mcp_search",
            description: "Search messages across the workspace by keyword, user, or channel.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query (supports Slack search syntax like from:@user, in:channel)"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Max results (default 10)"
                    }
                },
                "required": ["query"]
            }),
        },
        ToolDefinition {
            name: "slack_mcp_profile",
            description: "Get user profile info (name, display name, email, title).",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "user_id": {
                        "type": "string",
                        "description": "User ID (e.g. U03V9E14Y2K)"
                    }
                },
                "required": ["user_id"]
            }),
        },
        ToolDefinition {
            name: "slack_mcp_unread",
            description: "List channels with unread badges from the sidebar.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "slack_mcp_send",
            description: "Send a message to a Slack channel.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "channel_id": {
                        "type": "string",
                        "description": "Channel ID (e.g. C017ZMHM4GY)"
                    },
                    "text": {
                        "type": "string",
                        "description": "Message text"
                    }
                },
                "required": ["channel_id", "text"]
            }),
        },
        ToolDefinition {
            name: "slack_mcp_reply",
            description: "Reply to a message thread.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "channel_id": {
                        "type": "string",
                        "description": "Channel ID (e.g. C017ZMHM4GY)"
                    },
                    "thread_ts": {
                        "type": "string",
                        "description": "Parent message timestamp (ts) to reply in thread"
                    },
                    "text": {
                        "type": "string",
                        "description": "Reply text"
                    }
                },
                "required": ["channel_id", "thread_ts", "text"]
            }),
        },
        ToolDefinition {
            name: "slack_mcp_edit",
            description: "Edit an existing message.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "channel_id": {
                        "type": "string",
                        "description": "Channel ID (e.g. C017ZMHM4GY)"
                    },
                    "ts": {
                        "type": "string",
                        "description": "Message timestamp (ts) to edit"
                    },
                    "text": {
                        "type": "string",
                        "description": "New message text"
                    }
                },
                "required": ["channel_id", "ts", "text"]
            }),
        },
        ToolDefinition {
            name: "slack_mcp_delete",
            description: "Delete a message.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "channel_id": {
                        "type": "string",
                        "description": "Channel ID (e.g. C017ZMHM4GY)"
                    },
                    "ts": {
                        "type": "string",
                        "description": "Message timestamp (ts) to delete"
                    }
                },
                "required": ["channel_id", "ts"]
            }),
        },
        ToolDefinition {
            name: "slack_mcp_eval",
            description: "Run raw JavaScript in Slack's page context.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "js": {
                        "type": "string",
                        "description": "JavaScript expression"
                    }
                },
                "required": ["js"]
            }),
        },
    ]
}

pub enum ToolResult {
    Success(Value),
    Error(String),
}

impl ToolResult {
    pub fn into_content(self) -> Value {
        match self {
            ToolResult::Success(val) => val,
            ToolResult::Error(msg) => serde_json::json!({"ok": false, "message": msg}),
        }
    }

    pub fn is_error(&self) -> bool {
        matches!(self, ToolResult::Error(_))
    }
}

pub async fn handle_tool_call(
    tool_name: &str,
    args: &Value,
    automation: &dyn SlackAutomation,
) -> ToolResult {
    match tool_name {
        "slack_mcp_messages" => handle_slack_get_messages(args, automation).await,
        "slack_mcp_channels" => handle_slack_get_channels(automation).await,
        "slack_mcp_channel" => handle_slack_get_channel_info(args, automation).await,
        "slack_mcp_search" => handle_slack_search(args, automation).await,
        "slack_mcp_profile" => handle_slack_get_profile(args, automation).await,
        "slack_mcp_unread" => handle_slack_get_unread(automation).await,
        "slack_mcp_send" => handle_slack_send(args, automation).await,
        "slack_mcp_reply" => handle_slack_reply(args, automation).await,
        "slack_mcp_edit" => handle_slack_edit(args, automation).await,
        "slack_mcp_delete" => handle_slack_delete(args, automation).await,
        "slack_mcp_eval" => handle_slack_eval(args, automation).await,
        _ => ToolResult::Error(format!("Unknown tool: {}", tool_name)),
    }
}

fn get_int_arg(args: &Value, key: &str, default: u32) -> u32 {
    args.get(key).and_then(|v| v.as_u64()).unwrap_or(default as u64) as u32
}

fn get_string_arg(args: &Value, key: &str) -> Result<String, String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| format!("Missing or invalid argument: {}", key))
}

fn escape_js_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn truncate_preview(text: &str, max_len: usize) -> String {
    if text.len() > max_len {
        format!("{}... (truncated)", &text[..max_len])
    } else {
        text.to_string()
    }
}

async fn run_js(automation: &dyn SlackAutomation, js: &str) -> Result<String, String> {
    let result = automation.devtools_eval(js).await.map_err(|e| {
        e.to_string()
    })?;
    Ok(result)
}

async fn handle_slack_get_channels(
    automation: &dyn SlackAutomation,
) -> ToolResult {
    info!("slack_get_channels");
    let js = r#"JSON.stringify(Array.from(document.querySelectorAll('[data-qa-channel-sidebar-channel]')).map(el => ({
        name: el.innerText.trim().split(/\n/)[0],
        id: el.getAttribute('data-qa-channel-sidebar-channel-id') || '',
        unread: el.querySelector('[data-qa-unread-indicator]') !== null,
        type: el.getAttribute('data-qa-channel-sidebar-channel-type') || 'channel'
    })).filter(c => c.name && c.id))"#;

    match run_js(automation, js).await {
        Ok(raw) => {
            let channels: Value = serde_json::from_str(&raw).unwrap_or(Value::String(raw));
            ToolResult::Success(serde_json::json!({
                "ok": true, "channels": channels,
                "count": channels.as_array().map(|a| a.len()).unwrap_or(0)
            }))
        }
        Err(e) => ToolResult::Error(e),
    }
}

async fn handle_slack_get_messages(
    args: &Value,
    automation: &dyn SlackAutomation,
) -> ToolResult {
    let channel_id = match get_string_arg(args, "channel_id") {
        Ok(c) => c,
        Err(e) => return ToolResult::Error(e),
    };
    let limit = get_int_arg(args, "limit", 20);
    info!("slack_get_messages: channel={}, limit={}", channel_id, limit);

    let js = format!(
        r#"(async () => {{
            const cfg = JSON.parse(localStorage.getItem('localConfig_v2') || '{{}}');
            const team = Object.values(cfg.teams || {{}})[0];
            if (!team) return JSON.stringify({{error: 'no team config'}});

            const infoRes = await fetch('/api/conversations.info?token=' + encodeURIComponent(team.token) + '&channel={channel}');
            const info = await infoRes.json();
            const channelName = info.channel?.name || '{channel}';

            const r = await fetch('/api/conversations.history?token=' + encodeURIComponent(team.token) + '&channel={channel}&limit={limit}&pretty=1');
            const data = await r.json();
            if (!data.ok) return JSON.stringify({{error: data.error}});

            const userIds = [...new Set(data.messages.filter(m => m.user).map(m => m.user))];
            const users = {{}};
            for (const uid of userIds) {{
                try {{
                    const ur = await fetch('/api/users.info?token=' + encodeURIComponent(team.token) + '&user=' + uid);
                    const ud = await ur.json();
                    if (ud.ok) users[uid] = ud.user.real_name || ud.user.name;
                }} catch(e) {{}}
            }}

            const parts = ['Channel: #' + channelName];
            for (const m of data.messages) {{
                const user = users[m.user] || m.user || '?';
                const d = new Date(parseFloat(m.ts) * 1000);
                const dateStr = d.toLocaleDateString('en-US', {{month:'long', day:'numeric', year:'numeric'}});
                const text = m.text || '';
                const quoted = text.split('\\n').map(l => '> ' + l).join('\\n');
                parts.push(user + ' (' + dateStr + ')\\n' + quoted);
            }}
            const formatted = parts.join('\\n\\n');

            const msgs = data.messages.map(m => {{
                const d = new Date(parseFloat(m.ts) * 1000);
                const dateStr = d.toLocaleDateString('en-US', {{month:'long', day:'numeric', year:'numeric'}});
                return {{
                    user: users[m.user] || m.user || '?',
                    text: m.text || '',
                    date: dateStr,
                    ts: m.ts,
                    thread_ts: m.thread_ts || null,
                    reply_count: m.reply_count || 0
                }};
            }});
            return JSON.stringify({{channelName, messages: msgs, formatted}});
        }})()"#,
        channel = channel_id, limit = limit
    );

    match run_js(automation, &js).await {
        Ok(raw) => {
            let parsed: Value = serde_json::from_str(&raw).unwrap_or(Value::String(raw));
            let channel_name = parsed.get("channelName").and_then(|v| v.as_str()).unwrap_or(&channel_id).to_string();
            let messages = parsed.get("messages").cloned().unwrap_or(Value::Null);
            let formatted = parsed.get("formatted").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let count = messages.as_array().map(|a| a.len()).unwrap_or(0);
            ToolResult::Success(serde_json::json!({
                "ok": true, "channel_id": channel_id, "channel_name": channel_name,
                "messages": messages, "count": count,
                "formatted": formatted
            }))
        }
        Err(e) => ToolResult::Error(format!("slack_get_messages failed: {}", e)),
    }
}

async fn handle_slack_get_channel_info(
    args: &Value,
    automation: &dyn SlackAutomation,
) -> ToolResult {
    let channel_id = match get_string_arg(args, "channel_id") {
        Ok(c) => c,
        Err(e) => return ToolResult::Error(e),
    };
    info!("slack_get_channel_info: channel={}", channel_id);

    let js = format!(
        r#"(async () => {{
            const cfg = JSON.parse(localStorage.getItem('localConfig_v2') || '{{}}');
            const team = Object.values(cfg.teams || {{}})[0];
            if (!team) return JSON.stringify({{error: 'no team config'}});
            const r = await fetch('/api/conversations.info?token=' + encodeURIComponent(team.token) + '&channel={channel}&pretty=1');
            const data = await r.json();
            if (!data.ok) return JSON.stringify({{error: data.error}});
            const c = data.channel;
            return JSON.stringify({{
                id: c.id, name: c.name,
                topic: c.topic?.value || '',
                topic_creator: c.topic?.creator || '',
                topic_last_set: c.topic?.last_set || 0,
                purpose: c.purpose?.value || '',
                purpose_creator: c.purpose?.creator || '',
                purpose_last_set: c.purpose?.last_set || 0,
                is_archived: c.is_archived || false,
                is_general: c.is_general || false,
                members: c.num_members || 0
            }});
        }})()"#,
        channel = channel_id
    );

    match run_js(automation, &js).await {
        Ok(raw) => {
            let info: Value = serde_json::from_str(&raw).unwrap_or(Value::String(raw));
            let name = info.get("name").and_then(|v| v.as_str()).unwrap_or(&channel_id);
            let topic = info.get("topic").and_then(|v| v.as_str()).unwrap_or("");
            let purpose = info.get("purpose").and_then(|v| v.as_str()).unwrap_or("");
            let members = info.get("members").and_then(|v| v.as_u64()).unwrap_or(0);
            let formatted = format!("Channel: #{}\nTopic: {}\nPurpose: {}\nMembers: {}",
                name, topic, purpose, members);
            ToolResult::Success(serde_json::json!({
                "ok": true, "channel_id": channel_id, "info": info,
                "formatted": formatted
            }))
        }
        Err(e) => ToolResult::Error(e),
    }
}

async fn handle_slack_search(
    args: &Value,
    automation: &dyn SlackAutomation,
) -> ToolResult {
    let query = match get_string_arg(args, "query") {
        Ok(q) => q,
        Err(e) => return ToolResult::Error(e),
    };
    let count = get_int_arg(args, "limit", 10);
    info!("slack_search: query={}, count={}", query, count);

    let js = format!(
        r#"(async () => {{
            const cfg = JSON.parse(localStorage.getItem('localConfig_v2') || '{{}}');
            const team = Object.values(cfg.teams || {{}})[0];
            if (!team) return JSON.stringify({{error: 'no team config'}});
            const r = await fetch('/api/search.messages?token=' + encodeURIComponent(team.token) + '&query=' + encodeURIComponent('{query}') + '&count={count}&pretty=1');
            const data = await r.json();
            if (!data.ok) return JSON.stringify({{error: data.error}});
            const matches = (data.messages?.matches || []).map(m => ({{
                user: m.username || m.user || '?',
                text: (m.text || '').substring(0, 500),
                channel: m.channel?.name || '',
                channel_id: m.channel?.id || '',
                ts: m.ts,
                permalink: m.permalink || '',
                team: m.team || ''
            }}));
            const total = data.messages?.paging?.total_count || 0;

            const parts = ['Search results (' + total + ' total)'];
            for (const m of matches) {{
                const quoted = m.text.split('\\n').map(l => '> ' + l).join('\\n');
                parts.push(m.user + ' in #' + m.channel + '\\n' + quoted);
            }}
            const formatted = parts.join('\\n\\n');

            return JSON.stringify({{matches, total, formatted}});
        }})()"#,
        query = query.replace("'", "\\'"),
        count = count
    );

    match run_js(automation, &js).await {
        Ok(raw) => {
            let parsed: Value = serde_json::from_str(&raw).unwrap_or(Value::String(raw));
            let matches = parsed.get("matches").cloned().unwrap_or(Value::Null);
            let total = parsed.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
            let formatted = parsed.get("formatted").and_then(|v| v.as_str()).unwrap_or("").to_string();
            ToolResult::Success(serde_json::json!({
                "ok": true, "query": query,
                "result": {
                    "matches": matches,
                    "total": total
                },
                "formatted": formatted
            }))
        }
        Err(e) => ToolResult::Error(format!("slack_search failed: {}", e)),
    }
}

async fn handle_slack_get_profile(
    args: &Value,
    automation: &dyn SlackAutomation,
) -> ToolResult {
    let user_id = match get_string_arg(args, "user_id") {
        Ok(u) => u,
        Err(e) => return ToolResult::Error(e),
    };
    info!("slack_get_profile: user={}", user_id);

    let js = format!(
        r#"(async () => {{
            const cfg = JSON.parse(localStorage.getItem('localConfig_v2') || '{{}}');
            const team = Object.values(cfg.teams || {{}})[0];
            if (!team) return JSON.stringify({{error: 'no team config'}});
            const r = await fetch('/api/users.info?token=' + encodeURIComponent(team.token) + '&user={user}&pretty=1');
            const data = await r.json();
            if (!data.ok) return JSON.stringify({{error: data.error}});
            const u = data.user;
            return JSON.stringify({{
                id: u.id, name: u.name, real_name: u.real_name,
                display_name: u.profile?.display_name || '',
                email: u.profile?.email || '',
                title: u.profile?.title || '',
                phone: u.profile?.phone || '',
                is_admin: u.is_admin || false,
                is_owner: u.is_owner || false,
                tz: u.tz || ''
            }});
        }})()"#,
        user = user_id
    );

    match run_js(automation, &js).await {
        Ok(raw) => {
            let profile: Value = serde_json::from_str(&raw).unwrap_or(Value::String(raw));
            ToolResult::Success(serde_json::json!({"ok": true, "user_id": user_id, "profile": profile}))
        }
        Err(e) => ToolResult::Error(format!("slack_get_profile failed: {}", e)),
    }
}

async fn handle_slack_get_unread(
    automation: &dyn SlackAutomation,
) -> ToolResult {
    info!("slack_get_unread");
    let js = r#"JSON.stringify(Array.from(document.querySelectorAll('[data-qa-unread-indicator]')).map(el => ({
        name: el.innerText.trim(),
        channel_id: el.closest('[data-qa-channel-sidebar-channel]')?.getAttribute('data-qa-channel-sidebar-channel-id') || ''
    })).filter(c => c.channel_id))"#;

    match run_js(automation, js).await {
        Ok(raw) => {
            let unread: Value = serde_json::from_str(&raw).unwrap_or(Value::String(raw));
            ToolResult::Success(serde_json::json!({"ok": true, "unread": unread}))
        }
        Err(e) => ToolResult::Error(e),
    }
}

async fn handle_slack_send(
    args: &Value,
    automation: &dyn SlackAutomation,
) -> ToolResult {
    let channel_id = match get_string_arg(args, "channel_id") {
        Ok(c) => c,
        Err(e) => return ToolResult::Error(e),
    };
    let text = match get_string_arg(args, "text") {
        Ok(t) => t,
        Err(e) => return ToolResult::Error(e),
    };
    info!("slack_send: channel={}, text_len={}", channel_id, text.len());

    let js = format!(
        r#"(async () => {{
            const cfg = JSON.parse(localStorage.getItem('localConfig_v2') || '{{}}');
            const team = Object.values(cfg.teams || {{}})[0];
            if (!team) return JSON.stringify({{error: 'no team config'}});
            const r = await fetch('/api/chat.postMessage', {{
                method: 'POST',
                headers: {{'Content-Type': 'application/json', 'Authorization': 'Bearer ' + team.token}},
                body: JSON.stringify({{channel: '{channel}', text: '{text}'}})
            }});
            const data = await r.json();
            return JSON.stringify(data);
        }})()"#,
        channel = channel_id,
        text = escape_js_string(&text)
    );

    match run_js(automation, &js).await {
        Ok(raw) => {
            let data: Value = serde_json::from_str(&raw).unwrap_or(Value::String(raw));
            if data.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
                ToolResult::Success(serde_json::json!({"ok": true, "message": data.get("message"), "ts": data.get("ts")}))
            } else {
                let err = data.get("error").and_then(|v| v.as_str()).unwrap_or("unknown");
                ToolResult::Error(format!("send failed: {}", err))
            }
        }
        Err(e) => ToolResult::Error(format!("send failed: {}", e)),
    }
}

async fn handle_slack_reply(
    args: &Value,
    automation: &dyn SlackAutomation,
) -> ToolResult {
    let channel_id = match get_string_arg(args, "channel_id") {
        Ok(c) => c,
        Err(e) => return ToolResult::Error(e),
    };
    let thread_ts = match get_string_arg(args, "thread_ts") {
        Ok(t) => t,
        Err(e) => return ToolResult::Error(e),
    };
    let text = match get_string_arg(args, "text") {
        Ok(t) => t,
        Err(e) => return ToolResult::Error(e),
    };
    info!("slack_reply: channel={}, thread={}, text_len={}", channel_id, thread_ts, text.len());

    let js = format!(
        r#"(async () => {{
            const cfg = JSON.parse(localStorage.getItem('localConfig_v2') || '{{}}');
            const team = Object.values(cfg.teams || {{}})[0];
            if (!team) return JSON.stringify({{error: 'no team config'}});
            const r = await fetch('/api/chat.postMessage', {{
                method: 'POST',
                headers: {{'Content-Type': 'application/json', 'Authorization': 'Bearer ' + team.token}},
                body: JSON.stringify({{
                    channel: '{channel}',
                    thread_ts: '{thread_ts}',
                    text: '{text}'
                }})
            }});
            const data = await r.json();
            return JSON.stringify(data);
        }})()"#,
        channel = channel_id,
        thread_ts = thread_ts,
        text = escape_js_string(&text)
    );

    match run_js(automation, &js).await {
        Ok(raw) => {
            let data: Value = serde_json::from_str(&raw).unwrap_or(Value::String(raw));
            if data.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
                ToolResult::Success(serde_json::json!({"ok": true, "message": data.get("message"), "ts": data.get("ts")}))
            } else {
                let err = data.get("error").and_then(|v| v.as_str()).unwrap_or("unknown");
                ToolResult::Error(format!("reply failed: {}", err))
            }
        }
        Err(e) => ToolResult::Error(format!("reply failed: {}", e)),
    }
}

async fn handle_slack_edit(
    args: &Value,
    automation: &dyn SlackAutomation,
) -> ToolResult {
    let channel_id = match get_string_arg(args, "channel_id") {
        Ok(c) => c,
        Err(e) => return ToolResult::Error(e),
    };
    let ts = match get_string_arg(args, "ts") {
        Ok(t) => t,
        Err(e) => return ToolResult::Error(e),
    };
    let text = match get_string_arg(args, "text") {
        Ok(t) => t,
        Err(e) => return ToolResult::Error(e),
    };
    info!("slack_edit: channel={}, ts={}, text_len={}", channel_id, ts, text.len());

    let js = format!(
        r#"(async () => {{
            const cfg = JSON.parse(localStorage.getItem('localConfig_v2') || '{{}}');
            const team = Object.values(cfg.teams || {{}})[0];
            if (!team) return JSON.stringify({{error: 'no team config'}});
            const r = await fetch('/api/chat.update', {{
                method: 'POST',
                headers: {{'Content-Type': 'application/json', 'Authorization': 'Bearer ' + team.token}},
                body: JSON.stringify({{
                    channel: '{channel}',
                    ts: '{ts}',
                    text: '{text}'
                }})
            }});
            const data = await r.json();
            return JSON.stringify(data);
        }})()"#,
        channel = channel_id,
        ts = ts,
        text = escape_js_string(&text)
    );

    match run_js(automation, &js).await {
        Ok(raw) => {
            let data: Value = serde_json::from_str(&raw).unwrap_or(Value::String(raw));
            if data.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
                ToolResult::Success(serde_json::json!({"ok": true, "message": data.get("message"), "ts": data.get("ts")}))
            } else {
                let err = data.get("error").and_then(|v| v.as_str()).unwrap_or("unknown");
                ToolResult::Error(format!("edit failed: {}", err))
            }
        }
        Err(e) => ToolResult::Error(format!("edit failed: {}", e)),
    }
}

async fn handle_slack_delete(
    args: &Value,
    automation: &dyn SlackAutomation,
) -> ToolResult {
    let channel_id = match get_string_arg(args, "channel_id") {
        Ok(c) => c,
        Err(e) => return ToolResult::Error(e),
    };
    let ts = match get_string_arg(args, "ts") {
        Ok(t) => t,
        Err(e) => return ToolResult::Error(e),
    };
    info!("slack_delete: channel={}, ts={}", channel_id, ts);

    let js = format!(
        r#"(async () => {{
            const cfg = JSON.parse(localStorage.getItem('localConfig_v2') || '{{}}');
            const team = Object.values(cfg.teams || {{}})[0];
            if (!team) return JSON.stringify({{error: 'no team config'}});
            const r = await fetch('/api/chat.delete', {{
                method: 'POST',
                headers: {{'Content-Type': 'application/json', 'Authorization': 'Bearer ' + team.token}},
                body: JSON.stringify({{
                    channel: '{channel}',
                    ts: '{ts}'
                }})
            }});
            const data = await r.json();
            return JSON.stringify(data);
        }})()"#,
        channel = channel_id,
        ts = ts
    );

    match run_js(automation, &js).await {
        Ok(raw) => {
            let data: Value = serde_json::from_str(&raw).unwrap_or(Value::String(raw));
            if data.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
                ToolResult::Success(serde_json::json!({"ok": true}))
            } else {
                let err = data.get("error").and_then(|v| v.as_str()).unwrap_or("unknown");
                ToolResult::Error(format!("slack_delete failed: {}", err))
            }
        }
        Err(e) => ToolResult::Error(format!("slack_delete failed: {}", e)),
    }
}

async fn handle_slack_eval(
    args: &Value,
    automation: &dyn SlackAutomation,
) -> ToolResult {
    let js = match get_string_arg(args, "js") {
        Ok(j) => j,
        Err(e) => return ToolResult::Error(e),
    };
    info!("slack_eval, len={}", js.len());
    match run_js(automation, &js).await {
        Ok(result) => {
            let preview = truncate_preview(&result, 500);
            ToolResult::Success(serde_json::json!({
                "ok": true, "result": result, "preview": preview
            }))
        }
        Err(e) => ToolResult::Error(format!("slack_eval failed: {}", e)),
    }
}
