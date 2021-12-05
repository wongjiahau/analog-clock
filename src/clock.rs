use bresenham::Bresenham;
use chrono::{DateTime, Local, Timelike};
use colored::{self, Colorize};
use colors_transform::Color;
use colors_transform::Rgb;
use image::{
    imageops::{crop, resize},
    GenericImageView, ImageBuffer, Luma,
};
use line_drawing::BresenhamCircle;
use std::f32::consts::PI;
use std::time::Duration;

pub struct RunClockOptions {
    /// Color of the clock.
    pub color: Rgb,

    /// How often should the clock be redrawn.
    pub tick_interval: Duration,

    pub show_second_hand: bool,
    pub show_hour_labels: bool,
    pub show_minute_labels: bool,
}

pub fn run_clock(options: RunClockOptions) -> Result<(), String> {
    let (width, height) =
        term_size::dimensions().ok_or_else(|| "Unable to get term size :(".to_string())?;
    loop {
        let midpoint_x = (width as f32) / 2.0;
        let midpoint_y = (height as f32) / 2.0;
        let matrix = Matrix {
            cells: vec![vec![Cell { on: false }; width]; height],
            width,
            height,
            midpoint_x,
            midpoint_y,
            circle_radius: midpoint_x.min(midpoint_y) / 1.1,
        };
        let datetime: DateTime<Local> = Local::now();

        let matrix = matrix.draw_circle();

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

        let matrix = if options.show_second_hand {
            matrix.draw_hand(Hand {
                degree: degree_second,
                width: 2.0,
                length: 0.9,
                line_start: HandLineStart::FromCenter,
            })
        } else {
            matrix
        };

        let matrix = matrix.draw_hand(Hand {
            degree: degree_minute,
            width: 3.0,
            length: 0.9,
            line_start: HandLineStart::FromCenter,
        });
        let matrix = matrix.draw_hand(Hand {
            degree: degree_hour,
            width: 4.0,
            length: 0.5,
            line_start: HandLineStart::FromCenter,
        });

        // Draw clock face: hour labels
        let matrix = if options.show_hour_labels {
            (0..12).into_iter().fold(matrix, |matrix, n| {
                matrix.draw_hand(Hand {
                    degree: (n as f32) / 12.0 * 360.0,
                    width: 2.0,
                    length: 0.15,
                    line_start: HandLineStart::FromCircumference,
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
                    width: 2.0,
                    length: 0.05,
                    line_start: HandLineStart::FromCircumference,
                })
            })
        } else {
            matrix
        };

        // After computing the final matrix, we have to apply vertical/horizontal scaling to it
        // such that the clock will look like a circle  instead of an ellipse.
        // This is because each "pixel" (or character) on a terminal is not square-ish, but a
        // vertical rectangle instead.

        let mut img = matrix_to_luma_image_buffer(&matrix);

        let padding = matrix.circle_radius / 10.0;
        let img = {
            let new_x = (matrix.midpoint_x - matrix.circle_radius - padding) as u32;
            let new_y = 0;
            let new_width = ((matrix.circle_radius + padding) * 2.0) as u32;
            let new_height = matrix.height as u32;
            crop(&mut img, new_x, new_y, new_width, new_height)
        };
        let img = resize(
            &img,
            matrix.width as u32,
            img.height(),
            image::imageops::FilterType::Nearest,
        );
        let matrix = Matrix {
            cells: luma_image_buffer_to_matrix(img),
            ..matrix
        };

        matrix.print_matrix(options.color);
        std::thread::sleep(options.tick_interval);

        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    }
}

fn matrix_to_luma_image_buffer(matrix: &Matrix) -> ImageBuffer<Luma<u8>, Vec<u8>> {
    ImageBuffer::from_fn(matrix.width as u32, matrix.height as u32, |x, y| {
        if matrix.cells[y as usize][x as usize].on {
            image::Luma([255u8])
        } else {
            image::Luma([0u8])
        }
    })
}

