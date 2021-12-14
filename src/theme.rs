#[derive(Clone, Copy)]
pub struct Theme {
    pub name: &'static str,
    pub hour: &'static str,
    pub minute: &'static str,
    pub second: &'static str,
    pub clock_face: &'static str,
}

pub const THEMES: [Theme; 7] = [
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
    // Gruvbox. https://github.com/morhetz/gruvbox
    Theme {
        name: "gruvbox-light",
        hour: "#928374",
        minute: "#a89984",
        second: "#bdae93",
        clock_face: "#d5c4a1",
    },
    Theme {
        name: "gruvbox-dark",
        hour: "#d5c4a1",
        minute: "#bdae93",
        second: "#a89984",
        clock_face: "#928374",
    },
    // Monokai. https://gist.github.com/r-malon/8fc669332215c8028697a0bbfbfbb32a
    Theme {
        name: "monokai",
        hour: "#66d9ef",
        minute: "#ae81ff",
        second: "#f92672",
        clock_face: "#a6e22e",
    },
    // Lime. https://encycolorpedia.com/b3cd4f#:~:text=The%20hexadecimal%20color%20code%20%23b3cd4f,%25%20saturation%20and%2056%25%20lightness.
    Theme {
        name: "lime-light",
        hour: "#8da729",
        minute: "#99b436",
        second: "#a6c043",
        clock_face: "#b3cd4f",
    },
    Theme {
        name: "lime-dark",
        hour: "#dbf474",
        minute: "#cde768",
        second: "#c0da5b",
        clock_face: "#b3cd4f",
    },
];
