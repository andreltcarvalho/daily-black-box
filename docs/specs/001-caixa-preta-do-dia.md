# Caixa Preta do Dia — especificação e plano de implementação

Status: **limites dos períodos do dia, maior lacuna interna e faixa visual de estados implementados, preservando agrupamento e filtros; validações reais do MVP ainda pendentes**.
Versão: **0.7.0**. Data: **2026-09-11**.
Workspace: `C:\Users\Usuario\Documents\ChatGPT\My-Apps`.

## Incremento v0.7.0 — primeiro/último período e estados da linha do tempo

Escopo implementado: somente os itens 7 e 8 autorizados. Um resumo integrado à linha do tempo apresenta início do primeiro período, fim confirmado do último e horário/duração da maior lacuna interna. Uma faixa compacta abaixo das categorias distingue VDI, navegador, aplicativo local, sistema, inatividade, pausa, desconhecido e não observado com padrões/bordas e legenda textual. A lista bruta inclui a coluna Estado; cada trecho da faixa possui nome acessível com origem, horários e duração.

Regras da derivação:

- Usar `report.sessions` na ordem observada entregue pelo Rust (`ORDER BY id`), sem ordenar pelo relógio civil nem usar blocos agrupados. Resumo independente dos filtros; incluir limites de registros recém-abertos com duração zero.
- Primeiro início e último fim usam `start_utc` e `end_utc`, com o `offset_seconds` do respectivo registro. Sessão aberta mostra “Em andamento · último instante confirmado”, sem consultar o relógio atual ou estimar duração.
- Lacuna é a diferença positiva entre o fim de um registro bruto e o início do seguinte. Cada extremidade usa seu próprio offset. Em empate, manter a primeira maior lacuna. Não considerar o tempo anterior ao primeiro registro nem posterior ao último; transições marcadas `clock_adjusted` não são evidência de lacuna. Pausa e `unobserved` já registrados têm estados próprios e não criam uma segunda lacuna sobre a mesma duração.
- Dia vazio mostra “Não disponível”; um registro mostra primeiro/último e “Sem lacuna interna”. Se ainda não houver duração confirmada, manter o resumo e explicar essa condição na área vazia.
- A faixa de estados representa as mesmas sessões uma única vez, com largura proporcional, sem largura mínima que estenda períodos curtos. As lacunas derivadas só aparecem em “Tudo”; filtrar não transforma registros ocultos em ausência de coleta. O agrupamento visual da v0.6.0 e o filtro da lista permanecem preservados.
- Sem mudanças em coleta, categorias, totais, gráficos, exportação, comandos Rust, contrato de relatório ou SQLite. Sem novas dependências, migração ou instalador.

Arquivos alterados neste incremento: `src/timeline.ts`, `src/timeline.test.ts`, `src/components/Dashboard.tsx`, `src/components/Dashboard.test.tsx`, `src/styles.css`, esta especificação e `docs/validation/windows-mvp.md`. `src/types.ts`, `src/App.tsx` e relatórios/gravador Rust foram inspecionados e não precisaram mudar. Alterações locais preexistentes da v0.6.0 foram preservadas.

Validação executada em 2026-09-11, com Node 24.19.0:

| Comando/check | Resultado |
| --- | --- |
| `npm test -- --run` | 31 testes aprovados em 4 arquivos; 18 casos acrescentados neste incremento |
| `npm run typecheck` | Aprovado para frontend e extensão |
| `npm run lint` | Aprovado |
| `npm run build` | Aprovado; Vite 7.3.6, 37 módulos |
| Interface web com dados sintéticos | 1280/800 px com oito estados, lacuna, domínio longo e aplicativo local; 800 px com 500 registros, um período e dia vazio. Sem overflow horizontal nem erros de página; filtro acionado por teclado; trechos de estados sem sobreposição nos cenários mistos |

Testes cobrem primeiro/último em vários registros, vazio, um registro, ausência de lacuna em consecutivos, maior lacuna em três períodos, offsets distintos, sessão aberta e duração zero, lacuna preservada apesar de agrupamento, ajustes de relógio, estados/legenda/texto acessível, domínio longo, dia denso, filtro, totais e callback de exportação preservados, sem mutação do relatório. O eixo termina no limite confirmado mesmo quando o último registro ainda tem duração zero.

Evidência visual local: `.cache/states-ui.mjs`, `.cache/ui/states-v070-result.json` e `.cache/ui/states-v070-*.png`; capturas de cenários mistos/denso conferidas. Artefatos sintéticos de desenvolvimento, sem dados de atividade real. A primeira tentativa de testes encontrou restrições de leitura do sandbox/Node 20; os comandos finais passaram fora do sandbox com Node 24. A primeira navegação sintética excedeu a espera por `load`; a execução posterior aguardou `domcontentloaded` e concluiu todos os cenários.

Pendências manuais: conferir leitura dos padrões, leitor de tela, cores forçadas, teclado e DPI 100/125/150% no WebView2 real, com dia conhecido, sessão aberta e ajustes de relógio/fuso. Em 2026-09-14, a build release foi iniciada como processo Windows, mas o controlador de UI retornou `apps: []` e não expôs a janela nativa; portanto nenhum desses cenários foi validado. Nenhum ensaio final de coleta no Windows foi executado. Cargo não foi reexecutado, pois Rust e o contrato não mudaram. Instalador não foi gerado; demais aceites reais continuam pendentes em `docs/validation/windows-mvp.md`.

## Incremento v0.6.0 — leitura da linha do tempo

Escopo aprovado: reduzir o ruído visual da linha do tempo e permitir filtrar as faixas por categoria.

Implementação: sessões SQLite continuam íntegras e a lista mantém cada registro bruto. Somente a visualização agrupa uma interrupção de até cinco segundos de Sistema ou Desconhecido quando ela estiver entre duas faixas da mesma categoria, sem lacuna superior a um segundo. A faixa agrupada informa essa condição no detalhe. Os filtros “Tudo” e por categoria atuam na linha do tempo e na lista de registros brutos filtrada, sem alterar totais, cartões, gráficos ou exportação.

Validação: suíte Vitest com 13 testes aprovada, incluindo agrupamento permitido, preservação quando as categorias diferem, edição de categoria, equivalentes textuais e filtro. Typecheck, ESLint e build Vite aprovados. A inspeção manual de um dia denso continua pendente em AC18.

## Incremento v0.4.0 — aplicativos locais em foco

Escopo aprovado: quando o foco estiver fora da VDI, registrar e exibir somente o nome amigável derivado do executável do aplicativo local. Não registrar título da janela, URL, conteúdo, PID nem caminho completo. O domínio continua sendo a identidade principal quando a extensão confirma uma aba do Chrome; o relatório também pode atribuir esse período ao aplicativo “Google Chrome”.

Plano executado: evento e sessão ampliados com `app_name` opcional; migrações v1→v3 preservam sessões; aplicativos identificados entram inicialmente em trabalho local e podem ser categorizados localmente como os sites; o relatório recalcula histórico, categorias e distrações a partir dessa escolha; estado atual, linha do tempo, lista e gráfico exibem o nome. A validação real deve confirmar alternância entre VDI, League of Legends, ChatGPT e Chrome sem capturar texto da janela.

Evidências: 17 testes Rust e 10 testes JS aprovados na v0.5.0, incluindo migração v1→v3, reclassificação histórica de League of Legends e edição da categoria na interface; `cargo fmt --check`, Clippy com `-D warnings`, typecheck, ESLint, build Vite e build release aprovados. O ensaio AC19/AC20 no Windows permanece pendente e não foi substituído pelos testes automatizados.

## Correção v0.4.1 — travamento ao abrir

Evidência: o Event Log do Windows registrou `Application Hang` (evento 1002) para `caixa-preta-do-dia.exe` em 2026-09-10 16:00:12.

Causa identificada: o loop de observação mantinha o mutex do runtime enquanto atualizava o tooltip da bandeja. A atualização pode precisar da thread de interface, que simultaneamente pode estar esperando o mesmo mutex para responder a uma chamada IPC. O callback de pausa da bandeja tinha o mesmo risco ao mostrar ou focar a janela.

Correção aplicada: o runtime calcula o texto da bandeja sob o mutex e o libera antes de chamar Tauri; o callback de pausa também libera o mutex antes de abrir ou focar a janela. A alteração não modifica os dados coletados, o esquema SQLite nem as regras de medição.

Limite da evidência: os testes e builds posteriores aprovam a correção compilada, mas abrir/fechar repetidamente a aplicação no Windows continua como validação manual pendente.

## Incremento v0.5.0 — categorias para aplicativos locais

Escopo aprovado: aplicativos locais identificados, como League of Legends e WhatsApp, podem receber as mesmas categorias usadas por sites. Eles começam em “Trabalho local” até que o usuário faça uma escolha.

Implementação: a migração v3 cria `app_categories`, com nome amigável do aplicativo, categoria e instante da alteração. O comando local `classify_app` grava a escolha; o relatório resolve a categoria ao consultar o dia, portanto histórico, linha do tempo, totais por categoria, distrações e maior distração são recalculados sem alterar sessões já registradas. A tabela “Aplicativos locais e categorias” permite editar a escolha. A VDI permanece uma categoria reservada e não pode ser atribuída a aplicativos locais.

