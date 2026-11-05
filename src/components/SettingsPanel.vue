<template>
  <div class="settings">
    <div class="section-label">{{ $t('groupGeneral') }}</div>
    <div class="card s-card">
      <div class="s-row">
        <span class="s-label">
          <span class="s-name">{{ $t('langName') }}</span>
        </span>
        <div class="s-seg">
          <button
            class="s-seg-btn"
            :class="{ on: langPref === 'auto' }"
            @click="$emit('set-lang', 'auto')"
          >
            {{ $t('langAuto') }}
          </button>
          <button
            class="s-seg-btn"
            :class="{ on: langPref === 'zh' }"
            @click="$emit('set-lang', 'zh')"
          >
            {{ $t('langZh') }}
          </button>
          <button
            class="s-seg-btn"
            :class="{ on: langPref === 'en' }"
            @click="$emit('set-lang', 'en')"
          >
            {{ $t('langEn') }}
          </button>
        </div>
      </div>
      <label class="s-row s-toggle-row">
        <span class="s-label">
          <span class="s-name">{{ $t('autostartName') }}</span>
        </span>
        <input
          type="checkbox"
          class="s-switch"
          :checked="autostart"
          @change="$emit('set-autostart', $event.target.checked)"
        />
      </label>
      <label class="s-row s-toggle-row">
        <span class="s-label">
          <span class="s-name">{{ $t('pinName') }}</span>
          <span class="s-desc">{{ $t('pinDesc') }}</span>
        </span>
        <input
          type="checkbox"
          class="s-switch"
          :checked="pinFavorites"
          @change="$emit('set-pin', $event.target.checked)"
        />
      </label>
      <label class="s-row s-toggle-row">
        <span class="s-label">
          <span class="s-name">{{ $t('presetFirstName') }}</span>
          <span class="s-desc">{{ $t('presetFirstDesc') }}</span>
        </span>
        <input
          type="checkbox"
          class="s-switch"
          :checked="presetFirst"
          @change="$emit('set-preset-first', $event.target.checked)"
        />
      </label>
      <div class="s-row">
        <span class="s-label">
          <span class="s-name">{{ $t('hotkeyName') }}</span>
          <span class="s-desc">{{ recording && invalidHint ? $t('hotkeyNeedMod') : $t('hotkeyDesc') }}</span>
        </span>
        <div class="s-hotkey">
          <button
            class="s-hotkey-field"
            :class="{ recording }"
            @click="toggleRecord"
          >
            {{
              recording
                ? $t('hotkeyRecording')
                : hotkey
                  ? formatHotkey(hotkey)
                  : $t('hotkeyNone')
            }}
          </button>
          <button
            v-if="hotkey && !recording"
            class="s-hotkey-clear"
            :title="$t('hotkeyClear')"
            @click="clearHotkey"
          >
            ✕
          </button>
        </div>
      </div>
    </div>

    <div class="section-label">{{ $t('groupSaved') }}</div>
    <div class="card s-card" :class="{ dragging: !!drag }">
      <div v-if="!rows.length" class="s-empty">{{ $t('presetsEmpty') }}</div>
      <div
        v-for="(r, ri) in rows"
        :key="r.p.name"
        class="s-row s-preset"
        :class="{ 's-lift': drag && drag.from === ri, 'drop-before': gapAt(ri), 'drop-after': gapAt(ri + 1) && ri === rows.length - 1 }"
        @pointerdown="rowDown(ri, $event)"
        @pointermove="rowMove($event)"
        @pointerup="rowUp"
        @pointercancel="drag = null"
      >
        <template v-if="renaming !== r.p.name">
          <div class="s-info">
            <span class="s-name">{{ r.p.name }}</span>
            <span class="s-desc">{{ r.available ? r.p.desc : $t('presetsUnavailable') }}</span>
          </div>
          <div class="s-acts">
            <button class="s-act" @click="startRename(r.p)">{{ $t('rename') }}</button>
            <button class="s-act s-act-del" @click="$emit('delete', r.p)">
              {{ $t('remove') }}
            </button>
          </div>
        </template>
        <div v-else class="s-rename">
          <input
            ref="renameInput"
            v-model="renameValue"
            class="s-input"
            @keyup.enter="commitRename(r.p)"
            @keyup.esc="renaming = null"
          />
          <button class="s-act" @click="commitRename(r.p)">{{ $t('save') }}</button>
          <button class="s-act" @click="renaming = null">{{ $t('cancel') }}</button>
        </div>
      </div>
    </div>

    <div class="section-label">{{ $t('groupAbout') }}</div>
    <div class="card s-card about">
      <div class="a-row a-brand">
        <img class="a-icon" :src="appIcon" alt="快切屏 DisplaySwitch 图标" />
        <div class="a-brand-text">
          <div class="a-brand-name">
            <span v-if="!isEn" class="a-name">快切屏</span>
            <span class="a-en">DisplaySwitch</span>
            <span class="a-ver">v{{ version }}</span>
            <span v-if="buildLabel" class="a-canary">{{ buildLabel }}</span>
          </div>
          <span class="a-desc">{{ $t('aboutDesc') }}</span>
        </div>
      </div>
      <div class="a-row">
        <span class="a-k">{{ $t('aboutDriverKey') }}</span>
        <span class="a-v">{{ $t('aboutDriverVal') }}</span>
      </div>
      <div class="a-row">
        <span class="a-k">{{ $t('aboutPathKey') }}</span>
        <span class="a-v a-path">~/Library/Application Support/com.displayswitch.app/presets/</span>
      </div>
      <div class="a-row a-row-center">
        <span class="a-k">{{ $t('updateName') }}</span>
        <span class="a-v" :class="{ 'a-v-new': updateState === 'available' }">{{ updateText }}</span>
        <button class="a-btn" :disabled="updateState === 'checking'" @click="onUpdate">
          {{ $t(updateState === 'available' ? 'updateOpen' : 'updateCheck') }}
        </button>
      </div>
      <div class="a-row a-foot">
        <span class="a-group">
          <span class="a-g-k">{{ $t('aboutHomeKey') }}</span>
          <button class="a-link" @click="$emit('open-url', links.home.github)">
            {{ $t('linkGithub') }}
          </button>
          <span class="a-link-sep">·</span>
          <button class="a-link" @click="$emit('open-url', links.home.gitee)">
            {{ $t('linkGiteeMirror') }}
          </button>
        </span>
        <span class="a-group">
          <span class="a-g-k">{{ $t('aboutFeedbackKey') }}</span>
          <button class="a-link" @click="$emit('open-url', links.feedback.github)">
            {{ $t('linkGithub') }}
          </button>
          <span class="a-link-sep">·</span>
          <button class="a-link" @click="$emit('open-url', links.feedback.gitee)">
            {{ $t('linkGiteeMirror') }}
          </button>
        </span>
      </div>
      <div v-if="!isEn" class="a-note">{{ $t('aboutMirrorNote') }}</div>
    </div>

    <button class="s-quit" @click="$emit('quit')">{{ $t('quitApp') }}</button>
  </div>
