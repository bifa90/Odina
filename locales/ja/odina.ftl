# Odina Antivirus — 日本語 (ja)
# المستوى: T-B (مجتمعية) — تحتاج مراجعة ناطق أصلي قبل ترقيتها إلى T-A (§30.2)
# راجع كل سطر يدويًا قبل الاعتماد. لا ترجمة آلية للنصوص الأمنية (§3.9.1).

app.name = Odina
app.tagline = あなたのデジタルガーディアン
app.window.title = Odina — { $state }
state.protected = 保護されています
state.scanning = スキャン中…
state.attention = 確認が必要です
state.paused = 保護は一時停止中
state.untrusted = まだ確認されていません
wall.signatures.title = ファイルデータベース
wall.signatures.on = 有効
wall.signatures.off = 無効
wall.behavior.title = 詳細な動作検査
wall.behavior.on = 待機中
wall.behavior.off = 停止中
wall.proactive.title = 事前保護
wall.proactive.on = 有効
wall.proactive.off = 無効
action.scan_now = 今すぐスキャン
action.pause = 一時停止
action.expert_mode = エキスパートモード
action.language = 言語
action.settings = 設定
action.why = なぜ？
scan.quick = クイックスキャン
scan.full = 完全スキャン
scan.usb = このドライブをスキャン
scan.running = { $target } をスキャン中…
scan.percent = { $percent }%
scan.files_checked = 検査したファイル
scan.threats_found = 検出した脅威
scan.stop = 停止
scan.complete = スキャン完了
scan.clean_result = 有害なものは見つかりませんでした。
alert.confirmed.title = 既知の脅威を止めました
alert.confirmed.body = { $file } は確認済みの脅威で、隔離しました。
alert.suspicious.title = プログラムがファイルにアクセスしようとしています
alert.suspicious.body = { $file } がドキュメント内のファイルを変更しています。一時的に止めました。
alert.suspicious.note = 一時的に止めました。
alert.unsure.title = このプログラムは珍しいものです
alert.unsure.body = { $file } は新しく、奇妙な動作をします。ご存じですか？
alert.pua.title = クラックツールのようです
alert.pua.body = { $file } は有料ソフトのライセンス保護を回避します。端末で何を動かすかはあなたが決めます。
alert.pua.allowed = それでも許可
alert.pua.quarantine = 脇へ移動
button.allow = 許可
button.block = ブロック
button.restore = 復元
button.remove_forever = 完全に削除
usb.detected = 新しいドライブを検出 — 先に確認します
usb.scanning = { $name } を確認中…
usb.priority_done = 危険なファイルを確認しました
usb.trust.new = 新しいドライブ
usb.trust.scanning = 確認中
usb.trust.trusted = 記憶済み
usb.trust.suspicious = 監視中
usb.trust.quarantined = ブロック済み
quarantine.title = 隔離した項目
quarantine.reason = 隔離した理由
quarantine.restore_note = 元の場所は保持されています。
quarantine.retention = { $days } 日間保持
pause.title = 保護を一時停止
pause.duration.10 = 10 分
pause.duration.30 = 30 分
pause.duration.60 = 1 時間
pause.warning = { $until } まで保護されません。
pause.confirm = それでも一時停止
pause.badge = 通知を一時停止 · 保護は継続
error.generic = 問題が発生しました。何も変更していません。
error.tamper = Odina 自身のファイルが変更されました。修復が必要です。
error.offline = 更新はまだありません。保護は続きます。
update.available = 更新があります
update.ask = 更新を確認してダウンロードしますか？
update.db_age_warning = 脅威リストは { $days } 日経過しています。

