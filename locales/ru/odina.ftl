# Odina Antivirus — Русский (ru)
# المستوى: T-B (مجتمعية) — تحتاج مراجعة ناطق أصلي قبل ترقيتها إلى T-A (§30.2)
# راجع كل سطر يدويًا قبل الاعتماد. لا ترجمة آلية للنصوص الأمنية (§3.9.1).

app.name = Odina
app.tagline = Ваш цифровой страж
app.window.title = Odina — { $state }
state.protected = Вы защищены
state.scanning = Идёт проверка…
state.attention = Требует внимания
state.paused = Защита приостановлена
state.untrusted = Ещё не проверено
wall.signatures.title = База файлов
wall.signatures.on = Работает
wall.signatures.off = Не работает
wall.behavior.title = Глубокая проверка поведения
wall.behavior.on = Активен
wall.behavior.off = Неактивна
wall.proactive.title = Упреждающая защита
wall.proactive.on = Включена
wall.proactive.off = Отключена
action.scan_now = Проверить сейчас
action.pause = Приостановить
action.expert_mode = Режим эксперта
action.language = Язык
action.settings = Настройки
action.why = Почему?
scan.quick = Быстрая проверка
scan.full = Полная проверка
scan.usb = Проверить этот диск
scan.running = Проверка { $target }…
scan.percent = { $percent } %
scan.files_checked = Проверено файлов
scan.threats_found = Найдено угроз
scan.stop = Остановить
scan.complete = Проверка завершена
scan.clean_result = Вредоносного не найдено.
alert.confirmed.title = Мы остановили известную угрозу
alert.confirmed.body = { $file } — подтверждённая угроза, изолирован — не удалён.
alert.suspicious.title = Программа пытается получить доступ к вашим файлам
alert.suspicious.body = { $file } изменяет файлы в Документах. Мы временно остановили её.
alert.suspicious.note = Мы временно остановили её.
alert.unsure.title = Эта программа необычна
alert.unsure.body = { $file } здесь новое и ведёт себя странно.
alert.pua.title = Похоже на средство взлома
alert.pua.body = { $file } обходит защиту лицензий платных программ.
alert.pua.allowed = Разрешить
alert.pua.quarantine = Убрать в сторону
button.allow = Разрешить
button.block = Заблокировать
button.restore = Восстановить
button.remove_forever = Удалить навсегда
usb.detected = Обнаружен новый диск — сначала проверим его
usb.scanning = Проверка { $name }…
usb.priority_done = Опасные файлы проверены
usb.trust.new = Новый диск
usb.trust.scanning = Проверка
usb.trust.trusted = Запомнен
usb.trust.suspicious = Под наблюдением
usb.trust.quarantined = Заблокирован
quarantine.title = Изолированные объекты
quarantine.reason = Причина изоляции
quarantine.restore_note = Исходное расположение сохранено.
quarantine.retention = Хранится { $days } дн.
pause.title = Пауза
pause.duration.10 = 10 минут
pause.duration.30 = 30 минут
pause.duration.60 = 1 час
pause.warning = Мы остановим уведомления до { $until }. Защита продолжается.
pause.confirm = Подтвердить
pause.badge = Уведомления на паузе · защита работает
error.generic = Что-то пошло не так. Мы ничего не меняли.
error.tamper = Файлы Odina были изменены. Требуется восстановление.
error.offline = Обновления пока нет. Защита работает.
update.available = Доступно обновление
update.ask = Проверить и загрузить обновления?
update.db_age_warning = Вашему списку угроз { $days } дн.

## ── Path ──────────────────────────────────────────────
path.label = Путь
path.copy = Копировать путь
path.open_folder = Открыть папку
trust.title = Основы доверия
word.isolated = изолирован
word.temporary = временно
tip.quarantine = Перемещён в зашифрованное хранилище — исходное расположение сохранено.
tip.suspended = Мы остановили его сейчас. Он вернётся только с вашего разрешения.
quarantine.hint = Всё остаётся, пока вы не решите · автоудаления нет
quarantine.keep = Оставить
quarantine.delete = Удалить навсегда
quarantine.delete_note = Отменить нельзя. Другой копии нет.
quarantine.cleanup_title = В изоляции { $days } дн.
quarantine.cleanup_choose = Удалить выбранные
error.false_positive = Мы ошиблись с этим файлом — исправлено.
error.locked = Не удалось изолировать: файл сейчас открыт.

