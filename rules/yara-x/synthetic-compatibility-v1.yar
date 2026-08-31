// SPDX-License-Identifier: Apache-2.0
// Original synthetic compatibility rules. These are not malware signatures.

rule impresari_synthetic_literal_v1 : impresari synthetic
{
    strings:
        $literal = "IMPRESARI_SYNTHETIC_LITERAL_7A31C9" ascii fullword
    condition:
        $literal
}

rule impresari_synthetic_hex_v1 : impresari synthetic
{
    strings:
        $hex = { 49 4D 50 52 45 53 41 52 49 5F 53 59 4E 54 48 45 54 49 43 5F 48 45 58 5F 34 44 32 38 }
    condition:
        $hex
}

rule impresari_synthetic_wide_v1 : impresari synthetic
{
    strings:
        $wide = "IMPRESARI_SYNTHETIC_WIDE_91B6" ascii wide fullword
    condition:
        $wide
}