## ── Path ──────────────────────────────────────────────
path.label = パス
path.copy = パスをコピー
path.open_folder = フォルダーを開く
trust.title = 信頼の基盤
word.isolated = 隔離済み
word.temporary = 一時的に
tip.quarantine = 暗号化された保管庫へ移動しました。元の場所は保持され、復元できます。
tip.suspended = 今は停止しています。許可した場合のみ再び動作します。
quarantine.hint = あなたが決めるまですべて残ります · 自動削除はありません
quarantine.keep = 保持する
quarantine.delete = 完全に削除
quarantine.delete_note = 元に戻せません。他のコピーはありません。
quarantine.cleanup_title = 隔離して { $days } 日
quarantine.cleanup_choose = 選択したものを削除
error.false_positive = このファイルは誤りでした — 修正しました。
error.locked = 隔離できませんでした：ファイルが開いています。

## ── Turn protection off (user's explicit decision) ────────
pause.confirm_disable = 保護を { $minutes} 分間オフにする
disable.title = 保護をオフにする
disable.warning_title = 重大なリスク
disable.body = 通知を止めるだけでなく、保護を停止します。ファイルの検査も脅威の阻止も行われません。端末は完全に無防備になります。
disable.responsibility = これはあなたの判断であり、その間の責任はあなたにあります。
disable.choose = 期間を選ぶ
disable.duration.peak = ピーク時間の終了まで
disable.duration.30 = 30 分
disable.duration.60 = 1 時間
disable.duration.until_reboot = 再起動まで
disable.refused = 現在は保護をオフにできません
disable.refused_quarantine = 未処理の隔離ファイルがあります。先に確認してください。
disable.refused_scanning = スキャン中です。終了を待つか停止してください。
disable.refused_active_threat = 未対応の脅威が残っています。その間は保護を停止しません。
state.unprotected = 保護はオフです
state.unprotected_until = { $until} まで保護オフ
state.unprotected_badge = 無防備 — あなたの判断
action.turn_off = 保護をオフにする
action.turn_on = 保護を戻す
disable.re_enable_note = いつでも保護を戻せます — パスワード不要。
disable.reminder = 保護が自動で戻るまで残り { $minutes} 分。
pause.resume_report = 通知を止めていた間に { $count} 件を隔離しました

## ── 外科的例外 (D45 · §4.8) ─────────────────────────
exception.title = 例外
exception.sub = 業務で 1 つのルールを緩める必要があるなら、その 1 つだけを許可します。他はそのままです。
exception.rule.a1.wsh = Windows スクリプト
exception.rule.a1.wsh.desc = .vbs と .js は、既知の発行者の署名がない限り制限付きで実行されます。
exception.rule.a1.powershell = PowerShell コマンド
exception.rule.a1.powershell.desc = PowerShell は制限モードで動作します。許可しても Windows Script Host は開きません。
exception.rule.a1.macros = Office マクロ
exception.rule.a1.macros.desc = 署名のない文書はマクロを実行できません。許可してもスクリプトには影響しません。
exception.rule.a3.downloads = ダウンロードの疑い
exception.rule.a3.downloads.desc = ダウンロード内の新しいファイルは、晴れるまで疑わしいと見なされます。
exception.rule.a4.periodic = 定期スキャン
exception.rule.a4.periodic.desc = 起動項目と重要なレジストリキーを 2 時間ごとに静かに検査します。
exception.rule.a6.doubt = 疑わしいときの慎重さ
exception.rule.a6.doubt.desc = 疑わしいときはまず止めてから尋ねます。緩めることはできますが、完全な許可はできません。
exception.state.on = 有効
exception.state.relaxed_until = { $until } まで緩和
exception.state.allowed_until = { $until } まで許可
exception.state.allowed_permanent = 永久に許可 — あなたの判断
exception.consequence.relax = { $rule } は { $until } まで通知なしで動作し、その後戻ります。その間は何もブロックされません。
exception.consequence.allow = { $rule } は { $until } まで制限なしで動作し、その後戻ります。他は変わりません。
exception.consequence.permanent = { $rule } は取り消すまで許可されたままです。これはホーム画面に表示され続けます。
exception.action.relax_10 = 10 分間緩和
exception.action.relax_30 = 30 分間緩和
exception.action.relax_60 = 1 時間緩和
exception.action.allow = 許可
exception.action.revoke = 取り消す
exception.action.revoke_all = すべての例外を取り消す
exception.action.advanced = 詳細オプション
exception.badge = { $rule } を許可 · { $until } まで
exception.badge_permanent = { $rule } を永久に許可 · あなたの判断
exception.badge_many = { $count } 件のルールがあなたの判断で許可 · 詳細
exception.until_reboot = 再起動まで
exception.permanent_note = 永久の例外は自然には終わりません。取り消すまでホーム画面に表示されます。
exception.revoke_note = いつでも取り消せます — ワンクリック、パスワード不要。
exception.expert_note = エキスパートモードで「永久」と「ルール全体」が現れます。パスワードも理由の質問もありません。
exception.expired_report = 例外が終了しました。その間に { $count } 件のファイルが疑わしく見えました。今すぐ確認してください。
exception.expired_clean = 例外が終了しました。確認が必要なものは現れませんでした。
exception.refused.not_allowed = このルールは完全に許可できません — 緩めることだけが可能です。
exception.refused.above_ceiling = このルールの上限は緩和です。
exception.refused.no_permanent = このルールは永久の例外にできません。
exception.refused.expert_only = ルール全体の永久例外にはエキスパートモードが必要です。
exception.refused.veto_stands = このファイルは確認済みです。例外はルールを緩めるもので、ファイルを晴らすものではありません。

