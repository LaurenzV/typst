use std::collections::HashMap;
use std::sync::Arc;
use krilla::color::rgb;
use krilla::font::GlyphId;
use krilla::geom::{Point, Transform};
use krilla::PageSettings;
use krilla::path::{Fill, PathBuilder, Stroke};
use krilla::surface::Surface;
use pdf_writer::types::StructRole::P;
use svg2pdf::usvg::{NormalizedF32, Rect};
use typst::foundations::{Datetime, Smart};
use typst::layout::{Frame, FrameItem, GroupItem, PageRanges, Size};
use typst::model::Document;
use typst::text::{Font, TextItem};
use typst::visualize::{ColorSpace, FillRule, FixedStroke, Geometry, Gradient, Image, ImageKind, LineCap, LineJoin, Paint, PathItem, RasterFormat, Rgb, Shape};
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

pub fn handle_group(group: &GroupItem, surface: &mut Surface, context: &mut ExportContext) {
    surface.push_transform(&convert_transform(group.transform));
    handle_frame(&group.frame, surface, context);
    surface.pop();
}

pub fn handle_text(t: &TextItem, surface: &mut Surface, context: &mut ExportContext) {
    let font = context.fonts.entry(t.font.clone()).or_insert_with(|| {
        krilla::font::Font::new(Arc::new(t.font.data().to_vec()), t.font.index(), vec![]).unwrap()
    }).clone();
    let (paint, opacity) = convert_paint(&t.fill);
    let fill = Fill {
        paint,
        opacity: NormalizedF32::new(opacity as f32 / 255.0).unwrap(),
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

    if let Some(stroke) = t.stroke.as_ref().map(convert_fixed_stroke) {
        surface.stroke_glyphs(
            Point::from_xy(0.0, 0.0),
            stroke,
            &glyphs,
            font.clone(),
            text
        );
    }
}

pub fn handle_image(image: &Image, size: &Size, surface: &mut Surface, _: &mut ExportContext) {
    match image.kind() {
        ImageKind::Raster(raster) => {
            let image = match raster.format() {
                RasterFormat::Png => krilla::image::Image::from_png(raster.data()),
                RasterFormat::Jpg => krilla::image::Image::from_jpeg(raster.data()),
                RasterFormat::Gif => krilla::image::Image::from_gif(raster.data()),
            }.unwrap();
            surface.draw_image(image, krilla::geom::Size::from_wh(size.x.to_f32(), size.y.to_f32()).unwrap());
        }
        ImageKind::Svg(svg) => {
            surface.draw_svg(svg.tree(), krilla::geom::Size::from_wh(size.x.to_f32(), size.y.to_f32()).unwrap());
        }
    }
}

pub fn handle_shape(shape: &Shape, surface: &mut Surface) {
    let mut path_builder = PathBuilder::new();

    match &shape.geometry {
        Geometry::Line(l) => {
            let mut path_builder = PathBuilder::new();
            path_builder.move_to(0.0, 0.0);
            path_builder.line_to(l.x.to_f32(), l.y.to_f32());
        }
        Geometry::Rect(r) => {
            let mut path_builder = PathBuilder::new();
            path_builder.push_rect(Rect::from_xywh(0.0, 0.0, r.x.to_f32(), r.y.to_f32()).unwrap());
        }
        Geometry::Path(p) => {
            for item in &p.0 {
                match item {
                    PathItem::MoveTo(p) => path_builder.move_to(p.x.to_f32(), p.y.to_f32()),
                    PathItem::LineTo(p) => path_builder.line_to(p.x.to_f32(), p.y.to_f32()),
                    PathItem::CubicTo(p1, p2, p3) => path_builder.cubic_to(
                        p1.x.to_f32(),
                        p1.y.to_f32(),
                        p2.x.to_f32(),
                        p2.y.to_f32(),
                        p3.x.to_f32(),
                        p3.y.to_f32()
                    ),
                    PathItem::ClosePath => path_builder.close()
                }
            }
        }
    }

    let path = path_builder.finish().unwrap();

    if let Some(paint) = &shape.fill {
        let (paint, opacity) = convert_paint(paint);

        let fill = Fill {
            paint,
            rule: convert_fill_rule(shape.fill_rule),
            opacity: NormalizedF32::new(opacity as f32 / 255.0).unwrap(),
        };
        surface.fill_path(&path, fill);
    }

    if let Some(stroke) = &shape.stroke {
        let stroke = convert_fixed_stroke(stroke);

        surface.stroke_path(&path, stroke);
    }
}

pub fn handle_frame(frame: &Frame, surface: &mut Surface, context: &mut ExportContext) {
    for (point, item) in frame.items() {
        surface.push_transform(&Transform::from_translate(point.x.to_f32(), point.y.to_f32()));

        match item {
            FrameItem::Group(g) => handle_group(g, surface, context),
            FrameItem::Text(t) => handle_text(t, surface, context),
            FrameItem::Shape(s, _) => {}
            FrameItem::Image(image, size, _) => handle_image(image, size, surface, context),
            FrameItem::Link(_, _) => {}
            FrameItem::Tag(_) => {}
        }

        surface.pop();
    }
}

fn convert_fill_rule(fill_rule: FillRule) -> krilla::path::FillRule {
    match fill_rule {
        FillRule::NonZero => krilla::path::FillRule::NonZero,
        FillRule::EvenOdd => krilla::path::FillRule::EvenOdd
    }
}

fn convert_fixed_stroke(stroke: &FixedStroke) -> Stroke<krilla::color::rgb::Rgb> {
    let (paint, opacity) = convert_paint(&stroke.paint);
    Stroke {
        paint,
        width: stroke.thickness.to_f32(),
        miter_limit: stroke.miter_limit.get() as f32,
        line_join: convert_linejoin(stroke.join),
        line_cap: convert_linecap(stroke.cap),
        opacity: NormalizedF32::new(opacity as f32 / 255.0).unwrap(),
        ..Default::default()
    }
}

fn convert_linecap(l: LineCap) -> krilla::path::LineCap {
    match l {
        LineCap::Butt => krilla::path::LineCap::Butt,
        LineCap::Round => krilla::path::LineCap::Round,
        LineCap::Square => krilla::path::LineCap::Square
    }
}

fn convert_linejoin(l: LineJoin) -> krilla::path::LineJoin {
    match l {
        LineJoin::Miter => krilla::path::LineJoin::Miter,
        LineJoin::Round => krilla::path::LineJoin::Round,
        LineJoin::Bevel => krilla::path::LineJoin::Bevel
    }
}

fn convert_transform(t: crate::Transform) -> krilla::geom::Transform {
    Transform::from_row(t.sx.get() as f32, t.ky.get() as f32, t.kx.get() as f32, t.sy.get() as f32, t.tx.to_f32(), t.ty.to_f32())
}

fn convert_paint(paint: &Paint) -> (krilla::paint::Paint<krilla::color::rgb::Rgb>, u8) {
    match paint {
        Paint::Solid(c) => {
            let components = c.to_space(ColorSpace::Srgb).to_vec4_u8();
            (krilla::paint::Paint::Color(rgb::Color::new(components[0], components[1], components[2])), components[3])
        }
        Paint::Gradient(g) => {
            (krilla::paint::Paint::Color(rgb::Color::black()), 255)
        }
        Paint::Pattern(_) => {
            (krilla::paint::Paint::Color(rgb::Color::black()), 255)
        }
    }
}

