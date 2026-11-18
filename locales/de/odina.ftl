# Odina Antivirus — Deutsch (de)
# المستوى: T-B (مجتمعية) — تحتاج مراجعة ناطق أصلي قبل ترقيتها إلى T-A (§30.2)
# راجع كل سطر يدويًا قبل الاعتماد. لا ترجمة آلية للنصوص الأمنية (§3.9.1).

app.name = Odina
app.tagline = Dein digitaler Wächter
app.window.title = Odina — { $state }
state.protected = Du bist geschützt
state.scanning = Wird geprüft…
state.attention = Braucht deine Aufmerksamkeit
state.paused = Schutz pausiert
state.untrusted = Noch nicht geprüft
wall.signatures.title = Dateidatenbank
wall.signatures.on = Aktiv
wall.signatures.off = Inaktiv
wall.behavior.title = Tiefe Verhaltensprüfung
wall.behavior.on = Hört zu
wall.behavior.off = Inaktiv
wall.proactive.title = Vorbeugender Schutz
wall.proactive.on = Aktiv
wall.proactive.off = Deaktiviert
action.scan_now = Jetzt prüfen
action.pause = Pausieren
action.expert_mode = Expertenmodus
action.language = Sprache
action.settings = Einstellungen
action.why = Warum?
scan.quick = Schnellprüfung
scan.full = Vollständige Prüfung
scan.usb = Dieses Laufwerk prüfen
scan.running = { $target } wird geprüft…
scan.percent = { $percent } %
scan.files_checked = Geprüfte Dateien
scan.threats_found = Gefundene Bedrohungen
scan.stop = Stoppen
scan.complete = Prüfung abgeschlossen
scan.clean_result = Es wurde nichts Schädliches gefunden.
alert.confirmed.title = Wir haben eine bekannte Bedrohung gestoppt
alert.confirmed.body = { $file } ist eine bestätigte Bedrohung und wurde isoliert.
alert.suspicious.title = Ein Programm versucht, auf deine Dateien zuzugreifen
alert.suspicious.body = { $file } ändert Dateien in Dokumente. Wir haben es vorübergehend gestoppt.
alert.suspicious.note = Wir haben es vorübergehend gestoppt.
alert.unsure.title = Dieses Programm ist ungewöhnlich
alert.unsure.body = { $file } ist neu hier und verhält sich seltsam. Kennst du es?
alert.pua.title = Das sieht nach einem Crack-Werkzeug aus
alert.pua.body = { $file } umgeht den Lizenzschutz kostenpflichtiger Software. Du entscheidest, was auf deinem Gerät läuft.
alert.pua.allowed = Trotzdem erlauben
alert.pua.quarantine = Beiseitelegen
button.allow = Erlauben
button.block = Blockieren
button.restore = Wiederherstellen
button.remove_forever = Endgültig löschen
usb.detected = Neues Laufwerk erkannt — wir prüfen es zuerst
usb.scanning = { $name } wird geprüft…
usb.priority_done = Gefährliche Dateien geprüft
usb.trust.new = Neues Laufwerk
usb.trust.scanning = Wird geprüft
usb.trust.trusted = Bekannt
usb.trust.suspicious = Unter Beobachtung
usb.trust.quarantined = Blockiert
quarantine.title = Isolierte Elemente
quarantine.reason = Grund der Isolierung
quarantine.restore_note = Der ursprüngliche Speicherort bleibt erhalten.
quarantine.retention = { $days } Tage aufbewahrt
pause.title = Schutz pausieren
pause.duration.10 = 10 Minuten
pause.duration.30 = 30 Minuten
pause.duration.60 = 1 Stunde
pause.warning = Du bist bis { $until } nicht geschützt.
pause.confirm = Trotzdem pausieren
pause.badge = Hinweise pausiert · Schutz läuft
error.generic = Etwas ist schiefgelaufen. Wir haben nichts geändert.
error.tamper = Odinas eigene Dateien wurden verändert. Bitte reparieren.
error.offline = Noch kein Update. Der Schutz läuft weiter.
update.available = Update verfügbar
update.ask = Nach Updates suchen und herunterladen?
update.db_age_warning = Deine Bedrohungsliste ist { $days } Tage alt.