exception.live_note = あなたの判断 · 他はすべて有効なまま · いつでも取り消せます
exception.lines_title = 例外にできない 4 つ:
exception.lines_body = 第 1 の壁の判定（例外中でも確認済みは隔離）· 改ざん防止 · 2 番目のルール（WSH の例外で PowerShell は開かない）·「高リスク」以上。取り消しは常にワンクリック、パスワード不要。

## ── アプリの地図とナビゲーション (憲章 §6أ.3) ─────────────────
nav.home = ホーム
nav.quarantine = 隔離ボックス
nav.proactive = 先制的保護
nav.alerts = 通知センター
nav.settings = 設定
nav.language = 言語
nav.privacy = プライバシー
back = 戻る
menu.more = その他
menu.advanced = 詳細オプション
map.title = アプリの地図
map.hint = 各画面への入口 · 最大 3 階層

## ── コード署名ポリシー（公開ページ） ─────────────────────────
signing.title = コード署名ポリシー
signing.sub = 何に署名し、何に署名しないか、鍵はどこにあり、自分でどう検証するか。
signing.status_title = 本日の状態
signing.status_body = このポリシーは署名開始前に公開されており、まだどのビルドにも署名していません。最初の署名済みビルドはハッシュとともにリリースページで告知します。
signing.status_until = それまではポータブル ZIP として配布し、未署名のインストーラーとしては絶対に配布しません。
signing.build_title = ビルド
signing.build_body = ビルドはすべて GitHub Actions 上で、リリースノートに記載されたコミットから行います。ローカルビルドはなく、署名前に再ビルドすることもありません。
signing.key_title = 鍵
signing.key_body = 署名鍵は保持しません。鍵は SignPath Foundation の HSM にあり、署名のたびにメンテナーの明示的な承認が必要です。
signing.signed_title = 署名するもの
signing.signed_installer = インストーラー
signing.signed_store = ストアパッケージ
signing.unsigned_title = 署名しないもの
signing.unsigned_db = 署名データベース — プログラムよりはるかに頻繁に更新されるため、別の Ed25519 鍵で署名します。
signing.unsigned_tools = tools 配下のすべて — MIT ライセンスでソースとして配布し、バイナリとしては配布しません。
signing.verify_title = 検証
signing.verify_body = 各成果物の SHA-256 は署名の前後にリリースページで公開します。Windows でご自身で検証できます：
signing.verify_must = Status が Valid であり、発行者が SignPath Foundation である必要があります。公開されたハッシュと異なる場合は、そのファイルを実行しないでください。
signing.revoke_title = 失効
signing.revoke_body = 未レビューのコードからビルドが作られた場合、それを失効させ、何よりも先に失効を告知します。
signing.credit = 無償のコード署名は SignPath.io、証明書は SignPath Foundation によるものです。
signing.updated = 最終更新