## ── Turn protection off (user's explicit decision) ────────
pause.confirm_disable = Отключить защиту на { $minutes} мин.
disable.title = Отключить защиту
disable.warning_title = Серьёзный риск
disable.body = Мы не просто перестанем уведомлять — мы перестанем защищать. Ни один файл не будет проверен, ни одна угроза не будет заблокирована.
disable.responsibility = Это ваше решение, и ответственность в этот период на вас.
disable.choose = Выберите срок
disable.duration.peak = До конца часа пик
disable.duration.30 = 30 минут
disable.duration.60 = 1 час
disable.duration.until_reboot = До перезагрузки
disable.refused = Сейчас защиту отключить нельзя
disable.refused_quarantine = Есть изолированный файл без решения. Сначала рассмотрите его.
disable.refused_scanning = Идёт проверка. Дождитесь её или остановите.
disable.refused_active_threat = Есть нерешённая угроза. Пока она открыта, защиту не отключим.
state.unprotected = Защита отключена
state.unprotected_until = Защита отключена до { $until}
state.unprotected_badge = Без защиты — по вашему решению
action.turn_off = Отключить защиту
action.turn_on = Включить защиту снова
disable.re_enable_note = Включить защиту можно в любой момент — без пароля.
disable.reminder = Через { $minutes} мин. защита включится автоматически.
pause.resume_report = Пока уведомления были на паузе: изолировано { $count}

## ── Точечное исключение (D45 · §4.8) ─────────────────────────
exception.title = Исключения — одно правило, а не все
exception.sub = Если для работы нужно ослабить одно правило, разрешите только его. Остальное остаётся как было.
exception.rule.a1.wsh = Скрипты Windows
exception.rule.a1.wsh.desc = Файлы .vbs и .js запускаются с ограничениями, если они не подписаны известным издателем.
exception.rule.a1.powershell = Команды PowerShell
exception.rule.a1.powershell.desc = PowerShell работает в ограниченном режиме. Разрешение не открывает Windows Script Host.
exception.rule.a1.macros = Макросы Office
exception.rule.a1.macros.desc = Документы без подписи не могут запускать макросы. Разрешение не затрагивает скрипты.
exception.rule.a3.downloads = Подозрение в «Загрузках»
exception.rule.a3.downloads.desc = Каждый новый файл в «Загрузках» считается подозрительным, пока не оправдан.
exception.rule.a4.periodic = Периодическая проверка
exception.rule.a4.periodic.desc = Тихая проверка элементов автозапуска и важных ключей каждые два часа.
exception.rule.a6.doubt = Осторожность при сомнении
exception.rule.a6.doubt.desc = При сомнении сначала блокирует, затем спрашивает. Можно ослабить, но не разрешить полностью.
exception.state.on = Активно
exception.state.relaxed_until = Ослаблено до { $until }
exception.state.allowed_until = Разрешено до { $until }
exception.state.allowed_permanent = Разрешено постоянно — ваше решение
exception.consequence.relax = { $rule } будет работать без уведомлений до { $until}, затем вернётся. В это время ничего не блокируется.
exception.consequence.allow = { $rule } будет работать без ограничений до { $until}, затем вернётся. Остальное не меняется.
exception.consequence.permanent = { $rule } останется разрешённым, пока вы не отмените. Эта строка будет видна на главном экране.
exception.action.relax_10 = Ослабить на 10 минут
exception.action.relax_30 = Ослабить на 30 минут
exception.action.relax_60 = Ослабить на час
exception.action.allow = Разрешить
exception.action.revoke = Отменить
exception.action.revoke_all = Отменить все исключения
exception.action.advanced = Дополнительно
exception.badge = { $rule } разрешено · до { $until }
exception.badge_permanent = { $rule } разрешено постоянно · ваше решение
exception.badge_many = { $count } правила разрешены по вашему решению · подробности
exception.until_reboot = до перезагрузки
exception.permanent_note = Постоянное исключение не заканчивается само. Оно остаётся на виду, пока вы его не отмените.
exception.revoke_note = Отменить можно в любой момент — одно нажатие, без пароля.
exception.expert_note = Экспертный режим показывает «постоянно» и «всё правило». Без пароля и без вопроса о причине.
exception.expired_report = Исключение закончилось. За это время { $count } файл(ов) показались подозрительными — проверьте сейчас.
exception.expired_clean = Исключение закончилось. Ничего, что требует проверки, не появилось.
exception.refused.not_allowed = Это правило нельзя разрешить полностью — только ослабить.
exception.refused.above_ceiling = Предел для этого правила — ослабление.
exception.refused.no_permanent = Для этого правила постоянное исключение невозможно.
exception.refused.expert_only = Постоянное исключение для всего правила требует экспертного режима.
exception.refused.veto_stands = Файл подтверждён. Исключение ослабляет правило; оно не оправдывает файл.

