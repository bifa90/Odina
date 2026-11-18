# Odina Antivirus — हिन्दी (hi)
# المستوى: T-B (مجتمعية) — تحتاج مراجعة ناطق أصلي قبل ترقيتها إلى T-A (§30.2)
# راجع كل سطر يدويًا قبل الاعتماد. لا ترجمة آلية للنصوص الأمنية (§3.9.1).

app.name = Odina
app.tagline = आपका डिजिटल रक्षक
app.window.title = Odina — { $state }
state.protected = आप सुरक्षित हैं
state.scanning = जाँच हो रही है…
state.attention = आपके ध्यान की ज़रूरत है
state.paused = सुरक्षा रोकी गई
state.untrusted = अभी जाँचा नहीं गया
wall.signatures.title = फ़ाइल डेटाबेस
wall.signatures.on = सक्रिय
wall.signatures.off = निष्क्रिय
wall.behavior.title = गहरी व्यवहार जाँच
wall.behavior.on = सुन रहा है
wall.behavior.off = निष्क्रिय
wall.proactive.title = पूर्वानुमानित सुरक्षा
wall.proactive.on = सक्रिय
wall.proactive.off = बंद
action.scan_now = अभी जाँचें
action.pause = रोकें
action.expert_mode = विशेषज्ञ मोड
action.language = भाषा
action.settings = सेटिंग्स
action.why = क्यों?
scan.quick = तेज़ जाँच
scan.full = पूरी जाँच
scan.usb = इस ड्राइव को जाँचें
scan.running = { $target } की जाँच हो रही है…
scan.percent = { $percent }%
scan.files_checked = जाँची गई फ़ाइलें
scan.threats_found = मिले ख़तरे
scan.stop = रोकें
scan.complete = जाँच पूरी
scan.clean_result = कोई हानिकारक चीज़ नहीं मिली।
alert.confirmed.title = हमने एक ज्ञात ख़तरा रोका
alert.confirmed.body = { $file } पुष्ट ख़तरा है और अलग कर दिया गया है।
alert.suspicious.title = एक प्रोग्राम आपकी फ़ाइलों तक पहुँचने की कोशिश कर रहा है
alert.suspicious.body = { $file } दस्तावेज़ों में फ़ाइलें बदल रहा है। हमने इसे अस्थायी रूप से रोका।
alert.suspicious.note = हमने इसे अस्थायी रूप से रोका।
alert.unsure.title = यह प्रोग्राम असामान्य है
alert.unsure.body = { $file } यहाँ नया है और अजीब व्यवहार करता है। क्या आप इसे जानते हैं?
alert.pua.title = यह क्रैकिंग टूल जैसा लगता है
alert.pua.body = { $file } सशुल्क सॉफ़्टवेयर की लाइसेंस सुरक्षा को बायपास करता है। आप तय करते हैं कि आपके डिवाइस पर क्या चले।
alert.pua.allowed = फिर भी अनुमति दें
alert.pua.quarantine = अलग रखें
button.allow = अनुमति दें
button.block = रोकें
button.restore = वापस लाएँ
button.remove_forever = स्थायी रूप से हटाएँ
usb.detected = नई ड्राइव मिली — पहले इसे जाँचते हैं
usb.scanning = { $name } की जाँच हो रही है…
usb.priority_done = ख़तरनाक फ़ाइलें जाँची गईं
usb.trust.new = नई ड्राइव
usb.trust.scanning = जाँच हो रही है
usb.trust.trusted = याद रखी गई
usb.trust.suspicious = निगरानी में
usb.trust.quarantined = अवरुद्ध
quarantine.title = अलग किए गए आइटम
quarantine.reason = अलग क्यों किया गया
quarantine.restore_note = मूल स्थान सुरक्षित रखा गया है।
quarantine.retention = { $days } दिन तक रखा जाएगा
pause.title = सुरक्षा रोकें
pause.duration.10 = 10 मिनट
pause.duration.30 = 30 मिनट
pause.duration.60 = 1 घंटा
pause.warning = { $until } तक आप सुरक्षित नहीं रहेंगे।
pause.confirm = फिर भी रोकें
pause.badge = सूचनाएँ रुकी · सुरक्षा चालू
error.generic = कुछ गड़बड़ हुई। हमने कुछ नहीं बदला।
error.tamper = Odina की अपनी फ़ाइलें बदल दी गई हैं। मरम्मत ज़रूरी है।
error.offline = अभी कोई अपडेट नहीं। सुरक्षा चालू है।
update.available = अपडेट उपलब्ध है
update.ask = अपडेट जाँचें और डाउनलोड करें?
update.db_age_warning = आपकी ख़तरा सूची { $days } दिन पुरानी है।

