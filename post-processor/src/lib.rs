use crate::fr::images::Image;
use fast_image_resize::{self as fr};
use napi::bindgen_prelude::{AsyncTask, Env, Task};
use napi_derive::napi;
use serde::Deserialize;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;

#[napi(object)]
#[derive(Deserialize, Debug, Clone)]
pub struct ZoomLogEntry {
    pub t: f64,
    pub zoom: f64,
}

#[napi(object)]
#[derive(Deserialize, Debug, Clone)]
pub struct MouseLogEntry {
    pub t: f64,
    pub x: f64,
    pub y: f64,
}

// Binary search helper for time-series logs
fn get_zoom_at_time(time: f64, log: &[ZoomLogEntry]) -> f64 {
    if log.is_empty() {
        return 1.0;
    }
    if time <= log[0].t {
        return log[0].zoom;
    }
    if time >= log[log.len() - 1].t {
        return log[log.len() - 1].zoom;
    }

    let mut low = 0;
    let mut high = log.len() - 1;
    let mut idx = 0;
    while low <= high {
        let mid = (low + high) / 2;
        if log[mid].t <= time {
            idx = mid;
            low = mid + 1;
        } else {
            high = mid - 1;
        }
    }
    let start = &log[idx];
    let end = &log[idx + 1];
    if (end.t - start.t).abs() < 1e-6 {
        return start.zoom;
    }

    start.zoom + (end.zoom - start.zoom) * ((time - start.t) / (end.t - start.t))
}

// TODO: Change from binary search to incremental
fn get_mouse_at_time(time: f64, log: &[MouseLogEntry]) -> (f64, f64) {
    if log.is_empty() {
        return (960.0, 540.0);
    } // Default center for 1080p
    if time <= log[0].t {
        return (log[0].x, log[0].y);
    }
    if time >= log[log.len() - 1].t {
        return (log[log.len() - 1].x, log[log.len() - 1].y);
    }

    let mut low = 0;
    let mut high = log.len() - 1;
    let mut idx = 0;
    while low <= high {
        let mid = (low + high) / 2;
        if log[mid].t <= time {
            idx = mid;
            low = mid + 1;
        } else {
            high = mid - 1;
        }
    }
    let start = &log[idx];
    let end = &log[idx + 1];
    if (end.t - start.t).abs() < 1e-6 {
        return (start.x, start.y);
    }

    let amt = (time - start.t) / (end.t - start.t);
    (
        start.x + (end.x - start.x) * amt,
        start.y + (end.y - start.y) * amt,
    )
}

pub struct VideoPipelineTask {
    pub video_path: String,
    pub zoom_log: Vec<ZoomLogEntry>,
    pub mouse_log: Vec<MouseLogEntry>,
}

#[napi]
impl Task for VideoPipelineTask {
    type Output = String;
    type JsValue = String;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;

        let video_path = self.video_path.clone();
        let zoom_log = self.zoom_log.clone();
        let mouse_log = self.mouse_log.clone();

        rt.block_on(
            async move { process_video_pipeline_impl(video_path, zoom_log, mouse_log).await },
        )
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(output)
    }
}

#[napi]
pub fn process_video_pipeline(
    video_path: String,
    zoom_log: Vec<ZoomLogEntry>,
    mouse_log: Vec<MouseLogEntry>,
) -> AsyncTask<VideoPipelineTask> {
    AsyncTask::new(VideoPipelineTask {
        video_path,
        zoom_log,
        mouse_log,
    })
}

