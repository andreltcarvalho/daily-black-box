# Caixa Preta do Dia — estímulo da VDI em segundo plano

Status: **Fase 0 iniciada; bloqueada antes do ensaio por indisponibilidade da janela VDI na automação**.
Data: **2026-09-14**.
Workspace: `C:\Users\Usuario\Documents\ChatGPT\My-Apps`.

## 1. Objetivo

Enquanto a sessão Windows App configurada permanece aberta em segundo plano, capturar a janela local atual, focar brevemente a VDI, emitir um estímulo mínimo à sessão remota e restaurar a janela anterior. O objetivo é que o Teams executado dentro da VDI reconheça atividade enquanto o usuário trabalha ou lê em outra janela local.

O app não deve afirmar que uma pessoa está disponível: ele apenas oferece uma ação local autorizada pelo proprietário do ambiente. A confirmação de que o Teams remoto reage ao estímulo é um requisito de aceitação, não uma premissa.

## 2. Escopo e limites

Entra no escopo, condicionado à viabilidade:

- opção local, explicitamente ativada, vinculada apenas à identidade de VDI já configurada (`msrdc.exe` e `TscShellContainerClass`);
- agendamento enquanto a VDI estiver conectada, mesmo se estiver em segundo plano; cada ciclo captura a janela atual, foca a VDI e tenta restaurar a anterior;
- parada imediata quando o rastreamento estiver pausado, a sessão Windows estiver bloqueada, o computador suspenso, a VDI não existir ou não corresponder à identidade configurada;
- indicação local de ligado/desligado, próximo estímulo e último resultado técnico, sem armazenar conteúdo, título, teclas, URLs ou dados do Teams.

Não entra no escopo:

- API, login, extensão ou alteração de status do Teams;
- enviar `Alt+Tab`, clicar ou digitar na máquina local; o foco temporário da VDI é a única exceção explícita;
- simular presença durante bloqueio, suspensão, VDI desconectada ou rastreamento pausado;
- serviço Windows, tarefa agendada, rede, nuvem ou dependência nova.

`SendInput` na sessão local só poderá ser considerado depois que a VDI estiver em foco e se a prova demonstrar que o estímulo chega à sessão remota. Ele pode atualizar `GetLastInputInfo`, usado hoje para medir inatividade; a implementação não pode converter esse evento sintético em atividade humana registrada.

## 3. Hipótese a validar

O Windows App pode — ou não — encaminhar uma entrada mínima quando sua janela é brevemente focada. Mesmo que o cliente remoto a receba, o Teams pode não contabilizá-la para presença. A ativação e a restauração do foco também são assíncronas no Windows: menos de um segundo é meta de medição, não garantia antecipada.

Se a restauração falhar, se o ciclo exceder o tempo aceitável medido ou se houver risco observável de uma entrada humana cair na VDI, a tarefa falha por limite de produto e não segue para implementação.

## 4. Fase 0 — prova de viabilidade manual

Executar primeiro em uma VDI de teste ou janela de trabalho sem ação crítica aberta.

1. Registrar a linha de base: Teams dentro da VDI, VDI em segundo plano e nenhuma entrada por tempo suficiente para observar o status Ausente. Anotar cliente Teams, data, estado inicial, instante de mudança e presença observada por uma segunda conta/dispositivo.
2. Com a VDI em segundo plano e outra janela local focada, medir o ciclo de captura do foco atual, ativação da VDI e restauração. Registrar janela inicial/final, duração monotônica e qualquer alteração visível.
3. Somente após o ciclo de foco/restauração passar, testar um estímulo sem clique ou tecla enquanto a VDI está focada. Registrar se a VDI o recebe e se o `GetLastInputInfo` local muda.
4. Repetir por pelo menos dois ciclos completos do limiar de ausência observado no Teams, comparando o status remoto por uma segunda conta/dispositivo.
5. Repetir com VDI em foco, em segundo plano, minimizada, desconectada, Windows bloqueado e após suspensão/retomada. Nos três últimos estados, não pode haver estímulo.

Resultado da Fase 0:

