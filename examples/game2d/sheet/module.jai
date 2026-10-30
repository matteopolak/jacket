// The layout of the generated art, shared by the program that draws it (tools/assetgen) and the
// game that cuts it up again. Cell numbers count left to right, top to bottom.

SHEET_CELL :: 16;
SHEET_COLUMNS :: 8;
SHEET_ROWS :: 4;

Cell :: enum u8 {
    WHITE;
    PLAYER;
    DRONE;
    CHASER;
    TANK;
    SHOT;
    ENEMY_SHOT;
    DOT;

    EXPLOSION_0;
    EXPLOSION_1;
    EXPLOSION_2;
    EXPLOSION_3;
    SHIELD;
    STAR_SMALL;
    STAR_BIG;
    LIFE;
}

// The font sheet holds ASCII 32 to 127, 16 glyphs a row.
GLYPH_W :: 8;
GLYPH_H :: 8;
GLYPH_COLUMNS :: 16;
GLYPH_ROWS :: 6;
GLYPH_FIRST :: 32;

// Sound effects are 16-bit mono at this rate.
SAMPLE_RATE :: 22050;
