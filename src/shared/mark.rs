// The Screen Translator mark: two Material 3 tonal tiles, a Latin "A" and
// a real 文 (from Microsoft YaHei UI), so the icon reads as translation at
// a glance.
//
// This is the single source of the logo. The settings window draws it with
// egui, the tray builds a Win32 icon from it, and `build.rs` rasterises it
// into the `.ico` embedded in the executable — all from the geometry below.
//
// Deliberately free of dependencies, and commented with `//` rather than `//!`:
// `build.rs` pulls this file in with `include!`, where an inner doc comment is
// a syntax error and an import would have to become a build dependency.

/// Primary tile — restrained Material blue.
pub const TILE_PRIMARY_RGB: [u8; 3] = [0x0B, 0x57, 0xD0];
/// Glyph on the primary tile.
pub const ON_PRIMARY_RGB: [u8; 3] = [0xFF, 0xFF, 0xFF];
/// Tonal container tile for 文.
pub const TILE_CONTAINER_RGB: [u8; 3] = [0xD3, 0xE3, 0xFD];
/// Glyph on the container tile (YaHei 文).
pub const ON_CONTAINER_RGB: [u8; 3] = [0x04, 0x1E, 0x49];

// Kept as aliases so any leftover call site still compiles.
pub const TILE_RGB: [u8; 3] = TILE_PRIMARY_RGB;
pub const GLYPH_RGB: [u8; 3] = ON_PRIMARY_RGB;

/// Side margin of the two-tile lockup, as a fraction of the canvas.
pub const TILE_MARGIN: f32 = 0.055;
/// Unscaled tile side, as a fraction of the canvas, before fitting.
const TILE_SIDE: f32 = 0.64;
/// How much the tiles overlap, as a fraction of the canvas, before fitting.
/// Kept just above `TILE_CORNER * TILE_SIDE` so the left tile's flat top and
/// bottom edges meet the right tile's rounded corners without a dip while
/// leaving generous horizontal room for `A`.
const TILE_OVERLAP: f32 = 0.14;
/// Corner radius, as a fraction of the tile side.
pub const TILE_CORNER: f32 = 0.215;
/// Tight corner at the smallest sizes so the tiles do not become circles.
pub const TILE_CORNER_SMALL: f32 = 0.20;

/// How much of the canvas half-side the painted lockup reaches.
pub fn extent() -> f32 {
    1.0 - TILE_MARGIN
}

/// A full turn still looks like rest; the working animation keeps the
/// monogram upright and only shows a progress arc while the icon is
/// breathing (scale < 1). The spin lands on this step so it can settle
/// in well under a second.
pub const SYMMETRY_TURN: f32 = 0.2;

