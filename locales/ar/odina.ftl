# عدينة — العربية (ar) — لغة الفريق
# الصيغة: Fluent (.ftl). لا تُغيَّر أسماء المفاتيح دون تحديث i18n-lint.
# قواعد: لا دمج سلاسل (3.9.4)، لا نص مضمَّن في الكود (18.2.9)، ولا ترجمة آلية للنصوص الأمنية (3.9.1).
# ملاحظة RTL: الاتجاه يقلب التخطيط لا المنطق (§8.2).

## ── هوية التطبيق ──────────────────────────────────────────
app.name = عدينة
app.tagline = حارسك الرقمي
app.window.title = عدينة — { $state }

## ── الحالات (الدرع) ───────────────────────────────────────
state.protected = أنت محمي
state.scanning = جارٍ الفحص…
state.attention = يحتاج انتباهك
state.paused = الحماية متوقفة مؤقتًا
state.untrusted = لم يُتحقق منه بعد

## ── الشاشة الرئيسية — جدران الحماية ───────────────────────
wall.signatures.title = قاعدة بيانات الملفات
wall.signatures.on = فعّال
wall.signatures.off = متوقف
wall.behavior.title = الفحص السلوكي العميق
wall.behavior.on = فعّال
wall.behavior.off = خامل
wall.proactive.title = الحماية الاستباقية
wall.proactive.on = فعّال
wall.proactive.off = متوقف

## ── الشاشة الرئيسية — الإجراءات (٨ عناصر كحد أقصى) ────────
action.scan_now = افحص الآن
action.pause = إيقاف مؤقت
action.expert_mode = وضع الخبير
action.language = اللغة
action.settings = الإعدادات
action.why = لماذا؟

## ── الفحص ─────────────────────────────────────────────────
scan.quick = فحص سريع
scan.full = فحص كامل
scan.usb = افحص هذا القرص
scan.running = جارٍ فحص { $target }…
scan.percent = { $percent }٪
scan.files_checked = الملفات المفحوصة
scan.threats_found = التهديدات المكتشفة
scan.stop = إيقاف
scan.complete = انتهى الفحص
scan.clean_result = لم يُعثر على شيء ضار.

## ── التنبيهات — مؤكد (لا يختفي تلقائيًا) ──────────────────
alert.confirmed.title = أوقفنا تهديدًا معروفًا
alert.confirmed.body = { $file } تهديد مؤكد وتم عزله — لم يُحذف.

## ── التنبيهات — مشتبه بشدة ────────────────────────────────
alert.suspicious.title = برنامج يحاول الوصول إلى ملفاتك
alert.suspicious.body = { $file } يعدّل ملفات في المستندات — أوقفناه مؤقتًا.
# قاعدة 3.9.2: «أوقفناه مؤقتًا» لا تُحذف ولا تُلطَّف في أي ترجمة.
alert.suspicious.note = أوقفناه مؤقتًا.

## ── التنبيهات — مشتبه ─────────────────────────────────────
alert.unsure.title = هذا البرنامج غير مألوف
alert.unsure.body = { $file } جديد هنا ويتصرف بغرابة.

## ── التنبيهات — أداة تخريب (خطر متوسط) ────────────────────
alert.pua.title = يبدو أنها أداة كسر حماية
alert.pua.body = { $file } يتجاوز حماية ترخيص برنامج مدفوع.
alert.pua.allowed = اسمح به
alert.pua.quarantine = انقله جانبًا

## ── أزرار التنبيه (زرّان فقط + «لماذا؟») ──────────────────
button.allow = السماح
button.block = المنع
button.restore = استرجاع
button.remove_forever = حذف نهائي

## ── USB ───────────────────────────────────────────────────
usb.detected = قرص جديد — نتحقق منه أولًا
usb.scanning = جارٍ فحص { $name }…
usb.priority_done = تم فحص الملفات الخطرة
usb.trust.new = قرص جديد
usb.trust.scanning = جارٍ الفحص
usb.trust.trusted = معروف لدينا
usb.trust.suspicious = تحت المراقبة
usb.trust.quarantined = محجوب

## ── العزل ─────────────────────────────────────────────────
quarantine.title = العناصر المعزولة
quarantine.reason = سبب العزل
quarantine.restore_note = المسار الأصلي محفوظ.
quarantine.retention = يُحفظ { $days } يومًا

