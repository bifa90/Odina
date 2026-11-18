# Odina Antivirus — 简体中文 (zh-Hans)
# المستوى: T-B (مجتمعية) — تحتاج مراجعة ناطق أصلي قبل ترقيتها إلى T-A (§30.2)
# راجع كل سطر يدويًا قبل الاعتماد. لا ترجمة آلية للنصوص الأمنية (§3.9.1).

app.name = Odina
app.tagline = 你的数字守护者
app.window.title = Odina — { $state }
state.protected = 你受到保护
state.scanning = 正在扫描…
state.attention = 需要注意
state.paused = 保护已暂停
state.untrusted = 尚未验证
wall.signatures.title = 文件特征库
wall.signatures.on = 运行中
wall.signatures.off = 未运行
wall.behavior.title = 深度行为检测
wall.behavior.on = 启用
wall.behavior.off = 休眠
wall.proactive.title = 主动防护
wall.proactive.on = 运行中
wall.proactive.off = 已关闭
action.scan_now = 立即扫描
action.pause = 暂停
action.expert_mode = 专家模式
action.language = 语言
action.settings = 设置
action.why = 为什么？
scan.quick = 快速扫描
scan.full = 完整扫描
scan.usb = 扫描此驱动器
scan.running = 正在扫描 { $target }…
scan.percent = { $percent }%
scan.files_checked = 已检查文件
scan.threats_found = 发现的威胁
scan.stop = 停止
scan.complete = 扫描完成
scan.clean_result = 未发现有害内容。
alert.confirmed.title = 我们拦截了一个已知威胁
alert.confirmed.body = { $file } 是已确认的威胁，已被隔离 —— 未删除。
alert.suspicious.title = 有程序正在访问你的文件
alert.suspicious.body = { $file } 正在修改文档中的文件——我们已暂时阻止它。
alert.suspicious.note = 我们已暂时阻止它。
alert.unsure.title = 这个程序不常见
alert.unsure.body = { $file } 是新来的，行为有些异常。
alert.pua.title = 这看起来是破解工具
alert.pua.body = { $file } 会绕过付费软件的授权保护。
alert.pua.allowed = 允许它
alert.pua.quarantine = 移到一边
button.allow = 允许
button.block = 阻止
button.restore = 恢复
button.remove_forever = 永久删除
usb.detected = 检测到新驱动器——先检查它
usb.scanning = 正在检查 { $name }…
usb.priority_done = 已检查危险文件
usb.trust.new = 新驱动器
usb.trust.scanning = 检查中
usb.trust.trusted = 已记住
usb.trust.suspicious = 监控中
usb.trust.quarantined = 已阻止
quarantine.title = 已隔离项目
quarantine.reason = 隔离原因
quarantine.restore_note = 原始位置已保留。
quarantine.retention = 保留 { $days } 天
pause.title = 暂停
pause.duration.10 = 10 分钟
pause.duration.30 = 30 分钟
pause.duration.60 = 1 小时
pause.warning = 我们会暂停提示，直到 { $until }。我们仍保护你。
pause.confirm = 确认
pause.badge = 已暂停提示 · 保护仍在运行
error.generic = 出现错误。我们没有更改任何内容。
error.tamper = Odina 自身的文件已被修改，请修复。
error.offline = 暂无更新。保护仍然有效。
update.available = 有可用更新
update.ask = 是否检查并下载更新？
update.db_age_warning = 你的威胁库已有 { $days } 天。

## ── Path ──────────────────────────────────────────────
path.label = 路径
path.copy = 复制路径
path.open_folder = 打开文件夹
trust.title = 信任基础
word.isolated = 已隔离
word.temporary = 暂时
tip.quarantine = 已移至加密存储——原位置保留，可随时恢复。
tip.suspended = 现在已停止。只有你允许后才会恢复。
quarantine.hint = 一切都会保留，直到你决定 · 绝不自动删除
quarantine.keep = 保留
quarantine.delete = 永久删除
quarantine.delete_note = 无法撤销。没有其他副本。
quarantine.cleanup_title = 已在隔离区 { $days } 天
quarantine.cleanup_choose = 删除我选中的
error.false_positive = 我们弄错了这个文件 — 已修正。
error.locked = 无法隔离：该文件当前已打开。

