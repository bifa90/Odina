# Odina Antivirus — Français (fr)
# المستوى: T-B (مجتمعية) — تحتاج مراجعة ناطق أصلي قبل ترقيتها إلى T-A (§30.2)
# راجع كل سطر يدويًا قبل الاعتماد. لا ترجمة آلية للنصوص الأمنية (§3.9.1).

app.name = Odina
app.tagline = Votre gardien numérique
app.window.title = Odina — { $state }
state.protected = Vous êtes protégé
state.scanning = Analyse en cours…
state.attention = Nécessite votre attention
state.paused = Protection en pause
state.untrusted = Pas encore vérifié
wall.signatures.title = Base de fichiers
wall.signatures.on = Active
wall.signatures.off = Inactive
wall.behavior.title = Analyse comportementale approfondie
wall.behavior.on = À l'écoute
wall.behavior.off = Inactive
wall.proactive.title = Protection proactive
wall.proactive.on = Activée
wall.proactive.off = Désactivée
action.scan_now = Analyser maintenant
action.pause = Mettre en pause
action.expert_mode = Mode expert
action.language = Langue
action.settings = Paramètres
action.why = Pourquoi ?
scan.quick = Analyse rapide
scan.full = Analyse complète
scan.usb = Analyser ce lecteur
scan.running = Analyse de { $target }…
scan.percent = { $percent } %
scan.files_checked = Fichiers vérifiés
scan.threats_found = Menaces trouvées
scan.stop = Arrêter
scan.complete = Analyse terminée
scan.clean_result = Rien de dangereux n'a été trouvé.
alert.confirmed.title = Nous avons arrêté une menace connue
alert.confirmed.body = { $file } est une menace confirmée et a été isolé.
alert.suspicious.title = Un programme tente d'accéder à vos fichiers
alert.suspicious.body = { $file } modifie des fichiers dans Documents. Nous l'avons arrêté temporairement.
alert.suspicious.note = Nous l'avons arrêté temporairement.
alert.unsure.title = Ce programme est inhabituel
alert.unsure.body = { $file } est nouveau ici et se comporte étrangement. Le connaissez-vous ?
alert.pua.title = Cela ressemble à un outil de crack
alert.pua.body = { $file } contourne la protection de licence d'un logiciel payant. Vous décidez de ce qui s'exécute sur votre appareil.
alert.pua.allowed = Autoriser quand même
alert.pua.quarantine = Mettre de côté
button.allow = Autoriser
button.block = Bloquer
button.restore = Restaurer
button.remove_forever = Supprimer définitivement
usb.detected = Nouveau lecteur détecté — nous le vérifions d'abord
usb.scanning = Vérification de { $name }…
usb.priority_done = Fichiers dangereux vérifiés
usb.trust.new = Nouveau lecteur
usb.trust.scanning = Vérification
usb.trust.trusted = Mémorisé
usb.trust.suspicious = Sous surveillance
usb.trust.quarantined = Bloqué
quarantine.title = Éléments isolés
quarantine.reason = Raison de l'isolement
quarantine.restore_note = L'emplacement d'origine est conservé.
quarantine.retention = Conservé { $days } jours
pause.title = Mettre la protection en pause
pause.duration.10 = 10 minutes
pause.duration.30 = 30 minutes
pause.duration.60 = 1 heure
pause.warning = Vous ne serez pas protégé jusqu'à { $until }.
pause.confirm = Mettre en pause quand même
pause.badge = Alertes en pause · la protection continue
error.generic = Une erreur est survenue. Nous n'avons rien modifié.
error.tamper = Les fichiers d'Odina ont été modifiés. Réparation nécessaire.
error.offline = Pas encore de mise à jour. La protection reste active.
update.available = Mise à jour disponible
update.ask = Rechercher et télécharger les mises à jour ?
update.db_age_warning = Votre liste de menaces date de { $days } jours.

