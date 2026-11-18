# Odina Antivirus — Português do Brasil (pt-BR)
# المستوى: T-B (مجتمعية) — تحتاج مراجعة ناطق أصلي قبل ترقيتها إلى T-A (§30.2)
# راجع كل سطر يدويًا قبل الاعتماد. لا ترجمة آلية للنصوص الأمنية (§3.9.1).

app.name = Odina
app.tagline = Seu guardião digital
app.window.title = Odina — { $state }
state.protected = Você está protegido
state.scanning = Verificando…
state.attention = Precisa da sua atenção
state.paused = Proteção pausada
state.untrusted = Ainda não verificado
wall.signatures.title = Banco de arquivos
wall.signatures.on = Ativo
wall.signatures.off = Inativo
wall.behavior.title = Verificação profunda de comportamento
wall.behavior.on = Escutando
wall.behavior.off = Inativa
wall.proactive.title = Proteção proativa
wall.proactive.on = Ativa
wall.proactive.off = Desativada
action.scan_now = Verificar agora
action.pause = Pausar
action.expert_mode = Modo especialista
action.language = Idioma
action.settings = Configurações
action.why = Por quê?
scan.quick = Verificação rápida
scan.full = Verificação completa
scan.usb = Verificar esta unidade
scan.running = Verificando { $target }…
scan.percent = { $percent }%
scan.files_checked = Arquivos verificados
scan.threats_found = Ameaças encontradas
scan.stop = Parar
scan.complete = Verificação concluída
scan.clean_result = Nada prejudicial foi encontrado.
alert.confirmed.title = Paramos uma ameaça conhecida
alert.confirmed.body = { $file } é uma ameaça confirmada e foi isolado.
alert.suspicious.title = Um programa está tentando acessar seus arquivos
alert.suspicious.body = { $file } está alterando arquivos em Documentos. Nós o paramos temporariamente.
alert.suspicious.note = Nós o paramos temporariamente.
alert.unsure.title = Este programa é incomum
alert.unsure.body = { $file } é novo aqui e se comporta de forma estranha. Você o conhece?
alert.pua.title = Parece uma ferramenta de crack
alert.pua.body = { $file } contorna a proteção de licença de softwares pagos. Você decide o que roda no seu dispositivo.
alert.pua.allowed = Permitir mesmo assim
alert.pua.quarantine = Mover para o lado
button.allow = Permitir
button.block = Bloquear
button.restore = Restaurar
button.remove_forever = Excluir permanentemente
usb.detected = Nova unidade detectada — verificamos primeiro
usb.scanning = Verificando { $name }…
usb.priority_done = Arquivos perigosos verificados
usb.trust.new = Unidade nova
usb.trust.scanning = Verificando
usb.trust.trusted = Lembrada
usb.trust.suspicious = Sob vigilância
usb.trust.quarantined = Bloqueada
quarantine.title = Itens isolados
quarantine.reason = Por que foi isolado
quarantine.restore_note = O local original é preservado.
quarantine.retention = Mantido por { $days } dias
pause.title = Pausar a proteção
pause.duration.10 = 10 minutos
pause.duration.30 = 30 minutos
pause.duration.60 = 1 hora
pause.warning = Você ficará desprotegido até { $until }.
pause.confirm = Pausar mesmo assim
pause.badge = Avisos pausados · a proteção continua
error.generic = Algo deu errado. Não alteramos nada.
error.tamper = Os arquivos do Odina foram modificados. É preciso reparar.
error.offline = Ainda sem atualização. A proteção continua.
update.available = Atualização disponível
update.ask = Verificar e baixar atualizações?
update.db_age_warning = Sua lista de ameaças tem { $days } dias.