## ── Path ──────────────────────────────────────────────
path.label = पथ
path.copy = पथ कॉपी करें
path.open_folder = फ़ोल्डर खोलें
trust.title = विश्वास की नींव
word.isolated = अलग किया
word.temporary = अस्थायी रूप से
tip.quarantine = एन्क्रिप्टेड भंडार में ले जाया गया — मूल स्थान सुरक्षित है।
tip.suspended = अभी रोका गया। अनुमति देने पर ही दोबारा चलेगा।
quarantine.hint = आपके निर्णय तक सब रहता है · कभी अपने आप नहीं हटता
quarantine.keep = रखें
quarantine.delete = स्थायी रूप से हटाएँ
quarantine.delete_note = वापस नहीं हो सकता। कोई दूसरी प्रतिलिपि नहीं।
quarantine.cleanup_title = { $days } दिन से अलग रखा
quarantine.cleanup_choose = जो मैंने चुने वे हटाएँ
error.false_positive = इस फ़ाइल में हमसे गलती हुई — ठीक कर दिया।
error.locked = अलग नहीं कर सके: फ़ाइल अभी खुली है।

## ── Turn protection off (user's explicit decision) ────────
pause.confirm_disable = { $minutes } मिनट के लिए सुरक्षा बंद करें
disable.title = सुरक्षा बंद करें
disable.warning_title = गंभीर जोखिम
disable.body = हम सिर्फ़ सूचनाएँ नहीं रोकेंगे — सुरक्षा रोक देंगे। कोई फ़ाइल जाँची नहीं जाएगी और कोई ख़तरा नहीं रुकेगा।
disable.responsibility = यह आपका निर्णय है और उस अवधि की ज़िम्मेदारी आपकी है।
disable.choose = अवधि चुनें
disable.duration.peak = व्यस्त घंटे के अंत तक
disable.duration.30 = 30 मिनट
disable.duration.60 = 1 घंटा
disable.duration.until_reboot = रीस्टार्ट तक
disable.refused = अभी सुरक्षा बंद नहीं की जा सकती
disable.refused_quarantine = एक अलग रखी फ़ाइल पर निर्णय बाकी है। पहले उसे देखें।
disable.refused_scanning = स्कैन चल रहा है। प्रतीक्षा करें या पहले रोकें।
disable.refused_active_threat = एक अनसुलझा ख़तरा अभी खुला है। उसके रहते सुरक्षा बंद नहीं करेंगे।
state.unprotected = सुरक्षा बंद है
state.unprotected_until = { $until } तक सुरक्षा बंद
state.unprotected_badge = असुरक्षित — आपका निर्णय
action.turn_off = सुरक्षा बंद करें
action.turn_on = सुरक्षा फिर चालू करें
disable.re_enable_note = आप कभी भी सुरक्षा फिर चालू कर सकते हैं — बिना पासवर्ड।
disable.reminder = { $minutes } मिनट बाद सुरक्षा अपने आप लौट आएगी।
pause.resume_report = सूचनाएँ रुकी रहने के दौरान: { $count } फ़ाइल अलग की गई