## ── 用語集 — 1 用語 1 行 ─────────────
tip.confirmed = ハッシュが脅威データベースにあります。推測ではありません。
tip.high_risk = 危険の強い兆候がありますが、ハッシュは未確認です。
tip.suspicious = 名前・場所・挙動が奇妙です。未検証です。
tip.pua = ウイルスではありませんが、別のプログラムの保護を迂回します。
tip.signature = 既知ファイルのハッシュ。更新すると保護が最新になります。
tip.behavior = プログラムの名前ではなく、動作を見ます。
tip.proactive = 起きた後に見つけるのではなく、起きる前に止めます。
tip.pause = ウィンドウを静めるだけです。検査と阻止は続きます。
tip.turn_off = 検査も阻止もしません。その間デバイスは無防備です。
tip.exception = あなたの判断で 1 つのルールを緩めます。他は変わりません。
tip.expert_mode = 永久とルール全体の選択肢を表示します。パスワードも質問もありません。
tip.wsh = Windows のスクリプトホスト。マルウェアの侵入経路としてよく使われます。
tip.powershell = Windows のコマンドツール。危険な出所からは既定で制限されます。
tip.macros = Office 文書内の命令。署名のない文書では実行されません。
tip.amsi = スクリプトが実行される前に見えるようにする Windows の仕組みです。
tip.wdac = 特定のファイルの実行を止める Windows のポリシーです。
tip.usb_trust = 初めて見るデバイスです。開く前に検査します。
tip.tamper = ファイルの改変やサービスの停止では Odina を切れません。
tip.hash = ファイル全体を表す文字列です。1 バイト変われば全部変わります。
tip.heuristic = 測定して重み付けする信号です。単独では十分ではありません。

## ── 設定・プライバシー・言語 ─────────────────────────────
settings.title = 設定
settings.sub = 4 つのセクション。「詳細」の裏に隠すのはまれな項目だけです。
settings.appearance = 外観
settings.appearance.body = ライトまたはダーク。既定ではシステムに従います。
settings.theme_light = ライト
settings.theme_dark = ダーク
settings.theme_system = システムに従う
settings.behavior = 動作
settings.startup = Windows と同時に起動
settings.startup.body = 起動時からバックグラウンドで動きます。オフにできます。
settings.notifications = 通知
settings.notifications.body = 1 日 3 件まで。重要なものが先です。
settings.updates = 更新
settings.updates.body = 脅威データベースは自動、プログラムは手動です。
settings.about = このアプリについて
settings.version = バージョン
settings.database = 脅威データベース
settings.database.date = 最終更新

privacy.title = プライバシー
privacy.sub = 端末から出るもの、決して出ないもの。例外はありません。
privacy.telemetry = テレメトリゼロ
privacy.telemetry.body = 統計も端末 ID も指紋もありません。自動送信は一切しません。
privacy.local = すべてローカル
privacy.local.body = 検査も判断も記録も、あなたのディスク上にあります。
privacy.report = レポートはあなたの判断
privacy.report.body = 送ると選んだときだけ送信します。
privacy.search = 必要時の検索
privacy.search.body = あなたの言語で検索エンジンを開きます。送るのは用語だけで、ファイルは送りません。
privacy.log = 記録
privacy.log.body = ローカルで、あなたの言語で読めます。いつでも消去できます。
privacy.log_clear = 記録を消去

lang.title = 言語
lang.sub = 10 言語。翻訳はすぐ反映され、フォントは言語に従います。
lang.current = 現在の言語
lang.rtl = 右から左に書きます
lang.ltr = 左から右に書きます
lang.review = 母語話者による校閲
lang.review.body = 母語話者が校閲していない言語は保留され、出荷されません。

