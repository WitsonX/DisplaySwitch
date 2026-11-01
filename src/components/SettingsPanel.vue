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
    </div>

    <div class="section-label">{{ $t('groupSaved') }}</div>
    <div class="card s-card">
      <div v-if="!rows.length" class="s-empty">{{ $t('presetsEmpty') }}</div>
      <div v-for="r in rows" :key="r.p.name" class="s-row">
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
            <span class="a-name">快切屏</span>
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
      <div class="a-row a-row-center">
        <span class="a-k">{{ $t('updateName') }}</span>
        <span class="a-v" :class="{ 'a-v-new': updateState === 'available' }">{{ updateText }}</span>
        <button class="a-btn" :disabled="updateState === 'checking'" @click="onUpdate">
          {{ $t(updateState === 'available' ? 'updateOpen' : 'updateCheck') }}
        </button>
      </div>
      <div class="a-row">
        <span class="a-k">{{ $t('aboutPathKey') }}</span>
        <span class="a-v a-path">~/Library/Application Support/com.displayswitch.app/presets/</span>
      </div>
    </div>

    <button class="s-quit" @click="$emit('quit')">{{ $t('quitApp') }}</button>
  </div>
</template>

<script>
import { version } from '../../package.json'
import appIcon from '../assets/app-icon.png'
import { i18n, t } from '../i18n'
import { checkForUpdate } from '../update'

export default {
  name: 'SettingsPanel',
  props: {
    presets: { type: Array, required: true },
    present: { type: Array, required: true },
    pinFavorites: { type: Boolean, default: true },
    autostart: { type: Boolean, default: false },
    langPref: { type: String, default: 'auto' }
  },
  emits: ['delete', 'rename', 'set-pin', 'set-autostart', 'set-lang', 'quit', 'open-url'],
  data() {
    return {
      version,
      // 绿色版构建标识（正式版为空）：由 Vite define 注入，见 vite.config.js
      buildStamp: typeof __DISPLAYSWITCH_BUILD__ === 'string' ? __DISPLAYSWITCH_BUILD__ : '',
      appIcon,
      renaming: null,
      renameValue: '',
      // idle / checking / upToDate / available / none / failed
      updateState: 'idle',
      updateLatest: '',
      updateUrl: ''
    }
  },
  computed: {
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
