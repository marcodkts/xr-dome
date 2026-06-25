# Roadmap — XR Dome Desktop

## 1. Propósito

Construir uma nova interface de área de trabalho espacial para Linux, capaz de organizar aplicações e superfícies em um ambiente curvo ou em formato de domo.

O sistema deve funcionar primeiro em um monitor comum, com mouse e teclado, e aceitar rastreamento **3DoF** como uma forma opcional de orientação. Óculos como o VITURE Pro são uma integração desejada, mas não podem ser uma dependência para desenvolver, testar ou utilizar o núcleo do projeto.

Este é um documento vivo. Ele registra a direção atual, as hipóteses e os critérios de validação, mas não congela decisões. Novas descobertas podem alterar, dividir, reordenar ou remover fases.

## 2. Visão do produto

O objetivo final é oferecer uma sessão gráfica espacial na qual aplicações Linux convencionais possam ser abertas, posicionadas e utilizadas dentro de um ambiente tridimensional.

O usuário deverá poder:

- executar o ambiente sem possuir óculos XR;
- olhar ao redor com mouse, controle, teclado ou rastreador 3DoF;
- organizar janelas em um arco, cilindro, domo ou combinação de superfícies;
- usar aplicações Wayland e, quando possível, aplicações X11 por XWayland;
- interagir com as janelas usando teclado e ponteiro;
- recentralizar e calibrar a orientação;
- utilizar uma saída comum ou estereoscópica, conforme o dispositivo;
- selecionar futuramente o XR Dome como uma sessão gráfica no login do Linux.

## 3. Princípios de desenvolvimento

1. **Independência de hardware:** o projeto precisa funcionar sem VITURE ou qualquer outro dispositivo XR.
2. **Validação progressiva:** cada fase deve responder a uma dúvida importante antes de aumentar a complexidade.
3. **Execução segura:** o compositor será executado inicialmente dentro do desktop atual. A sessão principal só será substituída quando houver recuperação confiável.
4. **Arquitetura modular:** renderização, rastreamento, entrada, composição e integração de dispositivos não devem depender diretamente uns dos outros.
5. **Fallback sempre disponível:** falha no rastreador deve devolver o controle ao mouse ou a outra fonte de orientação.
6. **Desempenho perceptível:** fluidez, baixa latência e legibilidade são requisitos do produto, não otimizações tardias.
7. **Decisões reversíveis:** experimentos devem permanecer descartáveis até serem validados.
8. **Evidência antes de compromisso:** uma tecnologia será adotada depois de um protótipo demonstrar que atende ao caso real.

## 4. Escopo inicial e limites

### Incluído

- renderização de um ambiente espacial;
- painéis planos e curvos;
- navegação por mouse e teclado;
- abstração de orientação 3DoF;
- suporte futuro a VITURE por integração opcional;
- aplicações reais executadas por um compositor Wayland aninhado;
- evolução para uma sessão gráfica completa.

### Fora do escopo inicial

- rastreamento posicional 6DoF;
- gestos e rastreamento de mãos;
- realidade aumentada com câmeras;
- compatibilidade imediata com todo hardware XR;
- substituir o GNOME durante as primeiras fases;
- criar versões para Windows ou macOS antes da validação no Linux.

Esses itens podem entrar futuramente por meio de uma decisão registrada.

## 5. Arquitetura pretendida

```text
Fontes de orientação
  mouse | teclado | controle | VITURE | OpenXR futuro
                       │
                       ▼
                 Orientation API
                       │
                       ▼
Aplicações ──► Compositor/Surface Manager ──► Cena espacial
 Wayland              │                           │
 XWayland              └── entrada/foco           ▼
                                               Renderer
                                                  │
                                  monitor | fullscreen | SBS/XR
```

Módulos inicialmente previstos:

```text
xr-dome/
├── app                 Ciclo de vida e configuração
├── renderer            Cena 3D e composição final
├── spatial             Transformações, painéis e ray casting
├── orientation         Contrato comum para fontes 3DoF
├── input               Mouse, teclado, foco e atalhos
├── wayland             Compositor aninhado e superfícies
├── xwayland            Compatibilidade legada, quando necessária
├── output              Janela, fullscreen e saída estereoscópica
└── integrations
    └── viture          Integração opcional com o SDK
```

A divisão final em crates só deverá acontecer quando os limites entre os módulos estiverem comprovados. No início, módulos Rust dentro de um único crate reduzem complexidade.

## 6. Fases e gates de validação

As fases expressam dependências lógicas, não datas. Atividades podem ocorrer em paralelo quando isso não esconder uma hipótese ainda não validada.