## ── Path ──────────────────────────────────────────────
path.label = Chemin
path.copy = Copier le chemin
path.open_folder = Ouvrir le dossier
trust.title = Fondements de la confiance
word.isolated = isolé
word.temporary = temporairement
tip.quarantine = Déplacé vers un stockage chiffré — l'emplacement d'origine est conservé.
tip.suspended = Nous l'avons arrêté. Il ne fonctionnera qu'avec votre autorisation.
quarantine.hint = Tout reste jusqu'à votre décision · jamais supprimé automatiquement
quarantine.keep = Conserver
quarantine.delete = Supprimer définitivement
quarantine.delete_note = Irréversible. Aucune autre copie.
quarantine.cleanup_title = En quarantaine depuis { $days } jours
quarantine.cleanup_choose = Supprimer ceux que j'ai choisis
error.false_positive = Nous nous sommes trompés sur ce fichier — corrigé.
error.locked = Impossible d'isoler : le fichier est ouvert.

## ── Turn protection off (user's explicit decision) ────────
pause.confirm_disable = Désactiver la protection pendant { $minutes} minutes
disable.title = Désactiver la protection
disable.warning_title = Risque grave
disable.body = Nous n'arrêtons pas seulement d'alerter : nous arrêtons de protéger. Aucun fichier ne sera analysé, aucune menace bloquée.
disable.responsibility = C'est votre décision, et la responsabilité vous revient pendant cette période.
disable.choose = Choisissez la durée
disable.duration.peak = Jusqu'à la fin de l'heure de pointe
disable.duration.30 = 30 minutes
disable.duration.60 = 1 heure
disable.duration.until_reboot = Jusqu'au redémarrage
disable.refused = Impossible de désactiver la protection maintenant
disable.refused_quarantine = Un fichier en quarantaine attend votre décision. Examinez-le d'abord.
disable.refused_scanning = Une analyse est en cours. Attendez ou arrêtez-la.
disable.refused_active_threat = Une menace non traitée est encore ouverte. Nous ne désactivons pas la protection tant qu'elle l'est.
state.unprotected = Protection désactivée
state.unprotected_until = Protection désactivée jusqu'à { $until}
state.unprotected_badge = Sans protection — votre décision
action.turn_off = Désactiver la protection
action.turn_on = Réactiver la protection
disable.re_enable_note = Vous pouvez réactiver la protection à tout moment, sans mot de passe.
disable.reminder = Encore { $minutes} minutes avant le retour automatique de la protection.
pause.resume_report = Pendant la pause des alertes : { $count} fichier(s) isolé(s)

## ── Exception chirurgicale (D45 · §4.8) ─────────────────────────
exception.title = Exceptions — une règle, pas toutes
exception.sub = Si votre travail exige qu'une règle cède, autorisez seulement celle-là. Le reste ne bouge pas.
exception.rule.a1.wsh = Scripts Windows
exception.rule.a1.wsh.desc = Les fichiers .vbs et .js s'exécutent en mode restreint sauf s'ils sont signés par un éditeur connu.
exception.rule.a1.powershell = Commandes PowerShell
exception.rule.a1.powershell.desc = PowerShell s'exécute en mode restreint. L'autoriser n'ouvre pas Windows Script Host.
exception.rule.a1.macros = Macros Office
exception.rule.a1.macros.desc = Les documents non signés ne peuvent pas exécuter de macros. L'autoriser ne touche pas aux scripts.
exception.rule.a3.downloads = Soupçon dans Téléchargements
exception.rule.a3.downloads.desc = Chaque nouveau fichier dans Téléchargements est suspect jusqu'à preuve du contraire.
exception.rule.a4.periodic = Analyse périodique
exception.rule.a4.periodic.desc = Toutes les deux heures, une analyse silencieuse des éléments de démarrage et des clés sensibles.
exception.rule.a6.doubt = Prudence en cas de doute
exception.rule.a6.doubt.desc = En cas de doute, il bloque d'abord puis demande. Peut être assoupli, jamais autorisé totalement.
exception.state.on = Active
exception.state.relaxed_until = Assouplie jusqu'à { $until }
exception.state.allowed_until = Autorisée jusqu'à { $until }
exception.state.allowed_permanent = Autorisée en permanence — votre décision
exception.consequence.relax = { $rule } fonctionnera sans alerte jusqu'à { $until}, puis reviendra. Rien n'est bloqué pendant ce temps.
exception.consequence.allow = { $rule } fonctionnera sans restriction jusqu'à { $until}, puis reviendra. Le reste ne change pas.
exception.consequence.permanent = { $rule } restera autorisée jusqu'à révocation. Cette ligne restera affichée sur l'écran d'accueil.
exception.action.relax_10 = Assouplir 10 minutes
exception.action.relax_30 = Assouplir 30 minutes
exception.action.relax_60 = Assouplir une heure
exception.action.allow = Autoriser
exception.action.revoke = Révoquer
exception.action.revoke_all = Révoquer toutes les exceptions
exception.action.advanced = Options avancées
exception.badge = { $rule } autorisée · jusqu'à { $until }
exception.badge_permanent = { $rule } autorisée en permanence · votre décision
exception.badge_many = { $count } règles autorisées par votre décision · détails
exception.until_reboot = jusqu'au redémarrage
exception.permanent_note = Une exception permanente ne s'arrête pas d'elle-même. Elle reste visible jusqu'à révocation.
exception.revoke_note = Vous pouvez révoquer à tout moment — un clic, sans mot de passe.
exception.expert_note = Le mode expert révèle « permanent » et « la règle entière ». Sans mot de passe et sans demander pourquoi.
exception.expired_report = L'exception est terminée. Pendant celle-ci, { $count } fichier(s) ont paru suspects — vérifiez-les maintenant.
exception.expired_clean = L'exception est terminée. Rien n'est apparu qui demande vérification.
exception.refused.not_allowed = Cette règle ne peut pas être totalement autorisée — seulement assouplie.
exception.refused.above_ceiling = Le maximum pour cette règle est l'assouplissement.
exception.refused.no_permanent = Cette règle ne peut pas faire l'objet d'une exception permanente.
exception.refused.expert_only = Une exception permanente sur toute la règle exige le mode expert.
exception.refused.veto_stands = Le fichier est confirmé. Une exception assouplit la règle ; elle n'innocente pas un fichier.