</template>

<script>
import { version } from '../../package.json'
import appIcon from '../assets/app-icon.png'
import { i18n, t } from '../i18n'
import { checkForUpdate, REPO_LINKS } from '../update'

export default {
  name: 'SettingsPanel',
  props: {
    presets: { type: Array, required: true },
    present: { type: Array, required: true },
    pinFavorites: { type: Boolean, default: true },
    presetFirst: { type: Boolean, default: false },
    autostart: { type: Boolean, default: false },
    langPref: { type: String, default: 'auto' },
    // 显示/隐藏主面板的全局快捷键（后端已注册的规范串），空串表示未设置
    hotkey: { type: String, default: '' }
  },
  emits: ['delete', 'rename', 'reorder', 'set-pin', 'set-preset-first', 'set-autostart', 'set-lang', 'set-hotkey', 'quit', 'open-url'],
  data() {
    return {
      version,
      // 关于页的项目主页 / 反馈建议两组地址，见 update.js 的 REPO_LINKS
      links: REPO_LINKS,
      // 绿色版构建标识（正式版为空）：由 Vite define 注入，见 vite.config.js
      buildStamp: typeof __DISPLAYSWITCH_BUILD__ === 'string' ? __DISPLAYSWITCH_BUILD__ : '',
      appIcon,
      renaming: null,
      renameValue: '',
      // 快捷键录入态：recording 是否在监听按键，invalidHint 是否刚按了缺主修饰键的组合
      recording: false,
      invalidHint: false,
      // 拖拽排序中的状态：{ from, over, y0, armed }，null 表示没在拖
      drag: null,
      // idle / checking / upToDate / available / none / failed
      updateState: 'idle',
      updateLatest: '',
      updateUrl: ''
    }
  },
  computed: {
    // 底部那句镜像说明只在中文界面出现。i18n.lang 是 reactive 的，切语言会跟着重渲染
    isEn() {
      return i18n.lang === 'en'
    },
    rows() {
      const present = new Set(this.present)
      return this.presets.map((p) => ({
        p,
        available: p.config.displays.every((d) => present.has(d.ctx_id))
      }))
    },
    updateText() {
      switch (this.updateState) {
        case 'checking':
          return t('updateChecking')
        case 'upToDate':
          return t('updateUpToDate')
        case 'available':
          return t('updateAvailable', { v: this.updateLatest })
        case 'none':
          return t('updateNone')
        case 'failed':
          return t('updateFailed')
        default:
          return t('updateIdle')
      }
    },
    // 绿色版在版本号旁标出构建号（canary-<sha> 去掉前缀），正式版为空不显示
    buildLabel() {
      return this.buildStamp ? `Canary ${this.buildStamp.replace('canary-', '')}` : ''
    }
  },
  beforeUnmount() {
    // 面板收起会直接卸载组件，确保录入监听不留残留
    if (this.recording) window.removeEventListener('keydown', this.onRecordKey, true)
  },
  methods: {
    startRename(p) {
      this.renaming = p.name
      this.renameValue = p.name
      this.$nextTick(() => {
        const r = this.$refs.renameInput
        const el = Array.isArray(r) ? r[0] : r
        if (el) el.focus()
      })
    },
    commitRename(p) {
      const name = this.renameValue.trim()
      if (name && name !== p.name) this.$emit('rename', p, name)
      this.renaming = null
    },
    // —— 全局快捷键录入 ——
    toggleRecord() {
      if (this.recording) this.stopRecord()
      else this.startRecord()
    },
    startRecord() {
      this.recording = true
      this.invalidHint = false
      // 用捕获阶段监听，配合 stopPropagation 抢在 App 的全局按键处理前，
      // 免得录入时误触 ⌘Q 退出或 ⌘1–⌘9 应用场景
      window.addEventListener('keydown', this.onRecordKey, true)
    },
    stopRecord() {
      this.recording = false
      this.invalidHint = false
      window.removeEventListener('keydown', this.onRecordKey, true)
    },
    onRecordKey(e) {
      e.preventDefault()
      e.stopPropagation()
      // 不带修饰键单按 Esc：取消录入，不当作快捷键
      const hasMod = e.metaKey || e.ctrlKey || e.altKey || e.shiftKey
      if (e.key === 'Escape' && !hasMod) {
        this.stopRecord()
        return
      }
      // 只按修饰键还没带主键，继续等下一个键
      if (['Meta', 'Control', 'Alt', 'Shift'].includes(e.key)) return
      // macOS 全局快捷键必须带 ⌘/⌃/⌥ 至少一个主修饰键，光按 ⇧ 或裸键会跟系统/其他 App 冲突
      if (!(e.metaKey || e.ctrlKey || e.altKey)) {
        this.invalidHint = true
        return
      }
      const key = this.keyToken(e)
      if (!key) return // 不支持的键，忽略并继续录入
      this.invalidHint = false
      const mods = []
      if (e.metaKey) mods.push('Command')
      if (e.ctrlKey) mods.push('Control')
      if (e.altKey) mods.push('Alt')
      if (e.shiftKey) mods.push('Shift')
      this.$emit('set-hotkey', [...mods, key].join('+'))
      this.stopRecord()
    },
    clearHotkey() {
      this.$emit('set-hotkey', '')
    },
    // 把 KeyboardEvent 映射成 global-shortcut 认的按键 token（keyboard-types Code 名）
    keyToken(e) {
      if (/^[a-zA-Z]$/.test(e.key)) return e.key.toUpperCase()
      if (/^[0-9]$/.test(e.key)) return e.key
      if (/^F(?:[1-9]|1[0-2])$/.test(e.key)) return e.key.toUpperCase()
      const map = {
        ' ': 'Space',
        Enter: 'Enter',
        Tab: 'Tab',
        Escape: 'Escape',
        Backspace: 'Backspace',
        Delete: 'Delete',
        ArrowUp: 'ArrowUp',
        ArrowDown: 'ArrowDown',
        ArrowLeft: 'ArrowLeft',
        ArrowRight: 'ArrowRight',
        Home: 'Home',
        End: 'End',
        PageUp: 'PageUp',
        PageDown: 'PageDown',
        ',': 'Comma',
        '.': 'Period',
        '/': 'Slash',
        ';': 'Semicolon',
        "'": 'Quote',
        '-': 'Minus',
        '=': 'Equal',
        '[': 'BracketLeft',
        ']': 'BracketRight',
        '\\': 'Backslash',
        '`': 'Backquote'
      }
      return map[e.key] || null
    },
    // 规范串转成 macOS 符号显示，如 Command+Shift+D → ⌘⇧D
    formatHotkey(str) {
      const sym = { Command: '⌘', Control: '⌃', Alt: '⌥', Shift: '⇧' }
      const keyDisp = {
        Space: 'Space',
        Enter: '↵',
        Escape: '⎋',
        Tab: '⇥',
        Backspace: '⌫',
        Delete: '⌦',
        ArrowUp: '↑',
        ArrowDown: '↓',
        ArrowLeft: '←',
        ArrowRight: '→'
      }
      const mods = []
      const keys = []
      for (const p of str.split('+')) {
        if (sym[p]) mods.push(sym[p])
        else keys.push(keyDisp[p] || p)
      }
      return mods.join('') + keys.join('')
    },
    rowDown(i, e) {
      if (e.button !== 0 || this.renaming !== null) return
      // 行里的改名/删除按钮自己管点击，不能被拖拽抢走
      if (e.target.closest('button') || e.target.closest('input')) return
      e.currentTarget.setPointerCapture(e.pointerId)
      this.drag = { from: i, over: -1, y0: e.clientY, armed: false }
    },
    rowMove(e) {
      const d = this.drag
      if (!d) return
      // 按下不动也要能正常选中文字，移动超过几像素才算拖
      if (!d.armed && Math.abs(e.clientY - d.y0) < 4) return
      const els = [...this.$el.querySelectorAll('.s-preset')]
      let over = els.length
      for (let i = 0; i < els.length; i++) {
        const r = els[i].getBoundingClientRect()
        if (e.clientY < r.top + r.height / 2) {
          over = i
          break
        }
      }
      this.drag = { ...d, armed: true, over }
    },
    rowUp() {
      const d = this.drag
      this.drag = null
      if (!d || !d.armed) return
      // over 是「插入到第 over 行之前」，先把被拖行摘掉，插入点越过后要左移一格
      const to = d.over > d.from ? d.over - 1 : d.over
      if (to === d.from) return
      const names = this.rows.map((r) => r.p.name)
      const [moved] = names.splice(d.from, 1)
      names.splice(to, 0, moved)
      this.$emit('reorder', names)
    },
    gapAt(k) {
      const d = this.drag
      if (!d || d.over < 0) return false
      if (d.over === d.from || d.over === d.from + 1) return false
      return d.over === k
    },
    async onUpdate() {
      if (this.updateState === 'available') {
        this.$emit('open-url', this.updateUrl)
        return
      }
      if (this.updateState === 'checking') return
      this.updateState = 'checking'
      const res = await checkForUpdate(i18n.lang !== 'en', version)
      this.updateState = res.status
      this.updateLatest = res.latest || ''
      this.updateUrl = res.url || ''
    }
  }
}
</script>

