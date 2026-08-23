fn main() {
    let sources = [
        "../dsf.c",
        "../findloop.c",
        "../malloc.c",
        "../midend.c",
        "../misc.c",
        "../net.c",
        "../random.c",
        "../terminal.c",
        "../tree234.c",
    ];

    let mut build = cc::Build::new();
    build.define("EXPOSE_GAME_STATE", None);
    for source in sources {
        build.file(source);
        println!("cargo:rerun-if-changed={source}");
    }
    println!("cargo:rerun-if-changed=../puzzles.h");
    println!("cargo:rerun-if-changed=../tree234.h");
    build.compile("netpuzzle");

    if !cfg!(windows) {
        println!("cargo:rustc-link-lib=m");
    }
}
