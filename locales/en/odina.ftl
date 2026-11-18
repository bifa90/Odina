# Odina Antivirus — English (en-US) — SOURCE LANGUAGE
# Format: Fluent (.ftl). Do not reorder keys without updating i18n-lint.
# Rules: no string concatenation (L-rule 3.9.4), no hardcoded strings (18.2.9).
# Key naming: domain.component.key

## ── App identity ──────────────────────────────────────────
app.name = Odina
app.tagline = Your Digital Guardian
app.window.title = Odina — { $state }

## ── States (shield) ───────────────────────────────────────
state.protected = You are protected
state.scanning = Scanning…
state.attention = Needs your attention
state.paused = Protection paused
state.untrusted = Not yet verified

## ── Main screen — protection walls ────────────────────────
wall.signatures.title = File database
wall.signatures.on = Active
wall.signatures.off = Inactive
wall.behavior.title = Deep behavior scan
wall.behavior.on = Active
wall.behavior.off = Idle
wall.proactive.title = Proactive protection
wall.proactive.on = Active
wall.proactive.off = Inactive

## ── Main screen — actions (MAX 8 elements on main screen) ─
action.scan_now = Scan now
action.pause = Pause
action.expert_mode = Expert mode
action.language = Language
action.settings = Settings
action.why = Why?

## ── Scan ──────────────────────────────────────────────────
scan.quick = Quick scan
scan.full = Full scan
scan.usb = Scan this drive
scan.running = Scanning { $target }…
scan.percent = { $percent }%
scan.files_checked = Files checked
scan.threats_found = Threats found
scan.stop = Stop
scan.complete = Scan complete
scan.clean_result = Nothing harmful was found.

## ── Alerts — CONFIRMED (never auto-dismiss) ───────────────
alert.confirmed.title = We stopped a known threat
alert.confirmed.body = { $file } is a confirmed threat and has been isolated — not deleted.

## ── Alerts — HIGHLY SUSPICIOUS ───────────────────────────
alert.suspicious.title = A program is trying to reach your files
alert.suspicious.body = { $file } is changing files in your Documents. We stopped it temporarily.
# RULE 3.9.2: "temporarily" must never be softened in translation.
alert.suspicious.note = We stopped it temporarily.

## ── Alerts — SUSPICIOUS ───────────────────────────────────
alert.unsure.title = This program is unusual
alert.unsure.body = { $file } is new here and behaving oddly.

## ── Alerts — PUA / CRACK (medium risk) ────────────────────
alert.pua.title = This looks like a cracking tool
alert.pua.body = { $file } bypasses licensing protection of paid software.
alert.pua.allowed = Allow it
alert.pua.quarantine = Move aside

## ── Alert buttons (exactly 2 + "why?") ────────────────────
button.allow = Allow
button.block = Block
button.restore = Restore
button.remove_forever = Delete permanently

## ── USB ───────────────────────────────────────────────────
usb.detected = New drive detected — checking it first
usb.scanning = Checking { $name }…
usb.priority_done = Dangerous files checked
usb.trust.new = New drive
usb.trust.scanning = Checking
usb.trust.trusted = Remembered
usb.trust.suspicious = Under watch
usb.trust.quarantined = Blocked

## ── Quarantine ────────────────────────────────────────────
quarantine.title = Isolated items
quarantine.reason = Why it was isolated
quarantine.restore_note = The original location is kept.
quarantine.retention = Kept for { $days } days

## ── Pause ─────────────────────────────────────────────────
pause.title = Pause
pause.duration.10 = 10 minutes
pause.duration.30 = 30 minutes
pause.duration.60 = 1 hour
pause.warning = We stop the alerts until { $until }. We are still protecting you.
pause.confirm = Confirm
pause.badge = Alerts paused · protection still on

## ── Status / errors ──────────────────────────────────────
error.generic = Something went wrong. We did not change anything.
error.tamper = Odina's own files have been changed. Please repair.
error.offline = No update yet. Protection still works.

## ── Updates ───────────────────────────────────────────────
update.available = Update available
update.ask = Check for updates and download them?
update.db_age_warning = Your threat list is { $days } days old.

