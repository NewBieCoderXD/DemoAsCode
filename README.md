<p align="center">
  <img src="https://raw.githubusercontent.com/NewBieCoderXD/DemoAsCode/refs/heads/main/apps/website/static/img/favicon.svg" width="128" height="128" alt="DemoAsCode logo"/>
</p>

<h1 align="center">DemoAsCode</h1>

<p align="center">A browser recording tool that captures Playwright sessions with mouse and zoom telemetry, then post-processes videos using FFmpeg and Rust.</p>

<p align="center">
  <a href="https://newbiecoderxd.github.io/DemoAsCode/">
    <img src="https://img.shields.io/badge/docs-demo--as--code-blue?style=for-the-badge" alt="Documentation">
  </a>
</p>

<video src='https://github.com/user-attachments/assets/4f0af336-f068-4018-b849-f2c41a0e2775' controls width="100%">Video link: https://github.com/user-attachments/assets/4f0af336-f068-4018-b849-f2c41a0e2775</video>

## Overview

DemoAsCode records browser interactions as polished demo videos. It tracks mouse movements and zoom levels during a Playwright session, then applies post-processing (zoom effects, cursor rendering, video encoding) via a Rust/NAPI backend.

## Architecture

```
DemoAsCode/
├── apps/
│   ├── engine/              # TypeScript recording layer
│   │   └── src/index.ts     # Playwright orchestration + telemetry capture
│   ├── post-processor/      # Rust NAPI native module
│   │   └── src/lib.rs       # Video processing pipeline (FFmpeg + image ops)
│   └── website/             # Documentation site (Docusaurus)
├── example/                 # Usage example
├── bin/                     # Bundled FFmpeg binary
└── dist/                    # Compiled output
```

## Prerequisites

- Node.js (ES2022+)
- pnpm
- Rust toolchain (for building the native post-processor)

## Supported Platforms

| OS      | Architecture | Support |
|---------|-------------|---------|
| Linux   | x64 (glibc) | ✅ |
| Linux   | x64 (musl)  | ✅ |
| macOS   | x64 (Intel) | ✅ |
| macOS   | ARM64 (M1+) | ✅ |
| Windows | x64         | ✅ |

## Installation

```bash
pnpm install
pnpm run build
```

## Usage

```typescript
import { Recorder } from "demo-as-code";

const recorder = new Recorder("./results");
const page = await recorder.initialize({
  size: { width: 1920, height: 1080 },
  initialMousePos: { x: 500, y: 500 },
  initialZoom: 1,
});

await page.goto("https://example.com");

// Zoom in at specific moments
recorder.logZoom(2.5);
// ... perform interactions ...
recorder.logZoom(1.0);

// Save video with post-processing
await recorder.closeAndSave();
```

## Build Commands

| Command               | Description                             |
| --------------------- | --------------------------------------- |
| `pnpm run build`      | Build both native module and TypeScript |
| `pnpm docs:build`     | Build documentation site                |

## Output

Recordings are saved to `./results/videos/` as processed MP4 files with:

- Smooth zoom transitions between captured zoom points
- Mouse cursor overlay
- Configurable quality (CRF) and resolution

## Documentation

Full documentation is available at [https://newbiecoderxd.github.io/DemoAsCode/](https://newbiecoderxd.github.io/DemoAsCode/).