## ── الإيقاف المؤقت ───────────────────────────────────────
pause.title = إيقاف مؤقت
pause.duration.10 = ١٠ دقائق
pause.duration.30 = ٣٠ دقيقة
pause.duration.60 = ساعة
pause.warning = سنوقف التنبيهات حتى { $until }. ما زلنا نحميك.
pause.confirm = تأكيد
pause.badge = التنبيهات متوقفة مؤقتًا · الحماية تعمل

## ── الحالة والأخطاء ───────────────────────────────────────
error.generic = حدث خطأ. لم نُغيّر شيئًا.
error.tamper = تم تعديل ملفات عدينة نفسها. يلزم الإصلاح.
error.offline = لا تحديث بعد. الحماية تعمل.

## ── التحديثات ─────────────────────────────────────────────
update.available = يتوفر تحديث
update.ask = هل نفحص التحديثات وننزّلها؟
update.db_age_warning = قاعدة التهديدات عمرها { $days } يومًا.

## ── المسار (يُعرض دائمًا LTR معزول) ───────────────────────
path.label = المسار
path.copy = نسخ المسار
path.open_folder = فتح المجلد

## ── أيقونات وأسس الثقة ────────────────────────────────────
trust.title = أسس الثقة
word.isolated = معزول
word.temporary = مؤقتًا

## ── الشرح عند الطلب ───────────────────────────────────────
tip.quarantine = نقلناه إلى مخزن مشفّر — مكانه الأصلي محفوظ ويمكن استرجاعه.
tip.suspended = أوقفناه الآن. سيعود للعمل إذا سمحت له.

## ── العزل — الأزرار (هنا فقط يوجد «حذف») ─────────────────
quarantine.hint = كل شيء يبقى حتى تقرر أنت · لا حذف تلقائي أبدًا
quarantine.keep = احتفظ به
quarantine.delete = حذف نهائي
quarantine.delete_note = لا يمكن التراجع. لا نسخة أخرى.
quarantine.cleanup_title = منذ { $days } يومًا وهذه الملفات في العزل
quarantine.cleanup_choose = حذف ما اخترته

## ── الاعتذار ──────────────────────────────────────────────
error.false_positive = أخطأنا في هذا الملف — أصلحناه.
error.locked = لم نستطع عزل الملف لأنه مفتوح الآن.

## ── Turn protection off (user's explicit decision) ────────
pause.confirm_disable = أوقف الحماية لمدة { $minutes } دقيقة
disable.title = إيقاف الحماية
disable.warning_title = خطر شديد
disable.body = لن نتوقف عن التنبيه — سنتوقف عن الحماية. لن يُفحص أي ملف، ولن يُمنع أي تهديد. جهازك يصبح غير محمي تمامًا.
disable.responsibility = القرار قرارك، والمسؤولية عليك أثناء هذه المدة.
disable.choose = اختر المدة
disable.duration.peak = حتى نهاية ساعة الذروة
disable.duration.30 = ٣٠ دقيقة
disable.duration.60 = ساعة
disable.duration.until_reboot = حتى إعادة التشغيل
disable.refused = لا يمكن إيقاف الحماية الآن
disable.refused_quarantine = يوجد ملف معزول لم يُحسم بعد. راجعه أولًا.
disable.refused_scanning = هناك فحص جارٍ الآن. انتظره أو أوقفه أولًا.
disable.refused_active_threat = يوجد تهديد لم يُعالَج بعد. لا نُوقف الحماية وتهديدها مفتوح.
state.unprotected = الحماية موقوفة
state.unprotected_until = الحماية موقوفة حتى { $until }
state.unprotected_badge = غير محمي — بقرارك
action.turn_off = إيقاف الحماية
action.turn_on = أعد الحماية
disable.re_enable_note = تستطيع إعادة الحماية في أي لحظة — بلا كلمة مرور.
disable.reminder = لديك { $minutes } دقيقة متبقية على إعادة الحماية تلقائيًا.
pause.resume_report = أثناء الإسكات: { $count } ملفًا عُزل

