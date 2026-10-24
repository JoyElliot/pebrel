$ErrorActionPreference = 'Stop'
$env:PATH = 'D:\Program Files\Git\bin;' + [Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [Environment]::GetEnvironmentVariable('Path','User')
$env:PATHEXT = [Environment]::GetEnvironmentVariable('PATHEXT','Machine')
$taskRoot = 'F:\Git\pebrel\tmp\pr316-split-20261024\c'
Set-Location "$taskRoot\source"
& "$taskRoot\source\scripts\tests\run_completion_e2e.ps1" -TestBinary "$taskRoot\product-test.exe" -OutputDirectory "$taskRoot\native-editor" -Case editor -RuntimeDirectory "$env:LOCALAPPDATA\Programs\Pebrel\runtime" -PowerShellProgram 'C:\Program Files\PowerShell\7\pwsh.exe'
exit $LASTEXITCODE