## ── सटीक अपवाद (D45 · §4.8) ─────────────────────────
exception.title = अपवाद
exception.sub = अगर काम के लिए एक नियम ढीला करना पड़े, तो सिर्फ़ वही अनुमति दें। बाकी सब वैसा ही रहेगा।
exception.rule.a1.wsh = Windows स्क्रिप्ट
exception.rule.a1.wsh.desc = ज्ञात प्रकाशक के हस्ताक्षर न हों तो .vbs और .js फ़ाइलें सीमित मोड में चलती हैं।
exception.rule.a1.powershell = PowerShell कमांड
exception.rule.a1.powershell.desc = PowerShell प्रतिबंधित मोड में चलता है। अनुमति देने से Windows Script Host नहीं खुलता।
exception.rule.a1.macros = Office मैक्रो
exception.rule.a1.macros.desc = अहस्ताक्षरित दस्तावेज़ मैक्रो नहीं चला सकते। अनुमति देने से स्क्रिप्ट पर असर नहीं पड़ता।
exception.rule.a3.downloads = डाउनलोड पर संदेह
exception.rule.a3.downloads.desc = डाउनलोड में हर नई फ़ाइल संदिग्ध मानी जाती है जब तक साफ़ न हो।
exception.rule.a4.periodic = समय-समय पर जाँच
exception.rule.a4.periodic.desc = हर दो घंटे में स्टार्टअप आइटम और संवेदनशील कुंजियों की शांत जाँच।
exception.rule.a6.doubt = संदेह में सावधानी
exception.rule.a6.doubt.desc = संदेह होने पर पहले रोकता है, फिर पूछता है। ढीला किया जा सकता है, पूरी अनुमति नहीं।
exception.state.on = सक्रिय
exception.state.relaxed_until = { $until } तक ढीला
exception.state.allowed_until = { $until } तक अनुमत
exception.state.allowed_permanent = स्थायी रूप से अनुमत — आपका निर्णय
exception.consequence.relax = { $rule } { $until } तक बिना सूचना चलेगा, फिर लौट आएगा। उस दौरान कुछ भी नहीं रोका जाएगा।
exception.consequence.allow = { $rule } { $until } तक बिना प्रतिबंध चलेगा, फिर लौट आएगा। बाकी कुछ नहीं बदलेगा।
exception.consequence.permanent = { $rule } तब तक अनुमत रहेगा जब तक आप रद्द न करें। यह होम स्क्रीन पर लिखा रहेगा।
exception.action.relax_10 = 10 मिनट ढीला करें
exception.action.relax_30 = 30 मिनट ढीला करें
exception.action.relax_60 = एक घंटा ढीला करें
exception.action.allow = अनुमति
exception.action.revoke = रद्द करें
exception.action.revoke_all = सभी अपवाद रद्द करें
exception.action.advanced = उन्नत विकल्प
exception.badge = { $rule } अनुमत · { $until } तक
exception.badge_permanent = { $rule } स्थायी रूप से अनुमत · आपका निर्णय
exception.badge_many = { $count } नियम आपके निर्णय से अनुमत · विवरण
exception.until_reboot = रीस्टार्ट तक
exception.permanent_note = स्थायी अपवाद अपने आप खत्म नहीं होता। जब तक आप रद्द न करें, यह दिखता रहेगा।
exception.revoke_note = आप कभी भी रद्द कर सकते हैं — एक क्लिक, बिना पासवर्ड।
exception.expert_note = विशेषज्ञ मोड में “स्थायी” और “पूरा नियम” दिखते हैं। न पासवर्ड, न कारण पूछना।
exception.expired_report = अपवाद खत्म हो गया। उस दौरान { $count } फ़ाइलें संदिग्ध लगीं — अब देखें।
exception.expired_clean = अपवाद खत्म हो गया। जाँचने योग्य कुछ नहीं दिखा।
exception.refused.not_allowed = इस नियम को पूरी अनुमति नहीं दी जा सकती — सिर्फ़ ढीला किया जा सकता है।
exception.refused.above_ceiling = इस नियम की अधिकतम सीमा ढील ही है।
exception.refused.no_permanent = इस नियम के लिए स्थायी अपवाद संभव नहीं।
exception.refused.expert_only = पूरे नियम का स्थायी अपवाद विशेषज्ञ मोड में ही होगा।
exception.refused.veto_stands = फ़ाइल पुष्ट है। अपवाद नियम ढीला करता है; फ़ाइल को साफ़ नहीं करता।