## ── Path ──────────────────────────────────────────────
path.label = Caminho
path.copy = Copiar caminho
path.open_folder = Abrir pasta
trust.title = Fundamentos da confiança
word.isolated = isolado
word.temporary = temporariamente
tip.quarantine = Movido para um armazenamento cifrado — o local original é mantido.
tip.suspended = Nós o paramos agora. Só volta a funcionar se você permitir.
quarantine.hint = Tudo fica até você decidir · nunca é excluído sozinho
quarantine.keep = Manter
quarantine.delete = Excluir permanentemente
quarantine.delete_note = Não pode ser desfeito. Não há outra cópia.
quarantine.cleanup_title = Em quarentena há { $days } dias
quarantine.cleanup_choose = Excluir os que eu selecionei
error.false_positive = Erramos sobre este arquivo — corrigido.
error.locked = Não foi possível isolar: o arquivo está aberto agora.

## ── Turn protection off (user's explicit decision) ────────
pause.confirm_disable = Desligar a proteção por { $minutes} minutos
disable.title = Desligar a proteção
disable.warning_title = Risco grave
disable.body = Não paramos só os avisos — paramos de proteger. Nenhum arquivo será analisado e nenhuma ameaça será bloqueada.
disable.responsibility = A decisão é sua, e a responsabilidade é sua durante esse período.
disable.choose = Escolha a duração
disable.duration.peak = Até o fim do horário de pico
disable.duration.30 = 30 minutos
disable.duration.60 = 1 hora
disable.duration.until_reboot = Até eu reiniciar
disable.refused = Não é possível desligar a proteção agora
disable.refused_quarantine = Há um arquivo em quarentena sem decisão. Veja-o primeiro.
disable.refused_scanning = Uma verificação está em andamento. Aguarde ou pare.
disable.refused_active_threat = Há uma ameaça não resolvida em aberto. Não desligamos a proteção enquanto estiver.
state.unprotected = Proteção desligada
state.unprotected_until = Proteção desligada até { $until}
state.unprotected_badge = Sem proteção — sua decisão
action.turn_off = Desligar a proteção
action.turn_on = Religar a proteção
disable.re_enable_note = Você pode religar a proteção a qualquer momento — sem senha.
disable.reminder = Faltam { $minutes} minutos para a proteção voltar sozinha.
pause.resume_report = Enquanto os avisos estavam pausados: { $count} arquivo(s) isolado(s)

## ── Exceção cirúrgica (D45 · §4.8) ─────────────────────────
exception.title = Exceções
exception.sub = Se o seu trabalho precisa que uma regra ceda, permita apenas essa. O resto fica como está.
exception.rule.a1.wsh = Scripts do Windows
exception.rule.a1.wsh.desc = Arquivos .vbs e .js rodam restritos, a menos que assinados por um editor conhecido.
exception.rule.a1.powershell = Comandos do PowerShell
exception.rule.a1.powershell.desc = O PowerShell roda em modo restrito. Permitir não abre o Windows Script Host.
exception.rule.a1.macros = Macros do Office
exception.rule.a1.macros.desc = Documentos não assinados não podem executar macros. Permitir não afeta os scripts.
exception.rule.a3.downloads = Suspeita em Downloads
exception.rule.a3.downloads.desc = Cada novo arquivo em Downloads é suspeito até ser liberado.
exception.rule.a4.periodic = Verificação periódica
exception.rule.a4.periodic.desc = Verificação silenciosa de itens de inicialização e chaves sensíveis a cada duas horas.
exception.rule.a6.doubt = Cautela na dúvida
exception.rule.a6.doubt.desc = Na dúvida, bloqueia e só depois pergunta. A regra pode ser relaxada, nunca liberada.
exception.state.on = Ativa
exception.state.relaxed_until = Flexibilizada até { $until }
exception.state.allowed_until = Permitida até { $until }
exception.state.allowed_permanent = Permitida permanentemente — sua decisão
exception.consequence.relax = { $rule } funcionará sem avisos até { $until} e depois voltará. Nada é bloqueado nesse período.
exception.consequence.allow = { $rule } funcionará sem restrições até { $until} e depois voltará. Nada mais muda.
exception.consequence.permanent = { $rule } continuará permitida até você revogar. Isso permanecerá escrito na tela inicial.
exception.action.relax_10 = Flexibilizar 10 minutos
exception.action.relax_30 = Flexibilizar 30 minutos
exception.action.relax_60 = Flexibilizar uma hora
exception.action.allow = Permitir
exception.action.revoke = Revogar
exception.action.revoke_all = Revogar todas as exceções
exception.action.advanced = Opções avançadas
exception.badge = { $rule } permitida · até { $until }
exception.badge_permanent = { $rule } permitida permanentemente · sua decisão
exception.badge_many = { $count } regras permitidas pela sua decisão · detalhes
exception.until_reboot = até reiniciar
exception.permanent_note = Uma exceção permanente não termina sozinha. Ela fica visível até você revogar.
exception.revoke_note = Você pode revogar a qualquer momento — um clique, sem senha.
exception.expert_note = O modo especialista revela “permanente” e “a regra inteira”. Sem senha e sem perguntar o motivo.
exception.expired_report = A exceção terminou. Durante ela, { $count } arquivo(s) pareceram suspeitos — revise agora.
exception.expired_clean = A exceção terminou. Nada apareceu que precise de revisão.
exception.refused.not_allowed = Esta regra não pode ser permitida por completo — apenas flexibilizada.
exception.refused.above_ceiling = O máximo desta regra é a flexibilização.
exception.refused.no_permanent = Esta regra não admite exceção permanente.
exception.refused.expert_only = Uma exceção permanente de toda a regra exige o modo especialista.
exception.refused.veto_stands = O arquivo está confirmado. Uma exceção flexibiliza a regra; não inocenta um arquivo.

