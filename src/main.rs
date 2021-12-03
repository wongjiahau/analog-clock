use chrono::{DateTime, Local, Timelike};
use std::f32::consts::PI;
use std::time::Duration;

fn main() {
    match run_clock() {
        Ok(_) => (),
        Err(error) => eprintln!("{}", error),
    }
}

fn run_clock() -> Result<(), String> {
    loop {
        let (width, height) =
            term_size::dimensions().ok_or_else(|| "Unable to get term size :(".to_string())?;
        let midpoint_x = (width as f32) / 2.0;
        let midpoint_y = (height as f32) / 2.0;
        let matrix = Matrix {
            cells: vec![vec![Cell { on: false }; width]; height],
            width,
            height,
            midpoint_x,
            midpoint_y,
            circle_radius: midpoint_x.min(midpoint_y) / 1.75,
        };
        let datetime: DateTime<Local> = Local::now();

        let matrix = draw_circle(matrix);

        let millisecond = datetime.timestamp_millis() % 1000;
        let second = datetime.second() as f32;
        let minute = datetime.minute() as f32;
        let hour = (datetime.hour() % 12) as f32;

        let degree_second = (second + (millisecond as f32) / 1000.0) / 60.0 * 360.0;
        let degree_minute = (minute + second / 60.0) / 60.0 * 360.0;
        let degree_hour = (hour + minute / 60.0) / 12.0 * 360.0;

        let matrix = draw_line(
            matrix,
            Line {
                degree: degree_second,
                width: 1.0,
                length: 1.0,
            },
        );
        let matrix = draw_line(
            matrix,
            Line {
                degree: degree_minute,
                width: 2.0,
                length: 1.0,
            },
        );
        let matrix = draw_line(
            matrix,
            Line {
                degree: degree_hour,
                width: 3.0,
                length: 0.7,
            },
        );
        print_matrix(matrix);
        std::thread::sleep(Duration::from_millis(100));

        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    }
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

fn draw_circle(matrix: Matrix) -> Matrix {
    let midpoint_x = matrix.midpoint_x;
    let midpoint_y = matrix.midpoint_y;
    let radius = matrix.circle_radius;
    let border_width = 1.0;
    let delta_upper = (radius as f32 + border_width).powf(2.0);
    let delta_lower = (radius as f32 - border_width).powf(2.0);

    // Paint circle based on approx. of the circle equation, (x - midpoint_x)^2 + (y - midpoint_y)^2 = radius^2

    // The height of character divides by the width of character in the terminal
    // Cannot be obtained programmatically, thus must be measured manually by user
    // Aspect ratio is necesary to make the circle look rounder, otherwise it will looks like
    // a vertical ellipse
    let aspect_ratio = 4_f32;

    draw_using_equation(matrix, |x, y| {
        let left = ((x as f32) - midpoint_x).powf(2.0) / aspect_ratio
            + ((y as f32) - midpoint_y).powf(2.0);
        let right = radius.powf(2.0);
        let diff = left - right;
        delta_lower <= diff && diff <= delta_upper
    })
}

fn draw_using_equation<F>(mut matrix: Matrix, equation: F) -> Matrix
where
    F: Fn(/* x */ f32, /* y */ f32) -> bool,
{
    for x in 0..matrix.width {
        for y in 0..matrix.height {
            // We have to correct y so that y will follows the normal Cartesian plane,
            // where up = increase & down = decrease.
            //
            // Otherwise it would be up = decrease & down = increase which is quite
            // counter-intuitive.
            let corrected_y = (matrix.height as f32) - (y as f32);
            if equation(x as f32, corrected_y) {
                matrix.cells[y][x].on = true
            }
        }
    }
    matrix
}

struct Line {
    degree: f32,
    width: f32,
    /// In terms of percentage. 0 is shortest, 1 is longest.
    length: f32,
}

/// Draw a line originated from the center,
/// with the given degree (where 0 means north).
/// The formula use here is: `(x - midpoint_x) = (tan s) * (y + midpoint_y)`
/// where `s` is in terms of radian.
fn draw_line(matrix: Matrix, line: Line) -> Matrix {
    let degree = line.degree;
    let midpoint_x = matrix.midpoint_x;
    let midpoint_y = matrix.midpoint_y;
    let radian = degree * (2.0 * PI) / 360.0;
    let radius = matrix.circle_radius;
    draw_using_equation(matrix, |x, y| {
        let error_margin = 0.00001_f32;

        // Check if the points fall outside of the circle
        if distance(
            Point {
                x: midpoint_x,
                y: midpoint_y,
            },
            Point { x, y },
        ) > radius * line.length
        {
            false
        } else if (degree - 90.0).abs() < error_margin {
            // need to specially handle 90.0 and 180.0, as tan(90degree) and tan(180degree) is
            // infinity.
            x >= midpoint_x && (y - midpoint_y).abs() < error_margin
        } else if (degree - 180.0).abs() < error_margin {
            x <= midpoint_x && (y - midpoint_y).abs() < error_margin
        } else if ((0.0..=90.0).contains(&degree) && !(x >= midpoint_x && y >= midpoint_y))
            || (90.0..=180.0).contains(&degree) && !(x >= midpoint_x && y <= midpoint_y)
            || (180.0..=270.0).contains(&degree) && !(x <= midpoint_x && y <= midpoint_y)
            || (270.0..=360.0).contains(&degree) && !(x <= midpoint_x && y >= midpoint_y)
        {
            false
        } else {
            let diff = (x - midpoint_x) - radian.tan() * (y - midpoint_y);
            diff.abs() < line.width
        }
    })
}

fn print_matrix(matrix: Matrix) {
    for row in matrix.cells {
        for cell in row {
            print!("{}", if cell.on { "█" } else { " " })
        }
        println!()
    }
}

struct Point {
    x: f32,
    y: f32,
}

fn distance(a: Point, b: Point) -> f32 {
    ((a.x - b.x).powf(2.0) + (a.y - b.y).powf(2.0)).sqrt()
}
