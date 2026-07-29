import { Browser, BrowserContext, Page } from "@playwright/test";
import { mkdirSync, unlinkSync, existsSync } from "node:fs";
import path from "node:path";
import { chromium } from "playwright";
import * as postProcessEngine from "@demo-as-code/postprocess";
import type { MouseLogEntry, ZoomLogEntry } from "@demo-as-code/postprocess";
import winston from "winston";

const logger = winston.createLogger({
  level: "info",
  format: winston.format.combine(
    winston.format.splat(), // Allows string substitution
    winston.format.simple(), // Simple plain-text display
  ),
  transports: [new winston.transports.Console()],
});

const DEFAULT_DEMO_OPTIONS: DemoAsCodeOptions = {
  size: { width: 1920, height: 1080 },
  initialMousePos: { x: 500, y: 500 },
  initialZoom: 1,
  crf: 4,
};

export interface DemoAsCodeOptions {
  size: { width: number; height: number };
  initialMousePos: { x: number; y: number };
  initialZoom: number;
  crf?: number;
}

export class Recorder {
  private readonly outputDir: string;
  private browser: Browser | null = null;
  private context: BrowserContext | null = null;
  private page: Page | null = null;

  private mouseLog: MouseLogEntry[] = [];
  private zoomLog: ZoomLogEntry[] = [];
  private startTime: number = 0;

  private readonly ffmpegPath: string | undefined = undefined;

  private options: DemoAsCodeOptions | null = null;

  constructor(outputDir: string = "./results") {
    this.outputDir = outputDir;
    mkdirSync(path.join(this.outputDir, "videos"), { recursive: true });
  }

  async initialize(options: Partial<DemoAsCodeOptions>): Promise<Page> {
    const config: DemoAsCodeOptions = {
      ...DEFAULT_DEMO_OPTIONS,
      ...options,
      size: { ...DEFAULT_DEMO_OPTIONS.size, ...options?.size },
      initialMousePos: {
        ...DEFAULT_DEMO_OPTIONS.initialMousePos,
        ...options?.initialMousePos,
      },
    };

    logger.info("Initializing headless Chromium");

    this.browser = await chromium.launch({ headless: true });

    this.options = config;

    this.context = await this.browser.newContext({
      viewport: { ...config.size },
      deviceScaleFactor: 1,
      recordVideo: {
        dir: path.join(this.outputDir, "videos"),
        size: { ...config.size },
      },
    });

    this.page = await this.context.newPage();
    await this.page.setViewportSize({ width: 1920, height: 1080 });

    this.mouseLog = [{ t: 0, ...config.initialMousePos }];
    this.zoomLog = [{ zoom: config.initialZoom, t: 0 }];
    this.startTime = Date.now();

    this.setupTelemetry(this.page);

    return this.page;
  }

  private async setupTelemetry(page: Page): Promise<void> {
    await page.exposeFunction(
      "__recorder_streamMouseLog",
      (frame: Omit<MouseLogEntry, "t">) => {
        const elapsedSeconds = (Date.now() - this.startTime) / 1000;
        this.mouseLog.push({ t: elapsedSeconds, ...frame });
      },
    );

    await page.evaluate(`(() => {
      const handler = (e) => {
        if (typeof window.__recorder_streamMouseLog === "function") {
          window.__recorder_streamMouseLog({
            x: e.clientX + window.scrollX,
            y: e.clientY + window.scrollY,
            clicked: e.type === "mousedown",
          });
        }
      };

      window.addEventListener("mousemove", handler, { passive: true });
      window.addEventListener("mousedown", handler, { passive: true });
    })()`);
  }

  logZoom(zoomFactor: number): void {
    const elapsedSeconds = (Date.now() - this.startTime) / 1000;
    this.zoomLog.push({
      zoom: zoomFactor,
      t: elapsedSeconds,
    });
  }

  async closeAndSave(): Promise<void> {
    let result = "";
    if (!this.page) {
      return;
    }
    if (this.options == null) {
      logger.error("Missing DemoAsCode options");
      return;
    }

    const video = this.page.video();
    if (!video) {
      return;
    }

    const originalVideoPath = await video.path();
    const tempVideoPath = path.join(
      path.dirname(originalVideoPath),

      // eslint-disable-next-line sonarjs/pseudo-random
      `temp-${Date.now()}-${Math.floor(Math.random() * 1000)}.webm`,
    );

    await this.context?.close();

    await video.saveAs(tempVideoPath);

    await this.browser?.close();

    logger.info("Starting post-processing");

    result = await postProcessEngine.processVideoPipelineImpl({
      videoPath: tempVideoPath,
      zoomLog: this.zoomLog,
      mouseLog: this.mouseLog,
      width: this.options!.size.width,
      height: this.options!.size.height,
      ffmpegPath: this.ffmpegPath,
      fps: 25,
    });
    logger.info(`Post-processing done: ${result}`);

    try {
      if (existsSync(tempVideoPath)) {
        unlinkSync(tempVideoPath);
      }
    } catch (e) {
      logger.error(`Failed to clean up temp video: ${e}`);
    }
  }
}