exception.live_note = ваше решение · всё остальное действует · отменить можно в любой момент
exception.lines_title = Четыре вещи не исключаются никогда:
exception.lines_body = вердикт первой стены (подтверждённое изолируется даже при исключении) · защита от подмены · второе правило (исключение для WSH не открывает PowerShell) · «высокий риск» и выше. Отмена — всегда одно нажатие, без пароля.

## ── Карта приложения и навигация (конституция §6أ.3) ─────────────────
nav.home = Главная
nav.quarantine = Карантин
nav.proactive = Упреждающая защита
nav.alerts = Центр уведомлений
nav.settings = Настройки
nav.language = Язык
nav.privacy = Приватность
back = Назад
menu.more = Ещё
menu.advanced = Дополнительно
map.title = Карта приложения
map.hint = Как открывается каждый экран · не больше трёх уровней

## ── Политика подписи кода (публичная страница) ─────────────────────────
signing.title = Политика подписи кода
signing.sub = Что мы подписываем, что не подписываем никогда, где хранится ключ и как проверить самому.
signing.status_title = Состояние сегодня
signing.status_body = Эта политика опубликована до начала подписывания, и ни одна сборка ещё не подписана. Первая подписанная сборка будет объявлена на странице релиза вместе с хешем.
signing.status_until = До тех пор программа распространяется как переносимый ZIP-архив, а не как неподписанный установщик.
signing.build_title = Сборка
signing.build_body = Все сборки идут на GitHub Actions из коммита, указанного в примечаниях к релизу. Локальной сборки нет, и бинарный файл никогда не пересобирается перед подписью.
signing.key_title = Ключ
signing.key_body = Мы не храним ключ подписи. Ключ находится в аппаратном модуле SignPath Foundation, и каждая подпись требует явного одобрения сопровождающего.
signing.signed_title = Что подписывается
signing.signed_installer = Установщик
signing.signed_store = Пакет для магазина
signing.unsigned_title = Что не подписывается
signing.unsigned_db = Базы подписей — подписываются отдельным ключом Ed25519, потому что обновляются гораздо чаще программы.
signing.unsigned_tools = Всё в папке tools — под лицензией MIT и как исходный код, никогда как двоичные файлы.
signing.verify_title = Проверка
signing.verify_body = SHA-256 каждого артефакта публикуется на странице релиза до и после подписи. Вы можете проверить это сами в Windows:
signing.verify_must = Статус должен быть Valid, а издатель — SignPath Foundation. Если хеш отличается от опубликованного, не запускайте файл.
signing.revoke_title = Отзыв
signing.revoke_body = Если сборка когда-либо будет сделана из непроверенного кода, она отзывается, и об отзыве объявляется прежде всего остального.
signing.credit = Бесплатная подпись кода от SignPath.io, сертификат от SignPath Foundation.
signing.updated = Последнее обновление

## ── Глоссарий — одна строка на термин ─────────────
tip.confirmed = Его хеш есть в базе угроз. Это не догадка.
tip.high_risk = Сильные признаки опасности, хеш ещё не подтверждён.
tip.suspicious = Выглядит странно: имя, место или поведение. Не проверено.
tip.pua = Не вирус, но обходит защиту другой программы.
tip.signature = Хеши известных файлов. Обновление держит защиту актуальной.
tip.behavior = Наблюдает за тем, что программа делает, а не как называется.
tip.proactive = Останавливает действие до того, как оно произойдёт, а не находит после.
tip.pause = Мы заглушаем только окна. Проверка и блокировка продолжаются.
tip.turn_off = Ни проверки, ни блокировки. Всё это время устройство без защиты.
tip.exception = Ослабить одно правило по вашему решению. Остальное не меняется.
tip.expert_mode = Показывает постоянный вариант и всё правило. Без пароля и без вопросов.
tip.wsh = Узел сценариев Windows. Частый путь для вредоносных программ.
tip.powershell = Инструмент команд в Windows. По умолчанию ограничен при опасных источниках.
tip.macros = Команды внутри документа Office. В неподписанном файле они не запускаются.
tip.amsi = Интерфейс Windows, который показывает сценарий до его запуска.
tip.wdac = Политика Windows, запрещающая запуск конкретного файла.
tip.usb_trust = Мы видим его впервые. Проверяется до открытия.
tip.tamper = Odina нельзя отключить, изменив его файлы или остановив службу.
tip.hash = Строка, представляющая файл целиком. Один байт меняет всё.
tip.heuristic = Сигнал, который измеряется и взвешивается. Один никогда не достаточен.

