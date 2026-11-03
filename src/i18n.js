import { reactive } from 'vue'

// 轻量语言包：一张 zh/en 对照表 + 一个 reactive 的当前语言。
// 语言偏好三态：auto（跟随系统）/ zh / en；auto 时按 WKWebView 的
// navigator.language（就是 macOS 的设置）解析成实际语言。
function detectLang() {
  const nav = typeof navigator !== 'undefined' ? navigator.language || '' : ''
  return nav.toLowerCase().startsWith('zh') ? 'zh' : 'en'
}

const LANG_PREFS = ['auto', 'zh', 'en']

export const i18n = reactive({
  // 用户选的偏好，原样存进设置
  pref: 'auto',
  // 实际生效的语言（zh / en），文案表按它取
  lang: detectLang()
})

function resolveLang(pref) {
  return pref === 'zh' || pref === 'en' ? pref : detectLang()
}

export function setLangPref(pref) {
  i18n.pref = LANG_PREFS.includes(pref) ? pref : 'auto'
  i18n.lang = resolveLang(i18n.pref)
  if (typeof document !== 'undefined') {
    document.documentElement.lang = i18n.lang === 'en' ? 'en' : 'zh-CN'
  }
}

const zh = {
  back: '返回',
  settings: '设置',
  close: '关闭',
  retry: '重试',

  appName: '快切屏',
  titleDisplays: '显示器',
  titleResolutions: '分辨率',
  titlePresets: '场景预设',

  groupGeneral: '通用',
  groupSaved: '已存方案',
  groupAbout: '关于',

  nameBuiltin: '内建显示器',
  nameExternal: '外接显示器',
  nameExternalN: '外接显示器 {n}',
  nativeTag: '原生',

  roleMainSeg: '主显',
  roleMirrorSeg: '镜像',
  roleExtendSeg: '扩展',
  roleMain: '主显示器',
  roleExtend: '扩展显示器',
  // 提示语里角色出现在句中，中英文的大小写/冠词不同，单列一套
  roleLabelMain: '主显示器',
  roleLabelExtend: '扩展显示器',
  roleLabelMirror: '镜像屏',
  mirrorWith: '与 {name} [{letter}] 镜像',

  statusSingle: '单屏 · {res}',
  statusMulti: '{count}屏 · 主 {name}',
  statusMirrors: ' · {n}屏镜像',

  descMainIs: '{name}为主',
  descCustomMain: '自定义主屏',
  descMirrors: '{n}屏镜像',

  follows: '跟随 {name} · {res}',
  gfxSameAs: '内容与 {name} 相同',
  gfxContent: '这块屏显示的内容',

  mirrorNote:
    '{name} 与 {target} 镜像，分辨率由 {target} 决定（当前 {res}）。要改分辨率，请选中 {target}。',

  favRes: '收藏该分辨率',
  unfavRes: '取消收藏',

  autostartName: '登录 macOS 时自动启动',
  pinName: '收藏的分辨率排在最前',
  pinDesc: '关闭后列表恢复原始顺序，标星保留',
  presetFirstName: '预设场景优先',
  presetFirstDesc: '把方案排到最前面，按钮和字号一起放大',
  langName: '界面语言',
  langAuto: '跟随系统',
  langZh: '简体中文',
  langEn: 'English',

  presetsEmpty: '还没有方案 · 回到面板保存当前配置',
  presetsUnavailable: '显示器未接入',
  rename: '改名',
  remove: '删除',
  save: '保存',
  cancel: '取消',

  aboutDesc: '菜单栏里切换显示器角色（主显 / 镜像 / 扩展）与分辨率档位',
  aboutDriverKey: '切换驱动',
  aboutDriverVal: '内嵌 displayplacer（MIT 开源）',
  aboutPathKey: '方案存放',
  updateName: '更新',
  updateIdle: '未检查',
  updateChecking: '正在检查…',
  updateUpToDate: '已是最新版本',
  updateAvailable: '发现新版本 v{v}',
  updateNone: '暂无发布版本',
  updateFailed: '检查失败，请重试',
  updateCheck: '检查更新',
  updateOpen: '打开发布页',
  quitApp: '退出快切屏',

  savePresetBtn: '+ 保存',
  savePresetTip: '存下当前角色与分辨率，之后一键恢复',
  presetPlaceholder: '预设名称，如：工位-扩展',
  mockNote: '浏览器预览 · 假数据 · 未接入真实切换',

  loadError: '无法读取显示器状态：{e}',
  favError: '收藏失败：{e}',
  setPinError: '设置失败：{e}',
  presetOrderError: '排序保存失败：{e}',
  langError: '语言设置失败：{e}',
  autostartReadError: '读取自启动状态失败：{e}',
  autostartSetError: '自启动设置失败：{e}',
  openError: '打开失败：{e}',
  autostartOnDemo: '已设为登录自启动（演示）',
  autostartOffDemo: '已取消登录自启动（演示）',
  applyError: '切换失败：{e}',
  roleSet: '已把 {name} 设为{label}',
  resSet: '已切换 {name} 到 {label}',
  presetApplied: '已应用「{name}」',
  presetApplyError: '应用失败：{e}',
  quickSwitchHint: '快速切换：选择场景后 3 秒自动退出',
  autoQuitIn: '{n} 秒后退出',
  hidePanel: '隐藏到菜单栏',
  presetDeleted: '已删除「{name}」',
  presetDeleteError: '删除失败：{e}',
  presetRenamed: '已改名为「{name}」',
  presetRenameError: '改名失败：{e}',
  presetNameRequired: '请输入预设名称',
  presetSaved: '已保存「{name}」',
  presetSaveError: '保存失败：{e}',
  demoClose: '演示：关闭面板',
  demoQuit: '演示：退出应用',
  demoSuffix: '（演示）',
  quitError: '退出失败'
}

