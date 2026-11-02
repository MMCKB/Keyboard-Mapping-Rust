"""生成应用图标（键盘线条图），不依赖任何现成图标资源。

每个尺寸单独渲染再打包进 ico，避免把 256 大图硬缩导致小尺寸线条糊掉：
  >= 28px：外框 + 两排键位 + 空格条
  <  28px：外框 + 空格条（只留最少信息）

用法：python tools/make_icon.py
产出：assets/app.ico、assets/app.ico.b64
"""

import base64
import io
import os

from PIL import Image, ImageDraw

COLOR = (43, 124, 211, 255)  # 单一线条色，浅色/深色任务栏都可见
SIZES = [16, 20, 24, 32, 40, 48, 64, 128, 256]
SS = 8  # 超采样倍数


def stroke_for(size: int) -> float:
    return min(max(size * 0.05, 1.4), 11.0)


def render(size: int) -> Image.Image:
    s = size * SS
    img = Image.new("RGBA", (s, s), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    w = stroke_for(size) * SS

    def px(v):
        return v * s

    def seg(x0, y0, x1, y1):
        pts = [(px(x0), px(y0)), (px(x1), px(y1))]
        d.line(pts, fill=COLOR, width=int(round(w)))
        r = w / 2
        for x, y in pts:
            d.ellipse([x - r, y - r, x + r, y + r], fill=COLOR)

    # 外框
    d.rounded_rectangle(
        [px(0.03), px(0.26), px(0.97), px(0.74)],
        radius=0.08 * s,
        outline=COLOR,
        width=int(round(w)),
    )

    if size >= 28:
        for row_y in (0.40, 0.52):
            for i in range(4):
                x0 = 0.11 + i * 0.21
                seg(x0, row_y, x0 + 0.14, row_y)
        seg(0.30, 0.635, 0.70, 0.635)
    else:
        seg(0.24, 0.55, 0.76, 0.55)

    return img.resize((size, size), Image.LANCZOS)


def build_ico(frames) -> bytes:
    """手写 ICO 容器：每个尺寸放一张 PNG，避免 Pillow 统一缩放。"""
    payloads = []
    for frame in frames:
        buffer = io.BytesIO()
        frame.save(buffer, format="PNG", optimize=True)
        payloads.append(buffer.getvalue())

    header = bytearray()
    header += (0).to_bytes(2, "little")  # reserved
    header += (1).to_bytes(2, "little")  # type = icon
    header += len(payloads).to_bytes(2, "little")

    offset = 6 + 16 * len(payloads)
    entries = bytearray()
    for frame, payload in zip(frames, payloads):
        size = frame.width
        entries += bytes([0 if size >= 256 else size, 0 if size >= 256 else size, 0, 0])
        entries += (1).to_bytes(2, "little")  # color planes
        entries += (32).to_bytes(2, "little")  # bits per pixel
        entries += len(payload).to_bytes(4, "little")
        entries += offset.to_bytes(4, "little")
        offset += len(payload)

    return bytes(header + entries + b"".join(payloads))


def main():
    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    frames = [render(size) for size in SIZES]
    data = build_ico(frames)

    ico_path = os.path.join(root, "assets", "app.ico")
    b64_path = os.path.join(root, "assets", "app.ico.b64")
    with open(ico_path, "wb") as handle:
        handle.write(data)
    with open(b64_path, "w", encoding="ascii", newline="\n") as handle:
        handle.write(base64.b64encode(data).decode("ascii"))

    # 回读校验
    check = Image.open(ico_path)
    print("sizes:", sorted(check.info["sizes"]), "| bytes:", len(data))


if __name__ == "__main__":
    main()
