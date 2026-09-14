#!/bin/sh
# Impresari Context hook recipe v1; ownership=exact_fixed_artifact:impresari-context
#
# Claude Code runs this after a Bash command succeeds (PostToolUse, matcher
# "Bash", each entry filtered by "if" to one test or build runner). It hands
# Claude Code's payload to `impresari-context hook claude-code post-tool-use`,
# which prints either nothing or a replacement holding only lines of the
# command's own output, in their original order, plus a note saying so.
# The binary runs with an empty environment. This script reads no repository
# file, opens no network connection, writes nothing, and always exits 0, so it
# can never block or fail a command. Without impresari-context on PATH it does
# nothing.
bin=$(command -v impresari-context) || { cat >/dev/null; exit 0; }
env -i "$bin" hook claude-code post-tool-use || true
exit 0