## ── Path ──────────────────────────────────────────────
path.label = Path
path.copy = Copy path
path.open_folder = Open folder
trust.title = Trust foundations
word.isolated = isolated
word.temporary = temporarily
tip.quarantine = Moved to an encrypted store — its original location is kept and it can be restored.
tip.suspended = We stopped it now. It works again only if you allow it.
quarantine.hint = Everything stays until you decide · never auto-deleted
quarantine.keep = Keep it
quarantine.delete = Delete permanently
quarantine.delete_note = Cannot be undone. No other copy.
quarantine.cleanup_title = In quarantine for { $days } days
quarantine.cleanup_choose = Delete the ones I selected
error.false_positive = We got this file wrong — fixed.
error.locked = Could not isolate it: the file is open right now.

## ── Turn protection off (user's explicit decision) ────────
pause.confirm_disable = Turn protection off for { $minutes} minutes
disable.title = Turn protection off
disable.warning_title = Serious risk
disable.body = We do not just stop the alerts — we stop protecting. No file will be scanned and no threat blocked. Your device is completely unprotected.
disable.responsibility = This is your decision, and the responsibility is yours for that period.
disable.choose = Choose a duration
disable.duration.peak = Until the peak hour ends
disable.duration.30 = 30 minutes
disable.duration.60 = 1 hour
disable.duration.until_reboot = Until I restart
disable.refused = Protection cannot be turned off right now
disable.refused_quarantine = There is a quarantined file you have not decided on yet. Review it first.
disable.refused_scanning = A scan is running. Wait for it or stop it first.
disable.refused_active_threat = An unresolved threat is still open. We will not turn protection off while it is.
state.unprotected = Protection is off
state.unprotected_until = Protection off until { $until}
state.unprotected_badge = Unprotected — by your decision
action.turn_off = Turn protection off
action.turn_on = Turn protection back on
disable.re_enable_note = You can turn protection back on at any moment — no password.
disable.reminder = { $minutes} minutes left before protection comes back automatically.
pause.resume_report = While alerts were paused: { $count} file(s) isolated

## ── Surgical exception (D45 · §4.8) ─────────────────────────
exception.title = Exceptions
exception.sub = If your work needs one rule to bend, allow just that one. The rest stays as it was.
exception.rule.a1.wsh = Windows scripts
exception.rule.a1.wsh.desc = .vbs and .js files run restricted unless signed by a known publisher.
exception.rule.a1.powershell = PowerShell commands
exception.rule.a1.powershell.desc = PowerShell runs in constrained mode. Allowing it does not open Windows Script Host.
exception.rule.a1.macros = Office macros
exception.rule.a1.macros.desc = Unsigned documents cannot run macros. Allowing it does not touch scripts.
exception.rule.a3.downloads = Downloads suspicion
exception.rule.a3.downloads.desc = Every new file in Downloads counts as suspicious until cleared.
exception.rule.a4.periodic = Periodic scan
exception.rule.a4.periodic.desc = A silent scan every two hours of startup items and sensitive keys.
exception.rule.a6.doubt = Caution when doubtful
exception.rule.a6.doubt.desc = When doubtful it blocks first, then asks. It can be relaxed, never allowed outright.
exception.state.on = Active
exception.state.relaxed_until = Relaxed until { $until }
exception.state.allowed_until = Allowed until { $until }
exception.state.allowed_permanent = Allowed permanently — your decision
exception.consequence.relax = { $rule } will run without alerts until { $until }, then return. Nothing is blocked during that time.
exception.consequence.allow = { $rule } will run unrestricted until { $until }, then return. Nothing else changes.
exception.consequence.permanent = { $rule } stays allowed until you revoke it. This will remain written on the home screen.
exception.action.relax_10 = Relax for 10 minutes
exception.action.relax_30 = Relax for 30 minutes
exception.action.relax_60 = Relax for an hour
exception.action.allow = Allow
exception.action.revoke = Revoke
exception.action.revoke_all = Revoke all exceptions
exception.action.advanced = Advanced options
exception.badge = { $rule } allowed · until { $until }
exception.badge_permanent = { $rule } allowed permanently · your decision
exception.badge_many = { $count } rules allowed by your decision · details
exception.until_reboot = until restart
exception.permanent_note = A permanent exception does not end on its own. It stays visible on the home screen until you revoke it.
exception.revoke_note = You can revoke at any moment — one click, no password.
exception.expert_note = Expert mode reveals “permanent” and “the whole rule”. No password, and no question about why.
exception.expired_report = The exception has ended. During it, { $count } file(s) looked suspicious — review them now.
exception.expired_clean = The exception has ended. Nothing appeared that needs review.
exception.refused.not_allowed = This rule cannot be allowed outright — only relaxed.
exception.refused.above_ceiling = The furthest this rule can go is relaxed.
exception.refused.no_permanent = This rule cannot be excepted permanently.
exception.refused.expert_only = A permanent whole-rule exception needs expert mode.
exception.refused.veto_stands = The file is confirmed. An exception relaxes the rule; it does not clear a file.

