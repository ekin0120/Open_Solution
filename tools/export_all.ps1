param(
    [string]$Root = 'C:\Users\ekinz\WorkBuddy\crafteverything\Open Solutions\import_work'
)
$ErrorActionPreference = 'Continue'
$root = $Root
$p32  = "$env:WINDIR\SysWOW64\WindowsPowerShell\v1.0\powershell.exe"
$one  = Join-Path $root 'export_one.ps1'
$out  = Join-Path $root 'dump2'
$mdbDir = Join-Path $root 'mdb'
$log  = Join-Path $root 'export.log'

New-Item -ItemType Directory -Force -Path $out | Out-Null
foreach ($x in [System.IO.Directory]::GetFiles($out, '*.json')) { [System.IO.File]::Delete($x) }
[System.IO.File]::WriteAllText($log, '', (New-Object System.Text.UTF8Encoding($false)))

$all = @(Get-ChildItem -LiteralPath $mdbDir -Filter '*.mdb' | Sort-Object Name)
$fail = @()
foreach ($f in $all) {
    $tries = 0
    $done = $false
    # Jet 4.0 caches a failed password check for the whole process. If one
    # attempt fails, start a brand-new 32-bit process instead of retrying
    # inside this one.
    for ($a = 1; $a -le 15; $a++) {
        foreach ($x in [System.IO.Directory]::GetFiles($mdbDir, '*.ldb')) {
            try { [System.IO.File]::Delete($x) } catch {}
        }
        $tries = $a
        $line = & $p32 -NoProfile -NonInteractive -ExecutionPolicy Bypass -File $one -MdbPath $f.FullName -OutDir $out -LogPath $log
        if ($LASTEXITCODE -eq 0) {
            Write-Output ("{0}  (attempt {1})" -f $line, $a)
            $done = $true
            break
        }
    }
    if (-not $done) { $fail += $f.Name; Write-Output "FAILED $($f.Name) after $tries attempts" }
}
Write-Output ("DONE  ok={0}  failed={1} {2}" -f ($all.Count - $fail.Count), $fail.Count, ($fail -join ','))