## ── Path ──────────────────────────────────────────────
path.label = Pfad
path.copy = Pfad kopieren
path.open_folder = Ordner öffnen
trust.title = Vertrauensgrundlagen
word.isolated = isoliert
word.temporary = vorübergehend
tip.quarantine = In einen verschlüsselten Speicher verschoben — der ursprüngliche Ort bleibt erhalten.
tip.suspended = Wir haben es soeben gestoppt. Es läuft nur weiter, wenn du es erlaubst.
quarantine.hint = Alles bleibt, bis du entscheidest · nie automatisch gelöscht
quarantine.keep = Behalten
quarantine.delete = Endgültig löschen
quarantine.delete_note = Nicht rückgängig zu machen. Keine weitere Kopie.
quarantine.cleanup_title = Seit { $days } Tagen in Quarantäne
quarantine.cleanup_choose = Nur die ausgewählten löschen
error.false_positive = Wir lagen bei dieser Datei falsch — behoben.
error.locked = Konnte nicht isoliert werden: Die Datei ist gerade geöffnet.

## ── Turn protection off (user's explicit decision) ────────
pause.confirm_disable = Schutz für { $minutes} Minuten ausschalten
disable.title = Schutz ausschalten
disable.warning_title = Ernstes Risiko
disable.body = Wir hören nicht nur auf zu warnen — wir hören auf zu schützen. Es wird keine Datei geprüft und keine Bedrohung blockiert.
disable.responsibility = Das ist deine Entscheidung — und die Verantwortung liegt für diesen Zeitraum bei dir.
disable.choose = Dauer wählen
disable.duration.peak = Bis zum Ende der Stoßzeit
disable.duration.30 = 30 Minuten
disable.duration.60 = 1 Stunde
disable.duration.until_reboot = Bis zum Neustart
disable.refused = Der Schutz kann jetzt nicht ausgeschaltet werden
disable.refused_quarantine = Es gibt eine isolierte Datei ohne Entscheidung. Bitte prüfe sie zuerst.
disable.refused_scanning = Ein Scan läuft. Warte oder stoppe ihn zuerst.
disable.refused_active_threat = Eine ungeklärte Bedrohung ist offen. Solange sie offen ist, schalten wir den Schutz nicht aus.
state.unprotected = Schutz ist aus
state.unprotected_until = Schutz aus bis { $until}
state.unprotected_badge = Ungeschützt — deine Entscheidung
action.turn_off = Schutz ausschalten
action.turn_on = Schutz wieder einschalten
disable.re_enable_note = Du kannst den Schutz jederzeit wieder einschalten — ohne Passwort.
disable.reminder = Noch { $minutes} Minuten, dann kommt der Schutz automatisch zurück.
pause.resume_report = Während die Hinweise pausiert waren: { $count} Datei(en) isoliert