exception.live_note = sua decisão · todo o resto continua ativo · você pode revogar a qualquer momento
exception.lines_title = Quatro coisas nunca são excetuadas:
exception.lines_body = o veredito do primeiro muro (um arquivo confirmado é isolado mesmo sob exceção) · a proteção contra adulteração · a segunda regra (uma exceção de WSH não abre o PowerShell) · “risco alto” e acima. Revogar é sempre um clique, sem senha.

## ── Mapa do aplicativo e navegação (constituição §6أ.3) ─────────────────
nav.home = Início
nav.quarantine = Quarentena
nav.proactive = Proteção proativa
nav.alerts = Central de avisos
nav.settings = Configurações
nav.language = Idioma
nav.privacy = Privacidade
back = Voltar
menu.more = Mais
menu.advanced = Opções avançadas
map.title = Mapa do aplicativo
map.hint = Como cada tela é alcançada · no máximo três níveis

## ── Política de assinatura de código (página pública) ─────────────────────────
signing.title = Política de assinatura de código
signing.sub = O que assinamos, o que nunca assinamos, onde fica a chave e como verificar você mesmo.
signing.status_title = Situação hoje
signing.status_body = Esta política é publicada antes do início da assinatura, e nenhuma compilação foi assinada ainda. A primeira compilação assinada será anunciada na página da versão junto com o hash.
signing.status_until = Até então o programa é distribuído como arquivo ZIP portátil, nunca como instalador sem assinatura.
signing.build_title = Compilação
signing.build_body = As compilações rodam no GitHub Actions a partir do commit indicado nas notas da versão. Não há compilação local, e um binário nunca é recompilado antes da assinatura.
signing.key_title = A chave
signing.key_body = Não guardamos nenhuma chave de assinatura. A chave fica no HSM da SignPath Foundation, e cada pedido de assinatura exige aprovação explícita de um mantenedor.
signing.signed_title = Assinado
signing.signed_installer = O instalador
signing.signed_store = O pacote da loja
signing.unsigned_title = Não assinado
signing.unsigned_db = As bases de assinaturas — assinadas com uma chave Ed25519 separada, porque são atualizadas muito mais que o programa.
signing.unsigned_tools = Tudo em tools — licenciado em MIT e distribuído como código-fonte, nunca como binário.
signing.verify_title = Verificação
signing.verify_body = Um SHA-256 de cada artefato é publicado na página da versão, antes e depois da assinatura. Você pode verificar no Windows:
signing.verify_must = O status precisa ser Valid e o editor precisa ser SignPath Foundation. Se o hash diferir do publicado, não execute o arquivo.
signing.revoke_title = Revogação
signing.revoke_body = Se alguma compilação vier de código não revisado, ela é revogada e a revogação é anunciada antes de qualquer outra coisa.
signing.credit = Assinatura de código gratuita fornecida pela SignPath.io, certificado pela SignPath Foundation.
signing.updated = Última atualização

