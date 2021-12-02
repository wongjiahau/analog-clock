use chrono::offset::Utc;
use chrono::DateTime;
use colored::*;
use std::process::{self, exit};
use std::time::Duration;

use term_size;
fn main() {
    match run_clock() {
        Ok(_) => (),
        Err(error) => eprintln!("{}", error),
    }
}

fn run_clock() -> Result<(), String> {
    loop {
        let (width, height) =
            term_size::dimensions().ok_or("Unable to get term size :(".to_string())?;
        let mut matrix = vec![vec![Cell { on: false }; width]; height];
        let block = "█".to_string().blue();
        let system_time = std::time::SystemTime::now();
        let datetime: DateTime<Utc> = system_time.into();

        let matrix = make_circle(matrix);
        print_matrix(matrix);
        std::thread::sleep(Duration::from_millis(100));

        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    }
}

#[derive(Clone, Debug)]
struct Cell {
    on: bool,
}

type Matrix = Vec<Vec<Cell>>;

fn make_circle(mut matrix: Matrix) -> Matrix {
    let height = matrix.len();
    let width = matrix[0].len();
    let midpoint_x = (width as f32) / 2.0;
    let midpoint_y = (height as f32) / 2.0;
    let radius = if midpoint_x < midpoint_y {
        midpoint_x
    } else {
        midpoint_y
    } as f32
        / 2.0;
    let border_width = 1.0;
    let delta_upper = (radius as f32 + border_width).powf(2.0);
    let delta_lower = (radius as f32 - border_width).powf(2.0);

    // Paint circle based on approx. of the circle equation, (x - midpoint)^2 + (y - midpoint)^2 = radius^2

    // The height of character divides by the width of character in the terminal
    // Cannot be obtained programmatically, thus must be measured manually by user
    // Aspect ratio is necesary to make the circle look rounder, otherwise it will looks like
    // a vertical ellipse
    let aspect_ratio = 4 as f32;

    for x in 0..width {
        for y in 0..height {
            let left = ((x as f32) - midpoint_x).powf(2.0) / aspect_ratio
                + ((y as f32) - midpoint_y).powf(2.0);
            let right = radius.powf(2.0);
            let diff = left - right;
            if delta_lower <= diff && diff <= delta_upper {
                matrix[y][x].on = true
            }
        }
    }
    return matrix;
}

fn print_matrix(matrix: Matrix) {
    for row in matrix {
        for cell in row {
            print!("{}", if cell.on { "█" } else { " " })
        }
        println!("")
    }
}
