"use strict";

const state = { snapshot: null, policy: null, lastPreview: null };
const byId = (id) => document.getElementById(id);

function showJson(target, value) {
  target.textContent = JSON.stringify(value, null, 2);
}

function option(select, value) {
  if (![...select.options].some((item) => item.value === value)) {
    const item = document.createElement("option");
    item.value = value;
    item.textContent = value;
    select.append(item);
  }
}

function render() {
  if (!state.snapshot) return;
  const snapshot = state.snapshot;
  const records = snapshot.records || [];
  const summary = byId("summary");
  summary.replaceChildren();
  for (const [label, value] of [["Events", records.length], ["Unavailable rows", snapshot.unavailable_rows], ["Stream sequence", snapshot.stream_sequence]]) {
    const group = document.createElement("div");
    const term = document.createElement("dt");
    const detail = document.createElement("dd");
    term.textContent = label;
    detail.textContent = String(value);
    group.append(term, detail);
    summary.append(group);
  }
  const outcome = byId("outcome-filter");
  const capability = byId("capability-filter");
  for (const record of records) {
    option(outcome, record.outcome);
    option(capability, record.capability);
  }
  const visible = records.filter((record) => (!outcome.value || record.outcome === outcome.value) && (!capability.value || record.capability === capability.value));
  const body = byId("events");
  body.replaceChildren();
  for (const record of visible) {
    const row = document.createElement("tr");
    for (const value of [record.occurred_at, record.capability, record.outcome, `${record.duration_ms} ms`, record.workspace_label || "—"]) {
      const cell = document.createElement("td");
      cell.textContent = String(value);
      row.append(cell);
    }
    body.append(row);
  }
  showJson(byId("policy-state"), state.policy);
}

async function request(path, options = {}) {
  const headers = new Headers(options.headers || {});
  headers.set("X-Impresari-CSRF", "1");
  const response = await fetch(path, { ...options, headers, credentials: "same-origin" });
  const value = await response.json();
  if (!response.ok) throw new Error(value.code || "local request failed");
  return value;
}

async function refresh() {
  const value = await request("/api/state");
  state.snapshot = value.snapshot;
  state.policy = value.policy;
  render();
}

async function mutatePolicy(apply) {
  const draft = JSON.parse(byId("policy-draft").value);
  const current = state.policy || {};
  const result = await request("/api/policy/apply", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ draft, expected_policy_id: current.current_policy_id || null, expected_revision: current.current_revision || null, expected_preview_receipt_id: apply && state.lastPreview?.operation === "apply" ? state.lastPreview.receipt_id : null, apply })
  });
  state.lastPreview = apply ? null : result;
  showJson(byId("policy-result"), result);
  await refresh();
}

async function lifecyclePolicy(operation, apply) {
  const current = state.policy || {};
  const result = await request(`/api/policy/${operation}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ expected_policy_id: current.current_policy_id || null, expected_revision: current.current_revision || null, expected_preview_receipt_id: apply && state.lastPreview?.operation === operation ? state.lastPreview.receipt_id : null, apply })
  });
  state.lastPreview = apply ? null : result;
  showJson(byId("policy-result"), result);
  await refresh();
}

async function bootstrap() {
  const token = location.hash.slice(1);
  history.replaceState(null, "", location.pathname);
  if (!/^[a-f0-9]{64}$/.test(token)) throw new Error("missing local bootstrap capability");
  const response = await fetch("/api/bootstrap", { method: "POST", headers: { "X-Impresari-Bootstrap": token, "X-Impresari-CSRF": "bootstrap" }, credentials: "same-origin" });
  if (!response.ok) throw new Error("local bootstrap rejected");
  await refresh();
  const events = new EventSource("/api/events");
  events.addEventListener("snapshot", (event) => { state.snapshot = JSON.parse(event.data); render(); });
  events.addEventListener("reset_required", (event) => { state.snapshot = JSON.parse(event.data); render(); });
  events.onerror = () => { byId("connection").textContent = "Local stream disconnected"; };
  byId("connection").textContent = "Connected to this foreground process";
}

byId("outcome-filter").addEventListener("change", render);
byId("capability-filter").addEventListener("change", render);
byId("preview-policy").addEventListener("click", () => mutatePolicy(false).catch((error) => { byId("policy-result").textContent = error.message; }));
byId("apply-policy").addEventListener("click", () => mutatePolicy(true).catch((error) => { byId("policy-result").textContent = error.message; }));
byId("preview-remove").addEventListener("click", () => lifecyclePolicy("remove", false).catch((error) => { byId("policy-result").textContent = error.message; }));
byId("apply-remove").addEventListener("click", () => lifecyclePolicy("remove", true).catch((error) => { byId("policy-result").textContent = error.message; }));
byId("preview-rollback").addEventListener("click", () => lifecyclePolicy("rollback", false).catch((error) => { byId("policy-result").textContent = error.message; }));
byId("apply-rollback").addEventListener("click", () => lifecyclePolicy("rollback", true).catch((error) => { byId("policy-result").textContent = error.message; }));
byId("shutdown").addEventListener("click", () => request("/api/shutdown", { method: "POST" }).then(() => { byId("connection").textContent = "Local session ended"; }).catch((error) => { byId("connection").textContent = error.message; }));
bootstrap().catch((error) => { byId("connection").textContent = error.message; });
