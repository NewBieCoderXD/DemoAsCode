# DemoAsCode

A browser recording tool that captures Playwright sessions with mouse and zoom telemetry, then post-processes videos using FFmpeg and Rust

<video src='https://github.com/user-attachments/assets/4f0af336-f068-4018-b849-f2c41a0e2775' controls width="100%">Video link: https://github.com/user-attachments/assets/4f0af336-f068-4018-b849-f2c41a0e2775</video>

## Overview

DemoAsCode records browser interactions as polished demo videos. It tracks mouse movements and zoom levels during a Playwright session, then applies post-processing (zoom effects, cursor rendering, video encoding) via a Rust/NAPI backend.

## Architecture

```
DemoAsCode/
├── engine/                  # TypeScript recording layer
│   └── recorder.ts          # Playwright orchestration + telemetry capture
├── post-processor/          # Rust NAPI native module
│   └── src/lib.rs           # Video processing pipeline (FFmpeg + image ops)
├── bin/                     # Bundled FFmpeg binary
└── dist/                    # Compiled output
```

## Prerequisites

- Node.js (ES2022+)
- pnpm
- Rust toolchain (for building the native post-processor)

## Installation

```bash
pnpm install
pnpm run build
```

## Usage

```typescript
import { Recorder } from "./engine/recorder.js";

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
| `pnpm run napi:build` | Build Rust NAPI module only             |
| `pnpm run ts:build`   | Compile TypeScript only                 |

## Output

Recordings are saved to `./results/videos/` as processed MP4 files with:

- Smooth zoom transitions between captured zoom points
- Mouse cursor overlay
- Configurable quality (CRF) and resolution