| Resultado | Decisão |
| --- | --- |
| Teams permanece disponível, VDI recebe o estímulo e a janela original volta ao foco no tempo aceitável medido | Autorizar a implementação limitada descrita na Fase 1 |
| A VDI não recebe o estímulo, Teams não reage ou há resultado inconsistente | Registrar a limitação e não implementar |
| A restauração falha, o ciclo interfere na janela original ou uma entrada humana pode cair na VDI | Rejeitar a abordagem |

## 5. Fase 1 — implementação, somente após Fase 0 aprovada

Áreas prováveis, confirmadas antes de editar:

| Área | Mudança prevista |
| --- | --- |
| `src-tauri/src/windows_tracker.rs` | encapsular captura/restauração de foco e somente o mecanismo aprovado de estímulo à VDI, expondo resultado sem dados sensíveis |
| `src-tauri/src/runtime.rs` | agendar o intervalo `X`, respeitando pausa, bloqueio, suspensão, desconexão e encerramento |
| `src-tauri/src/store.rs` e `src-tauri/src/commands.rs` | persistir a preferência e validar os valores permitidos de `X` |
| `src/types.ts` e `src/App.tsx` | expor controle, estado e falha de forma clara; sem alterar o dashboard de atividade |
| testes Rust e frontend existentes | cobrir regras de habilitação, parada e configuração; a entrega depende também do ensaio Windows real |

O intervalo e o valor padrão só serão definidos após a linha de base da Fase 0. Ele deve ser menor que o limiar real de ausência observado, mas não será aplicado em rajada nem ajustado automaticamente.

## 6. Aceites

- AC21: com VDI conectada em segundo plano e estímulo habilitado, a janela local inicial volta ao foco ao fim de cada ciclo; o tempo do ciclo é medido e registrado.
- AC22: nenhum clique, tecla ou deslocamento visível do cursor é produzido; se o estímulo atualizar `GetLastInputInfo`, ele não prolonga atividade humana registrada pela Caixa Preta.
- AC23: em dois ciclos acima do limiar real medido, o Teams dentro da VDI permanece disponível quando verificado por uma segunda conta/dispositivo.
- AC24: pausa, bloqueio, suspensão, VDI inexistente/desconectada e encerramento do app impedem imediatamente novos estímulos.
- AC25: falha do mecanismo é visível no estado local, não inicia repetição agressiva e não altera os dados de rastreamento.

Testes automatizados não substituem AC21–AC24: o encaminhamento pelo Windows App e o cálculo de presença do Teams exigem confirmação no ambiente real.

## 7. Validação prevista

| Camada | Evidência |
| --- | --- |
| Rust | testes determinísticos de configuração, agenda, pausa, bloqueio, suspensão, desconexão e retorno de erro |
| Frontend | teste do toggle, configuração de `X`, mensagem de estado e falha |
| Integração Windows | roteiro da Fase 0, incluindo foco preservado e ausência de alteração no idle local |
| Teams remoto | observação por segunda conta/dispositivo durante dois ciclos completos |

## 8. Registro de decisão

| Data | Decisão | Motivo |
| --- | --- | --- |
| 2026-09-14 | Planejar somente estímulo de VDI em segundo plano | O Teams roda dentro da VDI; o usuário precisa continuar lendo/trabalhando em outra janela local sem perder foco |
| 2026-09-14 | Foco temporário da VDI e restauração da janela anterior aprovados | O usuário aceitou esse fallback; meta inferior a um segundo, sem promessa de garantia pelo Windows |

## 9. Registro da Fase 0

| Data | Etapa | Resultado | Impacto |
| --- | --- | --- | --- |
| 2026-09-14 | Descoberta de janelas para o ensaio Windows real | A superfície de automação expôs zero aplicativos nativos e não oferece API para selecionar o Windows App. A inspeção não interativa confirma `msrdc` ativo; captura do usuário confirma a janela `ELS-Windows365-US East with M365 Apps - Carvalho, Andre Teixeira (ELS-CON)` aberta. | O bloqueio é da ponte de automação, não da VDI. Nenhum estímulo, mudança de foco, abertura de VDI ou interação com Teams ocorreu. AC21–AC24 continuam pendentes até que a automação consiga vincular essa janela existente. |
