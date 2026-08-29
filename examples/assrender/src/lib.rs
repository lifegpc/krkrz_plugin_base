use anyhow::Result;
use krkrz_plugin_base::{tp_stub::*, *};
use log::Log;
use rassa_core::{RendererConfig, ass};
use rassa_fonts::{
    AttachedFontProvider, CrossfontProvider, FontAttachment, FontProvider, MergedFontProvider,
};
use rassa_parse::{ParsedTrack, parse_script_text};
use rassa_render::RenderEngine;
use serde::Deserialize;
use std::io::Read;
use std::ptr;

generate_origin_static_block!(ass_render);

struct Logger {}

impl Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Debug
    }
    fn flush(&self) {}
    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            krkrz_plugin_base::log!(
                "[ass-render][{}][{}]{}",
                record.level(),
                record.target(),
                record.args()
            );
        }
    }
}

const LOGGER: Logger = Logger {};

#[derive(Debug, Default, Deserialize)]
struct SizeConfig {
    width: Option<i32>,
    height: Option<i32>,
}

#[derive(Debug, Default, Deserialize)]
struct MarginsConfig {
    top: Option<i32>,
    bottom: Option<i32>,
    left: Option<i32>,
    right: Option<i32>,
}

#[derive(Debug, Deserialize)]
enum HintingName {
    #[serde(alias = "none")]
    None,
    #[serde(alias = "light")]
    Light,
    #[serde(alias = "normal")]
    Normal,
    #[serde(alias = "native")]
    Native,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum HintingConfig {
    Value(i32),
    Name(HintingName),
}

impl HintingConfig {
    fn into_rassa(self) -> Result<ass::Hinting> {
        match self {
            Self::Value(value) => match value {
                0 => Ok(ass::Hinting::None),
                1 => Ok(ass::Hinting::Light),
                2 => Ok(ass::Hinting::Normal),
                3 => Ok(ass::Hinting::Native),
                _ => anyhow::bail!("Invalid hinting value: {value}"),
            },
            Self::Name(name) => Ok(match name {
                HintingName::None => ass::Hinting::None,
                HintingName::Light => ass::Hinting::Light,
                HintingName::Normal => ass::Hinting::Normal,
                HintingName::Native => ass::Hinting::Native,
            }),
        }
    }
}

#[derive(Debug, Deserialize)]
enum ShapingName {
    #[serde(alias = "simple")]
    Simple,
    #[serde(alias = "complex")]
    Complex,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ShapingConfig {
    Value(i32),
    Name(ShapingName),
}

impl ShapingConfig {
    fn into_rassa(self) -> Result<ass::ShapingLevel> {
        match self {
            Self::Value(value) => match value {
                0 => Ok(ass::ShapingLevel::Simple),
                1 => Ok(ass::ShapingLevel::Complex),
                _ => anyhow::bail!("Invalid shaping value: {value}"),
            },
            Self::Name(name) => Ok(match name {
                ShapingName::Simple => ass::ShapingLevel::Simple,
                ShapingName::Complex => ass::ShapingLevel::Complex,
            }),
        }
    }
}

/// Optional overrides for rassa's renderer configuration.
///
/// `frame` is intentionally omitted: the constructor's `width` and `height`
/// arguments always define the output frame size.
#[derive(Debug, Default, Deserialize)]
pub struct Config {
    storage: Option<SizeConfig>,
    margins: Option<MarginsConfig>,
    use_margins: Option<bool>,
    pixel_aspect: Option<f64>,
    font_scale: Option<f64>,
    selective_font_scale: Option<bool>,
    line_spacing: Option<f64>,
    line_position: Option<f64>,
    hinting: Option<HintingConfig>,
    shaping: Option<ShapingConfig>,
    wrap_unicode: Option<bool>,
    bidi_brackets: Option<bool>,
    whole_text_layout: Option<bool>,
}

impl Config {
    fn apply_to(self, target: &mut RendererConfig) -> Result<()> {
        if let Some(storage) = self.storage {
            if let Some(width) = storage.width {
                target.storage.width = width;
            }
            if let Some(height) = storage.height {
                target.storage.height = height;
            }
        }
        if let Some(margins) = self.margins {
            if let Some(top) = margins.top {
                target.margins.top = top;
            }
            if let Some(bottom) = margins.bottom {
                target.margins.bottom = bottom;
            }
            if let Some(left) = margins.left {
                target.margins.left = left;
            }
            if let Some(right) = margins.right {
                target.margins.right = right;
            }
        }
        if let Some(value) = self.use_margins {
            target.use_margins = value;
        }
        if let Some(value) = self.pixel_aspect {
            target.pixel_aspect = value;
        }
        if let Some(value) = self.font_scale {
            target.font_scale = value;
        }
        if let Some(value) = self.selective_font_scale {
            target.selective_font_scale = value;
        }
        if let Some(value) = self.line_spacing {
            target.line_spacing = value;
        }
        if let Some(value) = self.line_position {
            target.line_position = value;
        }
        if let Some(value) = self.hinting {
            target.hinting = value.into_rassa()?;
        }
        if let Some(value) = self.shaping {
            target.shaping = value.into_rassa()?;
        }
        if let Some(value) = self.wrap_unicode {
            target.wrap_unicode = value;
        }
        if let Some(value) = self.bidi_brackets {
            target.bidi_brackets = value;
        }
        if let Some(value) = self.whole_text_layout {
            target.whole_text_layout = value;
        }
        Ok(())
    }
}

pub struct AssRender {
    script: ParsedTrack,
    renderer: RenderEngine,
    config: RendererConfig,
    provider: Box<dyn FontProvider>,
}

#[Tjs2Class]
impl AssRender {
    #[tjs(serde)]
    pub fn new(
        width: i64,
        height: i64,
        ass_path: String,
        font_paths: Option<Vec<String>>,
        config: Option<Config>,
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
        log::info!("Loaded ass: {}", ass_path);
        let script = parse_script_text(&text)?;
        log::info!("ParsedTrack: {} Events", script.events.len());
        let mut renderer_config = rassa_render::default_renderer_config(&script);
        renderer_config.frame.height = height as i32;
        renderer_config.frame.width = width as i32;
        if let Some(overrides) = config {
            overrides.apply_to(&mut renderer_config)?;
        }
        let renderer = RenderEngine::new();
        let provider = if let Some(paths) = font_paths {
            let mut attachs = Vec::new();
            for s in paths {
                let mut stream = match create_istream(&s, TJS_BS_READ) {
                    Some(stream) => com::IStreamWrapper(stream),
                    None => anyhow::bail!("Can not load font file: {}", s),
                };
                let mut data = Vec::new();
                stream.read_to_end(&mut data)?;
                log::info!("Loaded font: {}", s);
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
            config: renderer_config,
            provider,
        })
    }

    pub fn render(&self, layer: &mut tTJSVariant, now_ms: i64) -> Result<()> {
        let layer = layer.as_object_no_add_ref();
        if layer.is_null() {
            anyhow::bail!("Layer is not a object.");
        }
        let image = self.renderer.render_frame_with_provider_and_config(
            &self.script,
            &self.provider,
            now_ms,
            &self.config,
        );
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
        for img in image {
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
    if let Err(e) = log::set_logger(&LOGGER) {
        krkrz_plugin_base::log!("[ass-render]Failed to set log crate: {}", e);
    }
    log::set_max_level(log::LevelFilter::Debug);
    0
}

#[unsafe(export_name = "V2Unlink")]
unsafe extern "system" fn v2_unlink() -> i32 {
    log!("[assrender]This plugin can not unlink safely.");
    TJS_E_FAIL
}
