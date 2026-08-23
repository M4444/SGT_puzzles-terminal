mod ffi;
mod net;
mod render;

fn main() {
    let puzzle = net::generate();
    let cursor_style = render::CursorStyle::default();
    println!("{}", render::render_board(&puzzle, cursor_style));
}