## ── Gezielte Ausnahme (D45 · §4.8) ─────────────────────────
exception.title = Ausnahmen — eine Regel, nicht alle
exception.sub = Wenn Ihre Arbeit eine Ausnahme bei einer Regel braucht, erlauben Sie nur diese. Der Rest bleibt, wie er ist.
exception.rule.a1.wsh = Windows-Skripte
exception.rule.a1.wsh.desc = .vbs- und .js-Dateien laufen eingeschränkt, sofern sie nicht von einem bekannten Herausgeber signiert sind.
exception.rule.a1.powershell = PowerShell-Befehle
exception.rule.a1.powershell.desc = PowerShell läuft im eingeschränkten Modus. Das Erlauben öffnet Windows Script Host nicht.
exception.rule.a1.macros = Office-Makros
exception.rule.a1.macros.desc = Unsignierte Dokumente können keine Makros ausführen. Das Erlauben betrifft keine Skripte.
exception.rule.a3.downloads = Verdacht im Download-Ordner
exception.rule.a3.downloads.desc = Jede neue Datei im Download-Ordner gilt als verdächtig, bis sie entlastet ist.
exception.rule.a4.periodic = Regelmäßiger Scan
exception.rule.a4.periodic.desc = Alle zwei Stunden ein stiller Scan von Autostart-Einträgen und sensiblen Schlüsseln.
exception.rule.a6.doubt = Vorsicht bei Zweifel
exception.rule.a6.doubt.desc = Bei Zweifel wird zuerst blockiert, dann gefragt. Lockerbar, aber nie ganz erlaubt.
exception.state.on = Aktiv
exception.state.relaxed_until = Gelockert bis { $until }
exception.state.allowed_until = Erlaubt bis { $until }
exception.state.allowed_permanent = Dauerhaft erlaubt — Ihre Entscheidung
exception.consequence.relax = { $rule } läuft bis { $until } ohne Hinweise und kommt dann zurück. In dieser Zeit wird nichts blockiert.
exception.consequence.allow = { $rule } läuft bis { $until } uneingeschränkt und kommt dann zurück. Alles andere bleibt unverändert.
exception.consequence.permanent = { $rule } bleibt erlaubt, bis Sie es widerrufen. Dies steht weiterhin auf dem Startbildschirm.
exception.action.relax_10 = 10 Minuten lockern
exception.action.relax_30 = 30 Minuten lockern
exception.action.relax_60 = Eine Stunde lockern
exception.action.allow = Erlauben
exception.action.revoke = Widerrufen
exception.action.revoke_all = Alle Ausnahmen widerrufen
exception.action.advanced = Erweiterte Optionen
exception.badge = { $rule } erlaubt · bis { $until }
exception.badge_permanent = { $rule } dauerhaft erlaubt · Ihre Entscheidung
exception.badge_many = { $count } Regeln durch Ihre Entscheidung erlaubt · Details
exception.until_reboot = bis zum Neustart
exception.permanent_note = Eine dauerhafte Ausnahme endet nicht von selbst. Sie bleibt sichtbar, bis Sie sie widerrufen.
exception.revoke_note = Sie können jederzeit widerrufen — ein Klick, kein Passwort.
exception.expert_note = Der Expertenmodus zeigt „dauerhaft“ und „die ganze Regel“. Kein Passwort und keine Frage nach dem Grund.
exception.expired_report = Die Ausnahme ist beendet. Währenddessen wirkten { $count } Datei(en) verdächtig — sehen Sie sie jetzt an.
exception.expired_clean = Die Ausnahme ist beendet. Es ist nichts aufgetaucht, das geprüft werden müsste.
exception.refused.not_allowed = Diese Regel kann nicht ganz erlaubt werden — nur gelockert.
exception.refused.above_ceiling = Weiter als gelockert geht diese Regel nicht.
exception.refused.no_permanent = Für diese Regel ist keine dauerhafte Ausnahme möglich.
exception.refused.expert_only = Eine dauerhafte Ausnahme für die ganze Regel erfordert den Expertenmodus.
exception.refused.veto_stands = Die Datei ist bestätigt. Eine Ausnahme lockert die Regel; sie spricht keine Datei frei.

exception.live_note = Ihre Entscheidung · alles andere bleibt aktiv · Sie können jederzeit widerrufen
exception.lines_title = Vier Dinge werden nie ausgenommen:
exception.lines_body = das Urteil der ersten Wand (Bestätigtes wird auch mit Ausnahme isoliert) · Manipulationsschutz · die zweite Regel (eine WSH-Ausnahme öffnet PowerShell nicht) · „hohes Risiko“ und darüber. Widerrufen ist immer ein Klick, ohne Passwort.

## ── App-Karte und Navigation (Verfassung §6أ.3) ─────────────────
nav.home = Start
nav.quarantine = Quarantäne
nav.proactive = Vorbeugender Schutz
nav.alerts = Benachrichtigungszentrale
nav.settings = Einstellungen
nav.language = Sprache
nav.privacy = Datenschutz
back = Zurück
menu.more = Mehr
menu.advanced = Erweiterte Optionen
map.title = App-Karte
map.hint = Wie jeder Bildschirm erreicht wird · höchstens drei Ebenen