## ── الاستثناء الجراحي (D45 · §4.8) ─────────────────────────
exception.title = الاستثناءات
exception.sub = إن احتاج عملك قاعدة تخالف الأصل، اسمح بها وحدها. الباقي يبقى كما هو.
exception.rule.a1.wsh = تشغيل سكربتات Windows
exception.rule.a1.wsh.desc = ملفات .vbs و .js تعمل مقيّدة ما لم تكن موقّعة من ناشر معروف.
exception.rule.a1.powershell = أوامر PowerShell
exception.rule.a1.powershell.desc = PowerShell يعمل في الوضع المقيّد. استثناؤه لا يفتح Windows Script Host.
exception.rule.a1.macros = ماكرو Office
exception.rule.a1.macros.desc = المستندات غير الموقّعة لا تُشغّل ماكرو. استثناؤه لا يمسّ السكربتات.
exception.rule.a3.downloads = اشتباه مجلد التنزيلات
exception.rule.a3.downloads.desc = كل ملف جديد في التنزيلات يُعدّ مشتبهًا حتى يُثبت العكس.
exception.rule.a4.periodic = الفحص الدوري
exception.rule.a4.periodic.desc = فحص صامت كل ساعتين لملفات الإقلاع والمفاتيح الحساسة.
exception.rule.a6.doubt = الحذر عند الشك
exception.rule.a6.doubt.desc = عند الشك يمنع أولًا ثم يسأل. يُرخّى فقط، ولا يُسمح به كاملًا.
exception.state.on = فعّالة
exception.state.relaxed_until = مُرخّاة حتى { $until }
exception.state.allowed_until = مسموحة حتى { $until }
exception.state.allowed_permanent = مسموحة دائمًا — بقرارك
exception.consequence.relax = ستعمل { $rule } بلا تنبيه حتى { $until } ثم تعود. لن يُمنع شيء في هذه المدة.
exception.consequence.allow = ستعمل { $rule } بلا قيد حتى { $until } ثم تعود. الباقي لا يتغيّر.
exception.consequence.permanent = ستبقى { $rule } مسموحة حتى تبطلها. سيبقى هذا مكتوبًا في الرئيسية.
exception.action.relax_10 = ‏رخّها ١٠ دقائق
exception.action.relax_30 = ‏رخّها ٣٠ دقيقة
exception.action.relax_60 = ‏رخّها ساعة
exception.action.allow = اسمح
exception.action.revoke = أبطِل
exception.action.revoke_all = أبطِل كل الاستثناءات
exception.action.advanced = خيارات متقدمة
exception.badge = { $rule } مسموحة · حتى { $until }
exception.badge_permanent = { $rule } مسموحة دائمًا · بقرارك
exception.badge_many = { $count } قواعد مسموحة بقرارك · التفاصيل
exception.until_reboot = حتى إعادة التشغيل
exception.permanent_note = استثناء دائم لا ينتهي وحده. سيبقى ظاهرًا في الرئيسية حتى تبطلته.
exception.revoke_note = تستطيع الإبطال في أي لحظة — بضغطة واحدة، بلا كلمة مرور.
exception.expert_note = الوضع الخبير: يفتح «دائم» و«القاعدة كاملة». لا كلمة مرور، ولا سؤال عن السبب.
exception.expired_report = انتهى الاستثناء. أثناءه: { $count } ملفًا اشتبه فيه، راجعه الآن.
exception.expired_clean = انتهى الاستثناء. لم يظهر ما يستحق مراجعة.
exception.refused.not_allowed = هذه القاعدة لا تُسمح بالكامل — تُرخّى فقط.
exception.refused.above_ceiling = أقصى ما تسمح به هذه القاعدة هو الإرخاء.
exception.refused.no_permanent = هذه القاعدة لا تُستثنى بشكل دائم.
exception.refused.expert_only = الاستثناء الدائم الكامل يحتاج وضع الخبير.
exception.refused.veto_stands = الملف مؤكد. الاستثناء يُرخي القاعدة ولا يُبرئ ملفًا.

exception.live_note = بقرارك · الباقي فعّال كما هو · تستطيع الإبطال في أي لحظة
exception.lines_title = أربعة لا تُستثنى:
exception.lines_body = حكم الجدار 1 (مؤكد يُعزل حتى تحت الاستثناء) · حماية العبث · القاعدة الثانية (استثناء WSH لا يفتح PowerShell) · «مشتبه بشدة» وأعلى. والتراجع بضغطة واحدة، بلا كلمة مرور، دائمًا.

