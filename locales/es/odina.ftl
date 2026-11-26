# Odina Antivirus — Español (es)
# المستوى: T-B (مجتمعية) — تحتاج مراجعة ناطق أصلي قبل ترقيتها إلى T-A (§30.2)
# راجع كل سطر يدويًا قبل الاعتماد. لا ترجمة آلية للنصوص الأمنية (§3.9.1).

app.name = Odina
app.tagline = Tu guardián digital
app.window.title = Odina — { $state }
state.protected = Estás protegido
state.scanning = Analizando…
state.attention = Necesita tu atención
state.paused = Protección en pausa
state.untrusted = Aún no verificado
wall.signatures.title = Base de datos de archivos
wall.signatures.on = Activa
wall.signatures.off = Inactiva
wall.behavior.title = Análisis profundo de comportamiento
wall.behavior.on = Escuchando
wall.behavior.off = Inactivo
wall.proactive.title = Protección proactiva
wall.proactive.on = Activa
wall.proactive.off = Desactivada
action.scan_now = Analizar ahora
action.pause = Pausar
action.expert_mode = Modo experto
action.language = Idioma
action.settings = Ajustes
action.why = ¿Por qué?
scan.quick = Análisis rápido
scan.full = Análisis completo
scan.usb = Analizar esta unidad
scan.running = Analizando { $target }…
scan.percent = { $percent } %
scan.files_checked = Archivos revisados
scan.threats_found = Amenazas encontradas
scan.stop = Detener
scan.complete = Análisis completado
scan.clean_result = No se encontró nada dañino.
alert.confirmed.title = Detuvimos una amenaza conocida
alert.confirmed.body = Infección confirmada en { $file }. Se bloqueó su ejecución; no se borró nada.
alert.suspicious.title = Un programa intenta acceder a tus archivos
alert.suspicious.body = { $file } está modificando archivos en Documentos. Lo detuvimos temporalmente.
alert.suspicious.note = Lo detuvimos temporalmente.
alert.unsure.title = Este programa es poco común
alert.unsure.body = { $file } es nuevo aquí y se comporta de forma extraña. ¿Lo conoces?
alert.pua.title = Parece una herramienta de crackeo
alert.pua.body = { $file } evade la protección de licencias de software de pago. Tú decides qué se ejecuta en tu equipo.
alert.pua.allowed = Permitir de todos modos
alert.pua.quarantine = Apartar
button.allow = Permitir
button.block = Bloquear
button.restore = Restaurar
button.remove_forever = Eliminar permanentemente
usb.detected = Nueva unidad detectada — la revisamos primero
usb.scanning = Revisando { $name }…
usb.priority_done = Archivos peligrosos revisados
usb.trust.new = Unidad nueva
usb.trust.scanning = Revisando
usb.trust.trusted = Recordada
usb.trust.suspicious = Bajo vigilancia
usb.trust.quarantined = Bloqueada
quarantine.title = Elementos aislados
quarantine.reason = Por qué se aisló
quarantine.restore_note = Se conserva la ubicación original.
quarantine.retention = Se conserva { $days } días
pause.title = Pausar la protección
pause.duration.10 = 10 minutos
pause.duration.30 = 30 minutos
pause.duration.60 = 1 hora
pause.warning = No estarás protegido hasta { $until }.
pause.confirm = Pausar igualmente
pause.badge = Avisos en pausa · la protección sigue
error.generic = Algo falló. No cambiamos nada.
error.tamper = Los archivos de Odina fueron modificados. Hay que repararlo.
error.offline = Aún no hay actualización. La protección sigue activa.
update.available = Actualización disponible
update.ask = ¿Buscar y descargar actualizaciones?
update.db_age_warning = Tu lista de amenazas tiene { $days } días.

