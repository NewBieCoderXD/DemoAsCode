import { test, expect, Browser, BrowserContext, Page } from "@playwright/test";
import { writeFileSync, mkdirSync, unlinkSync, existsSync } from "fs";
import path from "path";
import { chromium } from "playwright";
import * as nativeEngine from "../../dist/index.js";
import type { MouseLogEntry, ZoomLogEntry } from "../../dist/index.js";

export interface DemoAsCodeOptions {
  size: { width: number; height: number };
  initialMousePos: { x: number; y: number };
  initialZoom: number;
  fps: number;
}

export class TelemetryRecorder {
  private outputDir: string;
  private browser: Browser | null = null;
  private context: BrowserContext | null = null;
  private page: Page | null = null;

  private mouseLog: MouseLogEntry[] = [];
  private zoomLog: ZoomLogEntry[] = [];
  private startTime: number = 0;

  private options: DemoAsCodeOptions | null = null;

  constructor(outputDir: string = "./results") {
    this.outputDir = outputDir;
    // Ensure output directories exist before run execution
    mkdirSync(path.join(this.outputDir, "videos"), { recursive: true });
  }

  /**
   * Initializes the headless Chromium instance and injects the telemetry listeners
   */
  async initialize(
    options: DemoAsCodeOptions = {
      size: { width: 1920, height: 1080 },
      initialMousePos: { x: 500, y: 500 },
      initialZoom: 1,
      fps: 60,
    },
  ): Promise<Page> {
    this.browser = await chromium.launch({ headless: true });

    this.options = options;

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
    this.mouseLog = [{ t: 0, ...options.initialMousePos }];
    this.zoomLog = [{ zoom: options.initialZoom, t: 0 }];
    this.startTime = Date.now();

    // Bind real-time execution streaming bridges
    await this.page.exposeFunction(
      "streamMouseLogEntry",
      (frame: Omit<MouseLogEntry, "t">) => {
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
        window["streamMouseLogEntry"]({
          x: e.clientX + window.scrollX,
          y: e.clientY + window.scrollY,
          clicked: false,
        });
      });

      window.addEventListener("mousedown", (e) => {
        window["streamMouseLogEntry"]({
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
    let result = "";
    if (!this.page) {
      return;
    }
    if (this.options == null) {
      console.error("Failed to fetch DemoAsCode options");
      return;
    }

    // Pull down extra evaluations if present
    const pageLogs = (await this.page.evaluate(
      () => window["_mouseLog"] || [],
    )) as MouseLogEntry[];
    this.mouseLog.push(...pageLogs);

    const video = this.page.video();
    if (!video) {
      return;
    }

    const originalVideoPath = await video.path();
    const tempVideoPath = path.join(
      path.dirname(originalVideoPath),
      `temp-${Date.now()}-${Math.floor(Math.random() * 1000)}.webm`,
    );

    await this.context?.close();

    // Explicitly wait for Playwright to finish writing and flushing the video file to our temp path
    await video.saveAs(tempVideoPath);

    await this.browser?.close();

    console.log(`✨ Start post-processing...`);

    result = nativeEngine.processVideoPipelineImpl({
      videoPath: tempVideoPath,
      zoomLog: this.zoomLog,
      mouseLog: this.mouseLog,
      width: this.options!.size.width,
      height: this.options!.size.height,
      fps: this.options!.fps,
    });
    console.log(`✨ Done... log: ${result}`);

    // Clean up the temporary video file
    try {
      if (existsSync(tempVideoPath)) {
        unlinkSync(tempVideoPath);
      }
    } catch (e) {
      console.error("Failed to clean up temp video:", e);
    }
  }
}