## ── Code-Signaturrichtlinie (öffentliche Seite) ─────────────────────────
signing.title = Code-Signaturrichtlinie
signing.sub = Was wir signieren, was nie, wo der Schlüssel liegt und wie Sie es selbst prüfen.
signing.status_title = Stand heute
signing.status_body = Diese Richtlinie ist vor Beginn der Signierung veröffentlicht, und noch kein Build wurde signiert. Der erste signierte Build wird auf seiner Release-Seite samt Hash angekündigt.
signing.status_until = Bis dahin wird das Programm als portables ZIP-Archiv verteilt, niemals als unsignierter Installer.
signing.build_title = Build
signing.build_body = Builds laufen auf GitHub Actions aus dem in den Versionshinweisen genannten Commit. Es gibt keinen lokalen Build, und eine Binärdatei wird vor dem Signieren nie neu gebaut.
signing.key_title = Der Schlüssel
signing.key_body = Wir halten keinen Signaturschlüssel. Der Schlüssel liegt im HSM der SignPath Foundation, und jede Signaturanfrage erfordert die ausdrückliche Freigabe eines Maintainers.
signing.signed_title = Signiert
signing.signed_installer = Der Installer
signing.signed_store = Das Store-Paket
signing.unsigned_title = Nicht signiert
signing.unsigned_db = Signaturdatenbanken — signiert mit einem separaten Ed25519-Schlüssel, weil sie viel häufiger aktualisiert werden als das Programm.
signing.unsigned_tools = Alles unter tools — MIT-lizenziert und als Quelltext verteilt, nie als Binärdatei.
signing.verify_title = Prüfung
signing.verify_body = Für jedes Artefakt wird vor und nach dem Signieren ein SHA-256 auf der Release-Seite veröffentlicht. Sie können es unter Windows selbst prüfen:
signing.verify_must = Der Status muss Valid sein und der Herausgeber SignPath Foundation. Weicht der Hash vom veröffentlichten ab, führen Sie die Datei nicht aus.
signing.revoke_title = Widerruf
signing.revoke_body = Wird je ein Build aus ungeprüftem Code erzeugt, wird er widerrufen, und der Widerruf wird vor allem anderen bekanntgegeben.
signing.credit = Kostenlose Code-Signierung von SignPath.io, Zertifikat von der SignPath Foundation.
signing.updated = Zuletzt aktualisiert

## ── Glossar — eine Zeile pro Begriff ─────────────
tip.confirmed = Sein Hash steht in der Bedrohungsdatenbank. Keine Vermutung.
tip.high_risk = Starke Anzeichen für Gefahr, der Hash ist noch nicht bestätigt.
tip.suspicious = Sieht seltsam aus: der Name, der Ort oder das Verhalten. Ungeprüft.
tip.pua = Kein Virus, aber es umgeht den Schutz eines anderen Programms.
tip.signature = Hashes bekannter Dateien. Aktualisieren hält den Schutz aktuell.
tip.behavior = Beobachtet, was das Programm tut, nicht wie es heißt.
tip.proactive = Stoppt die Tat, bevor sie geschieht, statt sie danach zu finden.
tip.pause = Wir stellen nur die Fenster still. Prüfung und Blockieren laufen weiter.
tip.turn_off = Keine Prüfung und kein Blockieren. Ihr Gerät ist derweil ungeschützt.
tip.exception = Eine Regel nach Ihrer Entscheidung lockern. Sonst ändert sich nichts.
tip.expert_mode = Zeigt dauerhafte und regelweite Optionen. Kein Passwort, keine Fragen.
tip.wsh = Der Windows-Skripthost. Ein häufiger Weg für Schadsoftware.
tip.powershell = Ein Befehlswerkzeug in Windows. Standardmäßig eingeschränkt bei riskanten Quellen.
tip.macros = Befehle in einem Office-Dokument. In unsignierten Dateien laufen sie nicht.
tip.amsi = Eine Windows-Schnittstelle, die uns ein Skript vor der Ausführung zeigt.
tip.wdac = Eine Windows-Richtlinie, die genau eine Datei vom Starten abhält.
tip.usb_trust = Das erste Mal, dass wir es sehen. Geprüft, bevor es geöffnet wird.
tip.tamper = Odina lässt sich nicht abschalten, indem man seine Dateien ändert oder den Dienst beendet.
tip.hash = Eine Zeichenfolge für die ganze Datei. Ein Byte ändert alles.
tip.heuristic = Ein Signal, das gemessen und gewichtet wird. Allein nie genug.