## ── Turn protection off (user's explicit decision) ────────
pause.confirm_disable = 关闭保护 { $minutes} 分钟
disable.title = 关闭保护
disable.warning_title = 严重风险
disable.body = 不只是停止提示——我们将停止保护。不会扫描任何文件，也不会拦截任何威胁。你的设备将完全失去保护。
disable.responsibility = 这是你的决定，这段时间的责任由你承担。
disable.choose = 选择时长
disable.duration.peak = 直到高峰时段结束
disable.duration.30 = 30 分钟
disable.duration.60 = 1 小时
disable.duration.until_reboot = 直到重启
disable.refused = 现在无法关闭保护
disable.refused_quarantine = 有隔离文件尚未处理。请先查看。
disable.refused_scanning = 正在扫描。请等待或先停止。
disable.refused_active_threat = 仍有未处理的威胁。威胁未处理前不会关闭保护。
state.unprotected = 保护已关闭
state.unprotected_until = 保护关闭至 { $until}
state.unprotected_badge = 未受保护——你的决定
action.turn_off = 关闭保护
action.turn_on = 重新开启保护
disable.re_enable_note = 你随时可以重新开启保护——无需密码。
disable.reminder = 距保护自动恢复还有 { $minutes} 分钟。
pause.resume_report = 暂停提示期间：已隔离 { $count} 个文件

## ── 精准例外 (D45 · §4.8) ─────────────────────────
exception.title = 例外
exception.sub = 如果你的工作需要放宽某一条规则，只放宽那一条。其余保持不变。
exception.rule.a1.wsh = Windows 脚本
exception.rule.a1.wsh.desc = .vbs 和 .js 文件在受限模式下运行，除非由已知发布者签名。
exception.rule.a1.powershell = PowerShell 命令
exception.rule.a1.powershell.desc = PowerShell 在受限模式下运行。允许它不会打开 Windows 脚本宿主。
exception.rule.a1.macros = Office 宏
exception.rule.a1.macros.desc = 未签名的文档不能运行宏。允许它不会影响脚本。
exception.rule.a3.downloads = 下载文件夹可疑
exception.rule.a3.downloads.desc = 下载文件夹中的每个新文件都会被视为可疑，直到被排除。
exception.rule.a4.periodic = 定期扫描
exception.rule.a4.periodic.desc = 每两小时静默扫描启动项和敏感注册表键。
exception.rule.a6.doubt = 存疑时谨慎
exception.rule.a6.doubt.desc = 存疑时先阻止再询问。可放宽，但不能完全允许。
exception.state.on = 生效
exception.state.relaxed_until = 已放宽至 { $until }
exception.state.allowed_until = 已允许至 { $until }
exception.state.allowed_permanent = 永久允许 —— 你的决定
exception.consequence.relax = { $rule } 将在 { $until } 前无提示运行，之后恢复。此期间不会阻止任何内容。
exception.consequence.allow = { $rule } 将在 { $until } 前不受限制地运行，之后恢复。其余不变。
exception.consequence.permanent = { $rule } 将一直允许，直到你撤销。这一行会一直显示在主页。
exception.action.relax_10 = 放宽 10 分钟
exception.action.relax_30 = 放宽 30 分钟
exception.action.relax_60 = 放宽 1 小时
exception.action.allow = 允许
exception.action.revoke = 撤销
exception.action.revoke_all = 撤销全部例外
exception.action.advanced = 高级选项
exception.badge = { $rule } 已允许 · 至 { $until }
exception.badge_permanent = { $rule } 永久允许 · 你的决定
exception.badge_many = { $count } 条规则经你决定被允许 · 详情
exception.until_reboot = 至重启
exception.permanent_note = 永久例外不会自行结束。它会一直显示在主页，直到你撤销。
exception.revoke_note = 你随时可以撤销 —— 一次点击，无需密码。
exception.expert_note = 专家模式显示“永久”和“整条规则”。无需密码，也不问原因。
exception.expired_report = 例外已结束。期间有 { $count } 个文件看似可疑，请现在查看。
exception.expired_clean = 例外已结束。没有出现需要查看的内容。
exception.refused.not_allowed = 此规则不能完全允许 —— 只能放宽。
exception.refused.above_ceiling = 此规则最多只能放宽。
exception.refused.no_permanent = 此规则不能永久例外。
exception.refused.expert_only = 整条规则的永久例外需要专家模式。
exception.refused.veto_stands = 该文件已确认。例外放宽的是规则，不会为文件开脱。

