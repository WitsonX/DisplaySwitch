// 图标生成器：一份几何描述同时产出菜单栏模板 glyph 与 App 图标，保证两处形状一致。
// 用法：swift tools/gen_icons.swift [输出目录]
//   产出 <输出目录>/tray-template.png、app-icon-1024.png、preview-*.png（CHOSEN 那一版，直接进仓库）
//   以及 candidates/ 下全部备选（含对比页 index.html）
import AppKit

let outDir = CommandLine.arguments.count > 1 ? CommandLine.arguments[1] : "tools/out"
let candDir = "\(outDir)/candidates"
try? FileManager.default.createDirectory(atPath: outDir, withIntermediateDirectories: true)
try? FileManager.default.createDirectory(atPath: candDir, withIntermediateDirectories: true)

let sRGB = CGColorSpace(name: CGColorSpace.sRGB)!

func col(_ hex: Int, _ a: CGFloat = 1) -> CGColor {
  CGColor(
    colorSpace: sRGB,
    components: [CGFloat((hex >> 16) & 0xFF) / 255, CGFloat((hex >> 8) & 0xFF) / 255, CGFloat(hex & 0xFF) / 255, a]
  )!
}

// ---------- 几何：都画在 18×18 画布里（y 轴向上） ----------
typealias Ctx = CGContext
typealias Knock = (CGRect) -> Void

func unit(_ box: CGRect, _ s: CGFloat, _ x: CGFloat, _ y: CGFloat, _ w: CGFloat, _ h: CGFloat) -> CGRect {
  CGRect(x: box.minX + x * s, y: box.minY + y * s, width: w * s, height: h * s)
}

func roundPath(_ box: CGRect, _ s: CGFloat, _ x: CGFloat, _ y: CGFloat, _ w: CGFloat, _ h: CGFloat, _ r: CGFloat) -> CGPath {
  CGPath(roundedRect: unit(box, s, x, y, w, h), cornerWidth: r * s, cornerHeight: r * s, transform: nil)
}

func setStroke(_ ctx: Ctx, _ stroke: CGFloat, _ color: CGColor) {
  ctx.setStrokeColor(color)
  ctx.setLineWidth(stroke)
  ctx.setLineCap(.round)
  ctx.setLineJoin(.round)
}

/// 显示器外框 + 底座（A/D 共用）
func monitorFrame(_ ctx: Ctx, _ box: CGRect, _ s: CGFloat, _ stroke: CGFloat, _ color: CGColor) {
  monitorAt(ctx, box, s, stroke, color, 1.6, 5.2, 14.8, 11.2, 1.9, neck: 2.6, base: 2.6)
}
func monitorAt(_ ctx: Ctx, _ box: CGRect, _ s: CGFloat, _ stroke: CGFloat, _ color: CGColor,
               _ x: CGFloat, _ y: CGFloat, _ w: CGFloat, _ h: CGFloat, _ r: CGFloat,
               neck: CGFloat = 1.4, base: CGFloat = 1.7) {
  func pt(_ ax: CGFloat, _ ay: CGFloat) -> CGPoint { CGPoint(x: box.minX + ax * s, y: box.minY + ay * s) }
  let cx = x + w / 2
  setStroke(ctx, stroke, color)
  ctx.addPath(roundPath(box, s, x, y, w, h, r))
  ctx.strokePath()
  ctx.move(to: pt(cx, y)); ctx.addLine(to: pt(cx, y - neck)); ctx.strokePath()
  ctx.move(to: pt(cx - base, y - neck)); ctx.addLine(to: pt(cx + base, y - neck)); ctx.strokePath()
}

func swapArrow(_ ctx: Ctx, _ box: CGRect, _ s: CGFloat, _ stroke: CGFloat, _ color: CGColor) {
  setStroke(ctx, stroke, color)
  let midY = box.minY + 10.8 * s
  ctx.move(to: CGPoint(x: box.minX + 5.4 * s, y: midY))
  ctx.addLine(to: CGPoint(x: box.minX + 12.6 * s, y: midY))
  ctx.strokePath()
  ctx.move(to: CGPoint(x: box.minX + 7.2 * s, y: midY + 1.8 * s))
  ctx.addLine(to: CGPoint(x: box.minX + 5.4 * s, y: midY))
  ctx.addLine(to: CGPoint(x: box.minX + 7.2 * s, y: midY - 1.8 * s))
  ctx.strokePath()
  ctx.move(to: CGPoint(x: box.minX + 10.8 * s, y: midY - 1.8 * s))
  ctx.addLine(to: CGPoint(x: box.minX + 12.6 * s, y: midY))
  ctx.addLine(to: CGPoint(x: box.minX + 10.8 * s, y: midY + 1.8 * s))
  ctx.strokePath()
}

