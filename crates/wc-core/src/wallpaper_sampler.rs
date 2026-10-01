//! 壁纸取色引擎 — 按窗口位置采样壁纸颜色，生成渐变主题
//!
//! 混合方案（回答"一定要截屏吗"）：
//! - 静态壁纸：从注册表读壁纸文件路径，直接读图片采样（性能好，< 1ms）
//! - 动态/幻灯片壁纸：壁纸路径不存在或不是图片 → 截屏采样（兼容性好，< 5ms）
//!
//! 采样逻辑：
//! 1. 获取窗口位置（x, y, width, height）
//! 2. 采样该区域的壁纸像素
//! 3. 左半区域平均色 = 渐变起始色
//! 4. 右半区域平均色 = 渐变结束色
//! 5. 整体亮度决定文本颜色（亮背景深色文本，暗背景浅色文本）
//!
//! 刷新策略：
//! - 每 2 秒周期采样（动态壁纸同步）
//! - 收到 ChatWindowMoved 事件时立即重新采样

use crate::event_bus::EventBus;
use crate::module::Module;
use async_trait::async_trait;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Duration;
use wc_common::{Event, EventPayload, PeerKind};
use winreg::enums::*;
use winreg::RegKey;
use image::GenericImageView;

// ═══════════════════════════════════════════════════════════════
// 颜色工具
// ═══════════════════════════════════════════════════════════════

/// RGB 颜色
#[derive(Debug, Clone, Copy)]
struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

impl Rgb {
    fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// 转 CSS hex 字符串
    fn to_css(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// 计算亮度（0-255），用于判断文本颜色
    fn luminance(&self) -> f32 {
        0.299 * self.r as f32 + 0.587 * self.g as f32 + 0.114 * self.b as f32
    }

    /// 根据背景亮度返回合适的文本颜色
    fn text_color(&self) -> Rgb {
        if self.luminance() > 140.0 {
            Rgb::new(30, 30, 30) // 亮背景 → 深色文本
        } else {
            Rgb::new(240, 240, 240) // 暗背景 → 浅色文本
        }
    }
}

/// 像素累加器（用于计算平均色）
#[derive(Debug, Clone, Copy, Default)]
struct PixelAccumulator {
    r: u64,
    g: u64,
    b: u64,
    count: u64,
}

impl PixelAccumulator {
    fn add(&mut self, r: u8, g: u8, b: u8) {
        self.r += r as u64;
        self.g += g as u64;
        self.b += b as u64;
        self.count += 1;
    }

    fn average(&self) -> Option<Rgb> {
        if self.count == 0 {
            return None;
        }
        Some(Rgb::new(
            (self.r / self.count) as u8,
            (self.g / self.count) as u8,
            (self.b / self.count) as u8,
        ))
    }
}

// ═══════════════════════════════════════════════════════════════
// 壁纸采样结果
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
struct SampledTheme {
    primary: Rgb,
    secondary: Rgb,
    text_color: Rgb,
    gradient_css: String,
    source: &'static str, // "static_file" or "screenshot"
}

// ═══════════════════════════════════════════════════════════════
// 壁纸取色引擎
// ═══════════════════════════════════════════════════════════════

pub struct WallpaperSampler {
    window_rect: Arc<RwLock<(i32, i32, u32, u32)>>,
    current_theme: Arc<RwLock<Option<SampledTheme>>>,
    sample_interval_secs: u64,
}

impl WallpaperSampler {
    pub fn new() -> Self {
        Self {
            window_rect: Arc::new(RwLock::new((100, 100, 400, 600))),
            current_theme: Arc::new(RwLock::new(None)),
            sample_interval_secs: 2,
        }
    }

    /// 从注册表读取当前壁纸文件路径
    fn get_wallpaper_path() -> Option<String> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let desktop_key = hkcu.open_subkey(r"Control Panel\Desktop").ok()?;
        let path: String = desktop_key.get_value("Wallpaper").ok()?;
        if path.is_empty() {
            None
        } else {
            Some(path)
        }
    }