## ── Настройки, приватность и язык ─────────────────────────────
settings.title = Настройки
settings.sub = Четыре раздела. За «дополнительно» только редкое.
settings.appearance = Оформление
settings.appearance.body = Светлое или тёмное. По умолчанию как в системе.
settings.theme_light = Светлое
settings.theme_dark = Тёмное
settings.theme_system = Как в системе
settings.behavior = Поведение
settings.startup = Запускается с Windows
settings.startup.body = Работает в фоне с самого запуска. Можно отключить.
settings.notifications = Уведомления
settings.notifications.body = Не больше трёх в день. Сначала важное.
settings.updates = Обновления
settings.updates.body = Автоматически для базы угроз, вручную для программы.
settings.about = О программе
settings.version = Версия
settings.database = База угроз
settings.database.date = Последнее обновление

privacy.title = Приватность
privacy.sub = Что покидает устройство и что не покидает никогда. Без исключений.
privacy.telemetry = Нулевая телеметрия
privacy.telemetry.body = Ни статистики, ни идентификатора устройства, ни отпечатка. Ничего не уходит само.
privacy.local = Всё локально
privacy.local.body = Проверка, решение и журнал лежат на вашем диске.
privacy.report = Отчёт — ваше решение
privacy.report.body = Отчёт отправляется только если вы решили его отправить.
privacy.search = Поиск по запросу
privacy.search.body = Открывает поисковик на вашем языке. Отправляем только термин, никогда ваши файлы.
privacy.log = Журнал
privacy.log.body = Локальный и читаемый на вашем языке. Стереть можно в любой момент.
privacy.log_clear = Стереть журнал

lang.title = Язык
lang.sub = Десять языков. Перевод виден сразу, а шрифт следует за языком.
lang.current = Текущий язык
lang.rtl = Пишется справа налево
lang.ltr = Пишется слева направо
lang.review = Проверка носителем
lang.review.body = Язык без проверки носителем приостанавливается, а не выпускается.

home.walls = Три стены
home.action.scan = Проверить сейчас
home.action.pause = Пауза
home.action.expert = Экспертный режим
home.action.why = Почему?
home.goto.quarantine = Карантин
home.goto.alerts = Центр уведомлений
home.goto.settings = Настройки
home.goto.proactive = Упреждающая защита
home.quarantine_count = { $count } файл(ов) в карантине
home.alerts_count = { $count } уведомление(й) стоит взгляда
home.proactive_summary = Шесть правил · одно исключено
proactive.title = Упреждающая защита
proactive.sub = Шесть правил. У каждого свой переключатель, и ни один не открывает другой.
alerts.title = Центр уведомлений
alerts.sub = Всё, что стоит взгляда, в одном месте. Без всплывающих окон.
alerts.empty = Сейчас нет ничего, что стоит взгляда.
exception.detail.title = Исключение — одно правило
exception.detail.sub = Что меняется, на сколько и как возвращается.
map.node.home = Главная
map.node.quarantine = Карантин
map.node.proactive = Упреждающая
map.edge.from_home = с главной
map.edge.from_alert = из уведомления
map.legend = Сплошная линия — постоянный вход · пунктир требует контекста

## ── Объяснение по запросу ─────────────────────────────
help.title = Объяснение по запросу
help.hint = Строка при наведении · полный поиск по нажатию, на вашем языке.
help.tip.title = Мгновенная подсказка
help.tip.body = Одно подчёркнутое слово и строка не длиннее 90 знаков.
help.guide.title = Подробнее
help.guide.body = Открывается кнопкой внутри подсказки и читается в знакомом поисковике.
help.guide.cta = Открыть поиск
help.engine.title = Поисковик следует языку и месту
help.col.lang = Язык
help.col.engine = Поисковик
help.col.why = Причина
help.why.default = Наибольший охват для этого языка
help.why.zh = Ориентир внутри Китая, где Google ограничен
help.why.ja = Сначала японские страницы, не переводные
help.why.ru = Лучше других понимает русскую морфологию
help.query.note = Отправляем только термин. Ни имени файла, ни пути, ни идентификатора устройства.
path.copy_hint = копировать
nav.preview = Предпросмотр интерфейса