exception.live_note = your decision · everything else is still active · you can revoke at any moment
exception.lines_title = Four things are never excepted:
exception.lines_body = the first wall's verdict (confirmed is quarantined even under an exception) · tamper protection · the second rule (a WSH exception does not open PowerShell) · “high risk” and above. Revoking is always one click, without a password.

## ── App map and navigation (constitution §6أ.3) ─────────────────
nav.home = Home
nav.quarantine = Quarantine
nav.proactive = Proactive protection
nav.alerts = Alert centre
nav.settings = Settings
nav.language = Language
nav.privacy = Privacy
back = Back
menu.more = More
menu.advanced = Advanced options
map.title = App map
map.hint = How every screen is reached · three levels at most

## ── Code signing policy (public page) ─────────────────────────
signing.title = Code signing policy
signing.sub = What we sign, what we never sign, where the key lives, and how to verify it yourself.
signing.status_title = Status today
signing.status_body = This policy is published before signing begins, and no build has been signed yet. The first signed build will be announced on its release page with its hash.
signing.status_until = Until then the program ships as a portable ZIP archive, never as an unsigned installer.
signing.build_title = Build
signing.build_body = Builds run on GitHub Actions from the commit named in the release notes. There is no local build, and a binary is never rebuilt before signing.
signing.key_title = Key
signing.key_body = We hold no signing key. The key lives in SignPath Foundation's HSM, and every signing request needs explicit maintainer approval.
signing.signed_title = Signed
signing.signed_installer = The installer
signing.signed_store = The Store package
signing.unsigned_title = Not signed
signing.unsigned_db = Signature databases — signed with a separate Ed25519 key, because they update far more often than the program.
signing.unsigned_tools = Everything under tools — MIT-licensed and distributed as source, never as binaries.
signing.verify_title = Verification
signing.verify_body = A SHA-256 for every artefact is published on the release page, before and after signing. You can verify it yourself on Windows:
signing.verify_must = Status must read Valid and the publisher must be SignPath Foundation. If the hash differs from the published one, do not run the file.
signing.revoke_title = Revocation
signing.revoke_body = If a build is ever produced from unreviewed code, it is revoked and the revocation is announced before anything else.
signing.credit = Free code signing provided by SignPath.io, certificate by SignPath Foundation.
signing.updated = Last updated

## ── Glossary — one line per term ─────────────
tip.confirmed = Its hash is in the threat database. Not a guess.
tip.high_risk = Strong signs of danger, and the hash is not confirmed yet.
tip.suspicious = Looks odd: the name, the place, or the behaviour. Unverified.
tip.pua = Not a virus, but it bypasses another program's protection.
tip.signature = Hashes of known files. Updating them keeps protection current.
tip.behavior = Watches what the program does, not what it is called.
tip.proactive = Stops the act before it happens instead of finding it after.
tip.pause = We silence the windows only. Scanning and blocking continue.
tip.turn_off = No scanning and no blocking. Your device is unprotected meanwhile.
tip.exception = Relaxing one rule by your decision. Nothing else changes.
tip.expert_mode = Reveals permanent and whole-rule options. No password, no questions.
tip.wsh = The Windows script host. A common way in for malware.
tip.powershell = A command tool in Windows. Restricted by default from risky sources.
tip.macros = Commands inside an Office document. They do not run in unsigned files.
tip.amsi = A Windows interface that lets us see a script before it runs.
tip.wdac = A Windows policy that blocks one specific file from running.
tip.usb_trust = The first time we see it. Scanned before it is opened.
tip.tamper = Odina cannot be switched off by editing its files or killing its service.
tip.hash = A string that represents the whole file. One byte changes all of it.
tip.heuristic = A signal that is measured and weighed. Never enough on its own.

