$ErrorActionPreference='Stop'
$env:PATH='D:\Program Files\Git\bin;'+[Environment]::GetEnvironmentVariable('Path','Machine')+';'+[Environment]::GetEnvironmentVariable('Path','User')
$env:PATHEXT=[Environment]::GetEnvironmentVariable('PATHEXT','Machine')
$taskRoot='F:\Git\pebrel\tmp\pr316-conflict-20261015'
$tests=@('gpui_shell::workspace::settings_navigation::ui_tests::density_changes_real_tab_bounds_and_sidebar_drag_pitch','gpui_shell::settings_pane::mobile::tests::mobile_three_states_and_manual_copy_use_the_rendered_controls','shell_detect::tests::wsl_prompt_command_executes_repeatedly_without_checkout_carriage_returns')
$results=@()
foreach($variant in @('merged','main')) {
 $working=if($variant -eq 'main'){"$taskRoot\main-source"}else{"$taskRoot\source"}
 foreach($test in $tests) {
  $name=$test.Split(':')[-1]
  $log="$taskRoot\$variant-$name.log"
  $process=Start-Process -FilePath "$taskRoot\$variant-test.exe" -ArgumentList @($test,'--exact','--nocapture','--test-threads=1') -WorkingDirectory $working -RedirectStandardOutput $log -RedirectStandardError "$log.stderr" -PassThru -Wait
  $results+=@{variant=$variant;test=$test;exit_code=$process.ExitCode}
 }
}
$results|ConvertTo-Json|Set-Content "$taskRoot\baseline-controls.json" -Encoding utf8
$results|ConvertTo-Json
