fn main() {
    let sources = [
        "../../dsf.c",
        "../../findloop.c",
        "../../malloc.c",
        "../../midend.c",
        "../../misc.c",
        "../../random.c",
        "../../terminal.c",
        "../../tree234.c",
    ];
    let headers = ["../../puzzles.h", "../../tree234.h"];

    let mut build = cc::Build::new();
    build.define("EXPOSE_GAME_STATE", None);
    // Match upstream, which compiles its C with -Wall but not -Wextra.
    build.extra_warnings(false);
    for source in sources {
        build.file(source);
        println!("cargo:rerun-if-changed={source}");
    }
    for header in headers {
        println!("cargo:rerun-if-changed={header}");
    }
    build.compile("common");

    if !cfg!(windows) {
        println!("cargo:rustc-link-lib=m");
    }
}