const en = {
  back: 'Back',
  settings: 'Settings',
  close: 'Close',
  retry: 'Retry',

  appName: 'DisplaySwitch',
  titleDisplays: 'Displays',
  titleResolutions: 'Resolution',
  titlePresets: 'Scene presets',

  groupGeneral: 'General',
  groupSaved: 'Saved setups',
  groupAbout: 'About',

  nameBuiltin: 'Built-in Display',
  nameExternal: 'External Display',
  nameExternalN: 'External Display {n}',
  nativeTag: 'Native',

  roleMainSeg: 'Main',
  roleMirrorSeg: 'Mirror',
  roleExtendSeg: 'Extend',
  roleMain: 'Main display',
  roleExtend: 'Extended display',
  roleLabelMain: 'the main display',
  roleLabelExtend: 'an extended display',
  roleLabelMirror: 'a mirrored display',
  mirrorWith: 'Mirror {name} [{letter}]',

  statusSingle: 'One display · {res}',
  statusMulti: '{count} displays · main {name}',
  statusMirrors: ' · {n} mirrored',

  descMainIs: '{name} is main',
  descCustomMain: 'Custom main display',
  descMirrors: '{n} mirrored',

  follows: 'Follows {name} · {res}',
  gfxSameAs: 'Shows the same content as {name}',
  gfxContent: 'What this display shows',

  mirrorNote:
    '{name} mirrors {target}, so {target} sets its resolution (currently {res}). To change it, select {target}.',

  favRes: 'Star this resolution',
  unfavRes: 'Remove star',

  autostartName: 'Launch at macOS login',
  pinName: 'Starred resolutions first',
  pinDesc: 'Turn off to restore the original order; stars are kept',
  presetFirstName: 'Saved presets first',
  presetFirstDesc: 'Pin presets to the top with larger buttons and text',
  langName: 'Language',
  langAuto: 'System',
  langZh: '简体中文',
  langEn: 'English',

  presetsEmpty: 'No setups yet · save the current one from the main panel',
  presetsUnavailable: 'Displays not connected',
  rename: 'Rename',
  remove: 'Delete',
  save: 'Save',
  cancel: 'Cancel',

  aboutDesc: 'Switch display roles (main / mirror / extend) and resolution presets from the menu bar',
  aboutDriverKey: 'Driver',
  aboutDriverVal: 'displayplacer bundled (MIT)',
  aboutPathKey: 'Setup files',
  updateName: 'Updates',
  updateIdle: 'Not checked',
  updateChecking: 'Checking…',
  updateUpToDate: 'Up to date',
  updateAvailable: 'New version v{v}',
  updateNone: 'No releases yet',
  updateFailed: 'Check failed',
  updateCheck: 'Check for updates',
  updateOpen: 'View release',
  quitApp: 'Quit DisplaySwitch',

  savePresetBtn: '+ Save',
  savePresetTip: 'Store the current roles and resolutions',
  presetPlaceholder: 'Setup name, e.g. Desk - Extended',
  mockNote: 'Browser preview · mock data · no real switching',

  loadError: 'Cannot read display state: {e}',
  favError: 'Could not star: {e}',
  setPinError: 'Could not change setting: {e}',
  presetOrderError: 'Could not save the order: {e}',
  langError: 'Could not change language: {e}',
  autostartReadError: 'Could not read login item state: {e}',
  autostartSetError: 'Could not change login item: {e}',
  openError: 'Could not open: {e}',
  autostartOnDemo: 'Set to launch at login (demo)',
  autostartOffDemo: 'Removed from login items (demo)',
  applyError: 'Switch failed: {e}',
  roleSet: 'Set {name} as {label}',
  resSet: 'Switched {name} to {label}',
  presetApplied: 'Applied “{name}”',
  presetApplyError: 'Could not apply: {e}',
  quickSwitchHint: 'Quick switch: auto-quit 3s after applying',
  autoQuitIn: 'Quit in {n}s',
  hidePanel: 'Hide to Menu Bar',
  presetDeleted: 'Deleted “{name}”',
  presetDeleteError: 'Could not delete: {e}',
  presetRenamed: 'Renamed to “{name}”',
  presetRenameError: 'Could not rename: {e}',
  presetNameRequired: 'Enter a name for this setup',
  presetSaved: 'Saved “{name}”',
  presetSaveError: 'Could not save: {e}',
  demoClose: 'Demo: close panel',
  demoQuit: 'Demo: quit app',
  demoSuffix: ' (demo)',
  quitError: 'Could not quit'
}

const tables = { zh, en }

function fill(str, vars) {
  if (!vars) return str
  return str.replace(/\{(\w+)\}/g, (_, key) => (vars[key] === undefined ? '' : String(vars[key])))
}

export function t(key, vars) {
  const table = tables[i18n.lang] || zh
  const text = table[key] !== undefined ? table[key] : zh[key]
  return fill(text !== undefined ? text : key, vars)
}

// 显示器型号名（U2790B）不动，只把「外接显示器」这类通用叫法缩短，行内更紧凑
export function shortName(name) {
  const n = (name || '').trim()
  if (i18n.lang === 'en') return n.replace(/\s*display\b/i, '').trim() || n
  return n.replace(/显示器/g, '').trim() || n
}
