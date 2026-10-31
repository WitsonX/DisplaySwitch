<template>
  <div class="toast">{{ message }}</div>
</template>

<script>
export default {
  name: 'Toast',
  props: {
    message: { type: String, required: true }
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
</style>
