import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
  // 绿色版（canary）构建把提交号注入前端，用于在「关于」卡片标出构建来源；
  // 正式 release 不设置该环境变量，注入为空串，界面上不显示。
  define: {
    __DISPLAYSWITCH_BUILD__: JSON.stringify(process.env.DISPLAYSWITCH_BUILD ?? '')
  },
  server: {
    port: 5173,
    strictPort: true
  }
})