## ── Settings, privacy and language ─────────────────────────────
settings.title = Settings
settings.sub = Four sections. Nothing hidden behind “advanced” except what is rare.
settings.appearance = Appearance
settings.appearance.body = Light or dark. Follows the system by default.
settings.theme_light = Light
settings.theme_dark = Dark
settings.theme_system = Follow system
settings.behavior = Behaviour
settings.startup = Starts with Windows
settings.startup.body = Runs in the background from boot. You can turn it off.
settings.notifications = Notifications
settings.notifications.body = Three a day at most. The most important first.
settings.updates = Updates
settings.updates.body = Automatic for the threat database, manual for the program.
settings.about = About
settings.version = Version
settings.database = Threat database
settings.database.date = Last updated

privacy.title = Privacy
privacy.sub = What leaves your device, and what never does. No exceptions.
privacy.telemetry = Zero telemetry
privacy.telemetry.body = No statistics, no device id, no fingerprint. Nothing is sent automatically.
privacy.local = Everything local
privacy.local.body = Scanning, decisions and the log live on your disk.
privacy.report = The report is your decision
privacy.report.body = A report is sent only if you choose to send it.
privacy.search = Search on demand
privacy.search.body = Opens a search engine in your language. We send the term only, never your files.
privacy.log = The log
privacy.log.body = Local, and readable in your language. You can clear it whenever you like.
privacy.log_clear = Clear the log

lang.title = Language
lang.sub = Ten languages. The translation shows immediately, and the font follows your language.
lang.current = Current language
lang.rtl = Written right to left
lang.ltr = Written left to right
lang.review = Native review
lang.review.body = A language no native speaker has reviewed is suspended, not shipped.

home.walls = The three walls
home.action.scan = Scan now
home.action.pause = Pause
home.action.expert = Expert mode
home.action.why = Why?
home.goto.quarantine = Quarantine
home.goto.alerts = Alert centre
home.goto.settings = Settings
home.goto.proactive = Proactive protection
home.quarantine_count = { $count } file(s) in quarantine
home.alerts_count = { $count } alert(s) worth a look
home.proactive_summary = Six rules · one excepted
proactive.title = Proactive protection
proactive.sub = Six rules. Each has its own switch, and none opens another.
alerts.title = Alert centre
alerts.sub = Everything worth a look in one place. No pop-ups.
alerts.empty = Nothing worth a look right now.
exception.detail.title = Exception — one rule
exception.detail.sub = What changes, for how long, and how it comes back.
map.node.home = Home
map.node.quarantine = Quarantine
map.node.proactive = Proactive
map.edge.from_home = from Home
map.edge.from_alert = from an alert
map.legend = A solid line is a permanent entry · a dotted one needs a context

## ── Explanation on demand ─────────────────────────────
help.title = Explanation on demand
help.hint = One line on hover · a full search on click, in your language.
help.tip.title = Instant hint
help.tip.body = One underlined word, and one line of at most 90 characters.
help.guide.title = A longer guide
help.guide.body = Opens from a button inside the hint, and reads in an engine you know.
help.guide.cta = Open the search
help.engine.title = The engine follows language and place
help.col.lang = Language
help.col.engine = Engine
help.col.why = Why
help.why.default = The widest coverage for this language
help.why.zh = The reference inside China, where Google is limited
help.why.ja = Ranks Japanese pages first, not translated ones
help.why.ru = Understands Russian morphology better than the rest
help.query.note = We send the term alone. No file name, no path, no device id.
path.copy_hint = copy
nav.preview = UI preview

## ── التنبيهات والشارات ────────────────────────────
alert.confirmed.cta = Open quarantine
badge.confirmed = Confirmed
badge.high = Highly suspicious
badge.low = Suspicious
badge.pua = Medium risk
button.off = Off
button.on = On

## ── الإيقاف المؤقت ────────────────────────────
lvl.app = The app
lvl.d10 = { $n } min
lvl.d30 = { $n } min
lvl.d60 = 1 hour
lvl.dpeak = Until peak hour ends
lvl.dreboot = Until I restart
lvl.hint = We tell you, and you decide.
lvl.no_permanent = No permanent pause. The longest ends at restart.
lvl.state_off = Protection is off
lvl.state_off_sub = until 14:30 · by your decision
lvl.title = Turning off — two levels, not one
lvl.what_continues = What continues
lvl.what_stops = What stops
lvl1.app = runs · scans · blocks · asks nothing
lvl1.body = No window, no sound. Scanning and blocking continue.
lvl1.cont = All three walls
lvl1.name = Silence alerts
lvl1.stops = The alerts only
lvl2.body = No scanning, no blocking. Your device is unprotected.
lvl2.name = Stop protection
lvl2.resp = The responsibility is yours for that time.
lvl2.warn = We stop protecting, not telling.