<style scoped>
.settings {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 0 12px 12px;
}

.settings::-webkit-scrollbar {
  width: 0;
}

.section-label {
  min-height: 20px;
  padding-bottom: 5px;
}

.s-card {
  margin-bottom: 10px;
}

.s-empty {
  padding: 12px;
  font-size: 12px;
  color: var(--text-2);
}

.s-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  border-bottom: 1px solid var(--divider);
}

.s-row:last-child {
  border-bottom: 0;
}

/* 拖动排序的行：整行可抓，抬起的那行变淡，插入位置用一条横线表示。
   禁选要常驻：.dragging 是 pointerdown 之后下一帧才挂上的，浏览器按下那一刻已经起选了 */
.s-preset {
  position: relative;
  cursor: grab;
  user-select: none;
  -webkit-user-select: none;
}

.s-preset .s-input {
  user-select: text;
  -webkit-user-select: text;
}

.s-card.dragging .s-preset {
  cursor: grabbing;
}

.s-lift {
  opacity: 0.4;
}

.drop-before::before,
.drop-after::after {
  content: '';
  position: absolute;
  left: 12px;
  right: 12px;
  height: 2px;
  border-radius: 2px;
  background: var(--accent);
}

.drop-before::before {
  top: 0;
}

.drop-after::after {
  bottom: 0;
}

