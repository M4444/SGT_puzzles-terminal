/*
 * terminal-net.c: the parts of the terminal front end that only Net
 * needs. The Rust side reads net.c's own WINDOW_OFFSET from
 * window_offset().
 */

#include "puzzles.h"

/* Returns net.c's own WINDOW_OFFSET, which it never exposes outside
 * its own compilation unit. Mirrors its #ifdef exactly, so this
 * always matches net.c's real value regardless of whether
 * SMALL_SCREEN is defined. */
int window_offset(void);

int window_offset(void)
{
#ifndef SMALL_SCREEN
    return 16;
#else
    return 4;
#endif
}
