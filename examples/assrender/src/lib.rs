use anyhow::Result;
use krkrz_plugin_base::{tp_stub::*, *};
use log::Log;
use rassa::{FontProvider, Renderer, RendererConfig, Script};
use rassa_fonts::{AttachedFontProvider, CrossfontProvider, FontAttachment, MergedFontProvider};
use std::io::Read;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};

static IS_UNLOADING: AtomicBool = AtomicBool::new(false);
generate_origin_static_block!(ass_render);

struct Logger {}

impl Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        if IS_UNLOADING.load(Ordering::Relaxed) {
            return false;
        }
        metadata.level() <= log::Level::Debug
    }
    fn flush(&self) {}
    fn log(&self, record: &log::Record) {
        if IS_UNLOADING.load(Ordering::Relaxed) {
            return;
        }
        log!("[ass-render][{}]{}", record.level(), record.args()); // 打印rassa消息到TVP控制台，由于没全局引入log::*，因此不存在循环调用log
    }
}

const LOGGER: Logger = Logger {};

pub struct AssRender {
    script: Script,
    renderer: Renderer,
    config: RendererConfig,
    provider: Box<dyn FontProvider>,
}

#[Tjs2Class]
impl AssRender {
    pub fn new(
        width: i64,
        height: i64,
        ass_path: String,
        font_paths: Option<Vec<String>>,
    ) -> Result<Self> {
        if ass_path.is_empty() {
            anyhow::bail!("Ass path can not be empty.");
        }
        let mut stream = match create_istream(&ass_path, TJS_BS_READ) {
            Some(stream) => com::IStreamWrapper(stream),
            None => anyhow::bail!("Can not load ass file: {}", ass_path),
        };
        let mut text = String::new();
        stream.read_to_string(&mut text)?;
        let script = Script::parse(&text)?;
        let mut config = script.default_config();
        config.frame.height = height as i32;
        config.frame.width = width as i32;
        let renderer = Renderer::new();
        let provider = if let Some(paths) = font_paths {
            let mut attachs = Vec::new();
            for s in paths {
                let mut stream = match create_istream(&s, TJS_BS_READ) {
                    Some(stream) => com::IStreamWrapper(stream),
                    None => anyhow::bail!("Can not load font file: {}", s),
                };
                let mut data = Vec::new();
                stream.read_to_end(&mut data)?;
                attachs.push(FontAttachment { name: s, data });
            }
            Box::new(MergedFontProvider::new(
                AttachedFontProvider::from_attachments(&attachs),
                CrossfontProvider::with_fallback_family("Microsoft YaHei"),
            )) as Box<dyn FontProvider>
        } else {
            Box::new(CrossfontProvider::with_fallback_family("Microsoft YaHei"))
                as Box<dyn FontProvider>
        };
        Ok(Self {
            script,
            renderer,
            config,
            provider,
        })
    }

