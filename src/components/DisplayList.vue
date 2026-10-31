<template>
  <div class="card">
    <div
      v-for="(d, i) in displays"
      :key="d.id"
      class="d-row"
      :class="{ sel: d.id === selectedId, bt: i > 0, plain: isSingle }"
      @click="onRowClick(d)"
    >
      <div class="d-icon">
        <svg
          v-if="d.builtin"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.6"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <rect x="4" y="5" width="16" height="10" rx="1.5" />
          <path d="M2.5 18.5h19" />
        </svg>
        <svg
          v-else
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.6"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <rect x="3" y="4.5" width="18" height="11.5" rx="1.5" />
          <path d="M9 20h6M12 16v4" />
        </svg>
      </div>
      <div class="d-main">
        <div class="d-line1">
          <span class="d-name">{{ d.name }}</span>
          <span v-if="d.sizeIn" class="d-size">{{ d.sizeIn }}″</span>
          <div v-if="showRole && !useSelect" class="role-seg" @click.stop>
            <button class="seg" :class="{ on: d.role === 'main' }" @click="setRole(d, 'main')">
              主显
            </button>
            <button
              class="seg"
              :class="{ on: d.role === 'mirror' }"
              @click="toggleMirror(d)"
            >
              镜像
            </button>
            <button class="seg" :class="{ on: d.role === 'extend' }" @click="setRole(d, 'extend')">
              扩展
            </button>
            <div v-if="menuFor === d.id" class="mirror-menu">
              <button
                v-for="t in mirrorTargets(d)"
                :key="t.id"
                class="menu-item"
                @click="pickMirror(d, t)"
              >
                与 {{ short(t.name) }} [{{ t.letter }}] 镜像
              </button>
            </div>
          </div>
          <select
            v-else-if="showRole"
            class="role-select"
            :class="{ 'role-main': d.role === 'main' }"
            :value="roleValue(d)"
            @click.stop
            @change="$emit('role-change', d, $event.target.value)"
          >
            <option v-for="opt in d.roleOptions" :key="opt.value" :value="opt.value">
              {{ opt.label }}
            </option>
          </select>
          <span v-else class="role-static">主显示器</span>
        </div>
        <div class="d-line2">
          <span
            v-if="!isSingle"
            class="gfx"
            :class="{ 'gfx-mirror': d.role === 'mirror' }"
            :title="d.role === 'mirror' ? `内容与 ${d.mirrorTargetName} 相同` : '这块屏显示的内容'"
            >{{ d.letter }}</span
          >
          <span class="d-sub">{{ d.subText }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
export default {
  name: 'DisplayList',
  props: {
    displays: { type: Array, required: true },
    selectedId: { type: String, default: '' },
    showRole: { type: Boolean, default: true }
  },
  emits: ['select', 'role-change'],
  data() {
    return { menuFor: null }
  },
  computed: {
    isSingle() {
      return this.displays.length <= 1
    },
    // 三屏及以上时镜像候选可能多于一个，分段按钮要塞不下，改用下拉直接列全
    useSelect() {
      return this.displays.length >= 3
    }
  },
  methods: {
    roleValue(d) {
      return d.role === 'mirror' ? `mirror:${d.mirrorOf}` : d.role
    },
    short(name) {
      return (name || '').replace('显示器', '')
    },
    mirrorTargets(d) {
      return this.displays.filter((o) => o.id !== d.id && o.role !== 'mirror')
    },
    onRowClick(d) {
      if (this.isSingle) return
      this.menuFor = null
      this.$emit('select', d.id)
    },
    setRole(d, val) {
      this.menuFor = null
      this.$emit('role-change', d, val)
    },
    toggleMirror(d) {
      const targets = this.mirrorTargets(d)
      if (!targets.length) return
      if (d.role === 'mirror') {
        if (targets.length > 1) this.menuFor = this.menuFor === d.id ? null : d.id
        return
      }
      if (targets.length === 1) this.setRole(d, `mirror:${targets[0].id}`)
      else this.menuFor = this.menuFor === d.id ? null : d.id
    },
    pickMirror(d, t) {
      this.setRole(d, `mirror:${t.id}`)
    }
  }
}
</script>

<style scoped>
.card {
  overflow: visible;
}

.d-row:first-child {
  border-radius: var(--radius) var(--radius) 0 0;
}

.d-row:last-child {
  border-radius: 0 0 var(--radius) var(--radius);
}

.d-row:only-child {
  border-radius: var(--radius);
}

.d-row {
  display: flex;
  gap: 10px;
  padding: 10px 12px;
  cursor: pointer;
  border-left: 2px solid transparent;
}

.d-row.bt {
  border-top: 0.5px solid var(--divider);
}

.d-row.plain {
  cursor: default;
}

.d-row.sel {
  background: var(--accent-soft);
  border-left-color: var(--accent);
}

.d-row.sel.bt {
  border-top-color: transparent;
}

.d-icon {
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.04);
  border-radius: 8px;
  color: #555;
  flex-shrink: 0;
}

