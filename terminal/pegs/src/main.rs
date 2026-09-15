//! Draws a hardcoded Pegs board for the terminal.

use common::canvas::{Canvas, Coord, Rect, Size, Weight, draw_line, flatten_to_lines};

fn main() {
    // The 7x7 cross, with squares 4 columns and 2 rows apart.
    let board_rect = Rect::new(Coord::new(0, 0), Size::new(29, 15));
    let mut canvas = Canvas::new(board_rect, Size::new(0, 0));

    // The outline, clockwise from the top left of the top arm.
    draw_line(&mut canvas, (8, 0), (20, 0), Weight::Light);
    draw_line(&mut canvas, (20, 0), (20, 4), Weight::Light);
    draw_line(&mut canvas, (20, 4), (28, 4), Weight::Light);
    draw_line(&mut canvas, (28, 4), (28, 10), Weight::Light);
    draw_line(&mut canvas, (28, 10), (20, 10), Weight::Light);
    draw_line(&mut canvas, (20, 10), (20, 14), Weight::Light);
    draw_line(&mut canvas, (20, 14), (8, 14), Weight::Light);
    draw_line(&mut canvas, (8, 14), (8, 10), Weight::Light);
    draw_line(&mut canvas, (8, 10), (0, 10), Weight::Light);
    draw_line(&mut canvas, (0, 10), (0, 4), Weight::Light);
    draw_line(&mut canvas, (0, 4), (8, 4), Weight::Light);
    draw_line(&mut canvas, (8, 4), (8, 0), Weight::Light);

    canvas.draw_text((10, 1), "⬤   ⬤   ⬤");
    canvas.draw_text((10, 3), "⬤   ⬤   ⬤");
    canvas.draw_text((2, 5), "⬤   ⬤   ⬤   ⬤   ⬤   ⬤   ⬤");
    canvas.draw_text((2, 7), "⬤   ⬤   ⬤   ◯   ⬤   ⬤   ⬤");
    canvas.draw_text((2, 9), "⬤   ⬤   ⬤   ⬤   ⬤   ⬤   ⬤");
    canvas.draw_text((10, 11), "⬤   ⬤   ⬤");
    canvas.draw_text((10, 13), "⬤   ⬤   ⬤");

    for line in flatten_to_lines(&canvas, usize::MAX) {
        println!("{line}");
    }
}
