use anyhow::{Context, Result};

use super::{BlackboardItem, pii_check, pii_scrub};

#[derive(Debug, Default)]
pub(crate) struct BlackboardCliOutput {
    pub(crate) exit_code: Option<i32>,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

pub(crate) async fn run_blackboard(
    text: Option<String>,
    search: Option<String>,
    from: Option<String>,
    since_hours: Option<f64>,
    limit: usize,
    port: u16,
) -> Result<BlackboardCliOutput> {
    let mut output = BlackboardCliOutput::default();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    let base = format!("http://127.0.0.1:{port}");

    let status_resp = client.get(format!("{base}/api/status")).send().await;
    if status_resp.is_err() {
        output.exit_code = Some(1);
        output.stderr.push_str(&format!(
            "No mesh-llm node running on port {port}.\n\n\
             Blackboard requires a running mesh node:\n\
             Private mesh:  mesh-llm client  (share the join token printed out)\n\
             Join a mesh:   mesh-llm client --join <token>\n\
             Public mesh:   mesh-llm client --auto\n\n\
             See https://github.com/Mesh-LLM/mesh-llm for setup guide.\n"
        ));
        return Ok(output);
    }

    let feed_check = client
        .get(format!("{base}/api/plugins/blackboard/http/feed?limit=1"))
        .send()
        .await;
    if let Ok(resp) = feed_check
        && resp.status().as_u16() == 404
    {
        output.exit_code = Some(1);
        output.stderr.push_str(
            "Mesh is running but blackboard is disabled on that node.\n\
             Install or enable the external blackboard plugin in the mesh config.\n",
        );
        return Ok(output);
    }

    let default_hours = 24.0;
    let since_secs = {
        let hours = since_hours.unwrap_or(default_hours);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now.saturating_sub((hours * 3600.0) as u64)
    };

    if let Some(msg) = text {
        let issues = pii_check(&msg);
        if !issues.is_empty() {
            output.stderr.push_str("⚠️  PII/secret issues detected:\n");
            for issue in &issues {
                output.stderr.push_str(&format!("   • {issue}\n"));
            }
            output.stderr.push_str("Scrubbing and posting...\n");
        }
        let clean = pii_scrub(&msg);

        let body = serde_json::json!({ "text": clean });
        let resp = client
            .post(format!("{base}/api/plugins/blackboard/http/post"))
            .json(&body)
            .send()
            .await
            .context("Cannot reach mesh-llm — is it running?")?;
        if resp.status().is_success() {
            let item: BlackboardItem = resp.json().await?;
            output
                .stderr
                .push_str(&format!("📝 Posted (id: {:x})\n", item.id));
        } else {
            let err = resp.text().await.unwrap_or_default();
            output.stderr.push_str(&format!("Error: {err}\n"));
        }
        return Ok(output);
    }

    if let Some(q) = search {
        let resp = client
            .get(format!("{base}/api/plugins/blackboard/http/search"))
            .query(&[
                ("q", q.as_str()),
                ("limit", &limit.to_string()),
                ("since", &since_secs.to_string()),
            ])
            .send()
            .await
            .context("Cannot reach mesh-llm — is it running?")?;
        let items: Vec<BlackboardItem> = resp.json().await?;
        if items.is_empty() {
            output.stderr.push_str("No results.\n");
        } else {
            output.stdout.push_str(&format_blackboard_items(&items));
        }
        return Ok(output);
    }

    let mut params = vec![
        ("limit", limit.to_string()),
        ("since", since_secs.to_string()),
    ];
    if let Some(ref f) = from {
        params.push(("from", f.clone()));
    }
    let resp = client
        .get(format!("{base}/api/plugins/blackboard/http/feed"))
        .query(&params)
        .send()
        .await
        .context("Cannot reach mesh-llm — is it running?")?;
    let items: Vec<BlackboardItem> = resp.json().await?;
    if items.is_empty() {
        output.stderr.push_str("Blackboard is empty.\n");
    } else {
        output.stdout.push_str(&format_blackboard_items(&items));
    }
    Ok(output)
}

fn format_blackboard_items(items: &[BlackboardItem]) -> String {
    let mut output = String::new();
    for item in items {
        let time = chrono_format(item.timestamp);
        output.push_str(&format!("{:x} │ {} │ {}\n", item.id, time, item.from));
        for line in item.text.lines() {
            output.push_str(&format!("  {line}\n"));
        }
        output.push('\n');
    }
    output
}

fn chrono_format(ts: u64) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let ago = now.saturating_sub(ts);
    if ago < 60 {
        format!("{ago}s ago")
    } else if ago < 3600 {
        format!("{}m ago", ago / 60)
    } else if ago < 86400 {
        format!("{}h ago", ago / 3600)
    } else {
        format!("{}d ago", ago / 86400)
    }
}

pub(crate) fn install_skill() -> Result<()> {
    let skill_content = include_str!("../skills/blackboard/SKILL.md");
    let home =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?;
    let skill_dir = home.join(".agents").join("skills").join("blackboard");
    std::fs::create_dir_all(&skill_dir)?;
    let skill_path = skill_dir.join("SKILL.md");
    std::fs::write(&skill_path, skill_content)?;
    eprintln!("✅ Installed blackboard skill to {}", skill_path.display());
    eprintln!("   Works with pi, Goose, and other agents that read ~/.agents/skills/");
    eprintln!(
        "   Make sure mesh-llm is running and the blackboard plugin is not disabled in config."
    );
    Ok(())
}
