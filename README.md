# 快切屏 DisplaySwitch

macOS 菜单栏托盘小工具：给每台显示器指定角色（主显示器 / 与某屏镜像 / 扩展），点选分辨率档位，并保存整套场景为预设随时恢复。支持三屏及以上的部分镜像组合，界面中英双语。替代反复进出「系统设置 → 显示器」的繁琐流程。

## 核心场景

外接显示器有双输入、同时接着 Mac 和另一台电脑。显示器切给另一台电脑用时（不拔线），Mac 仍认为外接屏连着。此时一键切到「内置屏为主、外接只是镜像」的模式，显示器收回时一键切回——这两种状态各存成一个场景预设即可。

## 功能

- **纯托盘交互**：启动后不显示在程序坞、不占 ⌘Tab，只有菜单栏那枚图标；左键点击弹出/收起面板、点面板外任意处自动隐藏，右键是 显示面板 / 设置… / 退出（菜单文案跟着界面语言）；所有切换点选即时生效、无需确认
- **角色模型**：每台显示器一行；双屏用分段按钮「主显 / 镜像 / 扩展」，三屏及以上改用下拉（镜像候选多于一个时直接列全，选项带目标屏的内容字母如「与 U2790B [A] 镜像」，与行内 A/B 示意对应）；主显示器全局唯一（降级时自动移交），镜像屏升为主显时与原主屏互换角色，镜像目标只能是主屏或扩展屏（不套娃），镜像屏分辨率跟随目标屏
- **部分镜像**：每屏的角色各自独立，所以能表达「外接1 只镜像外接2、内建保持独立」这类组合，不再只有「全部镜像 / 全部扩展」两种状态
- **名称与尺寸**：外接屏行内显示型号名（U2790B / UHD …）与物理尺寸徽章（27″ 这类），内建屏固定显示「内建显示器」且不带尺寸徽章；尺寸在镜像状态下也保持准确
- **左右结构面板**：左列显示器列表（含内容示意字母 A/B/C，镜像屏为虚线同款字母），右列常驻所选屏的分辨率列表；选中镜像屏时右列显示「跟随」说明
- **收藏分辨率**：每档分辨率尾部一颗星，点星只收藏不切换分辨率（已收藏常显实心金色，未收藏悬浮该档才显出轮廓）；收藏按显示器分别记录（serial id + 分辨率），默认把已收藏的档位置顶，置顶组内仍按原顺序排；镜像屏不显示星（分辨率由目标屏决定）
- **场景预设**：底部通栏一行胶囊，点名字直接应用整套配置，鼠标移上去即刻显示该预设内容摘要（自绘 tooltip，不等系统延迟）；「+ 保存当前配置…」点开即聚焦输入框，回车保存、Esc 取消；预设初始为空，不预置默认方案；方案记录所属显示器（serial id），对应显示器未接入时该方案不出现在胶囊行里
- **设置面板**：右上角齿轮进入，托盘右键菜单「设置…」直接进（顶栏「‹ 返回」回到主面板；设置视图下顶栏只留返回、标题与关闭，不再重复显示器标题和状态胶囊）——「通用」区三项按面板顺序：界面语言（分段按钮，见下条）、登录 macOS 时自动启动、收藏的分辨率是否排在最前（关闭后标星保留、列表恢复原始顺序）——后两项是 macOS 风格滑动开关，控件靠右、说明在标题下方；「已存方案」列出全部方案（含未接入的，以灰色「显示器未接入」标注），可改名、删除；「关于」以 App 大图标 + 名称版本 + 用途开头，下面接 displayplacer 致谢与方案存放路径，卡片下面一枚整宽「退出快切屏」按钮（悬浮转红；没有 Dock 图标后，托盘右键「退出」之外的第二个退出入口）
- **双语界面**：简体中文 / English，设置里三态分段「跟随系统 / 简体中文 / English」，**默认跟随 macOS 系统语言**；手动选过哪一档就固定存哪一档（选「跟随系统」也照样存成 `lang: "auto"`，之后系统语言变了界面跟着变）。切换即时生效、不用重启：面板文案、显示器通用名（内建显示器 / Built-in Display）、档位「原生 / Native」标签、预设摘要、提示气泡全部跟着换；托盘右键菜单和后端错误信息也同步
- **单屏适配**：合盖等只剩一台显示器的场景下自动退回单列布局（分辨率列表直接放在显示器行下方），状态胶囊显示「单屏 · 分辨率」；此时不显示内容示意字母（只有一块屏，没有可比对的内容），显示器行也不作为可选项（光标不变手型、点击无选中态）
- **自启动**：设置面板「通用」区的「登录 macOS 时自动启动」开关，**默认关闭**；开启后写 `~/Library/LaunchAgents/DisplaySwitch.plist`，登录即出现在菜单栏（仍不进程序坞）；开关状态每次进设置都回读系统真实注册结果，手动删掉 plist 也会如实显示为未开启
- **热插拔感知**：注册 CoreGraphics 显示器重配置回调（`CGDisplayRegisterReconfigurationCallback`），合盖/开盖/插拔后防抖 800ms 推送 `displays-changed` 事件自动刷新列表；面板获得焦点时也会刷新一次
- **displayplacer 内嵌**：universal 二进制（arm64 + x86_64，ad-hoc 签名）与 MIT LICENSE 放在 `src-tauri/binaries/`，经 `bundle.resources` 打入 `Contents/Resources/binaries/`；运行时 `displayplacer_path()` 优先解析包内路径、未命中回落 PATH；正式签名分发时随 App 一起签名