## ── خريطة البرنامج والتنقل (الدستور §6أ.3) ─────────────────
nav.home = الرئيسية
nav.quarantine = صندوق العزل
nav.proactive = الحماية الاستباقية
nav.alerts = مركز التنبيهات
nav.settings = الإعدادات
nav.language = اللهجة واللغة
nav.privacy = الخصوصية
back = رجوع
menu.more = المزيد
menu.advanced = خيارات متقدمة
map.title = خريطة البرنامج
map.hint = من أين تُفتح كل شاشة · ثلاث طبقات كحد أقصى

## ── سياسة توقيع الشيفرة (صفحة عامة) ────────────────────────
signing.title = سياسة توقيع الشيفرة
signing.sub = ما نُوقّعه، وما لا نُوقّعه، وأين يُحفظ المفتاح، وكيف تتحقق بنفسك.
signing.status_title = الحالة اليوم
signing.status_body = هذه السياسة منشورة قبل بدء التوقيع، ولم يُوقَّع أي ملف بعد. أول ملف موقّع سيُعلن في صفحة الإصدار مع بصمته.
signing.status_until = وحتى ذلك الحين يُوزَّع البرنامج كأرشيف ZIP محمول، لا كمثبّت بلا توقيع.
signing.build_title = البناء
signing.build_body = كل توقيع يحدث على عدّاد GitHub Actions من الالتزام المُعلَن في سجل التغييرات. لا بناء محلي، ولا إعادة بناء قبل التوقيع.
signing.key_title = المفتاح
signing.key_body = لا نحتفظ بأي مفتاح توقيع. المفتاح في وحدة أمنية معزولة لدى SignPath Foundation، ولكل توقيع موافقة مشرف صريحة.
signing.signed_title = ما يُوقَّع
signing.signed_installer = المثبّت
signing.signed_store = حزمة المتجر
signing.unsigned_title = ما لا يُوقَّع
signing.unsigned_db = قواعد البصمات — تُوقَّع بمفتاح Ed25519 منفصل، لأنها تُحدَّث أكثر من البرنامج.
signing.unsigned_tools = الأدوات في مجلد tools — تُوزَّع بترخيص MIT كنص مصدري، لا كملفات تنفيذية.
signing.verify_title = التحقق
signing.verify_body = بصمة SHA-256 لكل ملف تُنشر في صفحة الإصدار قبل التوقيع وبعده. وتستطيع التحقق بنفسك على ويندوز:
signing.verify_must = يجب أن تكون الحالة Valid، وأن يكون الناشر SignPath Foundation. وإن اختلفت البصمة عن المنشورة، فلا تشغّل الملف.
signing.revoke_title = الإلغاء
signing.revoke_body = إن بُني إصدار من شيفرة غير مُراجَعة، نُلغيه ونُعلن الإلغاء قبل أي شيء آخر.
signing.credit = التوقيع المجاني مقدَّم من SignPath.io، والشهادة من SignPath Foundation.
signing.updated = آخر تحديث

## ── المفردات — سطر واحد لكل مصطلح (glossary-v1) ─────────────
tip.confirmed = بصمته موجودة في قاعدة التهديدات. ليس تخمينًا.
tip.high_risk = علامات قوية على خطر، ولم تتأكد البصمة بعد.
tip.suspicious = يبدو غريبًا: الاسم أو المكان أو السلوك. لم نتحقق.
tip.pua = ليس فيروسًا، لكنه يتجاوز حماية برنامج آخر.
tip.signature = بصمات ملفات معروفة. تحديثها يجعل الحماية أحدث.
tip.behavior = يراقب ما يفعله البرنامج، لا اسمه.
tip.proactive = تمنع الفعل قبل وقوعه بدل أن تكتشفه بعده.
tip.pause = نُسكت النوافذ فقط. الفحص والمنع يستمران.
tip.turn_off = لا فحص ولا منع. جهازك غير محمي أثناء ذلك.
tip.exception = كل استثناء يخصّ قاعدة واحدة. تبقى بقية القواعد كما هي.
tip.expert_mode = يفتح الخيارات الدائمة والكاملة. لا كلمة مرور، ولا سؤال.
tip.wsh = مشغّل سكربتات ويندوز. باب شائع للبرمجيات الخبيثة.
tip.powershell = أداة أوامر في ويندوز. مقيّدة افتراضيًا عند المصادر الخطرة.
tip.macros = أوامر داخل مستند Office. لا تعمل في مستند غير موقّع.
tip.amsi = واجهة ويندوز تمكّننا من رؤية السكربت قبل تنفيذه.
tip.wdac = سياسة ويندوز تمنع تشغيل ملف بعينه.
tip.usb_trust = أول مرة نراه. يُفحص قبل أن يُفتح.
tip.tamper = لا يمكن إطفاء Odina بتعديل ملفاته أو قتل خدمته.
tip.hash = سلسلة تمثّل الملف كاملًا. تغيّر بايت واحد يغيّرها كلها.
tip.heuristic = علامة تُقاس وتُوزن، وحدها لا تكفي للحكم.