## ── Glossário — uma linha por termo ─────────────
tip.confirmed = O hash dele está na base de ameaças. Não é um palpite.
tip.high_risk = Sinais fortes de perigo, e o hash ainda não foi confirmado.
tip.suspicious = Parece estranho: o nome, o lugar ou o comportamento. Não verificado.
tip.pua = Não é um vírus, mas contorna a proteção de outro programa.
tip.signature = Hashes de arquivos conhecidos. Atualizá-los mantém a proteção em dia.
tip.behavior = Observa o que o programa faz, não como ele se chama.
tip.proactive = Impede o ato antes que aconteça, em vez de achá-lo depois.
tip.pause = Silenciamos apenas as janelas. A verificação e o bloqueio continuam.
tip.turn_off = Sem verificação e sem bloqueio. Seu dispositivo fica desprotegido nesse tempo.
tip.exception = Flexibilizar uma regra pela sua decisão. Nada mais muda.
tip.expert_mode = Revela as opções permanente e de regra inteira. Sem senha nem perguntas.
tip.wsh = O host de scripts do Windows. Um caminho comum para malwares.
tip.powershell = Uma ferramenta de comandos no Windows. Restrita por padrão a partir de fontes de risco.
tip.macros = Comandos dentro de um documento do Office. Não rodam em arquivos sem assinatura.
tip.amsi = Uma interface do Windows que nos deixa ver um script antes de ele rodar.
tip.wdac = Uma política do Windows que impede um arquivo específico de rodar.
tip.usb_trust = A primeira vez que o vemos. Verificado antes de ser aberto.
tip.tamper = O Odina não pode ser desligado editando seus arquivos ou encerrando seu serviço.
tip.hash = Uma string que representa o arquivo inteiro. Um byte muda tudo.
tip.heuristic = Um sinal medido e ponderado. Nunca basta sozinho.

## ── Configurações, privacidade e idioma ─────────────────────────────
settings.title = Configurações
settings.sub = Quatro seções. Nada escondido atrás de “avançado” além do raro.
settings.appearance = Aparência
settings.appearance.body = Claro ou escuro. Segue o sistema por padrão.
settings.theme_light = Claro
settings.theme_dark = Escuro
settings.theme_system = Seguir o sistema
settings.behavior = Comportamento
settings.startup = Inicia com o Windows
settings.startup.body = Roda em segundo plano desde a inicialização. Você pode desligar.
settings.notifications = Avisos
settings.notifications.body = Três por dia no máximo. O mais importante primeiro.
settings.updates = Atualizações
settings.updates.body = Automáticas para a base de ameaças, manuais para o programa.
settings.about = Sobre
settings.version = Versão
settings.database = Base de ameaças
settings.database.date = Última atualização

privacy.title = Privacidade
privacy.sub = O que sai do seu dispositivo e o que nunca sai. Sem exceções.
privacy.telemetry = Zero telemetria
privacy.telemetry.body = Sem estatísticas, sem id de dispositivo, sem impressão digital. Nada é enviado sozinho.
privacy.local = Tudo local
privacy.local.body = Análise, decisão e registro ficam no seu disco.
privacy.report = O relatório é sua decisão
privacy.report.body = Um relatório só é enviado se você escolher enviá-lo.
privacy.search = Busca sob demanda
privacy.search.body = Abre um buscador no seu idioma. Enviamos só o termo, nunca seus arquivos.
privacy.log = O registro
privacy.log.body = Local e legível no seu idioma. Você pode apagá-lo quando quiser.
privacy.log_clear = Apagar o registro