## 依赖

- macOS（仅支持 macOS）
- displayplacer：已内嵌在 App 包内（`src-tauri/binaries/displayplacer`，universal 二进制，MIT 协议），终端用户无需安装；开发环境若未命中包内路径会回落到 PATH（可 `brew install displayplacer`）
- 开发：Node.js 18+、Rust 工具链（Tauri 2）

## 开发与构建

```bash
npm install
npm run tauri dev     # 开发运行（热重载）
npm run tauri build   # 打包
cd src-tauri && cargo test   # 解析器 / 设置 / 语言单元测试
```

浏览器里直接 `npm run dev` 打开 http://localhost:5173 可预览界面（假数据，底部有提示，不接入真实切换）。

## 技术说明

- **架构**：Tauri 2（Rust 后端）+ Vue 3（Options API，不使用 `<script setup>`）+ Vite；托盘面板为 600×520 逻辑点的透明无边框窗口（单屏时收窄为 380 宽，宽度按 `LAST_DISPLAY_COUNT` 在托盘点击时决定），HUD 毛玻璃质感（window-vibrancy）；`setup` 里 `set_activation_policy(Accessory)` 让它不进程序坞、不占 ⌘Tab，窗口失焦（`WindowEvent::Focused(false)`）自动隐藏
- **显示器名称与尺寸**：`src-tauri/src/sysinfo.rs` 一次探测、按 ctx id 组合缓存。型号名来自 `system_profiler SPDisplaysDataType -json` 的 `_name`，用 `_spdisplays_displayID` 对上 Contextual id（= CGDirectDisplayID）；尺寸来自 IOKit `IOMobileFramebufferShim` 的 `DisplayAttributes`（`MaxHorizontal/VerticalImageSize` 换算对角英寸），按其中的 SerialNumber 对上 displayplacer 的 serial——`CGDisplayScreenSize` 在镜像态会返回整个镜像组的尺寸所以不可直接采信，只在内建屏拿不到 EDID 且该屏未参与镜像时兜底
- **显示器状态读取**：Rust 侧解析 `displayplacer list` 输出（`src-tauri/src/state.rs`），含单元测试（fixture 为真实输出）；主显示器从 `Origin` 行的 `- main display` 标记读取，镜像组从末尾建议命令的 `id:主+从1+从2` token 读取（支持多组）
- **显示器身份**：预设与应用配置以 Serial screen id（`sXXXX`）为规范 id，重插线/换接口后最稳定
- **模式档位过滤**：模拟系统设置逻辑——只保留与原生宽高比一致（±2%）且为 HiDPI（scaling:on）或原生分辨率的模式，同分辨率优先取 HiDPI 变体，按宽度降序
- **应用配置**：按角色生成多组 displayplacer spec——镜像组合并为 `id:目标+从1+从2 res:... scaling:on`（分辨率取目标屏），主显示器放 `origin:(0,0)`（displayplacer 借此设置系统主屏），其余组按宽度从左到右排开避免重叠；独立屏附带 hz/color_depth，镜像组省略以免遇到刷新率不兼容
- **预设存储**：`~/Library/Application Support/com.displayswitch.app/presets/<名称>.json`；v2 角色格式，读取 v1 旧格式（`mode` + `primary`）时自动转换
- **设置存储**：`~/Library/Application Support/com.displayswitch.app/settings.json`（`src-tauri/src/settings.rs`），形如 `{ "favorites": { "s1001": ["1512x982"] }, "pinFavorites": true, "lang": "auto" }`；命令 `get_settings` / `toggle_favorite` / `set_pin_favorites` / `set_lang` 每次都回传最新全量设置，前端不做本地推断；`pinFavorites` 缺字段按 `true` 读（老文件不含该键时保持默认行为），`lang` 存 `"auto" | "zh" | "en"`，缺字段（老设置）等同于 `"auto"`
- **国际化**：前端不引 vue-i18n，`src/i18n.js` 是一张 zh/en 对照表加一个 `reactive({ pref, lang })`——`pref` 是用户选的三态偏好（`auto` / `zh` / `en`，界面按它点亮分段按钮），`lang` 是生效语言（`auto` 时按 `navigator.language` 解析，就是 macOS 的设置），`setLangPref()` 一次同时更新两者；`main.js` 把 `t()` 挂成全局 `$t`——模板读它即建立响应依赖，换语言只重渲染一次；带变量的文案用 `{name}` 占位（英文语序不同，不做拼接式翻译）；`shortName()` 按语言剥掉通用名后缀（「内建显示器」→「内建」，"Built-in Display"→"Built-in"）。Rust 侧只有 `src-tauri/src/lang.rs`：一个 `AtomicBool` 生效语言缓存 + `tr()/tr_fill()` 取词，`resolve()` 把 `"auto"` 或老设置里的缺字段展开成 `defaults read -globalDomain AppleLanguages` 首项，`settings::effective_lang()` 是它的读盘入口。托盘菜单是原生控件、不会跟着网页重渲染，所以启动时先定生效语言再建菜单，`set_lang` 存好偏好后就地 `set_menu()` 重建一份
- **自启动实现**：`tauri-plugin-autostart`（`MacosLauncher::LaunchAgent`，底层 `auto-launch` 0.6）注册为插件后，由 Rust 侧 `get_autostart` / `set_autostart` 两个命令包一层，前端不引 `@tauri-apps/plugin-autostart`、也不需要额外 capabilities；`is_enabled` 直接看 plist 在不在，所以 settings.json 里不重复存一份状态
- **托盘定位**：托盘点击坐标为物理像素，按 `scale_factor` 换算后定位（Retina 2x 下窗口尺寸用 `LogicalSize`，避免物理/逻辑单位混用）
- **前端双路径**：通过 `window.__TAURI_INTERNALS__` 检测运行环境，浏览器预览走内置 mock（三屏假数据，apply 会真实改写 mock 状态），数据形状与真实后端一致
- **图标**：菜单栏图标是一枚单色模板 glyph（`src-tauri/icons/tray-template.png`，36×36 = 18pt @2x），配 `icon_as_template(true)` 交给 `NSImage.isTemplate`，深色菜单栏自动描白、浅色描黑（此前直接用彩色 App 图标会被 `tray-icon` 压成 18pt，深色菜单栏下整块糊掉）；形状是「两块屏斜向叠放 + 前屏一根底座横线」——没有那根横线时它等于系统的复制图标。App 图标同一套 glyph（石墨底白形）。面板顶栏的「显示器」标题图标是同一份几何的内联 SVG（18 viewBox + 遮罩挖缝），设置面板「关于」的 48pt 大图标则读 `src/assets/app-icon.png`。三者都由 `tools/gen_icons.swift` 从同一份路径描述生成：`swift tools/gen_icons.swift tools/out`（在仓库根目录跑）出 glyph + 1024 源图 + `src/assets/app-icon.png` + 深浅两条预览，再 `npx tauri icon tools/out/app-icon-1024.png` 重生成 icns/ico/各尺寸（`tauri icon` 不会动 `tray-template.png`，但要记得把新生成的 glyph 拷进 `src-tauri/icons/`）
- **命名**：中文名「快切屏」，英文名 DisplaySwitch（界面上中文在前）。技术标识不动——bundle 文件名、`CFBundleName`、bundle id、LaunchAgent plist 仍是 DisplaySwitch / com.displayswitch.app，只通过 `src-tauri/Info.plist` 往生成的 Info.plist 里合并 `CFBundleDisplayName = 快切屏`，所以系统设置「登录项」等 macOS 界面会显示中文名
- **界面草图存档**：v2 方案讨论的可交互草图在 `doc/ui-prototype/`（镜像组版、角色竖版、角色左右结构版）

## 暂不支持

Windows 支持、旋转、刷新率微调、色彩配置文件。
