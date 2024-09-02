use std::collections::HashMap;
use std::sync::Arc;
use krilla::color::rgb;
use krilla::font::GlyphId;
use krilla::geom::{Point, Transform};
use krilla::PageSettings;
use krilla::path::{Fill, Stroke};
use krilla::surface::Surface;
use typst::foundations::{Datetime, Smart};
use typst::layout::{Frame, FrameItem, PageRanges};
use typst::model::Document;
use typst::text::Font;
use typst::visualize::{ColorSpace, ImageKind, Paint, RasterFormat, Rgb};
use crate::page::{alloc_page_refs, traverse_pages, write_page_tree};
use crate::{AbsExt, GlobalRefs, PdfBuilder, References};
use crate::catalog::write_catalog;
use crate::color::alloc_color_functions_refs;
use crate::color_font::write_color_fonts;
use crate::extg::write_graphic_states;
use crate::font::write_fonts;
use crate::gradient::write_gradients;
use crate::image::write_images;
use crate::named_destination::write_named_destinations;
use crate::pattern::write_patterns;
use crate::resources::{alloc_resources_refs, write_resource_dictionaries};

pub struct ExportContext {
    fonts: HashMap<Font, krilla::font::Font>
}

impl ExportContext {
    pub fn new() -> Self {
        Self {
            fonts: Default::default(),
        }
    }
}

pub fn pdf(
    typst_document: &Document,
) -> Vec<u8> {
    let mut document = krilla::Document::new();
    let mut context = ExportContext::new();

    for typst_page in &typst_document.pages {
        let settings = PageSettings::new(typst_page.frame.width().to_f32(), typst_page.frame.height().to_f32());
        let mut page = document.start_page_with(settings);
        let mut surface = page.surface();
        handle_frame(&typst_page.frame, &mut surface, &mut context);
    }

    document.finish().unwrap()
}

pub fn handle_frame(frame: &Frame, surface: &mut Surface, context: &mut ExportContext) {
    for (point, item) in frame.items() {
        surface.push_transform(&Transform::from_translate(point.x.to_f32(), point.y.to_f32()));

        match item {
            FrameItem::Group(g) => {
                surface.push_transform(&convert_transform(g.transform));
                handle_frame(&g.frame, surface, context);
                surface.pop();
            }
            FrameItem::Text(t) => {
                let font = context.fonts.entry(t.font.clone()).or_insert_with(|| {
                    krilla::font::Font::new(Arc::new(t.font.data().to_vec()), t.font.index(), vec![]).unwrap()
                }).clone();
                let paint = convert_paint(&t.fill);
                let fill = Fill {
                    paint,
                    ..Default::default()
                };
                let text = t.text.as_str();
                let size = t.size;

                let glyphs = t.glyphs.iter().map(|g| {
                    krilla::font::Glyph::new(
                        GlyphId::new(g.id as u32),
                        g.x_advance.at(size).to_f32(),
                        g.x_offset.at(size).to_f32(),
                        0.0,
                        g.range.start as usize..g.range.end as usize,
                        size.to_f32()
                    )
                }).collect::<Vec<_>>();

                surface.fill_glyphs(
                    Point::from_xy(0.0, 0.0),
                    fill,
                    &glyphs,
                    font.clone(),
                    text
                );

                if let Some(stroke) = &t.stroke {
                    let krilla_stroke = Stroke {
                        paint: convert_paint(&stroke.paint),
                        width: stroke.thickness.to_f32(),
                        miter_limit: stroke.miter_limit.get() as f32,
                        ..Default::default()
                    };

                    surface.stroke_glyphs(
                        Point::from_xy(0.0, 0.0),
                        krilla_stroke,
                        &glyphs,
                        font.clone(),
                        text
                    );
                }
            }
            FrameItem::Shape(_, _) => {}
            FrameItem::Image(image, size, span) => {
                match image.kind() {
                    ImageKind::Raster(raster) => {
                        let image = match raster.format() {
                            RasterFormat::Png => krilla::image::Image::from_png(raster.data()),
                            RasterFormat::Jpg => krilla::image::Image::from_jpeg(raster.data()),
                            RasterFormat::Gif => krilla::image::Image::from_gif(raster.data()),
                        }.unwrap();
                        surface.draw_image(image, krilla::geom::Size::from_wh(size.x.to_f32(), size.y.to_f32()).unwrap());
                    }
                    ImageKind::Svg(_) => {}
                }
            }
            FrameItem::Link(_, _) => {}
            FrameItem::Tag(_) => {}
        }

        surface.pop();
    }
}

fn convert_transform(t: crate::Transform) -> krilla::geom::Transform {
    Transform::from_row(t.sx.get() as f32, t.ky.get() as f32, t.kx.get() as f32, t.sy.get() as f32, t.tx.to_f32(), t.ty.to_f32())
}

fn convert_paint(paint: &Paint) -> krilla::paint::Paint<krilla::color::rgb::Rgb> {
    match paint {
        Paint::Solid(c) => {
            let components = c.to_space(ColorSpace::Srgb).to_vec4_u8();
            krilla::paint::Paint::Color(rgb::Color::new(components[0], components[1], components[2]))
        }
        Paint::Gradient(_) => {
            krilla::paint::Paint::Color(rgb::Color::black())
        }
        Paint::Pattern(_) => {
            krilla::paint::Paint::Color(rgb::Color::black())
        }
    }
}

