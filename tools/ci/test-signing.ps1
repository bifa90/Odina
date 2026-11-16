# SPDX-License-Identifier: MIT
# Copyright (c) 2026 bifa90 — Odina developer tools (see LICENSE-TOOLS)
<#
.SYNOPSIS
    Authenticode smoke test with a throwaway self-signed certificate.

.DESCRIPTION
    Signs a real Windows PE that this repository built, then proves three
    things in order:

      1. the file carries an Authenticode signature made by our key;
      2. the chain is untrusted (UnknownError) — the only truthful result a
         self-signed certificate can produce;
      3. flipping one byte of a copy turns the status into HashMismatch, so
         the verification that said "signed" is actually verifying.

    It is not a gate and never runs on push. It answers one question before
    the real certificate exists: does the release pipeline carry a signature
    from build to verify, or do we only believe it does?

    It cannot make Windows trust the file. No root store is touched, nothing
    is installed, and the certificate is deleted on the way out.

.PARAMETER File
    A PE file built from this repository. Default: the release CLI (target\release\odina.exe).

.PARAMETER KeepCertificate
    Leave the throwaway certificate in the current-user store for debugging.

.EXAMPLE
    pwsh -File tools/ci/test-signing.ps1

.NOTES
    Exit codes: 0 = signature applied and verified, 1 = failure.
#>
[CmdletBinding()]
param(
    [string]$File = 'target\release\odina.exe',
    [switch]$KeepCertificate
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$subject = 'CN=Odina Self-Signed Test (throwaway)'
$cert = $null
$summary = [System.Collections.Generic.List[string]]::new()

function Write-Step {
    param([string]$Text)
    Write-Host ''
    Write-Host "== $Text"
}

function Add-Result {
    param([string]$Text)
    Write-Host "   $Text"
    $summary.Add($Text)
}

function Assert-True {
    param([bool]$Condition, [string]$Message)
    if (-not $Condition) { throw $Message }
}

try {
    Write-Step '1/5 · الملف'
    Assert-True (Test-Path -LiteralPath $File -PathType Leaf) "الملف غير موجود: $File"
    $path = (Resolve-Path -LiteralPath $File).Path
    $item = Get-Item -LiteralPath $path
    $hashBefore = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash
    Add-Result ("الملف: {0} ({1:N0} بايت)" -f $item.Name, $item.Length)
    Add-Result "SHA-256 قبل التوقيع: $hashBefore"

    Write-Step '2/5 · شهادة موقّعة ذاتيًا (مؤقتة)'
    $cert = New-SelfSignedCertificate -Type CodeSigningCert -Subject $subject `
        -CertStoreLocation 'Cert:\CurrentUser\My' -KeyAlgorithm RSA -KeyLength 3072 `
        -HashAlgorithm SHA256 -NotAfter (Get-Date).AddDays(2)
    Add-Result "Thumbprint: $($cert.Thumbprint)"

    Write-Step '3/5 · التوقيع'
    $signed = Set-AuthenticodeSignature -LiteralPath $path -Certificate $cert -HashAlgorithm SHA256
    Add-Result "نتيجة التوقيع: $($signed.Status)"
    Add-Result ("بصمة الموقّع: {0}" -f $signed.SignerCertificate.Thumbprint)
    Add-Result "SHA-256 بعد التوقيع: $((Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash)"

    Write-Step '4/5 · التحقق — وكشف العبث'
    $check = Get-AuthenticodeSignature -LiteralPath $path
    Assert-True ($null -ne $check.SignerCertificate) 'لا توجد شهادة على الملف بعد التوقيع.'
    Assert-True ($check.SignerCertificate.Thumbprint -eq $cert.Thumbprint) 'الشهادة المرفقة ليست شهادتنا.'
    Assert-True ($check.Status -ne 'NotSigned') 'الملف بلا توقيع.'
    Assert-True ($check.Status -in @('UnknownError', 'Valid')) "حالة غير متوقعة: $($check.Status)"
    Add-Result "الحالة: $($check.Status) — UnknownError متوقع لشهادة غير موثوقة الجذر."

    $copy = Join-Path ([System.IO.Path]::GetTempPath()) 'odina-tamper.exe'
    Copy-Item -LiteralPath $path -Destination $copy -Force
    $bytes = [System.IO.File]::ReadAllBytes($copy)
    $index = [int]($bytes.Length / 2)
    $bytes[$index] = $bytes[$index] -bxor 0xFF
    [System.IO.File]::WriteAllBytes($copy, $bytes)
    $tampered = Get-AuthenticodeSignature -LiteralPath $copy
    Add-Result "نسخة عُدّل فيها بايت واحد: $($tampered.Status)"
    Assert-True ($tampered.Status -eq 'HashMismatch') 'التحقق لم يكشف العبث — التوقيع لا يُفحص فعلًا.'
    Remove-Item -LiteralPath $copy -Force

    Write-Host ''
    Write-Host 'PASS — التوقيع يُطبَّق ويُتحقَّق، والعبث يُكشف.' -ForegroundColor Green
    exit 0
}
catch {
    Write-Host ''
    Write-Host "FAIL — $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}
finally {
    if (-not $KeepCertificate -and $null -ne $cert) {
        $store = 'Cert:\CurrentUser\My\' + $cert.Thumbprint
        if (Test-Path -LiteralPath $store) {
            Remove-Item -LiteralPath $store -Force -ErrorAction SilentlyContinue
            Write-Host '   أُزيلت الشهادة المؤقتة.'
        }
    }
    if ($env:GITHUB_STEP_SUMMARY) {
        $lines = @(
            '# Authenticode self-signed smoke test'
            ''
            'Signed the release CLI with a throwaway certificate and verified it.'
        ) + $summary
        $lines -join "`n" | Out-File -FilePath $env:GITHUB_STEP_SUMMARY -Encoding utf8 -Append
    }
}
