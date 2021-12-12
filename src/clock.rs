use super::theme::Theme;
use bresenham::Bresenham;
use chrono::{DateTime, Local, Timelike};
use colors_transform::Color;
use colors_transform::Rgb;
use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode},
    style::{self, Stylize},
    terminal, ExecutableCommand, QueueableCommand, Result,
};
use image::{imageops::resize, ImageBuffer, Rgb as RgbPixel};
use line_drawing::BresenhamCircle;
use std::f32::consts::PI;
use std::io::{stdout, Write};
use std::process;
use std::time::Duration;

pub struct RunClockOptions {
    pub theme: Theme,

    /// How often should the clock be redrawn.
    pub tick_interval: Duration,

    pub show_second_hand: bool,
    pub show_hour_labels: bool,
    pub show_minute_labels: bool,
}

fn new_error(message: String) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, message.as_str())
}

struct UiState {
    /// Aspect ratio  = character_height / character_width
    ///
    /// where:
    ///
    ///   character_height is the actual height of each character in the terminal,
    ///   character_width is the actual width of each character in terminal
    ///
    /// This is needed to circularize the clock, otherwise it will look like an ellipse.
    aspect_ratio: f32,
}

pub fn run_clock(options: RunClockOptions) -> Result<()> {
    terminal::enable_raw_mode()?;

    stdout().execute(cursor::Hide)?;

    let mut state = UiState { aspect_ratio: 2.0 };

    loop {
        // Read for user input in a non-blocking manner
        // Refer https://docs.rs/crossterm/latest/crossterm/event/index.html#examples
        if poll(options.tick_interval)? {
            match read()? {
                Event::Key(event) => {
                    if event.code == KeyCode::Char('+') || event.code == KeyCode::Char('=') {
                        state.aspect_ratio += 0.1;
                    } else if event.code == KeyCode::Char('-') {
                        state.aspect_ratio -= 0.1;
                    } else if event.code == KeyCode::Char('q') {
                        stdout().execute(cursor::Show)?;
                        process::exit(0)
                    }
                }
                Event::Mouse(event) => println!("{:?}", event),
                Event::Resize(width, height) => println!("New size {}x{}", width, height),
            }
        }
        draw_clock(&state, &options)?
    }
}

fn draw_clock(state: &UiState, options: &RunClockOptions) -> Result<()> {
    let (screen_width, height) = term_size::dimensions()
        .ok_or_else(|| new_error("Unable to get term size :(".to_string()))?;
    let clock_width = (screen_width as f32) / state.aspect_ratio;
    let midpoint_x = (clock_width as f32) / 2.0;
    let midpoint_y = (height as f32) / 2.0;
    let matrix = Matrix {
        cells: vec![vec![None; clock_width as usize]; height],
        width: clock_width as usize,
        height,
        midpoint_x,
        midpoint_y,
        circle_radius: midpoint_x.min(midpoint_y) / 1.1,
    };
    let datetime: DateTime<Local> = Local::now();

    let matrix = matrix.draw_circle(Rgb::from_hex_str(options.theme.clock_face).unwrap());

    // Draw clock face: hour labels
    let matrix = if options.show_hour_labels {
        (0..12).into_iter().fold(matrix, |matrix, n| {
            matrix.draw_hand(Hand {
                degree: (n as f32) / 12.0 * 360.0,
                thickness: HandThickness::Thin,
                length: 0.15,
                line_start: HandLineStart::FromCircumference,
                color: Rgb::from_hex_str(options.theme.clock_face).unwrap(),
            })
        })
    } else {
        matrix
    };

    // Draw clock face: minute/seconds labels
    let matrix = if options.show_minute_labels {
        (0..60).into_iter().fold(matrix, |matrix, n| {
            matrix.draw_hand(Hand {
                degree: (n as f32) / 60.0 * 360.0,
                thickness: HandThickness::Thin,
                length: 0.05,
                line_start: HandLineStart::FromCircumference,
                color: Rgb::from_hex_str("#4C566A").unwrap(),
            })
        })
    } else {
        matrix
    };

    let millisecond = datetime.timestamp_millis() % 1000;
    let second = datetime.second() as f32;
    let minute = datetime.minute() as f32;
    let hour = (datetime.hour() % 12) as f32;

    let second = if options.tick_interval.as_millis() < 1000 {
        second + (millisecond as f32) / 1000.0
    } else {
        second
    };
    let degree_second = second / 60.0 * 360.0;
    let degree_minute = (minute + second / 60.0) / 60.0 * 360.0;
    let degree_hour = (hour + minute / 60.0) / 12.0 * 360.0;

    // Firstly, draw minute hand
    let matrix = matrix.draw_hand(Hand {
        degree: degree_minute,
        thickness: HandThickness::Bold,
        length: 0.9,
        line_start: HandLineStart::FromCenter,
        color: Rgb::from_hex_str(options.theme.minute).unwrap(),
    });

    // Secondly, draw hour hand, as hour hand must be on top of minute hand
    let matrix = matrix.draw_hand(Hand {
        degree: degree_hour,
        thickness: HandThickness::Bold,
        length: 0.5,
        line_start: HandLineStart::FromCenter,
        color: Rgb::from_hex_str(options.theme.hour).unwrap(),
    });

    // Thirdly, draw second hand, which should be on top of hour hand & minute hand
    let matrix = if options.show_second_hand {
        matrix.draw_hand(Hand {
            degree: degree_second,
            thickness: HandThickness::Thin,
            length: 0.9,
            line_start: HandLineStart::FromCenter,
            color: Rgb::from_hex_str(options.theme.second).unwrap(),
        })
    } else {
        matrix
    };

    // After computing the final matrix, we have to resize it
    let matrix = matrix.rescale(screen_width);

    matrix.print()?;
    Ok(())
}