exception.live_note = आपका निर्णय · बाकी सब सक्रिय है · आप कभी भी रद्द कर सकते हैं
exception.lines_title = चार चीज़ें कभी अपवाद नहीं होतीं:
exception.lines_body = पहली दीवार का निर्णय (अपवाद के बावजूद पुष्ट फ़ाइल अलग की जाती है) · छेड़छाड़ सुरक्षा · दूसरा नियम (WSH अपवाद PowerShell नहीं खोलता) · “उच्च जोखिम” और ऊपर। रद्द करना हमेशा एक क्लिक, बिना पासवर्ड।

## ── ऐप का नक्शा और नेविगेशन (संविधान §6أ.3) ─────────────────
nav.home = होम
nav.quarantine = क्वारंटीन
nav.proactive = पूर्ववर्ती सुरक्षा
nav.alerts = सूचना केंद्र
nav.settings = सेटिंग्स
nav.language = भाषा
nav.privacy = गोपनीयता
back = वापस
menu.more = और
menu.advanced = उन्नत विकल्प
map.title = ऐप का नक्शा
map.hint = हर स्क्रीन कहाँ से खुलती है · अधिकतम तीन स्तर

## ── कोड साइनिंग नीति (सार्वजनिक पृष्ठ) ─────────────────────────
signing.title = कोड साइनिंग नीति
signing.sub = हम क्या साइन करते हैं, क्या कभी नहीं, कुंजी कहाँ है, और आप स्वयं कैसे जाँचें।
signing.status_title = आज की स्थिति
signing.status_body = यह नीति हस्ताक्षर शुरू होने से पहले प्रकाशित है, और अभी कोई बिल्ड साइन नहीं हुआ। पहला हस्ताक्षरित बिल्ड अपने रिलीज़ पृष्ठ पर हैश के साथ घोषित होगा।
signing.status_until = तब तक प्रोग्राम पोर्टेबल ZIP के रूप में मिलता है, बिना हस्ताक्षर वाले इंस्टॉलर के रूप में कभी नहीं।
signing.build_title = बिल्ड
signing.build_body = सभी बिल्ड GitHub Actions पर उसी कमिट से बनते हैं जो रिलीज़ नोट्स में लिखा है। कोई स्थानीय बिल्ड नहीं, और साइन करने से पहले दोबारा बिल्ड कभी नहीं।
signing.key_title = कुंजी
signing.key_body = हम कोई साइनिंग कुंजी नहीं रखते। कुंजी SignPath Foundation के HSM में है, और हर हस्ताक्षर के लिए अनुरक्षक की स्पष्ट स्वीकृति चाहिए।
signing.signed_title = हस्ताक्षरित
signing.signed_installer = इंस्टॉलर
signing.signed_store = स्टोर पैकेज
signing.unsigned_title = बिना हस्ताक्षर
signing.unsigned_db = सिग्नेचर डेटाबेस — अलग Ed25519 कुंजी से हस्ताक्षरित, क्योंकि ये प्रोग्राम से कहीं अधिक बार अपडेट होते हैं।
signing.unsigned_tools = tools में सब कुछ — MIT लाइसेंस और स्रोत के रूप में, बाइनरी के रूप में कभी नहीं।
signing.verify_title = सत्यापन
signing.verify_body = हर आर्टिफ़ैक्ट का SHA-256 हस्ताक्षर से पहले और बाद में रिलीज़ पृष्ठ पर प्रकाशित होता है। आप Windows पर स्वयं जाँच सकते हैं:
signing.verify_must = स्थिति Valid होनी चाहिए और प्रकाशक SignPath Foundation। यदि हैश प्रकाशित से भिन्न है, तो फ़ाइल न चलाएँ।
signing.revoke_title = निरसन
signing.revoke_body = यदि कभी बिना समीक्षा वाले कोड से बिल्ड बने, तो उसे निरस्त किया जाएगा और निरसन की घोषणा सबसे पहले होगी।
signing.credit = मुफ़्त कोड हस्ताक्षर SignPath.io द्वारा, प्रमाणपत्र SignPath Foundation द्वारा।
signing.updated = अंतिम अद्यतन