fn luma_image_buffer_to_matrix(img: ImageBuffer<Luma<u8>, Vec<u8>>) -> Vec<Vec<Cell>> {
    let width = img.width() as usize;
    let mut cells = vec![vec![Cell { on: false }; width]; img.height() as usize];
    img.pixels().enumerate().for_each(|(index, pixel)| {
        let y = (index as f32 / width as f32).floor() as usize;
        let x = index % width;
        cells[y][x].on = pixel.0[0] == 255
    });
    cells
}

#[derive(Clone, Debug)]
struct Cell {
    on: bool,
}

struct Matrix {
    cells: Vec<Vec<Cell>>,
    height: usize,
    width: usize,
    midpoint_x: f32,
    midpoint_y: f32,
    circle_radius: f32,
}

impl Matrix {
    fn draw_circle(mut self) -> Matrix {
        let points = BresenhamCircle::new(
            self.midpoint_x as i32,
            self.midpoint_y as i32,
            self.circle_radius as i32,
        );
        for (x, y) in points {
            self.cells[y as usize][x as usize].on = true;
        }
        self
    }

    /// Draw a line originated from the center.
    /// We will be using [Bresenham Line Algorithm](https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm#History).
    fn draw_hand(self, hand: Hand) -> Matrix {
        let degree = hand.degree;
        let midpoint_x = self.midpoint_x;
        let midpoint_y = self.midpoint_y;
        let midpoint = (midpoint_x as isize, midpoint_y as isize);

        let radian = PI / 2.0 - (degree).to_radians();

        let radius = self.circle_radius;

        // We treat radius as the hypotenuse
        let hypotenuse = radius;

        // Trigonometry hints:
        // Adjacent = Hypotenuse * cos theta
        // Opposite = Hypotenuse * sin theta

        // Calculate startpoint based on line_start
        let startpoint = {
            match hand.line_start {
                HandLineStart::FromCenter => midpoint,
                HandLineStart::FromCircumference => {
                    let x = midpoint_x + hypotenuse * (1.0 - hand.length) * radian.cos();
                    let y = midpoint_y + hypotenuse * (1.0 - hand.length) * radian.sin();
                    (x as isize, y as isize)
                }
            }
        };

        // Calculate endpoint based on line_start
        let endpoint = {
            match hand.line_start {
                HandLineStart::FromCenter => {
                    let x = midpoint_x + hypotenuse * hand.length * radian.cos();
                    let y = midpoint_y + hypotenuse * hand.length * radian.sin();
                    (x as isize, y as isize)
                }
                HandLineStart::FromCircumference => {
                    let x = midpoint_x + hypotenuse * radian.cos();
                    let y = midpoint_y + hypotenuse * radian.sin();
                    (x as isize, y as isize)
                }
            }
        };

        let points = Bresenham::new(startpoint, endpoint)
            .map(|(x, y)| Point {
                x,

                // We have to invert y because the result returned by Bresenham is based on Cartesian plane
                // where (0, 0) is at the bottom left corner.
                // However for our matrix, (0, 0) is at the top left corner, which is like the Cartesian
                // plane flip around the x-axis.
                y: self.height as isize - y,
            })
            .collect();

        self.draw_using_points(points)
    }

    fn print_matrix(self, color: Rgb) {
        let block = "█".truecolor(
            color.get_red() as u8,
            color.get_green() as u8,
            color.get_blue() as u8,
        );
        for row in self.cells {
            for cell in row {
                if cell.on {
                    print!("{}", block)
                } else {
                    print!(" ")
                }
            }
            println!()
        }
    }

    fn draw_using_points(mut self, points: Vec<Point>) -> Matrix {
        for point in points {
            self.cells[point.y as usize][point.x as usize].on = true
        }
        self
    }
}

struct Point {
    x: isize,
    y: isize,
}
struct Hand {
    /// 0 to 360, where:
    /// 0 = North,
    /// 90 = East,
    /// 180 = South,
    /// 270 = West.
    degree: f32,
    width: f32,
    /// In terms of percentage. 0 is shortest, 1 is longest.
    length: f32,
    line_start: HandLineStart,
}
enum HandLineStart {
    FromCenter,
    FromCircumference,
}
