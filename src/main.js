import { createApp } from 'vue'
import App from './App.vue'
import { isTauri } from './api'
import { t, i18n, setLangPref } from './i18n'
import './style.css'

if (!isTauri) document.body.classList.add('browser-preview')
// 把探测到的语言写进 <html lang>，读屏软件与 CSS 的 :lang 才和面板一致
setLangPref(i18n.pref)

const app = createApp(App)
// 模板里满屏都是 $t(...)；组件不再各自 import，语言切换也靠它触发重渲染
app.config.globalProperties.$t = t
app.mount('#app')