func loopArrow(_ ctx: Ctx, _ box: CGRect, _ s: CGFloat, _ stroke: CGFloat, _ color: CGColor) {
  setStroke(ctx, stroke, color)
  let c = CGPoint(x: box.minX + 9 * s, y: box.minY + 11.2 * s)
  let r = 2.7 * s
  ctx.addArc(center: c, radius: r, startAngle: .pi / 4, endAngle: .pi * 7 / 4, clockwise: false)
  ctx.strokePath()
  // 端点在 315°（右下），逆时针运动的切线指向右上
  let tip = CGPoint(x: c.x + r * 0.7071, y: c.y - r * 0.7071)
  let dir = CGPoint(x: 0.7071, y: 0.7071)
  let head = 2.0 * s
  let back = CGPoint(x: tip.x - dir.x * head, y: tip.y - dir.y * head)
  ctx.move(to: CGPoint(x: back.x - dir.y * head * 0.55, y: back.y + dir.x * head * 0.55))
  ctx.addLine(to: tip)
  ctx.addLine(to: CGPoint(x: back.x + dir.y * head * 0.55, y: back.y - dir.x * head * 0.55))
  ctx.strokePath()
}

// 四款图形
func markSwap(_ ctx: Ctx, _ box: CGRect, _ stroke: CGFloat, _ primary: CGColor, _: CGColor, _: Knock) {
  let s = box.width / 18.0
  monitorFrame(ctx, box, s, stroke, primary)
  swapArrow(ctx, box, s, stroke, primary)
}

func markTwoScreens(_ ctx: Ctx, _ box: CGRect, _ stroke: CGFloat, _ primary: CGColor, _: CGColor, _: Knock) {
  let s = box.width / 18.0
  // 左：外接显示器（带底座） 右：笔电（屏 + 实心键盘座）——就是本工具的真实场景
  setStroke(ctx, stroke, primary)
  ctx.addPath(roundPath(box, s, 0.8, 7.6, 8.8, 6.6, 1.4))
  ctx.strokePath()
  ctx.move(to: CGPoint(x: box.minX + 5.2 * s, y: box.minY + 7.6 * s))
  ctx.addLine(to: CGPoint(x: box.minX + 5.2 * s, y: box.minY + 6.0 * s))
  ctx.strokePath()
  ctx.move(to: CGPoint(x: box.minX + 3.4 * s, y: box.minY + 5.6 * s))
  ctx.addLine(to: CGPoint(x: box.minX + 7.0 * s, y: box.minY + 5.6 * s))
  ctx.strokePath()
  ctx.addPath(roundPath(box, s, 11.4, 8.6, 5.4, 4.4, 1.0))
  ctx.strokePath()
  ctx.addPath(roundPath(box, s, 10.6, 6.6, 7.0, 1.4, 0.7))
  ctx.setFillColor(primary)
  ctx.fillPath()
}

func markLoop(_ ctx: Ctx, _ box: CGRect, _ stroke: CGFloat, _ primary: CGColor, _: CGColor, _: Knock) {
  let s = box.width / 18.0
  monitorFrame(ctx, box, s, stroke, primary)
  loopArrow(ctx, box, s, stroke, primary)
}

// ---------- C 系：石墨底 + 斜向叠放 ----------
// 复制图标的特征是「两个等大圆角矩形斜向重叠」，C 用前屏下面那根横线破掉它：
// 加了横线，前面那块就读成一台显示器，而不是被复制出来的副本。

/// C：两块屏斜向叠放 + 挖缝；前屏下面加一根横线（可选立柱）→ 前屏读作显示器，破掉复制轮廓
enum Stand { case bare, line, neckAndLine }