## ── Path ──────────────────────────────────────────────
path.label = Ruta
path.copy = Copiar ruta
path.open_folder = Abrir carpeta
trust.title = Bases de la confianza
word.isolated = aislado
word.temporary = temporalmente
tip.quarantine = Se movió a un almacén cifrado: su ubicación original se conserva.
tip.suspended = Lo detuvimos ahora. Solo vuelve a funcionar si lo permites.
quarantine.hint = Todo permanece hasta que tú decidas · nunca se borra solo
quarantine.keep = Conservarlo
quarantine.delete = Eliminar permanentemente
quarantine.delete_note = No se puede deshacer. No hay otra copia.
quarantine.cleanup_title = En cuarentena desde hace { $days } días
quarantine.cleanup_choose = Eliminar los que elegí
error.false_positive = Nos equivocamos con este archivo — corregido.
error.locked = No se pudo aislar: el archivo está abierto ahora.

## ── Turn protection off (user's explicit decision) ────────
pause.confirm_disable = Desactivar la protección durante { $minutes} minutos
disable.title = Desactivar la protección
disable.warning_title = Riesgo grave
disable.body = No solo dejamos de avisar: dejamos de proteger. No se analizará ningún archivo ni se bloqueará ninguna amenaza. Tu equipo queda sin protección total.
disable.responsibility = Es tu decisión y la responsabilidad es tuya durante ese periodo.
disable.choose = Elige la duración
disable.duration.peak = Hasta que acabe la hora punta
disable.duration.30 = 30 minutos
disable.duration.60 = 1 hora
disable.duration.until_reboot = Hasta que reinicie
disable.refused = Ahora no se puede desactivar la protección
disable.refused_quarantine = Hay un archivo en cuarentena sin decidir. Revísalo primero.
disable.refused_scanning = Hay un análisis en curso. Espera o detenlo primero.
disable.refused_active_threat = Hay una amenaza sin resolver. No desactivamos la protección mientras siga abierta.
state.unprotected = Protección desactivada
state.unprotected_until = Protección desactivada hasta { $until}
state.unprotected_badge = Sin protección — por tu decisión
action.turn_off = Desactivar la protección
action.turn_on = Reactivar la protección
disable.re_enable_note = Puedes reactivar la protección en cualquier momento, sin contraseña.
disable.reminder = Quedan { $minutes} minutos para que la protección vuelva sola.
pause.resume_report = Durante el silencio: { $restored } reparados · { $isolated } aislados · { $pending } pendientes