exception.live_note = 你的决定 · 其余仍然有效 · 你随时可以撤销
exception.lines_title = 四项永不可例外：
exception.lines_body = 第一道墙的判定（即使有例外，已确认的文件仍会隔离）· 防篡改 · 第二条规则（WSH 例外不会打开 PowerShell）·“高度风险”及以上。撤销始终只需一次点击，无需密码。

## ── 程序地图与导航 (章程 §6أ.3) ─────────────────
nav.home = 主页
nav.quarantine = 隔离区
nav.proactive = 主动防护
nav.alerts = 通知中心
nav.settings = 设置
nav.language = 语言
nav.privacy = 隐私
back = 返回
menu.more = 更多
menu.advanced = 高级选项
map.title = 程序地图
map.hint = 每个屏幕从哪里打开 · 最多三层

## ── 代码签名政策（公开页） ─────────────────────────
signing.title = 代码签名政策
signing.sub = 我们签什么，不签什么，密钥在哪里，以及如何自行验证。
signing.status_title = 今日状态
signing.status_body = 本政策在开始签名之前发布，尚未签署任何构建。首个已签名构建将在其发布页连同哈希一起公告。
signing.status_until = 在此之前以便携 ZIP 归档分发，绝不以未签名的安装程序分发。
signing.build_title = 构建
signing.build_body = 所有构建在 GitHub Actions 上，从发布说明中指定的提交开始。没有本地构建，签名前绝不重新构建。
signing.key_title = 密钥
signing.key_body = 我们不持有任何签名密钥。密钥保存在 SignPath Foundation 的硬件安全模块中，每次签名都需要维护者明确批准。
signing.signed_title = 已签名
signing.signed_installer = 安装程序
signing.signed_store = 商店包
signing.unsigned_title = 不签名
signing.unsigned_db = 签名数据库 — 使用独立的 Ed25519 密钥签名，因为其更新远比程序频繁。
signing.unsigned_tools = tools 目录下的一切 — 以 MIT 许可按源码分发，绝不作为二进制。
signing.verify_title = 验证
signing.verify_body = 每个构建的 SHA-256 都会在签名前后发布在发布页上。你可以在 Windows 上自行验证：
signing.verify_must = Status 必须为 Valid，发布者必须是 SignPath Foundation。如果哈希与公布的不一致，请勿运行该文件。
signing.revoke_title = 撤销
signing.revoke_body = 如果从未经审查的代码构建了版本，我们会撤销它，并首先公告撤销。
signing.credit = 免费代码签名由 SignPath.io 提供，证书由 SignPath Foundation 提供。
signing.updated = 最后更新

## ── 术语表 —— 每个术语一行 ─────────────
tip.confirmed = 其哈希在威胁数据库中。不是猜测。
tip.high_risk = 有强烈的危险迹象，哈希尚未确认。
tip.suspicious = 看起来奇怪：名称、位置或行为。尚未核实。
tip.pua = 不是病毒，但绕过了另一个程序的保护。
tip.signature = 已知文件的哈希。更新它可保持防护最新。
tip.behavior = 观察程序做了什么，而不是它的名字。
tip.proactive = 在行为发生前阻止它，而不是事后发现。
tip.pause = 我们只静音窗口。扫描和拦截继续。
tip.turn_off = 不扫描也不拦截。此期间你的设备没有保护。
tip.exception = 按你的决定放宽一条规则。其余不变。
tip.expert_mode = 显示永久和整条规则的选项。无需密码，也不提问。
tip.wsh = Windows 脚本宿主。恶意软件常用的入口。
tip.powershell = Windows 中的命令工具。来自危险源时默认受限。
tip.macros = Office 文档内的命令。未签名的文档中不会运行。
tip.amsi = Windows 接口，让我们在脚本运行前看到它。
tip.wdac = Windows 策略，阻止某个特定文件运行。
tip.usb_trust = 我们第一次见到它。打开前先扫描。
tip.tamper = 无法通过修改其文件或结束其服务来关闭 Odina。
tip.hash = 代表整个文件的字符串。一个字节改变，全部改变。
tip.heuristic = 被测量和权衡的信号。单独从不足够。

