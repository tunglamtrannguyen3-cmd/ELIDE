// src/colors.rs
use crossterm::style::{Attribute, Color};

// ============================================================
// THEMES
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    Serenity,
    SpringAtTheShrine,
    TokyoNight,
    GraveyardOfTheFirefly,
    DeepOcean,
    MagicForest,
    Swampy,
    HarvestTime,
    Nord,
    CatppuccinMocha,
}

impl Theme {
    /// Safely parse theme from string input (used by command palette)
    pub fn from_input(input: &str) -> Self {
        match input.trim().to_lowercase().as_str() {
            "shrine" | "spring" => Theme::SpringAtTheShrine,
            "tokyo" | "tokyonight" => Theme::TokyoNight,
            "firefly" | "graveyard" => Theme::GraveyardOfTheFirefly,
            "ocean" | "deepocean" => Theme::DeepOcean,
            "forest" | "magicforest" => Theme::MagicForest,
            "swamp" | "swampy" => Theme::Swampy,
            "harvest" | "harvesttime" => Theme::HarvestTime,
            "nord" => Theme::Nord,
            "catppuccin" | "mocha" => Theme::CatppuccinMocha,
            _ => Theme::Serenity,
        }
    }
}

// ============================================================
// THEME PALETTE STRUCTURE
// ============================================================

#[derive(Debug, Clone, Copy)]
pub struct ThemePalette {
    pub bg_main: Color,
    pub bg_panel: Color,
    pub type_color: Color,
    pub function: Color,
    pub text: Color,
    pub comment: Color,
    pub border: Color,
    pub accent: Color,
    pub success: Color,
    pub error: Color,
    pub warning: Color,
    pub hint: Color,
}

// ============================================================
// SERENITY — Calm, comfortable, muted pastels
// ============================================================

pub const SERENITY: ThemePalette = ThemePalette {
    bg_main: Color::Rgb { r: 30, g: 30, b: 46 },
    bg_panel: Color::Rgb { r: 24, g: 24, b: 37 },
    type_color: Color::Rgb { r: 232, g: 207, b: 161 },
    function: Color::Rgb { r: 154, g: 188, b: 235 },
    text: Color::Rgb { r: 197, g: 206, b: 229 },
    comment: Color::Rgb { r: 148, g: 155, b: 181 },
    border: Color::Rgb { r: 69, g: 72, b: 94 },
    accent: Color::Rgb { r: 163, g: 185, b: 217 },
    success: Color::Rgb { r: 166, g: 214, b: 174 },
    error: Color::Rgb { r: 230, g: 160, b: 181 },
    warning: Color::Rgb { r: 232, g: 207, b: 161 },
    hint: Color::Rgb { r: 146, g: 212, b: 208 },
};

// ============================================================
// SPRING AT THE SHRINE — Sakura, sunlight, shrine vermilion
// ============================================================

pub const SPRING_AT_THE_SHRINE: ThemePalette = ThemePalette {
    bg_main: Color::Rgb { r: 36, g: 42, b: 36 },
    bg_panel: Color::Rgb { r: 28, g: 34, b: 29 },
    type_color: Color::Rgb { r: 238, g: 199, b: 139 },
    function: Color::Rgb { r: 157, g: 195, b: 180 },
    text: Color::Rgb { r: 225, g: 220, b: 202 },
    comment: Color::Rgb { r: 145, g: 161, b: 140 },
    border: Color::Rgb { r: 79, g: 99, b: 78 },
    accent: Color::Rgb { r: 207, g: 111, b: 133 },
    success: Color::Rgb { r: 155, g: 205, b: 143 },
    error: Color::Rgb { r: 230, g: 119, b: 140 },
    warning: Color::Rgb { r: 244, g: 194, b: 112 },
    hint: Color::Rgb { r: 130, g: 205, b: 191 },
};

// ============================================================
// TOKYO NIGHT — Neon-lit city after dark
// ============================================================

