use crate::fr::images::Image;
use fast_image_resize::{self as fr};
use napi_derive::napi;
use serde::Deserialize;
use std::process::Stdio;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::process::Child;
use tokio::process::ChildStderr;
use tokio::process::ChildStdin;
use tokio::process::ChildStdout;
use tokio::process::Command;
use tokio::sync::mpsc;

#[napi(object)]
#[derive(Deserialize, Debug, Clone)]
pub struct ZoomLogEntry {
    pub t: f64,
    pub zoom: f64,
}

#[napi(object)]
#[derive(Deserialize, Debug, Clone)]
pub struct PostProcessRequest {
    pub video_path: String,
    pub zoom_log: Vec<ZoomLogEntry>,
    pub mouse_log: Vec<MouseLogEntry>,
    pub crf: Option<u32>,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
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

fn draw_cursor(
    rel_x: i32,
    rel_y: i32,
    width: u32,
    height: u32,
    cursor_radius: i32,
    dst_buffer: &mut [u8],
) {
    let cursor_color = [255u8, 0, 0];
    let radius_range = -cursor_radius..=cursor_radius;
    for dy in radius_range.clone() {
        for dx in radius_range.clone() {
            if dx * dx + dy * dy <= cursor_radius * cursor_radius {
                let pixel_x = rel_x + dx;
                let pixel_y = rel_y + dy;
                if pixel_x >= 0 && pixel_x < width as i32 && pixel_y >= 0 && pixel_y < height as i32
                {
                    let idx = ((pixel_y * width as i32 + pixel_x) * 3) as usize;
                    dst_buffer[idx] = cursor_color[0];
                    dst_buffer[idx + 1] = cursor_color[1];
                    dst_buffer[idx + 2] = cursor_color[2];
                }
            }
        }
    }
}

fn spawn_decoder(
    ffmpeg_binary: &str,
    video_path: &str,
) -> napi::Result<(Child, ChildStdout, ChildStderr)> {
    let mut decoder_cmd = Command::new(ffmpeg_binary);
    decoder_cmd.args([
        "-i", video_path, "-f", "rawvideo", "-pix_fmt", "rgb24", "-an", "-",
    ]);
    decoder_cmd.stdin(Stdio::null());
    decoder_cmd.stdout(Stdio::piped());
    decoder_cmd.stderr(Stdio::piped());

    let mut decoder = decoder_cmd
        .spawn()
        .map_err(|e| napi::Error::from_reason(format!("Failed to spawn decoder: {}", e)))?;

    let decoder_stdout = decoder
        .stdout
        .take()
        .ok_or_else(|| napi::Error::from_reason("Failed to open decoder stdout pipe"))?;
    let decoder_stderr = decoder
        .stderr
        .take()
        .ok_or_else(|| napi::Error::from_reason("Failed to open decoder stderr pipe"))?;

    Ok((decoder, decoder_stdout, decoder_stderr))
}

/// Spawns the FFmpeg encoder process and extracts its I/O handles.
fn spawn_encoder(
    ffmpeg_binary: &str,
    width: u32,
    height: u32,
    crf: u32,
    fps: f64,
) -> napi::Result<(Child, ChildStdin, ChildStderr)> {
    let mut encoder_cmd = Command::new(ffmpeg_binary);
    encoder_cmd.args([
        "-y",
        "-f",
        "rawvideo",
        "-pix_fmt",
        "rgb24",
        "-s",
        &format!("{}x{}", &width, &height),
        "-r",
        &fps.to_string(),
        "-i",
        "-",
        "-c:v",
        "libvpx-vp9",
        "-crf",
        &crf.to_string(),
        "-b:v",
        "0",
        "-pix_fmt",
        "yuv420p",
        "./results/output_processed.webm",
    ]);
    encoder_cmd.stdin(Stdio::piped());
    encoder_cmd.stdout(Stdio::null());
    encoder_cmd.stderr(Stdio::piped());

    let mut encoder = encoder_cmd
        .spawn()
        .map_err(|e| napi::Error::from_reason(format!("Failed to spawn encoder: {}", e)))?;

    let encoder_stdin = encoder
        .stdin
        .take()
        .ok_or_else(|| napi::Error::from_reason("Failed to open encoder stdin pipe"))?;
    let encoder_stderr = encoder
        .stderr
        .take()
        .ok_or_else(|| napi::Error::from_reason("Failed to open encoder stderr pipe"))?;

    Ok((encoder, encoder_stdin, encoder_stderr))
}

async fn process_and_encode_frame<'a>(
    frame_buffer: &[u8],
    request: &PostProcessRequest,
    current_frame: &mut u64,
    smooth_zoom: &mut f64,
    resizer: &mut fr::Resizer,
    dst_image: &mut Image<'a>,
    encoder_stdin: &mut ChildStdin,
) -> napi::Result<()> {
    let time = *current_frame as f64 / request.fps as f64;

    // Calculate dynamic zoom and panning constraints
    let target_zoom = get_zoom_at_time(time, &request.zoom_log);
    let (target_x, target_y) = get_mouse_at_time(time, &request.mouse_log);
    *smooth_zoom += (target_zoom - *smooth_zoom) * 0.15;

    let crop_w = (request.width as f64 / *smooth_zoom) as u32;
    let crop_h = (request.height as f64 / *smooth_zoom) as u32;

    let mut pan_x = (target_x - (crop_w as f64 / 2.0)) as i32;
    let mut pan_y = (target_y - (crop_h as f64 / 2.0)) as i32;
    pan_x = pan_x.clamp(0, (request.width - crop_w) as i32);
    pan_y = pan_y.clamp(0, (request.height - crop_h) as i32);

    // Extract cropped window from raw frame buffer
    let mut cropped_buffer = vec![0u8; (crop_w * crop_h * 3) as usize];
    for row in 0..crop_h {
        let src_start = (((pan_y + row as i32) * request.width as i32 + pan_x) * 3) as usize;
        let dest_start = (row * crop_w * 3) as usize;
        let stride = (crop_w * 3) as usize;

        cropped_buffer[dest_start..(dest_start + stride)]
            .copy_from_slice(&frame_buffer[src_start..(src_start + stride)]);
    }

    // Resize cropped region to target dimensions
    let src_image = Image::from_vec_u8(crop_w, crop_h, cropped_buffer, fr::PixelType::U8x3)
        .map_err(|e| napi::Error::from_reason(format!("Failed to create source image: {}", e)))?;

    resizer
        .resize(
            &src_image,
            dst_image,
            &fr::ResizeOptions::new()
                .resize_alg(fr::ResizeAlg::Interpolation(fr::FilterType::Bilinear)),
        )
        .map_err(|e| napi::Error::from_reason(format!("Resize failed: {}", e)))?;

    // 4. Calculate relative cursor position and apply drawing modifications
    let rel_x = ((target_x - pan_x as f64) * (request.width as f64 / crop_w as f64)) as i32;
    let rel_y = ((target_y - pan_y as f64) * (request.height as f64 / crop_h as f64)) as i32;
    let cursor_radius = 8i32;

    // Mutate dst_image buffer directly inside your helper
    let dst_buffer = dst_image.buffer_mut();

    draw_cursor(
        rel_x,
        rel_y,
        request.width,
        request.height,
        cursor_radius,
        dst_buffer,
    );

    // 5. Asynchronously write down into the encoder standard input stream
    encoder_stdin
        .write_all(dst_image.buffer())
        .await
        .map_err(|_| napi::Error::from_reason("Encoder pipeline stdin closed prematurely"))?;

    *current_frame += 1;
    Ok(())
}

