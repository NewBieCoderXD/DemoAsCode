import { test, expect, Browser, BrowserContext, Page } from "@playwright/test";
import { writeFileSync, mkdirSync } from "fs";
import path from "path";
import { chromium } from "playwright";

export interface MouseFrame {
  t: number; // Timestamp in seconds relative to recording start
  x: number; // X coordinate containing scroll offsets
  y: number; // Y coordinate containing scroll offsets
  clicked: boolean;
}

export interface ZoomFrame {
  t: number; // Timestamp in seconds relative to recording start
  zoom: number; // Zoom scale multiplier
}

export class TelemetryRecorder {
  private outputDir: string;
  private browser: Browser | null = null;
  private context: BrowserContext | null = null;
  private page: Page | null = null;

  private mouseLog: MouseFrame[] = [];
  private zoomLog: ZoomFrame[] = [];
  private startTime: number = 0;

  constructor(outputDir: string = "./results") {
    this.outputDir = outputDir;
    // Ensure output directories exist before run execution
    mkdirSync(path.join(this.outputDir, "videos"), { recursive: true });
  }

  /**
   * Initializes the headless Chromium instance and injects the telemetry listeners
   */
  async initialize(
    options = {
      size: { width: 1920, height: 1080 },
      initialMousePos: { x: 500, y: 500 },
      initialZoom: 1,
    },
  ): Promise<Page> {
    this.browser = await chromium.launch({ headless: true });

    this.context = await this.browser.newContext({
      viewport: { ...options.size },
      deviceScaleFactor: 1,
      recordVideo: {
        dir: path.join(this.outputDir, "videos"),
        size: { ...options.size },
      },
    });

    this.page = await this.context.newPage();
    await this.page.setViewportSize({ width: 1920, height: 1080 });

    // Seed historical baselines
    this.mouseLog = [{ t: 0, ...options.initialMousePos, clicked: false }];
    this.zoomLog = [{ zoom: options.initialZoom, t: 0 }];
    this.startTime = Date.now();

    // Bind real-time execution streaming bridges
    await this.page.exposeFunction(
      "streamMouseFrame",
      (frame: Omit<MouseFrame, "t">) => {
        this.mouseLog.push({
          t: (Date.now() - this.startTime - 300) / 1000, // Matching your original 0.3s calibration drag
          ...frame,
        });
      },
    );

    // Inject document mouse capture hooks natively
    await this.page.addInitScript(() => {
      window.addEventListener("DOMContentLoaded", () => {
        const style = document.createElement("style");
        style.innerHTML = "html { cursor: crosshair !important; }";
        document.documentElement.appendChild(style);
      });

      window.addEventListener("mousemove", (e) => {
        window["streamMouseFrame"]({
          x: e.clientX + window.scrollX,
          y: e.clientY + window.scrollY,
          clicked: false,
        });
      });

      window.addEventListener("mousedown", (e) => {
        window["streamMouseFrame"]({
          x: e.clientX + window.scrollX,
          y: e.clientY + window.scrollY,
          clicked: true,
        });
      });
    });

    return this.page;
  }

  /**
   * Pushes a target zoom scale milestone checkpoint into the telemetry track
   */
  logZoom(zoomFactor: number): void {
    const elapsedSeconds = (Date.now() - this.startTime) / 1000;
    this.zoomLog.push({
      zoom: zoomFactor,
      t: elapsedSeconds,
    });
  }

  /**
   * Flushes out logs to disk and terminates internal automation dependencies cleanly
   */
  async closeAndSave(): Promise<void> {
    if (this.page) {
      // Pull down extra evaluations if present
      const pageLogs = (await this.page.evaluate(
        () => window["_mouseLog"] || [],
      )) as MouseFrame[];
      this.mouseLog.push(...pageLogs);
    }

    // Write out separate telemetry channels
    writeFileSync(
      path.join(this.outputDir, "zoom_log.json"),
      JSON.stringify(this.zoomLog, null, 2),
    );

    writeFileSync(
      path.join(this.outputDir, "mouse_log.json"),
      JSON.stringify(this.mouseLog, null, 2),
    );

    console.log(
      `✨ Saved ${this.mouseLog.length} mouse and ${this.zoomLog.length} zoom milestones.`,
    );

    if (this.browser) {
      await this.browser.close();
    }
  }
}
