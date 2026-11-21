"""Ink-based spacing shared by the font builder and offline verification."""
SIDE_BEARING = 2.0
LOWER_MARKS = tuple(range(0x05B0, 0x05BC)) + (0x05BD, 0x05C7)


def spaced_metrics(bearing_x, advance, ink_box):
    """Reserve clear ink margins without shrinking the existing advance."""
    left, _, right, _ = ink_box
    bearing_x += max(0.0, SIDE_BEARING - (bearing_x + left))
    return bearing_x, max(advance, bearing_x + right + SIDE_BEARING)