.d-icon svg {
  width: 17px;
  height: 17px;
}

.d-main {
  flex: 1;
  min-width: 0;
}

.d-line1 {
  display: flex;
  align-items: center;
  gap: 8px;
}

.d-name {
  flex: 1;
  min-width: 0;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.d-size {
  font-size: 11px;
  font-weight: 600;
  color: var(--accent);
  background: var(--accent-soft);
  padding: 2px 7px;
  border-radius: 5px;
  flex-shrink: 0;
  font-variant-numeric: tabular-nums;
}

.role-select {
  font: inherit;
  font-size: 11px;
  padding: 3px 6px;
  border-radius: 7px;
  border: 1px solid var(--divider);
  background: rgba(0, 0, 0, 0.04);
  color: var(--text-1);
  cursor: pointer;
  max-width: 150px;
  flex-shrink: 0;
  outline: none;
}

.role-select.role-main {
  border-color: var(--accent);
  background: var(--accent-soft);
  color: var(--accent);
  font-weight: 500;
}

.role-seg {
  position: relative;
  display: flex;
  gap: 2px;
  padding: 2px;
  background: rgba(0, 0, 0, 0.05);
  border-radius: 8px;
  flex-shrink: 0;
}

.seg {
  font-size: 11px;
  padding: 2.5px 9px;
  border-radius: 6px;
  color: var(--text-2);
}

.seg:hover {
  color: var(--text-1);
}

.seg.on {
  background: var(--card);
  color: var(--accent);
  font-weight: 600;
  box-shadow: 0 0.5px 2px rgba(0, 0, 0, 0.14);
}

.mirror-menu {
  position: absolute;
  top: calc(100% + 4px);
  right: 0;
  z-index: 30;
  min-width: 118px;
  padding: 4px;
  background: var(--card);
  border-radius: 9px;
  box-shadow: 0 6px 22px rgba(0, 0, 0, 0.16), 0 0 0 0.5px rgba(0, 0, 0, 0.08);
}

.menu-item {
  display: block;
  width: 100%;
  padding: 6px 10px;
  text-align: left;
  font-size: 12px;
  border-radius: 6px;
  color: var(--text-1);
  white-space: nowrap;
}

.menu-item:hover {
  background: var(--accent-soft);
  color: var(--accent);
}

.role-static {
  font-size: 11px;
  color: var(--text-2);
  flex-shrink: 0;
}

.d-line2 {
  display: flex;
  align-items: center;
  gap: 9px;
  margin-top: 7px;
}

.gfx {
  width: 52px;
  height: 34px;
  border-radius: 6px;
  border: 1.5px solid var(--accent);
  background: var(--accent-soft);
  color: var(--accent);
  font-size: 16px;
  font-weight: 600;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.gfx-mirror {
  border-style: dashed;
}

.d-sub {
  font-size: 11px;
  color: var(--text-2);
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
