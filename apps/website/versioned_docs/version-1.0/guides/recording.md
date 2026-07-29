---
sidebar_position: 1
---

# Recording

## Creating a Recorder

```typescript
import { Recorder } from "demo-as-code";

const recorder = new Recorder("./output");
```

The recorder manages a headless Chromium instance and captures telemetry.

## Initialization

```typescript
const page = await recorder.initialize({
  size: { width: 1920, height: 1080 },
  initialMousePos: { x: 500, y: 500 },
  initialZoom: 1,
  crf: 4,
});
```

### Options

| Option            | Type                | Default              | Description                    |
| ----------------- | ------------------- | -------------------- | ------------------------------ |
| `size`            | `{ width, height }` | `1920x1080`          | Viewport dimensions            |
| `initialMousePos` | `{ x, y }`          | `{ x: 500, y: 500 }` | Starting cursor position       |
| `initialZoom`     | `number`            | `1`                  | Starting zoom level            |
| `crf`             | `number`            | `4`                  | Video quality (lower = better) |

## Tracking Zoom

Call `logZoom()` to mark zoom transitions:

```typescript
// Zoom in before important interaction
recorder.logZoom(2.5);
await page.click(".important-button");

// Zoom back out
recorder.logZoom(1.0);
```

The post-processor interpolates smooth transitions between zoom points.

## Mouse Tracking

Mouse movements and clicks are captured automatically via injected event listeners. No manual setup required.

## Saving

```typescript
await recorder.closeAndSave();
```

This flushes the browser, runs the video pipeline, and cleans up temporary files.
