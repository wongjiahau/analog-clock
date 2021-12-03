use bresenham::Bresenham;
use chrono::{DateTime, Local, Timelike};
use image::{
    imageops::{crop, resize},
    GenericImage, GenericImageView, ImageBuffer, Luma, RgbImage,
};
use std::f32::consts::PI;
use std::time::Duration;

fn main() {
    match run_clock() {
        Ok(_) => (),
        Err(error) => eprintln!("{}", error),
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
            aspect_ratio: 1.0,
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
        // let matrix = (0..12).into_iter().fold(matrix, |matrix, n| {
        //     draw_hand(
        //         matrix,
        //         Hand {
        //             degree: (n as f32) / 12.0 * 360.0,
        //             width: 2.0,
        //             length: 1.0,
        //         },
        //     )
        // });

        let mut img = matrix_to_luma_image_buffer(&matrix);
        let stretch_ratio = 2.0;
        let img = crop(
            &mut img,
            (matrix.midpoint_x - matrix.circle_radius * stretch_ratio) as u32,
            0,
            (matrix.circle_radius * stretch_ratio * 2.0) as u32,
            matrix.height as u32,
        );
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
/// We will be using [Bresenham Line Algorithm](https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm#History).
fn draw_hand(matrix: Matrix, hand: Hand) -> Matrix {
    let degree = hand.degree;
    let midpoint_x = matrix.midpoint_x;
    let midpoint_y = matrix.midpoint_y;
    let radian = (PI / 2.0 - (degree).to_radians());

    // adjust radian based on aspect_ratio
    let radian = radian; //+ radian.cos() / matrix.aspect_ratio;

    let radius = matrix.circle_radius;
    let aspect_ratio = matrix.aspect_ratio;

    // We treat radius as the hypotenuse
    // Adjacent = Hypotenuse * cos theta
    let endpoint_x = midpoint_x + (radius) * (radian.cos() * matrix.aspect_ratio);

    // Opposite = Hypotenuse * sin theta
    let endpoint_y = midpoint_y + (radius) * (radian.sin() * matrix.aspect_ratio);

    draw_using_equation(matrix, |x, y| {
        let error_margin = 0.001_f32;

        // Check if the points fall outside of the circle
        // by using the circle equation: (x - midpoint_x)^2 + (y - midpoint_y)^2 = radius^2
        let left = ((x as f32) - midpoint_x).powf(2.0) / aspect_ratio
            + ((y as f32) - midpoint_y).powf(2.0);
        let right = (radius * hand.length).powf(2.0);
        if left > right {
            return false;
        }

        let mut line = Bresenham::new(
            (midpoint_x as isize, midpoint_y as isize),
            (endpoint_x as isize, endpoint_y as isize),
        );
        // else check if the points falls in the Bresenham line
        if line.any(|(line_x, line_y)| line_x == x as isize && line_y == y as isize) {
            return true;
        }
        return false;
        // Check if the points fall outside of the desired quadrant
        if ((0.0..=90.0).contains(&degree) && !(x >= midpoint_x && y >= midpoint_y))
            || (90.0..=180.0).contains(&degree) && !(x >= midpoint_x && y <= midpoint_y)
            || (180.0..=270.0).contains(&degree) && !(x <= midpoint_x && y <= midpoint_y)
            || (270.0..=360.0).contains(&degree) && !(x <= midpoint_x && y >= midpoint_y)
        {
            false
        } else {
            // TODO: need to factor in the aspect_ratio when calculating the gradian
            // The formulas use here are:
            //  (x - midpoint_x) = (tan s) * (y - midpoint_y)
            //  and
            //  (tan (s + 90degree)) * (x - midpoint_x) = (y - midpoint_y)
            //
            // where s is in terms of radian.
            //
            // We have to use two formulas so that the hand will look almost equally wide at any
            // angle.
            let gradient1 = radian.tan() * aspect_ratio / 2.0;
            let diff1 = (x - midpoint_x) - gradient1 * (y - midpoint_y);

            let gradient2 = (radian + PI / 2.0).tan() * aspect_ratio / 2.0;
            let diff2 = gradient2 * (x - midpoint_x) - (y - midpoint_y);
            diff1.abs() < hand.width //|| diff2.abs() < hand.width
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
