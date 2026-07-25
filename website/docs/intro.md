---
sidebar_position: 1
slug: /
---

<p align="center">
  <img src="/DemoAsCode/img/favicon.svg" width="128" height="128" alt="DemoAsCode logo"/>
</p>

# DemoAsCode

Browser recording with zoom and mouse telemetry for creating polished demo videos.

<video src="https://github.com/user-attachments/assets/4f0af336-f068-4018-b849-f2c41a0e2775" controls width="100%"></video>

## Features

- Records Playwright browser sessions
- Tracks mouse movements and clicks
- Captures zoom levels at specific moments
- Post-processes videos with FFmpeg and Rust
- Cross-platform support (Linux, macOS, Windows)

## Quick Start

```bash
npm install demo-as-code
```

```typescript
import { Recorder } from "demo-as-code";

const recorder = new Recorder("./results");
const page = await recorder.initialize();

await page.goto("https://example.com");
recorder.logZoom(2.5);

await recorder.closeAndSave();
```
