# DisplaySwitch 技术说明

**简体中文** · [English](TECHNICAL.en.md)

一句话概括技术选型：**Tauri 2（Rust 后端）+ Vue 3（Options API）+ Vite 前端，真正的切换动作交给内嵌的 [displayplacer](https://github.com/jakehilborn/displayplacer)**。选 Tauri 而不是 Electron，是为了小包体和系统 WebView；显示切换不自己拿 CoreGraphics 硬写，而是「读状态 → 摆角色 → 生成 spec」交给这个成熟的 CLI。

用户看得见的部分在 [README](../README.md)，这里只讲怎么实现的、以及为什么这么实现。

## 架构与托盘窗口

- **窗口**：label `main`，600×520 逻辑点，`decorations:false` + `transparent:true` + `alwaysOnTop` + `skipTaskbar` + `visible:false`（`src-tauri/tauri.conf.json`）。只剩一台显示器时宽度收成 380（`SINGLE_PANEL_WIDTH`）——宽度在托盘点击那一刻按 `LAST_DISPLAY_COUNT` 决定，而这个值每次 `get_state` 都会刷新，所以插拔之后不必重开 App 就能拿到对的宽度。
- **质感**：`window-vibrancy` 的 `apply_vibrancy(NSVisualEffectMaterial::HudWindow)`，配合圆角卡片就是 macOS 那套 HUD 毛玻璃。
- **不进程序坞**：`src-tauri/Info.plist` 里的 `LSUIElement = true` 让 LaunchServices 在进程登记这一步就把它当后台 Agent，没有 Dock 图标、不占 ⌘Tab，只剩菜单栏那枚图标；`setup` 里那次 `set_activation_policy(Accessory)` 是给不读 Info.plist 的 `cargo run` 裸二进制兜底（开发时 Dock 会闪一下再消失，属正常）。
- **失焦即隐藏**：`WindowEvent::Focused(false)` → `hide_panel_window()`。所以点面板外任意处、切到别的 App，面板自己就收了；顶栏 × 走 `hide_panel` 命令，效果相同。唯一的例外是快速切换：从点下场景那一刻起 `PANEL_LOCK` 上锁，见「外部调用：快速切换」。
- **⌘Q 退出**：前端 `mounted` 挂一个 `window` 上的 `keydown`（`App.vue` 的 `onKeydown`），命中 ⌘Q 就 `preventDefault` 后走 `quit_app`。之所以不用担心误退别的 App：面板一失焦就收起，所以面板可见的这段时间键盘焦点必定在 DS，按键只走 DS 的响应链，菜单栏上别人的「退出 XX ⌘Q」拿不到它（快速切换 `PANEL_LOCK` 上锁、面板强行留在屏幕上时是例外，但那时焦点在谁那儿 ⌘Q 就归谁，跟系统习惯一致）。观感上仍可能让人犹豫，但又不想常驻一行字占版面，所以照浏览器的做法做临时提示：`keydown` / `keyup` 维护 `cmdHeld`，单按 ⌘ 时居中浮出一枚深色胶囊 `quitHint`（「按 ⌘Q 键即可退出」），松开 ⌘ 就消失；`window` 的 `blur` 再兼一道，因为按着 ⌘ 切走 App 时 `keyup` 不会补发。胶囊 `pointer-events:none`，不挡点选，也不会跟着面板重新弹出时残留。
- **⌘1–⌘9 选场景**：序号就是 `presets` 的显示顺序（拖过的排在前面），第 10 个起不编号也不响应；设置视图和保存/改名输入框还在编辑时不接。键盘和鼠标走的是同一个 `applyPreset`，参数取自同一份 `presets`，没分支差异；但 `onKeydown` 开头先挡一次 `e.repeat`——macOS 的自动重复会把按住不放的 ⌘1 连发成几十次 keydown，等于几十遍 `apply_config`，显示器反复重排、闪得比鼠标点一下严重得多（鼠标没这条路径，两边体感不一致的根源在重复不在方法）。按住 ⌘ 时每个胶囊正上方浮出一枚深色小方框 `⌘n`（`.pill-kbd`）——绝对定位加 `bottom:calc(100%+6px)`，所以它不参与排版：按下那一下整行零重排，也不用像内联角标那样常年留位。第一个胶囊的方框正好落在「场景预设」那行字上，所以按住 ⌘ 期间把标题 `visibility:hidden`（不是 v-if，位置留着，松开原样回来，胶囊几何不变）。方框和悬停气泡占的是胶囊上方同一条带，所以按住期间把气泡隐掉（`tip && !cmdHeld`），并在松开 ⌘ 的 `keyup` 里补跑一次 `clampTip`——按住期间那次 `$nextTick` 夹位没走到，不补的话气泡可能出条边界。气泡仍然只有一行，只在描述末尾接一段更暗的「 · ⌘n」（`.tip-kbd`，`white-space:nowrap`，免得描述折行时把序号孤零零甩到第二行）；序号框只在按住 ⌘ 时出现、那时气泡是隐的，所以两处不会同屏重复。
- **切换互斥**：`switching` 这枚标志由 `pushDraft` 和 `applyPreset` 共用（分辨率、角色、场景三条路径都从这里走），一次切换从 `apply_config` 到末尾 `load()` 跑完之前，后来的点击和按键直接丢弃，`finally` 负责释放（`load()` 自己把异常包在 `try` 里吐成 `error`，不会抛出来把锁卡死）。不锁的话两次重排叠在一起、闪得更久，后一次 `load()` 还会把前一次的读数盖掉。键盘侧已经先挡了 `e.repeat`，这里是再兜一层：按住 ⌘1 不放、连点两下、切到一半再按，都只跑一次。
- **全局快捷键**：`tauri-plugin-global-shortcut`（只用 Rust 侧，不装 JS 插件、不加 capability，和自启动同一个路子）。前端在设置里对 `window` 挂一次 capture 阶段的 `keydown`（抢在 App 的 ⌘Q / ⌘1–9 之前，`preventDefault` + `stopPropagation`），把组合键串（`Command+Shift+D` 这种，修饰键名用 `Command`/`Control`/`Alt`/`Shift`，和 keyboard-types 的解析名对齐）交给 `set_hotkey`；后端先 `parse::<Shortcut>()` 校验格式，再 `unregister_all()` 注册新键，注册失败（组合键被别的 App 占用）就把旧键注册回去并返回错误，旧绑定不落盘。`settings.json` 里的 `hotkey` 字段空串读成 `None`；`setup` 里读设置恢复注册，所以手动删文件、或被占用，下次启动就是不注册，界面如实显示「未设置」。触发时先取托盘图标 `rect()` 算出锚点走 `toggle_panel_at`——面板可见就收起，否则贴着托盘弹出；拿不到托盘坐标（Linux 或异常）才回落主屏居中。前端侧的录入口子：只认字母、数字和 F1–F12 作主键；纯修饰键不提交、继续等下一个键；不含 ⌘/⌃/⌥ 的组合（裸键或仅 ⇧）拒绝并在说明位浮出「需包含 ⌘ / ⌃ / ⌥」；单按 Esc（不带修饰键）取消录入。
- **托盘**：`TrayIconBuilder::with_id("main-tray")` + `icon_as_template(true)`。左键弹出/收起，右键是原生菜单（显示面板 / 设置… / 退出）。原生菜单不会跟网页一起重渲染，所以语言变了要就地 `tray_by_id("main-tray").set_menu(Some(tray_menu(app)))` 重建一份。
- **「失焦即隐藏」延迟 300ms**：点托盘会让面板先失焦，原来的即时隐藏把窗口收掉，托盘的抬起事件随后到达、又被 `toggle_panel_at` 弹回来——第二下永远收不起。现在 `Focused(false)` 走 `schedule_blur_hide`：挂一枚 `BLUR_HIDE_PENDING` 标志并起 300ms 定时器，到点时标志没被抢掉才真隐；托盘点击和热键切换进来先 `swap` 抢标志，抢到就是「再点一下收起」，直接隐藏、不再弹出。自己 `hide()` 触发的失焦此时窗口已不可见，不再重复挂起；`PANEL_LOCK`（快速切换倒计时）期间整个分支跳过。
- **定位**：托盘点击回传的坐标是物理像素，按 `window.scale_factor()` 换算后再定位；窗口尺寸本身用逻辑点表达，避免 Retina 2x 下两套单位混用。
- **热插拔**：`setup` 里注册 `CGDisplayRegisterReconfigurationCallback`。合盖/开盖/插拔会连续触发多次回调，所以用一个自增序号做 800ms 防抖，静默期过后只 `emit("displays-changed")` 一次；前端 `listen` 到就重新 `load()`，面板重新拿到焦点时也会刷一次。
- **系统显示器设置入口**：「显示器」分组标题行右端一枚淡图标（`App.vue` 的 `.sys-btn`，`opacity:0.3`，指针进到这行升到 0.7、按到才全亮并染 accent，只有 tooltip 不带文字）。提示是自绘的 `.sys-tip` 而不是原生 `title`（原生跟着鼠标走、出现时机不稳），而且只能锚在图标下方——面板本体是 `overflow-y:auto` 的滚动容器，「显示器」这行紧贴顶边，气泡放上面会被裁掉。入口本身走 `open_display_settings`。DS 只负责「不弄乱排列」（见「应用配置：按角色生成 spec」），拖拽精调和色彩配置这类还是在系统面板里做，所以这个入口是逃生门不是功能。深链的 pane id 在 macOS 13 换过（13+ 是 `com.apple.Displays-Settings.extension`，12 及以前只认 `com.apple.preference.displays`），而 `open` 拿到不存在的 id 照样返回 0、只是把设置开到首页，没法靠返回码兜底，所以由 `SystemVersion.plist` 的 `ProductVersion` 主版本号选 id（纯文本 XML，直接读，不 fork `sw_vers`）。点完之后前端主动 `closePanel()`，不等那 300ms 失焦延迟隐藏，否则看着像点完卡了一下才收。

## Tauri 命令清单

前端 `src/api.js` 是唯一出口，浏览器预览时同名函数走 mock（见「前端双路径」）。

| 命令 | 参数 | 返回 | 作用 |
|---|---|---|---|
| `get_state` | — | `AppState` | 读当前显示器、角色、镜像组与可选档位；顺带刷新 `LAST_DISPLAY_COUNT` |
| `apply_config` | `config` | — | 按角色生成 displayplacer spec 并执行 |
| `save_preset` | `name`, `config` | — | 存一整套配置 |
| `list_presets` | — | `PresetEntry[]` | 列出全部预设（含未接入显示器的） |
| `delete_preset` | `name` | — | 删除 |
| `get_settings` | — | `AppSettings` | 收藏、置顶开关、语言偏好、方案顺序、预设优先开关、全局快捷键 |
| `toggle_favorite` | `displayId`, `res` | `AppSettings` | 给某屏收藏/取消一档 |
| `set_pin_favorites` | `pin` | `AppSettings` | 收藏是否置顶 |
| `set_lang` | `lang` | `AppSettings` | 存语言偏好，并重建托盘菜单 |
| `hide_panel` | — | — | 收起面板 |
| `quit_app` | — | — | 退出（`app.exit(0)`） |
| `get_autostart` | — | `bool` | 回读系统里 LaunchAgent 的真实注册结果 |
| `set_autostart` | `enabled` | `bool` | 开/关登录自启动，返回最终状态 |
| `open_url` | `url` | — | 只允许打开本项目仓库的页面（发布页 / 主页 / issue），见「检查更新」 |
| `open_display_settings` | — | — | 打开系统自带的「显示器」设置，pane id 按系统版本选，见「架构与托盘窗口」 |
| `set_preset_order` | `order` | `AppSettings` | 拖动排序后整份覆盖方案顺序，空数组表示「没拖过，按读盘顺序」 |
| `set_preset_first` | `on` | `AppSettings` | 预设场景优先开关 |
| `quick_switch_launched` | — | `bool` | 前端挂载后取走启动参数标记（读一次即清零）；为真时后端顺带按屏数定宽、居中弹出面板 |
| `set_panel_lock` | `locked` | — | 锁住/放开「失焦即隐藏」，快速切换倒计时期间用 |
| `set_hotkey` | `shortcut` | `AppSettings` | 录入/清除全局快捷键：先解析校验再落盘，注册失败（组合键被占用）时把旧键注册回去、报错不写盘 |

七个设置类命令（`get_settings` / `toggle_favorite` / `set_pin_favorites` / `set_lang` / `set_preset_order` / `set_preset_first` / `set_hotkey`）**每次都回传最新全量设置**，前端不做本地推断——少一处「前端以为改了、后端没落盘」的错位。

App 自定义命令不需要 capability 条目，`src-tauri/capabilities/default.json` 里只有 `core:default`；插件命令（自启动）才需要，这里也通过 Rust 包一层避开了。

## 显示器身份、名称与尺寸

- **身份**：预设与应用配置一律以 Serial screen id（`sXXXX`）为规范 id，重插线、换接口之后仍然稳定；CGDirectDisplayID 每次重新枚举都会变，不能当持久 key。
- **一次探测**：`src-tauri/src/sysinfo.rs` 把型号名和物理尺寸一次拿全，按 ctx id 组合缓存，避免每次数显示器都起一遍 `system_profiler`。
- **型号名**：`system_profiler SPDisplaysDataType -json` 的 `_name`，用 `_spdisplays_displayID` 对上 Contextual id（= CGDirectDisplayID）。
- **物理尺寸**：IOKit `IOMobileFramebufferShim` 的 `DisplayAttributes` 里 `MaxHorizontal/VerticalImageSize` 换算对角英寸，按其中的 SerialNumber 对上 displayplacer 的 serial。
- **为什么不信 `CGDisplayScreenSize`**：镜像状态下它返回的是整个镜像组的尺寸，直接采信会让 27″ 和 14″ 显示成同一个数。它只在内建屏拿不到 EDID、且该屏本轮没参与镜像时兜底。

## 状态读取：解析 displayplacer 输出

`src-tauri/src/state.rs` 解析 `displayplacer list` 的文本输出，单元测试的 fixture 是真实机器的输出。

- **主显示器**：读 `Origin` 行尾的 `- main display` 标记。
- **镜像组**：displayplacer 在末尾给出一条「等效于当前状态」的建议命令，镜像屏会并成同一个 `id:主+从1+从2` token——按这个 token 切分就能还原出多组镜像，比逐屏比对分辨率可靠得多。
- 解析器只做「读」，任何写操作都在应用配置那一步重新生成 spec，不做增量 patch。

## 应用配置：按角色生成 spec

`build_specs()` 把前端的角色草稿翻译成 displayplacer 参数：

- 镜像组合并成一条 `id:目标+从1+从2 res:... scaling:on`，分辨率取目标屏的。
- **位置优先沿用调用方带进来的当前 `origin`**（`preserved_layout()`），只整体平移到「主屏 = `(0,0)`」这个系统锚点上。displayplacer 把 origin `(0,0)` 那块当主屏，所以「谁是主显」必须落到坐标上；换主屏时其余屏跟着平移，相对位置不变。以前无条件按下面那条自动排，会把用户在系统设置里拖好的上下错位抹掉。
- 两种情况回落自动布局（`auto_layout()`）：任一块屏没带 `origin`，或平移后矩形重叠（典型是解镜像那一刻——两块屏的历史位置都在 `(0,0)`，照搬反而不合法）。回落是整批的，半保留半重排比全重排更难解释。重叠判定拿 `res` 的宽高算矩形，`degree` 为 90/270 时宽高互换。
- 自动布局：主屏 `(0,0)`，其余按宽度从左到右排开，保证不重叠。
- 独立屏附带 `hz` / `color_depth`，**镜像组省略**：镜像组里两块屏的刷新率不一定兼容，带上就会整条命令失败。

**预设不存坐标**：排列属于物理工位、跟场景无关，所以 `draftToConfig` 只在应用路径带 `origin`，存预设前剔掉；`ApplyDisplay.origin` 是 `Option` + `#[serde(default, skip_serializing_if = "Option::is_none")]`，不带这个字段的老预设照常解析、写出去的文件结构也跟以前一模一样。应用预设时由前端 `withLiveOrigins()` 拿系统当下的坐标现填（按 serial id 对齐；预设只在全部屏都插着时才出现在场景条里，所以一定填得到）。

前端提交前还有一层角色约束（主显示器全局唯一、降级自动移交、镜像目标只能是主屏或扩展屏、镜像屏分辨率跟随目标），所以后端拿到的草稿一定是合法组合。

## 分辨率档位过滤

模拟系统设置的逻辑，避免列出一堆选了会花屏的档位：只保留与原生宽高比一致（±2%）的模式，且必须是 HiDPI（`scaling:on`）或原生分辨率；同一分辨率有 HiDPI 和普通两个变体时优先 HiDPI；按宽度降序。

## 存储

- **场景预设**：`~/Library/Application Support/com.displayswitch.app/presets/<名称>.json`，v2 角色格式；读到 v1 旧格式（`mode` + `primary`）自动转换，所以老用户的预设不会一夜消失。
- **设置**：`~/Library/Application Support/com.displayswitch.app/settings.json`（`src-tauri/src/settings.rs`），形如：

```json
{
  "favorites": { "s1001": ["1512x982"] },
  "pinFavorites": true,
  "lang": "auto",
  "presetOrder": ["方案A", "方案B"],
  "presetFirst": false,
  "hotkey": "Command+Shift+D"
}
```

`pinFavorites` 缺字段按 `true` 读、`presetFirst` 缺字段按 `false` 读、`lang` 缺字段等同于 `"auto"`、`hotkey` 的空串读成 `None`——老文件不加迁移也能保持默认行为。`presetOrder` 是没拖过就为空的数组（空 = 按读盘顺序显示），`hotkey` 只有成功注册过才落盘。收藏按显示器分别记录（serial id + 分辨率），置顶组内仍按原始顺序排。

## 国际化

前端不引 vue-i18n。`src/i18n.js` 是一张 zh/en 对照表加一个 `reactive({ pref, lang })`：

- `pref` 是用户选的三态偏好（`auto` / `zh` / `en`），界面按它点亮分段按钮；`lang` 是生效语言（`auto` 时按 `navigator.language` 解析，也就是 macOS 的设置）。`setLangPref()` 一次更新两者并写 `<html lang>`。
- `main.js` 把 `t()` 挂成全局 `$t`——模板读 `i18n.lang` 就建立响应依赖，换语言只重渲染一次，不需要 provide/inject 或 mixin。
- 带变量的文案一律 `{name}` 占位、整句翻译（中英文语序不同，禁止拼接）。`shortName()` 按语言剥掉通用名后缀（「内建显示器」→「内建」，"Built-in Display" → "Built-in"），型号名 U2790B 之类不动。
- 显示器通用名（内建显示器 / Built-in Display）和「原生 / Native」标签是在 `api.js normalizeState` 里生成的，所以**语言变了必须重新 `load()`** 才会刷新名字；`load()` 的顺序也因此是**先 `getSettings` 再 `getState`**。

Rust 侧只有 `src-tauri/src/lang.rs`：一个 `AtomicBool` 生效语言缓存 + `tr(zh, en)` / `tr_fill(zh, en, args)` 取词。不往每个函数穿 lang 参数（太碎）。`resolve()` 把 `"auto"` 或老设置里的缺字段展开成 `defaults read -globalDomain AppleLanguages` 的首项，`settings::effective_lang()` 是它的读盘入口。`set_lang` 的顺序是**先落盘、再按生效值 `set_active`、最后重建托盘菜单**——存的是偏好，缓存要的是生效值，反过来会把 `"auto"` 当成非 en 处理错。

所有会弹到用户面前的后端错误串都是双语的。注意 `format!("...{x}")` 的内联捕获插值不能用在运行时字符串上（双语模板是变量），所以那类文案走 `tr_fill` 的 `{x}` + `.replace()`。

## 检查更新

`src/update.js`，纯前端比对，**不引 Tauri updater 插件**，也不自动检测、不后台轮询——只有用户在「关于」里点「检查更新」才联网。

- 两个源各查一次 `releases/latest`：GitHub `WitsonX/DisplaySwitch`、Gitee `Witson/DisplaySwitch`（公开接口、无需 token、都回 CORS 头）。中文界面优先 Gitee、英文优先 GitHub，第一个源不可达就回落第二个；单次请求 8s 超时（`AbortController`）。
- tag 先过一遍只认版本号字符的正则，再和 `package.json` 的 `version` 比（`v` 前缀、`-beta` 这类后缀不参与比较）。两源都 404 读作「暂无发布版本」、都失败读作「检查失败」，四种结果都只在那一行里显示，不弹窗。
- **发布页地址由本模块用常量 + tag 自己拼，不采用接口返回的 `html_url`**；tag 已经过白名单正则，所以拼出来的 URL 只可能是我们自己仓库的发布页。
- 跳转走 Rust 的 `open_url` 命令（`Command::new("open")`）。命令端再收一次口子：只放过 `https://github.com/WitsonX/DisplaySwitch/` 和 `https://gitee.com/Witson/DisplaySwitch/` 两个前缀，其余一律拒绝并返回双语错误（`is_release_page()` 有单元测试，覆盖前缀混淆和 http 降级）。
- 发现新版后按钮变成「打开发布页」，不再触发检查；想重查一次，关掉面板重开即可（`v-if` 会销毁设置视图，状态回到「未检查」）。
- 「关于」最底下是一行居中的元信息：两组「标签 + 链接」并排、中间一道竖线（项目主页 GitHub · Gitee 镜像｜ 反馈建议 GitHub · Gitee 镜像），链接只写站点名，「国内镜像」挪到下面单独一句说明里；那句只在中文界面渲染，对英文使用者不构成信息。两组放得下就一行、放不下按组折行（单屏时窗口只有 380px），所以行数不写死。这行与「更新」那行各管各的，更新的状态机（未检查 / 检查中 / 已是最新 / 发现新版）仍在自己那行里。链接走同一条 `open_url`：地址在 `src/update.js` 的 `REPO_LINKS` 里拼，和检查更新共用同一组 owner/repo 常量，换账号只改一处。主页那两条必须带尾斜杠——白名单前缀是带斜杠的，裸仓库名匹配不上会被拒。

## 自启动

`tauri-plugin-autostart`（`MacosLauncher::LaunchAgent`，底层 `auto-launch`）注册为插件后，由 Rust 侧 `get_autostart` / `set_autostart` 包一层，前端不引 `@tauri-apps/plugin-autostart`、也不需要额外 capabilities。开关**默认关闭**，开启后写 `~/Library/LaunchAgents/DisplaySwitch.plist`。

`is_enabled` 直接看 plist 在不在，所以 settings.json 里不重复存一份状态——两处存同一个开关迟早对不上，用户手动删掉 plist 时也要能如实显示「未开启」。每次进设置面板都回读一次。

> 开发时别顺手点开这个开关：`npm run tauri dev` 跑的是 debug 构建，注册进去的自启动路径就是 `target/debug/` 里那个，之后开机自启动的会是调试版。

## 外部调用：快速切换

给 MarkLock 便签这类启动器留的入口，带参启动即进「切完就走」的模式：

```bash
open -a DisplaySwitch --args -quick-switch   # 等价短参 -qs
```

- **参数只认两个 token**：`QUICK_SWITCH_ARGS = ["-quick-switch", "-qs"]`（`src-tauri/src/lib.rs:33`），`run()` 在建 Builder 之前扫 `std::env::args_os()`，命中就把 `QUICK_SWITCH` 这枚 `AtomicBool` 置位。技术标识用英文：中文参数要穿过 AppleScript/剪贴板/便签几层编码，不值得为省两个字母冒险。
- **取一次即失效**：前端 `mounted` 调 `quick_switch_launched`，`QUICK_SWITCH.swap(false)` 读出并清零；为真时后端顺手 `show_quick_switch`——按当前屏数定宽（单屏 380，多屏 600）、在主屏居中弹出（托盘弹出有托盘坐标当锚点，这里没有，居中最好找），再 `show` + `set_focus`。所以只有那一次冷启动会进快速切换，之后任何唤出都是普通面板。
- **已在托盘驻留时带参启动只是唤出**：`tauri-plugin-single-instance` 让第二个进程把 argv 转交首个实例后自行 `exit(0)`；回调里命中快速切换参数才 `show_panel`，不进快速切换、不倒计时。macOS 上它靠 `/tmp/com_displayswitch_app_si.sock`（bundle id 的点换成下划线）认已有实例，因此 dev 裸二进制和 `/Applications` 里那份会互相认出来——同时开着两边时，后起的那个直接退。
- **应用后才起倒数**：`pushDraft` / `applyPreset` 成功 `load()` 之后调 `startCountdown()`，3 秒到点走 `quit_app`；重复点场景不会重置倒数（`countdown > 0` 直接 return），切换失败则不起倒数、只报错。
- **倒数期间不许面板跑掉**：displayplacer 重排显示器会让面板瞬间失焦，原本的「失焦即隐藏」会把倒计时连同取消入口一起收走。所以点下场景之前先 `set_panel_lock(true)`，`stopCountdown()`（倒数走完、点「隐藏到菜单栏」、切换失败）时解锁。底部那行只在 `quickSwitch && view === 'main'` 出现：有倒数时是两枚按钮——「隐藏到菜单栏」收面板留在托盘，「N 秒后退出」把倒数挂在真正会执行退出的那枚上；没倒数时只是一行说明文字。Toast 靠 `raised` 上移到 58px，不压住倒数。

## 图标

- **菜单栏图标**是一枚单色模板 glyph（`src-tauri/icons/tray-template.png`，36×36 = 18pt @2x），配 `icon_as_template(true)` 交给 `NSImage.isTemplate`，深色菜单栏自动描白、浅色描黑。此前直接用彩色 App 图标会被 `tray-icon` 压成 18pt，深色菜单栏下整块糊掉。形状是「两块屏斜向叠放 + 前屏一根底座横线」——没有那根横线时它等于系统的「复制」图标，会被认成快捷键指示器。
- **App 图标**同一套 glyph（石墨底白形）。面板顶栏的「显示器」标题图标是同一份几何的内联 SVG（18 viewBox + 遮罩挖缝），设置面板「关于」的 48pt 大图标读 `src/assets/app-icon.png`。
- 三者都由 `tools/gen_icons.swift` 从同一份路径描述生成：在仓库根目录跑 `swift tools/gen_icons.swift tools/out`，出 glyph + 1024 源图 + `src/assets/app-icon.png` + 深浅两条预览；再 `npx tauri icon tools/out/app-icon-1024.png` 重生成 icns/ico/各尺寸。
- `src/assets/app-icon.png` 是**从 1024 画布正中裁掉四周留白**的产物（只留圆角方本体）：带留白的图落在白色卡片上会读成一层白框。`.a-icon` 的 11px 圆角正好对上裁切后图形自身的圆角（185.5/824×48 ≈ 10.8px），所以阴影顺着圆角走、四角不露馅。
- `tauri icon` 不会动 `tray-template.png`，但新生成的 glyph 要记得手动拷进 `src-tauri/icons/`。

## 命名与 bundle 标识

中文名「快切屏」，英文名 DisplaySwitch，界面上中文在前。技术标识一律不动——bundle 文件名、`CFBundleName`、bundle id、LaunchAgent plist 仍是 `DisplaySwitch` / `com.displayswitch.app`，只通过 `src-tauri/Info.plist` 往生成的 Info.plist 里合并 `CFBundleDisplayName = 快切屏` 和 `LSUIElement`（Tauri 2 会自动读取并 merge）。这样系统设置「登录项」等 macOS 界面显示中文名，而 dmg、脚本、路径不受非 ASCII 影响。

## 前端双路径

通过 `window.__TAURI_INTERNALS__` 检测运行环境：浏览器预览走 `src/api.js` 里内置的 mock（三屏假数据，apply 会真实改写 mock 状态，所以镜像、置顶、预设、双语都能在浏览器里点通），数据形状与真实后端完全一致。这样不装 Rust 也能评审界面。

## 开发与调试

环境：macOS（仅支持 macOS）+ Xcode Command Line Tools + Node.js 18+ + Rust 工具链。

```bash
npm install
npm run tauri dev          # 开发运行（热重载）
npm run tauri build        # 打包
cd src-tauri && cargo test # 解析器 / 设置 / 语言 / 发布页白名单 / 布局保持单元测试
```

- 浏览器预览：`npm run dev` 打开 http://localhost:5173，底部有「假数据」提示，不接入真实切换。
- **带参调试要绕开 `tauri dev`**：`npm run tauri dev -- -- -quick-switch` 会把参数交给 cargo（cargo 报用法错误）。可行做法是先 `npm run dev` 起 vite，再 `cd src-tauri && cargo run -- -qs`——debug 构建读的是 `devUrl`，界面照样热重载。注意 `/Applications` 里那份会和你抢单实例锁，两边同时开时后起的那个自己就退了。
- displayplacer 已内嵌在 `src-tauri/binaries/`（universal 二进制 + MIT LICENSE），运行时 `displayplacer_path()` 优先解析包内 `Contents/Resources/binaries/`、未命中回落 PATH，所以开发环境也可以 `brew install displayplacer` 用系统那份。
- **不要跑 `cargo fmt`**：这个仓库不是 rustfmt-clean，跑一次会顺手重排四五个无关文件，把格式化噪声混进功能提交。新代码按周围风格手写即可。
- 前端一律 **Options API**，不使用 `<script setup>` / Composition API。

## 打包与发布

- `npm run tauri build` 产物在 `src-tauri/target/release/bundle/`（`.app` 与 `.dmg`）。二进制是 universal（arm64 + x86_64）、ad-hoc 签名，**未走 Apple 公证**：首次打开提示「Apple 无法验证」是固定流程，弹窗点「完成」→「系统设置 → 隐私与安全性 → 安全性」里点「仍要打开」，只需一次。正式签名分发时 `binaries/displayplacer` 要随 App 一起签。
- **版本号有三处要同步**：`package.json`、`src-tauri/tauri.conf.json`、`src-tauri/Cargo.toml`（外加 `src-tauri/Cargo.lock` 里 `display-switch` 那一条）。「关于」显示的和「检查更新」比对的都取 `package.json` 的 `version`，所以发布时的 Release tag 必须是 `v<version>`（如 `v1.0.0`），否则自己跟自己比不出结果。
- 两个托管站都要建 Release：GitHub `WitsonX/DisplaySwitch`、Gitee `Witson/DisplaySwitch`（Gitee 是镜像仓库，附件与 Release 不跟着镜像走，得单独发）。任一边有 Release 之前，App 里点「检查更新」只会显示「暂无发布版本」。

## 已知坑

- `CGDisplayScreenSize` 在镜像态返回整个镜像组的尺寸，不能直接当单屏尺寸用。
- 双语错误串走运行时模板，不能用 `format!` 的内联捕获插值，得用 `tr_fill`。
- 托盘点击坐标是物理像素，窗口尺寸是逻辑点，中间必须除一次 `scale_factor`。
- 设置视图在 380×520 下内容高 498、可视 469：方案列表为空时也要滚一点才看全「退出」按钮，任何一条已存方案都会溢出。这是固定窗口高度的必然结果，没有为了空状态去压缩已经定稿的间距。
- 镜像屏不显示收藏星——它的分辨率由目标屏决定，收藏了也不会生效。
- **第二次双击会把 App 拽回前台**：Tauri 只在 `App` 上暴露 `set_activation_policy`，`setup` 里调已经晚了——进程 check-in 时 LaunchServices 按普通前台 App 登记（`lsappinfo` 里 `type="Foreground"`）。再双击一次不产生新进程，走的是 reopen 事件，AppKit 按那份登记把策略打回 Regular，Dock 图标就此留住。写进 `src-tauri/Info.plist` 的 `LSUIElement` 让登记本身是 agent（`type="UIElement"`），首次启动的闪烁和第二次留住一起消失。查策略用 `NSWorkspace.shared.runningApplications` 读 `activationPolicy`（0=Regular / 1=Accessory）。
- **面板重新弹出后第一下点不到**：macOS 默认把「点击未激活窗口」的第一下用来激活窗口，而这套面板是失焦即隐藏、每次弹出都可能没焦点。`tauri.conf.json` 里 `acceptFirstMouse: true` 让第一下直接落到网页。
