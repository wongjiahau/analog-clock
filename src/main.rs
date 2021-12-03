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
            circle_radius: midpoint_x.min(midpoint_y) / 1.1,
            aspect_ratio: 4.0,
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

        let matrix = draw_hand(
            matrix,
            Hand {
                degree: degree_second,
                width: 2.0,
                length: 0.9,
            },
        );
        let matrix = draw_hand(
            matrix,
            Hand {
                degree: degree_minute,
                width: 3.0,
                length: 0.9,
            },
        );
        let matrix = draw_hand(
            matrix,
            Hand {
                degree: degree_hour,
                width: 4.0,
                length: 0.6,
            },
        );

        // Draw clock face
        // let matrix = (0..12).into_iter().fold(matrix, |matrix, n|{
        //     draw_hand(matrix, Line {
        //         degree: (n as f32) / 12.0 * 360.0,
        //         width: 2.0,
        //         length: 1.0
        //     })

        // });
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

    /// The height of character divides by the width of character in the terminal.
    /// Cannot be obtained programmatically, thus must be measured manually by user.
    /// Aspect ratio is necesary to make the circle look rounder, otherwise it will looks like a vertical ellipse.
    aspect_ratio: f32,
}

fn draw_circle(matrix: Matrix) -> Matrix {
    let midpoint_x = matrix.midpoint_x;
    let midpoint_y = matrix.midpoint_y;
    let radius = matrix.circle_radius;
    let aspect_ratio = matrix.aspect_ratio;

    // Paint circle based on approx. of the circle equation, (x - midpoint_x)^2 + (y - midpoint_y)^2 = radius^2
    draw_using_equation(matrix, |x, y| {
        let left = ((x as f32) - midpoint_x).powf(2.0) / aspect_ratio
            + ((y as f32) - midpoint_y).powf(2.0);
        let right = radius.powf(2.0);
        let diff = left - right;
        diff.abs() < radius
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
}

/// Draw a line originated from the center.
fn draw_hand(matrix: Matrix, hand: Hand) -> Matrix {
    let degree = hand.degree;
    let midpoint_x = matrix.midpoint_x;
    let midpoint_y = matrix.midpoint_y;
    let radian = degree * (2.0 * PI) / 360.0;
    let radius = matrix.circle_radius;
    let aspect_ratio = matrix.aspect_ratio;
    draw_using_equation(matrix, |x, y| {
        let error_margin = 0.001_f32;

        // Check if the points fall outside of the circle
        // by using the circle equation: (x - midpoint_x)^2 + (y - midpoint_y)^2 = radius^2
        let left = ((x as f32) - midpoint_x).powf(2.0) / aspect_ratio
            + ((y as f32) - midpoint_y).powf(2.0);
        let right = ( radius * hand.length ).powf(2.0);
        if left > right {
            false
        } else if (degree - 90.0).abs() < error_margin {
            // need to specially handle 90.0 and 180.0, as tan(90degree) and tan(180degree) is
            // infinity.
            x >= midpoint_x && (y - midpoint_y).abs() <= hand.width
        } else if (degree - 180.0).abs() < error_margin {
            x <= midpoint_x && (y - midpoint_y).abs() <= hand.width
        }
        // Check if the points fall outside of the desired quadrant
        else if ((0.0..=90.0).contains(&degree) && !(x >= midpoint_x && y >= midpoint_y))
            || (90.0..=180.0).contains(&degree) && !(x >= midpoint_x && y <= midpoint_y)
            || (180.0..=270.0).contains(&degree) && !(x <= midpoint_x && y <= midpoint_y)
            || (270.0..=360.0).contains(&degree) && !(x <= midpoint_x && y >= midpoint_y)
        {
            false
        } else {
            // TODO: need to factor in the aspect_ratio when calculating the gradian
            // The formula use here is: `(x - midpoint_x) = (tan s) * (y + midpoint_y)`
            // where `s` is in terms of radian.
            let diff = (x - midpoint_x) - radian.tan() * (y - midpoint_y);
            diff.abs() < hand.width
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