pub const TOKYO_NIGHT: ThemePalette = ThemePalette {
    bg_main: Color::Rgb { r: 26, g: 27, b: 38 },
    bg_panel: Color::Rgb { r: 22, g: 22, b: 30 },
    type_color: Color::Rgb { r: 224, g: 175, b: 104 },
    function: Color::Rgb { r: 122, g: 162, b: 247 },
    text: Color::Rgb { r: 192, g: 202, b: 245 },
    comment: Color::Rgb { r: 86, g: 95, b: 137 },
    border: Color::Rgb { r: 65, g: 72, b: 104 },
    accent: Color::Rgb { r: 125, g: 207, b: 195 },
    success: Color::Rgb { r: 158, g: 206, b: 106 },
    error: Color::Rgb { r: 247, g: 118, b: 142 },
    warning: Color::Rgb { r: 224, g: 175, b: 104 },
    hint: Color::Rgb { r: 125, g: 207, b: 195 },
};

// ============================================================
// GRAVEYARD OF THE FIREFLY — Fragile, temporary peace
// ============================================================

pub const GRAVEYARD_OF_THE_FIREFLY: ThemePalette = ThemePalette {
    bg_main: Color::Rgb { r: 23, g: 25, b: 35 },
    bg_panel: Color::Rgb { r: 32, g: 35, b: 43 },
    type_color: Color::Rgb { r: 216, g: 185, b: 133 },
    function: Color::Rgb { r: 158, g: 173, b: 197 },
    text: Color::Rgb { r: 202, g: 203, b: 194 },
    comment: Color::Rgb { r: 133, g: 135, b: 151 },
    border: Color::Rgb { r: 65, g: 70, b: 79 },
    accent: Color::Rgb { r: 229, g: 190, b: 130 },
    success: Color::Rgb { r: 166, g: 185, b: 154 },
    error: Color::Rgb { r: 201, g: 137, b: 145 },
    warning: Color::Rgb { r: 216, g: 185, b: 133 },
    hint: Color::Rgb { r: 145, g: 177, b: 173 },
};

// ============================================================
// DEEP OCEAN — Quiet, cold, deep blue
// ============================================================

pub const DEEP_OCEAN: ThemePalette = ThemePalette {
    bg_main: Color::Rgb { r: 16, g: 30, b: 43 },
    bg_panel: Color::Rgb { r: 12, g: 23, b: 35 },
    type_color: Color::Rgb { r: 166, g: 196, b: 216 },
    function: Color::Rgb { r: 85, g: 166, b: 209 },
    text: Color::Rgb { r: 192, g: 213, b: 226 },
    comment: Color::Rgb { r: 102, g: 137, b: 157 },
    border: Color::Rgb { r: 48, g: 78, b: 96 },
    accent: Color::Rgb { r: 112, g: 165, b: 198 },
    success: Color::Rgb { r: 121, g: 198, b: 176 },
    error: Color::Rgb { r: 220, g: 132, b: 145 },
    warning: Color::Rgb { r: 213, g: 184, b: 125 },
    hint: Color::Rgb { r: 106, g: 190, b: 207 },
};

// ============================================================
// MAGIC FOREST — Fresh, lush, relaxing nature
// ============================================================

pub const MAGIC_FOREST: ThemePalette = ThemePalette {
    bg_main: Color::Rgb { r: 23, g: 37, b: 29 },
    bg_panel: Color::Rgb { r: 29, g: 46, b: 35 },
    type_color: Color::Rgb { r: 213, g: 201, b: 155 },
    function: Color::Rgb { r: 145, g: 188, b: 168 },
    text: Color::Rgb { r: 208, g: 221, b: 201 },
    comment: Color::Rgb { r: 130, g: 151, b: 131 },
    border: Color::Rgb { r: 64, g: 91, b: 70 },
    accent: Color::Rgb { r: 160, g: 195, b: 139 },
    success: Color::Rgb { r: 159, g: 198, b: 155 },
    error: Color::Rgb { r: 212, g: 135, b: 139 },
    warning: Color::Rgb { r: 213, g: 201, b: 155 },
    hint: Color::Rgb { r: 143, g: 192, b: 166 },
};

