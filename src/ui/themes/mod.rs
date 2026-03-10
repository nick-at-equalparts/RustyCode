use ratatui::style::Color;

/// Complete theme definition for the TUI.
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: &'static str,
    pub bg: Color,
    pub fg: Color,
    pub accent: Color,
    pub border: Color,
    pub selection: Color,
    pub user_msg: Color,
    pub assistant_msg: Color,
    pub error: Color,
    pub warning: Color,
    pub success: Color,
    pub muted: Color,
    pub tool_pending: Color,
    pub tool_running: Color,
    pub tool_complete: Color,
    pub tool_error: Color,
}

// ---------------------------------------------------------------------------
// Built-in themes
// ---------------------------------------------------------------------------

static DEFAULT_THEME: Theme = Theme {
    name: "Default",
    bg: Color::Rgb(30, 30, 46),
    fg: Color::Rgb(205, 214, 244),
    accent: Color::Rgb(137, 180, 250),
    border: Color::Rgb(88, 91, 112),
    selection: Color::Rgb(69, 71, 90),
    user_msg: Color::Rgb(166, 227, 161),
    assistant_msg: Color::Rgb(205, 214, 244),
    error: Color::Rgb(243, 139, 168),
    warning: Color::Rgb(249, 226, 175),
    success: Color::Rgb(166, 227, 161),
    muted: Color::Rgb(108, 112, 134),
    tool_pending: Color::Rgb(249, 226, 175),
    tool_running: Color::Rgb(137, 180, 250),
    tool_complete: Color::Rgb(166, 227, 161),
    tool_error: Color::Rgb(243, 139, 168),
};

static CATPPUCCIN_MOCHA: Theme = Theme {
    name: "Catppuccin Mocha",
    bg: Color::Rgb(30, 30, 46),
    fg: Color::Rgb(205, 214, 244),
    accent: Color::Rgb(180, 190, 254),
    border: Color::Rgb(88, 91, 112),
    selection: Color::Rgb(69, 71, 90),
    user_msg: Color::Rgb(148, 226, 213),
    assistant_msg: Color::Rgb(205, 214, 244),
    error: Color::Rgb(243, 139, 168),
    warning: Color::Rgb(249, 226, 175),
    success: Color::Rgb(166, 227, 161),
    muted: Color::Rgb(108, 112, 134),
    tool_pending: Color::Rgb(249, 226, 175),
    tool_running: Color::Rgb(116, 199, 236),
    tool_complete: Color::Rgb(166, 227, 161),
    tool_error: Color::Rgb(243, 139, 168),
};

static DRACULA: Theme = Theme {
    name: "Dracula",
    bg: Color::Rgb(40, 42, 54),
    fg: Color::Rgb(248, 248, 242),
    accent: Color::Rgb(189, 147, 249),
    border: Color::Rgb(68, 71, 90),
    selection: Color::Rgb(68, 71, 90),
    user_msg: Color::Rgb(80, 250, 123),
    assistant_msg: Color::Rgb(248, 248, 242),
    error: Color::Rgb(255, 85, 85),
    warning: Color::Rgb(241, 250, 140),
    success: Color::Rgb(80, 250, 123),
    muted: Color::Rgb(98, 114, 164),
    tool_pending: Color::Rgb(241, 250, 140),
    tool_running: Color::Rgb(139, 233, 253),
    tool_complete: Color::Rgb(80, 250, 123),
    tool_error: Color::Rgb(255, 85, 85),
};

static GRUVBOX: Theme = Theme {
    name: "Gruvbox",
    bg: Color::Rgb(40, 40, 40),
    fg: Color::Rgb(235, 219, 178),
    accent: Color::Rgb(215, 153, 33),
    border: Color::Rgb(80, 73, 69),
    selection: Color::Rgb(60, 56, 54),
    user_msg: Color::Rgb(184, 187, 38),
    assistant_msg: Color::Rgb(235, 219, 178),
    error: Color::Rgb(204, 36, 29),
    warning: Color::Rgb(250, 189, 47),
    success: Color::Rgb(152, 151, 26),
    muted: Color::Rgb(146, 131, 116),
    tool_pending: Color::Rgb(250, 189, 47),
    tool_running: Color::Rgb(69, 133, 136),
    tool_complete: Color::Rgb(152, 151, 26),
    tool_error: Color::Rgb(204, 36, 29),
};

