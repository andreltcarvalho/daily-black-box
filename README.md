# Caixa Preta do Dia

Aplicativo Windows para observar o tempo em foco na VDI pelo **Windows App**, na aba ativa do **Google Chrome**, em aplicativos locais e em inatividade. Dados locais; sem IA, nuvem, login, leitura de páginas ou avaliação de produtividade.

O escopo, os critérios de aceite e o andamento estão em [docs/specs/001-caixa-preta-do-dia.md](docs/specs/001-caixa-preta-do-dia.md). As evidências de validação ficam em [docs/validation/windows-mvp.md](docs/validation/windows-mvp.md).

## Desenvolvimento

Requisitos: Windows, Node **22.12+**, npm, Rust estável com target MSVC, C++ Build Tools/Windows SDK e WebView2. A versão Node 20.15 instalada originalmente nesta máquina é insuficiente. Nesta sessão foi usado Node 24.19 do runtime já existente, sem alterar o Node global.

```powershell
npm ci
npm run tauri dev
```

`npm run dev` abre somente a interface web. A coleta e o banco são exclusivos do aplicativo Windows; não há dados de demonstração nem coletor no navegador dessa prévia.

## Primeira execução

1. Abra uma **sessão remota** no Windows App.
2. No Caixa Preta do Dia, clique em **Identificar minha VDI**. Dentro de 5 segundos, coloque a janela da sessão em foco. O app verifica o processo do pacote Windows App e a classe de janela remota; a tela de conexões não deve ser selecionada.
3. Instale a extensão conforme abaixo. Se preferir começar pela VDI, o navegador ficará como desconhecido até a conexão da extensão.
4. Clique em **Iniciar / retomar**. O rastreamento inicia pausado e não começa sozinho antes dessa ação.

Fechar a janela mantém o aplicativo na bandeja. **Sair e encerrar coleta** encerra o processo. Pausa persiste entre execuções; inicialização com o Windows começa desativada.

## Extensão e ponte local

A primeira versão suporta Chrome, um perfil, várias janelas normais. Navegação anônima e outros perfis não são coletados.

Em desenvolvimento:

```powershell
npm run build:extension
npm run build:native-host
powershell -NoProfile -File scripts/register-native-host.ps1
```

No Chrome, abra `chrome://extensions`, ative o modo desenvolvedor e use **Carregar sem compactação** na pasta `extension/dist`. O ID deve ser `pjmccgpomaddaokgbjfmoidakooahmph`. Abra o aplicativo e confira a conexão no popup da extensão.

No aplicativo instalado, o instalador registra a ponte por usuário. Carregue a pasta `extension` dentro do diretório de instalação no Chrome. Se o registro automático falhar, execute o script `native-host/register-native-host.ps1` com `-HostExecutable` apontando para o executável da ponte na mesma pasta. Políticas corporativas podem impedir instalação local ou Native Messaging; o app não altera essas políticas.

A extensão precisa de `tabs`, `nativeMessaging` e `alarms`. A URL da aba é usada transitoriamente para extrair somente o hostname; caminho, query, fragmento e título não são enviados. O protocolo não aceita campos extras. A comunicação é Native Messaging + named pipe local restrito ao usuário/sessão, sem porta HTTP.

## Regras de tempo

- VDI conta somente com a janela remota configurada em primeiro plano.
- Inatividade começa após 5 minutos sem teclado/mouse, sem retroagir; o limite pode ser ajustado.
- Bloqueio inicia inatividade. Suspensão e app encerrado geram lacunas sem coleta.
- Uma aba ao fundo nunca conta. Conexões interrompidas não prolongam indefinidamente o último domínio.
- Acesso significa uma entrada efetiva em uma aba/domínio em foco; reload no mesmo domínio não é um novo acesso.
- Todo site começa sem categoria. Você escolhe quais categorias entram como distração; essas escolhas também atualizam o histórico.
- Todo aplicativo local começa em “Trabalho local”. Em **Aplicativos locais e categorias**, escolha a categoria de League of Legends, WhatsApp ou qualquer outro aplicativo identificado; a escolha atualiza o histórico, os totais e as distrações quando a categoria estiver marcada.
- Checkpoints são gravados aproximadamente a cada 5 segundos e nas transições. O período não confirmado após uma falha não é inventado como foco.

## Dados

O caminho exato do SQLite aparece em **Configurações → Seus dados**; ele é resolvido no diretório local de dados do usuário pelo Tauri. Exportação gera JSON do dia selecionado. Exclusão pode apagar um dia ou toda atividade, mantendo categorias/configuração e pausando o rastreamento. Exportações criadas pelo usuário são independentes; exclusão não é apagamento forense do disco.

## Validação e pacote Windows

```powershell
npm run lint
npm run typecheck
npm test -- --run
npm run build
npm run build:extension
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
npm run package:windows
```

`package:windows` primeiro compila a ponte, depois o app e o instalador NSIS com extensão/registro. `tauri build` isolado compila o aplicativo, mas não gera o pacote completo. O instalador completo fica em `src-tauri/target/release/bundle/nsis/`. O pacote existente precisa ser regenerado antes de distribuir as mudanças v0.4.1 e v0.5.0. Nenhuma publicação em loja ou assinatura de código está incluída.

Validações de foco real, política do Chrome, suspensão, bloqueio, login automático e comportamento do instalador devem ser conferidas no Windows; os testes unitários não as substituem.

## Estado da validação

Os testes atuais aprovam migração SQLite v1→v3, nomes de aplicativos, reclassificação histórica, interface de categorias, typecheck, lint, build web, Clippy e build release. Ainda é necessário executar o roteiro real em [docs/validation/windows-mvp.md](docs/validation/windows-mvp.md), incluindo alternância entre VDI, League of Legends, WhatsApp e Chrome e abertura repetida pela bandeja.