## ── Excepción quirúrgica (D45 · §4.8) ─────────────────────────
exception.title = Excepciones
exception.sub = Si tu trabajo necesita que una regla ceda, permite solo esa. El resto queda igual.
exception.rule.a1.wsh = Scripts de Windows
exception.rule.a1.wsh.desc = Los .vbs y .js se ejecutan con límites salvo que los firme un editor conocido.
exception.rule.a1.powershell = Comandos de PowerShell
exception.rule.a1.powershell.desc = PowerShell se ejecuta en modo restringido. Permitirlo no abre Windows Script Host.
exception.rule.a1.macros = Macros de Office
exception.rule.a1.macros.desc = Los documentos sin firma no pueden ejecutar macros. Permitirlo no afecta a los scripts.
exception.rule.a3.downloads = Sospecha en Descargas
exception.rule.a3.downloads.desc = Cada archivo nuevo en Descargas se considera sospechoso hasta descartarlo.
exception.rule.a4.periodic = Análisis periódico
exception.rule.a4.periodic.desc = Un análisis silencioso cada dos horas de los elementos de inicio y las claves sensibles.
exception.rule.a6.doubt = Cautela ante la duda
exception.rule.a6.doubt.desc = Ante la duda bloquea primero y luego pregunta. Se puede relajar, nunca permitir del todo.
exception.state.on = Activa
exception.state.relaxed_until = Relajada hasta { $until }
exception.state.allowed_until = Permitida hasta { $until }
exception.state.allowed_permanent = Permitida de forma permanente — tu decisión
exception.consequence.relax = { $rule } funcionará sin avisos hasta { $until } y luego volverá. Durante ese tiempo no se bloquea nada.
exception.consequence.allow = { $rule } funcionará sin restricciones hasta { $until } y luego volverá. Nada más cambia.
exception.consequence.permanent = { $rule } seguirá permitida hasta que la revoques. Esto permanecerá escrito en la pantalla principal.
exception.action.relax_10 = Relajar 10 minutos
exception.action.relax_30 = Relajar 30 minutos
exception.action.relax_60 = Relajar una hora
exception.action.allow = Permitir
exception.action.revoke = Revocar
exception.action.revoke_all = Revocar todas las excepciones
exception.action.advanced = Opciones avanzadas
exception.badge = { $rule } permitida · hasta { $until }
exception.badge_permanent = { $rule } permitida de forma permanente · tu decisión
exception.badge_many = { $count } reglas permitidas por tu decisión · detalles
exception.until_reboot = hasta reiniciar
exception.permanent_note = Una excepción permanente no termina sola. Seguirá visible en la pantalla principal hasta que la revoques.
exception.revoke_note = Puedes revocarla en cualquier momento: un clic, sin contraseña.
exception.expert_note = El modo experto muestra «permanente» y «la regla completa». Sin contraseña y sin preguntar por qué.
exception.expired_report = La excepción terminó. Durante ella, { $count } archivo(s) parecieron sospechosos; revísalos ahora.
exception.expired_clean = La excepción terminó. No apareció nada que requiera revisión.
exception.refused.not_allowed = Esta regla no puede permitirse del todo: solo relajarse.
exception.refused.above_ceiling = Lo máximo que permite esta regla es relajarla.
exception.refused.no_permanent = Esta regla no admite una excepción permanente.
exception.refused.expert_only = Una excepción permanente de toda la regla requiere el modo experto.
exception.refused.veto_stands = El archivo está confirmado. Una excepción relaja la regla; no absuelve un archivo.

exception.live_note = tu decisión · todo lo demás sigue activo · puedes revocar en cualquier momento
exception.lines_title = Cuatro cosas nunca se exceptúan:
exception.lines_body = el veredicto del primer muro (una excepción no autoriza ejecutar un archivo infectado confirmado) · la protección contra manipulación · la segunda regla (una excepción de WSH no abre PowerShell) · «riesgo alto» y superior. Revocar siempre es un clic, sin contraseña.

## ── Mapa de la app y navegación (constitución §6أ.3) ─────────────────
nav.home = Inicio
nav.quarantine = Cuarentena
nav.proactive = Protección proactiva
nav.alerts = Centro de avisos
nav.settings = Ajustes
nav.language = Idioma
nav.privacy = Privacidad
back = Volver
menu.more = Más
menu.advanced = Opciones avanzadas
map.title = Mapa de la app
map.hint = Cómo se llega a cada pantalla · tres niveles como máximo

## ── Política de firma de código (página pública) ─────────────────────────
signing.title = Política de firma de código
signing.sub = Qué firmamos, qué nunca firmamos, dónde está la clave y cómo verificarlo tú mismo.
signing.status_title = Estado hoy
signing.status_body = Esta política se publica antes de empezar a firmar, y todavía no se ha firmado ninguna compilación. La primera compilación firmada se anunciará en su página de versión con su hash.
signing.status_until = Hasta entonces el programa se distribuye como archivo ZIP portátil, nunca como instalador sin firma.
signing.build_title = Compilación
signing.build_body = Todas las compilaciones se hacen en GitHub Actions desde el commit indicado en las notas de la versión. No hay compilación local, y un binario nunca se recompila antes de firmarlo.
signing.key_title = La clave
signing.key_body = No guardamos ninguna clave de firma. La clave vive en el HSM de SignPath Foundation, y cada solicitud de firma requiere la aprobación explícita de un mantenedor.
signing.signed_title = Firmado
signing.signed_installer = El instalador
signing.signed_store = El paquete de la tienda
signing.unsigned_title = Sin firma
signing.unsigned_db = Las bases de firmas: se firman con una clave Ed25519 aparte, porque se actualizan mucho más que el programa.
signing.unsigned_tools = Todo lo que hay en tools: con licencia MIT y distribuido como código fuente, nunca como binarios.
signing.verify_title = Verificación
signing.verify_body = El SHA-256 de cada artefacto se publica en la página de la versión, antes y después de firmar. Puedes verificarlo tú mismo en Windows:
signing.verify_must = El estado debe ser Valid y el editor debe ser SignPath Foundation. Si el hash difiere del publicado, no ejecutes el archivo.
signing.revoke_title = Revocación
signing.revoke_body = Si alguna vez se produce una compilación con código no revisado, se revoca y la revocación se anuncia antes que cualquier otra cosa.
signing.credit = Firma de código gratuita proporcionada por SignPath.io, certificado de SignPath Foundation.
signing.updated = Última actualización