pub async fn process_video_pipeline_impl(
    video_path: String,
    zoom_log: Vec<ZoomLogEntry>,
    mouse_log: Vec<MouseLogEntry>,
) -> napi::Result<String> {
    let (done_tx, done_rx) = tokio::sync::oneshot::channel();

    println!("{}, {:?}, {:?}", video_path, zoom_log, mouse_log);
    let width = 1920 as u32;
    let height = 1080 as u32;
    let fps = 25.0;
    let frame_size = (width as usize) * (height as usize) * 3; // RGB24 layout
    // let mut frame_buffer = vec![0u8; frame_size];

    // Bounded capacity of 3 frames. This limits memory usage to ~18.6 MB (3 * 6.2MB)
    // while providing a perfect buffer cushion for heavy CPU computation spikes.
    let (frame_tx, frame_rx) = crossbeam_channel::bounded::<Vec<u8>>(3);

    let static_ffmpeg_path = Path::new("./bin").join("ffmpeg");

    let cwd = std::env::current_dir().unwrap();
    println!("🎥 CWD: {:?}", cwd);
    println!("🎥 INPUT FILE PATH: {}", video_path);
    if let Ok(meta) = std::fs::metadata(&video_path) {
        println!("🎥 INPUT FILE SIZE: {}", meta.len());
    } else {
        println!("🎥 INPUT FILE DOES NOT EXIST OR CANNOT BE READ");
    }

    // Fallback gracefully to system location if local build asset is missing during development edge cases
    let ffmpeg_binary = if static_ffmpeg_path.exists() {
        let abs_path = static_ffmpeg_path.canonicalize().unwrap();
        println!("🎥 FFMPEG BINARY: {:?}", abs_path);
        abs_path
    } else {
        panic!("ggg")
    };

    let ffmpeg_bin_clone = ffmpeg_binary.clone();
    let video_path_clone = video_path.clone();

    std::thread::spawn(move || {
        let mut decoder_cmd = Command::new(&ffmpeg_bin_clone);
        decoder_cmd.args([
            "-i",
            &video_path_clone,
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgb24",
            "-an",
            "-",
        ]);
        decoder_cmd.stdout(Stdio::piped());
        decoder_cmd.stderr(Stdio::null());

        let mut decoder = decoder_cmd.spawn().expect("Failed to spawn FFmpeg decoder");
        let mut decoder_stdout = decoder
            .stdout
            .take()
            .expect("Failed to grab decoder stdout");

        loop {
            let mut buffer = vec![0u8; frame_size];
            if decoder_stdout.read_exact(&mut buffer).is_err() {
                break; // End of file stream or pipe collapsed
            }
            // Send buffer to consumer. If the consumer thread is running slow,
            // crossbeam will naturally block this producer right here, applying backpressure upstream.
            if frame_tx.send(buffer).is_err() {
                break;
            }
        }
        let _ = decoder.wait();
    });
    std::thread::spawn(move || {
        let result = || -> std::io::Result<String> {
            let mut encoder_cmd = Command::new(&ffmpeg_binary);
            encoder_cmd.args([
                "-y",
                "-f",
                "rawvideo",
                "-pix_fmt",
                "rgb24",
                "-s",
                "1920x1080",
                "-r",
                &fps.to_string(),
                "-i",
                "-",
                "-c:v",
                "libvpx-vp9",
                "-crf",
                "30",
                "-b:v",
                "0",
                "-pix_fmt",
                "yuv420p",
                "./results/output_processed.webm",
            ]);
            encoder_cmd.stdin(Stdio::piped());
            encoder_cmd.stdout(Stdio::null());
            encoder_cmd.stderr(Stdio::inherit());

            let mut encoder = encoder_cmd.spawn().expect("Failed to spawn FFmpeg encoder");
            let mut encoder_stdin = encoder.stdin.take().expect("Failed to grab encoder stdin");

            let mut current_frame = 0;
            let mut smooth_zoom = if !zoom_log.is_empty() {
                zoom_log[0].zoom
            } else {
                1.0
            };
            let mut resizer = fr::Resizer::new();

            // Pre-allocate destination image canvas once to prevent inner loop memory thrashing
            let mut dst_image = Image::new(width, height, fr::PixelType::U8x3);

            // Pull raw frame buffers from the Producer channel
            while let Ok(frame_buffer) = frame_rx.recv() {
                let time = current_frame as f64 / fps;

                // --- COMPUTATIONALLY EXPENSIVE WORKLOAD START ---
                let target_zoom = get_zoom_at_time(time, &zoom_log);
                let (target_x, target_y) = get_mouse_at_time(time, &mouse_log);

                smooth_zoom += (target_zoom - smooth_zoom) * 0.15;

                let crop_w = (width as f64 / smooth_zoom) as u32;
                let crop_h = (height as f64 / smooth_zoom) as u32;

                let mut pan_x = (target_x - (crop_w as f64 / 2.0)) as i32;
                let mut pan_y = (target_y - (crop_h as f64 / 2.0)) as i32;

                pan_x = pan_x.clamp(0, (width - crop_w) as i32);
                pan_y = pan_y.clamp(0, (height - crop_h) as i32);

                // Slice crop window from raw incoming frame
                let mut cropped_buffer = vec![0u8; (crop_w * crop_h * 3) as usize];
                for row in 0..crop_h {
                    let src_start = (((pan_y + row as i32) * width as i32 + pan_x) * 3) as usize;
                    let dest_start = (row * crop_w * 3) as usize;
                    cropped_buffer[dest_start..(dest_start + (crop_w * 3) as usize)]
                        .copy_from_slice(
                            &frame_buffer[src_start..(src_start + (crop_w * 3) as usize)],
                        );
                }

                let src_image =
                    Image::from_vec_u8(crop_w, crop_h, cropped_buffer, fr::PixelType::U8x3)
                        .unwrap();

                resizer
                    .resize(
                        &src_image,
                        &mut dst_image,
                        &fr::ResizeOptions::new()
                            .resize_alg(fr::ResizeAlg::Interpolation(fr::FilterType::Bilinear)),
                    )
                    .unwrap();

                // Draw Cursor Overlays onto destination canvas
                let dst_buffer = dst_image.buffer_mut();
                let cursor_radius = 8i32;
                let cursor_color = [255u8, 0, 0];

                let rel_x = ((target_x - pan_x as f64) * (width as f64 / crop_w as f64)) as i32;
                let rel_y = ((target_y - pan_y as f64) * (height as f64 / crop_h as f64)) as i32;

                for dy in -cursor_radius..=cursor_radius {
                    for dx in -cursor_radius..=cursor_radius {
                        if dx * dx + dy * dy <= cursor_radius * cursor_radius {
                            let pixel_x = rel_x + dx;
                            let pixel_y = rel_y + dy;

                            if pixel_x >= 0
                                && pixel_x < width as i32
                                && pixel_y >= 0
                                && pixel_y < height as i32
                            {
                                let idx = ((pixel_y * width as i32 + pixel_x) * 3) as usize;
                                dst_buffer[idx] = cursor_color[0];
                                dst_buffer[idx + 1] = cursor_color[1];
                                dst_buffer[idx + 2] = cursor_color[2];
                            }
                        }
                    }
                }
                // --- COMPUTATIONALLY EXPENSIVE WORKLOAD END ---

                // Pipe clean frame down into the encoder's standard input stream
                encoder_stdin.write_all(dst_image.buffer())?;
                current_frame += 1;
            }

            // Flush streams and close process handles safely
            encoder_stdin.flush()?;
            drop(encoder_stdin);
            let _ = encoder.wait();

            Ok(format!("Successfully processed {} frames.", current_frame))
        }();

        let _ = done_tx.send(result.map_err(|e| napi::Error::from_reason(e.to_string())));
    });

    done_rx.await.map_err(|_| {
        napi::Error::from_reason("Video processor runtime worker thread collapsed abnormally")
    })?
}
