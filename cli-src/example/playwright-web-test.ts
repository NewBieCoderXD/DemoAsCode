import { TelemetryRecorder } from "../engine/telemetry-recorder.js";
import { expect } from "@playwright/test";

async function run() {
  const recorder = new TelemetryRecorder("./results");
  const page = await recorder.initialize({
    size: { width: 1920, height: 1080 },
    initialMousePos: { x: 100, y: 600 },
    initialZoom: 1,
  });
  let isOk = true;

  try {
    // 1. Navigate
    await page.goto("https://playwright.dev/");
    await page.waitForLoadState("networkidle");

    // 2. Interact & Scale zoom tracking points
    const searchInput = page.locator("a[href='/docs/intro']").first();
    await expect(searchInput).toBeVisible();

    // Zoom in right before making the interaction click
    recorder.logZoom(3.0);

    await searchInput.click();
    await page.waitForTimeout(1000);

    // Return zoom scale back down to baseline canvas
    recorder.logZoom(1.0);
    await page.waitForTimeout(500);
  } catch (error) {
    console.error("Interaction thread crashed:", error);
    isOk = false;
  } finally {
    // 3. Guarantees browser closing and files flushing even if actions fail
    await recorder.closeAndSave();
  }
}

run();
