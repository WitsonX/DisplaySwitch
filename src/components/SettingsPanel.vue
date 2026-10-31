<template>
  <div class="settings">
    <div class="section-label">通用</div>
    <div class="card s-card">
      <label class="s-row s-toggle-row">
        <span class="s-label">
          <span class="s-name">登录 macOS 时自动启动</span>
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
          <span class="s-name">收藏的分辨率排在最前</span>
          <span class="s-desc">关闭后列表恢复原始顺序，标星保留</span>
        </span>
        <input
          type="checkbox"
          class="s-switch"
          :checked="pinFavorites"
          @change="$emit('set-pin', $event.target.checked)"
        />
      </label>
    </div>

    <div class="section-label">已存方案</div>
    <div class="card s-card">
      <div v-if="!rows.length" class="s-empty">还没有方案 · 回到面板保存当前配置</div>
      <div v-for="r in rows" :key="r.p.name" class="s-row">
        <template v-if="renaming !== r.p.name">
          <div class="s-info">
            <span class="s-name">{{ r.p.name }}</span>
            <span class="s-desc">{{ r.available ? r.p.desc : '显示器未接入' }}</span>
          </div>
          <div class="s-acts">
            <button class="s-act" @click="startRename(r.p)">改名</button>
            <button class="s-act s-act-del" @click="$emit('delete', r.p)">删除</button>
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
          <button class="s-act" @click="commitRename(r.p)">保存</button>
          <button class="s-act" @click="renaming = null">取消</button>
        </div>
      </div>
    </div>

    <div class="section-label">关于</div>
    <div class="card s-card about">
      <div class="a-row a-brand">
        <img class="a-icon" :src="appIcon" alt="DisplaySwitch 图标" />
        <div class="a-brand-text">
          <div class="a-brand-name">
            <span class="a-name">DisplaySwitch</span>
            <span class="a-ver">v{{ version }}</span>
          </div>
          <span class="a-desc">菜单栏里切换显示器角色（主显 / 镜像 / 扩展）与分辨率档位</span>
        </div>
      </div>
      <div class="a-row">
        <span class="a-k">切换驱动</span>
        <span class="a-v">内嵌 displayplacer（MIT 开源）</span>
      </div>
      <div class="a-row">
        <span class="a-k">方案存放</span>
        <span class="a-v a-path">~/Library/Application Support/com.displayswitch.app/presets/</span>
      </div>
    </div>

    <button class="s-quit" @click="$emit('quit')">退出 DisplaySwitch</button>
  </div>
</template>

<script>
import { version } from '../../package.json'
import appIcon from '../assets/app-icon.png'

export default {
  name: 'SettingsPanel',
  props: {
    presets: { type: Array, required: true },
    present: { type: Array, required: true },
    pinFavorites: { type: Boolean, default: true },
    autostart: { type: Boolean, default: false }
  },
  emits: ['delete', 'rename', 'set-pin', 'set-autostart', 'quit'],
  data() {
    return {
      version,
      appIcon,
      renaming: null,
      renameValue: ''
    }
  },
  computed: {
    rows() {
      const present = new Set(this.present)
      return this.presets.map((p) => ({
        p,
        available: p.config.displays.every((d) => present.has(d.ctx_id))
      }))
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
  gap: 7px;
}

.a-name {
  font-size: 14px;
  font-weight: 600;
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

.a-k {
  font-size: 11.5px;
  color: var(--text-2);
  width: 56px;
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
