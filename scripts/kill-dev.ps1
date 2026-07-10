# Kill the dev server bound to port 8080 (free it before "make c").
# Only the process listening on 8080 is stopped; other dx instances on other ports stay up.
# NOTE (000.PS.ASCII): keep this file ASCII-only - no Cyrillic, no em-dash. PowerShell 5.1
# reads BOM-less files as ANSI and mojibake breaks parsing.
$conns = Get-NetTCPConnection -LocalPort 8080 -State Listen -ErrorAction SilentlyContinue
if (-not $conns) {
    Write-Output 'port 8080 already free'
    return
}
$conns.OwningProcess | Select-Object -Unique | ForEach-Object {
    Stop-Process -Id $_ -Force -ErrorAction SilentlyContinue
}
Write-Output 'port 8080 killed'