lang.title = Idioma
lang.sub = Dez idiomas. A tradução aparece na hora e a fonte segue o seu idioma.
lang.current = Idioma atual
lang.rtl = Escrito da direita para a esquerda
lang.ltr = Escrito da esquerda para a direita
lang.review = Revisão nativa
lang.review.body = Um idioma sem revisor nativo é suspenso, não publicado.

home.walls = Os três muros
home.action.scan = Verificar agora
home.action.pause = Pausar
home.action.expert = Modo especialista
home.action.why = Por quê?
home.goto.quarantine = Quarentena
home.goto.alerts = Central de avisos
home.goto.settings = Configurações
home.goto.proactive = Proteção proativa
home.quarantine_count = { $count } arquivo(s) em quarentena
home.alerts_count = { $count } aviso(s) que merecem um olhar
home.proactive_summary = Nove atividades em três grupos
proactive.title = Proteção proativa
proactive.sub = Permitir uma atividade aqui não ignora a verificação de arquivos nem as regras do firewall.
proactive.tier.mandatory = Risco muito alto · Bloqueio automático
proactive.tier.recommended = Risco médio · Bloqueio recomendado
proactive.tier.optional = Risco menor · Bloqueio opcional
proactive.surface.office_child = Interpretadores iniciados pelo Office
proactive.surface.office_child.tip = Word ou Excel iniciando cmd, PowerShell ou mshta, não todos os processos filhos do Office.
proactive.surface.wmi_subscription = Assinaturas WMI persistentes
proactive.surface.wmi_subscription.tip = Novas assinaturas permanentes de eventos de origem não confiável, não consultas WMI.
proactive.surface.system_tools = Ferramentas com entradas não confiáveis
proactive.surface.system_tools.tip = mshta, certutil ou regsvr32 com dados não confiáveis de pasta temporária ou da internet.
proactive.surface.scheduled_task = Tarefas agendadas
proactive.surface.scheduled_task.tip = Criar ou alterar uma tarefa. O bloqueio não interrompe tarefas existentes.
proactive.surface.com_registration = Registro de componentes COM
proactive.surface.com_registration.tip = Alterações no registro COM por usuário, sem desativar o COM no Windows.
proactive.surface.registry_autorun = Entradas de inicialização
proactive.surface.registry_autorun.tip = Adicionar ou alterar entradas do registro em Run ou RunOnce.
proactive.surface.service_driver = Instalação de serviços e drivers
proactive.surface.service_driver.tip = Criar um serviço ou instalar um driver, sem interromper os que estão em execução.
proactive.surface.inbound_smb_rdp = SMB e RDP de entrada
proactive.surface.inbound_smb_rdp.tip = Novas conexões de entrada para compartilhamento ou acesso remoto. Não abre portas.
proactive.surface.wsh_script = Scripts do Windows
proactive.surface.wsh_script.tip = Execução de .js, .vbs ou .wsf pelo WSH. As outras verificações de scripts continuam.
proactive.state.blocked = Bloqueado
proactive.state.allowed = Permitido
proactive.state.contained = Bloqueio temporário
proactive.state.off = Não aplicado
proactive.saved_until = { $rule } · exceção salva até { $until }
proactive.user_wsh_block = O bloqueio de scripts do Windows que você escolheu prevalece sobre esta exceção.
proactive.demo.title = Simulação de contenção
proactive.demo.note = Simulação local. Nenhum arquivo é verificado e nenhum ajuste do Windows é alterado.
proactive.demo.mode = Modo de proteção simulado
proactive.demo.ready = Quando a verificação confirma uma infecção, as nove atividades ficam bloqueadas até o reparo terminar com sucesso.
proactive.demo.start = Simular infecção confirmada
proactive.demo.scan = Verificando infecção confirmada · as nove atividades estão bloqueadas temporariamente.
proactive.demo.clean = Limpando · verificação concluída, bloqueio mantido.
proactive.demo.repair = Reparando · limpeza concluída, bloqueio mantido.
proactive.demo.cleaned = Limpeza concluída
proactive.demo.repaired = Reparo concluído
proactive.demo.restored = Reparo concluído. O bloqueio temporário terminou e suas escolhas anteriores voltaram a valer.
proactive.demo.off = Você desativou a proteção. A contenção não inicia e suas escolhas não mudam.
proactive.demo.fail = Falha na etapa
proactive.demo.failed = A etapa falhou. A contenção continua até a recuperação ser concluída.
alerts.title = Central de avisos
alerts.sub = Tudo o que merece um olhar em um só lugar. Sem janelas surgindo.
alerts.empty = Nada que mereça um olhar agora.
exception.detail.title = Exceção — uma regra
exception.detail.sub = O que muda, por quanto tempo e como volta.
map.node.home = Início
map.node.quarantine = Quarentena
map.node.proactive = Proativa
map.edge.from_home = do Início
map.edge.from_alert = de um aviso
map.legend = Linha cheia é entrada permanente · pontilhada precisa de contexto

