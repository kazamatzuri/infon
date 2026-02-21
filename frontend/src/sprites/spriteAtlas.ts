/**
 * Sprite atlas coordinates for theme.png sprite sheet.
 * Extracted from the original C source (sdl_sprite.c).
 */

export interface SpriteRect {
  x: number;
  y: number;
  w: number;
  h: number;
}

// ---------------------------------------------------------------------------
// Tiles (all 16x16)
// ---------------------------------------------------------------------------

/** Border tiles: 16 variants at Y=192, X = col*16 */
export const BORDER_SPRITES: SpriteRect[] = Array.from({ length: 16 }, (_, i) => ({
  x: i * 16,
  y: 192,
  w: 16,
  h: 16,
}));

/** Solid (wall) tiles: 16 variants at Y=208, X = col*16 */
export const SOLID_SPRITES: SpriteRect[] = Array.from({ length: 16 }, (_, i) => ({
  x: i * 16,
  y: 208,
  w: 16,
  h: 16,
}));

/** Plain (walkable) tiles: 16 variants at Y=224, X = col*16 */
export const PLAIN_SPRITES: SpriteRect[] = Array.from({ length: 16 }, (_, i) => ({
  x: i * 16,
  y: 224,
  w: 16,
  h: 16,
}));

/** Snow border tiles: 16 variants at Y=192, X = (16+col)*16 */
export const SNOW_BORDER_SPRITES: SpriteRect[] = Array.from({ length: 16 }, (_, i) => ({
  x: (16 + i) * 16,
  y: 192,
  w: 16,
  h: 16,
}));

/** Snow solid tiles: 16 variants at Y=208, X = (16+col)*16 */
export const SNOW_SOLID_SPRITES: SpriteRect[] = Array.from({ length: 16 }, (_, i) => ({
  x: (16 + i) * 16,
  y: 208,
  w: 16,
  h: 16,
}));

/** Snow plain tiles: 16 variants at Y=224, X = (16+col)*16 */
export const SNOW_PLAIN_SPRITES: SpriteRect[] = Array.from({ length: 16 }, (_, i) => ({
  x: (16 + i) * 16,
  y: 224,
  w: 16,
  h: 16,
}));

/** King of the Hill tile: single sprite at (0, 240) */
export const KOTH_SPRITE: SpriteRect = { x: 0, y: 240, w: 16, h: 16 };

/** Water tiles: 4 animation frames at Y=288, X = col*16 */
export const WATER_SPRITES: SpriteRect[] = Array.from({ length: 4 }, (_, i) => ({
  x: i * 16,
  y: 288,
  w: 16,
  h: 16,
}));

/** Lava tiles: 4 animation frames at Y=304, X = col*16 */
export const LAVA_SPRITES: SpriteRect[] = Array.from({ length: 4 }, (_, i) => ({
  x: i * 16,
  y: 304,
  w: 16,
  h: 16,
}));

/** Desert tiles: 10 variants at Y=240, X = (6+col)*16 */
export const DESERT_SPRITES: SpriteRect[] = Array.from({ length: 10 }, (_, i) => ({
  x: (6 + i) * 16,
  y: 240,
  w: 16,
  h: 16,
}));

// ---------------------------------------------------------------------------
// Food (16x16)
// ---------------------------------------------------------------------------

/** Normal food sprites: 10 levels at Y=256, X = f*16 (index 0 = food level 1) */
export const FOOD_SPRITES: SpriteRect[] = Array.from({ length: 10 }, (_, i) => ({
  x: i * 16,
  y: 256,
  w: 16,
  h: 16,
}));

/** Snow food sprites: 10 levels at Y=272, X = f*16 */
export const SNOW_FOOD_SPRITES: SpriteRect[] = Array.from({ length: 10 }, (_, i) => ({
  x: i * 16,
  y: 272,
  w: 16,
  h: 16,
}));

// ---------------------------------------------------------------------------
// Creatures (16x16)
// ---------------------------------------------------------------------------