## ── Glosario — una línea por término ─────────────
tip.confirmed = Su hash está en la base de amenazas. No es una suposición.
tip.high_risk = Señales fuertes de peligro, y el hash aún no se confirma.
tip.suspicious = Se ve extraño: el nombre, el lugar o el comportamiento. Sin verificar.
tip.pua = No es un virus, pero evita la protección de otro programa.
tip.signature = Hashes de archivos conocidos. Actualizarlos mantiene la protección al día.
tip.behavior = Observa qué hace el programa, no cómo se llama.
tip.proactive = Detiene el acto antes de que ocurra en vez de hallarlo después.
tip.pause = Solo silenciamos las ventanas. El análisis y el bloqueo continúan.
tip.turn_off = Sin análisis y sin bloqueo. Tu equipo queda sin protección mientras tanto.
tip.exception = Relajar una regla por tu decisión. Nada más cambia.
tip.expert_mode = Muestra las opciones permanente y de regla completa. Sin contraseña ni preguntas.
tip.wsh = El host de scripts de Windows. Una vía habitual del malware.
tip.powershell = Una herramienta de comandos de Windows. Restringida por defecto desde fuentes de riesgo.
tip.macros = Comandos dentro de un documento de Office. No se ejecutan en archivos sin firma.
tip.amsi = Una interfaz de Windows que nos deja ver un script antes de que se ejecute.
tip.wdac = Una política de Windows que impide ejecutar un archivo concreto.
tip.usb_trust = La primera vez que lo vemos. Se analiza antes de abrirlo.
tip.tamper = No se puede apagar Odina editando sus archivos ni matando su servicio.
tip.hash = Una cadena que representa el archivo entero. Un byte la cambia por completo.
tip.heuristic = Una señal que se mide y se pondera. Nunca basta por sí sola.

## ── Ajustes, privacidad e idioma ─────────────────────────────
settings.title = Ajustes
settings.sub = Cuatro secciones. Nada escondido tras «avanzado» salvo lo poco frecuente.
settings.appearance = Apariencia
settings.appearance.body = Claro u oscuro. Sigue al sistema por defecto.
settings.theme_light = Claro
settings.theme_dark = Oscuro
settings.theme_system = Seguir al sistema
settings.behavior = Comportamiento
settings.startup = Se inicia con Windows
settings.startup.body = Funciona en segundo plano desde el arranque. Puedes desactivarlo.
settings.notifications = Avisos
settings.notifications.body = Tres al día como máximo. Lo importante primero.
settings.updates = Actualizaciones
settings.updates.body = Los datos de amenazas se actualizan automáticamente; el programa, a mano. La base de reparación va aparte.
settings.about = Acerca de
settings.version = Versión
settings.database = Base de amenazas
settings.database.date = Última actualización

