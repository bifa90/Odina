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