## ── الإعدادات والخصوصية واللغة ─────────────────────────────
settings.title = الإعدادات
settings.sub = أربعة أقسام. لا شيء مخفي وراء «متقدم» إلا ما ندر.
settings.appearance = المظهر
settings.appearance.body = فاتح أو داكن. يتبع النظام افتراضيًا.
settings.theme_light = فاتح
settings.theme_dark = داكن
settings.theme_system = حسب النظام
settings.behavior = السلوك
settings.startup = يبدأ مع ويندوز
settings.startup.body = يعمل في الخلفية منذ الإقلاع. يمكنك إيقافه.
settings.notifications = الإشعارات
settings.notifications.body = ثلاثة في اليوم كحد أقصى. الأهم أولًا.
settings.updates = التحديثات
settings.updates.body = تلقائي لقاعدة التهديدات، ويدوي للبرنامج.
settings.about = عن البرنامج
settings.version = النسخة
settings.database = قاعدة التهديدات
settings.database.date = آخر تحديث

privacy.title = الخصوصية
privacy.sub = ما يخرج من جهازك، وما لا يخرج. بلا استثناءات.
privacy.telemetry = صفر تيليمتري
privacy.telemetry.body = لا إحصاء، لا معرّف جهاز، لا بصمة. لا شيء يُرسل تلقائيًا.
privacy.local = كل شيء محلي
privacy.local.body = الفحص والقرار والسجل على قرصك.
privacy.report = التقرير بقرارك
privacy.report.body = لا يُرسل تقرير إلا إذا اخترت أنت إرساله.
privacy.search = البحث عند الشرح
privacy.search.body = يفتح محرك البحث بلغتك. نُرسل المصطلح فقط، لا ملفاتك.
privacy.log = السجل
privacy.log.body = محلي، ويُقرأ بلغتك. تستطيع مسحه متى شئت.
privacy.log_clear = امسح السجل

lang.title = اللهجة واللغة
lang.sub = عشرة ألسنة. الترجمة تراها فورًا، والخط يتبع لغتك.
lang.current = اللغة الحالية
lang.rtl = يُكتب من اليمين إلى اليسار
lang.ltr = يُكتب من اليسار إلى اليمين
lang.review = مراجعة ناطق أصلي
lang.review.body = اللغات التي لم يراجعها ناطق أصلي تُعلَّق، لا تُشحن.