## ── शब्दावली — प्रति शब्द एक पंक्ति ─────────────
tip.confirmed = इसका हैश ख़तरे के डेटाबेस में है। यह अनुमान नहीं है।
tip.high_risk = ख़तरे के प्रबल संकेत, और हैश अभी पुष्ट नहीं।
tip.suspicious = अजीब लगता है: नाम, जगह या व्यवहार। जाँचा नहीं।
tip.pua = वायरस नहीं, पर किसी दूसरे प्रोग्राम की सुरक्षा को बायपास करता है।
tip.signature = ज्ञात फ़ाइलों के हैश। इन्हें अद्यतन रखना सुरक्षा को ताज़ा रखता है।
tip.behavior = देखता है प्रोग्राम क्या करता है, नाम क्या है नहीं।
tip.proactive = होने के बाद ढूँढने के बजाय होने से पहले रोकता है।
tip.pause = हम सिर्फ़ विंडो शांत करते हैं। जाँच और रोक जारी रहती है।
tip.turn_off = न जाँच, न रोक। इस दौरान आपका डिवाइस असुरक्षित है।
tip.exception = आपके निर्णय से एक नियम ढीला। बाकी कुछ नहीं बदलता।
tip.expert_mode = स्थायी और पूरे नियम के विकल्प दिखाता है। न पासवर्ड, न सवाल।
tip.wsh = Windows स्क्रिप्ट होस्ट। मैलवेयर का आम रास्ता।
tip.powershell = Windows में कमांड टूल। जोखिम भरे स्रोतों से डिफ़ॉल्ट रूप से प्रतिबंधित।
tip.macros = Office दस्तावेज़ के अंदर कमांड। अहस्ताक्षरित फ़ाइल में नहीं चलते।
tip.amsi = Windows इंटरफ़ेस जो स्क्रिप्ट को चलने से पहले दिखाता है।
tip.wdac = Windows नीति जो एक ख़ास फ़ाइल को चलने से रोकती है।
tip.usb_trust = पहली बार देखा। खोलने से पहले जाँचा जाता है।
tip.tamper = Odina की फ़ाइलें बदलकर या सेवा बंद करके इसे बंद नहीं किया जा सकता।
tip.hash = पूरी फ़ाइल को दर्शाने वाली स्ट्रिंग। एक बाइट सब बदल देती है।
tip.heuristic = मापा और तौला जाने वाला संकेत। अकेले कभी पर्याप्त नहीं।

## ── सेटिंग्स, गोपनीयता और भाषा ─────────────────────────────
settings.title = सेटिंग्स
settings.sub = चार खंड। “उन्नत” के पीछे सिर्फ़ दुर्लभ चीज़ें हैं।
settings.appearance = रूप
settings.appearance.body = हल्का या गहरा। डिफ़ॉल्ट रूप से सिस्टम के अनुसार।
settings.theme_light = हल्का
settings.theme_dark = गहरा
settings.theme_system = सिस्टम के अनुसार
settings.behavior = व्यवहार
settings.startup = Windows के साथ शुरू
settings.startup.body = बूट से ही पृष्ठभूमि में चलता है। आप इसे बंद कर सकते हैं।
settings.notifications = सूचनाएँ
settings.notifications.body = दिन में अधिकतम तीन। सबसे ज़रूरी पहले।
settings.updates = अद्यतन
settings.updates.body = ख़तरा डेटाबेस के लिए स्वचालित, प्रोग्राम के लिए मैन्युअल।
settings.about = ऐप के बारे में
settings.version = संस्करण
settings.database = ख़तरा डेटाबेस
settings.database.date = अंतिम अद्यतन

