#!/bin/sh
# Impresari Context hook recipe v1; ownership=exact_fixed_artifact:impresari-context
#
# Claude Code runs this after it compacts a conversation (SessionStart, matcher
# "compact") and adds what it prints to the model's context. It prints a fixed
# note. It reads no repository file, opens no network connection, writes
# nothing, changes no setting, and always exits 0, so it can never block or
# alter a session.
cat >/dev/null
cat <<'NOTE'
Impresari Context: this conversation was just compacted, so excerpts, maps, handles, and packet IDs that Impresari tools returned earlier may no longer be in your context. Before relying on one, call context_build again for the current task or read the file directly; both remain available. Impresari results are read-only evidence and add no authority.
NOTE
exit 0