// ============================================================
// SWAMPY — Still water, moss, reeds, earthy greens
// ============================================================

pub const SWAMPY: ThemePalette = ThemePalette {
    bg_main: Color::Rgb { r: 32, g: 40, b: 32 },
    bg_panel: Color::Rgb { r: 41, g: 50, b: 41 },
    type_color: Color::Rgb { r: 198, g: 185, b: 148 },
    function: Color::Rgb { r: 160, g: 182, b: 164 },
    text: Color::Rgb { r: 208, g: 212, b: 189 },
    comment: Color::Rgb { r: 141, g: 152, b: 133 },
    border: Color::Rgb { r: 74, g: 89, b: 71 },
    accent: Color::Rgb { r: 155, g: 174, b: 133 },
    success: Color::Rgb { r: 158, g: 178, b: 138 },
    error: Color::Rgb { r: 199, g: 128, b: 120 },
    warning: Color::Rgb { r: 218, g: 184, b: 105 },
    hint: Color::Rgb { r: 128, g: 178, b: 158 },
};

// ============================================================
// HARVEST TIME — Warm, expressive autumn colors
// ============================================================

pub const HARVEST_TIME: ThemePalette = ThemePalette {
    bg_main: Color::Rgb { r: 41, g: 35, b: 31 },
    bg_panel: Color::Rgb { r: 52, g: 42, b: 35 },
    type_color: Color::Rgb { r: 232, g: 200, b: 121 },
    function: Color::Rgb { r: 214, g: 166, b: 200 },
    text: Color::Rgb { r: 240, g: 223, b: 196 },
    comment: Color::Rgb { r: 174, g: 155, b: 131 },
    border: Color::Rgb { r: 102, g: 80, b: 59 },
    accent: Color::Rgb { r: 214, g: 166, b: 200 },
    success: Color::Rgb { r: 169, g: 189, b: 130 },
    error: Color::Rgb { r: 217, g: 130, b: 120 },
    warning: Color::Rgb { r: 232, g: 200, b: 121 },
    hint: Color::Rgb { r: 131, g: 189, b: 176 },
};

// ============================================================
// NORD — Arctic, north-bluish clean aesthetic
// ============================================================

pub const NORD: ThemePalette = ThemePalette {
    bg_main: Color::Rgb { r: 46, g: 52, b: 64 },
    bg_panel: Color::Rgb { r: 59, g: 66, b: 82 },
    type_color: Color::Rgb { r: 143, g: 188, b: 187 },
    function: Color::Rgb { r: 129, g: 161, b: 193 },
    text: Color::Rgb { r: 216, g: 222, b: 233 },
    comment: Color::Rgb { r: 76, g: 86, b: 106 },
    border: Color::Rgb { r: 67, g: 76, b: 94 },
    accent: Color::Rgb { r: 136, g: 192, b: 208 },
    success: Color::Rgb { r: 163, g: 190, b: 140 },
    error: Color::Rgb { r: 191, g: 97, b: 106 },
    warning: Color::Rgb { r: 235, g: 203, b: 139 },
    hint: Color::Rgb { r: 143, g: 188, b: 187 },
};

// ============================================================
// CATPPUCCIN MOCHA — Soothing pastel dark theme
// ============================================================