privacy.title = Privacidad
privacy.sub = Lo que sale de tu equipo y lo que nunca sale. Sin excepciones.
privacy.telemetry = Cero telemetría
privacy.telemetry.body = Sin estadísticas, sin id de equipo, sin huella. Nada se envía solo.
privacy.local = Todo local
privacy.local.body = El análisis, la decisión y el registro viven en tu disco.
privacy.report = El informe es tu decisión
privacy.report.body = Un informe se envía solo si tú eliges enviarlo.
privacy.search = Búsqueda a demanda
privacy.search.body = Abre un buscador en tu idioma. Enviamos el término solo, nunca tus archivos.
privacy.log = El registro
privacy.log.body = Local y legible en tu idioma. Puedes borrarlo cuando quieras.
privacy.log_clear = Borrar el registro

lang.title = Idioma
lang.sub = Diez idiomas. La traducción aparece al instante y la fuente sigue a tu idioma.
lang.current = Idioma actual
lang.rtl = Se escribe de derecha a izquierda
lang.ltr = Se escribe de izquierda a derecha
lang.review = Revisión nativa
lang.review.body = Un idioma sin revisor nativo se suspende, no se publica.

home.walls = Los tres muros
home.action.scan = Analizar ahora
home.action.pause = Pausar
home.action.expert = Modo experto
home.action.why = ¿Por qué?
home.goto.quarantine = Cuarentena
home.goto.alerts = Centro de avisos
home.goto.settings = Ajustes
home.goto.proactive = Protección proactiva
home.quarantine_count = { $count } archivo(s) en cuarentena
home.alerts_count = { $count } aviso(s) que valen una mirada
home.proactive_summary = Nueve actividades en tres grupos
proactive.title = Protección proactiva
proactive.sub = Permitir una actividad aquí no anula los análisis de archivos ni las reglas del cortafuegos.
proactive.tier.mandatory = Riesgo muy alto · Bloqueo automático
proactive.tier.recommended = Riesgo medio · Bloqueo recomendado
proactive.tier.optional = Riesgo menor · Bloqueo opcional
proactive.surface.office_child = Intérpretes iniciados por Office
proactive.surface.office_child.tip = Inicio de cmd, PowerShell o mshta desde Word o Excel, no todos los procesos secundarios.
proactive.surface.wmi_subscription = Suscripciones WMI persistentes
proactive.surface.wmi_subscription.tip = Nuevas suscripciones permanentes de origen no fiable, no las consultas WMI normales.
proactive.surface.system_tools = Herramientas con entradas no fiables
proactive.surface.system_tools.tip = mshta, certutil o regsvr32 con entradas no fiables de carpetas temporales o de Internet.
proactive.surface.scheduled_task = Tareas programadas
proactive.surface.scheduled_task.tip = Crear o modificar una tarea. El bloqueo no detiene las tareas existentes.
proactive.surface.com_registration = Registro de componentes COM
proactive.surface.com_registration.tip = Cambios en el registro COM de un usuario, sin desactivar COM en Windows.
proactive.surface.registry_autorun = Entradas de inicio del registro
proactive.surface.registry_autorun.tip = Añadir o cambiar entradas bajo Run o RunOnce en el registro.
proactive.surface.service_driver = Instalación de servicios y controladores
proactive.surface.service_driver.tip = Crear un servicio o instalar un controlador, sin detener los que ya se ejecutan.
proactive.surface.inbound_smb_rdp = SMB y RDP entrantes
proactive.surface.inbound_smb_rdp.tip = Nuevas conexiones entrantes de uso compartido o escritorio remoto. No se abren puertos.
proactive.surface.wsh_script = Scripts de Windows
proactive.surface.wsh_script.tip = Ejecutar .js, .vbs o .wsf mediante WSH. Siguen vigentes los demás controles de scripts.
proactive.state.blocked = Bloqueado
proactive.state.allowed = Permitido
proactive.state.contained = Bloqueo temporal
proactive.state.off = Sin aplicar
proactive.saved_until = { $rule } · excepción guardada hasta { $until }
proactive.user_wsh_block = Tu bloqueo de scripts de Windows prevalece sobre esta excepción.
proactive.demo.title = Simulación de contención
proactive.demo.note = Simulación del tratamiento automático. No se analizan archivos ni se cambian ajustes de Windows.
proactive.demo.mode = Modo de protección simulado
proactive.demo.ready = Si el análisis confirma una infección, las nueve actividades se bloquean hasta que se complete la reparación.
proactive.demo.start = Simular infección confirmada
proactive.demo.scan = Análisis con las tres barreras. Las nueve actividades siguen bloqueadas temporalmente.
proactive.demo.clean = Peligro neutralizado. Buscando una regla fiable para desinfectar el archivo.
proactive.demo.repair = Reparando daños y comprobando el archivo. El bloqueo sigue hasta la verificación final.
proactive.demo.cleaned = Limpieza completada
proactive.demo.repaired = Reparación completada
proactive.demo.restored = Reparación verificada. Vuelven a aplicarse tus preferencias.
proactive.demo.off = Has desactivado la protección. No se inicia la contención ni se cambian tus preferencias.
proactive.demo.fail = Falló el paso
proactive.demo.failed = El paso no se completó. La contención se mantiene hasta que finalice la recuperación.
alerts.title = Centro de avisos
alerts.sub = Todo lo que vale una mirada en un solo lugar. Sin ventanas emergentes.
alerts.empty = Nada que valga una mirada ahora mismo.
exception.detail.title = Excepción — una regla
exception.detail.sub = Qué cambia, por cuánto tiempo y cómo vuelve.
map.node.home = Inicio
map.node.quarantine = Cuarentena
map.node.proactive = Proactiva
map.edge.from_home = desde Inicio
map.edge.from_alert = desde un aviso
map.legend = Línea continua es entrada permanente · punteada necesita contexto