## ── Einstellungen, Datenschutz und Sprache ─────────────────────────────
settings.title = Einstellungen
settings.sub = Vier Bereiche. Nichts hinter „erweitert“ versteckt außer Seltenem.
settings.appearance = Darstellung
settings.appearance.body = Hell oder dunkel. Standardmäßig wie das System.
settings.theme_light = Hell
settings.theme_dark = Dunkel
settings.theme_system = Wie das System
settings.behavior = Verhalten
settings.startup = Startet mit Windows
settings.startup.body = Läuft ab dem Start im Hintergrund. Sie können es abschalten.
settings.notifications = Benachrichtigungen
settings.notifications.body = Höchstens drei am Tag. Das Wichtigste zuerst.
settings.updates = Aktualisierungen
settings.updates.body = Automatisch für die Bedrohungsdatenbank, manuell für das Programm.
settings.about = Über
settings.version = Version
settings.database = Bedrohungsdatenbank
settings.database.date = Zuletzt aktualisiert

privacy.title = Datenschutz
privacy.sub = Was Ihr Gerät verlässt und was nie. Ohne Ausnahmen.
privacy.telemetry = Keine Telemetrie
privacy.telemetry.body = Keine Statistik, keine Geräte-ID, kein Fingerabdruck. Nichts geht von selbst hinaus.
privacy.local = Alles lokal
privacy.local.body = Prüfung, Entscheidung und Protokoll liegen auf Ihrer Festplatte.
privacy.report = Der Bericht ist Ihre Entscheidung
privacy.report.body = Ein Bericht wird nur gesendet, wenn Sie ihn senden wollen.
privacy.search = Suche auf Anfrage
privacy.search.body = Öffnet eine Suchmaschine in Ihrer Sprache. Gesendet wird nur der Begriff, nie Ihre Dateien.
privacy.log = Das Protokoll
privacy.log.body = Lokal und in Ihrer Sprache lesbar. Sie können es jederzeit löschen.
privacy.log_clear = Protokoll löschen

lang.title = Sprache
lang.sub = Zehn Sprachen. Die Übersetzung erscheint sofort, die Schrift folgt Ihrer Sprache.
lang.current = Aktuelle Sprache
lang.rtl = Wird von rechts nach links geschrieben
lang.ltr = Wird von links nach rechts geschrieben
lang.review = Muttersprachliche Prüfung
lang.review.body = Eine Sprache ohne muttersprachliche Prüfung wird zurückgehalten, nicht ausgeliefert.

home.walls = Die drei Wände
home.action.scan = Jetzt prüfen
home.action.pause = Anhalten
home.action.expert = Expertenmodus
home.action.why = Warum?
home.goto.quarantine = Quarantäne
home.goto.alerts = Benachrichtigungszentrale
home.goto.settings = Einstellungen
home.goto.proactive = Vorbeugender Schutz
home.quarantine_count = { $count } Datei(en) in Quarantäne
home.alerts_count = { $count } Hinweis(e) für einen Blick
home.proactive_summary = Sechs Regeln · eine ausgenommen
proactive.title = Vorbeugender Schutz
proactive.sub = Sechs Regeln. Jede mit eigenem Schalter, keine öffnet eine andere.
alerts.title = Benachrichtigungszentrale
alerts.sub = Alles, was einen Blick verdient, an einem Ort. Keine Fenster, die aufpoppen.
alerts.empty = Derzeit nichts, was einen Blick verdient.
exception.detail.title = Ausnahme — eine Regel
exception.detail.sub = Was sich ändert, wie lange, und wie es zurückkommt.
map.node.home = Start
map.node.quarantine = Quarantäne
map.node.proactive = Vorbeugend
map.edge.from_home = von Start
map.edge.from_alert = von einem Hinweis
map.legend = Durchgezogen ist ein fester Zugang · gestrichelt braucht einen Kontext