/// Axis-aligned tile, in canvas units (0..1 before scaling to pixels).
#[derive(Debug, Clone, Copy)]
pub struct TileBox {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

impl TileBox {
    pub fn width(self) -> f32 {
        self.x1 - self.x0
    }
    pub fn height(self) -> f32 {
        self.y1 - self.y0
    }
    pub fn corner_radius(self, size: u32) -> f32 {
        let frac = if size <= 20 {
            TILE_CORNER_SMALL
        } else {
            TILE_CORNER
        };
        frac * self.width().min(self.height())
    }
}

/// Layout of the two tiles in unit-canvas space, with pixel snapping on
/// sizes 32 px and under so both tiles share the exact same snapped side.
pub fn overlap_geometry(size: u32) -> (TileBox, TileBox) {
    let total_w = 2.0 * TILE_SIDE - TILE_OVERLAP;
    let fit = (1.0 - 2.0 * TILE_MARGIN) / total_w;
    let side = TILE_SIDE * fit;
    let overlap = TILE_OVERLAP * fit;

    if size > 0 && size <= 32 {
        let s = size as f32;
        let side_px = (side * s).round().clamp(4.0, s);
        let overlap_px = (overlap * s).round().clamp(1.0, (side_px * 0.35).floor().max(1.0));
        let total_px = (2.0 * side_px - overlap_px).min(s);
        let x0_px = ((s - total_px) * 0.5).floor();
        let y0_px = ((s - side_px) * 0.5).floor();
        let rx0_px = x0_px + total_px - side_px;
        let left = TileBox {
            x0: x0_px / s,
            y0: y0_px / s,
            x1: (x0_px + side_px) / s,
            y1: (y0_px + side_px) / s,
        };
        let right = TileBox {
            x0: rx0_px / s,
            y0: y0_px / s,
            x1: (rx0_px + side_px) / s,
            y1: (y0_px + side_px) / s,
        };
        return (left, right);
    }

    let x0 = TILE_MARGIN;
    let y0 = (1.0 - side) / 2.0;
    let left = TileBox {
        x0,
        y0,
        x1: x0 + side,
        y1: y0 + side,
    };
    let right = TileBox {
        x0: x0 + side - overlap,
        y0,
        x1: x0 + side - overlap + side,
        y1: y0 + side,
    };
    (left, right)
}

pub fn glyph_pad(size: u32) -> f32 {
    if size <= 16 {
        0.12
    } else if size <= 24 {
        0.14
    } else {
        0.18
    }
}

/// Use the bolder cuts at tray and small ICO sizes.
pub fn use_bold_glyphs(size: u32) -> bool {
    size <= 48
}

const A_REG_OUTER: &[[f32; 2]] = &[
    [0.9449, 1.0000],
    [0.8152, 1.0000],
    [0.7092, 0.7197],
    [0.2852, 0.7197],
    [0.1855, 1.0000],
    [0.0551, 1.0000],
    [0.4386, 0.0000],
    [0.5600, 0.0000],
];

const A_REG_HOLE1: &[[f32; 2]] = &[
    [0.6709, 0.6144],
    [0.5139, 0.1883],
    [0.5124, 0.1838],
    [0.5109, 0.1789],
    [0.5093, 0.1735],
    [0.5078, 0.1675],
    [0.5063, 0.1611],
    [0.5047, 0.1541],
    [0.5032, 0.1467],
    [0.5017, 0.1387],
    [0.5001, 0.1303],
    [0.4986, 0.1213],
    [0.4958, 0.1213],
    [0.4944, 0.1297],
    [0.4929, 0.1376],
    [0.4914, 0.1452],
    [0.4899, 0.1525],
    [0.4883, 0.1593],
    [0.4867, 0.1659],
    [0.4850, 0.1720],
    [0.4833, 0.1778],
    [0.4816, 0.1832],
    [0.4798, 0.1883],
    [0.3243, 0.6144],
];

const A_BD_OUTER: &[[f32; 2]] = &[
    [0.9934, 1.0000],
    [0.7479, 1.0000],
    [0.6768, 0.7775],
    [0.3211, 0.7775],
    [0.2507, 1.0000],
    [0.0066, 1.0000],
    [0.3706, 0.0000],
    [0.6377, 0.0000],
];

const A_BD_HOLE1: &[[f32; 2]] = &[
    [0.6252, 0.6046],
    [0.5178, 0.2685],
    [0.5155, 0.2608],
    [0.5133, 0.2528],
    [0.5113, 0.2446],
    [0.5094, 0.2360],
    [0.5077, 0.2272],
    [0.5061, 0.2180],
    [0.5046, 0.2086],
    [0.5033, 0.1989],
    [0.5021, 0.1888],
    [0.5010, 0.1785],
    [0.4955, 0.1785],
    [0.4947, 0.1873],
    [0.4937, 0.1961],
    [0.4924, 0.2048],
    [0.4910, 0.2136],
    [0.4894, 0.2223],
    [0.4875, 0.2310],
    [0.4855, 0.2397],
    [0.4832, 0.2484],
    [0.4807, 0.2570],
    [0.4780, 0.2657],
    [0.3692, 0.6046],
];

const WEN_REG_OUTER: &[[f32; 2]] = &[
    [0.0102, 0.1800],
    [0.5010, 0.1800],
    [0.4909, 0.1609],
    [0.4813, 0.1428],
    [0.4721, 0.1258],
    [0.4633, 0.1099],
    [0.4550, 0.0951],
    [0.4472, 0.0813],
    [0.4398, 0.0686],
    [0.4328, 0.0569],
    [0.4263, 0.0463],
    [0.4202, 0.0368],
    [0.4990, 0.0000],
    [0.5050, 0.0093],
    [0.5116, 0.0198],
    [0.5187, 0.0313],
    [0.5265, 0.0440],
    [0.5348, 0.0578],
    [0.5436, 0.0726],
    [0.5531, 0.0886],
    [0.5631, 0.1057],
    [0.5737, 0.1239],
    [0.5849, 0.1431],
    [0.5112, 0.1800],
    [0.9918, 0.1800],
    [0.9918, 0.2556],
    [0.8032, 0.2556],
    [0.7910, 0.3129],
    [0.7763, 0.3678],
    [0.7590, 0.4205],
    [0.7393, 0.4708],
    [0.7170, 0.5189],
    [0.6922, 0.5647],
    [0.6650, 0.6082],
    [0.6352, 0.6494],
    [0.6028, 0.6883],
    [0.5680, 0.7249],
    [0.5968, 0.7446],
    [0.6288, 0.7638],
    [0.6640, 0.7826],
    [0.7024, 0.8010],
    [0.7440, 0.8190],
    [0.7888, 0.8366],
    [0.8368, 0.8538],
    [0.8880, 0.8706],
    [0.9424, 0.8869],
    [1.0000, 0.9029],
    [0.9908, 0.9133],
    [0.9821, 0.9234],
    [0.9738, 0.9332],
    [0.9660, 0.9426],
    [0.9586, 0.9517],
    [0.9517, 0.9604],
    [0.9452, 0.9688],
    [0.9391, 0.9768],
    [0.9336, 0.9845],
    [0.9284, 0.9918],
    [0.8723, 0.9718],
    [0.8191, 0.9516],
    [0.7690, 0.9312],
    [0.7219, 0.9106],
    [0.6778, 0.8898],
    [0.6367, 0.8689],
    [0.5986, 0.8477],
    [0.5636, 0.8264],
    [0.5316, 0.8049],
    [0.5026, 0.7832],
    [0.4733, 0.8047],
    [0.4407, 0.8263],
    [0.4049, 0.8478],
    [0.3657, 0.8694],
    [0.3232, 0.8911],
    [0.2775, 0.9128],
    [0.2284, 0.9345],
    [0.1760, 0.9563],
    [0.1203, 0.9781],
    [0.0613, 1.0000],
    [0.0560, 0.9928],
    [0.0504, 0.9854],
    [0.0447, 0.9778],
    [0.0388, 0.9701],
    [0.0327, 0.9622],
    [0.0265, 0.9541],
    [0.0201, 0.9459],
    [0.0136, 0.9375],
    [0.0069, 0.9290],
    [0.0000, 0.9202],
    [0.0570, 0.9025],
    [0.1112, 0.8843],
    [0.1624, 0.8657],
    [0.2108, 0.8467],
    [0.2563, 0.8273],
    [0.2988, 0.8075],
    [0.3385, 0.7874],
    [0.3753, 0.7668],
    [0.4092, 0.7458],
    [0.4402, 0.7244],
    [0.4042, 0.6839],
    [0.3710, 0.6419],
    [0.3405, 0.5985],
    [0.3127, 0.5537],
    [0.2876, 0.5075],
    [0.2652, 0.4600],
    [0.2456, 0.4110],
    [0.2287, 0.3606],
    [0.2144, 0.3088],
    [0.2030, 0.2556],
    [0.0102, 0.2556],
];

const WEN_REG_HOLE1: &[[f32; 2]] = &[
    [0.7157, 0.2556],
    [0.2848, 0.2556],
    [0.2952, 0.3051],
    [0.3083, 0.3527],
    [0.3239, 0.3987],
    [0.3420, 0.4429],
    [0.3627, 0.4853],
    [0.3860, 0.5260],
    [0.4118, 0.5649],
    [0.4402, 0.6021],
    [0.4711, 0.6376],
    [0.5046, 0.6713],
    [0.5366, 0.6393],
    [0.5662, 0.6052],
    [0.5934, 0.5690],
    [0.6181, 0.5307],
    [0.6405, 0.4902],
    [0.6604, 0.4475],
    [0.6778, 0.4028],
    [0.6929, 0.3558],
    [0.7055, 0.3068],
    [0.7157, 0.2556],
];

const WEN_BD_OUTER: &[[f32; 2]] = &[
    [0.0111, 0.1588],
    [0.4325, 0.1588],
    [0.4292, 0.1482],
    [0.4258, 0.1372],
    [0.4221, 0.1257],
    [0.4182, 0.1138],
    [0.4141, 0.1013],
    [0.4097, 0.0884],
    [0.4052, 0.0749],
    [0.4004, 0.0610],
    [0.3954, 0.0466],
    [0.3901, 0.0318],
    [0.5444, 0.0005],
    [0.5517, 0.0201],
    [0.5588, 0.0389],
    [0.5654, 0.0569],
    [0.5717, 0.0740],
    [0.5776, 0.0902],
    [0.5832, 0.1056],
    [0.5884, 0.1202],
    [0.5932, 0.1339],
    [0.5977, 0.1468],
    [0.6018, 0.1588],
    [0.9859, 0.1588],
    [0.9859, 0.2949],
    [0.8362, 0.2949],
    [0.8203, 0.3399],
    [0.8031, 0.3835],
    [0.7845, 0.4258],
    [0.7646, 0.4666],
    [0.7433, 0.5059],
    [0.7207, 0.5439],
    [0.6968, 0.5804],
    [0.6715, 0.6155],
    [0.6449, 0.6492],
    [0.6169, 0.6815],
    [0.6487, 0.7024],
    [0.6819, 0.7224],
    [0.7166, 0.7413],
    [0.7527, 0.7592],
    [0.7903, 0.7761],
    [0.8294, 0.7920],
    [0.8698, 0.8068],
    [0.9118, 0.8206],
    [0.9552, 0.8335],
    [1.0000, 0.8453],
    [0.9838, 0.8651],
    [0.9688, 0.8838],
    [0.9549, 0.9014],
    [0.9421, 0.9178],
    [0.9304, 0.9332],
    [0.9199, 0.9475],
    [0.9105, 0.9606],
    [0.9023, 0.9727],
    [0.8951, 0.9836],
    [0.8891, 0.9934],
    [0.8437, 0.9775],
    [0.7998, 0.9606],
    [0.7572, 0.9427],
    [0.7160, 0.9239],
    [0.6762, 0.9040],
    [0.6377, 0.8831],
    [0.6007, 0.8613],
    [0.5651, 0.8384],
    [0.5308, 0.8146],
    [0.4980, 0.7898],
    [0.4649, 0.8139],
    [0.4303, 0.8372],
    [0.3943, 0.8599],
    [0.3568, 0.8819],
    [0.3178, 0.9032],
    [0.2773, 0.9239],
    [0.2354, 0.9438],
    [0.1920, 0.9630],
    [0.1471, 0.9816],
    [0.1008, 0.9995],
    [0.0900, 0.9822],
    [0.0794, 0.9656],
    [0.0689, 0.9495],
    [0.0585, 0.9340],
    [0.0484, 0.9191],
    [0.0384, 0.9048],
    [0.0285, 0.8910],
    [0.0189, 0.8779],
    [0.0094, 0.8653],
    [0.0000, 0.8533],
    [0.0445, 0.8389],
    [0.0876, 0.8238],
    [0.1292, 0.8080],
    [0.1693, 0.7916],
    [0.2079, 0.7744],
    [0.2451, 0.7567],
    [0.2808, 0.7382],
    [0.3150, 0.7191],
    [0.3478, 0.6994],
    [0.3790, 0.6789],
    [0.3508, 0.6460],
    [0.3244, 0.6118],
    [0.2996, 0.5764],
    [0.2765, 0.5398],
    [0.2550, 0.5020],
    [0.2353, 0.4630],
    [0.2173, 0.4228],
    [0.2010, 0.3814],
    [0.1863, 0.3387],
    [0.1734, 0.2949],
    [0.0111, 0.2949],
];

const WEN_BD_HOLE1: &[[f32; 2]] = &[
    [0.6648, 0.2949],
    [0.3291, 0.2949],
    [0.3402, 0.3275],
    [0.3525, 0.3592],
    [0.3661, 0.3900],
    [0.3810, 0.4199],
    [0.3972, 0.4490],
    [0.4147, 0.4771],
    [0.4334, 0.5043],
    [0.4535, 0.5307],
    [0.4748, 0.5561],
    [0.4975, 0.5806],
    [0.5203, 0.5560],
    [0.5418, 0.5305],
    [0.5619, 0.5041],
    [0.5806, 0.4769],
    [0.5980, 0.4487],
    [0.6141, 0.4197],
    [0.6288, 0.3898],
    [0.6422, 0.3590],
    [0.6542, 0.3274],
    [0.6648, 0.2949],
];

pub struct GlyphContours {
    pub outer: &'static [[f32; 2]],
    pub hole: &'static [[f32; 2]],
}

pub fn glyph_a(bold: bool) -> GlyphContours {
    if bold {
        GlyphContours {
            outer: A_BD_OUTER,
            hole: A_BD_HOLE1,
        }
    } else {
        GlyphContours {
            outer: A_REG_OUTER,
            hole: A_REG_HOLE1,
        }
    }
}

pub fn glyph_wen(bold: bool) -> GlyphContours {
    if bold {
        GlyphContours {
            outer: WEN_BD_OUTER,
            hole: WEN_BD_HOLE1,
        }
    } else {
        GlyphContours {
            outer: WEN_REG_OUTER,
            hole: WEN_REG_HOLE1,
        }
    }
}

/// Maps a unit-square glyph centred inside `box` at an explicit `side` so both
/// glyphs share identical cap height and baseline.
fn map_glyph_centered(
    glyph: GlyphContours,
    bx0: f32,
    by0: f32,
    bx1: f32,
    by1: f32,
    side: f32,
) -> (Vec<[f32; 2]>, Vec<[f32; 2]>) {
    let ox = (bx0 + bx1 - side) * 0.5;
    let oy = (by0 + by1 - side) * 0.5;
    let map = |pts: &[[f32; 2]]| -> Vec<[f32; 2]> {
        pts.iter()
            .map(|p| [ox + p[0] * side, oy + p[1] * side])
            .collect()
    };
    (map(glyph.outer), map(glyph.hole))
}

/// Renders the mark into `size × size` straight-alpha RGBA.
pub fn rasterise(size: u32) -> Vec<u8> {
    rasterise_with(size, 0.0, 1.0)
}

/// Renders the mark scaled by `scale`. `turns` rotates a progress arc that
/// only appears while the icon is breathing (`scale` below 1), so the
/// monogram never tips on its side and a settled frame always looks like rest.
pub fn rasterise_with(size: u32, turns: f32, scale: f32) -> Vec<u8> {
    let sub: u32 = if size <= 48 { 6 } else { 4 };
    let canvas = size as f32;
    let scale = scale.clamp(0.2, 1.0);
    let origin = canvas * (1.0 - scale) * 0.5;
    let unit = canvas * scale;

    let (left_u, right_u) = overlap_geometry(size);
    let to_px = |t: TileBox| TileBox {
        x0: origin + t.x0 * unit,
        y0: origin + t.y0 * unit,
        x1: origin + t.x1 * unit,
        y1: origin + t.y1 * unit,
    };
    let left = to_px(left_u);
    let right = to_px(right_u);
    let pad = glyph_pad(size);
    let bold = use_bold_glyphs(size);

    // Centre `A` strictly within the visible portion of the left tile
    // (`left.x0 .. right.x0`) and `文` within the full right tile (which sits
    // completely on top), sharing one optical height so both align cleanly.
    let visible_left_w = (right.x0 - left.x0).max(1.0);
    let x_pad = if size <= 24 { 0.09 } else { 0.115 };
    let glyph_side = (left.height() * (1.0 - 2.0 * pad)).min(visible_left_w * (1.0 - 2.0 * x_pad));
    let (a_outer, a_hole) =
        map_glyph_centered(glyph_a(bold), left.x0, left.y0, right.x0, left.y1, glyph_side);
    let (w_outer, w_hole) = map_glyph_centered(
        glyph_wen(bold),
        right.x0,
        right.y0,
        right.x1,
        right.y1,
        glyph_side,
    );

    let show_arc = scale < 0.995 && size >= 24;
    let cx = canvas * 0.5;
    let cy = canvas * 0.5;
    let arc_r = (unit * 0.5 - unit * TILE_MARGIN * 0.35).min(canvas * 0.5 - 0.6);
    let arc_half = (unit * 0.03).max(1.0);
    let arc_start = turns * std::f32::consts::TAU;
    let arc_sweep = std::f32::consts::TAU * 0.62;

    let left_r = left.corner_radius(size) * scale;
    let right_r = right.corner_radius(size) * scale;

    let mut out = vec![0u8; (size as usize) * (size as usize) * 4];
    let total = (sub * sub) as f32;

    for py in 0..size {
        for px in 0..size {
            let mut rgb = [0.0_f32; 3];
            let mut covered = 0u32;
            for sy in 0..sub {
                for sx in 0..sub {
                    let x = px as f32 + (sx as f32 + 0.5) / sub as f32;
                    let y = py as f32 + (sy as f32 + 0.5) / sub as f32;
                    let sample = if in_rounded_rect(x, y, right.x0, right.y0, right.x1, right.y1, right_r) {
                        if inside_glyph(&w_outer, &w_hole, x, y) {
                            Some(ON_CONTAINER_RGB)
                        } else {
                            Some(TILE_CONTAINER_RGB)
                        }
                    } else if in_rounded_rect(x, y, left.x0, left.y0, left.x1, left.y1, left_r) {
                        if inside_glyph(&a_outer, &a_hole, x, y) {
                            Some(ON_PRIMARY_RGB)
                        } else {
                            Some(TILE_PRIMARY_RGB)
                        }
                    } else if show_arc && on_arc(x, y, cx, cy, arc_r, arc_half, arc_start, arc_sweep) {
                        Some(TILE_PRIMARY_RGB)
                    } else {
                        None
                    };
                    if let Some(col) = sample {
                        covered += 1;
                        rgb[0] += col[0] as f32;
                        rgb[1] += col[1] as f32;
                        rgb[2] += col[2] as f32;
                    }
                }
            }

            if covered > 0 {
                let idx = ((py as usize) * (size as usize) + px as usize) * 4;
                let cov = covered as f32;
                out[idx] = (rgb[0] / cov).round() as u8;
                out[idx + 1] = (rgb[1] / cov).round() as u8;
                out[idx + 2] = (rgb[2] / cov).round() as u8;
                out[idx + 3] = ((cov / total) * 255.0).round() as u8;
            }
        }
    }
    out
}

fn in_rounded_rect(x: f32, y: f32, x0: f32, y0: f32, x1: f32, y1: f32, r: f32) -> bool {
    if x < x0 || x > x1 || y < y0 || y > y1 {
        return false;
    }
    let r = r.min((x1 - x0).abs() * 0.5).min((y1 - y0).abs() * 0.5);
    let cx = x.clamp(x0 + r, x1 - r);
    let cy = y.clamp(y0 + r, y1 - r);
    let dx = x - cx;
    let dy = y - cy;
    dx * dx + dy * dy <= r * r
}

fn inside_glyph(outer: &[[f32; 2]], hole: &[[f32; 2]], x: f32, y: f32) -> bool {
    winding(outer, x, y) != 0 && winding(hole, x, y) == 0
}

fn winding(poly: &[[f32; 2]], x: f32, y: f32) -> i32 {
    if poly.len() < 3 {
        return 0;
    }
    let mut wn = 0i32;
    for i in 0..poly.len() {
        let (x1, y1) = (poly[i][0], poly[i][1]);
        let (x2, y2) = (poly[(i + 1) % poly.len()][0], poly[(i + 1) % poly.len()][1]);
        if y1 <= y {
            if y2 > y && is_left(x1, y1, x2, y2, x, y) > 0.0 {
                wn += 1;
            }
        } else if y2 <= y && is_left(x1, y1, x2, y2, x, y) < 0.0 {
            wn -= 1;
        }
    }
    wn
}

fn is_left(x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) -> f32 {
    (x2 - x1) * (y - y1) - (x - x1) * (y2 - y1)
}

fn on_arc(
    x: f32,
    y: f32,
    cx: f32,
    cy: f32,
    radius: f32,
    half: f32,
    start: f32,
    sweep: f32,
) -> bool {
    let dx = x - cx;
    let dy = y - cy;
    let r = (dx * dx + dy * dy).sqrt();
    if (r - radius).abs() > half {
        return false;
    }
    let mut ang = dy.atan2(dx) - start;
    while ang < 0.0 {
        ang += std::f32::consts::TAU;
    }
    while ang >= std::f32::consts::TAU {
        ang -= std::f32::consts::TAU;
    }
    ang <= sweep
}

#[allow(dead_code)]
pub const ICON_SIZES: [u32; 9] = [16, 20, 24, 32, 40, 48, 64, 128, 256];

#[cfg(test)]
mod tests {
    use super::*;