## ── Explicação sob demanda ─────────────────────────────
help.title = Explicação sob demanda
help.hint = Uma linha ao passar · uma busca completa ao clicar, no seu idioma.
help.tip.title = Dica imediata
help.tip.body = Uma palavra sublinhada e uma linha de no máximo 90 caracteres.
help.guide.title = Um guia mais longo
help.guide.body = Abre por um botão dentro da dica e se lê num buscador que você conhece.
help.guide.cta = Abrir a busca
help.engine.title = O buscador segue idioma e lugar
help.col.lang = Idioma
help.col.engine = Buscador
help.col.why = Motivo
help.why.default = A maior cobertura para este idioma
help.why.zh = A referência dentro da China, onde o Google é limitado
help.why.ja = Prioriza páginas japonesas, não traduzidas
help.why.ru = Entende melhor a morfologia russa que os demais
help.query.note = Enviamos só o termo. Sem nome de arquivo, sem caminho, sem id de dispositivo.
path.copy_hint = copiar
nav.preview = Prévia da interface

## ── التنبيهات والشارات ────────────────────────────
alert.confirmed.cta = Abrir a quarentena
badge.confirmed = Confirmado
badge.high = Risco alto
badge.low = Suspeito
badge.pua = Risco médio
button.off = Desativado
button.on = Ativado

## ── الإيقاف المؤقت ────────────────────────────
lvl.app = O aplicativo
lvl.d10 = { $n } minutos
lvl.d30 = { $n } minutos
lvl.d60 = Uma hora
lvl.dpeak = Até o fim do horário de pico
lvl.dreboot = Até reiniciar
lvl.hint = Avisamos, e você decide.
lvl.no_permanent = Não há pausa permanente. A mais longa termina ao reiniciar.
lvl.state_off = Proteção pausada
lvl.state_off_sub = Até 14:30 · sua decisão
lvl.title = Pausa — dois níveis, não um
lvl.what_continues = O que continua
lvl.what_stops = O que para
lvl1.app = Roda · verifica · bloqueia · não pergunta
lvl1.body = Sem janela nem som. A verificação e o bloqueio continuam.
lvl1.cont = Os três muros
lvl1.name = Silenciar avisos
lvl1.stops = Só os avisos
lvl2.body = Sem verificação nem bloqueio. Seu dispositivo fica desprotegido.
lvl2.name = Parar a proteção
lvl2.resp = A responsabilidade é sua durante esse tempo.
lvl2.warn = Paramos de proteger, não de avisar.

## ── دستور الواجهة — القواعد الخمس ────────────────────────────
notes.1b = Um nome de muro é uma promessa, não um rótulo.
notes.1t = Três muros, três promessas
notes.2b = O caminho seguro é o visível, nunca o escondido.
notes.2t = Uma decisão recomendada
notes.3b = Um ícone sem palavra é um enigma; uma frase abaixo o resolve.
notes.3t = Os ícones se explicam
notes.4b = Nenhuma janela que toma a tela. O aviso vive na central.
notes.4t = Nunca um modal
notes.5b = Uma frase de estado. Dois números. Um botão.
notes.5t = O Início se lê em dez segundos