pub const CATPPUCCIN_MOCHA: ThemePalette = ThemePalette {
    bg_main: Color::Rgb { r: 30, g: 30, b: 46 },
    bg_panel: Color::Rgb { r: 24, g: 24, b: 37 },
    type_color: Color::Rgb { r: 249, g: 226, b: 175 },
    function: Color::Rgb { r: 137, g: 180, b: 250 },
    text: Color::Rgb { r: 205, g: 214, b: 244 },
    comment: Color::Rgb { r: 108, g: 112, b: 134 },
    border: Color::Rgb { r: 88, g: 91, b: 112 },
    accent: Color::Rgb { r: 245, g: 194, b: 231 },
    success: Color::Rgb { r: 166, g: 227, b: 161 },
    error: Color::Rgb { r: 243, g: 139, b: 168 },
    warning: Color::Rgb { r: 250, g: 179, b: 135 },
    hint: Color::Rgb { r: 148, g: 226, b: 213 },
};

// ============================================================
// PALETTE LOOKUP
// ============================================================

pub fn palette_for(theme: Theme) -> &'static ThemePalette {
    match theme {
        Theme::Serenity => &SERENITY,
        Theme::SpringAtTheShrine => &SPRING_AT_THE_SHRINE,
        Theme::TokyoNight => &TOKYO_NIGHT,
        Theme::GraveyardOfTheFirefly => &GRAVEYARD_OF_THE_FIREFLY,
        Theme::DeepOcean => &DEEP_OCEAN,
        Theme::MagicForest => &MAGIC_FOREST,
        Theme::Swampy => &SWAMPY,
        Theme::HarvestTime => &HARVEST_TIME,
        Theme::Nord => &NORD,
        Theme::CatppuccinMocha => &CATPPUCCIN_MOCHA,
    }
}

// ============================================================
// STATUS SYSTEM
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Modified,
    Saved,
    Error,
}

pub fn color_for_status_in(theme: Theme, status: Status) -> Color {
    let palette = palette_for(theme);

    match status {
        Status::Saved => palette.success,
        Status::Modified => palette.warning,
        Status::Error => palette.error,
    }
}

pub fn attribute_for_status(status: Status) -> Attribute {
    match status {
        Status::Modified | Status::Error => Attribute::Bold,
        _ => Attribute::Reset,
    }
}

// ============================================================
// UNIT TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_from_input() {
        assert_eq!(Theme::from_input("shrine"), Theme::SpringAtTheShrine);
        assert_eq!(Theme::from_input("SPRING"), Theme::SpringAtTheShrine);
        assert_eq!(Theme::from_input("tokyonight"), Theme::TokyoNight);
        assert_eq!(Theme::from_input("firefly"), Theme::GraveyardOfTheFirefly);
        assert_eq!(Theme::from_input("ocean"), Theme::DeepOcean);
        assert_eq!(Theme::from_input("forest"), Theme::MagicForest);
        assert_eq!(Theme::from_input("swamp"), Theme::Swampy);
        assert_eq!(Theme::from_input("harvest"), Theme::HarvestTime);
        assert_eq!(Theme::from_input("nord"), Theme::Nord);
        assert_eq!(Theme::from_input("catppuccin"), Theme::CatppuccinMocha);
        assert_eq!(Theme::from_input("unknown_theme"), Theme::Serenity);
    }

    #[test]
    fn test_palette_lookup() {
        let themes = [
            Theme::Serenity,
            Theme::SpringAtTheShrine,
            Theme::TokyoNight,
            Theme::GraveyardOfTheFirefly,
            Theme::DeepOcean,
            Theme::MagicForest,
            Theme::Swampy,
            Theme::HarvestTime,
            Theme::Nord,
            Theme::CatppuccinMocha,
        ];

        for theme in themes {
            let palette = palette_for(theme);
            let _ = palette.bg_main;
            let _ = palette.success;
        }
    }

    #[test]
    fn test_status_systems() {
        let color = color_for_status_in(Theme::Serenity, Status::Saved);
        assert_eq!(color, SERENITY.success);

        let mod_color = color_for_status_in(Theme::Serenity, Status::Modified);
        assert_eq!(mod_color, SERENITY.warning);

        let err_color = color_for_status_in(Theme::Serenity, Status::Error);
        assert_eq!(err_color, SERENITY.error);

        assert_eq!(attribute_for_status(Status::Modified), Attribute::Bold);
        assert_eq!(attribute_for_status(Status::Error), Attribute::Bold);
        assert_eq!(attribute_for_status(Status::Saved), Attribute::Reset);
    }
    // ==========================================
