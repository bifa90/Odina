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
pause.badge = Protection is paused

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
