# Validação do MVP Windows

Atualizado em 2026-09-11. Este registro distingue checks automatizados de comportamento real ainda pendente.

## Executado

| Check | Resultado |
| --- | --- |
| Probe Rust/MSVC | Executável compilou e executou |
| Metadados da janela Windows App | Processo do pacote Windows365, classe `TscShellContainerClass`; sem ler título/conteúdo |
| TypeScript frontend + extensão | Aprovado após o incremento v0.5.0 |
| ESLint | Aprovado após o incremento v0.5.0 |
| Vitest | 10 testes aprovados; inclui nomes de aplicativos na linha do tempo e no total por aplicativo |
| Cargo test | 17 testes aprovados; inclui migração v1→v3, identificação, agrupamento e reclassificação de aplicativos locais |
| Cargo check | Aprovado para todos os targets |
| Clippy | Aprovado com `-D warnings` após o incremento v0.5.0 |
| Builds web/extensão | Build web v0.5.0 aprovado; extensão não mudou neste incremento |
| Build release | App compilado após o incremento v0.5.0; host release aprovado anteriormente |
| NSIS | Pacote gerado na v0.3.2 antecede v0.4.1 e v0.5.0; regeneração do instalador atual permanece pendente |
| WebView2 real, banco de teste | 6 checks aprovados: IPC/status, dia vazio, início recusado sem VDI, Configurações, Histórico e captura visual |
| Categorização de aplicativos v0.5.0 | 17 testes Rust e 10 JS aprovados; migração v1→v3, reclassificação histórica e seleção “Entretenimento” para League of Legends cobertas |
| Typecheck, ESLint, build Vite e build release v0.5.0 | Aprovados |
| Layout 1280 e 800 px | Sem overflow horizontal em dados sintéticos; confirmação das capturas pós-contraste concluída na retomada v0.3.2 |
| Ponte nativa real | Timeout corrigido; 4 checks reais aprovados com binários debug: handshake/framing, named pipe, pausa e origem inválida. Extensão no Chrome real ainda pendente |
| Travamento v0.4.1 | Event Log confirmou Application Hang anterior; mutex liberado antes de chamadas de bandeja/UI. Coberto pelos testes e builds posteriores, mas abertura/fechamento repetidos ainda exigem ensaio Windows |

A sessão v0.3.1 foi interrompida a pedido do usuário; retomada registrada na especificação v0.3.2. Nenhuma validação real pendente deve ser considerada concluída por causa dos checks acima.

## Roteiro real ainda necessário

- [ ] Instalar a versão final por usuário e conferir registro do host e diretório da extensão.
- [ ] Carregar a extensão no Chrome; confirmar ID `pjmccgpomaddaokgbjfmoidakooahmph` e handshake pelo navegador real.
- [ ] Configurar a janela da sessão remota; distinguir launcher, login, conexão, desconexão e reconexão do Windows App.
- [ ] Cronometrar VDI em foco/ao fundo, Alt-Tab, tela cheia e dois monitores.
- [ ] Alternar entre VDI, League of Legends, ChatGPT e Chrome; conferir nomes, totais, trocas de contexto e ausência de título/caminho nos dados.
- [ ] Categorizar League of Legends e WhatsApp; conferir alteração retroativa em categorias e distrações, sem título ou caminho nos dados.
- [ ] Verificar entrada local encaminhada à VDI, limiar de inatividade, bloqueio/desbloqueio.
- [ ] Comparar duas abas, dois domínios, duas janelas, outro perfil e janela anônima; nenhum domínio antigo pode continuar indevidamente.
- [ ] Encerrar worker/host; confirmar corte no último estado válido e reconexão sem duplicação.
- [ ] Suspender, hibernar, reiniciar e encerrar processo forçadamente: não preencher lacunas com foco.
- [ ] Conferir bandeja, fechamento da janela e pausa persistida.
- [ ] Abrir e fechar o aplicativo repetidamente, incluindo pausar/retomar pela bandeja, e confirmar que a interface continua respondendo.
- [ ] Ativar/desativar autostart e verificar novo login/reinício; restaurar preferência escolhida.
- [ ] Exportar um dia e ler JSON; excluir um dia/toda atividade com confirmação, sem restaurar dados apagados por mensagens antigas.
- [ ] Executar instalado offline e verificar escalas Windows 100%, 125% e 150%.

## Evidência local de desenvolvimento

`.cache/ui/native-result.json`, `.cache/ui/browser-result.json` e screenshots em `.cache/ui/`. O banco `.cache/native-test-data/data.sqlite3` foi reservado aos testes; nenhuma coleta real foi iniciada. Esses artefatos não são distribuídos no aplicativo e não estão versionados.

Não tratar o instalador gerado nem a aprovação de unit tests como aprovação do roteiro real acima.

## Retomada v0.3.2

- Inspeção visual pós-contraste concluída nas duas capturas existentes. `browser-result.json` já registrava ausência de overflow e `contrast: []`; resultados conferidos, sem repetir a execução.
- Host e app release recompilados com sucesso (`cargo build --release --bins`). Frontend atualizado compilado com Node 24/Vite; o bundle anterior não incluía o CSS corrigido.
- Empacotamento do código atual aprovado (exit 0): `tauri build --config src-tauri/tauri.bundle.json --config .cache/bundle-current.json`, invocado com Node 24. Artefato: `src-tauri/target/release/bundle/nsis/Caixa Preta do Dia_0.1.0_x64-setup.exe`. Instalação ainda pendente.
- Chrome: conector indisponível; fallback computer-use abriu Chrome, mas o helper interrompeu a automação por não conseguir determinar a URL atual com confiança para aplicar a política. O handshake pelo Chrome não foi testado. Nenhuma extensão foi instalada nesta retomada.
- Nenhuma suite anterior foi reexecutada; nenhuma coleta real iniciada. Roteiro real acima continua aberto.

### Revisão dos aceites restantes

| Aceites | Evidência existente e parte ainda necessária |
| --- | --- |
| AC01–03 | Metadados e testes determinísticos preservados; sessão/launcher, foco cronometrado, idle e bloqueio reais pendentes |
| AC04–07 | Testes JS/protocolo e ponte direta preservados; Chrome real, perfis/anônimo, reconexão e inspeção dos artefatos de navegação pendentes |
| AC08–10 | Fixtures de totais/reclassificação/relógio e navegação WebView2 preservadas; histórico conhecido e edição pela UI ainda necessários |
| AC11–13 | Pausa na ponte e testes de exclusão/exportação preservados; reinício, exclusão durante coleta e exportação pelo diálogo real pendentes |
| AC14–17 | Implementação e testes de recuperação/transações preservados; instalação/offline, autostart/login, suspensão/crash real e erro visível de armazenamento pendentes |
| AC18 | Capturas 1280/800 pós-contraste confirmadas; teclado, dia denso/domínios longos e DPI 100/125/150% pendentes |
| AC19 | Migração, agrupamento, nomes e interface aprovados em testes Rust/React; alternância real entre VDI, League of Legends, ChatGPT e Chrome pendente |
| AC20 | Migração e reclassificação de aplicativos locais aprovadas em testes Rust/React; teste real de categorização de League of Legends/WhatsApp pendente |
