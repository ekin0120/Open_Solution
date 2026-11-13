param(
    [Parameter(Mandatory=$true)][string]$MdbPath,
    [Parameter(Mandatory=$true)][string]$OutDir,
    [string]$LogPath = ''
)
$ErrorActionPreference = 'Continue'
Add-Type -AssemblyName System.Data

$f = Get-Item -LiteralPath $MdbPath
$result = [ordered]@{}
$result.mdb = $f.Name
$result.size = $f.Length
$result.jet_version = [int]([System.IO.File]::ReadAllBytes($f.FullName)[0x14])
$result.password_id = ''
$result.error = ''
$result.tables = @()

$passwords = @('FelixRick','Leonardo')
$conn = $null
foreach ($pw in $passwords) {
    $cs = "Provider=Microsoft.Jet.OLEDB.4.0;Data Source=$($f.FullName);Mode=Read;Jet OLEDB:Database Password=$pw;"
    $c = $null
    try {
        $c = New-Object System.Data.OleDb.OleDbConnection($cs)
        $c.Open()
        $conn = $c
        $result.password_id = $pw
        break
    } catch {
        if ($LogPath) { [System.IO.File]::AppendAllText($LogPath, "$($f.Name) pw=$pw -> $($_.Exception.Message)`r`n") }
        if ($c) { try { $c.Dispose() } catch {} }
    }
}

if ($conn) {
    try {
        $schema = $conn.GetSchema('Tables')
        $tbls = New-Object System.Collections.ArrayList
        foreach ($sr in $schema.Rows) {
            if ($sr['TABLE_TYPE'] -ne 'TABLE') { continue }
            $tname = [string]$sr['TABLE_NAME']
            $t = [ordered]@{ name = $tname; columns = @(); rows = @(); error = '' }
            try {
                $cmd = $conn.CreateCommand()
                $cmd.CommandText = "SELECT * FROM [$tname]"
                $da = New-Object System.Data.OleDb.OleDbDataAdapter($cmd)
                $ds = New-Object System.Data.DataSet
                $da.Fill($ds) | Out-Null
                $dt = $ds.Tables[0]
                $cols = @()
                foreach ($c in $dt.Columns) { $cols += [ordered]@{ name = $c.ColumnName; clr = $c.DataType.Name } }
                $t.columns = $cols

                $rows = New-Object System.Collections.ArrayList
                foreach ($r in $dt.Rows) {
                    $o = [ordered]@{}
                    foreach ($c in $dt.Columns) {
                        $v = $r[$c]
                        if ($v -is [System.DBNull]) { $o[$c.ColumnName] = $null }
                        elseif ($v -is [byte[]]) { $o[$c.ColumnName] = 'base64:' + [Convert]::ToBase64String($v) }
                        elseif ($v -is [datetime]) { $o[$c.ColumnName] = $v.ToString('o') }
                        elseif ($v -is [bool]) { $o[$c.ColumnName] = [int]$v }
                        else { $o[$c.ColumnName] = $v }
                    }
                    $rows.Add($o) | Out-Null
                }
                $t.rows = [object[]]$rows.ToArray()
                $da.Dispose(); $cmd.Dispose(); $ds.Dispose()
            } catch {
                $t.error = $_.Exception.Message
                if ($LogPath) { [System.IO.File]::AppendAllText($LogPath, "TABLE $($f.Name)/$tname -> $($_.Exception.Message)`r`n") }
            }
            $tbls.Add($t) | Out-Null
        }
        $result.tables = [object[]]$tbls.ToArray()
        $conn.Close()
    } catch {
        $result.error = $_.Exception.Message
    } finally {
        $conn.Dispose()
    }
} else {
    $result.error = 'no valid password'
}

$json = ConvertTo-Json -InputObject $result -Depth 12
$out = Join-Path $OutDir ($f.BaseName + '.json')
[System.IO.File]::WriteAllText($out, $json, (New-Object System.Text.UTF8Encoding($false)))
$n = @($result.tables).Count
$r = 0; foreach ($t in $result.tables) { $r += @($t.rows).Count }
Write-Output ("{0,-26} pw={1,-10} tables={2,-4} rows={3,-6} err={4}" -f $f.Name, $result.password_id, $n, $r, $result.error)
if ($conn) { exit 0 } else { exit 2 }
