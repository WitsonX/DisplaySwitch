<template>
  <div class="panel" :class="{ single: isSingle }">
    <header class="panel-header">
      <template v-if="view === 'settings'">
        <button class="back-btn" @click="view = 'main'">
          <svg
            width="12"
            height="12"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.6"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M15 18l-6-6 6-6" />
          </svg>
          {{ $t('back') }}
        </button>
        <div class="title title-sub">{{ $t('settings') }}</div>
        <span class="head-spacer" />
      </template>
      <template v-else>
        <div class="title">
          <img class="title-icon" :src="appIcon" :alt="$t('appName')" />
          <span>{{ $t('appName') }}</span>
          <span v-if="!isEnglish" class="title-en">DisplaySwitch</span>
        </div>
        <span v-if="statusText" class="status-pill">{{ statusText }}</span>
        <button class="gear-btn" :title="$t('settings')" @click="view = 'settings'">
          <svg
            width="15"
            height="15"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <circle cx="12" cy="12" r="3" />
            <path
              d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09a1.65 1.65 0 0 0-1-1.51 1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09a1.65 1.65 0 0 0 1.51-1 1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33h.09a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51h.09a1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82v.09a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"
            />
          </svg>
        </button>
      </template>
      <button class="close-btn" :title="$t('close')" @click="closePanel">✕</button>
    </header>

    <SettingsPanel
      v-if="view === 'settings'"
      :presets="describedPresets"
      :present="presentIds"
      :pin-favorites="pinFavorites"
      :preset-first="presetFirst"
      :autostart="autostart"
      :lang-pref="langPref"
      @delete="removePreset"
      @rename="renamePreset"
      @reorder="reorderPresets"
      @set-pin="setPin"
      @set-preset-first="changePresetFirst"
      @set-autostart="changeAutostart"
      @set-lang="changeLang"
      @open-url="openUrl"
      @quit="quitApp"
    />

    <div v-else-if="error" class="error-box">
      <p class="error-text">{{ error }}</p>
      <button class="retry-btn" @click="load">{{ $t('retry') }}</button>
    </div>

    <div v-else class="panel-body" :class="{ single: isSingle, 'preset-first': presetFirstLayout }">
      <div class="cols">
        <div class="col col-left">
          <div class="section-label">{{ $t('titleDisplays') }}</div>
          <DisplayList
            :displays="enriched"
            :selected-id="isSingle ? '' : selectedId"
            :show-role="!isSingle"
            @select="selectedId = $event"
            @role-change="setRole"
          />
        </div>
        <div v-if="!isSingle" class="col col-right">
          <div class="section-label">{{ $t('titleResolutions') }}</div>
          <ResolutionPanel
            :display="selectedDisplay"
            :favorites="favList(selectedDisplay)"
            :pin="pinFavorites"
            @res-select="applyRes"
            @toggle-fav="toggleFav(selectedDisplay, $event)"
          />
        </div>
      </div>

      <template v-if="isSingle && displays.length === 1">
        <div class="section-label single-label">{{ $t('titleResolutions') }}</div>
        <ResolutionPanel
          :display="enriched[0]"
          :show-head="false"
          :favorites="favList(enriched[0])"
          :pin="pinFavorites"
          @res-select="applyRes"
          @toggle-fav="toggleFav(enriched[0], $event)"
        />
      </template>

      <div class="preset-area">
        <div class="section-label">{{ $t('titlePresets') }}</div>
        <div v-if="saving" class="save-box">
          <input
            ref="saveInput"
            :value="newPresetName"
            class="save-input"
            :placeholder="$t('presetPlaceholder')"
            @input="newPresetName = $event.target.value"
            @keyup.enter="savePreset"
            @keyup.esc="cancelSave"
          />
          <button class="save-ok" @click="savePreset">{{ $t('save') }}</button>
          <button class="save-cancel" @click="cancelSave">{{ $t('cancel') }}</button>
        </div>
        <div v-else class="preset-bar">
          <button
            v-for="p in presets"
            :key="p.name"
            class="preset-pill"
            @click="applyPreset(p)"
            @mouseenter="showTip(p.desc, $event)"
            @mouseleave="hideTip"
            @focus="showTip(p.desc, $event)"
            @blur="hideTip"
            >{{ p.name }}</button
          >
          <button
            class="preset-pill preset-save"
            @click="toggleSave"
            @mouseenter="showTip($t('savePresetTip'), $event)"
            @mouseleave="hideTip"
            @focus="showTip($t('savePresetTip'), $event)"
            @blur="hideTip"
            >{{ $t('savePresetBtn') }}</button
          >
          <div v-if="tip" ref="tip" class="preset-tip" :style="{ left: tip.left + 'px' }">
            {{ tip.desc }}
          </div>
        </div>
        <div v-if="isMock" class="mock-note">{{ $t('mockNote') }}</div>
      </div>
    </div>

    <div v-if="quickFooter" class="quick-footer">
      <template v-if="countdown > 0">
        <button class="quick-btn" @click="closePanel">{{ $t('hidePanel') }}</button>
        <button class="quick-btn quick-quit" @click="quitApp">
          {{ $t('autoQuitIn', { n: countdown }) }}
        </button>
      </template>
      <span v-else class="quick-hint">{{ $t('quickSwitchHint') }}</span>
    </div>

    <Toast v-if="toast" :message="toast" :raised="quickFooter" />
  </div>