/**
 * Creature base sprites: [type][anim]
 * type: 0=small, 1=big, 2=flyer
 * anim: 0 or 1
 * X = anim*16, Y = type*16
 */
export const CREATURE_BASE_SPRITES: SpriteRect[][] = Array.from({ length: 3 }, (_, type) =>
  Array.from({ length: 2 }, (_, anim) => ({
    x: anim * 16,
    y: type * 16,
    w: 16,
    h: 16,
  })),
);

/**
 * Creature overlay sprites: [type][anim]
 * X = 32 + anim*16, Y = type*16
 */
export const CREATURE_OVERLAY_SPRITES: SpriteRect[][] = Array.from({ length: 3 }, (_, type) =>
  Array.from({ length: 2 }, (_, anim) => ({
    x: 32 + anim * 16,
    y: type * 16,
    w: 16,
    h: 16,
  })),
);

// ---------------------------------------------------------------------------
// Thought Bubbles (16x16)
// ---------------------------------------------------------------------------

/**
 * Thought bubble sprites: 9 entries (indices 0-8)
 * States: 0=idle, 1=walk, 2=heal, 3=eat, 4=attack, 5=convert, 6=spawn, 7=feed, 8=smile
 * X=0, Y = 48 + state*16
 */
export const THOUGHT_SPRITES: SpriteRect[] = Array.from({ length: 9 }, (_, i) => ({
  x: 0,
  y: 48 + i * 16,
  w: 16,
  h: 16,
}));

// ---------------------------------------------------------------------------
// Special
// ---------------------------------------------------------------------------

/** Crown sprite: 64x50 at (0, 350) */
export const CROWN_SPRITE: SpriteRect = { x: 0, y: 350, w: 64, h: 50 };

/** Halo sprite: 32x32 at (16, 48) */
export const HALO_SPRITE: SpriteRect = { x: 16, y: 48, w: 32, h: 32 };

// ---------------------------------------------------------------------------
// Gfx index mapping
// ---------------------------------------------------------------------------

/**
 * Map from backend tile gfx enum to sprite data.
 *
 * Backend gfx values (from config.rs):
 *   0 = TILE_GFX_SOLID
 *   1 = TILE_GFX_PLAIN
 *   2 = TILE_GFX_BORDER
 *   3 = TILE_GFX_SNOW_SOLID
 *   4 = TILE_GFX_SNOW_PLAIN
 *   5 = TILE_GFX_SNOW_BORDER
 *   6 = TILE_GFX_WATER
 *   7 = TILE_GFX_LAVA
 *   8 = TILE_GFX_NONE
 *   9 = TILE_GFX_KOTH
 *  10 = TILE_GFX_DESERT
 *
 * Each type returns an array of variant sprites for deterministic random selection.
 */
export function getTileSpriteForGfx(gfx: number): { sprites: SpriteRect[]; animated: boolean } {
  switch (gfx) {
    case 0: return { sprites: SOLID_SPRITES, animated: false };
    case 1: return { sprites: PLAIN_SPRITES, animated: false };
    case 2: return { sprites: BORDER_SPRITES, animated: false };
    case 3: return { sprites: SNOW_SOLID_SPRITES, animated: false };
    case 4: return { sprites: SNOW_PLAIN_SPRITES, animated: false };
    case 5: return { sprites: SNOW_BORDER_SPRITES, animated: false };
    case 6: return { sprites: WATER_SPRITES, animated: true };
    case 7: return { sprites: LAVA_SPRITES, animated: true };
    case 8: return { sprites: [SOLID_SPRITES[0]], animated: false }; // NONE → solid fallback
    case 9: return { sprites: [KOTH_SPRITE], animated: false };
    case 10: return { sprites: DESERT_SPRITES, animated: false };
    default: return { sprites: [PLAIN_SPRITES[0]], animated: false };
  }
}

/** Check if a gfx value is a snow variant (snow solid=3, snow plain=4, snow border=5). */
export function isSnowGfx(gfx: number): boolean {
  return gfx >= 3 && gfx <= 5;
}
