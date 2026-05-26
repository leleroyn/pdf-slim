## Why

PDF 文件因嵌入高清图片、冗余字体和未压缩对象而体积膨胀，缺乏一个轻量、跨平台的原生压缩工具。本工具通过图片重编码、流压缩和对象精简，快速缩小 PDF 体积。

## What Changes

- 新增 `pdf-slim` 命令行工具，支持单文件和批量 PDF 压缩
- 支持三种压缩预设：`gentle`、`balanced`（默认）、`aggressive`
- 自动检测并跳过带数字签名的 PDF，避免破坏签名有效性
- 默认不覆盖原文件，输出 `.slim.pdf`；`--force` 原地覆盖（自动备份）
- 压缩报告以 JSON 格式输出，便于 CI/CD 集成和脚本处理

## Capabilities

### New Capabilities
- `pdf-core-pipeline`: PDF 解析、对象遍历、图片提取与重编码、流压缩、保存的完整处理管线
- `image-compression`: 图片解码、降采样、JPEG 重编码、PNG 转 JPEG、并行处理
- `signature-detection`: 检测 PDF 数字签名（AcroForm, DocMDP, Permissions 字典），标记跳过
- `cli-interface`: CLI 参数解析、预设选择、覆盖控制、JSON 输出
- `compression-report`: 压缩前后统计、逐文件明细、JSON 报告结构

### Modified Capabilities
- None

## Impact

- 全新 Rust 项目，无已有代码影响
- 新增依赖：`clap`、`lopdf`、`image`、`rayon`、`tracing`、`serde`
- 输出为可执行 CLI 二进制，Windows/macOS/Linux 跨平台