// EXPANDED UNIT TESTS
// ==========================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_from_input() {
        assert_eq!(Theme::from_input("shrine"), Theme::SpringAtTheShrine);
        assert_eq!(Theme::from_input("SPRING"), Theme::SpringAtTheShrine);
        assert_eq!(Theme::from_input("tokyonight"), Theme::TokyoNight);
        assert_eq!(Theme::from_input("firefly"), Theme::GraveyardOfTheFirefly);
        assert_eq!(Theme::from_input("ocean"), Theme::DeepOcean);
        assert_eq!(Theme::from_input("forest"), Theme::MagicForest);
        assert_eq!(Theme::from_input("swamp"), Theme::Swampy);
        assert_eq!(Theme::from_input("harvest"), Theme::HarvestTime);
        assert_eq!(Theme::from_input("nord"), Theme::Nord);
        assert_eq!(Theme::from_input("catppuccin"), Theme::CatppuccinMocha);
        assert_eq!(Theme::from_input("unknown_theme"), Theme::Serenity);
    }

    #[test]
    fn test_theme_from_input_edge_cases() {
        assert_eq!(Theme::from_input("   shrine   "), Theme::SpringAtTheShrine);
        assert_eq!(Theme::from_input("ToKyO"), Theme::TokyoNight);
        assert_eq!(Theme::from_input(""), Theme::Serenity);
        assert_eq!(Theme::from_input("random_nonsense"), Theme::Serenity);
        assert_eq!(Theme::from_input("mocha"), Theme::CatppuccinMocha);
    }

    #[test]
    fn test_theme_default() {
        assert_eq!(Theme::default(), Theme::Serenity);
    }

    #[test]
    fn test_palette_lookup() {
        let themes = [
            Theme::Serenity,
            Theme::SpringAtTheShrine,
            Theme::TokyoNight,
            Theme::GraveyardOfTheFirefly,
            Theme::DeepOcean,
            Theme::MagicForest,
            Theme::Swampy,
            Theme::HarvestTime,
            Theme::Nord,
            Theme::CatppuccinMocha,
        ];

        for theme in themes {
            let palette = palette_for(theme);
            let _ = palette.bg_main;
            let _ = palette.success;
        }
    }

    #[test]
    fn test_status_systems() {
        let color = color_for_status_in(Theme::Serenity, Status::Saved);
        assert_eq!(color, SERENITY.success);

        let mod_color = color_for_status_in(Theme::Serenity, Status::Modified);
        assert_eq!(mod_color, SERENITY.warning);

        let err_color = color_for_status_in(Theme::Serenity, Status::Error);
        assert_eq!(err_color, SERENITY.error);

        assert_eq!(attribute_for_status(Status::Modified), Attribute::Bold);
        assert_eq!(attribute_for_status(Status::Error), Attribute::Bold);
        assert_eq!(attribute_for_status(Status::Saved), Attribute::Reset);
    }

    #[test]
    fn test_color_for_status_all_themes() {
        let themes = [
            Theme::Serenity,
            Theme::SpringAtTheShrine,
            Theme::TokyoNight,
            Theme::GraveyardOfTheFirefly,
            Theme::DeepOcean,
            Theme::MagicForest,
            Theme::Swampy,
            Theme::HarvestTime,
            Theme::Nord,
            Theme::CatppuccinMocha,
        ];

        for theme in themes {
            let saved = color_for_status_in(theme, Status::Saved);
            let modified = color_for_status_in(theme, Status::Modified);
            let error = color_for_status_in(theme, Status::Error);

            let palette = palette_for(theme);
            assert_eq!(saved, palette.success);
            assert_eq!(modified, palette.warning);
            assert_eq!(error, palette.error);
        }
    }
}
}