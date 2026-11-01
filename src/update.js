// 手动「检查更新」：不接 Tauri updater，只比对最新 Release 的 tag。
// 双源顺序按界面语言定（中文优先 Gitee，国内网络更稳），一个源不可达就回落另一个。
// 发布页地址由本模块用常量 + tag 自己拼出来，不采用接口返回的 URL。

const OWNER = { github: 'WitsonX', gitee: 'Witson' }
const REPO = 'DisplaySwitch'

const SOURCES = [
  {
    id: 'gitee',
    api: `https://gitee.com/api/v5/repos/${OWNER.gitee}/${REPO}/releases/latest`,
    page: (tag) => `https://gitee.com/${OWNER.gitee}/${REPO}/releases/tag/${tag}`
  },
  {
    id: 'github',
    api: `https://api.github.com/repos/${OWNER.github}/${REPO}/releases/latest`,
    page: (tag) => `https://github.com/${OWNER.github}/${REPO}/releases/tag/${tag}`
  }
]

// tag 直接进拼接的 URL，只放过版本号该有的字符
const TAG_RE = /^v?[0-9]+(\.[0-9]+){0,3}([-.+][0-9A-Za-z.-]+)?$/

function stripV(tag) {
  return tag.replace(/^v/i, '')
}

function parseVersion(tag) {
  return stripV(tag)
    .split(/[-+]/)[0]
    .split('.')
    .map((n) => parseInt(n, 10) || 0)
}

/** a>b 返回正数，a<b 返回负数，相等 0；-beta 之类后缀不参与比较 */
export function compareVersions(a, b) {
  const va = parseVersion(a)
  const vb = parseVersion(b)
  const len = Math.max(va.length, vb.length)
  for (let i = 0; i < len; i++) {
    const diff = (va[i] ?? 0) - (vb[i] ?? 0)
    if (diff !== 0) return diff
  }
  return 0
}

async function fetchJson(url, timeoutMs = 8000) {
  const ctrl = new AbortController()
  const timer = setTimeout(() => ctrl.abort(), timeoutMs)
  try {
    const res = await fetch(url, {
      signal: ctrl.signal,
      headers: { Accept: 'application/json' },
      cache: 'no-store'
    })
    if (res.status === 404) {
      const err = new Error('no release')
      err.notFound = true
      throw err
    }
    if (!res.ok) throw new Error(`HTTP ${res.status}`)
    return await res.json()
  } finally {
    clearTimeout(timer)
  }
}

/**
 * @param preferZh 中文界面优先 Gitee
 * @param current 当前版本（不带 v）
 * @returns {status: 'available'|'upToDate'|'none'|'failed', current, latest?, url?, source?}
 *   none = 两边都还没发过 Release；failed = 网络/接口都不可达
 */
export async function checkForUpdate(preferZh, current) {
  const order = preferZh ? SOURCES : [SOURCES[1], SOURCES[0]]
  let sawNotFound = false
  let sawError = false
  for (const src of order) {
    let tag = ''
    try {
      const data = await fetchJson(src.api)
      tag = typeof data?.tag_name === 'string' ? data.tag_name.trim() : ''
      if (!TAG_RE.test(tag)) throw new Error('bad tag')
    } catch (e) {
      if (e && e.notFound) sawNotFound = true
      else sawError = true
      continue
    }
    return {
      status: compareVersions(tag, current) > 0 ? 'available' : 'upToDate',
      current,
      latest: stripV(tag),
      url: src.page(tag),
      source: src.id
    }
  }
  return { status: sawError || !sawNotFound ? 'failed' : 'none', current }
}
