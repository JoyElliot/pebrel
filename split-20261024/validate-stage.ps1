param([ValidateSet('a','b','c')][string]$Stage, [switch]$Resume)
$ErrorActionPreference = 'Stop'
$OutputEncoding = [Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
$env:PATH = 'D:\Program Files\Git\bin;' + [Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [Environment]::GetEnvironmentVariable('Path','User')
$env:PATHEXT = [Environment]::GetEnvironmentVariable('PATHEXT','Machine')
$taskRoot = "F:\Git\pebrel\tmp\pr316-split-20261024\$Stage"
$taskSource = "$taskRoot\source"
$taskCargo = 'D:\DevTools\Rust\rustup\toolchains\1.97.1-x86_64-pc-windows-msvc\bin\cargo.exe'
$taskTarget = 'F:\Git\pebrel\tmp\acrylic-repro-20260921\target-current'
$taskConfig = 'F:\Git\pebrel\tmp\pr-316-verification\native-compiler.toml'
$taskCommon = @('--locked','--target-dir',$taskTarget,'--config',$taskConfig)
$taskResults = @{}
function Invoke-StageCargo([string]$Name, [string[]]$CargoArguments) {
    Write-Host "$Stage $Name"
    $process = Start-Process -FilePath $taskCargo -ArgumentList $CargoArguments -WorkingDirectory $taskSource -WindowStyle Hidden -RedirectStandardOutput "$taskRoot\$Name.stdout" -RedirectStandardError "$taskRoot\$Name.stderr" -PassThru
    $taskHandle = $process.Handle
    $process.WaitForExit()
    $process.Refresh()
    if ($null -eq $process.ExitCode) { throw "No exit code for $Name (process $($process.Id))" }
    $taskResults[$Name] = $process.ExitCode
    $taskResults | ConvertTo-Json | Set-Content "$taskRoot\progress.json" -Encoding utf8
    return $process.ExitCode
}
if (-not $Resume) {
$cleanCode = Invoke-StageCargo 'clean' @('clean','-p','nebula_terminal','-p','nebula','--target-dir',$taskTarget,'--config',$taskConfig)
if ($cleanCode -ne 0) { exit $cleanCode }
$null = Invoke-StageCargo 'core' (@('test','-p','nebula_terminal','--all-targets') + $taskCommon + @('--','--test-threads=4'))
$null = Invoke-StageCargo 'file-budget' (@('test','-p','nebula','--test','file_line_budget','--features','gpui-shell') + $taskCommon)
$null = Invoke-StageCargo 'gpui-check' (@('check','-p','nebula','--bin','pebrel','--tests','--features','gpui-shell') + $taskCommon)
$null = Invoke-StageCargo 'legacy-product-check' (@('check','-p','nebula','--bin','pebrel','--no-default-features','--features','legacy-shell') + $taskCommon)
$null = Invoke-StageCargo 'legacy-tests-check' (@('check','-p','nebula','--bin','pebrel','--tests','--no-default-features','--features','legacy-shell') + $taskCommon)
$buildCode = Invoke-StageCargo 'build' (@('test','-p','nebula','--bin','pebrel','--features','gpui-test-support','--no-run','--message-format=json') + $taskCommon)
if ($buildCode -ne 0) { exit $buildCode }
} else {
    $saved = Get-Content "$taskRoot\progress.json" -Raw -Encoding utf8 | ConvertFrom-Json
    foreach ($property in $saved.PSObject.Properties) { $taskResults[$property.Name] = $property.Value }
    if ($taskResults['build'] -ne 0) { throw 'Cannot resume an unsuccessful build' }
}
$artifact = Get-Content "$taskRoot\build.stdout" -Encoding utf8 | ForEach-Object { $_ | ConvertFrom-Json } | Where-Object { $_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'pebrel' -and $_.profile.test -and $_.executable } | Select-Object -Last 1
if (-not $artifact) { throw 'No product test executable returned' }
Copy-Item -LiteralPath $artifact.executable -Destination "$taskRoot\product-test.exe"
Write-Host "$Stage full-product-tests"
$full = Start-Process -FilePath "$taskRoot\product-test.exe" -ArgumentList '--test-threads=4' -WorkingDirectory $taskSource -WindowStyle Hidden -RedirectStandardOutput "$taskRoot\full.stdout" -RedirectStandardError "$taskRoot\full.stderr" -PassThru
$taskHandle = $full.Handle
$full.WaitForExit()
$full.Refresh()
if ($null -eq $full.ExitCode) { throw 'No exit code for the product test executable' }
$taskResults['full'] = $full.ExitCode
$taskResults['test_binary_sha256'] = (Get-FileHash "$taskRoot\product-test.exe" -Algorithm SHA256).Hash
$taskResults['source'] = Get-Content "$taskRoot\source-manifest.json" -Raw -Encoding utf8 | ConvertFrom-Json | Select-Object commit,tree,base
$taskResults | ConvertTo-Json -Depth 4 | Set-Content "$taskRoot\results.json" -Encoding utf8
Write-Host "$Stage complete"
exit 0