home.walls = 3 つの壁
home.action.scan = 今すぐ検査
home.action.pause = 一時停止
home.action.expert = エキスパートモード
home.action.why = なぜ？
home.goto.quarantine = 隔離ボックス
home.goto.alerts = 通知センター
home.goto.settings = 設定
home.goto.proactive = 先制的保護
home.quarantine_count = 隔離ボックスに { $count } 件
home.alerts_count = 確認に値する通知 { $count } 件
home.proactive_summary = 3 グループ、9 種類の操作
proactive.title = 先制的保護
proactive.sub = ここで許可しても、ファイル検査やファイアウォールの規則は適用されます。
proactive.tier.mandatory = 非常に高いリスク · 自動ブロック
proactive.tier.recommended = 中程度のリスク · ブロック推奨
proactive.tier.optional = 比較的低いリスク · 任意でブロック
proactive.surface.office_child = Office からのコマンド起動
proactive.surface.office_child.tip = Word や Excel による cmd、PowerShell、mshta の起動が対象です。すべての子プロセスではありません。
proactive.surface.wmi_subscription = WMI の永続サブスクリプション
proactive.surface.wmi_subscription.tip = 信頼できない発行元による永続イベント購読の新規作成が対象です。通常の WMI クエリは含みません。
proactive.surface.system_tools = 未信頼の入力を扱うシステムツール
proactive.surface.system_tools.tip = 一時フォルダーやインターネットの未信頼データを扱う mshta、certutil、regsvr32 が対象です。
proactive.surface.scheduled_task = スケジュールされたタスク
proactive.surface.scheduled_task.tip = タスクの作成・変更が対象です。既存のタスクは停止しません。
proactive.surface.com_registration = COM コンポーネント登録
proactive.surface.com_registration.tip = ユーザー単位の COM 登録の変更が対象です。Windows の COM は無効にしません。
proactive.surface.registry_autorun = レジストリの自動起動項目
proactive.surface.registry_autorun.tip = レジストリの Run または RunOnce への項目追加や変更が対象です。
proactive.surface.service_driver = サービスとドライバーの追加
proactive.surface.service_driver.tip = サービスの作成やドライバーのインストールが対象です。稼働中のものは停止しません。
proactive.surface.inbound_smb_rdp = SMB・RDP の受信接続
proactive.surface.inbound_smb_rdp.tip = 共有やリモートデスクトップへの新しい受信接続が対象です。許可してもポートは開きません。
proactive.surface.wsh_script = Windows スクリプト
proactive.surface.wsh_script.tip = WSH による .js、.vbs、.wsf の起動が対象です。他のスクリプト検査は引き続き行われます。
proactive.state.blocked = ブロック
proactive.state.allowed = 許可
proactive.state.contained = 一時ブロック
proactive.state.off = 適用なし
proactive.saved_until = { $rule } · 例外は { $until } まで保存されています
proactive.user_wsh_block = 選択した Windows スクリプトのブロックがこの例外より優先されます。
proactive.demo.title = 封じ込めのシミュレーション
proactive.demo.note = ローカルのシミュレーションです。ファイル検査や Windows 設定の変更は行いません。
proactive.demo.mode = シミュレーションの保護モード
proactive.demo.ready = 検査で感染を確認したら、修復が成功するまで 9 種類の操作をすべてブロックします。
proactive.demo.start = 確認済みの感染を模擬
proactive.demo.scan = 確認済みの感染を検査中 · 9 種類の操作を一時ブロックしています。
proactive.demo.clean = 駆除中 · 検査は完了しました。ブロックは継続します。
proactive.demo.repair = 修復中 · 駆除は完了しました。ブロックは継続します。
proactive.demo.cleaned = 駆除成功
proactive.demo.repaired = 修復成功
proactive.demo.restored = 修復に成功しました。一時ブロックを解除し、元の設定を再び適用します。
proactive.demo.off = ユーザーが保護を無効にしています。封じ込めは開始せず、設定も変更しません。
proactive.demo.fail = 手順の失敗
proactive.demo.failed = この手順は失敗しました。復旧が成功するまでブロックを継続します。
alerts.title = 通知センター
alerts.sub = 確認に値するすべてが一か所に。ポップアップはありません。
alerts.empty = 今は確認に値するものはありません。
exception.detail.title = 例外 — 1 つのルール
exception.detail.sub = 何が変わり、いつまで続き、どう戻るか。
map.node.home = ホーム
map.node.quarantine = 隔離ボックス
map.node.proactive = 先制的保護
map.edge.from_home = ホームから
map.edge.from_alert = 通知から
map.legend = 実線は常設の入口 · 点線は文脈が必要

