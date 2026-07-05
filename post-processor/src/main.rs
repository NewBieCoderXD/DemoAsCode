use crate::fr::images::Image;
use fast_image_resize::{self as fr};
use serde::Deserialize;
use std::fs::File;
use std::io::{self};
use std::process::Stdio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;

#[derive(Deserialize, Debug, Clone)]
struct ZoomLogEntry {
    t: f64,
    zoom: f64,
}

#[derive(Deserialize, Debug, Clone)]
struct MouseLogEntry {
    t: f64,
    x: f64,
    y: f64,
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

#[tokio::main]
async fn main() -> io::Result<()> {
    // 1. Read and parse both decoupled telemetry log files
    let zoom_file = File::open("./results/zoom_log.json")?;
    let zoom_log: Vec<ZoomLogEntry> = serde_json::from_reader(zoom_file).unwrap();

    let mouse_file = File::open("./results/mouse_log.json")?;
    let mouse_log: Vec<MouseLogEntry> = serde_json::from_reader(mouse_file).unwrap();

    let width = 1920 as u32;
    let height = 1080 as u32;
    let fps = 25.0;
    let frame_size = (width as usize) * (height as usize) * 3; // RGB24 layout
    let mut frame_buffer = vec![0u8; frame_size];

    // 2. Spawn the internal FFmpeg DECODER (Extracts raw RGB24 frames from file)
    let mut decoder_cmd = Command::new("ffmpeg");
    decoder_cmd.args([
        "-i",
        "./results/videos/page@54ab6586642de9dee262d5313c115624.webm",
        "-f",
        "rawvideo",
        "-pix_fmt",
        "rgb24",
        "-an",
        "-",
    ]);
    decoder_cmd.stdout(Stdio::piped());
    // This setting ensures that if Rust exits, this decoder dies instantly
    decoder_cmd.kill_on_drop(true);

    let mut decoder = decoder_cmd.spawn().expect("Failed to start FFmpeg encoder");

    let mut encoder_cmd = Command::new("ffmpeg");
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
        "yuva420p",
        "./results/output_processed.webm",
    ]);
    encoder_cmd.stdin(Stdio::piped());
    // This setting ensures that if Rust exits, this encoder dies instantly
    encoder_cmd.kill_on_drop(true);

    let mut encoder = encoder_cmd.spawn().expect("Failed to start FFmpeg encoder");

    let mut decoder_stdout = decoder.stdout.take().unwrap();
    let mut encoder_stdin = encoder.stdin.take().unwrap();

    let mut current_frame = 0;
    let mut smooth_zoom = if !zoom_log.is_empty() {
        zoom_log[0].zoom
    } else {
        1.0
    };
    let mut resizer = fr::Resizer::new();

    // 4. Processing Loop: Pulled directly across internal OS pipes
    while decoder_stdout.read_exact(&mut frame_buffer).await.is_ok() {
        let time = current_frame as f64 / fps;

        // Fetch target attributes from separate logs across shared timelines
        let target_zoom = get_zoom_at_time(time, &zoom_log);
        let (target_x, target_y) = get_mouse_at_time(time, &mouse_log);

        // Compute lagging focus tracking
        smooth_zoom += (target_zoom - smooth_zoom) * 0.15;

        let crop_w = (width as f64 / smooth_zoom) as u32;
        let crop_h = (height as f64 / smooth_zoom) as u32;

        let mut pan_x = (target_x - (crop_w as f64 / 2.0)) as i32;
        let mut pan_y = (target_y - (crop_h as f64 / 2.0)) as i32;

        pan_x = pan_x.clamp(0, (width - crop_w) as i32);
        pan_y = pan_y.clamp(0, (height - crop_h) as i32);

        // Vector extract for crop target window arrays
        let mut cropped_buffer = vec![0u8; (crop_w * crop_h * 3) as usize];
        for row in 0..crop_h {
            let src_start = (((pan_y + row as i32) * width as i32 + pan_x) * 3) as usize;
            let dest_start = (row * crop_w * 3) as usize;
            cropped_buffer[dest_start..(dest_start + (crop_w * 3) as usize)]
                .copy_from_slice(&frame_buffer[src_start..(src_start + (crop_w * 3) as usize)]);
        }

        let src_image = Image::from_vec_u8(
            crop_w as u32,
            crop_h as u32,
            cropped_buffer,
            fr::PixelType::U8x3,
        )
        .unwrap();

        let mut dst_image = Image::new(width as u32, height as u32, fr::PixelType::U8x3);

        let src_view = src_image;

        resizer
            .resize(
                &src_view,
                &mut dst_image,
                &fr::ResizeOptions::new()
                    .resize_alg(fr::ResizeAlg::Interpolation(fr::FilterType::Bilinear)),
            )
            .unwrap();

        // --- START CURSOR DRAWING OVERLAY ---
        // Get mutable access to the underlying destination buffer raw pixels
        let dst_buffer = dst_image.buffer_mut();

        // Define cursor appearance properties
        let cursor_radius = 8i32;
        let cursor_color = [255u8, 0, 0]; // Bright Red [R, G, B]

        // Map mouse position to destination space
        // target_x/y are inside the absolute raw coordinates, but since we map
        // the cropped view directly back to full 1920x1080 canvas, we interpolate relative positions:
        let rel_x = ((target_x - pan_x as f64) * (width as f64 / crop_w as f64)) as i32;
        let rel_y = ((target_y - pan_y as f64) * (height as f64 / crop_h as f64)) as i32;

        // Draw a solid crosshair style circle indicator directly onto the RGB slice
        for dy in -cursor_radius..=cursor_radius {
            for dx in -cursor_radius..=cursor_radius {
                // Circular boundary checking (x^2 + y^2 <= r^2)
                if dx * dx + dy * dy <= cursor_radius * cursor_radius {
                    let pixel_x = rel_x + dx;
                    let pixel_y = rel_y + dy;

                    // Bounds tracking safety check to prevent pan buffer out-of-bounds panics
                    if pixel_x >= 0
                        && pixel_x < width as i32
                        && pixel_y >= 0
                        && pixel_y < height as i32
                    {
                        let idx = ((pixel_y * width as i32 + pixel_x) * 3) as usize;
                        dst_buffer[idx] = cursor_color[0]; // Red channel
                        dst_buffer[idx + 1] = cursor_color[1]; // Green channel
                        dst_buffer[idx + 2] = cursor_color[2]; // Blue channel
                    }
                }
            }
        }
        // --- END CURSOR DRAWING OVERLAY ---

        // Push directly forward into your encoder's native pipeline stream
        encoder_stdin.write_all(dst_image.buffer()).await?;
        current_frame += 1;
    }

    // 5. Explicit cleanup to guarantee file system synchronizations finish
    drop(encoder_stdin);
    let _ = decoder.wait().await;
    let _ = encoder.wait().await;

    println!("✨ Processed {} frames successfully.", current_frame);
    Ok(())
}
