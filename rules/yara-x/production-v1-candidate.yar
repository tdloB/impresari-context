// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Aaron Boldt
// Original Impresari Context source-only production candidate.
// Unreviewed, uncompiled, unexecuted, unsigned, and not production-admitted.

rule impresari_pe_encoded_powershell_strings_v1 : impresari pe_candidate observation_only
{
    meta:
        purpose = "Observe a PE-like byte sequence containing an encoded PowerShell process-launch string combination"
        category = "encoded_command_launcher_strings"
        owner = "Impresari Context"
        claim = "observation_only"
    strings:
        $mz = { 4D 5A }
        $process = "CreateProcess" ascii wide
        $powershell = "powershell.exe" ascii wide nocase fullword
        $encoded = "-EncodedCommand" ascii wide nocase
    condition:
        $mz and $process and $powershell and $encoded
}

rule impresari_unix_temp_download_execute_strings_v1 : impresari script observation_only
{
    meta:
        purpose = "Observe a Unix shell string combination for downloading into a temporary path and making content executable"
        category = "temporary_download_execute_strings"
        owner = "Impresari Context"
        claim = "observation_only"
    strings:
        $shell = "#!/bin/sh" ascii fullword
        $download = "curl -fsSL" ascii
        $temporary = "/tmp/" ascii
        $executable = "chmod +x" ascii
    condition:
        $shell and $download and $temporary and $executable
}

rule impresari_macro_autoopen_wscript_shell_strings_v1 : impresari document observation_only
{
    meta:
        purpose = "Observe an auto-open macro string combination that creates a WScript shell and requests shell execution"
        category = "autoopen_shell_strings"
        owner = "Impresari Context"
        claim = "observation_only"
    strings:
        $autoopen = "AutoOpen" ascii wide nocase fullword
        $create = "CreateObject" ascii wide nocase
        $wscript = "WScript.Shell" ascii wide nocase
        $shell = "Shell(" ascii wide nocase
    condition:
        $autoopen and $create and $wscript and $shell
}