## ── Explicación a demanda ─────────────────────────────
help.title = Explicación a demanda
help.hint = Una línea al pasar · una búsqueda completa al pulsar, en tu idioma.
help.tip.title = Aviso inmediato
help.tip.body = Una palabra subrayada y una línea de 90 caracteres como máximo.
help.guide.title = Una guía más larga
help.guide.body = Se abre con un botón dentro del aviso y se lee en un buscador que conoces.
help.guide.cta = Abrir la búsqueda
help.engine.title = El buscador sigue idioma y lugar
help.col.lang = Idioma
help.col.engine = Buscador
help.col.why = Motivo
help.why.default = La mayor cobertura para este idioma
help.why.zh = La referencia dentro de China, donde Google es limitado
help.why.ja = Prioriza páginas japonesas, no traducidas
help.why.ru = Entiende mejor la morfología rusa que los demás
help.query.note = Enviamos el término solo. Ni nombre de archivo, ni ruta, ni id de equipo.
path.copy_hint = copiar
nav.preview = Vista previa

## ── التنبيهات والشارات ────────────────────────────
alert.confirmed.cta = Ver tratamiento
badge.confirmed = Confirmado
badge.high = Muy sospechoso
badge.low = Sospechoso
badge.pua = Riesgo medio
button.off = Desactivado
button.on = Activado

## ── الإيقاف المؤقت ────────────────────────────
lvl.app = La aplicación
lvl.d10 = { $n } minutos
lvl.d30 = { $n } minutos
lvl.d60 = 1 hora
lvl.dpeak = Hasta que acabe la hora punta
lvl.dreboot = Hasta que reinicie
lvl.hint = Te avisamos y tú decides.
lvl.no_permanent = No hay pausa permanente. La más larga acaba al reiniciar.
lvl.state_off = Protección desactivada
lvl.state_off_sub = hasta las 14:30 · por tu decisión
lvl.title = Desactivar — dos niveles, no uno
lvl.what_continues = Qué continúa
lvl.what_stops = Qué se detiene
lvl1.app = funciona · analiza · bloquea · no pregunta
lvl1.body = Sin ventana ni sonido. El análisis y el bloqueo siguen.
lvl1.cont = Los tres muros
lvl1.name = Silenciar avisos
lvl1.stops = Solo los avisos
lvl2.body = Sin análisis ni bloqueo. Tu equipo queda desprotegido.
lvl2.name = Detener la protección
lvl2.resp = La responsabilidad es tuya durante ese tiempo.
lvl2.warn = Dejamos de proteger, no de avisar.