    fn pixel(buf: &[u8], size: u32, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * size + x) * 4) as usize;
        [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]
    }

    fn close_rgb(px: [u8; 4], rgb: [u8; 3], tol: u8) -> bool {
        (0..3).all(|c| px[c].abs_diff(rgb[c]) <= tol)
    }

    #[test]
    fn the_corners_are_transparent() {
        for size in ICON_SIZES {
            let buf = rasterise(size);
            for (x, y) in [(0, 0), (size - 1, 0), (0, size - 1), (size - 1, size - 1)] {
                assert_eq!(
                    pixel(&buf, size, x, y)[3],
                    0,
                    "corner ({x},{y}) of {size}px is not clear"
                );
            }
        }
    }

    #[test]
    fn the_left_tile_is_primary_blue() {
        let size = 64;
        let buf = rasterise(size);
        let (left, _) = overlap_geometry(size);
        // A point in the left tile, above the A.
        let x = ((left.x0 + left.width() * 0.30) * size as f32).round() as u32;
        let y = ((left.y0 + left.height() * 0.12) * size as f32).round() as u32;
        let px = pixel(&buf, size, x, y);
        assert_eq!(px[3], 255, "left tile sample should be opaque, got {px:?}");
        assert!(
            close_rgb(px, TILE_PRIMARY_RGB, 8),
            "left tile should be primary blue, got {px:?}"
        );
    }

    #[test]
    fn the_right_tile_is_the_tonal_container() {
        let size = 64;
        let buf = rasterise(size);
        let (_, right) = overlap_geometry(size);
        let x = ((right.x0 + right.width() * 0.80) * size as f32).round() as u32;
        let y = ((right.y0 + right.height() * 0.12) * size as f32).round() as u32;
        let px = pixel(&buf, size, x, y);
        assert_eq!(px[3], 255, "right tile sample should be opaque, got {px:?}");
        assert!(
            close_rgb(px, TILE_CONTAINER_RGB, 8),
            "right tile should be the container tint, got {px:?}"
        );
    }

    #[test]
    fn the_a_and_wen_are_present_and_separated() {
        let size = 64;
        let buf = rasterise(size);
        let mut a_xs = Vec::new();
        let mut w_xs = Vec::new();
        for y in 0..size {
            for x in 0..size {
                let px = pixel(&buf, size, x, y);
                if px[3] < 200 {
                    continue;
                }
                if close_rgb(px, ON_PRIMARY_RGB, 30) {
                    a_xs.push(x);
                }
                if close_rgb(px, ON_CONTAINER_RGB, 30) {
                    w_xs.push(x);
                }
            }
        }
        assert!(!a_xs.is_empty(), "the Latin A was not found");
        assert!(!w_xs.is_empty(), "the 文 glyph was not found");
        let a_max = *a_xs.iter().max().unwrap();
        let w_min = *w_xs.iter().min().unwrap();
        assert!(
            a_max < w_min,
            "A and 文 collide or are in the wrong order: A max x={a_max}, 文 min x={w_min}"
        );
        // A gap of at least a pixel keeps the two letters distinct.
        assert!(
            w_min - a_max >= 1,
            "A and 文 touch: A max x={a_max}, 文 min x={w_min}"
        );
    }

    #[test]
    fn overlap_seam_stays_fully_opaque() {
        // Previously pixels straddling the right tile's left edge dropped the
        // left tile's subsamples and became semi-transparent, showing a dark
        // or light crack along the seam.
        let size = 128;
        let buf = rasterise(size);
        let (_, right) = overlap_geometry(size);
        let seam_x = (right.x0 * size as f32).round() as u32;
        let y_start = ((right.y0 + right.height() * 0.25) * size as f32).round() as u32;
        let y_end = ((right.y0 + right.height() * 0.75) * size as f32).round() as u32;
        for y in y_start..=y_end {
            for dx in -1_i32..=1 {
                let x = (seam_x as i32 + dx) as u32;
                let a = pixel(&buf, size, x, y)[3];
                assert_eq!(
                    a, 255,
                    "seam pixel ({x},{y}) is semi-transparent (alpha={a})"
                );
            }
        }
    }

    #[test]
    fn glyphs_are_symmetrically_centred_in_their_visible_tiles() {
        let size = 256;
        let buf = rasterise(size);
        let (left, right) = overlap_geometry(size);
        let mut a_min = size;
        let mut a_max = 0;
        let mut w_min = size;
        let mut w_max = 0;
        for y in 0..size {
            for x in 0..size {
                let px = pixel(&buf, size, x, y);
                if px[3] < 200 {
                    continue;
                }
                if close_rgb(px, ON_PRIMARY_RGB, 30) {
                    a_min = a_min.min(x);
                    a_max = a_max.max(x);
                }
                if close_rgb(px, ON_CONTAINER_RGB, 30) {
                    w_min = w_min.min(x);
                    w_max = w_max.max(x);
                }
            }
        }
        let lx0 = left.x0 * size as f32;
        let rx0 = right.x0 * size as f32;
        let rx1 = right.x1 * size as f32;
        let a_left_margin = a_min as f32 - lx0;
        let a_right_margin = rx0 - a_max as f32;
        assert!(
            a_right_margin >= 10.0,
            "A sits too close to the right tile: right margin = {a_right_margin:.1}px"
        );
        assert!(
            (a_left_margin - a_right_margin).abs() <= 2.5,
            "A is off-centre in visible left tile: left={a_left_margin:.1}, right={a_right_margin:.1}"
        );
        let w_left_margin = w_min as f32 - rx0;
        let w_right_margin = rx1 - w_max as f32;
        assert!(
            (w_left_margin - w_right_margin).abs() <= 2.5,
            "文 is off-centre in right tile: left={w_left_margin:.1}, right={w_right_margin:.1}"
        );
    }

    #[test]
    fn every_icon_size_renders_something() {
        for size in ICON_SIZES {
            let buf = rasterise(size);
            assert_eq!(buf.len(), (size * size * 4) as usize);
            let opaque = buf.chunks_exact(4).filter(|p| p[3] > 128).count();
            let total = (size * size) as usize;
            assert!(
                opaque > total / 5,
                "{size}px is nearly empty: {opaque}/{total}"
            );
            assert!(opaque < total, "{size}px has no margin at all");
        }
    }

    #[test]
    fn a_settled_spin_frame_matches_rest() {
        let a = rasterise_with(48, 0.0, 1.0);
        let b = rasterise_with(48, SYMMETRY_TURN, 1.0);
        assert_eq!(a, b);
    }

    #[test]
    fn a_busy_arc_actually_moves() {
        let a = rasterise_with(48, 0.0, 0.9);
        let b = rasterise_with(48, 0.1, 0.9);
        assert_ne!(a, b);
    }

    #[test]
    fn scaling_down_shrinks_the_covered_area() {
        let full = rasterise_with(48, 0.0, 1.0);
        let small = rasterise_with(48, 0.0, 0.7);
        let count = |b: &[u8]| b.chunks_exact(4).filter(|p| p[3] > 128).count();
        assert!(count(&small) < count(&full));
        assert!(count(&small) > 0);
    }

    #[test]
    fn the_mark_stays_inside_its_box() {
        assert!(extent() <= 1.0);
        assert!(TILE_MARGIN > 0.0);
    }

    #[test]
    fn glyph_and_tile_pairs_are_readable() {
        let lin = |v: u8| {
            let v = v as f32 / 255.0;
            if v <= 0.03928 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        let lum = |c: [u8; 3]| 0.2126 * lin(c[0]) + 0.7152 * lin(c[1]) + 0.0722 * lin(c[2]);
        let contrast = |a: [u8; 3], b: [u8; 3]| {
            let (x, y) = (lum(a), lum(b));
            let (hi, lo) = if x > y { (x, y) } else { (y, x) };
            (hi + 0.05) / (lo + 0.05)
        };
        let a = contrast(ON_PRIMARY_RGB, TILE_PRIMARY_RGB);
        let w = contrast(ON_CONTAINER_RGB, TILE_CONTAINER_RGB);
        assert!(a >= 4.5, "A on primary contrast {a:.2}");
        assert!(w >= 4.5, "文 on container contrast {w:.2}");
    }

    #[test]
    fn export_preview_when_asked() {
        let Ok(dir) = std::env::var("ICON_PREVIEW_DIR") else {
            return;
        };
        let dir = std::path::PathBuf::from(dir);
        std::fs::create_dir_all(&dir).expect("preview dir");
        for size in ICON_SIZES {
            let buf = rasterise(size);
            let path = dir.join(format!("icon-{size}.png"));
            image::save_buffer(&path, &buf, size, size, image::ExtendedColorType::Rgba8)
                .unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
        }
        if let Ok(assets) = std::env::var("ICON_ASSETS_PNG") {
            let buf = rasterise(256);
            image::save_buffer(assets, &buf, 256, 256, image::ExtendedColorType::Rgba8)
                .expect("assets/icon.png");
        }
    }
}