home.walls = الجدران الثلاثة
home.action.scan = افحص الآن
home.action.pause = إيقاف مؤقت
home.action.expert = وضع الخبير
home.action.why = لماذا؟
home.goto.quarantine = صندوق العزل
home.goto.alerts = مركز التنبيهات
home.goto.settings = الإعدادات
home.goto.proactive = الحماية الاستباقية
home.quarantine_count = { $count } ملفًا في العزل
home.alerts_count = { $count } تنبيهًا يستحق النظر
home.proactive_summary = تسعة أنشطة في ثلاث فئات
proactive.title = الحماية الاستباقية
proactive.sub = السماح بنشاط هنا لا يلغي فحص الملفات أو قواعد جدار الحماية.
proactive.tier.mandatory = خطورة شديدة · منع تلقائي
proactive.tier.recommended = خطورة متوسطة · يُنصح بالمنع
proactive.tier.optional = خطورة أقل · منع اختياري
proactive.surface.office_child = مفسّرات الأوامر من Office
proactive.surface.office_child.tip = بدء cmd أو PowerShell أو mshta من Word أو Excel، وليس كل عمليات Office الفرعية.
proactive.surface.wmi_subscription = اشتراكات WMI الدائمة
proactive.surface.wmi_subscription.tip = إنشاء اشتراك أحداث دائم من مصدر غير موثوق، لا استعلامات WMI العادية.
proactive.surface.system_tools = أدوات النظام مع محتوى غير موثوق
proactive.surface.system_tools.tip = استخدام mshta أو certutil أو regsvr32 مع مدخل غير موثوق من مجلد مؤقت أو الإنترنت.
proactive.surface.scheduled_task = المهام المجدولة
proactive.surface.scheduled_task.tip = إنشاء مهمة أو تعديلها؛ لا تتوقف المهام القائمة بمجرد اختيار المنع.
proactive.surface.com_registration = تسجيل مكوّنات COM
proactive.surface.com_registration.tip = تغيير تسجيل مكوّن COM للمستخدم الحالي، لا تعطيل COM في ويندوز.
proactive.surface.registry_autorun = مفاتيح بدء التشغيل
proactive.surface.registry_autorun.tip = إضافة إدخال إلى Run أو RunOnce في السجل أو تغييره.
proactive.surface.service_driver = تثبيت الخدمات والتعريفات
proactive.surface.service_driver.tip = إنشاء خدمة أو تثبيت تعريف جديد؛ لا تُوقَف الخدمات والتعريفات العاملة.
proactive.surface.inbound_smb_rdp = اتصالات SMB وRDP الواردة
proactive.surface.inbound_smb_rdp.tip = اتصالات واردة جديدة للمشاركة أو سطح المكتب البعيد؛ السماح لا يفتح منفذًا.
proactive.surface.wsh_script = سكربتات ويندوز
proactive.surface.wsh_script.tip = تشغيل ملفات ‎.js و‎.vbs و‎.wsf عبر WSH. تبقى فحوص السكربت سارية.
proactive.state.blocked = ممنوع
proactive.state.allowed = مسموح
proactive.state.contained = منع مؤقت
proactive.state.off = غير مطبّق
proactive.saved_until = { $rule } · الاستثناء محفوظ حتى { $until }
proactive.user_wsh_block = منع سكربتات ويندوز الذي اخترته مقدَّم على هذا الاستثناء.
proactive.demo.title = محاكاة الاحتواء
proactive.demo.note = هذه تجربة محلية. لا تُفحص ملفاتك ولا تتغيّر إعدادات ويندوز.
proactive.demo.mode = وضع الحماية في التجربة
proactive.demo.ready = عند فحص إصابة مؤكدة، تُمنع الأنشطة التسعة حتى ينجح الإصلاح.
proactive.demo.start = حاكِ إصابة مؤكدة
proactive.demo.scan = فحص إصابة مؤكدة · الأنشطة التسعة ممنوعة مؤقتًا.
proactive.demo.clean = جارٍ التنظيف · اكتمل الفحص وما زال المنع قائمًا.
proactive.demo.repair = جارٍ الإصلاح · اكتمل التنظيف وما زال المنع قائمًا.
proactive.demo.cleaned = اكتمل التنظيف
proactive.demo.repaired = اكتمل الإصلاح
proactive.demo.restored = نجح الإصلاح. رُفع المنع المؤقت وعادت اختياراتك السابقة.
proactive.demo.off = الحماية موقوفة بقرارك. لا يبدأ الاحتواء ولا تتغيّر اختياراتك.
proactive.demo.fail = تعذّرت الخطوة
proactive.demo.failed = لم تنجح الخطوة. يبقى المنع حتى تنجح المعالجة.
alerts.title = مركز التنبيهات
alerts.sub = كل ما يستحق النظر في مكان واحد. لا نوافذ منبثقة.
alerts.empty = لا شيء يستحق النظر الآن.
exception.detail.title = الاستثناء — قاعدة واحدة
exception.detail.sub = ماذا ستغيّر، وإلى متى، وكيف تعود.
map.node.home = الرئيسية
map.node.quarantine = صندوق العزل
map.node.proactive = الحماية الاستاقية
map.edge.from_home = من الرئيسية
map.edge.from_alert = من التنبيه
map.legend = الخط المتصل مدخل دائم · والمنقّط مدخل من سياق

