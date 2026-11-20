import { createApp } from 'vue'
import App from './App.vue'

// WebView 自带「刷新 / 打印 / 检查」等原生右键菜单，与桌面宠物场景不符。
// 全局屏蔽默认右键菜单，改由 App.vue 里的自定义菜单接管。
window.addEventListener('contextmenu', (e) => e.preventDefault())

createApp(App).mount('#app')
