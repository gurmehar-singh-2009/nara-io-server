pub type Color = [f32; 4];

pub struct DiepTheme {
    pub background: Color,
    pub border: Color,
    pub border_alpha: f32,

    pub grid: Color,
    pub grid_alpha: f32,

    pub tank_body: Color,
    pub tank_outline: Color,
    pub barrel: Color,
    pub bullet: Color,

    pub team_blue: Color,
    pub team_red: Color,
    pub team_purple: Color,
    pub team_green: Color,

    pub health_bar_background: Color,
    pub health_bar_foreground: Color,

    pub xp_bar_fill: Color,
    pub score_bar_fill: Color,
    pub bar_background: Color,

    pub minimap_background: Color,
    pub minimap_border: Color,

    pub score_text: Color,

    pub square: Color,
    pub triangle: Color,
    pub pentagon: Color,
    pub crashers: Color,

    pub arena_closer: Color,
    pub maze_walls: Color,
    pub fallen_boss: Color,
}

impl DiepTheme {
    pub const fn dark() -> Self {
        Self {
            background: [0.020, 0.020, 0.020, 1.0],

            border: [0.0, 0.0, 0.0, 1.0],
            border_alpha: 0.35,

            grid: [0.078, 0.078, 0.078, 1.0],
            grid_alpha: 1.0,

            tank_body: [0.0, 0.698, 0.882, 1.0],
            tank_outline: [0.0, 0.220, 0.275, 1.0],
            barrel: [0.329, 0.329, 0.329, 1.0],
            bullet: [0.0, 0.698, 0.882, 1.0],

            team_blue: [0.0, 0.698, 0.882, 1.0], 
            team_red: [0.988, 0.463, 0.467, 1.0],
            team_purple: [0.945, 0.467, 0.867, 1.0],
            team_green: [0.0, 0.882, 0.431, 1.0],

            health_bar_background: [0.020, 0.020, 0.020, 0.80],
            health_bar_foreground: [0.522, 0.890, 0.490, 1.0],

            xp_bar_fill: [1.0, 0.871, 0.263, 1.0],
            score_bar_fill: [0.0, 0.882, 0.431, 1.0],

            bar_background: [0.020, 0.020, 0.020, 0.85],

            minimap_background: [0.020, 0.020, 0.020, 0.80],
            minimap_border: [0.078, 0.078, 0.078, 1.0],

            score_text: [0.900, 0.920, 0.950, 1.0],

            square: [1.0, 0.910, 0.412, 1.0],
            triangle: [0.988, 0.463, 0.467, 1.0],
            pentagon: [0.463, 0.553, 1.0, 1.0],
            crashers: [0.945, 0.467, 0.867, 1.0],

            arena_closer: [1.0, 0.910, 0.412, 1.0],

            maze_walls: [0.078, 0.078, 0.078, 1.0],
            fallen_boss: [0.450, 0.480, 0.520, 1.0],
        }
    }

    pub const fn light() -> Self {
        Self {
            background: [0.804, 0.804, 0.804, 1.0],
            border: [0.0, 0.0, 0.0, 1.0],
            border_alpha: 0.10,

            grid: [0.0, 0.0, 0.0, 1.0],
            grid_alpha: 0.10,

            tank_body: [0.0, 0.698, 0.882, 1.0],
            tank_outline: [0.333, 0.333, 0.333, 1.0],
            barrel: [0.600, 0.600, 0.600, 1.0],
            bullet: [0.0, 0.698, 0.882, 1.0],

            team_blue: [0.0, 0.698, 0.882, 1.0],
            team_red: [0.945, 0.306, 0.329, 1.0],
            team_purple: [0.749, 0.498, 0.961, 1.0],
            team_green: [0.0, 0.882, 0.431, 1.0],

            health_bar_background: [0.333, 0.333, 0.333, 1.0],
            health_bar_foreground: [0.522, 0.890, 0.490, 1.0],

            xp_bar_fill: [1.0, 0.871, 0.263, 1.0],
            score_bar_fill: [0.263, 1.0, 0.569, 1.0],

            bar_background: [0.0, 0.0, 0.0, 1.0],

            minimap_background: [0.804, 0.804, 0.804, 1.0],
            minimap_border: [0.333, 0.333, 0.333, 1.0],

            score_text: [0.0, 0.0, 0.0, 1.0],

            square: [1.0, 0.910, 0.412, 1.0],
            triangle: [0.988, 0.465, 0.467, 1.0],
            pentagon: [0.463, 0.553, 0.988, 1.0],
            crashers: [0.945, 0.467, 0.867, 1.0],

            arena_closer: [1.0, 0.910, 0.412, 1.0],

            maze_walls: [0.733, 0.733, 0.733, 1.0],
            fallen_boss: [0.753, 0.753, 0.753, 1.0],
        }
    }
}

pub const DEFAULT_THEME: DiepTheme = DiepTheme::dark();
pub const DARK_THEME: DiepTheme = DiepTheme::dark();