func markStacked(_ ctx: Ctx, _ box: CGRect, _ stroke: CGFloat, _ primary: CGColor, _ accent: CGColor, _ knock: Knock, stand: Stand) {
  let s = box.width / 18.0
  let k = stroke * 0.9
  let cx: CGFloat = 11.4
  let baseY: CGFloat = 1.8
  // 加了底座整组要上移，否则底部留白过大；没有底座时回到原来的位置
  let dy: CGFloat = stand == .bare ? -1.0 : 0
  setStroke(ctx, k, primary)
  ctx.addPath(roundPath(box, s, 1.4, 9.4 + dy, 8.8, 6.2, 1.6))
  ctx.strokePath()
  knock(unit(box, s, 6.2 - 1.1, 4.0 + dy - 1.1, 10.4 + 2.2, 7.4 + 2.2))
  setStroke(ctx, k, accent)
  ctx.addPath(roundPath(box, s, 6.2, 4.0 + dy, 10.4, 7.4, 1.6))
  ctx.strokePath()
  guard stand != .bare else { return }
  if stand == .neckAndLine {
    ctx.move(to: CGPoint(x: box.minX + cx * s, y: box.minY + 4.0 * s))
    ctx.addLine(to: CGPoint(x: box.minX + cx * s, y: box.minY + baseY * s))
    ctx.strokePath()
  }
  ctx.move(to: CGPoint(x: box.minX + (cx - 1.8) * s, y: box.minY + baseY * s))
  ctx.addLine(to: CGPoint(x: box.minX + (cx + 1.8) * s, y: box.minY + baseY * s))
  ctx.strokePath()
}

// ---------- 配色方案 ----------
struct Scheme {
  let id: String
  let title: String
  let note: String
  let top: Int
  let bottom: Int
  let primary: Int
  let accent: Int
  let mark: (Ctx, CGRect, CGFloat, CGColor, CGColor, Knock) -> Void
}

let schemes: [Scheme] = [
  Scheme(id: "A-swap-blue", title: "A · 屏内双向箭头（当前）", note: "显示器 + ↔，蓝渐变白形",
         top: 0x4C9BFF, bottom: 0x0B52CE, primary: 0xFFFFFF, accent: 0xFFFFFF, mark: markSwap),
  Scheme(id: "B-two-blue", title: "B · 笔电 + 外接屏", note: "本工具的真实场景：一台内建一台外接",
         top: 0x4C9BFF, bottom: 0x0B52CE, primary: 0xFFFFFF, accent: 0xFFFFFF, mark: markTwoScreens),
  Scheme(id: "D-loop-teal", title: "D · 屏内循环箭头", note: "强调「一键切换」的动作感",
         top: 0x2FCFC0, bottom: 0x0B6FA8, primary: 0xFFFFFF, accent: 0xFFFFFF, mark: markLoop),
  Scheme(id: "E-swap-light", title: "E · 浅底蓝字", note: "贴系统设置图标风格，浅色桌面最融",
         top: 0xFFFFFF, bottom: 0xDCE6F5, primary: 0x0A6CF0, accent: 0x0A6CF0, mark: markSwap),
  Scheme(id: "F-swap-night", title: "F · 深夜蓝 + 亮青", note: "对比最强，深色桌面里最跳",
         top: 0x1D2E52, bottom: 0x060A14, primary: 0x74E3FF, accent: 0x74E3FF, mark: markSwap),
  Scheme(id: "C-plain", title: "C0 · 原样（对照）", note: "两块屏斜向叠放 —— 构图是你选的，但轮廓正好等于复制图标",
         top: 0x5C5C63, bottom: 0x191A1E, primary: 0xFFFFFF, accent: 0xFFFFFF,
         mark: { c, b, k, p, a, kn in markStacked(c, b, k, p, a, kn, stand: .bare) }),
  Scheme(id: "C-line", title: "C1 · 前屏下面加一根横线", note: "横线当底座，前面那块立刻读成一台显示器而不是副本；笔画最少，18pt 最干净",
         top: 0x5C5C63, bottom: 0x191A1E, primary: 0xFFFFFF, accent: 0xFFFFFF,
         mark: { c, b, k, p, a, kn in markStacked(c, b, k, p, a, kn, stand: .line) }),
  Scheme(id: "C-stand", title: "C2 · 横线 + 立柱", note: "底座画完整，显示器感最强，代价是菜单栏里多一根竖笔",
         top: 0x5C5C63, bottom: 0x191A1E, primary: 0xFFFFFF, accent: 0xFFFFFF,
         mark: { c, b, k, p, a, kn in markStacked(c, b, k, p, a, kn, stand: .neckAndLine) }),
]