## ── التنبيهات والشارات ────────────────────────────
alert.confirmed.cta = Открыть изолятор
badge.confirmed = Подтверждено
badge.high = Сильное подозрение
badge.low = Подозрительно
badge.pua = Средний риск
button.off = Выключено
button.on = Включено

## ── الإيقاف المؤقت ────────────────────────────
lvl.app = Приложение
lvl.d10 = 10 минут
lvl.d30 = 30 минут
lvl.d60 = 1 час
lvl.dpeak = До конца часа пик
lvl.dreboot = До перезагрузки
lvl.hint = Устройство ваше · мы сообщаем, решение за вами · ответственность тоже
lvl.no_permanent = «Навсегда» нет. Самый долгий вариант заканчивается перезагрузкой — единственное ограничение.
lvl.state_off = Защита отключена
lvl.state_off_sub = до 14:30 · по вашему решению
lvl.title = Отключение — два уровня, не один
lvl.what_continues = Что продолжается
lvl.what_stops = Что останавливается
lvl1.app = работает · проверяет · блокирует · не спрашивает
lvl1.body = Без окон, уведомлений и звука. Но проверка и блокировка продолжаются в фоне: заражённый файл изолируется молча и будет показан по возвращении.
lvl1.cont = Все три стены
lvl1.name = Заглушить уведомления
lvl1.stops = Только уведомления
lvl2.body = Без проверки, блокировки и уведомлений. Служба работает, но простаивает. Устройство полностью без защиты.
lvl2.name = Отключить защиту
lvl2.resp = Это ваше решение, и ответственность в этот период на вас.
lvl2.warn = Мы не просто перестаём уведомлять — мы перестаём защищать. Ни один файл не проверяется и ни одна угроза не блокируется.

## ── دستور الواجهة — القواعد الخمس ────────────────────────────
notes.1b = меняет направление и вёрстку сразу; числа остаются латинскими в процентах и хешах.
notes.1t = Смена языка:
notes.2b = Разрешить/Заблокировать сохраняют логический порядок; RTL меняет только визуальную сторону.
notes.2t = Кнопки:
notes.3b = здесь системные шрифты; после загрузки пакета с CDN станут Noto без перезапуска.
notes.3t = CJK:
notes.4b = файл берёт шрифты из соседней папки fonts/vendor/ — открывайте через локальный сервер, не file://.
notes.4t = Шрифты:
notes.5b = логотип, значок в трее и состояния ошибок — отдельная проверка (R3).
notes.5t = Чего нет в превью:

## ── صندوق العزل ────────────────────────────
q.reason1 = Подтверждённая угроза
q.reason2 = Средство взлома
quarantine.opened_from = Открывается из уведомления и из счётчика на главной — никогда из скрытого меню.
quarantine.restore = Восстановить

## ── دستور الواجهة — القواعد الخمس ────────────────────────────
sec.alerts = 2 · Четыре уведомления
sec.alerts.hint = 2 кнопки + «Почему?» · цвет не передаёт смысл один
sec.color = 4 · Палитра (WCAG измерено)
sec.color.hint = у каждого цвета показан контраст к текущему фону
sec.fonts = 5 · Шрифты по языкам
sec.fonts.hint = только 400 и 600 · системные шрифты как мгновенная замена
sec.main = 1 · Главный экран
sec.main.hint = ≤ 8 элементов · 3 стены · 2 кнопки
sec.notes = 6 · Заметки для проверки
sec.usb = 3 · Уведомление USB
sec.usb.hint = один индикатор · один этап

## ── الهوية ────────────────────────────
shield.sub = 3 стены активны · база угроз обновлена

## ── الشرح عند الطلب ────────────────────────────
tips.body = Наведите на подчёркнутое слово — одна строка · нажмите, чтобы открыть поясняющий поиск на вашем языке (Google · Яндекс).
tips.legend = Объяснение по запросу:

## ── دستور الواجهة — القواعد الخمس ────────────────────────────
trust.1d = Говорим, что случилось, а не как устроено. Одно предложение. Без «готово» без причины.
trust.1t = Ясность
trust.2d = Не более двух кнопок. Одно главное действие. Настройка не нужна.
trust.2t = Простота
trust.3d = Нет события — нет окна. ≤ 3 уведомлений в день. Никаких модальных окон.
trust.3t = Спокойствие
trust.4d = Каждая кнопка сообщает результат до нажатия. Безопасный выбор — заметный.
trust.4t = Понятные последствия
trust.sub = Каждый экран проверяется по этим четырём. При конфликте: безопасность → не мешать → ясность → простота → красота.

## ── USB ────────────────────────────
usb.stage = P0 · опасные файлы в корне