privacy.title = गोपनीयता
privacy.sub = आपके डिवाइस से क्या जाता है और क्या कभी नहीं। कोई अपवाद नहीं।
privacy.telemetry = शून्य टेलीमेट्री
privacy.telemetry.body = न आँकड़े, न डिवाइस आईडी, न फ़िंगरप्रिंट। अपने आप कुछ नहीं जाता।
privacy.local = सब कुछ स्थानीय
privacy.local.body = जाँच, निर्णय और लॉग आपकी डिस्क पर रहते हैं।
privacy.report = रिपोर्ट आपका निर्णय है
privacy.report.body = रिपोर्ट तभी जाती है जब आप भेजना चुनें।
privacy.search = माँग पर खोज
privacy.search.body = आपकी भाषा में सर्च इंजन खोलता है। सिर्फ़ शब्द भेजते हैं, आपकी फ़ाइलें कभी नहीं।
privacy.log = लॉग
privacy.log.body = स्थानीय, और आपकी भाषा में पढ़ने योग्य। जब चाहें मिटा सकते हैं।
privacy.log_clear = लॉग मिटाएँ

lang.title = भाषा
lang.sub = दस भाषाएँ। अनुवाद तुरंत दिखता है और फ़ॉन्ट आपकी भाषा के अनुसार।
lang.current = वर्तमान भाषा
lang.rtl = दाएँ से बाएँ लिखी जाती है
lang.ltr = बाएँ से दाएँ लिखी जाती है
lang.review = मातृभाषी समीक्षा
lang.review.body = जिस भाषा की मातृभाषी समीक्षा नहीं हुई, वह रोक दी जाती है, भेजी नहीं जाती।

home.walls = तीन दीवारें
home.action.scan = अभी जाँचें
home.action.pause = रोकें
home.action.expert = विशेषज्ञ मोड
home.action.why = क्यों?
home.goto.quarantine = क्वारंटीन
home.goto.alerts = सूचना केंद्र
home.goto.settings = सेटिंग्स
home.goto.proactive = पूर्ववर्ती सुरक्षा
home.quarantine_count = { $count } फ़ाइलें क्वारंटीन में
home.alerts_count = { $count } सूचनाएँ देखने योग्य
home.proactive_summary = छह नियम · एक अपवादित
proactive.title = पूर्ववर्ती सुरक्षा
proactive.sub = छह नियम। हर एक का अपना स्विच, और कोई दूसरे को नहीं खोलता।
alerts.title = सूचना केंद्र
alerts.sub = देखने योग्य सब कुछ एक जगह। कोई पॉप-अप नहीं।
alerts.empty = अभी देखने योग्य कुछ नहीं।
exception.detail.title = अपवाद — एक नियम
exception.detail.sub = क्या बदलता है, कितनी देर, और कैसे लौटता है।
map.node.home = होम
map.node.quarantine = क्वारंटीन
map.node.proactive = पूर्ववर्ती
map.edge.from_home = होम से
map.edge.from_alert = सूचना से
map.legend = ठोस रेखा स्थायी प्रवेश · बिंदुदार को संदर्भ चाहिए

## ── माँग पर व्याख्या ─────────────────────────────
help.title = माँग पर व्याख्या
help.hint = होवर पर एक पंक्ति · क्लिक पर पूरी खोज, आपकी भाषा में।
help.tip.title = तुरंत संकेत
help.tip.body = एक रेखांकित शब्द, और 90 अक्षरों तक की एक पंक्ति।
help.guide.title = लंबी मार्गदर्शिका
help.guide.body = संकेत के अंदर के बटन से खुलती है और जाने-पहचाने सर्च इंजन में पढ़ी जाती है।
help.guide.cta = खोज खोलें
help.engine.title = इंजन भाषा और स्थान के अनुसार
help.col.lang = भाषा
help.col.engine = इंजन
help.col.why = कारण
help.why.default = इस भाषा के लिए सबसे व्यापक कवरेज
help.why.zh = चीन के भीतर का संदर्भ, जहाँ Google सीमित है
help.why.ja = अनूदित नहीं, जापानी पृष्ठ पहले
help.why.ru = रूसी शब्द-रूपों को दूसरों से बेहतर समझता है
help.query.note = सिर्फ़ शब्द भेजते हैं। फ़ाइल नाम, पथ या डिवाइस आईडी नहीं।
path.copy_hint = कॉपी
nav.preview = इंटरफ़ेस पूर्वावलोकन

