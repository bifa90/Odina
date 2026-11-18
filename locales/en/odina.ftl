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
exception.title = Exceptions — one rule, not all of them
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
