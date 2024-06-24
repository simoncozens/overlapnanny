mod bezpen;
mod wonkiness;
use bezpen::Paths;
use read_fonts::{tables::glyf::Glyph, TableProvider};
use std::{collections::BTreeSet, path::PathBuf};

use clap::Parser;
use skrifa::{
    instance::{LocationRef, Size},
    outline::DrawSettings,
    setting::VariationSetting,
    FontRef, GlyphId, MetadataProvider, OutlineGlyphCollection,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[clap(long = "tolerance", default_value_t = 0.25, value_parser)]
    /// The tolerance for wonkiness
    tolerance: f32,

    /// Glyphs to compare
    #[clap(long = "glyphset")]
    glyphset: Option<String>,

    /// The font file to compare
    font: PathBuf,
}

fn gid_to_name<'a>(font: &impl TableProvider<'a>, gid: GlyphId) -> String {
    if let Ok(Some(name)) = font
        .post()
        .map(|post| post.glyph_name(gid).map(|x| x.to_string()))
    {
        name
    } else {
        format!("gid{:}", gid.to_u16())
    }
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();

    let font_binary = std::fs::read(cli.font).expect("Couldn't open file");
    let fontref = FontRef::new(&font_binary).expect("Couldn't parse font");
    let instances = fontref.named_instances();
    let glyphs_to_check: BTreeSet<String> = if let Some(glyphset) = cli.glyphset {
        glyphset
            .split_ascii_whitespace()
            .map(|x| x.to_string())
            .collect()
    } else {
        BTreeSet::new()
    };
    if instances.is_empty() {
        test_font(
            &fontref,
            LocationRef::default(),
            &glyphs_to_check,
            cli.tolerance,
        );
        std::process::exit(0);
    }

    for instance in instances.iter() {
        let user_coords = instance.user_coords();
        let location = instance.location();
        let mut userlocation = String::new();
        for (a, v) in fontref.axes().iter().zip(user_coords) {
            userlocation.push_str(&format!("{}={} ", a.tag(), v));
        }
        println!(
            "Testing instance {:} ({})",
            fontref
                .localized_strings(instance.subfamily_name_id())
                .english_or_first()
                .map(|x| x.to_string())
                .unwrap_or("Unnamed".to_string()),
            userlocation
        );
        test_font(
            &fontref,
            (&location).into(),
            &glyphs_to_check,
            cli.tolerance,
        );
    }
}
fn test_font(
    fontref: &FontRef,
    location: LocationRef,
    glyphs_to_check: &BTreeSet<String>,
    tolerance: f32,
) {
    let outlines = fontref.outline_glyphs();
    let glyphcount = fontref
        .maxp()
        .map(|maxp| maxp.num_glyphs())
        .unwrap_or_default();
    for glyphid in 0..glyphcount {
        let glyphid = GlyphId::new(glyphid);
        let glyphname = gid_to_name(fontref, glyphid);
        if glyphs_to_check.len() > 0 && !glyphs_to_check.contains(&glyphname) {
            continue;
        }
        let glyph = fontref
            .loca(None)
            .unwrap()
            .get_glyf(glyphid, &fontref.glyf().unwrap())
            .expect("Couldn't read a glyph");
        if matches!(glyph, Some(Glyph::Composite(_))) {
            continue;
        }
        let settings = DrawSettings::unhinted(Size::unscaled(), location);
        let comparison = compare_glyph(&outlines, settings, glyphid, tolerance);
        if comparison > 0.0 && comparison < 1000.0 {
            println!(
                " Wonkiness increased by {:.2}% in glyph {}",
                comparison, glyphname
            );
        }
    }
}

fn compare_glyph(
    outlines: &OutlineGlyphCollection,
    settings: DrawSettings,
    glyph_id: GlyphId,
    tolerance: f32,
) -> f32 {
    let glyph = outlines.get(glyph_id).unwrap();

    let mut paths = Paths::default();
    glyph
        .draw(settings, &mut paths)
        .expect("Couldn't draw glyph");
    let total_wonkiness_before = paths.wonkiness();
    // println!("Total wonkiness before: {}", total_wonkiness_before);
    let total_wonkiness_after = paths.remove_overlaps().wonkiness();
    // println!("Total wonkiness after: {}", total_wonkiness_after);
    if total_wonkiness_after > (total_wonkiness_before) * (1.0 + tolerance) {
        (total_wonkiness_after / total_wonkiness_before - 1.0) * 100.0
    } else {
        0.0
    }
}
