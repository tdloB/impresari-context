# CI-3c GitHub Copilot CLI guided-delivery admission

Date: 2026-08-28  
Platform: macOS arm64  
Admitted client: GitHub Copilot CLI `1.0.80`  
Protocol: `programmatic_prompt_json_events_v1`

Two independent live rehearsals delivered distinct, snapshot-bound canonical
packets through Copilot's programmatic prompt lifecycle. Both used an isolated
`COPILOT_HOME`, an explicitly named isolated GitHub CLI authentication directory
in place, a disposable empty runtime, disabled MCP servers, and an empty
available-tool set. Both observed the exact prompt event, one successful
terminal result, zero tool requests or executions, immutable source, clean
runtime removal, no source-workspace exposure, and no added authority.

| Run | Packet | Plan | Snapshot | Result |
| --- | --- | --- | --- | --- |
| 1 | `sha256:c5741d8a933995d6a67c462c5bc556d519b2529b815b96ab6dca7fc91733bd23` | `sha256:7c478fedf2e49f6d2740da6fae02ed9f4f3e96d5b0a857fc48e28a97a4269e70` | `sha256:878abcc9f49aa94c62dfdc93ffbe5728b68d733f8a309200780ffe045a546187` | delivered |
| 2 | `sha256:855a64dc12ff46e9092ab8823a01e58d23c83cc4366174d5bb6d71ae63aa216b` | `sha256:bf072fd7fba2c33487d01d7f74ea59dd21155c17284f09ebc3e3c46c38ad2d23` | `sha256:9c06834e6d802ae0b782a55481a87857064d8daf8f4d82602e6eee9bca769fc0` | delivered |

The source digest remained
`0c327c4bcb0f06ab595264a0efc26d1f78ce4802020c20e1e16857810087efc2`
in both runs. Credential state was neither copied nor deleted.

Copilot CLI `1.0.81` is explicitly not admitted. A compatibility probe showed
that it supplied 18 built-in tool schemas to the model even with an empty
`--available-tools` value and `--excluded-tools=*`. No tool executed, but that
expanded surface violates CI-3c's zero-tool boundary. The live record used the
intact locally cached `1.0.80` runtime directly.

```text
ruby scripts/rehearse-copilot-guided-delivery.rb \
  --copilot /absolute/path/to/copilot-1.0.80/app.js \
  --copilot-home /absolute/path/to/isolated/copilot-home \
  --github-auth-config /absolute/path/to/isolated/gh-config \
  --runs 2
```
