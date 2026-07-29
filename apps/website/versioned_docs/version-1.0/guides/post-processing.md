---
sidebar_position: 2
---

# Post-Processing

## Overview

The post-processor is a Rust native module that handles video encoding and effects. It runs automatically when you call `closeAndSave()`.

## What It Does

1. **Zoom interpolation** - Smoothly transitions between captured zoom points
2. **Cursor overlay** - Renders mouse position and clicks
3. **Video encoding** - Outputs H.264 MP4 with configurable quality

## Pipeline

```
Raw Webm (Playwright)
    ↓
Zoom + Mouse Processing (Rust)
    ↓
FFmpeg Encoding
    ↓
Final MP4
```

## Configuration

The pipeline accepts these parameters:

| Parameter    | Type              | Description               |
| ------------ | ----------------- | ------------------------- |
| `videoPath`  | `string`          | Input video file          |
| `zoomLog`    | `ZoomLogEntry[]`  | Zoom level timeline       |
| `mouseLog`   | `MouseLogEntry[]` | Mouse position timeline   |
| `width`      | `number`          | Output width              |
| `height`     | `number`          | Output height             |
| `fps`        | `number`          | Output frame rate         |
| `ffmpegPath` | `string?`         | Custom FFmpeg binary path |

## Custom FFmpeg

By default, the bundled FFmpeg binary is used. To use a custom one:

```typescript
const recorder = new Recorder("./results");
recorder.setFfmpegPath("/usr/local/bin/ffmpeg");
```