### Fase 0 — Fundação e ambiente reproduzível

Objetivo: permitir que qualquer colaborador execute o mesmo experimento.

- [ ] Criar o repositório Rust e definir a versão mínima do toolchain.
- [ ] Adicionar formatação, lint e testes ao fluxo local.
- [ ] Documentar dependências Linux, GPU e comandos de execução.
- [ ] Registrar informações de GPU, backend gráfico e sessão X11/Wayland nos logs.
- [ ] Criar uma configuração simples e versionada com valores padrão.
- [ ] Adotar ADRs para decisões arquiteturais relevantes.

**Gate:** uma instalação limpa consegue compilar e abrir a janela de demonstração seguindo apenas a documentação.

### Fase 1 — Prova visual do ambiente espacial

Objetivo: validar a geometria e a sensação básica de navegação sem depender do desktop ou de óculos.

- [ ] Renderizar uma grade em painel cilíndrico ou domo usando GPU.
- [ ] Implementar câmera com yaw, pitch e roll.
- [ ] Navegar segurando e arrastando o mouse.
- [ ] Adicionar recentralização e limites configuráveis de pitch.
- [ ] Redimensionar a janela sem distorções indevidas.
- [ ] Exibir FPS, tempo de frame e backend gráfico em modo de diagnóstico.
- [ ] Comparar painel plano, cilindro e domo quanto à legibilidade.

**Gate:** o usuário consegue navegar por pelo menos alguns minutos sem comportamento desorientador, e a equipe escolhe uma geometria inicial sem torná-la permanente.

### Fase 2 — Painéis espaciais e interação

Objetivo: deixar de tratar o ambiente como uma única textura e passar a trabalhar com superfícies independentes.

- [ ] Criar, mover, redimensionar e remover painéis.
- [ ] Definir transformações locais e globais para cada painel.
- [ ] Implementar seleção por ray casting.
- [ ] Converter a posição do ponteiro em coordenadas locais do painel.
- [ ] Definir foco visual e foco de teclado.
- [ ] Persistir e restaurar um layout experimental.
- [ ] Avaliar painéis planos distribuídos em arco versus uma superfície contínua.

**Gate:** o usuário consegue identificar, selecionar e reorganizar painéis sem ambiguidades frequentes.

### Fase 3 — Conteúdo real e captura exploratória

Objetivo: validar atualização de conteúdo, nitidez e latência antes de construir um compositor completo.

- [ ] Aplicar imagens e texturas dinâmicas aos painéis.
- [ ] Testar captura de uma janela ou desktop por mecanismo apropriado à sessão.
- [ ] Medir custo de cópia entre captura e GPU.
- [ ] Verificar legibilidade de texto em diferentes distâncias e curvaturas.
- [ ] Identificar recursão de captura e outros problemas do modo fullscreen.
- [ ] Usar a captura apenas como experimento, sem assumi-la como arquitetura final.

**Gate:** existem dados suficientes para decidir se a captura continuará como modo auxiliar ou se o caminho principal seguirá diretamente para composição Wayland.

### Fase 4 — Compositor Wayland aninhado

Objetivo: executar aplicações reais dentro do XR Dome sem substituir a sessão atual.

- [ ] Criar uma prova de conceito com Smithay ou alternativa avaliada por ADR.
- [ ] Executar o compositor em uma janela do GNOME ou de outra sessão existente.
- [ ] Abrir pelo menos uma aplicação Wayland real dentro do compositor.
- [ ] Receber buffers das aplicações e apresentá-los como painéis espaciais.
- [ ] Encaminhar foco, teclado, movimento, clique e rolagem.
- [ ] Implementar fechamento e mudança de tamanho de superfícies.
- [ ] Avaliar importação de buffers e cópias entre o compositor e o renderer.
- [ ] Registrar limitações encontradas na integração com `wgpu`.

**Gate:** uma aplicação como terminal ou navegador pode ser aberta e utilizada dentro do domo, com entrada funcional e sem depender de captura da tela externa.

### Fase 5 — Gerenciamento espacial de janelas

Objetivo: transformar o protótipo em uma interface de trabalho coerente.

- [ ] Definir comportamento de novas janelas, diálogos e janelas filhas.
- [ ] Criar alternância de foco e visão geral das aplicações.
- [ ] Adicionar encaixe, agrupamento e layouts espaciais.
- [ ] Definir distância, escala e curvatura por painel.
- [ ] Criar atalhos para centralizar, aproximar, afastar e recuperar janelas perdidas.
- [ ] Tratar notificações sem interromper a orientação do usuário.
- [ ] Testar sessões reais de desenvolvimento, navegação e terminal.
- [ ] Incluir recursos básicos de acessibilidade e escala de interface.