## ── دستور الواجهة — القواعد الخمس ────────────────────────────
notes.1b = changes direction and layout instantly; numbers stay Latin in ratios and hashes.
notes.1t = Language switch:
notes.2b = Allow/Block keep a fixed logical order; RTL flips visual position, not the role.
notes.2t = Buttons:
notes.3b = rendered with system fonts here; after the CDN pack downloads they become Noto without restart.
notes.3t = CJK:
notes.4b = this file loads fonts from the adjacent fonts/vendor/ folder — serve it over HTTP, not file://.
notes.4t = Fonts:
notes.5b = logotype, tray icon and error states — reviewed separately (R3).
notes.5t = Not covered here:

## ── صندوق العزل ────────────────────────────
q.reason1 = Confirmed threat
q.reason2 = Cracking tool
quarantine.opened_from = Opened from an alert, and from the counter on Home — never from a hidden menu.
quarantine.restore = Restore

## ── دستور الواجهة — القواعد الخمس ────────────────────────────
sec.alerts = 2 · The four alerts
sec.alerts.hint = 2 buttons + “Why?” · color never carries meaning alone
sec.color = 4 · Palette (measured WCAG)
sec.color.hint = every value shows its contrast against the current background
sec.fonts = 5 · Fonts per language
sec.fonts.hint = only 400 and 600 · system fonts as instant fallback
sec.main = 1 · Main screen
sec.main.hint = ≤ 8 elements · 3 walls · 2 buttons · no jargon
sec.notes = 6 · Review notes
sec.usb = 3 · USB alert
sec.usb.hint = one progress bar · one stage · RTL-aware

## ── الهوية ────────────────────────────
shield.sub = { $n } walls active · threat list up to date

## ── الشرح عند الطلب ────────────────────────────
tips.body = Hover the underlined word for one line · click it to open an explaining search in your language (Google · Baidu · Yahoo Japan · Yandex).
tips.legend = Explanation on demand:

## ── دستور الواجهة — القواعد الخمس ────────────────────────────
trust.1d = We say what happened, not how it works. One sentence. No “done” without a reason.
trust.1t = Clarity
trust.2d = Two buttons at most. One primary action. Zero setup required.
trust.2t = Simplicity
trust.3d = No event = no window. ≤ 3 notifications a day. Never a modal.
trust.3t = Calm
trust.4d = Every button states its outcome before you press it. The safe choice is the visible one.
trust.4t = Known consequences
trust.sub = Every screen is measured against these four. On conflict: safety → no interruption → clarity → simplicity → beauty.

## ── USB ────────────────────────────
usb.stage = P0 · dangerous files in root

## ── Numerals ─────────────────────────────
num.title = Numerals
num.sub = The numbers in counters, alerts and the log.
num.body = Choose the shape you read fastest. Every calculation is the same.
num.arabic = 0 1 2 3 4 5 6 7 8 9
num.hindi = ٠ ١ ٢ ٣ ٤ ٥ ٦ ٧ ٨ ٩
num.arabic.name = Arabic numerals
num.hindi.name = Eastern Arabic numerals
num.sample = 3 alerts · 12 files
num.note = The change is instant. No restart needed.

## ── Pause tips — on demand ─────────────────────────────
tip.lvl1 = No window, no notice, no sound, but scanning and blocking continue in the background: an infected file is isolated silently and shown when you return.
tip.lvl2 = No scanning, no blocking, no alert. The service runs but does nothing, and your device is fully unprotected.
tip.no_permanent = The longest pause ends at a restart, and that is the only limit we impose.

## ── Rule four: visual first ─────────────────────────────
visual.title = Visual first
visual.rule = The picture before the sentence: what a screen does is seen, not explained.
visual.why = Someone who reads slowly, or does not read your language, understands the icon and the colour before the line.
visual.n1 = Every element carries shape, colour and place, so it never rests on a word.
visual.n2 = The same shape means the same thing on every screen.
visual.n3 = Colour separates safe from danger; text names the reason.
visual.n4 = An icon without a word is a test: if it needs explaining, it is the wrong icon.
visual.n5 = Text explains the picture, it does not repeat it.
visual.sample = Hover the icon: one word. Click it: a full explanation in your language.
trust.count = Five, not four.
help.sample.title = A live example