.s-info {
  display: flex;
  align-items: baseline;
  gap: 8px;
  min-width: 0;
  flex: 1;
}

.s-name {
  font-size: 12.5px;
  font-weight: 500;
  white-space: nowrap;
}

.s-desc {
  font-size: 11.5px;
  color: var(--text-2);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.s-acts {
  display: flex;
  gap: 2px;
  flex-shrink: 0;
}

.s-act {
  font-size: 12px;
  color: var(--accent);
  padding: 3px 7px;
  border-radius: 6px;
}

.s-act:hover {
  background: var(--accent-soft);
}

.s-act-del {
  color: #ff3b30;
}

.s-act-del:hover {
  background: rgba(255, 59, 48, 0.1);
}

.s-toggle-row {
  cursor: pointer;
  gap: 12px;
}

.s-label {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 2px;
  min-width: 0;
  flex: 1;
}

.s-label .s-desc {
  white-space: normal;
  font-size: 11px;
  line-height: 1.35;
}

.s-switch {
  appearance: none;
  -webkit-appearance: none;
  position: relative;
  width: 34px;
  height: 20px;
  flex-shrink: 0;
  margin: 0;
  border-radius: 20px;
  background: rgba(0, 0, 0, 0.16);
  cursor: pointer;
  transition: background 0.18s ease;
}

.s-switch::after {
  content: '';
  position: absolute;
  top: 2px;
  left: 2px;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: #fff;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.28);
  transition: transform 0.18s ease;
}

.s-switch:checked {
  background: #34c759;
}

.s-switch:checked::after {
  transform: translateX(14px);
}

.s-switch:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
}