#[derive(Clone, Debug)]
struct Cell {
    color: Rgb,
}

struct Matrix {
    cells: Vec<Vec<Option<Cell>>>,
    width: usize,
    height: usize,
    midpoint_x: f32,
    midpoint_y: f32,
    circle_radius: f32,
}

impl Matrix {
    fn draw_circle(mut self, color: Rgb) -> Matrix {
        let points = BresenhamCircle::new(
            self.midpoint_x as i32,
            self.midpoint_y as i32,
            self.circle_radius as i32,
        );
        for (x, y) in points {
            self.cells[y as usize][x as usize] = Some(Cell { color })
        }
        self
    }

    /// Draw a line originated from the center.
    /// We will be using [Bresenham Line Algorithm](https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm#History).
    fn draw_hand(self, hand: Hand) -> Matrix {
        let degree = hand.degree;
        let radian = PI / 2.0 - (degree).to_radians();
        let radius = self.circle_radius;

        let origins = {
            let x = self.midpoint_x;
            let y = self.midpoint_y;

            match hand.thickness {
                HandThickness::Thin => vec![(x, y)],
                HandThickness::Bold => vec![
                    (x - 1.0, y + 1.0), // top left
                    (x, y + 1.0),       // top
                    (x + 1.0, y + 1.0), // top right
                    (x - 1.0, y),       // left
                    (x, y),             // center
                    (x + 1.0, y),       // right
                    (x - 1.0, y - 1.0), // top left
                    (x, y - 1.0),       // top
                    (x + 1.0, y - 1.0), // top right
                ],
            }
        };

        origins
            .into_iter()
            .fold(self, |matrix, (midpoint_x, midpoint_y)| {
                // We treat radius as the hypotenuse
                // Trigonometry hints:
                // Adjacent = Hypotenuse * cos theta
                // Opposite = Hypotenuse * sin theta

                let get_point = |hypotenuse: f32| -> (isize, isize) {
                    let x = midpoint_x + hypotenuse * radian.cos();
                    let y = midpoint_y + hypotenuse * radian.sin();
                    (x as isize, y as isize)
                };

                // Calculate startpoint based on line_start
                let startpoint = match hand.line_start {
                    HandLineStart::FromCenter => get_point(0.0),
                    HandLineStart::FromCircumference => get_point(radius * (1.0 - hand.length)),
                };

                // Calculate endpoint based on line_start
                let endpoint = match hand.line_start {
                    HandLineStart::FromCenter => get_point(radius * hand.length),
                    HandLineStart::FromCircumference => get_point(radius),
                };

                let points = Bresenham::new(startpoint, endpoint)
                    .map(|(x, y)| Point {
                        x,

                        // We have to invert y because the result returned by Bresenham is based on Cartesian plane
                        // where (0, 0) is at the bottom left corner.
                        // However for our matrix, (0, 0) is at the top left corner, which is like the Cartesian
                        // plane flip around the x-axis.
                        y: matrix.height as isize - y,
                        color: hand.color,
                    })
                    .collect();

                matrix.draw_using_points(points)
            })
    }

