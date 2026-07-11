import * as nativeEngine from "../dist/index.js";

async function main() {
  const result = nativeEngine.processVideoPipelineImpl(
    "/home/frook/Desktop/coding/demo-capture/results/videos/page@e0a5f41dbb490e2f9e42648326011540.webm",
    [
      { t: 0.0, zoom: 1.0 },
      {
        t: 3.081,
        zoom: 3.0,
      },
      {
        t: 4.116,
        zoom: 1.0,
      },
    ],
    [
      {
        t: 0.0,
        x: 100.0,
        y: 600.0,
      },
      {
        t: 2.807,
        x: 189.0,
        y: 29.0,
      },
      {
        t: 2.809,
        x: 189.0,
        y: 29.0,
      },
    ],
  );

  console.log("result", result);
}

main();