/* 三态语言偏好（跟随系统 / 简体中文 / English）用分段按钮，和主界面的角色按钮同一套视觉 */
.s-seg {
  display: flex;
  gap: 2px;
  padding: 2px;
  background: rgba(0, 0, 0, 0.05);
  border-radius: 8px;
  flex-shrink: 0;
}

.s-seg-btn {
  font-size: 11.5px;
  padding: 3px 9px;
  border-radius: 6px;
  color: var(--text-2);
  white-space: nowrap;
}

.s-seg-btn:hover {
  color: var(--text-1);
}

.s-seg-btn.on {
  background: var(--card);
  color: var(--accent);
  font-weight: 600;
  box-shadow: 0 0.5px 2px rgba(0, 0, 0, 0.14);
}

.s-rename {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 1;
  min-width: 0;
}

.s-input {
  flex: 1;
  min-width: 0;
  font: inherit;
  font-size: 12.5px;
  padding: 4px 8px;
  border: 1px solid var(--divider);
  border-radius: 6px;
  background: var(--bg);
  color: var(--text-1);
}

/* 全局快捷键录入：右侧一个像输入框的按钮，点击进入录入态 */
.s-hotkey {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

.s-hotkey-field {
  font: inherit;
  font-size: 12px;
  font-weight: 500;
  min-width: 74px;
  text-align: center;
  padding: 5px 10px;
  border: 0.5px solid var(--divider);
  border-radius: 7px;
  background: var(--card);
  color: var(--text-1);
  box-shadow: 0 0.5px 1.5px rgba(0, 0, 0, 0.06);
  white-space: nowrap;
}

.s-hotkey-field:hover {
  border-color: rgba(10, 132, 255, 0.35);
}

.s-hotkey-field.recording {
  border-color: var(--accent);
  color: var(--accent);
  background: var(--accent-soft);
}

.s-hotkey-clear {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  color: var(--text-2);
  border-radius: 6px;
  flex-shrink: 0;
}

.s-hotkey-clear:hover {
  background: rgba(255, 59, 48, 0.1);
  color: #ff3b30;
}

.about {
  padding: 2px 12px;
}

.a-row {
  display: flex;
  align-items: baseline;
  gap: 10px;
  padding: 6px 0;
  border-bottom: 1px solid var(--divider);
}

.a-row:last-child {
  border-bottom: 0;
}

/* 更新行里有按钮，基线对齐会让按钮沉下去 */
.a-row-center {
  align-items: center;
}

.a-brand {
  align-items: center;
  gap: 12px;
  padding: 10px 0;
}

.a-icon {
  width: 48px;
  height: 48px;
  border-radius: 11px;
  box-shadow: 0 1px 5px rgba(0, 0, 0, 0.22);
  flex: none;
}

.a-brand-text {
  display: flex;
  flex-direction: column;
  gap: 3px;
  min-width: 0;
}

.a-brand-name {
  display: flex;
  align-items: baseline;
  gap: 6px;
}

.a-name {
  font-size: 14px;
  font-weight: 600;
}

.a-en {
  font-size: 11.5px;
  color: var(--text-2);
}

/* 英文版隐藏「快切屏」后，DisplaySwitch 变为首个名称元素，升级为主标题样式 */
.a-brand-name > .a-en:first-child {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-1);
}

