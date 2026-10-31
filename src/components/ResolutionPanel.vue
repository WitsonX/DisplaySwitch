<template>
  <div class="card res-card" :class="{ 'res-card-fill': display && display.role !== 'mirror' }">
    <div v-if="display && display.role === 'mirror'" class="mirror-note">
      <span class="gfx gfx-mirror">{{ display.letter }}</span>
      <span class="note-text">
        {{
          $t('mirrorNote', {
            name: display.name,
            target: display.mirrorTargetName,
            res: pretty(display.mirrorTargetRes)
          })
        }}
      </span>
    </div>
    <template v-else-if="display">
      <div v-if="showHead" class="res-head">
        <span class="res-head-name">{{ display.name }}</span>
        <span v-if="display.sizeIn" class="res-size">{{ display.sizeIn }}″</span>
        <span v-if="display.hz" class="res-hz">{{ display.hz }}Hz</span>
      </div>
      <div class="res-list">
        <div
          v-for="m in rows"
          :key="m.id"
          class="res-row"
          role="button"
          tabindex="0"
          :class="{ current: m.id === display.current }"
          @click="$emit('res-select', display, m)"
          @keyup.enter="$emit('res-select', display, m)"
          @keyup.space.prevent="$emit('res-select', display, m)"
        >
          <span class="res-val">{{ m.label }}</span>
          <span v-if="m.tag" class="res-tag">{{ m.tag }}</span>
          <svg
            class="res-check"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M20 6L9 17l-5-5" />
          </svg>
          <button
            class="res-star"
            :class="{ on: isFav(m) }"
            :aria-pressed="isFav(m) ? 'true' : 'false'"
            :title="isFav(m) ? $t('unfavRes') : $t('favRes')"
            @click.stop="$emit('toggle-fav', m)"
            @keyup.enter.stop.prevent="$emit('toggle-fav', m)"
          >
            <svg
              viewBox="0 0 24 24"
              :fill="isFav(m) ? 'currentColor' : 'none'"
              stroke="currentColor"
              stroke-width="1.7"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path
                d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"
              />
            </svg>
          </button>
        </div>
      </div>
    </template>
  </div>
</template>

<script>
export default {
  name: 'ResolutionPanel',
  props: {
    display: { type: Object, default: null },
    showHead: { type: Boolean, default: true },
    favorites: { type: Array, default: () => [] },
    pin: { type: Boolean, default: true }
  },
  emits: ['res-select', 'toggle-fav'],
  computed: {
    // 置顶只改显示顺序，收藏之间仍按原来的分辨率顺序排
    rows() {
      const modes = this.display ? this.display.modes : []
      if (!this.pin || !this.favorites.length) return modes
      const fav = new Set(this.favorites)
      return [...modes.filter((m) => fav.has(m.id)), ...modes.filter((m) => !fav.has(m.id))]
    }
  },
  methods: {
    pretty(res) {
      return (res || '').replace('x', '×')
    },
    isFav(m) {
      return this.favorites.includes(m.id)
    }
  }
}
</script>

<style scoped>
.res-card {
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.res-card-fill {
  flex: 1 1 auto;
  min-height: 0;
}

.res-head {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 9px 12px;
  border-bottom: 0.5px solid var(--divider);
  font-size: 12px;
  font-weight: 500;
  color: var(--text-1);
  flex-shrink: 0;
}

.res-head-name {
  flex: 1;
  min-width: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.res-hz {
  font-size: 11px;
  font-weight: 400;
  color: var(--text-2);
}

.res-size {
  font-size: 11px;
  font-weight: 600;
  color: var(--accent);
  background: var(--accent-soft);
  padding: 2px 7px;
  border-radius: 5px;
  font-variant-numeric: tabular-nums;
}

.res-list {
  flex: 1 1 auto;
  min-height: 0;
  overflow-y: auto;
}

.res-list::-webkit-scrollbar {
  width: 0;
}

.res-row {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 7px 12px;
  text-align: left;
  cursor: pointer;
}

.res-row + .res-row {
  border-top: 0.5px solid var(--divider);
}

.res-row:hover {
  background: rgba(0, 0, 0, 0.025);
}

.res-row:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: -2px;
}

.res-val {
  font-size: 12.5px;
  font-weight: 500;
  color: var(--text-1);
  font-variant-numeric: tabular-nums;
}

.res-tag {
  font-size: 10px;
  color: var(--text-2);
  background: rgba(0, 0, 0, 0.05);
  padding: 1px 6px;
  border-radius: 4px;
}

.res-check {
  margin-left: auto;
  width: 14px;
  height: 14px;
  color: var(--accent);
  opacity: 0;
  flex-shrink: 0;
}

.res-row.current .res-check {
  opacity: 1;
}

.res-row.current .res-val {
  color: var(--accent);
}

.res-star {
  width: 18px;
  height: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  color: #f0a020;
  opacity: 0;
  border-radius: 5px;
}

.res-star svg {
  width: 13px;
  height: 13px;
}

.res-row:hover .res-star,
.res-star.on {
  opacity: 1;
}

.res-star:not(.on) {
  color: var(--text-2);
}

.res-star:hover {
  background: rgba(0, 0, 0, 0.06);
}

.res-star:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: -1px;
}

.mirror-note {
  padding: 14px 12px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  align-items: flex-start;
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

.note-text {
  font-size: 11px;
  color: var(--text-2);
  line-height: 1.6;
}
</style>