Privacidade: a chave armazenada é apenas o nome amigável do aplicativo; título da janela, conteúdo, PID, caminho do executável e URL não são armazenados.

## Retomada v0.3.2 — 2026-09-10

Plano aprovado preservado; sem nova entrevista, planejamento ou repetição das suites concluídas. O checkpoint v0.3.1 abaixo é histórico; esta seção registra as novas evidências.

- Confirmada a inspeção das capturas pós-contraste `.cache/ui/dashboard-1280.png` e `dashboard-800.png`, geradas depois de `src/styles.css`. Layout legível e sem cortes; o resultado já salvo em `browser-result.json` contém `contrast: []`. Não foi executada novamente a validação anterior. AC18 ainda exige teclado, dia denso e DPI reais.
- `cargo build --manifest-path src-tauri/Cargo.toml --release --bins` aprovado: app e host release regenerados com a correção da ponte.
- Frontend de distribuição estava anterior ao ajuste de contraste; build Vite com Node 24 aprovado para atualizar o pacote. A primeira tentativa via npm global usou Node 20 e falhou com EPERM; Vite direto no sandbox falhou no esbuild por acesso negado. A execução autorizada fora do sandbox passou.
- Empacotamento NSIS com frontend atualizado aprovado (exit 0): `src-tauri/target/release/bundle/nsis/Caixa Preta do Dia_0.1.0_x64-setup.exe`. O pacote inclui a ponte release corrigida; instalação e execução instalada ainda não validadas. Usado override local `.cache/bundle-current.json` para não repetir o build web já concluído.
- Chrome real ainda não validado. O conector respondeu `Browser is not available: chrome`. Chrome foi aberto pela skill computer-use, mas o controle foi interrompido pelo helper: não conseguiu determinar a URL atual com confiança para aplicar a política. Nenhuma extensão foi instalada, nenhuma coleta real iniciada e nenhuma política alterada nesta retomada.
- Suites anteriores (9 JS, 14 Rust, typecheck/lint/Clippy e quatro checks diretos da ponte) preservadas, sem reexecução. Não houve mudança de comportamento de código que exigisse novos testes.

Pendências: instalação por usuário e conteúdo instalado; extensão/handshake Chrome e duas janelas/perfis; reconhecimento completo Windows App; foco/idle/bloqueio/energia; bandeja e pausa no reinício; exportação/exclusão pela UI; autostart/login; execução offline, teclado e DPI. Consultar `docs/validation/windows-mvp.md`. F0–F6 permanecem abertas até os respectivos aceites reais; não confundir geração do pacote com validação instalada.

## Checkpoint histórico v0.3.1 — preservado para referência

O usuário aprovou todas as premissas e a implementação. Não repetir a entrevista ou o planejamento. Continuar no **mesmo diretório local My-Apps**: ainda não há commits, portanto um worktree novo não receberá automaticamente estes arquivos.

Código presente: dashboard React, núcleo Rust/SQLite, observação Windows App por `WindowsApp/msrdc.exe` + `TscShellContainerClass`, extensão Chrome MV3, ponte Native Messaging/named pipe, configurações, pausa, categorias, exportação, exclusão, bandeja, autostart e instalador NSIS. Nenhuma coleta foi ativada com dados reais do usuário.

Evidências já obtidas: typecheck frontend/extensão, lint, **9 testes JS e 14 testes Rust aprovados na última execução**, cargo check, Clippy com `-D warnings`, build frontend/extensão/release e geração de instalador. Seis checks na WebView2 real confirmaram IPC, banco isolado vazio, bloqueio de início sem VDI, Configurações e Histórico. Inspeção visual com dados sintéticos não apresentou overflow em 1280/800 px; cores de textos abaixo de 4,5:1 foram corrigidas. A última tentativa de confirmação visual não foi conferida antes do pedido de parada; não a tratar como aprovada.

**Ponte corrigida e verificada:** o timeout foi resolvido removendo flush do framing genérico e mantendo flush somente em stdout do native host. App e host de debug foram recompilados; **4 checks reais passaram**: framing/handshake, named pipe do usuário, estado pausado e rejeição de origem inválida. O roteiro reproduzível está em `scripts/validate-native-host.mjs` e requer o app de teste aberto. Isso não valida ainda a extensão carregada no Chrome real. **O instalador gerado antecede a correção: não entregar como pacote final sem regenerá-lo.**

**Próximos passos, somente quando o usuário solicitar na nova sessão:** confirmar a inspeção visual após ajustes de contraste; revisar os aceites ainda não cobertos; regenerar host release e instalador com o código atual; validar extensão no Chrome e o roteiro Windows em `docs/validation/windows-mvp.md`. Os últimos 14 testes Rust já incluem exportação e relógio; não repetir toda exploração inicial. Não declarar F0–F6 concluídas enquanto os aceites reais estiverem pendentes.

**Resumo das fases:** F0 — toolchain/app abrem, reconhecimento de metadados observado, ensaio completo VDI/launcher pendente; F1 — núcleo e banco implementados e testes atuais aprovados; F2 — coletor implementado, foco/bloqueio/suspensão reais pendentes; F3 — extensão compila e ponte passa integração direta, Chrome real pendente; F4 — dashboard implementado e testes aprovados, confirmação visual final pendente; F5 — controles/instalador implementados, pacote precisa ser regenerado e instalação/autostart validados; F6 — não concluída. As linhas de fases abaixo descrevem os gates originais; este checkpoint informa o andamento efetivo.

**Encerramento desta sessão:** o usuário pediu explicitamente parar para economizar tokens e retomar em outra sessão. Não iniciar mais builds, testes ou implementação nesta sessão. Os apps nativos de teste foram encerrados antes da última suite Rust. Não foram feitos commits nem instalada a extensão no perfil pessoal do Chrome.

Ambiente: Node 24.19 em `C:\Users\Usuario\.cache\codex-runtimes\codex-primary-runtime\dependencies\node\bin\node.exe`; npm CLI em `C:\Program Files\nodejs\node_modules\npm\bin\npm-cli.js`. Invocar Node explicitamente e usar `.cache/npm`; o npm global pode resolver Node 20.15. Esbuild/TLS no sandbox falharam antes de executar; execução aprovada fora do sandbox permitiu validar. Rust/MSVC e WebView2 funcionam. Arquivos de teste nativo estão em `.cache/`, com SQLite separado e sem atividade real; não os confundir com dados do produto.

Detalhes finais diferem dos nomes apenas propostos: `runtime.rs` coordena a coleta; `windows_tracker.rs` contém os bindings; `sessions.rs` calcula os relatórios e `store.rs` contém o gravador; a UI agrupa gráficos/linha do tempo/tabela em `Dashboard.tsx`. `windows-sys` substitui o wrapper `windows` sobre as mesmas APIs. A duração monotônica é persistida como duração acumulada; a ordem é o ID da sessão. `npm run package:windows` gera o pacote completo em duas etapas, pois a ponte deve existir antes de ser adicionada aos recursos do instalador.

## 1. Objetivo e limite desta entrega

Construir um aplicativo local para Windows que responda: **“Onde meu tempo realmente esteve hoje?”**. Observar foco na VDI pelo Windows App, atividade no navegador local e inatividade, apresentando uma distribuição temporal descritiva, sem avaliar produtividade.

A entrega inicial foi somente documental. Em 2026-09-10 o usuário aprovou explicitamente o plano e suas premissas e solicitou implementação, validações e atualização deste documento. F0–F6 estão autorizadas dentro do escopo descrito.

### Método spec driven development

Este documento será a fonte de verdade do MVP. A implementação seguirá a sequência **requisito → critério de aceite → teste → implementação → evidência**, por fase. Os identificadores `RF`, `RNF`, `AC` e `F` permitem acompanhar essa relação sem criar um sistema de gestão separado.

Antes de alterar comportamento acordado, atualizar a seção correspondente e registrar a mudança no changelog ao final deste arquivo. Mudanças materiais de escopo, contrato ou produto exigem nova escolha do usuário. Ao concluir cada fase, atualizar seu status, os arquivos efetivamente alterados e as validações executadas; não substituir resultados anteriores nem marcar pendências como concluídas.

## 2. Evidências da inspeção

