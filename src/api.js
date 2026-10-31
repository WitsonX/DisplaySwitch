import { invoke } from '@tauri-apps/api/core'
import { t } from './i18n'

export const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

function prettyRes(res) {
  return res.replace('x', '×')
}

function normalizeState(s) {
  const externals = s.displays.filter((d) => /external/i.test(d.type_str))
  const displays = s.displays.map((d) => {
    let maxWidth = 0
    for (const m of d.modes) {
      const w = parseInt(m.res.split('x')[0], 10)
      if (w > maxWidth) maxWidth = w
    }
    const builtin = /built in/i.test(d.type_str)
    let name
    if (builtin) {
      name = t('nameBuiltin')
    } else if (/external/i.test(d.type_str)) {
      const idx = externals.indexOf(d)
      name = externals.length > 1 ? t('nameExternalN', { n: idx + 1 }) : t('nameExternal')
    } else {
      name = d.type_str
    }
    return {
      id: d.serial_id,
      name: builtin ? name : d.model || name,
      genericName: name,
      builtin,
      current: d.res,
      modes: d.modes.map((m) => {
        const w = parseInt(m.res.split('x')[0], 10)
        return {
          id: m.res,
          label: prettyRes(m.res),
          scaling: m.scaling,
          tag: w === maxWidth ? t('nativeTag') : undefined
        }
      }),
      hz: d.hz,
      color_depth: d.color_depth,
      scaling: d.scaling,
      degree: d.degree,
      enabled: d.enabled,
      main: !!d.main,
      mirrorOf: d.mirror_of || null,
      sizeIn: builtin ? null : d.size_in || null
    }
  })
  return { displays }
}

const mockState = {
  displays: [
    {
      serial_id: 's1001',
      type_str: 'MacBook built in screen',
      model: 'Color LCD',
      res: '1512x982',
      hz: 120,
      color_depth: 8,
      scaling: true,
      degree: 0,
      enabled: true,
      main: true,
      mirror_of: null,
      size_in: 14,
      modes: [
        { res: '1024x665', scaling: true },
        { res: '1147x745', scaling: true },
        { res: '1352x878', scaling: true },
        { res: '1512x982', scaling: true },
        { res: '1800x1169', scaling: true },
        { res: '3024x1964', scaling: false }
      ]
    },
    {
      serial_id: 's1002',
      type_str: '27 inch external screen',
      model: 'U2790B',
      res: '3840x2160',
      hz: 60,
      color_depth: 8,
      scaling: true,
      degree: 0,
      enabled: true,
      main: false,
      mirror_of: 's1003',
      size_in: 27,
      modes: [
        { res: '3840x2160', scaling: true },
        { res: '3360x1890', scaling: true },
        { res: '3008x1692', scaling: true },
        { res: '2560x1440', scaling: true },
        { res: '2304x1296', scaling: true },
        { res: '1920x1080', scaling: true },
        { res: '1504x846', scaling: true }
      ]
    },
    {
      serial_id: 's1003',
      type_str: '32 inch external screen',
      model: 'U3223Q',
      res: '3840x2160',
      hz: 60,
      color_depth: 8,
      scaling: true,
      degree: 0,
      enabled: true,
      main: false,
      mirror_of: null,
      size_in: 32,
      modes: [
        { res: '3840x2160', scaling: true },
        { res: '2560x1440', scaling: true },
        { res: '1920x1080', scaling: true },
        { res: '1280x720', scaling: true }
      ]
    }
  ]
}

const mockPresets = []

export async function getState() {
  if (isTauri) return normalizeState(await invoke('get_state'))
  return normalizeState(JSON.parse(JSON.stringify(mockState)))
}

export async function applyConfig(config) {
  if (isTauri) return invoke('apply_config', { config })
  await new Promise((resolve) => setTimeout(resolve, 250))
  for (const cd of config.displays) {
    const d = mockState.displays.find((m) => m.serial_id === cd.ctx_id)
    if (!d) continue
    d.main = cd.role === 'main'
    d.mirror_of = cd.role === 'mirror' ? cd.mirror_target : null
    d.res = cd.res.replace(/×/g, 'x')
    d.scaling = cd.scaling
  }
  for (const d of mockState.displays) {
    if (d.mirror_of) {
      const t = mockState.displays.find((m) => m.serial_id === d.mirror_of)
      if (t) d.res = t.res
    }
  }
}

export async function listPresets() {
  if (isTauri) {
    const list = await invoke('list_presets')
    return list.map((p) => ({ name: p.name, config: p.config }))
  }
  return JSON.parse(JSON.stringify(mockPresets))
}

export async function savePreset(name, config) {
  if (isTauri) return invoke('save_preset', { name, config })
  mockPresets.push({ name, config: JSON.parse(JSON.stringify(config)) })
}

export async function deletePreset(name) {
  if (isTauri) return invoke('delete_preset', { name })
  const i = mockPresets.findIndex((p) => p.name === name)
  if (i >= 0) mockPresets.splice(i, 1)
}

const mockSettings = { favorites: {}, pinFavorites: true, autostart: false, lang: 'auto' }

export async function getSettings() {
  if (isTauri) return invoke('get_settings')
  return JSON.parse(JSON.stringify(mockSettings))
}

export async function toggleFavorite(displayId, res) {
  if (isTauri) return invoke('toggle_favorite', { displayId, res })
  const list = mockSettings.favorites[displayId] || (mockSettings.favorites[displayId] = [])
  const i = list.indexOf(res)
  if (i >= 0) list.splice(i, 1)
  else list.push(res)
  if (!list.length) delete mockSettings.favorites[displayId]
  return JSON.parse(JSON.stringify(mockSettings))
}

export async function setPinFavorites(pin) {
  if (isTauri) return invoke('set_pin_favorites', { pin })
  mockSettings.pinFavorites = pin
  return JSON.parse(JSON.stringify(mockSettings))
}

export async function setLang(lang) {
  if (isTauri) return invoke('set_lang', { lang })
  mockSettings.lang = lang
  return JSON.parse(JSON.stringify(mockSettings))
}

export async function quitApp() {
  if (isTauri) return invoke('quit_app')
}

// 自启动的真实状态在系统的 LaunchAgent 注册里，不跟 settings.json 混存
export async function getAutostart() {
  if (isTauri) return invoke('get_autostart')
  return mockSettings.autostart
}

export async function setAutostart(enabled) {
  if (isTauri) return invoke('set_autostart', { enabled })
  mockSettings.autostart = enabled
  return enabled
}
