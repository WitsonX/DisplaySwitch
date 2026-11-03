<template>
  <div class="toast" :class="{ raised }">{{ message }}</div>
</template>

<script>
export default {
  name: 'Toast',
  props: {
    message: { type: String, required: true },
    // 快速切换底条显示时抬到条上方，免得盖住倒计时和取消按钮
    raised: { type: Boolean, default: false }
  }
}
</script>

<style scoped>
.toast {
  position: absolute;
  left: 50%;
  bottom: 14px;
  transform: translateX(-50%);
  background: rgba(30, 30, 32, 0.92);
  color: #fff;
  font-size: 12px;
  padding: 7px 14px;
  border-radius: 999px;
  white-space: nowrap;
  pointer-events: none;
}

.toast.raised {
  bottom: 58px;
}
</style>