static TOKYO_NIGHT: Theme = Theme {
    name: "Tokyo Night",
    bg: Color::Rgb(26, 27, 38),
    fg: Color::Rgb(192, 202, 245),
    accent: Color::Rgb(122, 162, 247),
    border: Color::Rgb(59, 66, 97),
    selection: Color::Rgb(41, 46, 66),
    user_msg: Color::Rgb(158, 206, 106),
    assistant_msg: Color::Rgb(192, 202, 245),
    error: Color::Rgb(247, 118, 142),
    warning: Color::Rgb(224, 175, 104),
    success: Color::Rgb(158, 206, 106),
    muted: Color::Rgb(84, 93, 134),
    tool_pending: Color::Rgb(224, 175, 104),
    tool_running: Color::Rgb(125, 207, 255),
    tool_complete: Color::Rgb(158, 206, 106),
    tool_error: Color::Rgb(247, 118, 142),
};

static ONE_DARK: Theme = Theme {
    name: "One Dark",
    bg: Color::Rgb(40, 44, 52),
    fg: Color::Rgb(171, 178, 191),
    accent: Color::Rgb(97, 175, 239),
    border: Color::Rgb(76, 82, 99),
    selection: Color::Rgb(62, 68, 81),
    user_msg: Color::Rgb(152, 195, 121),
    assistant_msg: Color::Rgb(171, 178, 191),
    error: Color::Rgb(224, 108, 117),
    warning: Color::Rgb(229, 192, 123),
    success: Color::Rgb(152, 195, 121),
    muted: Color::Rgb(92, 99, 112),
    tool_pending: Color::Rgb(229, 192, 123),
    tool_running: Color::Rgb(86, 182, 194),
    tool_complete: Color::Rgb(152, 195, 121),
    tool_error: Color::Rgb(224, 108, 117),
};

static MONOKAI: Theme = Theme {
    name: "Monokai",
    bg: Color::Rgb(39, 40, 34),
    fg: Color::Rgb(248, 248, 242),
    accent: Color::Rgb(102, 217, 239),
    border: Color::Rgb(73, 72, 62),
    selection: Color::Rgb(62, 61, 50),
    user_msg: Color::Rgb(166, 226, 46),
    assistant_msg: Color::Rgb(248, 248, 242),
    error: Color::Rgb(249, 38, 114),
    warning: Color::Rgb(253, 151, 31),
    success: Color::Rgb(166, 226, 46),
    muted: Color::Rgb(117, 113, 94),
    tool_pending: Color::Rgb(253, 151, 31),
    tool_running: Color::Rgb(102, 217, 239),
    tool_complete: Color::Rgb(166, 226, 46),
    tool_error: Color::Rgb(249, 38, 114),
};

static ALL_THEMES: &[&Theme] = &[
    &DEFAULT_THEME,
    &CATPPUCCIN_MOCHA,
    &DRACULA,
    &GRUVBOX,
    &TOKYO_NIGHT,
    &ONE_DARK,
    &MONOKAI,
];

/// Return a theme by (case-insensitive) name. Falls back to Default.
pub fn get_theme(name: &str) -> &'static Theme {
    let lower = name.to_lowercase();
    for theme in ALL_THEMES {
        if theme.name.to_lowercase() == lower {
            return theme;
        }
    }
    &DEFAULT_THEME
}

/// Return a list of all available theme names.
pub fn list_themes() -> Vec<&'static str> {
    ALL_THEMES.iter().map(|t| t.name).collect()
}
