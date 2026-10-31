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
          返回
        </button>
        <div class="title title-sub">设置</div>
        <span class="head-spacer" />
      </template>
      <template v-else>
        <div class="title">
          <!-- 与 tools/gen_icons.swift 的 markStacked 同一套几何（18 单位画布，y 轴向下） -->
          <svg
            width="17"
            height="17"
            viewBox="0 0 18 18"
            fill="none"
            stroke="currentColor"
            stroke-width="1.35"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <mask id="ds-title-knock">
              <rect x="0" y="0" width="18" height="18" fill="#fff" />
              <rect x="5.1" y="5.5" width="12.6" height="9.6" rx="2.2" fill="#000" />
            </mask>
            <rect x="1.4" y="2.4" width="8.8" height="6.2" rx="1.6" mask="url(#ds-title-knock)" />
            <rect x="6.2" y="6.6" width="10.4" height="7.4" rx="1.6" />
            <path d="M11.4 14v2.2M9.6 16.2h3.6" />
          </svg>
          显示器
        </div>
        <span v-if="statusText" class="status-pill">{{ statusText }}</span>
        <button class="gear-btn" title="设置" @click="view = 'settings'">
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
      <button class="close-btn" title="关闭" @click="closePanel">✕</button>
    </header>

    <SettingsPanel
      v-if="view === 'settings'"
      :presets="allPresets"
      :present="presentIds"
      :pin-favorites="pinFavorites"
      :autostart="autostart"
      @delete="removePreset"
      @rename="renamePreset"
      @set-pin="setPin"
      @set-autostart="changeAutostart"
      @quit="quitApp"
    />

    <div v-else-if="error" class="error-box">
      <p class="error-text">{{ error }}</p>
      <button class="retry-btn" @click="load">重试</button>
    </div>

    <div v-else class="panel-body" :class="{ single: isSingle }">
      <div class="cols">
        <div class="col col-left">
          <div class="section-label">显示器</div>
          <DisplayList
            :displays="enriched"
            :selected-id="isSingle ? '' : selectedId"
            :show-role="!isSingle"
            @select="selectedId = $event"
            @role-change="setRole"
          />
        </div>
        <div v-if="!isSingle" class="col col-right">
          <div class="section-label">分辨率</div>
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
        <div class="section-label single-label">分辨率</div>
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
        <div class="section-label">场景预设</div>
        <div v-if="saving" class="save-box">
          <input
            ref="saveInput"
            :value="newPresetName"
            class="save-input"
            placeholder="预设名称，如：工位-扩展"
            @input="newPresetName = $event.target.value"
            @keyup.enter="savePreset"
            @keyup.esc="cancelSave"
          />
          <button class="save-ok" @click="savePreset">保存</button>
          <button class="save-cancel" @click="cancelSave">取消</button>
        </div>
        <div v-else class="preset-bar">
          <button
            v-for="p in presets"
            :key="p.name"
            class="preset-pill"
            @click="applyPreset(p)"
            @mouseenter="showTip(p, $event)"
            @mouseleave="hideTip"
            @focus="showTip(p, $event)"
            @blur="hideTip"
            >{{ p.name }}</button
          >
          <button class="preset-pill preset-save" @click="toggleSave">+ 保存当前配置…</button>
          <div v-if="tip" class="preset-tip" :style="{ left: tip.left + 'px' }">
            {{ tip.desc }}
          </div>
        </div>
        <div v-if="isMock" class="mock-note">浏览器预览 · 假数据 · 未接入真实切换</div>
      </div>
    </div>

    <Toast v-if="toast" :message="toast" />
  </div>
</template>

<script>
import DisplayList from './components/DisplayList.vue'
import ResolutionPanel from './components/ResolutionPanel.vue'
import SettingsPanel from './components/SettingsPanel.vue'
import Toast from './components/Toast.vue'
import * as api from './api'

