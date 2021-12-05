#[derive(Clone, Copy)]
pub struct Theme {
    pub name: &'static str,
    pub hour: &'static str,
    pub minute: &'static str,
    pub second: &'static str,
    pub clock_face: &'static str,
}

pub const THEMES: [Theme; 2] = [
    // Nord themes, https://www.nordtheme.com/
    Theme {
        name: "nord-frost",
        hour: "#5E81AC",
        minute: "#81A1C1",
        second: "#88C0D0",
        clock_face: "#8FBCBB",
    },
    Theme {
        name: "nord-aurora",
        hour: "#BF616A",
        minute: "#D08770",
        second: "#EBCB8B",
        clock_face: "#B48EAD",
    },
];