## ── الشرح عند الطلب ─────────────────────────────
help.title = الشرح عند الطلب
help.hint = سطر واحد عند الوقوف · وبحث كامل عند الضغط، بلغتك.
help.tip.title = تلميح فوري
help.tip.body = كلمة واحدة مسطّرة، وسطر واحد لا يتجاوز ٩٠ حرفًا.
help.guide.title = دليل أطول
help.guide.body = يُفتح بزر داخل التلميح، ويُقرأ في محرك تعرفه.
help.guide.cta = افتح البحث
help.engine.title = المحرك يتبع اللغة والموقع
help.col.lang = اللغة
help.col.engine = المحرك
help.col.why = السبب
help.why.default = الأكثر تغطية لهذه اللغة
help.why.zh = المرجع داخل الصين، وغوغل محدود هناك
help.why.ja = يُقدّم صفحات يابانية أولًا، لا مترجمة
help.why.ru = يفهم الصرف الروسي أفضل من غيره
help.query.note = نُرسل المصطلح وحده. لا اسم ملف، ولا مسار، ولا معرّف جهاز.
path.copy_hint = نسخ
nav.preview = معاينة الواجهة

## ── التنبيهات والشارات ────────────────────────────
alert.confirmed.cta = فتح صندوق العزل
badge.confirmed = مؤكد
badge.high = مشتبه بشدة
badge.low = مشتبه
badge.pua = خطر متوسط
button.off = متوقف
button.on = مفعّل

## ── الإيقاف المؤقت ────────────────────────────
lvl.app = التطبيق
lvl.d10 = { $n } دقائق
lvl.d30 = { $n } دقيقة
lvl.d60 = ساعة
lvl.dpeak = حتى نهاية ساعة الذروة
lvl.dreboot = حتى إعادة التشغيل
lvl.hint = نُخبرك، والقرار لك.
lvl.no_permanent = لا إيقاف دائم. أقصاه إعادة تشغيل.
lvl.state_off = الحماية موقوفة
lvl.state_off_sub = حتى 14:30 · بقرارك
lvl.title = الإيقاف — مستويان لا مستوى واحد
lvl.what_continues = ما يستمر
lvl.what_stops = ما يتوقف
lvl1.app = يعمل · يفحص · يمنع · لا يسأل
lvl1.body = لا نافذة ولا صوت. الفحص والمنع مستمران.
lvl1.cont = الجدران الثلاثة
lvl1.name = إسكات التنبيهات
lvl1.stops = التنبيهات فقط
lvl2.body = لا فحص ولا منع. جهازك غير محمي.
lvl2.name = إيقاف الحماية
lvl2.resp = المسؤولية عليك أثناء المدة.
lvl2.warn = نتوقف عن الحماية، لا عن الإخبار.

## ── دستور الواجهة — القواعد الخمس ────────────────────────────
notes.1b = يغيّر الاتجاه والتخطيط فورًا، والأرقام تبقى لاتينية في النسب والبصمات.
notes.1t = تبديل اللغة:
notes.2b = «السماح» و«المنع» بترتيب منطقي ثابت؛ RTL يقلب الموضع البصري لا الدور.
notes.2t = الأزرار:
notes.3b = تُعرض هنا بخط النظام. بعد تنزيل حزمة CDN تتحول إلى Noto بلا إعادة تشغيل.
notes.3t = الآسيوية:
notes.4b = هذا الملف يحمّل الخطوط من مجلد fonts/vendor/ المجاور — إن لم تظهر فافتحه عبر سيرفر محلي لا بـ file://.
notes.4t = الخطوط:
notes.5b = الـlogotype، وأيقونة التطبيق داخل شريط المهام، وحالات الخطأ — تُراجع منفصلة (R3).
notes.5t = ما ينقص هذه المعاينة:

## ── صندوق العزل ────────────────────────────
q.reason1 = تهديد مؤكد
q.reason2 = أداة كسر حماية
quarantine.opened_from = يُفتح من التنبيه، ومن العدّاد في الرئيسية — لا من قائمة مخفية.
quarantine.restore = استرجع