</template>

<script>
import DisplayList from './components/DisplayList.vue'
import ResolutionPanel from './components/ResolutionPanel.vue'
import SettingsPanel from './components/SettingsPanel.vue'
import Toast from './components/Toast.vue'
import * as api from './api'
import appIcon from './assets/app-icon.png'
import { i18n, t, shortName, setLangPref } from './i18n'

export default {
  name: 'App',
  components: { DisplayList, ResolutionPanel, SettingsPanel, Toast },
  data() {
    return {
      isMock: !api.isTauri,
      appIcon,
      view: 'main',
      displays: [],
      selectedId: '',
      allPresets: [],
      presetOrder: [],
      favorites: {},
      pinFavorites: true,
      presetFirst: false,
      quickSwitch: false,
      countdown: 0,
      countdownTimer: null,
      autostart: false,
      saving: false,
      newPresetName: '',
      toast: null,
      toastTimer: null,
      tip: null,
      error: null
    }
  },
  computed: {
    langPref() {
      return i18n.pref
    },
    isEnglish() {
      return i18n.lang === 'en'
    },
    isSingle() {
      return this.displays.length <= 1
    },
    // 快速切换会话里无视用户开关，强制预设场景优先布局
    presetFirstLayout() {
      return this.presetFirst || this.quickSwitch
    },
    quickFooter() {
      return this.quickSwitch && this.view === 'main'
    },
    presentIds() {
      return this.displays.map((d) => d.id)
    },
    // 拖过的顺序在前，没进过顺序表的（刚存的方案）保持读盘顺序跟在后面
    orderedPresets() {
      const rank = (name, i) => {
        const k = this.presetOrder.indexOf(name)
        return k < 0 ? this.presetOrder.length + i : k
      }
      return this.allPresets
        .map((p, i) => ({ p, k: rank(p.name, i) }))
        .sort((a, b) => a.k - b.k)
        .map((x) => x.p)
    },
    // 预设描述每次渲染现算，切换语言后不用重新读盘就能跟着变
    describedPresets() {
      return this.orderedPresets.map((p) => ({ ...p, desc: this.descOf(p.config) }))
    },
    presets() {
      const present = new Set(this.presentIds)
      // 方案记录的显示器没全插上就不出现，避免应用出半套配置
      return this.describedPresets.filter((p) =>
        p.config.displays.every((d) => present.has(d.ctx_id))
      )
    },
    letterMap() {
      const letters = {}
      let next = 65
      for (const d of this.displays) {
        if (!d.mirrorOf) letters[d.id] = String.fromCharCode(next++)
      }
      for (const d of this.displays) {
        if (d.mirrorOf) letters[d.id] = letters[d.mirrorOf] || '?'
      }
      return letters
    },
    enriched() {
      const letters = this.letterMap
      return this.displays.map((d) => {
        const role = d.mirrorOf ? 'mirror' : d.main ? 'main' : 'extend'
        const target = d.mirrorOf ? this.displays.find((x) => x.id === d.mirrorOf) : null
        const subText =
          role === 'mirror' && target
            ? t('follows', { name: shortName(target.name), res: pretty(target.current) })
            : `${pretty(d.current)}${d.hz ? ` · ${d.hz}Hz` : ''}`
        return {
          ...d,
          role,
          roleOptions: this.roleOptionsFor(d),
          letter: letters[d.id],
          subText,
          mirrorTargetName: target ? shortName(target.name) : '',
          mirrorTargetRes: target ? target.current : ''
        }
      })
    },
    selectedDisplay() {
      return this.enriched.find((d) => d.id === this.selectedId) || this.enriched[0] || null
    },
    statusText() {
      if (!this.displays.length) return ''
      if (this.displays.length === 1) {
        return t('statusSingle', { res: pretty(this.displays[0].current) })
      }
      const main = this.displays.find((d) => d.main) || this.displays[0]
      const mirrors = this.displays.filter((d) => d.mirrorOf).length
      const base = t('statusMulti', { count: this.displays.length, name: shortName(main.name) })
      return mirrors ? base + t('statusMirrors', { n: mirrors }) : base
    }
  },
  watch: {
    // 每次进设置都回读一次：用户可能手动删了 LaunchAgent，界面要如实显示
    view(v) {
      if (v === 'settings') this.refreshAutostart()
    }
  },
  mounted() {
    this.load()
    if (api.isTauri) {
      // 冷启动带 -quick-switch / -qs：后端已弹窗，这里消费标志进入快速切换形态
      api.quickSwitchLaunched().then((qs) => {
        if (qs) this.enterQuickSwitch()
      })
      import('@tauri-apps/api/event').then(({ listen }) => {
        listen('displays-changed', () => this.load())
        listen('tray-view', (e) => {
          this.view = e.payload === 'settings' ? 'settings' : 'main'
        })
      })
      import('@tauri-apps/api/window').then(({ getCurrentWindow }) => {
        getCurrentWindow().onFocusChanged(({ payload: focused }) => {
          if (focused) this.load()
        })
      })
    }
  },
  methods: {
    roleOptionsFor(d) {
      const opts = [{ value: 'main', label: t('roleMain') }]
      for (const o of this.displays) {
        if (o.id === d.id || o.mirrorOf) continue
        opts.push({
          value: `mirror:${o.id}`,
          label: t('mirrorWith', { name: shortName(o.name), letter: this.letterMap[o.id] })
        })
      }
      opts.push({ value: 'extend', label: t('roleExtend') })
      return opts
    },
    async load() {
      this.error = null
      try {
        // 先取设置：语言决定显示器通用名和档位标签，读状态前必须定下来
        this.applySettings(await api.getSettings())
        const state = await api.getState()
        this.displays = state.displays
        if (!this.displays.some((d) => d.id === this.selectedId)) {
          const main = this.displays.find((d) => d.main)
          this.selectedId = main ? main.id : (this.displays[0] && this.displays[0].id) || ''
        }
        this.allPresets = (await api.listPresets()).map((p) => ({ name: p.name, config: p.config }))
      } catch (e) {
        this.error = t('loadError', { e })
      }
    },
    applySettings(s) {
      this.favorites = s.favorites || {}
      this.pinFavorites = s.pinFavorites !== false
      this.presetOrder = s.presetOrder || []
      this.presetFirst = s.presetFirst === true
      setLangPref(s.lang || 'auto')
    },
    async reorderPresets(names) {
      const prev = this.presetOrder
      this.presetOrder = names
      try {
        this.applySettings(await api.setPresetOrder(names))
      } catch (e) {
        this.presetOrder = prev
        this.showToast(t('presetOrderError', { e }))
      }
    },
    async changeLang(pref) {
      if (pref === i18n.pref) return
      try {
        setLangPref(pref)
        this.applySettings(await api.setLang(pref))
        await this.load()
      } catch (e) {
        this.showToast(t('langError', { e }))
      }
    },
    favList(display) {
      return (display && this.favorites[display.id]) || []
    },
    async toggleFav(display, m) {
      if (!display) return
      try {
        this.applySettings(await api.toggleFavorite(display.id, m.id))
      } catch (e) {
        this.showToast(t('favError', { e }))
      }
    },
    async setPin(pin) {
      try {
        this.applySettings(await api.setPinFavorites(pin))
      } catch (e) {
        this.showToast(t('setPinError', { e }))
      }
    },
    async changePresetFirst(on) {
      try {
        this.applySettings(await api.setPresetFirst(on))
      } catch (e) {
        this.showToast(t('setPinError', { e }))
      }
    },
    async refreshAutostart() {
      try {
        this.autostart = await api.getAutostart()
      } catch (e) {
        this.showToast(t('autostartReadError', { e }))
      }
    },
    async changeAutostart(enabled) {
      try {
        this.autostart = await api.setAutostart(enabled)
        if (this.isMock)
          this.showToast(enabled ? t('autostartOnDemo') : t('autostartOffDemo'))
      } catch (e) {
        this.showToast(t('autostartSetError', { e }))
        // 先承认复选框当前显示的样子，再回读真实值，否则失败时勾会停在错的位置
        this.autostart = enabled
        await this.$nextTick()
        this.refreshAutostart()
      }
    },
    async openUrl(url) {
      try {
        await api.openUrl(url)
      } catch (e) {
        this.showToast(t('openError', { e }))
      }
    },
    toDraft() {
      return this.displays.map((x) => ({
        id: x.id,
        role: x.mirrorOf ? 'mirror' : x.main ? 'main' : 'extend',
        target: x.mirrorOf || null,
        res: x.current,
        scaling: x.scaling,
        hz: x.hz ?? null,
        color_depth: x.color_depth ?? null,
        degree: x.degree || 0,
        enabled: x.enabled !== false
      }))
    },
    draftToConfig(draft) {
      for (const x of draft) {
        if (x.role === 'mirror') {
          const t = draft.find((y) => y.id === x.target)
          if (t) {
            x.res = t.res
            x.scaling = t.scaling
          }
        }
      }
      return {
        displays: draft.map((x) => ({
          ctx_id: x.id,
          role: x.role,
          mirror_target: x.role === 'mirror' ? x.target : null,
          res: x.res,
          scaling: x.scaling,
          hz: x.hz,
          color_depth: x.color_depth,
          degree: x.degree,
          enabled: x.enabled
        }))
      }
    },
    descOf(config) {
      const main = config.displays.find((d) => d.role === 'main')
      const mirrors = config.displays.filter((d) => d.role === 'mirror').length
      const name = main ? this.displays.find((d) => d.id === main.ctx_id)?.name : null
      const resText = main ? pretty(main.res) : '?'
      const parts = [name ? t('descMainIs', { name: shortName(name) }) : t('descCustomMain')]
      if (mirrors) parts.push(t('descMirrors', { n: mirrors }))
      parts.push(resText)
      return parts.join(' · ')
    },
    showToast(msg) {
      if (this.toastTimer) clearTimeout(this.toastTimer)
      this.toast = msg
      this.toastTimer = setTimeout(() => {
        this.toast = null
        this.toastTimer = null
      }, 2000)
    },
    async pushDraft(draft, okMsg) {
      // 切换会重排显示器、面板瞬间失焦，锁住到倒计时结束
      if (this.quickSwitch) api.setPanelLock(true)
      let applied = false
      try {
        await api.applyConfig(this.draftToConfig(draft))
        this.showToast(this.isMock ? `${okMsg}${t('demoSuffix')}` : okMsg)
        applied = true
      } catch (e) {
        this.showToast(t('applyError', { e }))
        if (this.quickSwitch) api.setPanelLock(false)
      }
      await this.load()
      // 只有真切换成功才倒数：失败了还要用户看错误、重试，不该顺手把 App 收掉
      if (applied) this.startCountdown()
    },
    setRole(d, val) {
      const draft = this.toDraft()
      const me = draft.find((x) => x.id === d.id)
      if (val === 'main') {
        // 镜像屏升主时与原目标互换：原主屏改为镜像新主屏，原主屏其余跟随屏改跟新主屏
        const prevTarget = me.role === 'mirror' ? me.target : null
        for (const x of draft) {
          if (x.id === me.id || x.role !== 'main') continue
          if (prevTarget && x.id === prevTarget) {
            x.role = 'mirror'
            x.target = me.id
          } else {
            x.role = 'extend'
            x.target = null
          }
        }
        if (prevTarget) {
          for (const x of draft) {
            if (x.id !== me.id && x.id !== prevTarget && x.target === prevTarget) {
              x.target = me.id
            }
          }
        }
        me.role = 'main'
        me.target = null
      } else if (val === 'extend') {
        if (me.role === 'main') {
          const next = draft.find((x) => x.id !== me.id && x.role !== 'mirror')
          if (!next) return
          next.role = 'main'
        }
        me.role = 'extend'
        me.target = null
      } else {
        const t = val.slice('mirror:'.length)
        const td = draft.find((x) => x.id === t)
        if (!td) return
        if (me.role === 'main') td.role = 'main'
        for (const x of draft) {
          if (x.id !== me.id && x.target === me.id) x.target = t
        }
        me.role = 'mirror'
        me.target = t
      }
      const label =
        val === 'main'
          ? t('roleLabelMain')
          : val === 'extend'
            ? t('roleLabelExtend')
            : t('roleLabelMirror')
      this.pushDraft(draft, t('roleSet', { name: d.name, label }))
    },
    applyRes(display, m) {
      const draft = this.toDraft()
      const me = draft.find((x) => x.id === display.id)
      me.res = m.id
      me.scaling = m.scaling !== undefined ? m.scaling : me.scaling
      this.pushDraft(draft, t('resSet', { name: display.name, label: m.label }))
    },
    showTip(desc, e) {
      const el = e.currentTarget
      const pill = el.getBoundingClientRect()
      const bar = el.closest('.preset-bar').getBoundingClientRect()
      this.tip = { desc, left: pill.left - bar.left + pill.width / 2, barWidth: bar.width }
      // 气泡宽度要渲染了才知道，所以先居中再用真实半宽夹回条内
      this.$nextTick(this.clampTip)
    },
    clampTip() {
      const el = this.$refs.tip
      if (!el || !this.tip) return
      const half = el.getBoundingClientRect().width / 2
      const maxLeft = Math.max(half, this.tip.barWidth - half)
      this.tip.left = Math.min(Math.max(this.tip.left, half), maxLeft)
    },
    hideTip() {
      this.tip = null
    },
    async applyPreset(p) {
      if (this.quickSwitch) api.setPanelLock(true)
      try {
        await api.applyConfig(p.config)
        this.showToast(t('presetApplied', { name: p.name }))
        await this.load()
        this.startCountdown()
      } catch (e) {
        this.showToast(t('presetApplyError', { e }))
        if (this.quickSwitch) api.setPanelLock(false)
      }
    },
    async removePreset(p) {
      try {
        await api.deletePreset(p.name)
        // 顺手把这条从顺序表里摘掉，免得 setting.json 攒垃圾名字
        if (this.presetOrder.includes(p.name)) {
          await api.setPresetOrder(this.presetOrder.filter((n) => n !== p.name))
        }
        this.showToast(t('presetDeleted', { name: p.name }))
        await this.load()
      } catch (e) {
        this.showToast(t('presetDeleteError', { e }))
      }
    },
    async renamePreset(p, newName) {
      const name = newName.trim()
      if (!name || name === p.name) return
      try {
        await api.savePreset(name, p.config)
        await api.deletePreset(p.name)
        // 改名不该让方案跳出原来的位置，顺序表里原位换名
        if (this.presetOrder.includes(p.name)) {
          await api.setPresetOrder(
            this.presetOrder.map((n) => (n === p.name ? name : n))
          )
        }
        this.showToast(t('presetRenamed', { name }))
        await this.load()
      } catch (e) {
        this.showToast(t('presetRenameError', { e }))
      }
    },
    toggleSave() {
      this.saving = true
      this.newPresetName = ''
      this.tip = null
      this.$nextTick(() => {
        if (this.$refs.saveInput) this.$refs.saveInput.focus()
      })
    },
    cancelSave() {
      this.saving = false
      this.newPresetName = ''
    },
    async savePreset() {
      const name = this.newPresetName.trim()
      if (!name) {
        this.showToast(t('presetNameRequired'))
        return
      }
      try {
        await api.savePreset(name, this.draftToConfig(this.toDraft()))
        this.saving = false
        this.newPresetName = ''
        this.showToast(t('presetSaved', { name }))
        await this.load()
      } catch (e) {
        this.showToast(t('presetSaveError', { e }))
      }
    },
    enterQuickSwitch() {
      this.quickSwitch = true
      this.view = 'main'
      this.load()
    },
    // 应用成功（预设/分辨率/角色）后倒计时 3 秒退出；取消则关面板、程序留在托盘
    startCountdown() {
      if (!this.quickSwitch || this.countdown > 0) return
      this.countdown = 3
      this.countdownTimer = setInterval(() => {
        this.countdown -= 1
        if (this.countdown <= 0) {
          this.stopCountdown()
          this.quitApp()
        }
      }, 1000)
    },
    stopCountdown() {
      if (this.countdownTimer) {
        clearInterval(this.countdownTimer)
        this.countdownTimer = null
      }
      this.countdown = 0
      api.setPanelLock(false)
    },
    async closePanel() {
      this.stopCountdown()
      this.quickSwitch = false
      if (api.isTauri) {
        const { invoke } = await import('@tauri-apps/api/core')
        try {
          await invoke('hide_panel')
        } catch {
          /* 忽略 */
        }
      } else {
        this.showToast(t('demoClose'))
      }
    },
    async quitApp() {
      if (!api.isTauri) {
        this.showToast(t('demoQuit'))
        return
      }
      try {
        await api.quitApp()
      } catch {
        this.showToast(t('quitError'))
      }
    }
  }
}