## ── Erklärung auf Anfrage ─────────────────────────────
help.title = Erklärung auf Anfrage
help.hint = Eine Zeile beim Überfahren · eine volle Suche beim Klick, in Ihrer Sprache.
help.tip.title = Sofortiger Hinweis
help.tip.body = Ein unterstrichenes Wort und eine Zeile mit höchstens 90 Zeichen.
help.guide.title = Ein längerer Leitfaden
help.guide.body = Öffnet über eine Schaltfläche im Hinweis und liest sich in einer Suchmaschine, die Sie kennen.
help.guide.cta = Suche öffnen
help.engine.title = Die Suchmaschine folgt Sprache und Ort
help.col.lang = Sprache
help.col.engine = Suchmaschine
help.col.why = Grund
help.why.default = Die breiteste Abdeckung für diese Sprache
help.why.zh = Die Referenz in China, wo Google begrenzt ist
help.why.ja = Bevorzugt japanische Seiten, nicht übersetzte
help.why.ru = Versteht russische Wortformen besser als andere
help.query.note = Gesendet wird nur der Begriff. Kein Dateiname, kein Pfad, keine Geräte-ID.
path.copy_hint = kopieren
nav.preview = Oberflächenvorschau

## ── التنبيهات والشارات ────────────────────────────
alert.confirmed.cta = Quarantäne öffnen
badge.confirmed = Bestätigt
badge.high = Hohes Risiko
badge.low = Verdächtig
badge.pua = Mittleres Risiko
button.off = Aus
button.on = An

## ── الإيقاف المؤقت ────────────────────────────
lvl.app = Die App
lvl.d10 = { $n } Minuten
lvl.d30 = { $n } Minuten
lvl.d60 = Eine Stunde
lvl.dpeak = Bis zum Ende der Stoßzeit
lvl.dreboot = Bis zum Neustart
lvl.hint = Wir sagen es, Sie entscheiden.
lvl.no_permanent = Keine dauerhafte Pause. Die längste endet beim Neustart.
lvl.state_off = Schutz pausiert
lvl.state_off_sub = Bis 14:30 · Ihre Entscheidung
lvl.title = Pause — zwei Stufen, nicht eine
lvl.what_continues = Was weiterläuft
lvl.what_stops = Was stoppt
lvl1.app = Läuft · prüft · blockt · fragt nicht
lvl1.body = Kein Fenster, kein Ton. Prüfung und Blocken laufen weiter.
lvl1.cont = Die drei Wände
lvl1.name = Hinweise stumm
lvl1.stops = Nur die Hinweise
lvl2.body = Keine Prüfung, kein Blocken. Ihr Gerät ist ungeschützt.
lvl2.name = Schutz anhalten
lvl2.resp = Die Verantwortung liegt für diese Zeit bei Ihnen.
lvl2.warn = Wir hören auf zu schützen, nicht auf zu sagen.

## ── دستور الواجهة — القواعد الخمس ────────────────────────────
notes.1b = Ein Wandname ist ein Versprechen, kein Etikett.
notes.1t = Drei Wände, drei Versprechen
notes.2b = Der sichere Weg ist der sichtbare, nie der versteckte.
notes.2t = Eine empfohlene Entscheidung
notes.3b = Ein Icon ohne Wort ist ein Rätsel; ein Satz darunter löst es.
notes.3t = Icons erklären sich
notes.4b = Kein Fenster, das den Bildschirm übernimmt. Der Hinweis lebt in der Zentrale.
notes.4t = Nie ein Modal
notes.5b = Ein Statussatz. Zwei Zahlen. Ein Knopf.
notes.5t = Start in zehn Sekunden lesbar

## ── صندوق العزل ────────────────────────────
q.reason1 = Bestätigte Bedrohung
q.reason2 = Crack-Tool
quarantine.opened_from = Öffnet sich aus einem Hinweis und über den Zähler auf Start — nie aus einem versteckten Menü.
quarantine.restore = Wiederherstellen

