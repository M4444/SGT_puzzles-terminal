/*
 * terminal.c: terminal front end for my puzzle collection. Stub
 * implementations of the mid-end's frontend callbacks and the drawing
 * API shared by every game. Every drawing call here is a no-op, since
 * this front end renders text directly. The exceptions are
 * status_bar() and emit_state(), which hand the status line text and
 * the game state to the Rust side. Alongside the stubs, can_solve()
 * reports whether the game has a solver.
 * Each game's own file, terminal/<game>/terminal-<game>.c, holds what
 * only that game needs.
 */

#include <stdarg.h>

#include "puzzles.h"

extern void rust_status_bar(drawing *dr, const char *text);
#ifdef EXPOSE_GAME_STATE
extern void rust_emit_state(drawing *dr, const void *data);
#endif

void frontend_default_colour(frontend *fe, float *output) {}
void deactivate_timer(frontend *fe) {}
void activate_timer(frontend *fe) {}
drawing *drawing_new(const drawing_api *api, midend *me, void *handle)
{
    drawing *dr = snew(drawing);
    dr->api = api;
    dr->handle = handle;
    return dr;
}
void drawing_free(drawing *dr) { sfree(dr); }
void draw_text(drawing *dr, int x, int y, int fonttype, int fontsize,
               int align, int colour, const char *text) {}
void draw_rect(drawing *dr, int x, int y, int w, int h, int colour) {}
#ifndef STANDALONE_POLYGON
void draw_line(drawing *dr, int x1, int y1, int x2, int y2, int colour) {}
#endif
void draw_thick_line(drawing *dr, float thickness,
		     float x1, float y1, float x2, float y2, int colour) {}
void draw_polygon(drawing *dr, const int *coords, int npoints,
                  int fillcolour, int outlinecolour) {}
void draw_circle(drawing *dr, int cx, int cy, int radius,
                 int fillcolour, int outlinecolour) {}
char *text_fallback(drawing *dr, const char *const *strings, int nstrings)
{ return dupstr(strings[0]); }
void clip(drawing *dr, int x, int y, int w, int h) {}
void unclip(drawing *dr) {}
void start_draw(drawing *dr) {}
void draw_update(drawing *dr, int x, int y, int w, int h) {}
void end_draw(drawing *dr) {}
struct blitter { char dummy; };
blitter *blitter_new(drawing *dr, int w, int h) { return snew(blitter); }
void blitter_free(drawing *dr, blitter *bl) { sfree(bl); }
void blitter_save(drawing *dr, blitter *bl, int x, int y) {}
void blitter_load(drawing *dr, blitter *bl, int x, int y) {}
int print_mono_colour(drawing *dr, int grey) { return 0; }
int print_grey_colour(drawing *dr, float grey) { return 0; }
int print_hatched_colour(drawing *dr, int hatch) { return 0; }
int print_rgb_mono_colour(drawing *dr, float r, float g, float b, int grey)
{ return 0; }
int print_rgb_grey_colour(drawing *dr, float r, float g, float b, float grey)
{ return 0; }
int print_rgb_hatched_colour(drawing *dr, float r, float g, float b, int hatch)
{ return 0; }
void print_line_width(drawing *dr, int width) {}
void print_line_dotted(drawing *dr, bool dotted) {}
void status_bar(drawing *dr, const char *text) { rust_status_bar(dr, text); }
void document_add_puzzle(document *doc, const game *game, game_params *par,
			 game_ui *ui, game_state *st, game_state *st2) {}
#ifdef EXPOSE_GAME_STATE
const drawing_api terminal_drawing_api = {
    .version = 1,
    .emit_state = rust_emit_state,
};
#endif

/* Returns thegame.can_solve, which the mid-end has no call for. */
bool can_solve(void);

bool can_solve(void)
{
    return thegame.can_solve;
}

void fatal(const char *fmt, ...)
{
    va_list ap;

    fprintf(stderr, "fatal error: ");

    va_start(ap, fmt);
    vfprintf(stderr, fmt, ap);
    va_end(ap);

    fprintf(stderr, "\n");
    exit(1);
}

#ifdef DEBUGGING
void debug_printf(const char *fmt, ...)
{
    va_list ap;
    va_start(ap, fmt);
    vfprintf(stdout, fmt, ap);
    va_end(ap);
}
#endif