| Item | Resultado observado nesta sessão | Consequência |
| --- | --- | --- |
| Repositório | Apenas `.git`, branch `master`, nenhum commit e nenhum arquivo de aplicação | Não existe stack, estrutura ou convenção de código a preservar |
| Instruções | AGENTS.md fornecido na conversa; nenhum arquivo AGENTS.md encontrado no workspace ou nos ancestrais consultados | Escopo mínimo; plano antes de código; testes relevantes e relato honesto |
| Material de entrada | `pasted-text.txt` com a proposta completa | Requisitos incorporados neste plano |
| Imagem citada no texto | Não disponível entre os arquivos deste anexo | Direção visual baseada nas instruções textuais; nenhuma imagem foi presumida ou analisada |
| Cliente VDI | Usuário confirmou **Windows App** | Esse é o cliente alvo; não implementar catálogo de clientes |
| Processo VDI | `msrdc.exe` em `WindowsApps\MicrosoftCorporationII.Windows365_2.0.1375.0_x64__8wekyb3d8bbwe\msrdc\` | Evidência de que o processo observado pertence ao pacote do Windows App; não fixar caminho com versão |
| Navegadores instalados | Chrome 153.0.8010.36, Edge 134.0.3124.83 e Firefox 138.0.4, conforme registro local | Chrome é a proposta para o MVP; instalação não demonstra preferência do usuário |
| Navegador padrão | Consulta não retornou um ProgId | Não afirmar qual é o padrão |
| Node | `v20.15.0` | Abaixo do mínimo documentado pelo Vite atual; preparar runtime compatível antes de desenvolver |
| Rust | Cargo 1.98.0 e rustc 1.98.0 disponíveis; ambos emitiram aviso de canonicalização do diretório do usuário | Presença verificada, compilação ainda não validada |
| Windows build | Registro lista Build Tools 2017 e WebView2 Runtime | Presença não prova toolchain C++/SDK compatível; verificar na F0 |
| .NET | Executável disponível; `dotnet --list-sdks` não listou SDKs | Nenhuma aplicação ou base .NET encontrada para reaproveitar |
| Inspeção Appx | `Get-AppxPackage` falhou ao carregar o módulo nesta plataforma de execução | Identificação apoiada na confirmação do usuário e no caminho do processo, não em inventário Appx completo |
| Ferramentas de design | Leitura das diretrizes Impeccable; launcher de contexto não executou por ausência de engine/cache gravável | Planejamento feito com o briefing disponível; sem gerar artefatos visuais ou instalar engine |

O Vite documenta Node 20.19+ ou 22.12+; a versão a usar será fixada junto das dependências na F0. O Tauri requer ferramentas C++ e WebView2 no Windows. [Requisitos do Vite](https://vite.dev/guide/), [pré-requisitos do Tauri](https://v2.tauri.app/start/prerequisites/).

## 3. Decisões e premissas

| ID | Decisão | Situação |
| --- | --- | --- |
| D01 | Produto Windows local, sem IA, login, nuvem ou processamento externo | Confirmado pelo pedido |
| D02 | VDI acessada pelo **Windows App**, observada pelo computador local | Confirmado pelo usuário |
| D03 | Começar com **Google Chrome**, extensão Manifest V3, um perfil configurado | Proposta baseada no ambiente; não houve confirmação de preferência |
| D04 | Nenhuma categoria conta como “distração” inicialmente; usuário marca as categorias desejadas | Proposta conservadora, sem julgamento automático |
| D05 | Tauri 2 + React + TypeScript; Rust para integração Windows e dados; SQLite | Recomendação para aprovação |
| D06 | Native Messaging com ponte pequena e named pipe local; nenhum servidor HTTP | Recomendação para aprovação |
| D07 | Inatividade após 5 minutos sem entrada; contagem de inatividade começa ao atingir o limiar, sem retroagir | Padrão proposto, editável em Configurações |
| D08 | Histórico por dia; reclassificar domínio atualiza também a classificação dos dias anteriores | Semântica proposta, informada na edição |
| D09 | Inicialização automática desativada inicialmente; fechar janela mantém coleta na bandeja; “Sair” encerra | Comportamento proposto e explicado na primeira execução |
| D10 | Fora da VDI, guardar somente nome amigável derivado do executável; nunca título, URL, conteúdo, PID ou caminho | Confirmado pelo usuário em 2026-09-10 |

Foram apresentadas duas perguntas: navegador/cliente e definição de distração. O usuário esclareceu o cliente e posteriormente aprovou o plano e todas as premissas D03–D09, incluindo Chrome e nenhuma categoria de distração pré-selecionada. As situações na tabela acima registram a origem das propostas; todas estão agora aprovadas.

## 4. Escopo funcional

| ID | Requisito |
| --- | --- |
| RF01 | Configurar uma vez a janela da sessão VDI do Windows App e mostrar diagnóstico de reconhecimento |
| RF02 | Medir VDI somente em primeiro plano, descontando inatividade, bloqueio, suspensão e pausa |
| RF03 | Registrar domínio e identidade temporária da aba ativa do Chrome, seus intervalos e acessos efetivos |
| RF04 | Produzir uma única linha do tempo, sem contagem simultânea de VDI e domínio |
| RF05 | Abrir em Hoje, com linha do tempo, resumos, distribuições e tabela por domínio/categoria |
| RF06 | Consultar dias anteriores com a mesma tela e as mesmas regras de cálculo |
| RF07 | Classificar domínios localmente; manter os desconhecidos sem inferência automática |
| RF08 | Pausar/retomar manualmente e indicar coleta ativa, pausada ou com erro na janela e bandeja |
| RF09 | Excluir um dia ou todos os dados de atividade e exportar um dia localmente |
| RF10 | Oferecer início automático no login do Windows, se a integração simples for validada |
| RF11 | Recuperar de encerramento inesperado sem inventar tempo entre observações |
| RF12 | Registrar o aplicativo local em foco fora da VDI e apresentar seu tempo agregado, sem conteúdo da janela |

Categorias iniciais: VDI / trabalho, trabalho local, redes sociais, vídeo, entretenimento, pesquisa, comunicação, sistema e desconhecido. **Inatividade é um estado de medição**, não uma categoria de site. Pausa e ausência de coleta são lacunas explicadas separadamente.

O rótulo de VDI será “VDI / trabalho focado”, acompanhado de explicação curta: tempo com a janela em foco, sem avaliação de produtividade. Aplicativos locais identificados entram inicialmente em “Trabalho local”, aparecem pelo nome e podem receber qualquer categoria não-VDI na tabela de aplicativos. A escolha é guardada somente neste computador e recalcula os dias anteriores; aplicativos sem escolha continuam em “Trabalho local”. Janelas cujo executável não puder ser identificado permanecem desconhecidas.

### Fora do MVP

IA, APIs de IA, login, nuvem, banco remoto, telemetria externa, análise dentro da VDI, captura de tela, keylogging, áudio, leitura de páginas, conteúdo de mensagens, calendário, celular, bloqueio de sites, metas, gamificação, notificações recorrentes, recomendações, comparações semanais/mensais, suporte a outros navegadores, múltiplos perfis simultâneos, sincronização e importação de histórico do navegador. O histórico foi permitido como complemento, mas não é necessário à primeira versão.

### Requisitos não funcionais

- **RNF01 — Privacidade:** o produto funciona offline e armazena somente os metadados definidos abaixo. Recursos da UI, fontes e ícones são empacotados; nenhum favicon ou fonte é buscado na internet.
- **RNF02 — Exatidão honesta:** distinguir tempo observado, inativo, pausado e não observado. Não apresentar presença da janela como produtividade nem presença do navegador como tempo de site.
- **RNF03 — Integração mínima:** um processo principal por usuário/sessão Windows e uma ponte do navegador; sem serviço Windows, contêiner, servidor ou ORM.
- **RNF04 — Acessibilidade:** navegação por teclado, foco visível, contraste adequado, estados com texto, gráficos acompanhados de valores/tabela e funcionamento em escalas Windows de 100%, 125% e 150%.
- **RNF05 — Desempenho:** coleta independente da renderização, consultas restritas ao dia, escritas em transições/checkpoints; nada de gravar uma linha a cada segundo em repouso.
- **RNF06 — Persistência:** transações, recuperação testada e diagnóstico claro quando não é possível salvar. Não continuar exibindo “gravando” após erro persistente no banco.

## 5. Arquitetura do aplicativo

**Tauri 2** fornece janela, bandeja e empacotamento Windows; **React + TypeScript** compõem a interface; o núcleo **Rust** é a única autoridade para sessões, cálculos, configurações e acesso ao SQLite.

```text
Windows: foco, última entrada, bloqueio e energia
                     │
                     ▼
Chrome → extensão → ponte nativa → núcleo Rust → SQLite local
                                      ↕
                             interface Tauri/React