    /// Apply vertical/horizontal scaling to the given matrix,
    /// such that the clock will look like a circle instead of an ellipse.
    /// This is because each "pixel" (or character) on a terminal is not square-ish, but a
    /// vertical rectangle instead.
    fn rescale(self, screen_width: usize) -> Matrix {
        let img = matrix_to_luma_image_buffer(&self);
        let img = resize(
            &img,
            screen_width as u32,
            self.height as u32,
            image::imageops::FilterType::Nearest,
        );
        Matrix {
            cells: luma_image_buffer_to_matrix(img),
            ..self
        }
    }

    fn print(self) -> Result<()> {
        let mut stdout = stdout();

        stdout.execute(terminal::Clear(terminal::ClearType::All))?;

        for (y, row) in self.cells.into_iter().enumerate() {
            for (x, cell) in row.into_iter().enumerate() {
                let x = x as u16;
                let y = y as u16;
                let character = match cell {
                    Some(cell) => "█".with(style::Color::Rgb {
                        r: cell.color.get_red() as u8,
                        g: cell.color.get_green() as u8,
                        b: cell.color.get_blue() as u8,
                    }),
                    None => " ".stylize(),
                };

                stdout
                    .queue(cursor::MoveTo(x, y))?
                    .queue(style::PrintStyledContent(character))?;
            }
        }

        stdout.flush()?;

        Ok(())
    }

    fn draw_using_points(mut self, points: Vec<Point>) -> Matrix {
        for point in points {
            self.cells[point.y as usize][point.x as usize] = Some(Cell { color: point.color })
        }
        self
    }
}

fn matrix_to_luma_image_buffer(matrix: &Matrix) -> ImageBuffer<RgbPixel<u8>, Vec<u8>> {
    ImageBuffer::from_fn(
        matrix.width as u32,
        matrix.height as u32,
        |x, y| match &matrix.cells[y as usize][x as usize] {
            Some(cell) => RgbPixel([
                cell.color.get_red() as u8,
                cell.color.get_green() as u8,
                cell.color.get_blue() as u8,
            ]),
            None => RgbPixel([255, 255, 255]),
        },
    )
}

fn luma_image_buffer_to_matrix(img: ImageBuffer<RgbPixel<u8>, Vec<u8>>) -> Vec<Vec<Option<Cell>>> {
    let width = img.width() as usize;
    let mut cells = vec![vec![None; width]; img.height() as usize];
    img.pixels().enumerate().for_each(|(index, pixel)| {
        let y = (index as f32 / width as f32).floor() as usize;
        let x = index % width;
        cells[y][x] = if pixel != &RgbPixel([255, 255, 255]) {
            Some(Cell {
                color: Rgb::from(pixel.0[0] as f32, pixel.0[1] as f32, pixel.0[2] as f32),
            })
        } else {
            None
        }
    });
    cells
}
struct Point {
    x: isize,
    y: isize,
    color: Rgb,
}
struct Hand {
    /// 0 to 360, where:
    /// 0 = North,
    /// 90 = East,
    /// 180 = South,
    /// 270 = West.
    degree: f32,
    thickness: HandThickness,
    /// In terms of percentage. 0 is shortest, 1 is longest.
    length: f32,
    line_start: HandLineStart,
    color: Rgb,
}
enum HandThickness {
    Thin,
    Bold,
}
enum HandLineStart {
    FromCenter,
    FromCircumference,
}