## ── دستور الواجهة — القواعد الخمس ────────────────────────────
sec.alerts = 2 · التنبيهات الأربعة
sec.alerts.hint = زرّان + «لماذا؟» · اللون لا ينقل المعلومة وحده
sec.color = 4 · اللوحة اللونية (مقاسة WCAG)
sec.color.hint = كل قيمة تحمل نسبة تباينها مقابل الخلفية الحالية
sec.fonts = 5 · الخطوط لكل لغة
sec.fonts.hint = 400 و600 فقط · خطوط النظام بديلًا فوريًا للآسيوية
sec.main = 1 · الشاشة الرئيسية
sec.main.hint = ≤ 8 عناصر · 3 جدران · زرّان · لا مصطلحات تقنية
sec.notes = 6 · ملاحظات للمراجعة
sec.usb = 3 · تنبيه USB
sec.usb.hint = شريط تقدم واحد + مرحلة واحدة · RTL-aware

## ── الهوية ────────────────────────────
shield.sub = { $n } جدران فعّالة · قاعدة التهديدات محدّثة

## ── الشرح عند الطلب ────────────────────────────
tips.body = مرّر المؤشر فوق الكلمة المسطّرة لترى سطرًا واحدًا · اضغطها ليفتح بحثًا شارحًا بلغتك (Google · Baidu · Yahoo Japan · Yandex).
tips.legend = الشرح عند الطلب:

## ── دستور الواجهة — القواعد الخمس ────────────────────────────
trust.1d = نقول ما حدث، لا ما هو التقنية. جملة واحدة. ولا «تمّ» بلا سبب.
trust.1t = الوضوح
trust.2d = زرّان كحد أقصى. خيار واحد أساسي. صفر إعداد مطلوب.
trust.2t = البساطة
trust.3d = بلا حدث = بلا نافذة. ≤ 3 إشعارات يوميًا. لا Modal أبدًا.
trust.3t = الطمأنينة
trust.4d = عاقبة كل زر مكتوبة قبله لا بعده. والخيار الآمن هو الظاهر.
trust.4t = معرفة العواقب
trust.sub = كل شاشة تُقاس على هذه الأربعة. وعند التعارض: الأمان أولًا.

## ── USB ────────────────────────────
usb.stage = P0 · الملفات الخطرة في الجذر

## ── الأرقام ─────────────────────────────
num.title = الأرقام
num.sub = الأرقام في العدّاد والتنبيهات والسجل.
num.body = اختر الشكل الذي تقرأه أسرع. كل الحسابات واحدة.
num.arabic = 0 1 2 3 4 5 6 7 8 9
num.hindi = ٠ ١ ٢ ٣ ٤ ٥ ٦ ٧ ٨ ٩
num.arabic.name = أرقام عربية
num.hindi.name = أرقام هندية
num.sample = ٣ تنبيهات · ١٢ ملفًا
num.note = التغيير فوري. لا يحتاج إعادة تشغيل.

## ── تلميحات الإيقاف — عند الطلب ─────────────────────────────
tip.lvl1 = لا نافذة ولا إشعار ولا صوت، لكن الفحص والمنع يستمران في الخلفية: أي ملف مصاب يُعزل بصمت ويُعرض عليك عند العودة.
tip.lvl2 = لا فحص ولا منع ولا تنبيه. الخدمة تعمل لكنها خاملة، وجهازك غير محمي تمامًا.
tip.no_permanent = أقصى إيقاف ينتهي بإعادة التشغيل، وهذا الحدّ الوحيد الذي نفرضه.

## ── القاعدة الرابعة: التواصل البصري ─────────────────────────────
visual.title = التواصل البصري أولًا
visual.rule = الصورة قبل الجملة: ما تفعله الشاشة يُرى، ولا يُشرح.
visual.why = من يقرأ ببطء، أو لا يقرأ لغتك، يفهم الرمزَ واللون قبل السطر.
visual.n1 = كل عنصر مرئي يحمل الشكل واللون والموضع، فلا يعتمد على كلمة.
visual.n2 = الشكل نفسه يعني الشيء نفسه في كل شاشة.
visual.n3 = اللون يفرّق بين آمن وخطر، والنص يسمّي السبب.
visual.n4 = الرمز بلا كلمة يُختبر: إن احتاج شرحًا فهو رمز خاطئ.
visual.n5 = النص يشرح الصورة، لا يكرّرها.
visual.sample = وقوف على الرمز: كلمة واحدة. ضغط عليه: شرح كامل بلغتك.
trust.count = خمسة، لا أربعة.
help.sample.title = مثال حيّ