## ── دستور الواجهة — القواعد الخمس ────────────────────────────
notes.1b = cambia dirección y diseño al instante; los números siguen en latino en ratios y hashes.
notes.1t = Cambio de idioma:
notes.2b = Permitir/Bloquear mantienen orden lógico fijo; RTL invierte la posición visual, no el rol.
notes.2t = Botones:
notes.3b = aquí se usan fuentes del sistema; tras descargar el paquete CDN pasan a Noto sin reiniciar.
notes.3t = CJK:
notes.4b = este archivo carga desde fonts/vendor/ — ábrelo con un servidor local, no con file://.
notes.4t = Fuentes:
notes.5b = logotipo, icono de bandeja y estados de error — se revisan aparte (R3).
notes.5t = No incluido aquí:

## ── صندوق العزل ────────────────────────────
q.reason1 = Malware independiente, sin archivo anfitrión que salvar
q.reason2 = Herramienta de crackeo
quarantine.opened_from = Malware, archivos que no se pudieron desinfectar con seguridad y elementos que moviste aquí.
quarantine.restore = Restaurar

## ── دستور الواجهة — القواعد الخمس ────────────────────────────
sec.alerts = 2 · Las cuatro alertas
sec.alerts.hint = 2 botones + «¿Por qué?» · el color nunca informa solo
sec.color = 4 · Paleta (WCAG medida)
sec.color.hint = cada valor muestra su contraste sobre el fondo actual
sec.fonts = 5 · Fuentes por idioma
sec.fonts.hint = solo 400 y 600 · fuentes del sistema como respaldo inmediato
sec.main = 1 · Pantalla principal
sec.main.hint = ≤ 8 elementos · 3 muros · 2 botones
sec.notes = 6 · Notas de revisión
sec.usb = 3 · Alerta USB
sec.usb.hint = una barra · una etapa

## ── الهوية ────────────────────────────
shield.sub = { $n } muros activos · base de amenazas al día

## ── الشرح عند الطلب ────────────────────────────
tips.body = Pasa el cursor por la palabra subrayada para una línea · haz clic para abrir una búsqueda explicativa en tu idioma (Google · Baidu · Yandex).
tips.legend = Explicación a pedido:

## ── دستور الواجهة — القواعد الخمس ────────────────────────────
trust.1d = Decimos qué pasó, no cómo funciona. Una frase. Sin «hecho» sin motivo.
trust.1t = Claridad
trust.2d = Dos botones como máximo. Una acción principal. Cero configuración.
trust.2t = Sencillez
trust.3d = Sin evento, sin ventana. ≤ 3 avisos al día. Nunca un modal.
trust.3t = Calma
trust.4d = Cada botón dice su resultado antes de pulsarlo. La opción segura es la visible.
trust.4t = Consecuencias claras
trust.sub = Cada pantalla se mide con estas cuatro. En conflicto: seguridad → no molestar → claridad → sencillez → estética.

## ── USB ────────────────────────────
usb.stage = P0 · archivos peligrosos en la raíz

## ── Cifras ─────────────────────────────
num.title = Cifras
num.sub = Los números de contadores, avisos y registro.
num.body = Elige la forma que leas más rápido. Todo cálculo es el mismo.
num.arabic = 0 1 2 3 4 5 6 7 8 9
num.hindi = ٠ ١ ٢ ٣ ٤ ٥ ٦ ٧ ٨ ٩
num.arabic.name = Cifras árabes
num.hindi.name = Cifras árabes orientales
num.sample = 3 avisos · 12 archivos
num.note = El cambio es inmediato. No hace falta reiniciar.

