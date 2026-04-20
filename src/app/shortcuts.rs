#[derive(Clone, Copy, Debug)]
pub struct Shortcut {
    pub category: &'static str,
    pub key: &'static str,
    pub desc: &'static str,
}

pub fn get_all_shortcuts() -> Vec<Shortcut> {
    vec![
        // Editor Keys
        Shortcut { category: "Editor Keys", key: "/", desc: "Command bar" },
        Shortcut { category: "Editor Keys", key: "F1", desc: "Cheat sheet" },
        Shortcut { category: "Editor Keys", key: "Esc", desc: "Back to editor" },
        Shortcut { category: "Editor Keys", key: "^P", desc: "Settings pane" },
        Shortcut { category: "Editor Keys", key: "^E", desc: "Export pane" },
        Shortcut { category: "Editor Keys", key: "^H", desc: "Scene Navigator" },
        Shortcut { category: "Editor Keys", key: "^L", desc: "Ensemble" },
        Shortcut { category: "Editor Keys", key: "Tab", desc: "Autocomplete" },

        // File Commands
        Shortcut { category: "File Commands", key: "/w", desc: "Save" },
        Shortcut { category: "File Commands", key: "/ww", desc: "Save As" },
        Shortcut { category: "File Commands", key: "/o [path]", desc: "Open file" },
        Shortcut { category: "File Commands", key: "/new", desc: "New file" },
        Shortcut { category: "File Commands", key: "/bn / /bp", desc: "Next / prev file" },
        Shortcut { category: "File Commands", key: "/q", desc: "Close file" },
        Shortcut { category: "File Commands", key: "/q!", desc: "Force close" },
        Shortcut { category: "File Commands", key: "/wq", desc: "Save & close" },
        Shortcut { category: "File Commands", key: "/ex", desc: "Exit app" },
        Shortcut { category: "File Commands", key: "/home", desc: "Start screen" },

        // Sprint & Tools
        Shortcut { category: "Sprint & Tools", key: "/sprint [m]", desc: "Start sprint" },
        Shortcut { category: "Sprint & Tools", key: "/cancelsprint", desc: "Cancel sprint" },
        Shortcut { category: "Sprint & Tools", key: "/sprintstat", desc: "Sprint history" },
        Shortcut { category: "Sprint & Tools", key: "/snap", desc: "Snapshots" },
        Shortcut { category: "Sprint & Tools", key: "/xray", desc: "Visual analysis" },
        Shortcut { category: "Sprint & Tools", key: "/export", desc: "Export pane" },
        Shortcut { category: "Sprint & Tools", key: "/theme [name]", desc: "Switch theme" },
        Shortcut { category: "Sprint & Tools", key: "/ic", desc: "Index cards" },
        Shortcut { category: "Sprint & Tools", key: "/editor / /ed", desc: "Normal editor" },

        // Selection
        Shortcut { category: "Selection", key: "Shift+Arrow", desc: "Extend selection" },
        Shortcut { category: "Selection", key: "Shift+Home", desc: "Select to start" },
        Shortcut { category: "Selection", key: "Shift+End", desc: "Select to end" },
        Shortcut { category: "Selection", key: "^A", desc: "Select all" },
        Shortcut { category: "Selection", key: "^C", desc: "Copy" },
        Shortcut { category: "Selection", key: "^X", desc: "Cut" },
        Shortcut { category: "Selection", key: "^V", desc: "Paste" },

        // Navigation
        Shortcut { category: "Navigation", key: "/[line]", desc: "Jump to line" },
        Shortcut { category: "Navigation", key: "/s[num]", desc: "Jump to scene" },
        Shortcut { category: "Navigation", key: "/search [q]", desc: "Search text" },
        Shortcut { category: "Navigation", key: "Alt+Up/Dn", desc: "Next/Prev match" },
        Shortcut { category: "Navigation", key: "/ud / /rd", desc: "Undo / Redo" },
        Shortcut { category: "Navigation", key: "/pos", desc: "Cursor position" },
        Shortcut { category: "Navigation", key: "/copy", desc: "Copy clipboard" },
        Shortcut { category: "Navigation", key: "/cut", desc: "Cut clipboard" },
        Shortcut { category: "Navigation", key: "/paste", desc: "Paste clipboard" },
        Shortcut { category: "Navigation", key: "/selectall", desc: "Select all text" },

        // Movement
        Shortcut { category: "Movement", key: "^Left/Right", desc: "Jump by word" },
        Shortcut { category: "Movement", key: "^Backspace", desc: "Delete word \u{2190}" },
        Shortcut { category: "Movement", key: "^Delete", desc: "Delete word \u{2192}" },
        Shortcut { category: "Movement", key: "Home / End", desc: "Line start/end" },
        Shortcut { category: "Movement", key: "PgUp / PgDn", desc: "Scroll page" },
        Shortcut { category: "Movement", key: "^PgUp/PgDn", desc: "Switch files" },

        // Scene & Production
        Shortcut { category: "Scene & Production", key: "/renum", desc: "Renumber scenes" },
        Shortcut { category: "Scene & Production", key: "/clearnum", desc: "Clear numbers" },
        Shortcut { category: "Scene & Production", key: "/injectnum", desc: "Number scene" },
        Shortcut { category: "Scene & Production", key: "/locknum", desc: "Production lock" },
        Shortcut { category: "Scene & Production", key: "/unlocknum", desc: "Unlock numbers" },
        Shortcut { category: "Scene & Production", key: "/addtitle", desc: "Title page" },

        // Settings (/set)
        Shortcut { category: "Settings (/set)", key: "focus", desc: "Zen mode" },
        Shortcut { category: "Settings (/set)", key: "typewriter", desc: "Center cursor" },
        Shortcut { category: "Settings (/set)", key: "markup", desc: "Show/hide markup" },
        Shortcut { category: "Settings (/set)", key: "pagenums", desc: "Page numbers" },
        Shortcut { category: "Settings (/set)", key: "scenenums", desc: "Scene numbers" },
        Shortcut { category: "Settings (/set)", key: "contd", desc: "Auto (CONT'D)" },
        Shortcut { category: "Settings (/set)", key: "autosave", desc: "Auto-save 30s" },
        Shortcut { category: "Settings (/set)", key: "autocomplete", desc: "Suggestions" },
        Shortcut { category: "Settings (/set)", key: "autobreaks", desc: "Smart breaks" },
    ]
}
