use demo_as_code_postprocess::{MouseLogEntry, ZoomLogEntry, process_video_pipeline_impl};

#[tokio::test]
async fn test_ffmpeg_processing_loop() {
    // 1. Arrange mock log states
    let zoom_log = vec![
        ZoomLogEntry { t: 0.0, zoom: 1.0 },
        ZoomLogEntry {
            t: 3.081,
            zoom: 3.0,
        },
        ZoomLogEntry {
            t: 4.116,
            zoom: 1.0,
        },
    ];
    let mouse_log = vec![
        MouseLogEntry {
            t: 0.0,
            x: 100.0,
            y: 600.0,
        },
        MouseLogEntry {
            t: 2.807,
            x: 189.0,
            y: 29.0,
        },
        MouseLogEntry {
            t: 2.809,
            x: 189.0,
            y: 29.0,
        },
    ];

    let video_path = String::from(
        "/home/frook/Desktop/coding/demo-capture/results/videos/page@e0a5f41dbb490e2f9e42648326011540.webm",
    );

    // 2. Act
    let result = process_video_pipeline_impl(video_path, zoom_log, mouse_log).await;

    // 3. Assert
    assert!(
        result.is_ok(),
        "The pipeline execution panicked or failed early"
    );
}