## ── Avisos de la pausa — a demanda ─────────────────────────────
tip.lvl1 = El análisis, el bloqueo y la reparación continúan en silencio. Verás los resultados al volver.
tip.lvl2 = Sin análisis, sin bloqueo, sin aviso. El servicio funciona pero no hace nada, y tu equipo queda sin protección.
tip.no_permanent = La pausa más larga termina al reiniciar, y ese es el único límite que imponemos.

## ── Regla cuatro: visual primero ─────────────────────────────
visual.title = Visual primero
visual.rule = La imagen antes de la frase: lo que hace una pantalla se ve, no se explica.
visual.why = Quien lee despacio, o no lee tu idioma, entiende la forma y el color antes que la línea.
visual.n1 = Cada elemento lleva forma, color y lugar, así que nunca depende de una palabra.
visual.n2 = La misma forma significa lo mismo en toda pantalla.
visual.n3 = El color separa lo seguro de lo peligroso; el texto nombra el motivo.
visual.n4 = Un icono sin palabra es una prueba: si hay que explicarlo, el icono está mal.
visual.n5 = El texto explica la imagen, no la repite.
visual.sample = Pasa sobre el icono: una palabra. Púlsalo: explicación completa en tu idioma.
trust.count = Cinco, no cuatro.
help.sample.title = Ejemplo real

## ── Recovery (D47) ──
proactive.demo.neutralize = Deteniendo la amenaza activa antes de tocar el archivo infectado.
proactive.demo.verify = Verificando todo el incidente antes de levantar el bloqueo.
recovery.title = Tratamiento
recovery.tip = Neutralizar, limpiar y reparar. Aislar solo malware o archivos sin recuperación segura.
recovery.case = Caso de ejemplo
recovery.match = Archivo recuperable
recovery.no_rule = No hay una regla segura que coincida
recovery.no_db = Faltan datos de reparación fiables
recovery.bad_verify = La copia limpia no superó la verificación
recovery.scan = Análisis con tres barreras
recovery.neutralize = Neutralizar el peligro
recovery.clean = Desinfectar el archivo
recovery.repair = Reparar los daños
recovery.verify = Verificación final
recovery.next = Siguiente paso
recovery.file_ok = Archivo desinfectado, reparado y verificado.
recovery.file_isolated = Archivo aislado, no borrado.
settings.repair_db = Base de reparación
settings.repair_db.status = v1/r1 · dos reglas de prueba
settings.repair_db.help = Incluye reglas por familia. Descarga separada si el paquete supera los { $mb } MB.
recovery.case_malware = Malware independiente
recovery.case_unknown = Sin regla compatible
recovery.case_missing = Base ausente
recovery.case_invalid = Verificación fallida
settings.repair_db.install = Instalación aparte
settings.repair_db.instructions = Extrae { $package } de la misma versión de Odina junto al programa, sin cambiar las carpetas.
settings.repair_db.releases = Paquetes publicados

## Browser context menu (D48)
browser.menu = Analizar con Odina
browser.description = Comprueba la selección local solo cuando se lo pides.
browser.suspect = Este archivo es sospechoso.
browser.consequence = Odina leerá un archivo sospechoso de la selección para verificar su huella localmente. No cambiará nada.
browser.deep = Análisis profundo
browser.ignore = Ignorar
browser.confirmed = Odina ha encontrado una amenaza confirmada en la selección.
browser.limits = Esta comprobación no bloqueó la ejecución ni movió el archivo. La cuarentena aún no está disponible.
browser.stillSuspect = El análisis no confirmó ninguna infección. El archivo no ha cambiado.
browser.close = Cerrar
browser.working = Verificando localmente…
browser.quarantine = Mover a cuarentena

## ── عمر الإصدار وإعلام التحديث (D113 · D122) ─────────────────────
state.stale = Tu lista de amenazas está desactualizada
update.chip = Nueva versión { $v } disponible
home.version = Versión { $v }
