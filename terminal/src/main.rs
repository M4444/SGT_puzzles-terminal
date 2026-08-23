mod ffi;
mod net;
mod render;

fn main() {
    let puzzle = net::generate();
    println!("{}", render::render_board(&puzzle));
}