    /// 检测是否为静态壁纸（路径指向存在的图片文件）
    fn is_static_wallpaper(path: &str) -> bool {
        if !std::path::Path::new(path).exists() {
            return false;
        }
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "bmp" | "webp")
    }

    /// 采样指定区域的像素，返回左右两半的平均色
    fn sample_region(
        get_pixel: impl Fn(i32, i32) -> Option<(u8, u8, u8)>,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> (Option<Rgb>, Option<Rgb>) {
        let mut left_acc = PixelAccumulator::default();
        let mut right_acc = PixelAccumulator::default();

        let half_width = width / 2;
        let step = 4;

        for py in (0..height as i32).step_by(step) {
            for px in (0..width as i32).step_by(step) {
                if let Some((r, g, b)) = get_pixel(x + px, y + py) {
                    if (px as u32) < half_width {
                        left_acc.add(r, g, b);
                    } else {
                        right_acc.add(r, g, b);
                    }
                }
            }
        }

        (left_acc.average(), right_acc.average())
    }

    /// 从静态壁纸文件采样
    fn sample_from_file(
        path: &str,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> Option<(Rgb, Rgb)> {
        let img = image::open(path).ok()?;
        let (img_w, img_h) = img.dimensions();

        let sx = x.max(0) as u32;
        let sy = y.max(0) as u32;
        let sw = width.min(img_w.saturating_sub(sx));
        let sh = height.min(img_h.saturating_sub(sy));

        if sw == 0 || sh == 0 {
            return None;
        }

        let (left, right) = Self::sample_region(
            |px, py| {
                if px >= 0 && py >= 0 {
                    let ix = px as u32;
                    let iy = py as u32;
                    if ix < img_w && iy < img_h {
                        let p = img.get_pixel(ix, iy);
                        return Some((p.0[0], p.0[1], p.0[2]));
                    }
                }
                None
            },
            sx as i32,
            sy as i32,
            sw,
            sh,
        );

        match (left, right) {
            (Some(l), Some(r)) => Some((l, r)),
            _ => None,
        }
    }

    /// 从截屏采样（用于动态壁纸）
    fn sample_from_screenshot(
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> Option<(Rgb, Rgb)> {
        use windows::Win32::Foundation::*;
        use windows::Win32::Graphics::Gdi::*;
        use windows::Win32::UI::WindowsAndMessaging::*;

        unsafe {
            let screen_dc = GetDC(HWND_DESKTOP);
            if screen_dc.is_invalid() {
                return None;
            }

            let mem_dc = CreateCompatibleDC(screen_dc);
            if mem_dc.is_invalid() {
                ReleaseDC(HWND_DESKTOP, screen_dc);
                return None;
            }

            let bitmap = CreateCompatibleBitmap(screen_dc, width as i32, height as i32);
            if bitmap.is_invalid() {
                DeleteDC(mem_dc);
                ReleaseDC(HWND_DESKTOP, screen_dc);
                return None;
            }

            let old_bitmap = SelectObject(mem_dc, bitmap);

            BitBlt(
                mem_dc,
                0,
                0,
                width as i32,
                height as i32,
                screen_dc,
                x,
                y,
                SRCCOPY,
            );

            // 读取像素数据
            let mut bmi = BITMAPINFO::default();
            bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
            bmi.bmiHeader.biWidth = width as i32;
            bmi.bmiHeader.biHeight = -(height as i32); // 负值 = 从上到下
            bmi.bmiHeader.biPlanes = 1;
            bmi.bmiHeader.biBitCount = 32;
            bmi.bmiHeader.biCompression = BI_RGB.0;

            let mut pixels: Vec<u8> = vec![0; (width * height * 4) as usize];
            GetDIBits(
                mem_dc,
                bitmap,
                0,
                height,
                Some(pixels.as_mut_ptr() as *mut _),
                &mut bmi,
                DIB_RGB_COLORS,
            );

            // 清理
            SelectObject(mem_dc, old_bitmap);
            DeleteObject(bitmap);
            DeleteDC(mem_dc);
            ReleaseDC(HWND_DESKTOP, screen_dc);

            // 采样
            let (left, right) = Self::sample_region(
                |px, py| {
                    let idx = ((py as u32 * width + px as u32) * 4) as usize;
                    if idx + 2 < pixels.len() {
                        // BGRA 格式
                        Some((pixels[idx + 2], pixels[idx + 1], pixels[idx]))
                    } else {
                        None
                    }
                },
                0,
                0,
                width,
                height,
            );

            match (left, right) {
                (Some(l), Some(r)) => Some((l, r)),
                _ => None,
            }
        }
    }

    /// 执行一次采样，返回主题
    fn sample(&self) -> Option<SampledTheme> {
        let (x, y, width, height) = *self.window_rect.read();

        // 尝试静态壁纸
        let wallpaper_path = Self::get_wallpaper_path();
        let (primary, secondary, source) = if let Some(ref path) = wallpaper_path {
            if Self::is_static_wallpaper(path) {
                match Self::sample_from_file(path, x, y, width, height) {
                    Some((l, r)) => (l, r, "static_file"),
                    None => {
                        // 文件采样失败，回退到截屏
                        match Self::sample_from_screenshot(x, y, width, height) {
                            Some((l, r)) => (l, r, "screenshot_fallback"),
                            None => return None,
                        }
                    }
                }
            } else {
                // 不是静态壁纸（动态/幻灯片），截屏
                match Self::sample_from_screenshot(x, y, width, height) {
                    Some((l, r)) => (l, r, "screenshot"),
                    None => return None,
                }
            }
        } else {
            // 读不到壁纸路径，截屏
            match Self::sample_from_screenshot(x, y, width, height) {
                Some((l, r)) => (l, r, "screenshot"),
                None => return None,
            }
        };

        // 用主色（左半）决定文本颜色
        let text_color = primary.text_color();

        // 生成渐变 CSS（从左到右）
        let gradient_css = format!(
            "linear-gradient(135deg, {} 0%, {} 100%)",
            primary.to_css(),
            secondary.to_css()
        );

        Some(SampledTheme {
            primary,
            secondary,
            text_color,
            gradient_css,
            source,
        })
    }
}

impl Default for WallpaperSampler {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════
// Module trait
// ═══════════════════════════════════════════════════════════════

#[async_trait]
impl Module for WallpaperSampler {
    fn name(&self) -> &str {
        "WallpaperSampler"
    }

    async fn on_start(&mut self, bus: &EventBus) {
        tracing::info!("[Wallpaper] WallpaperSampler started");

        let bus_clone = bus.clone();
        let window_rect = self.window_rect.clone();
        let current_theme = self.current_theme.clone();
        let interval = self.sample_interval_secs;

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_secs(interval));
            loop {
                ticker.tick().await;

                // 执行采样（在阻塞线程池中运行，因为截屏和图片IO是同步的）
                let rect = *window_rect.read();
                let theme_result = tokio::task::spawn_blocking(move || {
                    let sampler = WallpaperSampler::new();
                    *sampler.window_rect.write() = rect;
                    sampler.sample()
                })
                .await
                .unwrap_or(None);

                if let Some(theme) = theme_result {
                    // 检查主题是否有明显变化（避免频繁发事件）
                    let changed = {
                        let current = current_theme.read();
                        match current.as_ref() {
                            Some(t) => {
                                (t.primary.r as i32 - theme.primary.r as i32).abs() > 8
                                    || (t.primary.g as i32 - theme.primary.g as i32).abs() > 8
                                    || (t.primary.b as i32 - theme.primary.b as i32).abs() > 8
                            }
                            None => true,
                        }
                    };

                    if changed {
                        tracing::info!(
                            "[Wallpaper] Theme updated: primary={}, secondary={}, source={}",
                            theme.primary.to_css(),
                            theme.secondary.to_css(),
                            theme.source
                        );

                        *current_theme.write() = Some(theme.clone());

                        bus_clone
                            .publish(Event::new(
                                PeerKind::Core,
                                EventPayload::WallpaperSampled {
                                    primary_color: theme.primary.to_css(),
                                    secondary_color: theme.secondary.to_css(),
                                    gradient_css: theme.gradient_css,
                                    text_color: theme.text_color.to_css(),
                                },
                            ))
                            .await;
                    }
                }
            }
        });
    }

    async fn on_event(&mut self, event: &Event) {
        match &event.payload {
            // 聊天窗位置变化 → 更新采样区域
            EventPayload::ChatWindowMoved { x, y, width, height } => {
                tracing::debug!(
                    "[Wallpaper] Window moved to ({},{}), size {}x{}, will resample",
                    x,
                    y,
                    width,
                    height
                );
                *self.window_rect.write() = (*x, *y, *width, *height);
            }
            // 聊天窗打开 → 设置初始采样区域
            EventPayload::ChatWindowOpened { x, y, width, height } => {
                tracing::info!(
                    "[Wallpaper] Chat window opened at ({},{}), size {}x{}",
                    x,
                    y,
                    width,
                    height
                );
                *self.window_rect.write() = (*x, *y, *width, *height);
            }
            _ => {}
        }
    }
}
