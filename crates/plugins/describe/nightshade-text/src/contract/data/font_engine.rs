use cosmic_text::{
    Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Stretch, Style, SwashCache, Weight,
};

pub const DEFAULT_FONT_DATA: &[u8] = include_bytes!("fonts/Roboto-Regular.ttf");
pub const DEFAULT_MONO_FONT_DATA: &[u8] = include_bytes!("fonts/JetBrainsMono-Regular.ttf");
pub const ICONS_MATERIAL_FONT_DATA: &[u8] = include_bytes!("fonts/MaterialIcons-Regular.ttf");
pub const ICONS_LUCIDE_FONT_DATA: &[u8] = include_bytes!("fonts/Lucide.ttf");

use crate::contract::data::text_data::FontKind;

pub struct FontEngine {
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
    default_family: String,
    monospace_family: String,
    icons_material_family: String,
    icons_lucide_family: String,
}

impl FontEngine {
    pub fn new() -> Self {
        let mut db = cosmic_text::fontdb::Database::new();
        db.load_font_data(DEFAULT_FONT_DATA.to_vec());
        db.load_font_data(DEFAULT_MONO_FONT_DATA.to_vec());
        db.load_font_data(ICONS_MATERIAL_FONT_DATA.to_vec());
        db.load_font_data(ICONS_LUCIDE_FONT_DATA.to_vec());

        let mut default_family = String::from("Roboto");
        let mut monospace_family = String::from("JetBrains Mono");
        let mut icons_material_family = String::from("Material Icons");
        let mut icons_lucide_family = String::from("lucide");

        for face in db.faces() {
            if let Some((family, _)) = face.families.first() {
                let lower = family.to_lowercase();
                if lower.contains("material icons") || lower.contains("material symbols") {
                    icons_material_family = family.clone();
                } else if lower.contains("lucide") {
                    icons_lucide_family = family.clone();
                } else if face.monospaced {
                    monospace_family = family.clone();
                } else {
                    default_family = family.clone();
                }
            }
        }

        db.set_sans_serif_family(default_family.clone());
        db.set_monospace_family(monospace_family.clone());

        let font_system = FontSystem::new_with_locale_and_db("en-US".to_string(), db);

        Self {
            font_system,
            swash_cache: SwashCache::new(),
            default_family,
            monospace_family,
            icons_material_family,
            icons_lucide_family,
        }
    }

    pub fn load_font(&mut self, data: Vec<u8>) {
        self.font_system.db_mut().load_font_data(data);
    }

    fn family_name(&self, font_kind: FontKind) -> &str {
        match font_kind {
            FontKind::Default => &self.default_family,
            FontKind::Mono => &self.monospace_family,
            FontKind::IconsMaterial => &self.icons_material_family,
            FontKind::IconsLucide => &self.icons_lucide_family,
        }
    }

    pub fn shape_buffer(
        &mut self,
        text: &str,
        font_size: f32,
        line_height: f32,
        max_width: Option<f32>,
        font_kind: FontKind,
    ) -> Buffer {
        let metrics = Metrics::new(font_size, line_height);
        let mut buffer = Buffer::new(&mut self.font_system, metrics);
        buffer.set_size(max_width, None);
        let family_name = self.family_name(font_kind).to_string();
        let attrs = Attrs::new()
            .family(Family::Name(&family_name))
            .weight(Weight::NORMAL)
            .style(Style::Normal)
            .stretch(Stretch::Normal);
        buffer.set_text(text, &attrs, Shaping::Advanced, None);
        buffer.shape_until_scroll(&mut self.font_system, true);
        buffer
    }
}

impl Default for FontEngine {
    fn default() -> Self {
        Self::new()
    }
}