## ── دستور الواجهة — القواعد الخمس ────────────────────────────
sec.alerts = 2 · Hinweise
sec.alerts.hint = Weniger Text · nie eine Löschtaste · Erklärung auf Anfrage
sec.color = 4 · Farbpalette
sec.color.hint = Kontrast auf der Seite gemessen, nicht behauptet
sec.fonts = 5 · Schriften
sec.fonts.hint = Eine Schrift pro Sprachfamilie · kein Download, keine Kosten
sec.main = 1 · Der Startbildschirm
sec.main.hint = ≤ 8 Elemente · 3 Wände · zwei Tasten · keine Fachbegriffe
sec.notes = Regeln, die aus Ideen Türen machen
sec.usb = 6 · USB
sec.usb.hint = Ein Fortschrittsbalken + eine Phase · nichts mehr

## ── الهوية ────────────────────────────
shield.sub = { $n } Wände aktiv · Bedrohungsdatenbank aktuell

## ── الشرح عند الطلب ────────────────────────────
tips.body = Fahren Sie über das <u>unterstrichene</u> Wort für eine Zeile · klicken Sie es für eine erklärende Suche in Ihrer Sprache (Google · Baidu · Yahoo Japan · Yandex).
tips.legend = Erklärung auf Anfrage:

## ── دستور الواجهة — القواعد الخمس ────────────────────────────
trust.1d = Wir sagen, was geschah, nicht die Technik. Ein Satz.
trust.1t = Klarheit
trust.2d = Höchstens zwei Tasten. Eine Standardoption. Null Einrichtung nötig.
trust.2t = Einfachheit
trust.3d = Kein Ereignis = kein Fenster. ≤ 3 Hinweise pro Tag. Nie ein Modal.
trust.3t = Ruhe
trust.4d = Die Folge jeder Taste steht davor, nicht danach. Und der sichere Weg ist der sichtbare.
trust.4t = Folgen kennen
trust.sub = Jeder Bildschirm wird an diesen vier gemessen. Bei Konflikt: Sicherheit zuerst.

## ── USB ────────────────────────────
usb.stage = Phase

## ── Ziffern ─────────────────────────────
num.title = Ziffern
num.sub = Die Zahlen in Zählern, Hinweisen und Protokoll.
num.body = Wählen Sie die Form, die Sie am schnellsten lesen. Jede Rechnung bleibt gleich.
num.arabic = 0 1 2 3 4 5 6 7 8 9
num.hindi = ٠ ١ ٢ ٣ ٤ ٥ ٦ ٧ ٨ ٩
num.arabic.name = Arabische Ziffern
num.hindi.name = Östliche arabische Ziffern
num.sample = 3 Hinweise · 12 Dateien
num.note = Die Änderung ist sofort. Kein Neustart nötig.

## ── Pausen-Hinweise — auf Anfrage ─────────────────────────────
tip.lvl1 = Kein Fenster, kein Hinweis, kein Ton, aber Prüfung und Blocken laufen im Hintergrund weiter: eine infizierte Datei wird still isoliert und bei der Rückkehr gezeigt.
tip.lvl2 = Keine Prüfung, kein Blocken, kein Hinweis. Der Dienst läuft, tut aber nichts, und Ihr Gerät ist völlig ungeschützt.
tip.no_permanent = Die längste Pause endet mit einem Neustart, und das ist die einzige Grenze, die wir setzen.

## ── Vierte Regel: Bild zuerst ─────────────────────────────
visual.title = Bild zuerst
visual.rule = Das Bild vor dem Satz: Was ein Bildschirm tut, wird gesehen, nicht erklärt.
visual.why = Wer langsam liest oder Ihre Sprache nicht liest, versteht Form und Farbe vor der Zeile.
visual.n1 = Jedes Element trägt Form, Farbe und Ort und hängt nie an einem Wort.
visual.n2 = Dieselbe Form bedeutet auf jedem Bildschirm dasselbe.
visual.n3 = Farbe trennt sicher von gefährlich; der Text nennt den Grund.
visual.n4 = Ein Icon ohne Wort ist ein Test: Muss es erklärt werden, ist es das falsche Icon.
visual.n5 = Text erklärt das Bild, er wiederholt es nicht.
visual.sample = Über das Icon fahren: ein Wort. Klicken: die ganze Erklärung in Ihrer Sprache.
trust.count = Fünf, nicht vier.
help.sample.title = Ein echtes Beispiel