```

O núcleo mantém uma fila sequencial de observações e uma máquina de estados pequena. A interface solicita o resumo de um dia e recebe atualizações de estado; não mantém um segundo contador independente. Fechar a janela não interrompe o núcleo; a bandeja oferece Abrir, Pausar/Retomar e Sair. Uma segunda abertura reutiliza a instância existente para impedir dois escritores e contagem duplicada.

A escolha é uma recomendação para um repositório vazio: permite dashboard com HTML/CSS e acesso direto às APIs Windows. Não há código Tauri existente aqui. A F0 deve comprovar a toolchain antes de avançar; se houver bloqueio estrutural, registrar a evidência e discutir mudança de stack, sem implementar duas alternativas.

## 6. Arquitetura da extensão

Uma extensão Chrome Manifest V3 pequena, em TypeScript, sem framework próprio de extensões, content scripts ou código injetado em páginas.

- Ouvir `tabs.onActivated`, alterações de URL da aba em `tabs.onUpdated`, `tabs.onRemoved`, `tabs.onReplaced`, `windows.onFocusChanged` e fechamento de janelas.
- Na conexão, reinício do worker ou reconciliação, consultar novamente a janela **efetivamente focada** e sua aba ativa; não confiar apenas na última janela usada.
- Converter URL de HTTP/HTTPS imediatamente em `hostname`; nunca transmitir ou persistir caminho, parâmetros, fragmentos, título, texto ou URL completa. A URL fornecida pela API existe apenas transitoriamente para essa extração.
- Usar hostname exato em minúsculas, sem ponto final; preservar subdomínios e não aplicar heurísticas como remover `www` ou usar apenas as duas últimas partes. Hosts IDN usam forma normalizada consistente.
- Páginas internas do navegador entram como sistema, sem endereço; esquemas não suportados e URLs indisponíveis entram como desconhecido, sem extrair caminhos locais.
- Não coletar navegação anônima. Ausência de cobertura fica visível; nunca reaproveitar o domínio da janela normal para uma janela anônima.
- `tabId` e `windowId` identificam a aba/janela apenas durante a conexão do navegador; não são identidade permanente e não são `HWND` do Windows.
- Popup mínimo da extensão: conectado/desconectado/pausado, explicação das permissões e tentativa de reconexão. A análise fica no aplicativo.

Permissões propostas: `tabs` para obter domínio sem gesto a cada troca; `nativeMessaging` para a ponte; `alarms` para tentativas moderadas de reconexão. Sem `history`, `scripting`, `webRequest`, permissões gerais de hosts ou `storage.sync`. A permissão `activeTab` isolada não atende coleta automática contínua, pois depende de ativação pelo usuário. Não é necessária uma permissão chamada `windows` para observar suas janelas. [API de abas](https://developer.chrome.com/docs/extensions/reference/api/tabs), [API de janelas](https://developer.chrome.com/docs/extensions/reference/api/windows).

O worker pode ser encerrado. Ao voltar, faz handshake e novo snapshot, sem prolongar a aba antiga. A conexão nativa oferece manutenção do worker, mas desconexão e reinício continuam tratados explicitamente. [Ciclo de vida do worker](https://developer.chrome.com/docs/extensions/develop/concepts/service-workers/lifecycle).

## 7. Comunicação extensão → aplicativo

Usar `runtime.connectNative()` com um executável de ponte em Rust, compilado pelo mesmo pacote Cargo do app. A ponte traduz o canal binário JSON do navegador para um named pipe do núcleo; ela não decide duração nem grava o banco.

O host é registrado por usuário em HKCU, com manifesto que aceita somente o ID exato da extensão. O instalador configura e remove apenas a chave pertencente ao produto. Na etapa de desenvolvimento, instalação local da extensão e registro do host serão documentados; estabilizar o ID antes de gerar o manifesto final. Não publicar em loja nesta etapa. [Native Messaging](https://developer.chrome.com/docs/extensions/develop/concepts/native-messaging).

O named pipe deve ser local, rejeitar clientes remotos e restringir acesso ao SID do usuário/sessão esperada. Não confiar na ACL padrão. Validação da origem da extensão, versão de protocolo e tamanho/tipos das mensagens complementa a restrição de acesso. Isso não promete proteção contra processos maliciosos já executados como o mesmo usuário. [Segurança de named pipes](https://learn.microsoft.com/en-us/windows/win32/ipc/named-pipe-security-and-access-rights).

Contrato mínimo:

| Mensagem | Conteúdo e uso |
| --- | --- |
| `hello` / `hello_ack` | Versão, identificação efêmera da conexão e estado ativo/pausado do núcleo |
| `browser_state` | Sequência, instante observado, janela focada, tabId, windowId, hostname ou motivo da ausência; sem título/URL |
| `snapshot_request` | Núcleo solicita estado atual ao detectar foco no Chrome ou restabelecer conexão |
| `tracking_state` | Núcleo comunica pausa, retomada ou indisponibilidade de gravação |

O núcleo atribui o instante de recebimento em relógio monotônico e UTC; timestamp da extensão é diagnóstico, não autoridade para reescrever o passado. Sequências repetidas/antigas são ignoradas. Consultas assíncronas da extensão descartam resultados de uma geração de foco anterior.

Enquanto Chrome estiver em primeiro plano, confirmar a validade do estado aproximadamente a cada 5 segundos; após 10 segundos sem confirmação, encerrar a atribuição no último instante confirmado e mostrar o trecho seguinte como desconhecido. Esses limites serão verificados na F3. Confirmar também a identidade de foco do Windows; um snapshot não pode reabrir um site se a VDI já ganhou foco.

Sem conexão, a extensão não acumula um histórico independente. Na reconexão envia somente o estado atual. Com o app encerrado, não inicia coleta escondida; a ponte retorna indisponibilidade. Repetir conexão com atraso limitado via alarmes, sem loop apertado. Enquanto pausado, não consultar nem transmitir domínios; na retomada descartar estado antigo e consultar novamente.

## 8. Detecção de foco da VDI / Windows App

1. Na primeira execução, o usuário abre sua sessão remota e aciona “Identificar minha VDI”. Após uma contagem curta, coloca essa janela em primeiro plano.
2. Obter `HWND`, PID, identidade do executável/pacote e classe da janela. O título não entra no histórico. Um seletor textual restrito só será proposto se realmente necessário para distinguir a janela da sessão.
3. Validar reconhecimento com o usuário: alternar entre sessão, launcher e outro aplicativo. O estado visível deve acompanhar a alternância.
4. Salvar identidade estável do aplicativo e o discriminador mínimo de janela; não persistir PID/HWND como identificadores duráveis nem fixar o diretório versionado do pacote.
5. Observar `EVENT_SYSTEM_FOREGROUND` com `SetWinEventHook`, fora do contexto do processo alvo, em thread com message loop. Usar `GetForegroundWindow` na inicialização e reconciliação de aproximadamente 1 segundo; esta é proteção contra perda de evento, não a única forma de observar trocas.
6. Só abrir sessão VDI quando a janela reconhecida for a janela em primeiro plano, com coleta habilitada e usuário ativo. Outra janela em foco encerra a sessão anterior.

As APIs identificam foco e requerem tratamento de janela nula e do ciclo de mensagens. [GetForegroundWindow](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getforegroundwindow), [SetWinEventHook](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwineventhook).

**Validação obrigatória:** janela remota versus tela de conexões/login, modo janela/tela cheia, dois monitores, minimizar, desconectar e reabrir sessão, e atualização do Windows App. Apenas detectar `msrdc.exe` não basta para prometer essa distinção. Se uma janela de erro/desconexão conservar metadados indistinguíveis da sessão, documentar o limite; não analisar pixels ou conteúdo remoto para contorná-lo.

A VDI aberta em outro monitor ou ao fundo recebe zero tempo de foco. Não se tenta descobrir o aplicativo remoto usado, nem se a pessoa está produzindo algo. O próprio Windows App pode usar rede para a VDI; o Caixa Preta do Dia não se conecta ao serviço remoto. [Finalidade do Windows App](https://learn.microsoft.com/en-us/windows-app/get-started-connect-devices-desktops-apps).

## 9. Inatividade, bloqueio e pausa

Consultar `GetLastInputInfo` aproximadamente a cada segundo na sessão local do usuário. A API fornece o instante da última entrada, não conteúdo de teclas ou cliques; sua informação é específica da sessão que a consulta. Tratar retorno inválido, wrap do contador e valores não monotônicos sem gerar duração negativa. [GetLastInputInfo](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getlastinputinfo).

Padrão: aos 5 minutos sem entrada, encerrar foco no instante `última entrada + limiar` e iniciar inatividade. Os primeiros 5 minutos permanecem atribuídos à janela focada. Ao voltar a haver entrada, encerrar inatividade e obter um novo estado de foco. Não reclassificar retroativamente todos os minutos sem entrada. Mudanças no limiar afetam apenas observações futuras e reiniciam a avaliação a partir da alteração.

Bloqueio de sessão inicia inatividade imediatamente. Suspensão interrompe a cobertura: seu período é “sem coleta — computador suspenso”, não tempo focado nem inatividade medida. Pausa manual deixa uma faixa “rastreamento pausado”, sem capturar domínios. Retomar nunca prolonga a sessão anterior.

Ouvir notificações de sessão Windows e energia, além de verificar continuidade dos instantes observados. [Notificações de sessão](https://learn.microsoft.com/en-us/windows/win32/api/wtsapi32/nf-wtsapi32-wtsregistersessionnotification), [notificações de energia](https://learn.microsoft.com/en-us/windows/win32/power/wm-powerbroadcast).

Limite conhecido: leitura longa ou vídeo sem interação pode alcançar o limiar; inatividade significa ausência de entrada local, não ausência comprovada da pessoa. O teste real deve verificar que teclado/mouse encaminhados ao Windows App atualizam a última entrada local como esperado.

## 10. Modelo de eventos e sessões

### Observações de entrada

Não criar event sourcing nem guardar um log permanente de cada tick. Eventos entram na máquina de estados; persistir as sessões resultantes, acessos e o checkpoint necessário à recuperação.

| Campo do evento | Regra |
| --- | --- |
| `run_id`, `sequence` | Identificam execução e ordem; sequências do navegador são isoladas por conexão |
| `kind` | `foreground_changed`, `browser_state`, `idle_changed`, `session_lock`, `session_unlock`, `suspend`, `resume`, `pause`, `tracking_resume`, `checkpoint`, `shutdown` |
| `observed_at_utc`, `monotonic_ms` | Âncora civil e ordem/duração observada no núcleo |
| `source` | Windows, navegador ou ação do usuário |
| `payload` | Apenas os campos necessários: alvo reconhecido, hostname, IDs temporários ou estado; nunca URL/título/conteúdo |

### Máquina de estados

Precedência: **sem coleta/pausado → bloqueado/inativo → VDI reconhecida → domínio válido do navegador em foco → aplicativo local identificado → sistema reconhecido → desconhecido**. Categorias são aplicadas depois da identificação da origem; não decidem qual janela estava em foco.

- Fechar o intervalo anterior e abrir o próximo na mesma fronteira quando muda o estado efetivo.
- Usar intervalos semiabertos `[início, fim)`: cada instante pertence a no máximo um estado da sequência de observação.
- Não criar novos blocos por heartbeat ou eventos duplicados; registrar apenas transições reais e checkpoints.
- Unir visualmente blocos adjacentes de mesma origem/domínio, sem cruzar inatividade, pausa, falta de dados ou mudança real de contexto. A tabela de acessos preserva entradas em abas distintas do mesmo domínio.
- Quando Chrome ganha foco, começar como desconhecido até receber snapshot válido para a nova geração de foco. A perda de foco nativa encerra o domínio imediatamente.
- Uma aba ativa em janela de navegador ao fundo nunca soma tempo. Múltiplas janelas do perfil configurado são permitidas; só uma pode estar efetivamente focada.
- Outro perfil, Chrome anônimo, permissões ausentes ou identificação ambígua não herdam o domínio anterior: aparecem sem domínio conhecido.
- Domínio observado sem categoria continua identificável na tabela; diferenciar `site_sem_categoria` de `origem_nao_identificada` e `extensao_indisponivel` nos motivos de desconhecido.

Não existe correspondência garantida entre `windowId` e HWND. A F3 deve comprovar a combinação de foco nativo e `focused` da extensão, inclusive troca para outro perfil/janela privada. Em qualquer ambiguidade, desconhecido tem precedência sobre uma atribuição possivelmente errada.

### Relógios e limites de dia

Usar relógio monotônico para duração e UTC para localização no calendário. Não calcular tempo decorrido exclusivamente com `Date.now()` ou diferença de relógios civis. [Relógios de alta resolução do Windows](https://learn.microsoft.com/en-us/windows/win32/sysinfo/acquiring-high-resolution-time-stamps).

Salvar também data local e offset de cada segmento. Segmentar na meia-noite local e ao mudar o fuso, sem novo acesso apenas por esse corte. Histórico usa a data/offset gravados, evitando mover atividades entre dias ao consultar depois em outro fuso. Não assumir que todo dia tem 24 horas.

Mudança abrupta no relógio civil fecha a âncora anterior e inicia outra, preservando duração monotônica e um motivo `clock_adjusted`. Horários civis podem se repetir; a UI deve sinalizar o ajuste e manter a ordem observada, em vez de criar duração negativa ou somar duas vezes. Invariantes de exclusividade são verificadas na sequência monotônica de cada execução, não apenas ordenando timestamps civis.

## 11. Armazenamento local

SQLite no diretório local de dados do usuário, por exemplo `%LOCALAPPDATA%\CaixaPretaDoDia\data.sqlite3`, resolvido pela API de diretórios do app. Não salvar dados de uso no repositório. Uma conexão escritora controlada pelo núcleo; consultas do frontend passam por comandos tipados, sem SQL arbitrário.

Usar transações com rollback journal padrão e sincronização durável; WAL não é necessário inicialmente para um escritor e consultas curtas serializadas. Migrações SQL numeradas e `PRAGMA user_version` bastam; sem ORM, servidor de banco ou framework de migrações. Testar interrupção de transação e recuperação. [Atomicidade do SQLite](https://www.sqlite.org/atomiccommit.html).

| Tabela | Campos essenciais e finalidade |
| --- | --- |
| `settings` | Chave/valor para configuração VDI, limiar de inatividade, pausa persistida e preferências locais |
| `categories` | ID estável, nome, ordem visual e `counts_as_distraction`; categorias iniciais fixas |
| `domain_categories` | Hostname normalizado como chave, categoria escolhida e instante da alteração |
| `app_categories` | Nome amigável do aplicativo local como chave, categoria escolhida e instante da alteração |
| `sessions` | ID, run_id, ordem, início/fim UTC, âncora monotônica/duração em ms, data local, offset, origem, hostname ou nome de aplicativo opcional, motivo e último instante confirmado |
| `accesses` | ID, instante/data/offset, run_id, identidade efêmera da aba e hostname; conta entradas efetivas sem confundir com fragmentação de sessões |
| `tracker_checkpoint` | Registro único: execução, instante confirmado, estado de encerramento e referência à sessão aberta |

Origens de sessão: `vdi`, `browser`, `app`, `system`, `unknown`, `idle`, `paused`, `unobserved`. Pausa e ausência de coleta não contêm hostname. Categorias de browser e aplicativos locais são resolvidas pelos mapas locais ao consultar, permitindo reclassificação histórica sem duplicar eventos.

Índices iniciais: sessões por data/início e por hostname/data; acessos por data/hostname. Constraints impedem duração negativa, categoria inválida e mais de uma sessão aberta. Atualizar sessões, acessos e checkpoint relacionados em uma transação.

Persistir em cada mudança de estado; durante bloco contínuo, atualizar o checkpoint e a fronteira confirmada aproximadamente a cada 5 segundos. O nome de aplicativo é derivado somente do nome do executável. Não armazenar PID, caminho de processo, título da janela ou conteúdo por sessão; metadados mínimos de reconhecimento ficam apenas na configuração da VDI.

Não incluir criptografia customizada no MVP. Dados ficam sob as permissões locais do usuário; o app não promete proteção contra acesso à conta/ao disco. Nenhum backup automático externo ou retenção silenciosa em logs. Histórico é mantido até exclusão solicitada; exportações são cópias deliberadamente criadas pelo usuário.

## 12. Cálculos e classificação

Somar durações em milissegundos e arredondar somente na apresentação. Usar os mesmos resultados do núcleo para cards, linha do tempo, gráficos, tabela e exportação.

| Métrica | Definição |
| --- | --- |
| Tempo VDI | Soma de sessões ativas `vdi`, recortadas no dia |
| Tempo por domínio | Soma de sessões `browser` com aquele hostname, já limitadas a foco, atividade e validade da extensão |
| Tempo por aplicativo | Soma das sessões com `app_name`, incluindo o Chrome quando houver domínio confirmado |
| Tempo por categoria | Soma dos domínios e aplicativos locais conforme mapas locais atuais, mais origens fixas VDI/sistema/desconhecido |
| Distrações | Sessões browser e de aplicativos locais cujas categorias foram marcadas pelo usuário; não incluir inatividade, lacunas ou VDI |
| Maior distração | Domínio ou aplicativo com maior duração entre as categorias marcadas; desempate alfabético; sem classificação mostrar “Defina as categorias” |
| Acessos | Uma entrada efetiva numa aba/domínio em primeiro plano: início de observação, troca de aba, mudança de domínio na aba ou retorno de outra janela/inatividade. Reload e navegação no mesmo domínio na mesma aba não contam novamente. Duplicatas, checkpoints e meia-noite também não |
| Trocas de contexto | Transições diretas entre identidades ativas diferentes: VDI, domínio ou origem local. Duas abas do mesmo domínio não são nova identidade. Pausa, inatividade e lacunas quebram a cadeia; não contar uma troca fictícia através delas |
| Maior bloco de foco | Maior intervalo contínuo de VDI no dia; qualquer perda de foco, inatividade, pausa ou falha de coleta interrompe |
| Tempo desconhecido | Origem não identificada mais sites ainda sem categoria, com detalhamento dos dois motivos |
| Tempo inativo | Sessões idle por limiar ou bloqueio; suspensão e app fechado ficam fora |
| Distribuição por horário | Interseção dos segmentos com cada hora civil do dia; preservar offset para horas repetidas |

Antes de o usuário marcar categorias, “distrações” não será apresentado como zero comprovado. O app exibirá “Categorias não definidas”. Após a configuração, zero significa nenhum tempo observado nessas categorias.

O denominador do gráfico de categorias é tempo ativo observado, incluindo desconhecido. Inatividade aparece separadamente. A comparação VDI/distrações mostra os dois valores absolutos e informa o restante ativo; não sugere que essas duas classes explicam o dia inteiro.

Invariante: **VDI + browser + sistema + origem desconhecida + inatividade = cobertura medida**. Pausa e ausência de coleta são apresentadas fora dessa soma. Classificações de sites são uma partição do tempo browser; não se soma “browser” novamente às suas categorias.

Exemplo de teste: 08:30–09:00 VDI, 09:00–09:10 YouTube ativo, 09:10–09:20 inatividade, 09:20–09:30 VDI resulta em 40 min de VDI, 10 min de YouTube, 10 min de inatividade, maior bloco de foco de 30 min e uma troca direta de contexto. Se vídeo foi marcado como distração, distrações = 10 min; caso contrário, esse total depende da configuração.

A edição de categoria acontece na linha do domínio. Informar que a mudança afeta o histórico e atualizar todas as métricas do dia. Não inferir categoria por palavras do domínio nem consultar listas online; todos começam desconhecidos até classificação explícita.

## 13. Fechamento inesperado, suspensão e reinicialização

| Situação | Comportamento esperado |
| --- | --- |
| Fechar janela | Continuar na bandeja com indicação de rastreamento; explicar esse comportamento no onboarding |
| Sair pelo menu | Fechar sessão e transação; gravar encerramento limpo e parar coleta |
| Crash/encerramento forçado | Na reabertura, fechar sessão pendente no último checkpoint confirmado; não atribuir tempo até o relançamento |
| Queda de energia | Mesmo procedimento de recuperação; eventual intervalo após checkpoint fica sem coleta, não estimado como foco |
| Suspensão/hibernação | Fechar cobertura ao receber evento; na volta criar lacuna e consultar novamente foco, atividade e navegador |
| Suspensão sem evento entregue | Detectar descontinuidade entre observações; fechar no último instante confirmado |
| Bloqueio/desbloqueio | Bloqueio encerra foco e abre idle; desbloqueio exige nova leitura antes de atribuir origem |
| Reinicialização/logoff | Nova execução e nova origem temporal; nenhum estado ativo da execução antiga pode continuar |
| Worker/ponte encerrado | Encerrar atribuição de domínio no último estado válido, sinalizar desconexão e reconciliar ao retornar |
| Banco indisponível/disco cheio | Encerrar confirmação de gravação, sinalizar erro persistente e não acumular fila ilimitada; retorno cria lacuna explícita |
| Migração inválida/banco corrompido | Não apagar/recriar silenciosamente; manter arquivo e informar erro e localização para recuperação |

O alvo é limitar perda de tempo recente a aproximadamente 5 segundos em execução normal, devido ao checkpoint; não é garantia contra falhas de disco/OS. Eventos de suspensão e retomada exigem teste no Windows real, não apenas simulação.

## 14. Proposta de interface

Modo de uso: **operação e análise**. A primeira leitura deve revelar a sequência do dia, com números que expliquem a linha do tempo. Interface em português, sem linguagem promocional ou motivacional.

```text
Caixa Preta do Dia            Hoje / data anterior          ● Ativo  [Pausar]
Hoje | Histórico | Configurações

