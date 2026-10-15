$ErrorActionPreference='Stop'
$env:PATH='D:\Program Files\Git\bin;'+[Environment]::GetEnvironmentVariable('Path','Machine')+';'+[Environment]::GetEnvironmentVariable('Path','User')
$env:PATHEXT=[Environment]::GetEnvironmentVariable('PATHEXT','Machine')
$taskRoot='F:\Git\pebrel\tmp\pr316-conflict-20261015'
$results=@()
foreach($variant in @('merged','main')) {
 $working=if($variant -eq 'main'){"$taskRoot\main-source"}else{"$taskRoot\source"}
 foreach($filter in @('gpui_shell::workspace::settings_navigation','gpui_shell::workspace')) {
  $name=$filter.Replace(':','_'); $log="$taskRoot\$variant-filter-$name.log"
  $p=Start-Process -FilePath "$taskRoot\$variant-test.exe" -ArgumentList @($filter,'--test-threads=4','--nocapture') -WorkingDirectory $working -RedirectStandardOutput $log -RedirectStandardError "$log.stderr" -PassThru -Wait
  $results+=@{variant=$variant;filter=$filter;exit_code=$p.ExitCode}
 }
}
$results|ConvertTo-Json|Set-Content "$taskRoot\density-groups.json" -Encoding utf8