function pretty(res) {
  return res.replace('x', '×')
}
</script>

<style scoped>
.panel {
  width: 100%;
  height: 100vh;
  background: var(--bg);
  position: relative;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  user-select: none;
}

.panel-header {
  display: flex;
  align-items: center;
  padding: 14px 16px 8px;
  flex-shrink: 0;
}

.title {
  display: flex;
  align-items: center;
  gap: 7px;
  font-size: 15px;
  font-weight: 600;
  color: var(--text-1);
}

.title-icon {
  width: 24px;
  height: 24px;
  flex: none;
}

.title-en {
  font-size: 11.5px;
  font-weight: 500;
  color: var(--text-2);
}

.status-pill {
  margin-left: auto;
  font-size: 11px;
  color: var(--text-2);
  background: rgba(0, 0, 0, 0.05);
  padding: 3px 9px;
  border-radius: 20px;
  white-space: nowrap;
}

.gear-btn {
  margin-left: 8px;
  width: 22px;
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-2);
  border-radius: 6px;
}

.gear-btn:hover {
  background: rgba(0, 0, 0, 0.06);
  color: var(--text-1);
}

.head-spacer {
  flex: 1;
}

.back-btn {
  display: flex;
  align-items: center;
  gap: 3px;
  font-size: 12.5px;
  font-weight: 500;
  color: var(--text-1);
  background: var(--card);
  border: 0.5px solid var(--divider);
  border-radius: 8px;
  padding: 4px 10px 4px 6px;
  box-shadow: 0 0.5px 1.5px rgba(0, 0, 0, 0.08);
}