## ── 设置、隐私和语言 ─────────────────────────────
settings.title = 设置
settings.sub = 四个部分。除罕见项外，没有藏在“高级”后面的东西。
settings.appearance = 外观
settings.appearance.body = 浅色或深色。默认跟随系统。
settings.theme_light = 浅色
settings.theme_dark = 深色
settings.theme_system = 跟随系统
settings.behavior = 行为
settings.startup = 随 Windows 启动
settings.startup.body = 从开机起在后台运行。你可以关闭它。
settings.notifications = 通知
settings.notifications.body = 每天最多三条。最重要的优先。
settings.updates = 更新
settings.updates.body = 威胁数据库自动更新，程序手动更新。
settings.about = 关于
settings.version = 版本
settings.database = 威胁数据库
settings.database.date = 最后更新

privacy.title = 隐私
privacy.sub = 什么会离开你的设备，什么永远不会。没有例外。
privacy.telemetry = 零遥测
privacy.telemetry.body = 无统计、无设备标识、无指纹。不会自动发送任何内容。
privacy.local = 一切都在本地
privacy.local.body = 扫描、决定和日志都在你的磁盘上。
privacy.report = 报告由你决定
privacy.report.body = 只有你选择发送时，报告才会发出。
privacy.search = 按需搜索
privacy.search.body = 用你的语言打开搜索引擎。只发送术语，绝不发送你的文件。
privacy.log = 日志
privacy.log.body = 本地，可用你的语言阅读。你随时可以清除。
privacy.log_clear = 清除日志

lang.title = 语言
lang.sub = 十种语言。翻译立即生效，字体跟随你的语言。
lang.current = 当前语言
lang.rtl = 从右到左书写
lang.ltr = 从左到右书写
lang.review = 母语审校
lang.review.body = 未经母语者审校的语言会被暂停，不会发布。

home.walls = 三道防线
home.action.scan = 立即扫描
home.action.pause = 暂停
home.action.expert = 专家模式
home.action.why = 为什么？
home.goto.quarantine = 隔离区
home.goto.alerts = 通知中心
home.goto.settings = 设置
home.goto.proactive = 主动防护
home.quarantine_count = 隔离区有 { $count } 个文件
home.alerts_count = 有 { $count } 条通知值得一看
home.proactive_summary = 九项活动，分为三组
proactive.title = 主动防护
proactive.sub = 在此允许某项活动不会绕过文件扫描或防火墙规则。
proactive.tier.mandatory = 极高风险 · 自动阻止
proactive.tier.recommended = 中等风险 · 建议阻止
proactive.tier.optional = 较低风险 · 可选阻止
proactive.surface.office_child = Office 启动命令解释器
proactive.surface.office_child.tip = 仅限 Word 或 Excel 启动 cmd、PowerShell 或 mshta，不是所有 Office 子进程。
proactive.surface.wmi_subscription = 永久 WMI 订阅
proactive.surface.wmi_subscription.tip = 来自不可信来源的新永久事件订阅，不包括普通 WMI 查询。
proactive.surface.system_tools = 系统工具处理不可信输入
proactive.surface.system_tools.tip = mshta、certutil 或 regsvr32 处理来自临时目录或互联网的不可信输入。
proactive.surface.scheduled_task = 计划任务
proactive.surface.scheduled_task.tip = 创建或更改任务。阻止此操作不会停止已有任务。
proactive.surface.com_registration = COM 组件注册
proactive.surface.com_registration.tip = 更改当前用户的 COM 注册，不是禁用 Windows COM。
proactive.surface.registry_autorun = 注册表启动项
proactive.surface.registry_autorun.tip = 在注册表的 Run 或 RunOnce 下添加或更改启动项。
proactive.surface.service_driver = 安装服务和驱动程序
proactive.surface.service_driver.tip = 创建服务或安装新驱动程序，不停止正在运行的服务和驱动程序。
proactive.surface.inbound_smb_rdp = 入站 SMB 和 RDP
proactive.surface.inbound_smb_rdp.tip = 新建入站共享或远程桌面连接。允许此项不会打开端口。
proactive.surface.wsh_script = Windows 脚本
proactive.surface.wsh_script.tip = 通过 WSH 启动 .js、.vbs 或 .wsf。其他脚本检查仍然有效。
proactive.state.blocked = 已阻止
proactive.state.allowed = 已允许
proactive.state.contained = 暂时阻止
proactive.state.off = 未执行
proactive.saved_until = { $rule } · 例外保留至 { $until }
proactive.user_wsh_block = 你设置的 Windows 脚本阻止规则优先于此例外。
proactive.demo.title = 遏制模拟
proactive.demo.note = 仅在本地模拟。不扫描文件，不更改 Windows 设置。
proactive.demo.mode = 模拟保护模式
proactive.demo.ready = 扫描确认感染后，九项活动会被阻止，直到修复成功。
proactive.demo.start = 模拟已确认的感染
proactive.demo.scan = 正在扫描已确认的感染 · 九项活动已暂时阻止。
proactive.demo.clean = 正在清理 · 扫描已完成，仍保持阻止。
proactive.demo.repair = 正在修复 · 清理已完成，仍保持阻止。
proactive.demo.cleaned = 清理成功
proactive.demo.repaired = 修复成功
proactive.demo.restored = 修复成功。临时阻止已解除，之前的选择再次生效。
proactive.demo.off = 你已关闭保护。不会启动遏制，也不会更改你的选择。
proactive.demo.fail = 步骤失败
proactive.demo.failed = 此步骤未成功。将继续阻止，直到恢复成功。
alerts.title = 通知中心
alerts.sub = 值得一看的一切都在一处。没有弹窗。
alerts.empty = 现在没有任何值得一看的内容。
exception.detail.title = 例外 —— 一条规则
exception.detail.sub = 改变什么，持续多久，以及如何恢复。
map.node.home = 主页
map.node.quarantine = 隔离区
map.node.proactive = 主动防护
map.edge.from_home = 来自主页
map.edge.from_alert = 来自通知
map.legend = 实线是永久入口 · 虚线需要上下文