#[napi]
pub async fn process_video_pipeline_impl(request: PostProcessRequest) -> napi::Result<String> {
    let frame_size = (request.width as usize) * (request.height as usize) * 3;

    // Use an async bounded channel instead of crossbeam for raw frame backpressure
    let (frame_tx, mut frame_rx) = mpsc::channel::<Vec<u8>>(5);

    // TODO: Allow ffmpeg binary path override
    let ffmpeg_binary = "ffmpeg";

    let (mut decoder, mut decoder_stdout, decoder_stderr) =
        spawn_decoder(&ffmpeg_binary, &request.video_path)?;

    let (mut encoder, mut encoder_stdin, encoder_stderr) = spawn_encoder(
        &ffmpeg_binary,
        request.width,
        request.height,
        request.crf.unwrap_or(4),
        request.fps,
    )?;

    // Stream raw input frame buffers out of decoder
    let frame_producer_task = tokio::spawn(async move {
        let mut frame_buffer = vec![0u8; frame_size];
        loop {
            if decoder_stdout.read_exact(&mut frame_buffer).await.is_err() {
                break;
            }
            if frame_tx.send(frame_buffer.clone()).await.is_err() {
                break;
            }
        }
    });

    // Initialize multiplexed logging engine
    let dec_log_reader = BufReader::new(decoder_stderr).lines();
    let enc_log_reader = BufReader::new(encoder_stderr).lines();

    let logging_task = tokio::spawn(async move {
        let mut dec_lines = dec_log_reader;
        let mut enc_lines = enc_log_reader;
        loop {
            tokio::select! {
                res = dec_lines.next_line() => {
                    if let Ok(Some(line)) = res { println!("[Decoder FFMPEG Log] {}", line); }
                    else { break; }
                }
                res = enc_lines.next_line() => {
                    if let Ok(Some(line)) = res { println!("[Encoder FFMPEG Log] {}", line); }
                    else { break; }
                }
            }
        }
    });

    // Main loop
    let mut current_frame = 0;
    let mut smooth_zoom = if !request.zoom_log.is_empty() {
        request.zoom_log[0].zoom
    } else {
        1.0
    };
    let mut resizer = fr::Resizer::new();
    let mut dst_image = Image::new(request.width, request.height, fr::PixelType::U8x3);

    // Asynchronously pull raw frames out of our queue
    while let Some(frame_buffer) = frame_rx.recv().await {
        process_and_encode_frame(
            &frame_buffer,
            &request,
            &mut current_frame,
            &mut smooth_zoom,
            &mut resizer,
            &mut dst_image,
            &mut encoder_stdin,
        )
        .await?;
    }

    // Cleanup
    let _ = encoder_stdin.flush().await;
    drop(encoder_stdin); // Drop the handle to signal EOF to FFmpeg encoder

    // Wait for all processes and minor tasks to safely yield back
    let _ = tokio::try_join!(
        async { decoder.wait().await.map_err(|e| e.to_string()) },
        async { encoder.wait().await.map_err(|e| e.to_string()) },
        async { frame_producer_task.await.map_err(|e| e.to_string()) },
        async { logging_task.await.map_err(|e| e.to_string()) }
    )
    .map_err(|e| napi::Error::from_reason(format!("Pipeline cleanup failure: {}", e)))?;

    Ok(format!("Successfully processed {} frames.", current_frame))
}