.a-desc {
  font-size: 11.5px;
  line-height: 1.45;
  color: var(--text-2);
}

.a-ver {
  font-size: 11.5px;
  color: var(--text-2);
}

.a-canary {
  font-size: 10.5px;
  color: #b7791f;
  background: rgba(183, 121, 31, 0.12);
  border-radius: 4px;
  padding: 1px 5px;
}

.a-k {
  font-size: 11.5px;
  color: var(--text-2);
  width: 66px;
  flex-shrink: 0;
}

.a-v {
  font-size: 11.5px;
  line-height: 1.45;
  flex: 1;
  min-width: 0;
  word-break: normal;
  overflow-wrap: anywhere;
}

.a-path {
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 10.5px;
  color: var(--text-2);
}

.a-v-new {
  color: var(--accent);
  font-weight: 600;
}

/* 底部元信息一行：两组「标签 + 链接」居中并排，组间一道竖线，整行按组折行。
   不写死行数：窗口宽度会变（单屏时只有 380px），英文也没有「镜像」两个字。
   每组 nowrap，所以折行点落在组边界而不是某条链接中间；字号比上面几行小半档 */
.a-foot {
  flex-wrap: wrap;
  justify-content: center;
  column-gap: 0;
  row-gap: 6px;
  border-bottom: 0;
  font-size: 11px;
}

.a-group {
  display: flex;
  align-items: baseline;
  gap: 5px;
  white-space: nowrap;
}

/* 竖线两侧各 8px：单屏时窗口 380px，按 11px 间距实测只剩 5px 余量，再挤就会折行、
   竖线跑到第二行行首（真机不会窄于 380，但得留出字体渲染差异的余量） */
.a-group + .a-group {
  margin-left: 8px;
  padding-left: 8px;
  border-left: 1px solid var(--divider);
}

.a-g-k {
  color: var(--text-2);
}

.a-link {
  font: inherit;
  line-height: 1.45;
  color: var(--accent);
  padding: 0;
}

.a-link:hover {
  text-decoration: underline;
}

.a-link-sep {
  color: var(--text-2);
  opacity: 0.6;
}

/* 镜像说明：中文界面单独占一行，英文界面不渲染（见模板里的 isEn）。
   以前把「国内镜像」写进每条链接，四个标签被撑成两行，反而读不清 */
.a-note {
  padding: 4px 0 4px;
  font-size: 11px;
  line-height: 1.5;
  color: var(--text-2);
  text-align: center;
}

.a-btn {
  flex-shrink: 0;
  font-size: 11px;
  color: var(--accent);
  background: var(--card);
  border: 0.5px solid var(--divider);
  border-radius: 7px;
  padding: 3px 8px;
  white-space: nowrap;
}

.a-btn:hover:not(:disabled) {
  background: var(--accent-soft);
}

.a-btn:disabled {
  opacity: 0.45;
  cursor: default;
}

.s-quit {
  display: block;
  width: 100%;
  font-size: 12.5px;
  font-weight: 500;
  color: var(--text-1);
  background: var(--card);
  border: 0.5px solid var(--divider);
  border-radius: 10px;
  padding: 8px 0;
  box-shadow: 0 0.5px 1.5px rgba(0, 0, 0, 0.05);
}

.s-quit:hover {
  color: #ff3b30;
  border-color: rgba(255, 59, 48, 0.35);
  background: rgba(255, 59, 48, 0.05);
}
</style>