## ── صندوق العزل ────────────────────────────
q.reason1 = Ameaça confirmada
q.reason2 = Ferramenta de crack
quarantine.opened_from = Abre de um aviso e do contador do Início — nunca de um menu escondido.
quarantine.restore = Restaurar

## ── دستور الواجهة — القواعد الخمس ────────────────────────────
sec.alerts = 2 · Avisos
sec.alerts.hint = Menos texto · nunca um botão excluir · explicação sob demanda
sec.color = 4 · Paleta de cores
sec.color.hint = Contraste medido na página, não afirmado
sec.fonts = 5 · Fontes
sec.fonts.hint = Uma fonte por família de idiomas · sem download, sem custo
sec.main = 1 · A tela inicial
sec.main.hint = ≤ 8 elementos · 3 muros · dois botões · sem termos técnicos
sec.notes = Regras que transformam ideias em portas
sec.usb = 6 · USB
sec.usb.hint = Uma barra de progresso + uma fase · nada mais

## ── الهوية ────────────────────────────
shield.sub = { $n } muros ativos · base de ameaças atualizada

## ── الشرح عند الطلب ────────────────────────────
tips.body = Passe o mouse na palavra <u>sublinhada</u> para uma linha · clique nela para uma busca explicativa no seu idioma (Google · Baidu · Yahoo Japan · Yandex).
tips.legend = Explicação sob demanda:

## ── دستور الواجهة — القواعد الخمس ────────────────────────────
trust.1d = Dizemos o que aconteceu, não a técnica. Uma frase.
trust.1t = Clareza
trust.2d = No máximo dois botões. Uma opção principal. Zero configuração obrigatória.
trust.2t = Simplicidade
trust.3d = Sem evento = sem janela. ≤ 3 avisos por dia. Nunca um modal.
trust.3t = Tranquilidade
trust.4d = A consequência de cada botão vem antes, não depois. E o caminho seguro é o visível.
trust.4t = Conhecer as consequências
trust.sub = Cada tela é medida por estes quatro. Em conflito: segurança primeiro.

## ── USB ────────────────────────────
usb.stage = Fase

## ── Algarismos ─────────────────────────────
num.title = Algarismos
num.sub = Os números dos contadores, avisos e registro.
num.body = Escolha a forma que você lê mais rápido. Toda conta é a mesma.
num.arabic = 0 1 2 3 4 5 6 7 8 9
num.hindi = ٠ ١ ٢ ٣ ٤ ٥ ٦ ٧ ٨ ٩
num.arabic.name = Algarismos árabes
num.hindi.name = Algarismos árabes orientais
num.sample = 3 avisos · 12 arquivos
num.note = A mudança é imediata. Sem reiniciar.

## ── Dicas da pausa — sob demanda ─────────────────────────────
tip.lvl1 = Sem janela, sem aviso, sem som, mas a verificação e o bloqueio continuam em segundo plano: um arquivo infectado é isolado em silêncio e mostrado na volta.
tip.lvl2 = Sem verificação, sem bloqueio, sem aviso. O serviço roda mas não faz nada, e seu dispositivo fica sem proteção.
tip.no_permanent = A pausa mais longa termina ao reiniciar, e esse é o único limite que impomos.

## ── Regra quatro: visual primeiro ─────────────────────────────
visual.title = Visual primeiro
visual.rule = A imagem antes da frase: o que a tela faz é visto, não explicado.
visual.why = Quem lê devagar, ou não lê o seu idioma, entende a forma e a cor antes da linha.
visual.n1 = Cada elemento carrega forma, cor e lugar, e nunca depende de uma palavra.
visual.n2 = A mesma forma significa a mesma coisa em toda tela.
visual.n3 = A cor separa o seguro do perigoso; o texto nomeia o motivo.
visual.n4 = Um ícone sem palavra é um teste: se precisa de explicação, o ícone está errado.
visual.n5 = O texto explica a imagem, não a repete.
visual.sample = Passe o mouse no ícone: uma palavra. Clique: a explicação inteira no seu idioma.
trust.count = Cinco, não quatro.
help.sample.title = Um exemplo real