.back-btn:hover {
  background: var(--accent-soft);
  border-color: rgba(10, 132, 255, 0.25);
  color: var(--accent);
}

.title-sub {
  margin-left: 9px;
  font-size: 14px;
}

.close-btn {
  margin-left: 8px;
  width: 22px;
  height: 22px;
  color: var(--text-2);
  font-size: 14px;
  border-radius: 6px;
  line-height: 1;
}

.close-btn:hover {
  background: rgba(0, 0, 0, 0.06);
  color: var(--text-1);
}

.panel-body {
  overflow-y: auto;
  flex: 1;
  min-height: 0;
  padding: 0 12px 12px;
  display: flex;
  flex-direction: column;
}

.panel-body::-webkit-scrollbar {
  width: 0;
}

.cols {
  display: flex;
  gap: 10px;
  align-items: stretch;
  flex: 1 1 auto;
  min-height: 0;
}

.panel-body.single .cols {
  flex: 0 0 auto;
}

.col-left {
  flex: 1 1 320px;
  min-width: 270px;
  align-self: flex-start;
}

/* 名字短就别占满：按内容收宽，名字长才涨到上限，再长由 .d-name 省略号截断。
   243px = 右列原本的 233px + 10px 列间距，即最长名字下左右宽度与改动前完全一致。 */