## ── 按需解释 ─────────────────────────────
help.title = 按需解释
help.hint = 悬停一行 · 点击展开完整搜索，用你的语言。
help.tip.title = 即时提示
help.tip.body = 一个带下划线的词，一行不超过 90 个字符。
help.guide.title = 更长的指南
help.guide.body = 提示里的按钮打开，在你知道的引擎里阅读。
help.guide.cta = 打开搜索
help.engine.title = 引擎跟随语言和地点
help.col.lang = 语言
help.col.engine = 引擎
help.col.why = 原因
help.why.default = 对该语言覆盖最广
help.why.zh = 中国境内的参照，Google 在那里受限
help.why.ja = 优先日文页面，而非译文
help.why.ru = 比其它引擎更懂俄语词形
help.query.note = 只发送术语。不发文件名、路径或设备标识。
path.copy_hint = 复制
nav.preview = 界面预览

## ── التنبيهات والشارات ────────────────────────────
alert.confirmed.cta = 打开隔离区
badge.confirmed = 已确认
badge.high = 高度可疑
badge.low = 可疑
badge.pua = 中等风险
button.off = 关
button.on = 开

## ── الإيقاف المؤقت ────────────────────────────
lvl.app = 应用状态
lvl.d10 = { $n } 分钟
lvl.d30 = { $n } 分钟
lvl.d60 = 1 小时
lvl.dpeak = 直到高峰时段结束
lvl.dreboot = 直到重启
lvl.hint = 我们告知，由你决定。
lvl.no_permanent = 没有永久暂停。最长到重启为止。
lvl.state_off = 保护已关闭
lvl.state_off_sub = 至 14:30 · 你的决定
lvl.title = 关闭 —— 两个层级，不是一个
lvl.what_continues = 继续的部分
lvl.what_stops = 停止的部分
lvl1.app = 运行 · 扫描 · 拦截 · 不打扰
lvl1.body = 无窗口、无声音。检查与拦截继续。
lvl1.cont = 三道防线
lvl1.name = 静音通知
lvl1.stops = 仅提示
lvl2.body = 不检查、不拦截。设备无防护。
lvl2.name = 停止防护
lvl2.resp = 这段时间由你负责。
lvl2.warn = 我们停止防护，不停止告知。

