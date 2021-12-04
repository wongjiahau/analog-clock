use bresenham::Bresenham;
use chrono::{DateTime, Local, Timelike};
use image::{
    imageops::{crop, resize},
    GenericImageView, ImageBuffer, Luma,
};
use line_drawing::BresenhamCircle;
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

        // The height of character divides by the width of character in the terminal.
        // Cannot be obtained programmatically, thus must be measured manually by user.
        // Stretch ratio is necesary to make the circle look rounder, otherwise it will looks like a vertical ellipse.
        let stretch_ratio = 1.0 / 0.5;
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

fn draw_circle(mut matrix: Matrix) -> Matrix {
    let points = BresenhamCircle::new(
        matrix.midpoint_x as i32,
        matrix.midpoint_y as i32,
        matrix.circle_radius as i32,
    );
    for (x, y) in points {
        matrix.cells[y as usize][x as usize].on = true;
    }
    matrix
}

struct Point {
    x: isize,
    y: isize,
}
fn draw_using_points(mut matrix: Matrix, points: Vec<Point>) -> Matrix {
    for point in points {
        matrix.cells[point.y as usize][point.x as usize].on = true
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
    let radian = PI / 2.0 - (degree).to_radians();

    let radius = matrix.circle_radius;

    // We treat radius as the hypotenuse
    let hypotenuse = radius * hand.length;

    // Adjacent = Hypotenuse * cos theta
    let endpoint_x = midpoint_x + hypotenuse * (radian.cos());

    // Opposite = Hypotenuse * sin theta
    let endpoint_y = midpoint_y + hypotenuse * (radian.sin());

    let points = Bresenham::new(
        (midpoint_x as isize, midpoint_y as isize),
        (endpoint_x as isize, endpoint_y as isize),
    )
    .map(|(x, y)| Point {
        x,

        // We have to invert y because the result returned by Bresenham is based on Cartesian plane 
        // where (0, 0) is at the bottom left corner.
        // However for our matrix, (0, 0) is at the top left corner, which is like the Cartesian
        // plane flip around the x-axis.
        y: matrix.height as isize - y,
    })
    .collect();

    draw_using_points(matrix, points)
}

fn print_matrix(matrix: Matrix) {
    for row in matrix.cells {
        for cell in row {
            print!("{}", if cell.on { "█" } else { " " })
        }
        println!()
    }
}