exception.live_note = votre décision · tout le reste reste actif · vous pouvez révoquer à tout moment
exception.lines_title = Quatre choses ne sont jamais exceptées :
exception.lines_body = le verdict du premier mur (un fichier confirmé est isolé même sous exception) · la protection contre les altérations · la seconde règle (une exception WSH n'ouvre pas PowerShell) · « risque élevé » et au-delà. Révoquer se fait toujours en un clic, sans mot de passe.

## ── Carte de l'application et navigation (constitution §6أ.3) ─────────────────
nav.home = Accueil
nav.quarantine = Quarantaine
nav.proactive = Protection proactive
nav.alerts = Centre d'alertes
nav.settings = Réglages
nav.language = Langue
nav.privacy = Confidentialité
back = Retour
menu.more = Plus
menu.advanced = Options avancées
map.title = Carte de l'application
map.hint = Comment chaque écran est atteint · trois niveaux au maximum

## ── Politique de signature de code (page publique) ─────────────────────────
signing.title = Politique de signature de code
signing.sub = Ce que nous signons, ce que nous ne signons jamais, où vit la clé et comment le vérifier vous-même.
signing.status_title = État aujourd'hui
signing.status_body = Cette politique est publiée avant le début de la signature, et aucune compilation n'a encore été signée. La première compilation signée sera annoncée sur sa page de version avec son empreinte.
signing.status_until = D'ici là, le programme est distribué en archive ZIP portable, jamais en installateur non signé.
signing.build_title = Compilation
signing.build_body = Toutes les compilations ont lieu sur GitHub Actions depuis le commit indiqué dans les notes de version. Pas de compilation locale, et un binaire n'est jamais recompilé avant signature.
signing.key_title = La clé
signing.key_body = Nous ne détenons aucune clé de signature. La clé vit dans le HSM de la SignPath Foundation, et chaque demande de signature exige l'approbation explicite d'un mainteneur.
signing.signed_title = Signé
signing.signed_installer = L'installateur
signing.signed_store = Le paquet du Store
signing.unsigned_title = Non signé
signing.unsigned_db = Les bases de signatures — signées avec une clé Ed25519 distincte, car elles se mettent à jour bien plus souvent que le programme.
signing.unsigned_tools = Tout ce qui est sous tools — sous licence MIT et distribué en source, jamais en binaire.
signing.verify_title = Vérification
signing.verify_body = Un SHA-256 pour chaque artefact est publié sur la page de version, avant et après signature. Vous pouvez le vérifier vous-même sous Windows :
signing.verify_must = Le statut doit être Valid et l'éditeur doit être SignPath Foundation. Si l'empreinte diffère de celle publiée, n'exécutez pas le fichier.
signing.revoke_title = Révocation
signing.revoke_body = Si une compilation provient de code non relu, elle est révoquée et la révocation est annoncée avant toute autre chose.
signing.credit = Signature de code gratuite fournie par SignPath.io, certificat par la SignPath Foundation.
signing.updated = Dernière mise à jour

## ── Glossaire — une ligne par terme ─────────────
tip.confirmed = Son empreinte est dans la base des menaces. Ce n'est pas une supposition.
tip.high_risk = Signes forts de danger, et l'empreinte n'est pas encore confirmée.
tip.suspicious = Semble étrange : le nom, l'endroit ou le comportement. Non vérifié.
tip.pua = Pas un virus, mais il contourne la protection d'un autre programme.
tip.signature = Empreintes de fichiers connus. Les mettre à jour garde la protection à jour.
tip.behavior = Observe ce que fait le programme, pas son nom.
tip.proactive = Arrête l'acte avant qu'il n'ait lieu au lieu de le trouver après.
tip.pause = Nous faisons taire les fenêtres seulement. L'analyse et le blocage continuent.
tip.turn_off = Ni analyse ni blocage. Votre appareil est sans protection pendant ce temps.
tip.exception = Assouplir une règle selon votre décision. Rien d'autre ne change.
tip.expert_mode = Révèle les options permanente et règle entière. Sans mot de passe ni questions.
tip.wsh = L'hôte de scripts Windows. Une voie fréquente pour les malwares.
tip.powershell = Un outil de commandes sous Windows. Restreint par défaut depuis les sources risquées.
tip.macros = Des commandes dans un document Office. Elles ne s'exécutent pas dans un fichier non signé.
tip.amsi = Une interface Windows qui nous montre un script avant son exécution.
tip.wdac = Une politique Windows qui empêche un fichier précis de s'exécuter.
tip.usb_trust = La première fois que nous le voyons. Analysé avant d'être ouvert.
tip.tamper = Odina ne peut pas être éteint en modifiant ses fichiers ou en tuant son service.
tip.hash = Une chaîne qui représente le fichier entier. Un octet la change entièrement.
tip.heuristic = Un signal mesuré et pondéré. Jamais suffisant à lui seul.

## ── Réglages, confidentialité et langue ─────────────────────────────
settings.title = Réglages
settings.sub = Quatre sections. Rien de caché derrière « avancé » sauf le rare.
settings.appearance = Apparence
settings.appearance.body = Clair ou sombre. Suit le système par défaut.
settings.theme_light = Clair
settings.theme_dark = Sombre
settings.theme_system = Suivre le système
settings.behavior = Comportement
settings.startup = Démarre avec Windows
settings.startup.body = Tourne en arrière-plan dès le démarrage. Vous pouvez le désactiver.
settings.notifications = Notifications
settings.notifications.body = Trois par jour au maximum. Le plus important d'abord.
settings.updates = Mises à jour
settings.updates.body = Automatiques pour la base des menaces, manuelles pour le programme.
settings.about = À propos
settings.version = Version
settings.database = Base des menaces
settings.database.date = Dernière mise à jour

privacy.title = Confidentialité
privacy.sub = Ce qui quitte votre appareil, et ce qui jamais ne le quitte. Sans exception.
privacy.telemetry = Zéro télémétrie
privacy.telemetry.body = Ni statistiques, ni identifiant d'appareil, ni empreinte. Rien n'est envoyé seul.
privacy.local = Tout est local
privacy.local.body = L'analyse, la décision et le journal vivent sur votre disque.
privacy.report = Le rapport est votre décision
privacy.report.body = Un rapport n'est envoyé que si vous choisissez de l'envoyer.
privacy.search = Recherche à la demande
privacy.search.body = Ouvre un moteur de recherche dans votre langue. Nous n'envoyons que le terme, jamais vos fichiers.
privacy.log = Le journal
privacy.log.body = Local et lisible dans votre langue. Vous pouvez l'effacer quand vous voulez.
privacy.log_clear = Effacer le journal

lang.title = Langue
lang.sub = Dix langues. La traduction s'affiche aussitôt, la police suit votre langue.
lang.current = Langue actuelle
lang.rtl = S'écrit de droite à gauche
lang.ltr = S'écrit de gauche à droite
lang.review = Relecture native
lang.review.body = Une langue sans relecteur natif est suspendue, pas publiée.

home.walls = Les trois murs
home.action.scan = Analyser maintenant
home.action.pause = Mettre en pause
home.action.expert = Mode expert
home.action.why = Pourquoi ?
home.goto.quarantine = Quarantaine
home.goto.alerts = Centre d'alertes
home.goto.settings = Réglages
home.goto.proactive = Protection proactive
home.quarantine_count = { $count } fichier(s) en quarantaine
home.alerts_count = { $count } alerte(s) qui valent un regard
home.proactive_summary = Six règles · une exceptée
proactive.title = Protection proactive
proactive.sub = Six règles. Chacune son interrupteur, aucune n'en ouvre une autre.
alerts.title = Centre d'alertes
alerts.sub = Tout ce qui vaut un regard au même endroit. Aucune fenêtre surgissante.
alerts.empty = Rien qui vaille un regard pour l'instant.
exception.detail.title = Exception — une règle
exception.detail.sub = Ce qui change, pour combien de temps, et comment cela revient.
map.node.home = Accueil
map.node.quarantine = Quarantaine
map.node.proactive = Proactive
map.edge.from_home = depuis l'accueil
map.edge.from_alert = depuis une alerte
map.legend = Trait plein : entrée permanente · pointillé : demande un contexte

## ── Explication à la demande ─────────────────────────────
help.title = Explication à la demande
help.hint = Une ligne au survol · une recherche complète au clic, dans votre langue.
help.tip.title = Indice immédiat
help.tip.body = Un mot souligné et une ligne de 90 caractères au plus.
help.guide.title = Un guide plus long
help.guide.body = S'ouvre par un bouton dans l'indice et se lit dans un moteur que vous connaissez.
help.guide.cta = Ouvrir la recherche
help.engine.title = Le moteur suit la langue et le lieu
help.col.lang = Langue
help.col.engine = Moteur
help.col.why = Raison
help.why.default = La couverture la plus large pour cette langue
help.why.zh = La référence en Chine, où Google est limité
help.why.ja = Priorise les pages japonaises, non traduites
help.why.ru = Comprend mieux la morphologie russe que les autres
help.query.note = Nous n'envoyons que le terme. Ni nom de fichier, ni chemin, ni identifiant d'appareil.
path.copy_hint = copier
nav.preview = Aperçu de l'interface

## ── التنبيهات والشارات ────────────────────────────
alert.confirmed.cta = Ouvrir la quarantaine
badge.confirmed = Confirmé
badge.high = Risque élevé
badge.low = Suspect
badge.pua = Risque moyen
button.off = Désactivé
button.on = Activé

## ── الإيقاف المؤقت ────────────────────────────
lvl.app = L'application
lvl.d10 = { $n } minutes
lvl.d30 = { $n } minutes
lvl.d60 = Une heure
lvl.dpeak = Jusqu'à la fin de l'heure de pointe
lvl.dreboot = Jusqu’au redémarrage
lvl.hint = Nous vous informons, vous décidez.
lvl.no_permanent = Pas de pause permanente. La plus longue finit au redémarrage.
lvl.state_off = Protection en pause
lvl.state_off_sub = Jusqu'à 14:30 · votre décision
lvl.title = Pause — deux niveaux, pas un
lvl.what_continues = Ce qui continue
lvl.what_stops = Ce qui s'arrête
lvl1.app = Tourne · analyse · bloque · ne demande pas
lvl1.body = Ni fenêtre ni son. L’analyse et le blocage continuent.
lvl1.cont = Les trois murs
lvl1.name = Mettre les alertes en sourdine
lvl1.stops = Seulement les alertes
lvl2.body = Ni analyse ni blocage. Votre appareil est sans protection.
lvl2.name = Arrêter la protection
lvl2.resp = La responsabilité vous revient pendant ce temps.
lvl2.warn = Nous arrêtons de protéger, pas d’avertir.

## ── دستور الواجهة — القواعد الخمس ────────────────────────────
notes.1b = Un nom de mur est une promesse, pas une étiquette.
notes.1t = Trois murs, trois promesses
notes.2b = Le chemin sûr est le visible, jamais le caché.
notes.2t = Une décision recommandée
notes.3b = Une icône sans mot est une énigme ; une phrase en dessous la résout.
notes.3t = Les icônes s'expliquent
notes.4b = Aucune fenêtre qui prend l'écran. L'alerte vit dans la centrale.
notes.4t = Jamais de modal
notes.5b = Une phrase d’état. Deux chiffres. Un bouton.
notes.5t = L'accueil se lit en dix secondes

## ── صندوق العزل ────────────────────────────
q.reason1 = Menace confirmée
q.reason2 = Outil de crack
quarantine.opened_from = S'ouvre depuis une alerte et depuis le compteur de l'accueil — jamais depuis un menu caché.
quarantine.restore = Restaurer

## ── دستور الواجهة — القواعد الخمس ────────────────────────────
sec.alerts = 2 · Alertes
sec.alerts.hint = Moins de texte · jamais de bouton supprimer · explication à la demande
sec.color = 4 · Palette de couleurs
sec.color.hint = Contraste mesuré sur la page, non affirmé
sec.fonts = 5 · Polices
sec.fonts.hint = Une police par famille de langues · pas de téléchargement, pas de coût
sec.main = 1 · L'écran d'accueil
sec.main.hint = ≤ 8 éléments · 3 murs · deux boutons · aucun terme technique
sec.notes = Des règles qui transforment des idées en portes
sec.usb = 6 · USB
sec.usb.hint = Une barre de progression + une phase · rien de plus

## ── الهوية ────────────────────────────
shield.sub = { $n } murs actifs · base des menaces à jour

## ── الشرح عند الطلب ────────────────────────────
tips.body = Survolez le mot <u>souligné</u> pour une ligne · cliquez-le pour une recherche explicative dans votre langue (Google · Baidu · Yahoo Japan · Yandex).
tips.legend = Explication à la demande :

## ── دستور الواجهة — القواعد الخمس ────────────────────────────
trust.1d = Nous disons ce qui est arrivé, pas la technique. Une phrase.
trust.1t = Clarté
trust.2d = Deux boutons au plus. Une option principale. Zéro réglage requis.
trust.2t = Simplicité
trust.3d = Aucun événement = aucune fenêtre. ≤ 3 alertes par jour. Jamais de modal.
trust.3t = Calme
trust.4d = La conséquence de chaque bouton s'écrit avant, pas après. Et le chemin sûr est le visible.
trust.4t = Connaître les conséquences
trust.sub = Chaque écran se mesure à ces quatre. En cas de conflit : la sécurité d'abord.

## ── USB ────────────────────────────
usb.stage = Phase

## ── Chiffres ─────────────────────────────
num.title = Chiffres
num.sub = Les nombres des compteurs, des alertes et du journal.
num.body = Choisissez la forme que vous lisez le plus vite. Tout calcul reste le même.
num.arabic = 0 1 2 3 4 5 6 7 8 9
num.hindi = ٠ ١ ٢ ٣ ٤ ٥ ٦ ٧ ٨ ٩
num.arabic.name = Chiffres arabes
num.hindi.name = Chiffres arabes orientaux
num.sample = 3 alertes · 12 fichiers
num.note = Le changement est immédiat. Aucun redémarrage nécessaire.

## ── Conseils de pause — à la demande ─────────────────────────────
tip.lvl1 = Ni fenêtre, ni avis, ni son, mais l’analyse et le blocage continuent en arrière-plan : un fichier infecté est isolé en silence et montré à votre retour.
tip.lvl2 = Ni analyse, ni blocage, ni alerte. Le service tourne mais ne fait rien, et votre appareil est sans protection.
tip.no_permanent = La plus longue pause finit au redémarrage, et c’est la seule limite que nous posons.

## ── Règle quatre : le visuel d’abord ─────────────────────────────
visual.title = Le visuel d’abord
visual.rule = L’image avant la phrase : ce que fait un écran se voit, ne s’explique pas.
visual.why = Qui lit lentement, ou ne lit pas votre langue, comprend la forme et la couleur avant la ligne.
visual.n1 = Chaque élément porte forme, couleur et place, et ne tient jamais à un mot.
visual.n2 = La même forme veut dire la même chose sur chaque écran.
visual.n3 = La couleur sépare le sûr du dangereux ; le texte nomme la raison.
visual.n4 = Une icône sans mot est une épreuve : s’il faut l’expliquer, c’est la mauvaise icône.
visual.n5 = Le texte explique l’image, il ne la répète pas.
visual.sample = Survolez l’icône : un mot. Cliquez : l’explication entière dans votre langue.
trust.count = Cinq, pas quatre.
help.sample.title = Un exemple réel
