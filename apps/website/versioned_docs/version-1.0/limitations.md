---
sidebar_position: 3
---

# Limitations

DemoAsCode is designed for straightforward screen-capture style demos. The following limitations are important to understand before relying on it for a production workflow.

## Frame rate

The output video is rendered at a fixed **25 fps** (`fps: 25` in the recording pipeline), and the source capture is capped by Playwright's own video recording. This means:

- Higher frame rates are not supported.
- Fast motion may appear slightly choppy.
- Motion smoothness cannot be tuned through configuration.

## CSS loading delay

There is **no automatic wait** for stylesheets, web fonts, or images to load. Recording starts as soon as the browser session is initialized and telemetry is attached. If interactions begin before the page is fully styled:

- The opening frames may capture unstyled or partially rendered content.
- Layout shifts and late-loading fonts will be visible in the recording.

Always wait for the page to settle before interacting, for example:

```typescript
await page.goto("https://example.com");
await page.waitForLoadState("networkidle");
await page.evaluate(() => document.fonts.ready);
```

## Cursor overlay

The mouse overlay is a **plain red circle with a fixed radius** (`[255, 0, 0]`, ~8px in the native post-processor). Specifically:

- Click events are recorded but **not visualized** — there is no click indicator or ripple.
- There is no support for custom cursor images or styles.
- There are no cursor animations or easing effects.
- The cursor is drawn for the full duration of the video, including while idle.

## Output format and path

- The final video is encoded as **WebM (VP9)**, not MP4.
- The output is written to a fixed `./results/output_processed.webm` path regardless of the `outputDir` passed to the `Recorder` constructor.
- Because the output path is fixed, running multiple recordings concurrently will overwrite each other's results.

## Browser and platform

- Only **Chromium** is supported.
- Recording runs in **headless** mode only.
- A custom `size` is overridden by a hardcoded 1920x1080 viewport.

## Telemetry scope

Only `mousemove` and `mousedown` events are captured. The following are **not** recorded:

- Scroll events
- Keyboard input
- Hover states on elements
- Audio