## ── 必要時の説明 ─────────────────────────────
help.title = 必要時の説明
help.hint = ホバーで一行 · クリックで検索、あなたの言語で。
help.tip.title = 即時のヒント
help.tip.body = 下線の一語と、90 文字以内の一行。
help.guide.title = より長いガイド
help.guide.body = ヒント内のボタンで開き、よく知る検索エンジンで読みます。
help.guide.cta = 検索を開く
help.engine.title = エンジンは言語と場所に従う
help.col.lang = 言語
help.col.engine = エンジン
help.col.why = 理由
help.why.default = この言語で最も網羅的
help.why.zh = 中国国内の基準。Google は制限される
help.why.ja = 翻訳ではなく日本語ページを優先する
help.why.ru = ロシア語の語形を他より理解する
help.query.note = 送るのは用語だけ。ファイル名もパスも端末 ID も送りません。
path.copy_hint = コピー
nav.preview = 画面プレビュー

## ── التنبيهات والشارات ────────────────────────────
alert.confirmed.cta = 隔離箱を開く
badge.confirmed = 確認済み
badge.high = 強い疑い
badge.low = 疑わしい
badge.pua = 中リスク
button.off = オフ
button.on = オン

## ── الإيقاف المؤقت ────────────────────────────
lvl.app = アプリ
lvl.d10 = { $n } 分
lvl.d30 = { $n } 分
lvl.d60 = 1 時間
lvl.dpeak = ピーク時間の終了まで
lvl.dreboot = 再起動まで
lvl.hint = 知らせます。決めるのはあなたです。
lvl.no_permanent = 恒久的な停止はありません。最長でも再起動まで。
lvl.state_off = 保護はオフです
lvl.state_off_sub = 14:30 まで · あなたの判断
lvl.title = オフにする — 1 段階ではなく 2 段階
lvl.what_continues = 続くもの
lvl.what_stops = 止まるもの
lvl1.app = 動作 · 検査 · 阻止 · 尋ねない
lvl1.body = 窓も音もなし。検査と遮断は続きます。
lvl1.cont = 3 つの壁すべて
lvl1.name = 通知を静かに
lvl1.stops = 通知だけ
lvl2.body = 検査も遮断もなし。端末は無防備です。
lvl2.name = 保護を停止
lvl2.resp = その間の責任はあなたにあります。
lvl2.warn = 保護を止め、知らせることは止めません。

## ── دستور الواجهة — القواعد الخمس ────────────────────────────
notes.1b = 方向とレイアウトが即座に変わります。比率やハッシュの数字はラテン数字のままです。
notes.1t = 言語切替：
notes.2b = 許可/ブロックの論理順は固定。RTL は視覚位置だけを反転します。
notes.2t = ボタン：
notes.3b = ここではシステムフォントで表示。CDN フォント取得後は再起動なしで Noto になります。
notes.3t = 日本語：
notes.4b = 同じ階層の fonts/vendor/ から読み込みます。file:// ではなくローカルサーバーで開いてください。
notes.4t = フォント：
notes.5b = ロゴタイプ、タスクトレイのアイコン、エラー状態 — 別途レビュー（R3）。
notes.5t = このプレビューに無いもの：

## ── صندوق العزل ────────────────────────────
q.reason1 = 確認済みの脅威
q.reason2 = クラックツール
quarantine.opened_from = 通知とホームの件数から開きます。隠れたメニューからは開きません。
quarantine.restore = 復元