.panel-body:not(.single) .col-left {
  flex: 0 1 auto;
  width: max-content;
  min-width: 0;
  max-width: calc(100% - 243px);
}

.col-right {
  flex: 1 1 220px;
  min-width: 200px;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.single-label {
  margin-top: 10px;
}

.preset-area {
  margin-top: 10px;
  flex-shrink: 0;
}

.preset-bar {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
  align-items: center;
  position: relative;
}

.preset-tip {
  position: absolute;
  bottom: calc(100% + 7px);
  transform: translateX(-50%);
  max-width: 264px;
  width: max-content;
  padding: 5px 9px;
  border-radius: 6px;
  background: rgba(0, 0, 0, 0.78);
  color: #fff;
  font-size: 11.5px;
  line-height: 1.45;
  text-align: center;
  white-space: normal;
  pointer-events: none;
  z-index: 40;
  box-shadow: 0 4px 14px rgba(0, 0, 0, 0.22);
}

.preset-pill {
  font-size: 12px;
  padding: 6px 13px;
  border-radius: 999px;
  border: 0.5px solid var(--divider);
  background: var(--card);
  color: var(--text-1);
}

.preset-pill:hover {
  background: var(--accent-soft);
  color: var(--accent);
}

.preset-save {
  color: var(--accent);
  background: transparent;
  border-color: transparent;
}

.preset-save:hover {
  background: var(--accent-soft);
}

/* 预设场景优先：方案条整块提到最前面，按钮和字号一起放大；关掉时一切回到原样 */
.panel-body.preset-first .preset-area {
  order: -1;
  margin-top: 0;
  margin-bottom: 10px;
}

.panel-body.preset-first .preset-bar {
  gap: 8px;
}

.panel-body.preset-first .preset-pill {
  font-size: 16px;
  font-weight: 600;
  padding: 9px 16px;
  border: 1px solid rgba(0, 0, 0, 0.16);
}

/* 「+ 保存」是次要动作，跟着放大反而抢掉场景的注意力，字号和内边距留在默认档 */
.panel-body.preset-first .preset-save {
  font-size: 12px;
  padding: 6px 13px;
  border-color: transparent;
}

/* 方案条提到顶上以后，气泡再往上就顶到标题栏了，改到下方 */
.panel-body.preset-first .preset-tip {
  top: calc(100% + 7px);
  bottom: auto;
}

.panel-body.preset-first .save-input {
  font-size: 14px;
  padding: 8px 11px;
}

.panel-body.preset-first .save-ok,
.panel-body.preset-first .save-cancel {
  font-size: 14px;
  padding: 8px 12px;
}

.panel-body.preset-first .preset-tip {
  font-size: 13px;
  max-width: 300px;
}

.save-box {
  display: flex;
  gap: 6px;
}

.save-input {
  flex: 1;
  min-width: 0;
  font: inherit;
  font-size: 12.5px;
  padding: 5px 9px;
  border: 1px solid var(--divider);
  border-radius: 7px;
  background: #fff;
  outline: none;
}

.save-input:focus {
  border-color: var(--accent);
}

.save-ok,
.save-cancel {
  font-size: 12.5px;
  padding: 5px 10px;
  border-radius: 7px;
  flex-shrink: 0;
}

.save-ok {
  background: var(--accent);
  color: #fff;
}

.save-cancel {
  color: var(--text-2);
}

.mock-note {
  text-align: center;
  font-size: 10px;
  color: var(--text-2);
  padding-top: 6px;
}

.quick-footer {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 9px 12px;
  border-top: 0.5px solid var(--divider);
  flex-shrink: 0;
  font-size: 12px;
}

.quick-hint {
  color: var(--text-2);
}

.quick-btn {
  font-size: 13px;
  padding: 6px 16px;
  border-radius: 8px;
  border: 0.5px solid var(--divider);
  background: var(--card);
  color: var(--text-1);
}

.quick-btn:hover {
  background: var(--accent-soft);
  color: var(--accent);
}

/* 退出按钮带倒数，做成主按钮：数字用等宽，3→2→1 不跳位 */
.quick-quit {
  background: var(--accent);
  border-color: transparent;
  color: #fff;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}

.quick-quit:hover {
  background: var(--accent);
  color: #fff;
  filter: brightness(1.08);
}

.error-box {
  padding: 24px 16px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.error-text {
  font-size: 12.5px;
  color: var(--text-2);
  text-align: center;
  line-height: 1.5;
}

.retry-btn {
  font-size: 12.5px;
  color: #fff;
  background: var(--accent);
  padding: 5px 16px;
  border-radius: 7px;
}
</style>
