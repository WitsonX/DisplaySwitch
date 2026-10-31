import { createApp } from 'vue'
import App from './App.vue'
import { isTauri } from './api'
import './style.css'

if (!isTauri) document.body.classList.add('browser-preview')

createApp(App).mount('#app')