**Gate:** o ambiente suporta uma tarefa real por um período prolongado sem exigir retorno constante ao desktop hospedeiro.

### Fase 6 — Saídas e renderização estereoscópica

Objetivo: separar o conteúdo espacial da forma como ele é apresentado.

- [ ] Manter saída monoscópica em janela como referência obrigatória.
- [ ] Implementar fullscreen selecionando explicitamente o monitor.
- [ ] Implementar duas câmeras com distância interpupilar configurável.
- [ ] Compor saída Full SBS para dispositivos compatíveis.
- [ ] Permitir ajustar FOV, escala, distância e convergência com segurança.
- [ ] Validar que o modo estereoscópico possa ser desativado sem reiniciar o núcleo.

**Gate:** a mesma cena funciona em monitor normal e em saída SBS sem alterar o compositor ou o modelo de janelas.

### Fase 7 — Orientação 3DoF e integração VITURE

Objetivo: adicionar rastreamento sem criar dependência de um fabricante.

- [ ] Estabilizar o contrato `OrientationSource`.
- [ ] Criar uma fonte simulada para testes automatizados.
- [ ] Implementar suavização, recentralização e compensação de drift.
- [ ] Definir política para perda ou congelamento do rastreador.
- [ ] Executar o quick start oficial do VITURE como diagnóstico de hardware.
- [ ] Criar bindings FFI isolados para o SDK C.
- [ ] Implementar a fonte `VitureOrientation` como feature opcional.
- [ ] Carregar a integração dinamicamente quando isso melhorar a distribuição.
- [ ] Comparar rastreamento direto, smooth follow e diferentes filtros.
- [ ] Garantir fallback imediato para mouse/controle.

**Gate:** conectar ou desconectar o VITURE não altera o funcionamento básico, e o movimento de cabeça controla a câmera com estabilidade aceitável.

### Fase 8 — Compatibilidade e robustez

Objetivo: ampliar o conjunto de aplicações e reduzir falhas que inviabilizam uso diário.

- [ ] Avaliar e integrar XWayland.
- [ ] Tratar múltiplas janelas, pop-ups, clipboard e drag-and-drop.
- [ ] Implementar recuperação após falha do renderer ou troca de monitor.
- [ ] Salvar estado sem corromper layouts.
- [ ] Testar diferentes GPUs e drivers Linux.
- [ ] Verificar comportamento com suspensão, retorno e hotplug.
- [ ] Criar uma tela ou modo seguro de recuperação.
- [ ] Revisar limites de confiança entre clientes Wayland e compositor.

**Gate:** os principais fluxos sobrevivem a falhas comuns e um conjunto representativo de aplicações funciona de maneira previsível.

### Fase 9 — Sessão gráfica completa

Objetivo: permitir iniciar o XR Dome diretamente pelo gerenciador de login.

- [ ] Criar backend para DRM/KMS, input e gerenciamento de sessão.
- [ ] Detectar e configurar monitores com segurança.
- [ ] Criar entrada de sessão para o display manager.
- [ ] Garantir uma forma documentada de retornar ao GNOME ou a um TTY.
- [ ] Tratar bloqueio de tela, suspensão e encerramento de sessão.
- [ ] Definir componentes mínimos de shell: launcher, status, configurações e saída.
- [ ] Testar inicialmente em usuário separado ou equipamento não crítico.

**Gate:** a sessão inicia, executa aplicações, encerra corretamente e nunca impede o usuário de retornar a uma sessão conhecida.

### Fase 10 — Distribuição e evolução

Objetivo: tornar os testes externos e a evolução sustentáveis.

- [ ] Definir formato de distribuição inicial.
- [ ] Produzir builds reproduzíveis e checksums.
- [ ] Separar configurações estáveis de opções experimentais.
- [ ] Coletar diagnóstico somente com consentimento explícito.
- [ ] Publicar uma matriz de hardware testado.
- [ ] Documentar APIs para novos rastreadores e modos de saída.
- [ ] Manter migrações para configurações e layouts persistidos.

**Gate:** outra pessoa consegue instalar, executar, diagnosticar e remover o projeto sem assistência direta.

## 7. Estratégia de testes

O código gráfico também deve ser desenvolvido com testes proporcionais ao risco.

### Testes unitários

- conversões entre yaw, pitch, roll e quaternions;
- matrizes de câmera e projeção;
- geração das malhas do domo;
- interseção do raio com painéis planos e curvos;
- transformação entre coordenadas da tela, mundo e aplicação;
- filtros, recentralização e detecção de amostras inválidas;
- serialização e migração de layouts.