## ── التنبيهات والشارات ────────────────────────────
alert.confirmed.cta = क्वारंटीन खोलें
badge.confirmed = पुष्ट
badge.high = अत्यधिक संदिग्ध
badge.low = संदिग्ध
badge.pua = मध्यम जोखिम
button.off = बंद
button.on = चालू

## ── الإيقاف المؤقت ────────────────────────────
lvl.app = ऐप
lvl.d10 = { $n } मिनट
lvl.d30 = { $n } मिनट
lvl.d60 = 1 घंटा
lvl.dpeak = व्यस्त घंटे के अंत तक
lvl.dreboot = रीस्टार्ट तक
lvl.hint = हम बताते हैं, फ़ैसला आपका।
lvl.no_permanent = कोई स्थायी विराम नहीं। सबसे लंबा रीस्टार्ट तक।
lvl.state_off = सुरक्षा बंद है
lvl.state_off_sub = 14:30 तक · आपके निर्णय से
lvl.title = बंद करना — एक नहीं, दो स्तर
lvl.what_continues = क्या चलता रहता है
lvl.what_stops = क्या रुकता है
lvl1.app = चलता · जाँचता · रोकता · पूछता नहीं
lvl1.body = न विंडो, न आवाज़। जाँच और रोक जारी है।
lvl1.cont = तीनों दीवारें
lvl1.name = सूचनाएँ शांत
lvl1.stops = सिर्फ़ सूचनाएँ
lvl2.body = न जाँच, न रोक। आपका डिवाइस असुरक्षित है।
lvl2.name = सुरक्षा बंद
lvl2.resp = उस दौरान ज़िम्मेदारी आपकी है।
lvl2.warn = हम सुरक्षा रोकते हैं, बताना नहीं।

## ── دستور الواجهة — القواعد الخمس ────────────────────────────
notes.1b = दिशा और लेआउट तुरंत बदलते हैं; अनुपात और हैश में अंक लैटिन रहते हैं।
notes.1t = भाषा बदलना:
notes.2b = अनुमति/रोकें का तार्किक क्रम स्थिर रहता है; RTL केवल दृश्य स्थान बदलता है।
notes.2t = बटन:
notes.3b = यहाँ सिस्टम फ़ॉन्ट; CDN पैक डाउनलोड के बाद बिना रीस्टार्ट Noto बन जाते हैं।
notes.3t = CJK:
notes.4b = यह फ़ाइल fonts/vendor/ से लोड करती है — स्थानीय सर्वर से खोलें, file:// से नहीं।
notes.4t = फ़ॉन्ट:
notes.5b = लोगोटाइप, ट्रे आइकन और त्रुटि अवस्थाएँ — अलग समीक्षा (R3)।
notes.5t = इसमें शामिल नहीं:

## ── صندوق العزل ────────────────────────────
q.reason1 = पुष्ट ख़तरा
q.reason2 = क्रैकिंग टूल
quarantine.opened_from = सूचना और होम के काउंटर से खुलता है — कभी छिपे मेनू से नहीं।
quarantine.restore = पुनर्स्थापित

## ── دستور الواجهة — القواعد الخمس ────────────────────────────
sec.alerts = 2 · चार चेतावनियाँ
sec.alerts.hint = 2 बटन + «क्यों?» · रंग अकेले अर्थ नहीं देता
sec.color = 4 · रंग पटल (WCAG मापा गया)
sec.color.hint = हर रंग वर्तमान पृष्ठभूमि पर अपना कंट्रास्ट दिखाता है
sec.fonts = 5 · प्रति भाषा फ़ॉन्ट
sec.fonts.hint = केवल 400 और 600 · सिस्टम फ़ॉन्ट तत्काल विकल्प
sec.main = 1 · मुख्य स्क्रीन
sec.main.hint = ≤ 8 तत्व · 3 दीवारें · 2 बटन
sec.notes = 6 · समीक्षा नोट्स
sec.usb = 3 · USB चेतावनी
sec.usb.hint = एक प्रगति पट्टी · एक चरण