[VDI / trabalho focado] [Distrações*] [Maior bloco de foco] [Trocas de contexto]
Inativo: …     Desconhecido: …     Maior distração: …     Cobertura: …

Linha do tempo do dia                     [todos / categoria selecionada]
08h          10h          12h          14h          16h          18h
════════════ blocos proporcionais, legenda e lacunas identificadas ═══════
Seleção: intervalo, origem/domínio, categoria, duração e motivo se houver

Tempo por categoria                       Tempo por domínio
[barras horizontais com valores]           [barras horizontais com valores]
Distribuição por horário                   VDI e distrações selecionadas
[barras por hora]                          [comparação com valores]

Sites e categorias                              [filtro] [Exportar dia]
Domínio           Categoria editável           Tempo ativo       Acessos
…                 …                            …                 …
```

Quatro cards principais, demais métricas em faixa compacta. Evitar sete cards equivalentes competindo por atenção. Os gráficos têm funções distintas; clicar em bloco/barra aplica filtro visível à tabela, com ação para removê-lo. Cards permanecem totais do dia e são identificados como tal, para não mudar de significado silenciosamente.

- Canvas cinza muito claro, superfícies brancas, bordas discretas e cobalto nas ações; sem gradientes, glassmorphism ou elementos de chat.
- Tipografia de sistema, preferencialmente Segoe UI; números tabulares, escala curta de tamanhos e espaçamento consistente.
- Cantos moderadamente arredondados e sombras leves somente onde distinguem camadas.
- Categorias usam poucas cores estáveis, texto/legenda e padrões para lacunas; desconhecido em cinza e inatividade em tom discreto. Não usar vermelho/verde para julgar a pessoa.
- Layout principal para desktop; abaixo de aproximadamente 1100 px empilhar gráficos e reduzir navegação. Em largura estreita, linha do tempo pode rolar, mantendo alternativa textual acessível.
- Não exibir um segmento como zero apenas porque é estreito: permitir seleção por lista de intervalos e detalhe por teclado. Tempo anterior ao primeiro registro e posterior ao último permanece fora das lacunas, pois não há evidência de que a coleta deveria estar ativa.
- Componentes simples em HTML/CSS e SVG para linha do tempo/barras; sem biblioteca de gráficos inicialmente. Se os requisitos acessíveis não puderem ser atendidos de forma pequena, justificar uma dependência no plano antes de adotá-la.

### Estados de interface

| Estado | Conteúdo e ação |
| --- | --- |
| Primeira execução | Explicar coleta local; configurar janela VDI; orientar instalação da extensão; mostrar teste de conexão e botão explícito de iniciar rastreamento |
| Sem extensão | VDI continua disponível; navegador identificado como desconhecido e instrução “Conectar extensão” |
| VDI não configurada | Mostrar configuração pendente; nunca classificar todo Windows App automaticamente |
| Hoje vazio | “Ainda não há períodos registrados hoje”, estado da coleta e ação de iniciar/retomar conforme o caso |
| Histórico vazio | “Sem registros neste dia”; não preencher com dados de demonstração |
| Ativo | Texto e ícone na janela/bandeja, origem atual e última gravação confirmada |
| Pausado | Estado persistente, início da pausa e ação Retomar; nenhum contador ativo |
| Permissão/política bloqueada | Explicar qual capacidade falhou: extensão, native host ou leitura de processo; oferecer nova tentativa sem solicitar elevação indiscriminada |
| Desconhecido | Diferenciar site sem categoria de falta de sinal; ação de classificar só se hostname foi observado |
| Erro de gravação | Mensagem permanente no app/bandeja, sem toast recorrente; não afirmar que dados foram salvos |
| Recuperação após falha | Faixa discreta com lacuna e último instante confirmado; sem estimar o que aconteceu |
| Carregando dia | Estrutura estável com indicação de carregamento; não mostrar zeros transitórios como resultado |
| Exclusão/exportação | Confirmação específica para apagar; destino escolhido localmente, progresso curto e resultado verificável |

## 15. Controle dos dados e inicialização

- **Pausa:** grava preferência local para persistir após reiniciar; para de capturar origens/domínios. Mantém apenas informação necessária para indicar a pausa e retomar por ação explícita.
- **Exclusão:** oferecer apagar o dia selecionado ou toda atividade. Mostrar escopo antes de confirmar. Pausar escritor, invalidar estado/conexões antigas, apagar sessões/acessos/checkpoint aplicáveis em transação e limpar caches da UI; preservar categorias/configuração, informando isso na confirmação. Não reimportar mensagens anteriores à exclusão. Se coleta voltar, começar no instante da retomada.
- **Limite da exclusão:** remover dados da aplicação/banco; não prometer apagamento forense do SSD nem apagar cópias exportadas pelo usuário. Limpar temporários próprios, não fazer varredura de diretórios externos.
- **Exportação:** um JSON por dia com versão de schema, data/offsets, sessões, acessos e categorias necessárias à interpretação. Sem PID, caminho de processo, conteúdo remoto ou configuração sensível; gravar em arquivo escolhido pelo usuário com substituição confirmada e escrita atômica. Importação e CSV não entram no MVP.
- **Autostart:** toggle desativado por padrão, execução por usuário no login, respeitando estado pausado e instância única. Usar o plugin oficial do Tauri se o teste local for satisfatório; não criar tarefa agendada ou serviço como alternativa automática. [Autostart do Tauri](https://v2.tauri.app/plugin/autostart/).
- **Distribuição:** instalador Windows por usuário com app e ponte; extensão instalada localmente no MVP pessoal. Testar política do Chrome para extensão não empacotada/native host antes de prometer instalação simples. Não alterar políticas corporativas nem publicar em loja sem pedido.

## 16. Arquivos previstos e dependências

Todos os caminhos abaixo são **propostos, ainda inexistentes**, relativos ao workspace. Podem ser reduzidos ao implementar, preservando responsabilidades claras; o plano deve registrar os caminhos efetivos por fase.

| Área | Arquivos prováveis |
| --- | --- |
| Especificação | `docs/specs/001-caixa-preta-do-dia.md` — este arquivo, com changelog e andamento |
| Projeto frontend | `package.json`, `package-lock.json`, `index.html`, `tsconfig*.json`, `vite.config.ts`, `eslint.config.js`, `.gitignore` |
| Interface | `src/main.tsx`, `src/App.tsx`, `src/styles.css`, `src/components/DayTimeline.tsx`, `src/components/DaySummary.tsx`, `src/components/DistributionCharts.tsx`, `src/components/DomainTable.tsx`, `src/components/Setup.tsx`, `src/components/Settings.tsx`, `src/api.ts`, `src/types.ts` |
| Aplicação nativa | `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/build.rs`, `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json`, `src-tauri/src/main.rs`, `src-tauri/src/lib.rs` |
| Coleta e dados | `src-tauri/src/windows_tracker.rs`, `src-tauri/src/sessions.rs`, `src-tauri/src/store.rs`, `src-tauri/src/commands.rs`, `src-tauri/src/bridge.rs`, `src-tauri/migrations/001_initial.sql` |
| Ponte | `src-tauri/src/bin/native_host.rs`, `native-host/manifest.template.json`, scripts locais de registro/desregistro e inclusão no instalador |
| Extensão | `extension/manifest.json`, `extension/src/background.ts`, `extension/src/popup.ts`, `extension/popup.html`, `extension/tsconfig.json`, `extension/vite.config.ts` |
| Validação | Testes `.test.ts(x)` próximos ao frontend/extensão; módulos Rust de teste e `src-tauri/tests/`; `docs/validation/windows-mvp.md` com roteiro e evidências |
| Uso | `README.md` com ambiente, execução local, instalação da extensão, permissões e localização dos dados |

### Dependências propostas, com motivo

| Dependência | Necessidade |
| --- | --- |
| React, React DOM, TypeScript, Vite e plugin React | Interface e build local; versões compatíveis fixadas em lockfile |
| Tauri 2, CLI e API JS | Janela, comandos nativos, bandeja e pacote Windows |
| `windows` crate | Foco, última entrada, energia, sessão, named pipe e identidade de processo/pacote |
| `rusqlite` com SQLite empacotado | Banco local sem serviço/instalação separada |
| `serde` e `serde_json` | Contratos de comandos, ponte e exportação JSON |
| `chrono` com suporte local | Datas, offsets e fronteiras de dia; duração fica no relógio monotônico |
| Plugin Tauri de instância única | Evitar duas coletas/escritores e reabrir a janela existente |
| Plugin Tauri de diálogo | Selecionar arquivo local de exportação com diálogo nativo |
| Plugin Tauri de autostart | Somente ao implementar e validar RF10 |
| ESLint, tipos React/Chrome, Vitest, Testing Library e ambiente DOM de teste | Lint, contratos e testes de comportamento da extensão/UI |

Usar um gerenciador npm e um conjunto de dependências de desenvolvimento na raiz; sem monorepo framework, biblioteca de estado global, router para três telas simples, ORM, servidor HTTP, biblioteca de IA, serviço de analytics ou biblioteca gráfica antes de necessidade demonstrada. Dependências de desenvolvimento podem exigir download; **o produto empacotado não depende de serviços externos para funcionar**.

## 17. Critérios de aceite e testes

| ID | Requisito | Aceite observável | Teste previsto |
| --- | --- | --- | --- |
| AC01 | RF01–02 | Só a janela configurada da sessão Windows App conta; launcher e janela ao fundo não contam | Máquina de estados Rust + teste manual Windows App/Alt-Tab/multimonitor |
| AC02 | RF02 | Perda de foco encerra VDI, ganho inicia novo bloco, sem sobreposição | Sequência determinística de eventos + cronômetro manual |
| AC03 | RF02, RF04 | Limiar de inatividade aplicado sem retroação; bloqueio imediato e retorno reconciliado | Relógio falso Rust + teste local de bloqueio/entrada na VDI |
| AC04 | RF03 | Troca de aba/domínio conta só enquanto a janela correta está em foco | Vitest da extensão + integração real com duas janelas |
| AC05 | RF03 | Aba ao fundo, outro perfil/anônimo, falta de URL/permissão não herdam domínio | Testes de snapshots fora de ordem + navegador real |
| AC06 | RF03, RNF01 | Nenhuma URL completa, título, texto ou conteúdo privado aparece na ponte, banco, logs ou exportação | Fixture com URL contendo caminho/query sentinela + inspeção dos artefatos |
| AC07 | RF04 | Duplicatas e reconexões não duplicam acesso/tempo; mensagens antigas não alteram passado | Testes de sequência, geração de foco, frames parciais e malformados |
| AC08 | RF04–06 | Totais consistem entre cards, gráficos, tabela e exportação; dia/fuso não perde ou duplica tempo | Fixtures determinísticas, meia-noite, hora repetida e ajuste de relógio |
| AC09 | RF05–06 | Hoje abre por padrão, seleção de dia atualiza todas as áreas e vazio não mostra dados falsos | Testing Library + execução local com histórico conhecido |
| AC10 | RF07 | Classificar domínio atualiza histórico; desconhecido não é inferido; distrações dependem da escolha | Teste SQLite/comandos + edição pela UI |
| AC11 | RF08 | Pausa impede coleta/transmissão de domínio e persiste após reinício | Integração núcleo/ponte + teste manual de reinício |
| AC12 | RF09 | Exclusão remove exatamente o escopo e mensagens antigas não restauram dados | Banco temporário e teste durante coleta ativa |
| AC13 | RF09 | JSON exportado corresponde ao dia, lê corretamente e não inclui dados fora do escopo | Exportação para diretório temporário + leitura e comparação |
| AC14 | RF10 | Toggle controla início no login sem instância duplicada; pausa é respeitada | Teste Windows real com login/reinício, se implementado |
| AC15 | RF11 | Crash/suspensão/reboot não prolonga sessão até o retorno; checkpoint limita perda recente | Testes Rust + encerramento forçado, suspensão e retomada reais |
| AC16 | RNF06 | Erro de disco/banco não exibe gravação bem-sucedida nem apaga dados silenciosamente | Falhas injetadas no armazenamento e banco inválido temporário |
| AC17 | RNF01, RNF03 | Instalado funciona sem conectividade externa; ponte rejeita origem/formato inválido | Execução offline e teste do host/ACL/ID da extensão |
| AC18 | RNF04–05 | Teclado, zoom/DPI, muitos blocos e domínios longos continuam utilizáveis | Inspeção UI em tamanhos/DPI previstos e fixture de dia denso |
| AC19 | RF12, RNF01 | Alternar da VDI para League of Legends, ChatGPT e Chrome cria blocos com nomes e totais corretos, sem título ou caminho | Testes Rust/React e ensaio real no Windows |
| AC20 | RF12, RF07 | Alterar a categoria de League of Legends ou WhatsApp atualiza categorias, distrações e histórico sem expor título ou caminho | Migração SQLite, relatório Rust e edição pela UI |

Testes protegem regras de tempo, privacidade e fluxos observáveis. Não testar nomes de classes CSS, organização interna ou reproduzir implementação sem verificar resultado. APIs Windows simuladas não substituem os testes reais de foco e energia.

Meta inicial para transições normais: diferença de até aproximadamente 1 segundo no roteiro manual, com medição separada do atraso extensão/ponte. Registrar tolerâncias observadas; não anunciar precisão subsegundo sem evidência. Contadores de um dia denso não devem crescer quando nenhum evento efetivo muda.

## 18. Validação técnica

Não há aplicação ou scripts de validação existentes. Os comandos abaixo são **contratos propostos para criar durante a implementação**, não comandos já executados com sucesso.

| Camada | Validação prevista |
| --- | --- |
| Frontend e extensão | `npm run lint`, `npm run typecheck`, `npm test -- --run` |
| Build web/extensão | `npm run build`, `npm run build:extension`; verificar artefatos locais e manifesto |
| Rust | `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check`; `cargo check --manifest-path src-tauri/Cargo.toml --all-targets`; `cargo test --manifest-path src-tauri/Cargo.toml`; `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` |
| Aplicação completa | `npm run tauri dev` e `npm run tauri build`; validar também o executável da ponte e sua inclusão no pacote |
| Instalação local | Instalar por usuário, conectar extensão/host, abrir/fechar/bandeja, reiniciar, exportar/apagar e executar offline |
| Windows App | Cronometrar foco/Alt-Tab, launcher, sessão, bloqueio, suspensão, tela cheia e multimonitor |
| Interface | Inspeção visual agrupada, fluxo por teclado, escalas Windows, vazio/erro/dados densos |

Executar por fase os checks específicos suficientes; rodar a integração completa após as partes estarem conectadas. Para cada execução registrar comando, resultado, data e limitação. Build frontend aprovado não valida Rust, instalador, extensão nem medição no Windows.

## 19. Fases de implementação

Cada fase só fica concluída quando os respectivos critérios forem atendidos. A primeira vertical completa de coleta precede acabamento visual extenso.

| Fase | Escopo e mudança | Validação/saída | Status |
| --- | --- | --- | --- |
| F0 — Viabilidade e base | Confirmar aprovação; fixar Chrome/premissas; verificar Node, toolchain C++/SDK e Tauri; validar políticas da extensão; comprovar identificação da janela Windows App | App mínimo abre; prova local diferencia janela alvo/launcher; host pode conectar; registrar bloqueios reais antes de expandir | Pendente |
| F1 — Núcleo e banco | Schema inicial, máquina de estados, relógios, classificação e consultas de dia com dados de teste | AC02–03, AC07–08, AC10, AC12–13 e AC16 nos casos independentes de SO; cargo check/test | Pendente |
| F2 — Coleta Windows | Foco VDI, idle, bloqueio, suspensão, checkpoints e pausa; estado mínimo visível | AC01–03, AC11 e AC15 com testes Rust e verificação real no Windows App | Pendente |
| F3 — Navegador integrado | Extensão Chrome, ponte, snapshot de foco, reconexão, privacidade e contagem por domínio | AC04–07 e parte de AC17; teste de duas janelas/perfis, worker reiniciado e perda da ponte | Pendente |
| F4 — Dashboard e histórico | Layout principal, linha do tempo, resumos, gráficos, detalhes, categorias e estados de interface | AC08–10 e AC18; lint/typecheck/test/build e inspeção visual/teclado | Pendente |
| F5 — Controles e instalação | Bandeja, instância única, exportação, exclusão, onboarding e instalador/registro do host; autostart se simples e validável | AC11–14, AC17; execução instalada e offline | Pendente |
| F6 — Validação final | Roteiro completo de dia conhecido; falhas, suspensão, restart, consistência e regressões; documentação de uso | Todos os aceites aplicáveis com evidência; build Tauri/pacote testados; pendências explícitas | Pendente |

Se F0 não conseguir distinguir a sessão Windows App do launcher com metadados mínimos, parar nesse bloqueio e apresentar a evidência antes de alterar o modelo de identificação. Se a política impedir Native Messaging/extensão local, não substituir silenciosamente por servidor, outro navegador ou publicação em loja.

## 20. Riscos e decisões ainda abertas

| Risco/decisão | Tratamento dentro deste plano |
| --- | --- |
| Preferência de navegador não confirmada | Chrome proposto; trocar antes da F0 se necessário. Firefox mudaria o contrato/empacotamento da extensão e exige revisão |
| Windows App muda executável, classe ou pacote | Configuração por identidade estável + diagnóstico/reconfiguração; comprovar em F0 e F2 |
| Launcher/desconexão indistinguível por metadados | Não afirmar detecção já resolvida; teste real obrigatório, sem captura de conteúdo |
| Inatividade durante leitura/vídeo | Semântica explícita e limiar configurável; nenhum mecanismo para inferir atenção |
| Permissões corporativas | Detectar bloqueio e explicar capacidade indisponível; não contornar políticas |
| Worker encerrado e corrida de eventos | Snapshot, geração de foco, sequência e prazo de validade; desconhecido quando falta confirmação |
| Um perfil Chrome no MVP | Outras janelas/perfis não podem contaminar o domínio observado; suporte adicional fica fora |
| Toolchain local insuficiente | Node observado é incompatível com Vite atual; C++/SDK ainda não validados; preparar somente na implementação |
| Reclassificação muda números antigos | Comportamento explícito na UI e teste; sem histórico complexo de regras |
| Sem referência visual anexada | Briefing textual é suficiente para planejar; nenhuma reprodução visual literal prometida |
| Autostart/instalador | Recurso opcional condicionado à integração simples; se pendente, relatar separadamente sem declarar o MVP integralmente validado |
| Relógio/fuso/suspensão | Duração monotônica, âncoras civis, segmentos por dia/offset e lacunas; testar cenários reais |

## 21. Andamento e evidências desta entrega

- [x] Ler o pedido e as instruções fornecidas.
- [x] Inspecionar o workspace, estado Git, ferramentas e navegadores disponíveis.
- [x] Incorporar a confirmação de **Windows App** e verificar a origem do processo observado.
- [x] Consultar documentação oficial das APIs e pré-requisitos usados na recomendação.
- [x] Escrever especificação, contratos, critérios de aceite e fases neste Markdown.
- [x] Aprovação do plano e confirmação/correção das premissas D03–D09.
- [x] Implementação solicitada em nova mensagem.
- [ ] F0–F6 e respectivas validações do produto.

**Validação do documento:** revisão de cobertura concluída para os 19 tópicos solicitados. Verificação estrutural confirmou 22 seções, RF01–RF11, AC01–AC18, sete fases pendentes, blocos Markdown balanceados e nenhum caractere de substituição. O estado Git confirmou somente este Markdown como arquivo novo. Nenhum teste, build ou execução do produto foi realizado, pois não existe código nesta etapa; a validação apropriada foi a revisão documental e de escopo.

## 22. Changelog do plano

Manter entradas em ordem cronológica. Toda alteração futura neste arquivo deve incluir data, versão, motivo, seção/requisitos afetados, impacto no plano e validação realizada ou pendente. Não usar o changelog para afirmar implementação sem evidências.

| Data | Versão | Alteração | Motivo e impacto | Validação |
| --- | --- | --- | --- | --- |
| 2026-09-10 | 0.1 | Criação da especificação e plano SDD; RF01–RF11, RNF01–RNF06, AC01–AC18 e F0–F6 | Pedido inicial; inclui esclarecimento do usuário de que a VDI usa Windows App. Chrome e categorias de distração permanecem propostas explícitas | Inspeção de ambiente e documentação concluída; revisão do Markdown pendente; nenhum código implementado |
| 2026-09-10 | 0.1.1 | Registro da revisão final na seção 21 e atualização da versão | Concluir a entrega documental sem modificar o escopo ou iniciar implementação | Cobertura dos 19 tópicos e estrutura verificadas; Git contém apenas este documento novo; testes/build do produto não se aplicam nesta etapa |
| 2026-09-10 | 0.2 | Aprovação registrada; início de F0, base Tauri/React e diagnóstico de janelas | Node 24.19 já disponível no runtime; Rust/MSVC compilou e executou probe; janela do Windows App observada como `msrdc` / `TscShellContainerClass`. Bindings `windows-sys` escolhidos para as mesmas APIs previstas, sem camada adicional | Probe nativo aprovado; instalação npm e build Tauri em andamento; launcher e medição completa ainda exigem validação |
| 2026-09-10 | 0.3 | Implementação inicial de RF01–RF11 e checkpoint de retomada | Núcleo, UI, extensão, ponte, controles de dados e pacote Windows criados; agrupamentos de arquivos e empacotamento em duas etapas registrados acima | Builds e suites anteriores passaram; teste nativo da ponte revelou timeout, correção e nova verificação em andamento; F6 ainda não concluída |
| 2026-09-10 | 0.3.1 | Checkpoint final e interrupção solicitada pelo usuário | Registrar estado real para retomada em outra sessão, sem novos testes/builds; pacote existente permanece anterior à correção da ponte | Última suite: 14 testes Rust, 9 JS, typecheck/lint e Clippy aprovados; 4 checks da ponte corrigida passaram; validação real restante e regeneração do pacote pendentes |

| 2026-09-10 | 0.3.2 | Retomada somente das pendências do checkpoint; confirmação visual pós-contraste e regeneração release | Plano aprovado preservado; Chrome bloqueado pelo helper de automação; aceites reais continuam abertos | Capturas e resultado de contraste conferidos; host/app release e frontend atualizado compilados; resultado NSIS registrado na seção de retomada; suites anteriores não repetidas |
| 2026-09-10 | 0.4.0 | RF12, D10 e AC19: identificação e relatório de aplicativos locais em foco | Usuário solicitou distinguir atividades fora da VDI, como League of Legends e ChatGPT, preservando privacidade | 17 testes Rust, 10 JS, fmt, Clippy, typecheck, lint e build web aprovados; ensaio real AC19 no Windows pendente |
| 2026-09-10 | 0.4.1 | Correção de travamento ao abrir e em ações da bandeja | Event Log registrou Application Hang; chamadas Tauri da bandeja ocorriam com mutex do runtime retido, permitindo espera circular com IPC | Testes, Clippy e build posteriores aprovados; abertura/fechamento repetidos no Windows ainda pendentes |
| 2026-09-11 | 0.5.0 | RF12 e AC20: categorização local de aplicativos | Usuário solicitou categorizar aplicativos locais, como League of Legends e WhatsApp, pelas mesmas categorias usadas em sites; adiciona mapa local aplicativo→categoria e reclassificação histórica | 17 testes Rust, 10 JS, fmt, Clippy, typecheck, lint, build Vite e build release aprovados; ensaio real AC19/AC20 no Windows pendente |
| 2026-09-11 | 0.6.0 | RF05–06 e AC18: agrupamento visual e filtros da linha do tempo | Usuário solicitou reduzir a fragmentação visual e filtrar por categoria; registros persistidos e cálculos permanecem inalterados | Suíte Vitest com 13 testes, typecheck, ESLint e build Vite aprovados; inspeção manual de dia denso pendente |
| 2026-09-11 | 0.7.0 | RF05–06/AC18: primeiro início, último fim confirmado, maior lacuna interna e faixa de estados; arquivos e regras detalhados no incremento acima | Somente itens 7 e 8; derivação do relatório bruto sem migração, mudanças de coleta ou novos totais; preserva agrupamento, filtros e exportação | 31 testes, typecheck, lint e build aprovados com Node 24; cenários web sintéticos em 1280/800 px aprovados. Leitor de tela, cores forçadas, DPI e ensaio WebView2/Windows reais pendentes; sem instalador |
| 2026-09-14 | 0.7.1 | Tentativa restrita de validação manual real v0.7.0 | A build release iniciou como processo, mas o controlador de UI não expôs janela nativa; nenhuma regra de coleta ou feature foi alterada | Processo observado; timeline densa/sessão aberta, teclado, leitor de tela, cores forçadas e DPI 100/125/150% continuam não validados e bloqueados. Evidência detalhada em `docs/validation/windows-mvp.md` |