let chosen = "C-stand"

/// 本次对比页要展示的版本
let sheetIds = ["C-plain", "C-line", "C-stand"]

// ---------- 渲染 ----------
func context(_ w: Int, _ h: Int) -> Ctx {
  Ctx(data: nil, width: w, height: h, bitsPerComponent: 8, bytesPerRow: 0, space: sRGB,
      bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue)!
}

func save(_ ctx: Ctx, _ path: String) {
  let dest = CGImageDestinationCreateWithURL(URL(fileURLWithPath: path) as CFURL, "public.png" as CFString, 1, nil)!
  CGImageDestinationAddImage(dest, ctx.makeImage()!, nil)
  if !CGImageDestinationFinalize(dest) { FileHandle.standardError.write("写入失败 \(path)\n".data(using: .utf8)!) }
}

func gradient(_ sc: Scheme) -> CGGradient {
  CGGradient(colorsSpace: sRGB, colors: [col(sc.top), col(sc.bottom)] as CFArray, locations: [0, 1])!
}

/// App 图标：size 画布，824/1024 圆角方 + glyph
func renderIcon(_ sc: Scheme, _ size: Int) -> Ctx {
  let ctx = context(size, size)
  ctx.clear(.init(x: 0, y: 0, width: size, height: size))
  let k = CGFloat(size) / 1024.0
  let art = CGRect(x: 100 * k, y: 100 * k, width: 824 * k, height: 824 * k)
  let g = gradient(sc)
  func paintBg(_ rect: CGRect) {
    ctx.saveGState()
    ctx.addPath(CGPath(roundedRect: rect, cornerWidth: rect.width * 0.225, cornerHeight: rect.height * 0.225, transform: nil))
    ctx.clip()
    ctx.drawLinearGradient(g, start: CGPoint(x: 0, y: size), end: CGPoint(x: 0, y: 0), options: [])
    ctx.restoreGState()
  }
  paintBg(art)
  let box = CGRect(x: 100 * k + (824 - 520) / 2 * k, y: 100 * k + (824 - 520) / 2 * k, width: 520 * k, height: 520 * k)
  let knock: Knock = { rect in
    ctx.saveGState()
    ctx.addPath(CGPath(roundedRect: rect, cornerWidth: rect.width * 0.21, cornerHeight: rect.height * 0.21, transform: nil))
    ctx.clip()
    ctx.drawLinearGradient(g, start: CGPoint(x: 0, y: size), end: CGPoint(x: 0, y: 0), options: [])
    ctx.restoreGState()
  }
  sc.mark(ctx, box, 48 * k, col(sc.primary), col(sc.accent), knock)
  return ctx
}

/// 菜单栏 glyph：36×36（18pt @2x），纯黑 + alpha，重叠处用 clear 挖缝
func renderTrayGlyph(_ sc: Scheme) -> Ctx {
  let ctx = context(36, 36)
  ctx.clear(.init(x: 0, y: 0, width: 36, height: 36))
  let knock: Knock = { rect in
    ctx.saveGState()
    ctx.setBlendMode(.clear)
    ctx.fill(rect)
    ctx.restoreGState()
  }
  sc.mark(ctx, CGRect(x: 0, y: 0, width: 36, height: 36), 3.0, col(0x000000), col(0x000000), knock)
  return ctx
}

/// 深浅两条菜单栏预览
func renderStrip(_ sc: Scheme, dark: Bool) -> Ctx {
  let ctx = context(420, 60)
  ctx.setFillColor(col(dark ? 0x2A2A2E : 0xEDEDF0))
  ctx.fill(.init(x: 0, y: 0, width: 420, height: 60))
  let ink = dark ? 0xFFFFFF : 0x1D1D1F
  let bg = dark ? 0x2A2A2E : 0xEDEDF0
  let knock: Knock = { rect in
    ctx.addPath(CGPath(roundedRect: rect, cornerWidth: rect.width * 0.21, cornerHeight: rect.height * 0.21, transform: nil))
    ctx.setFillColor(col(bg))
    ctx.fillPath()
  }
  for (i, px) in [18, 22, 32].enumerated() {
    let box = CGRect(x: CGFloat(26 + i * 120), y: 30 - CGFloat(px) / 2, width: CGFloat(px), height: CGFloat(px))
    sc.mark(ctx, box, CGFloat(px) * 3.0 / 36.0, col(ink), col(ink), knock)
  }
  return ctx
}

