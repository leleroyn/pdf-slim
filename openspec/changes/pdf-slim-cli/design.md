## Context

全新 Rust 项目，目标是在单二进制中实现 PDF 压缩功能。核心难点在于 PDF 对象树的遍历、多种图像编码的处理，以及数字签名的无损检测。

## Goals / Non-Goals

**Goals:**
- 单文件 CLI 工具，编译后 < 15MB 二进制
- 图片密集型 PDF 压缩率 50%+（balanced 预设）
- 检测并跳过带数字签名的 PDF，不破坏签名
- JSON 报告输出，支持脚本集成
- `--force` 原地覆盖前自动创建备份

**Non-Goals:**
- PDF 渲染预览
- 添加/编辑 PDF 内容
- OCR 功能
- 云存储集成

## Decisions

| 决策 | 理由 |
|---|---|
| **lopdf 而非 pdfium** | 纯 Rust 无 FFI，跨平台编译简单；pdfium 需要系统依赖，分发复杂 |
| **image crate 做图片重编码** | 支持 JPEG/PNG/WebP 解码，API 成熟；libvips 需要 FFI 绑定 |
| **rayon 并行处理图片** | 图片重编码是 CPU 密集型任务，天然可并行；比多线程手动管理更简洁 |
| **serde_json 做报告输出** | 生态标准，与 lopdf 对象序列化兼容 |
| **两阶段写入（写入临时文件 → 重命名）** | `--force` 覆盖时原子替换，避免半写文件损坏原文件 |
| **默认输出 .slim.pdf 而非覆盖** | 安全优先，用户明确 `--force` 才覆盖 |

## Risks / Trade-offs

| 风险 | 缓解 |
|---|---|
| [lopdf 不支持 JPXDecode (JPEG2000)] | 检测后跳过该图片，记录到报告中 |
| [CCITTFaxDecode 扫描图片质量损失大] | gentle 预设不重编码 CCITT 图片，balanced/aggressive 可选 |
| [CMYK 色彩空间无直接支持] | 检测 CMYK 图片时保持原编码，仅处理 RGB/Gray |
| [大文件内存占用] | 逐页处理，不一次性加载所有图片到内存 |
| [加密 PDF] | 检测后拒绝处理，返回明确错误 |

## Migration Plan

不适用 — 全新项目，无迁移需求。发布策略：Cargo publish + GitHub Releases 预编译二进制。

## Open Questions

- 是否支持配置文件（如 `~/.config/pdf-slim/config.toml`）保存常用预设？— P1 后评估
- 是否支持 GPU 加速图片处理？— 长期探索项