## ── دستور الواجهة — القواعد الخمس ────────────────────────────
sec.alerts = 2 · 4 つの通知
sec.alerts.hint = ボタン 2 つ + 「なぜ？」 · 色だけで意味を伝えない
sec.color = 4 · 配色（WCAG 実測）
sec.color.hint = 各色に現在の背景とのコントラスト比を表示
sec.fonts = 5 · 言語ごとのフォント
sec.fonts.hint = 400 と 600 のみ · システムフォントを即時代替に
sec.main = 1 · メイン画面
sec.main.hint = ≤8 要素 · 3 つの壁 · ボタン 2 つ
sec.notes = 6 · レビュー用メモ
sec.usb = 3 · USB 通知
sec.usb.hint = 進捗バー 1 本 · 段階 1 つ

## ── الهوية ────────────────────────────
shield.sub = { $n } つの壁が稼働中 · 脅威リストは最新

## ── الشرح عند الطلب ────────────────────────────
tips.body = 下線のある語にマウスを置くと 1 行表示 · クリックで母語の解説検索を開きます（Google · Yahoo! JAPAN · Yandex）。
tips.legend = 必要なときに説明：

## ── دستور الواجهة — القواعد الخمس ────────────────────────────
trust.1d = 技術ではなく、何が起きたかを伝えます。一文で。「完了」だけは言いません。
trust.1t = 明快
trust.2d = ボタンは最大 2 つ。主要操作は 1 つ。設定は不要。
trust.2t = 簡潔
trust.3d = 問題がなければ窓は出しません。通知は 1 日 3 件まで。モーダルは使いません。
trust.3t = 安心
trust.4d = 各ボタンは押す前に結果を書きます。安全な選択が目立つ位置に。
trust.4t = 結果がわかる
trust.sub = すべての画面をこの 4 つで測ります。衝突時：安全 → 邪魔しない → 明快 → 簡潔 → 美しさ。

## ── USB ────────────────────────────
usb.stage = P0 · ルート内の危険なファイル

## ── 数字 ─────────────────────────────
num.title = 数字
num.sub = 件数・通知・記録に出る数字。
num.body = いちばん速く読める形を選べます。計算はどれも同じです。
num.arabic = 0 1 2 3 4 5 6 7 8 9
num.hindi = ٠ ١ ٢ ٣ ٤ ٥ ٦ ٧ ٨ ٩
num.arabic.name = アラビア数字
num.hindi.name = 東アラビア数字
num.sample = 通知 3 件 · ファイル 12 個
num.note = すぐ反映されます。再起動は不要です。

## ── 一時停止のヒント — 必要時 ─────────────────────────────
tip.lvl1 = 窓も通知も音もありませんが、検査と遮断は裏で続きます。感染ファイルは黙って隔離され、戻ったときに見せます。
tip.lvl2 = 検査も遮断も通知もありません。サービスは動きますが何もせず、端末は完全に無防備です。
tip.no_permanent = 最長の一時停止は再起動で終わります。それが唯一の制限です。

## ── 第四条：視覚優先 ─────────────────────────────
visual.title = 視覚優先
visual.rule = 文より先に絵。画面のすることを見せ、説明させない。
visual.why = ゆっくり読む人も、あなたの言語を読まない人も、行より先に形と色を理解します。
visual.n1 = すべての要素が形・色・位置を持ち、一語に頼りません。
visual.n2 = 同じ形はどの画面でも同じ意味です。
visual.n3 = 色が安全と危険を分け、文字が理由を名指しします。
visual.n4 = 言葉のないアイコンは試験です。説明が要るなら、そのアイコンが間違いです。
visual.n5 = 文字は絵を説明し、繰り返しません。
visual.sample = アイコンに乗せる：一語。クリック：あなたの言語で全文。
trust.count = 四つではなく五つ。
help.sample.title = 実際の例