export default {
  name: 'App',
  components: { DisplayList, ResolutionPanel, SettingsPanel, Toast },
  data() {
    return {
      isMock: !api.isTauri,
      view: 'main',
      displays: [],
      selectedId: '',
      allPresets: [],
      favorites: {},
      pinFavorites: true,
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
    isSingle() {
      return this.displays.length <= 1
    },
    presentIds() {
      return this.displays.map((d) => d.id)
    },
    presets() {
      const present = new Set(this.presentIds)
      // 方案记录的显示器没全插上就不出现，避免应用出半套配置
      return this.allPresets.filter((p) => p.config.displays.every((d) => present.has(d.ctx_id)))
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
            ? `跟随 ${shortName(target.name)} · ${pretty(target.current)}`
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
        return `单屏 · ${pretty(this.displays[0].current)}`
      }
      const main = this.displays.find((d) => d.main) || this.displays[0]
      const mirrors = this.displays.filter((d) => d.mirrorOf).length
      return (
        `${this.displays.length}屏 · 主 ${shortName(main.name)}` +
        (mirrors ? ` · ${mirrors}屏镜像` : '')
      )
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
      const opts = [{ value: 'main', label: '主显示器' }]
      for (const o of this.displays) {
        if (o.id === d.id || o.mirrorOf) continue
        opts.push({
          value: `mirror:${o.id}`,
          label: `与 ${shortName(o.name)} [${this.letterMap[o.id]}] 镜像`
        })
      }
      opts.push({ value: 'extend', label: '扩展显示器' })
      return opts
    },
    async load() {
      this.error = null
      try {
        const state = await api.getState()
        this.displays = state.displays
        if (!this.displays.some((d) => d.id === this.selectedId)) {
          const main = this.displays.find((d) => d.main)
          this.selectedId = main ? main.id : (this.displays[0] && this.displays[0].id) || ''
        }
        this.applySettings(await api.getSettings())
        const saved = await api.listPresets()
        this.allPresets = saved.map((p) => ({
          name: p.name,
          desc: this.descOf(p.config),
          config: p.config
        }))
      } catch (e) {
        this.error = `无法读取显示器状态：${e}`
      }
    },
    applySettings(s) {
      this.favorites = s.favorites || {}
      this.pinFavorites = s.pinFavorites !== false
    },
    favList(display) {
      return (display && this.favorites[display.id]) || []
    },
    async toggleFav(display, m) {
      if (!display) return
      try {
        this.applySettings(await api.toggleFavorite(display.id, m.id))
      } catch (e) {
        this.showToast(`收藏失败：${e}`)
      }
    },
    async setPin(pin) {
      try {
        this.applySettings(await api.setPinFavorites(pin))
      } catch (e) {
        this.showToast(`设置失败：${e}`)
      }
    },
    async refreshAutostart() {
      try {
        this.autostart = await api.getAutostart()
      } catch (e) {
        this.showToast(`读取自启动状态失败：${e}`)
      }
    },
    async changeAutostart(enabled) {
      try {
        this.autostart = await api.setAutostart(enabled)
        if (this.isMock) this.showToast(enabled ? '已设为登录自启动（演示）' : '已取消登录自启动（演示）')
      } catch (e) {
        this.showToast(`自启动设置失败：${e}`)
        // 先承认复选框当前显示的样子，再回读真实值，否则失败时勾会停在错的位置
        this.autostart = enabled
        await this.$nextTick()
        this.refreshAutostart()
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
      const parts = [name ? `${shortName(name)}为主` : '自定义主屏']
      if (mirrors) parts.push(`${mirrors}屏镜像`)
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
      try {
        await api.applyConfig(this.draftToConfig(draft))
        this.showToast(this.isMock ? `${okMsg}（演示）` : okMsg)
      } catch (e) {
        this.showToast(`切换失败：${e}`)
      }
      await this.load()
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
        val === 'main' ? '主显示器' : val === 'extend' ? '扩展显示器' : '镜像屏'
      this.pushDraft(draft, `已把 ${d.name} 设为${label}`)
    },
    applyRes(display, m) {
      const draft = this.toDraft()
      const me = draft.find((x) => x.id === display.id)
      me.res = m.id
      me.scaling = m.scaling !== undefined ? m.scaling : me.scaling
      this.pushDraft(draft, `已切换 ${display.name} 到 ${m.label}`)
    },
    showTip(p, e) {
      const el = e.currentTarget
      const pill = el.getBoundingClientRect()
      const bar = el.closest('.preset-bar').getBoundingClientRect()
      const half = 132
      const center = pill.left - bar.left + pill.width / 2
      const maxLeft = Math.max(half, bar.width - half)
      this.tip = { desc: p.desc, left: Math.min(Math.max(center, half), maxLeft) }
    },
    hideTip() {
      this.tip = null
    },
    async applyPreset(p) {
      try {
        await api.applyConfig(p.config)
        this.showToast(`已应用「${p.name}」`)
        await this.load()
      } catch (e) {
        this.showToast(`应用失败：${e}`)
      }
    },
    async removePreset(p) {
      try {
        await api.deletePreset(p.name)
        this.showToast(`已删除「${p.name}」`)
        await this.load()
      } catch (e) {
        this.showToast(`删除失败：${e}`)
      }
    },
    async renamePreset(p, newName) {
      const name = newName.trim()
      if (!name || name === p.name) return
      try {
        await api.savePreset(name, p.config)
        await api.deletePreset(p.name)
        this.showToast(`已改名为「${name}」`)
        await this.load()
      } catch (e) {
        this.showToast(`改名失败：${e}`)
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
        this.showToast('请输入预设名称')
        return
      }
      try {
        await api.savePreset(name, this.draftToConfig(this.toDraft()))
        this.saving = false
        this.newPresetName = ''
        this.showToast(`已保存「${name}」`)
        await this.load()
      } catch (e) {
        this.showToast(`保存失败：${e}`)
      }
    },
    async closePanel() {
      if (api.isTauri) {
        const { invoke } = await import('@tauri-apps/api/core')
        try {
          await invoke('hide_panel')
        } catch {
          /* 忽略 */
        }
      } else {
        this.showToast('演示：关闭面板')
      }
    },
    async quitApp() {
      if (!api.isTauri) {
        this.showToast('演示：退出应用')
        return
      }
      try {
        await api.quitApp()
      } catch {
        this.showToast('退出失败')
      }
    }
  }
}

function pretty(res) {
  return res.replace('x', '×')
}

function shortName(name) {
  return name.replace('显示器', '').trim()
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
  padding: 14px 16px 10px;
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

.title svg {
  color: var(--accent);
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
  border-style: dashed;
  color: var(--accent);
  background: transparent;
}

.preset-save:hover {
  background: var(--accent-soft);
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