## ── الهوية ────────────────────────────
shield.sub = { $n } दीवारें सक्रिय · ख़तरा सूची अद्यतित

## ── الشرح عند الطلب ────────────────────────────
tips.body = रेखांकित शब्द पर माउस ले जाएँ — one line · क्लिक करने पर आपकी भाषा में समझाने वाली खोज खुलेगी (Google · Baidu · Yandex)।
tips.legend = ज़रूरत पर व्याख्या:

## ── دستور الواجهة — القواعد الخمس ────────────────────────────
trust.1d = हम बताते हैं क्या हुआ, न कि तकनीक कैसे चलती है। एक वाक्य। बिना कारण “हो गया” नहीं।
trust.1t = स्पष्टता
trust.2d = अधिकतम दो बटन। एक मुख्य क्रिया। कोई सेटअप नहीं।
trust.2t = सरलता
trust.3d = घटना नहीं = विंडो नहीं। दिन में ≤ 3 सूचनाएँ। कभी मोडल नहीं।
trust.3t = सुकून
trust.4d = हर बटन दबाने से पहले नतीजा लिखा हो। सुरक्षित विकल्प सामने हो।
trust.4t = नतीजे पता हों
trust.sub = हर स्क्रीन इन चार पर परखी जाती है। टकराव में: सुरक्षा → बिना टोकना → स्पष्टता → सरलता → सौंदर्य।

## ── USB ────────────────────────────
usb.stage = P0 · रूट में ख़तरनाक फ़ाइलें

## ── अंक ─────────────────────────────
num.title = अंक
num.sub = काउंटर, सूचनाओं और लॉग में दिखने वाले अंक।
num.body = वह रूप चुनें जो आप सबसे तेज़ पढ़ते हैं। गणना एक ही रहती है।
num.arabic = 0 1 2 3 4 5 6 7 8 9
num.hindi = ٠ ١ ٢ ٣ ٤ ٥ ٦ ٧ ٨ ٩
num.arabic.name = अरबी अंक
num.hindi.name = पूर्वी अरबी अंक
num.sample = 3 सूचनाएँ · 12 फ़ाइलें
num.note = बदलाव तुरंत। रीस्टार्ट की ज़रूरत नहीं।

## ── विराम के संकेत — माँग पर ─────────────────────────────
tip.lvl1 = न विंडो, न सूचना, न आवाज़, पर जाँच और रोक पृष्ठभूमि में जारी है: संक्रमित फ़ाइल चुपचाप अलग की जाती है और लौटने पर दिखाई जाती है।
tip.lvl2 = न जाँच, न रोक, न सूचना। सेवा चलती है पर कुछ नहीं करती, और आपका डिवाइस पूरी तरह असुरक्षित है।
tip.no_permanent = सबसे लंबा विराम रीस्टार्ट पर खत्म होता है, और यही हमारी एकमात्र सीमा है।

## ── चौथा नियम: पहले दृश्य ─────────────────────────────
visual.title = पहले दृश्य
visual.rule = वाक्य से पहले चित्र: स्क्रीन क्या करती है, दिखता है, समझाया नहीं जाता।
visual.why = जो धीरे पढ़ता है, या आपकी भाषा नहीं पढ़ता, वह पंक्ति से पहले आकार और रंग समझता है।
visual.n1 = हर तत्व में आकार, रंग और जगह है, वह किसी शब्द पर निर्भर नहीं।
visual.n2 = वही आकार हर स्क्रीन पर वही अर्थ रखता है।
visual.n3 = रंग सुरक्षित और ख़तरे को अलग करता है; पाठ कारण बताता है।
visual.n4 = बिना शब्द का आइकन एक परीक्षा है: अगर समझाना पड़े, तो आइकन ग़लत है।
visual.n5 = पाठ चित्र को समझाता है, दोहराता नहीं।
visual.sample = आइकन पर रुकें: एक शब्द। क्लिक करें: आपकी भाषा में पूरी व्याख्या।
trust.count = पाँच, चार नहीं।
help.sample.title = एक जीवंत उदाहरण