    pub fn render(&self, layer: &mut tTJSVariant, now_ms: i64) -> Result<()> {
        let layer = layer.as_object_no_add_ref();
        if layer.is_null() {
            anyhow::bail!("Layer is not a object.");
        }
        let image = self.renderer.render_frame_with_config(
            &self.script,
            &self.provider,
            now_ms,
            &self.config,
        )?;
        let mut val = tTJSVariant::new();
        let re = unsafe {
            (*layer).prop_get(
                0,
                tjs_w!("mainImageBufferForWrite"),
                ptr::null_mut(),
                &mut val,
                layer,
            )
        };
        if TJS_FAILED(re) {
            anyhow::bail!("Failed to get image buffer");
        }
        let buffer = val.as_integer() as *mut u8;
        if buffer.is_null() {
            anyhow::bail!("Failed to get image buffer (nullptr)")
        }
        val.clear();
        let re = unsafe {
            (*layer).prop_get(
                0,
                tjs_w!("mainImageBufferPitch"),
                ptr::null_mut(),
                &mut val,
                layer,
            )
        };
        if TJS_FAILED(re) {
            anyhow::bail!("Failed to get image buffer pitch");
        }
        let pitch = val.as_integer();
        val.clear();
        let re =
            unsafe { (*layer).prop_get(0, tjs_w!("imageWidth"), ptr::null_mut(), &mut val, layer) };
        if TJS_FAILED(re) {
            anyhow::bail!("Failed to get image width");
        }
        let width = val.as_integer();
        val.clear();
        let re = unsafe {
            (*layer).prop_get(0, tjs_w!("imageHeight"), ptr::null_mut(), &mut val, layer)
        };
        if TJS_FAILED(re) {
            anyhow::bail!("Failed to get image height");
        }
        let height = val.as_integer();
        val.clear();
        for img in image.planes {
            if img.size.height == 0 || img.size.width == 0 {
                continue;
            }
            let r = (img.color.0 >> 24) & 0xFF;
            let g = (img.color.0 >> 16) & 0xFF;
            let b = (img.color.0 >> 8) & 0xFF;
            let a = 255 - (img.color.0 & 0xFF);
            for y in 0..img.size.height {
                let target_y = img.destination.y + y;
                if target_y < 0 || (target_y as i64) >= height {
                    continue;
                }
                let row_ptr = unsafe { buffer.offset(target_y as isize * pitch as isize) };
                let row_buf =
                    unsafe { std::slice::from_raw_parts_mut(row_ptr, (width * 4) as usize) };
                for x in 0..img.size.width {
                    let target_x = img.destination.x + x;
                    if target_x < 0 || (target_x as i64) >= width {
                        continue;
                    }
                    let src_alpha = img.bitmap[y as usize * img.stride as usize + x as usize];
                    if src_alpha == 0 {
                        continue;
                    }
                    let final_alpha = (src_alpha as u32 * a) / 255;
                    let pos = (target_x as usize) * 4;
                    let dst_b = row_buf[pos];
                    let dst_g = row_buf[pos + 1];
                    let dst_r = row_buf[pos + 2];
                    let dst_a = row_buf[pos + 3];
                    let out_a = (final_alpha + dst_a as u32 * (255 - final_alpha) / 255) as u8;
                    let out_a_u32 = if out_a == 0 { 1 } else { out_a as u32 };
                    let out_r = ((r * final_alpha
                        + dst_r as u32 * dst_a as u32 * (255 - final_alpha) / 255)
                        / out_a_u32) as u8;
                    let out_g = ((g * final_alpha
                        + dst_g as u32 * dst_a as u32 * (255 - final_alpha) / 255)
                        / out_a_u32) as u8;
                    let out_b = ((b * final_alpha
                        + dst_b as u32 * dst_a as u32 * (255 - final_alpha) / 255)
                        / out_a_u32) as u8;
                    row_buf[pos] = out_b;
                    row_buf[pos + 1] = out_g;
                    row_buf[pos + 2] = out_r;
                    row_buf[pos + 3] = out_a;
                }
            }
        }
        let err = unsafe {
            (*layer).func_call(
                0,
                tjs_w!("update"),
                ptr::null_mut(),
                ptr::null_mut(),
                0,
                ptr::null_mut(),
                layer,
            )
        };
        if TJS_FAILED(err) {
            anyhow::bail!("Failed to update layer")
        }
        Ok(())
    }
}

#[unsafe(export_name = "V2Link")]
unsafe extern "system" fn v2_link(exporter: *mut iTVPFunctionExporter) -> i32 {
    unsafe {
        TVPInitImportStub(exporter);
    }
    let ass_render = AssRender::create_native_class().1;
    register_var!(ass_render);
    IS_UNLOADING.store(false, Ordering::SeqCst);
    let _ = log::set_logger(&LOGGER);
    0
}

#[unsafe(export_name = "V2Unlink")]
unsafe extern "system" fn v2_unlink() -> i32 {
    log!("[assrender]This plugin can not unlink safely.");
    TJS_E_FAIL
}
