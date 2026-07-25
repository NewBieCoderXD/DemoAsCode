---
sidebar_position: 1
slug: /
---

# DemoAsCode

Browser recording with zoom and mouse telemetry for creating polished demo videos.

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