## ── دستور الواجهة — القواعد الخمس ────────────────────────────
notes.1b = 立即改变方向与布局；比例与哈希中的数字保持拉丁数字。
notes.1t = 语言切换：
notes.2b = 允许/阻止保持固定逻辑顺序；RTL 只翻转视觉位置，不改变角色。
notes.2t = 按钮：
notes.3b = 此处使用系统字体；CDN 字库下载后无需重启即切换为 Noto。
notes.3t = 中文：
notes.4b = 本文件从同级 fonts/vendor/ 加载字体——请用本地服务器打开，不要用 file://。
notes.4t = 字体：
notes.5b = 文字标志、托盘图标与错误状态——另行评审（R3）。
notes.5t = 本预览未包含：

## ── صندوق العزل ────────────────────────────
q.reason1 = 已确认威胁
q.reason2 = 破解工具
quarantine.opened_from = 从通知和主页上的计数进入——绝不从隐藏菜单。
quarantine.restore = 恢复

## ── دستور الواجهة — القواعد الخمس ────────────────────────────
sec.alerts = 2 · 四种提示
sec.alerts.hint = 两个按钮 + “为什么？” · 颜色不单独传达信息
sec.color = 4 · 调色板（WCAG 实测）
sec.color.hint = 每个颜色都标注其与当前背景的对比度
sec.fonts = 5 · 各语言字体
sec.fonts.hint = 仅 400 与 600 · 系统字体作为即时备选
sec.main = 1 · 主界面
sec.main.hint = ≤ 8 个元素 · 3 道防线 · 2 个按钮
sec.notes = 6 · 评审说明
sec.usb = 3 · USB 提示
sec.usb.hint = 单一进度条 · 单一阶段

## ── الهوية ────────────────────────────
shield.sub = { $n } 道防线运行中 · 威胁库已更新

## ── الشرح عند الطلب ────────────────────────────
tips.body = 将鼠标移到带下划线的词上查看一行说明 · 单击可用你的语言打开解释搜索（Google · 百度 · Yandex）。
tips.legend = 按需查看说明：

## ── دستور الواجهة — القواعد الخمس ────────────────────────────
trust.1d = 我们说发生了什么，而不是技术原理。一句话。没有理由就不说“完成”。
trust.1t = 清晰
trust.2d = 最多两个按钮。一个主要操作。无需任何设置。
trust.2t = 简洁
trust.3d = 无事件就不弹窗。每天最多 3 条通知。绝不使用模态框。
trust.3t = 安心
trust.4d = 每个按钮在按下前就写明后果。安全的选项最显眼。
trust.4t = 后果可知
trust.sub = 每个界面都用这四条衡量。冲突时：安全 → 不打扰 → 清晰 → 简洁 → 美观。

## ── USB ────────────────────────────
usb.stage = P0 · 根目录中的危险文件

## ── 数字 ─────────────────────────────
num.title = 数字
num.sub = 计数、通知和日志中的数字。
num.body = 选择你读得最快的形式。所有计算结果相同。
num.arabic = 0 1 2 3 4 5 6 7 8 9
num.hindi = ٠ ١ ٢ ٣ ٤ ٥ ٦ ٧ ٨ ٩（东阿拉伯数字）
num.arabic.name = 阿拉伯数字
num.hindi.name = 东阿拉伯数字
num.sample = 3 条通知 · 12 个文件
num.note = 立即生效，无需重启。

## ── 暂停提示 — 按需 ─────────────────────────────
tip.lvl1 = 无窗口、无通知、无声音，但检查与拦截在后台继续：被感染文件会被静默隔离，回来时给你看。
tip.lvl2 = 不检查、不拦截、不提醒。服务在运行但什么也不做，设备完全无防护。
tip.no_permanent = 最长的暂停到重启为止，这是我们唯一的限制。

## ── 第四条：视觉优先 ─────────────────────────────
visual.title = 视觉优先
visual.rule = 图先于句：屏幕做什么应当看得见，不用解释。
visual.why = 读得慢的人，或不读你语言的人，先懂形状和颜色，再读那行字。
visual.n1 = 每个元素都带形状、颜色和位置，不依赖一个词。
visual.n2 = 同一形状在每个屏幕上意思相同。
visual.n3 = 颜色区分安全与危险，文字说出原因。
visual.n4 = 没有文字的图标就是一次测试：若需解释，这图标就错了。
visual.n5 = 文字解释图，不重复图。
visual.sample = 悬停图标：一个词。点击：用你的语言完整解释。
trust.count = 五条，不是四条。
help.sample.title = 实时示例
