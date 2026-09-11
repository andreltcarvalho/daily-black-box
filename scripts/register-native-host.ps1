param([string]$HostExecutable = (Join-Path $PSScriptRoot '..\src-tauri\target\release\caixa-preta-native-host.exe'))
$ErrorActionPreference = 'Stop'
$executable = (Resolve-Path -LiteralPath $HostExecutable).Path
if ([IO.Path]::GetFileName($executable) -ne 'caixa-preta-native-host.exe') { throw 'Executável de ponte inválido.' }
$manifestPath = Join-Path ([IO.Path]::GetDirectoryName($executable)) 'caixa-preta-host.json'
$manifest = @{ name = 'local.caixapreta.dodia'; description = 'Ponte local do Caixa Preta do Dia'; path = $executable; type = 'stdio'; allowed_origins = @('chrome-extension://pjmccgpomaddaokgbjfmoidakooahmph/') }
$manifest | ConvertTo-Json | Set-Content -LiteralPath $manifestPath -Encoding utf8
$key = 'HKCU:\Software\Google\Chrome\NativeMessagingHosts\local.caixapreta.dodia'
New-Item -Path $key -Force | Out-Null
Set-Item -LiteralPath $key -Value $manifestPath
Write-Output "Ponte registrada por usuário: $manifestPath"