### Testes de integração

- criação e destruição de superfícies Wayland;
- foco e encaminhamento de eventos de entrada;
- resize, pop-ups e múltiplas janelas;
- conexão e perda de uma fonte de orientação;
- troca de modo monoscópico para estereoscópico;
- recuperação após recriação da superfície gráfica.

### Testes visuais e de desempenho

- imagens de referência para cenas determinísticas;
- verificação de legibilidade em diferentes escalas;
- tempo de CPU e GPU por frame;
- frames perdidos e estabilidade de frame pacing;
- latência entre orientação e apresentação;
- cópias de buffer e uso de memória gráfica.

Os valores de desempenho serão tratados inicialmente como metas revisáveis. Como referência, devemos buscar 60 FPS estáveis em hardware suportado e preservar um caminho para 90/120 Hz, sem transformar números prematuros em promessa de produto.

## 8. Decisões em aberto

Cada decisão relevante deve ser registrada em `docs/adr/` com contexto, alternativas, consequências e condição de revisão.

| Decisão | Estado inicial | Experimento necessário |
|---|---|---|
| Domo contínuo ou painéis em arco | Em exploração | Comparar legibilidade e navegação |
| `wgpu` como renderer final | Hipótese | Validar integração eficiente com buffers Wayland |
| Smithay como base do compositor | Hipótese forte | Abrir e controlar aplicação em modo aninhado |
| Captura de desktop | Apenas experimento | Medir latência, cópias e limitações de entrada |
| Estratégia de ponteiro espacial | Em aberto | Comparar cursor 2D, ray casting e cursor central |
| Persistência de layouts | Em aberto | Definir modelo somente após painéis estabilizarem |
| Distribuição do SDK VITURE | Em aberto | Revisar binários, licença e carregamento dinâmico |
| X11 como alvo principal | Não recomendado | Priorizar Wayland e usar XWayland quando necessário |

Estados sugeridos para decisões: `ideia`, `em exploração`, `validada`, `adotada`, `substituída` e `revisar`.

## 9. Riscos conhecidos

| Risco | Impacto | Mitigação inicial |
|---|---|---|
| Integração entre buffers Wayland e `wgpu` exigir cópias caras | Latência e consumo de GPU | Protótipo antecipado na Fase 4 e ADR do renderer |
| Texto perder legibilidade em superfícies curvas | Ambiente impraticável para trabalho | Testar painéis planos, escala e distância desde a Fase 1 |
| Mapeamento do cursor ser confuso | Interação imprecisa | Ray casting, feedback visual e atalhos de recuperação |
| Drift ou ruído do 3DoF | Náusea e instabilidade | Filtros, recenter, smooth follow e fallback |
| Aplicações dependerem de protocolos Wayland não implementados | Compatibilidade limitada | Matriz de aplicações e implementação incremental |
| Substituição prematura da sessão gráfica | Usuário sem desktop funcional | Permanecer aninhado até a Fase 9 e manter sessão alternativa |
| Escopo crescer para um desktop completo cedo demais | Projeto não chega a um protótipo utilizável | Gates pequenos e tarefas reais de validação |

## 10. Regras para alterar este roadmap

- Uma fase pode ser dividida, reordenada ou removida quando houver justificativa.
- Checkboxes indicam progresso observado, não compromisso de prazo.
- Gates indicam perguntas a responder, não burocracia obrigatória.
- Toda decisão que mudar a arquitetura deve registrar o motivo e o que invalidou a hipótese anterior.
- Funcionalidades novas entram primeiro como hipótese ou experimento.
- Uma integração de hardware nunca deve quebrar o funcionamento sem hardware.
- O objetivo final pode evoluir, mas a compatibilidade 3DoF e o modo sem óculos devem permanecer como capacidades centrais enquanto não houver decisão explícita em contrário.

## 11. Próximas ações

1. Executar e corrigir o protótipo atual de domo com `wgpu`.
2. Adicionar diagnóstico de GPU, FPS e frame time.
3. Implementar recentralização e configuração de FOV.
4. Criar dois painéis independentes com seleção por mouse.
5. Registrar o primeiro ADR: geometria inicial e papel do `wgpu`.
6. Só então iniciar o experimento com conteúdo real e compositor aninhado.

## 12. Registro de evolução

| Data | Alteração |
|---|---|
| 2026-06-25 | Roadmap inicial criado com desenvolvimento progressivo, modo sem óculos e compatibilidade 3DoF como objetivos centrais. |
