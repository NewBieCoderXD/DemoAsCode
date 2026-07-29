---
sidebar_position: 2
---

# Getting Started

## Prerequisites

- Node.js 22+
- pnpm

## Installation

```bash
pnpm add demo-as-code
```

## Basic Usage

```typescript
import { Recorder } from "demo-as-code";

async function recordDemo() {
  const recorder = new Recorder("./results");
  const page = await recorder.initialize({
    size: { width: 1920, height: 1080 },
    initialMousePos: { x: 500, y: 500 },
    initialZoom: 1,
  });

  try {
    await page.goto("https://example.com");
    await page.waitForLoadState("networkidle");

    // Zoom in for emphasis
    recorder.logZoom(2.5);
    await page.click("some-button");

    // Zoom back out
    recorder.logZoom(1.0);
    await page.waitForTimeout(500);
  } finally {
    await recorder.closeAndSave();
  }
}

recordDemo();
```

## How It Works

1. **Initialize** - Creates a headless Chromium instance with telemetry hooks
2. **Record** - Playwright interacts with the page while mouse/zoom events are captured
3. **Post-process** - Rust engine applies zoom effects, cursor overlay, and encodes the final video

## Output

Recordings are saved to `./results/videos/` as processed MP4 files.