func b64(_ path: String) -> String {
  (try! Data(contentsOf: URL(fileURLWithPath: path))).base64EncodedString()
}

// 1) 选定版本 → 仓库要用的产物
let chosenScheme = schemes.first { $0.id == chosen }!
save(renderTrayGlyph(chosenScheme), "\(outDir)/tray-template.png")
save(renderIcon(chosenScheme, 1024), "\(outDir)/app-icon-1024.png")
// 前端「关于」面板要用的 App 图标（Vite 打进 dist）；从仓库根目录跑生成器
try? FileManager.default.createDirectory(atPath: "src/assets", withIntermediateDirectories: true)
save(renderIcon(chosenScheme, 256), "src/assets/app-icon.png")
save(renderStrip(chosenScheme, dark: true), "\(outDir)/preview-dark.png")
save(renderStrip(chosenScheme, dark: false), "\(outDir)/preview-light.png")

// 2) 对比页选中的版本（1024 落盘待选，页面内嵌 256 缩略 + 深色菜单栏预览）
var cards = ""
for sc in schemes where sheetIds.contains(sc.id) {
  let png256 = "\(candDir)/\(sc.id)-256.png"
  let strip = "\(candDir)/\(sc.id)-strip.png"
  save(renderIcon(sc, 1024), "\(candDir)/\(sc.id)-1024.png")
  save(renderIcon(sc, 256), png256)
  save(renderTrayGlyph(sc), "\(candDir)/\(sc.id)-tray.png")
  save(renderStrip(sc, dark: true), strip)
  cards += """
  <figure><div class="icon"><img src="data:image/png;base64,\(b64(png256))" alt="\(sc.title)"></div>\
  <div class="bar"><img src="data:image/png;base64,\(b64(strip))" alt="深色菜单栏预览"></div>\
  <figcaption><b>\(sc.title)</b><span>\(sc.note)</span></figcaption></figure>

  """
}
let html = """
<style>
.ds-icons{--t1:#1d1d1f;--t2:#86868b;padding:20px;background:#f5f5f7;font-family:-apple-system,"PingFang SC",sans-serif;color:var(--t1);border-radius:14px}
.ds-icons h1{font-size:15px;margin:0 0 4px}
.ds-icons .sub{margin:0 0 18px;font-size:12px;color:var(--t2)}
.ds-icons .grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(212px,1fr));gap:14px}
.ds-icons figure{margin:0;background:#fff;border-radius:14px;padding:14px;box-shadow:0 .5px 2px rgba(0,0,0,.07)}
.ds-icons .icon{display:flex;justify-content:center;padding:4px 0 10px}
.ds-icons .icon img{width:112px;height:112px;border-radius:22px;box-shadow:0 1px 4px rgba(0,0,0,.14)}
.ds-icons .bar{border-radius:8px;overflow:hidden;margin-bottom:10px}
.ds-icons .bar img{display:block;width:100%}
.ds-icons figcaption{display:flex;flex-direction:column;gap:3px}
.ds-icons figcaption b{font-size:13px}
.ds-icons figcaption span{font-size:11.5px;color:var(--t2);line-height:1.45}
</style>
<div class="ds-icons">
<h1>C 方案微调：前屏下面那根横线</h1>
<p class="sub">保持 C 的斜向叠放构图，只给前面那块屏加底座 —— 它就从「副本」变回「一台显示器」 · 上：App 图标，下：深色菜单栏 18 / 22 / 32pt</p>
<div class="grid">\(cards)</div>
</div>

"""
try! html.write(to: URL(fileURLWithPath: "\(candDir)/index.html"), atomically: true, encoding: .utf8)
print("ok -> \(outDir)  (备选与对比页在 \(candDir))")
