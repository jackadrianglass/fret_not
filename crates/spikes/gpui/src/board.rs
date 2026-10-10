use std::cell::RefCell;
use std::rc::Rc;

use gpui::{App, Bounds, Hsla, Pixels, Point, Size, Window, px};

use fretboard::fretboard_position::FretboardPosition;

pub fn accent() -> Hsla {
    gpui::hsla(27.0 / 360.0, 1.0, 0.62, 1.0)
}

fn accent_wave(wave: f32, alpha: f32) -> Hsla {
    gpui::hsla(27.0 / 360.0, 1.0, 0.24 + 0.38 * wave, alpha)
}

fn string_color() -> Hsla {
    gpui::hsla(0.0, 0.0, 0.61, 1.0)
}
fn nut_color() -> Hsla {
    gpui::hsla(0.0, 0.0, 0.81, 1.0)
}
fn fret_line_color() -> Hsla {
    gpui::hsla(0.0, 0.0, 0.33, 1.0)
}
fn off_key_color() -> Hsla {
    gpui::hsla(0.0, 0.0, 0.42, 1.0)
}
fn inlay_color() -> Hsla {
    gpui::hsla(0.0, 0.0, 0.29, 1.0)
}
fn halo_color() -> Hsla {
    gpui::black()
}

const INLAY_FRETS: [i32; 7] = [3, 5, 7, 9, 12, 15, 17];

// Everything the paint pass and the overlay elements need to agree on,
// rebuilt from the canvas bounds on every paint. The overlay reads the
// previous frame's geometry — one frame of staleness, invisible in
// practice, and the reason hover/click hit boxes are overlay divs instead
// of raw mouse math.
#[derive(Clone, Copy)]
pub struct BoardGeom {
    pub origin: Point<Pixels>,
    pub pad_x: f32,
    pub pad_top: f32,
    pub fw: f32,
    pub sg: f32,
    pub dot_r: f32,
    pub string_count: usize,
    pub max_fret: i32,
}

impl BoardGeom {
    pub fn new(bounds: Bounds<Pixels>, string_count: usize, max_fret: i32) -> BoardGeom {
        let width = f32::from(bounds.size.width);
        let height = f32::from(bounds.size.height);
        let pad_x = 24.0;
        let pad_top = 20.0;
        let pad_bottom = 30.0;
        let fw = (width - 2.0 * pad_x) / (max_fret as f32 + 1.0);
        BoardGeom {
            origin: bounds.origin,
            pad_x,
            pad_top,
            fw,
            sg: (height - pad_top - pad_bottom) / (string_count as f32 - 1.0),
            dot_r: (fw * 0.16).clamp(6.0, 16.0),
            string_count,
            max_fret,
        }
    }

    pub fn fret_x(&self, fret: i32) -> Pixels {
        self.origin.x + px(self.pad_x + fret as f32 * self.fw)
    }

    pub fn center_x(&self, fret: i32) -> Pixels {
        self.fret_x(fret) + px(self.fw * 0.5)
    }

    pub fn string_y(&self, string_index: usize) -> Pixels {
        self.origin.y + px(self.pad_top + string_index as f32 * self.sg)
    }

    // The overlay divs are absolutely positioned inside the board element,
    // so their coordinates are parent-relative, while paint coordinates
    // are window-space — these strip the origin.
    pub fn local_center_x(&self, fret: i32) -> f32 {
        self.pad_x + (fret as f32 + 0.5) * self.fw
    }

    pub fn local_string_y(&self, string_index: usize) -> f32 {
        self.pad_top + string_index as f32 * self.sg
    }
}

fn rect(x: Pixels, y: Pixels, w: Pixels, h: Pixels) -> Bounds<Pixels> {
    Bounds {
        origin: Point::new(x, y),
        size: Size {
            width: w,
            height: h,
        },
    }
}

fn quad_at(window: &mut Window, x: Pixels, y: Pixels, w: Pixels, h: Pixels, color: Hsla) {
    window.paint_quad(gpui::quad(
        rect(x, y, w, h),
        px(0.),
        color,
        px(0.),
        gpui::black(),
        gpui::BorderStyle::Solid,
    ));
}

fn circle(window: &mut Window, x: Pixels, y: Pixels, r: Pixels, color: Hsla) {
    window.paint_quad(gpui::quad(
        rect(x - r, y - r, r * 2., r * 2.),
        r,
        color,
        px(0.),
        gpui::black(),
        gpui::BorderStyle::Solid,
    ));
}

fn ring(window: &mut Window, x: Pixels, y: Pixels, r: Pixels, width: Pixels, color: Hsla) {
    window.paint_quad(gpui::quad(
        rect(x - r, y - r, r * 2., r * 2.),
        r,
        gpui::transparent_black(),
        width,
        color,
        gpui::BorderStyle::Solid,
    ));
}

// One drawable position with the non-time visual factors pre-interpolated
// by the view. Bloom is computed inside the paint pass from the shared
// with_animation delta, because the animation repaints without re-running
// render.
pub struct Visual {
    pub position: FretboardPosition,
    pub in_key: bool,
    pub root_halo: bool,
    pub selected: bool,
    pub wave: f32,
    pub scale: f32,
}

pub struct BoardSnapshot {
    pub visuals: Vec<Visual>,
    pub string_count: usize,
    pub max_fret: i32,
    pub geom: Rc<RefCell<Option<BoardGeom>>>,
    pub bloom_delta: Rc<RefCell<f32>>,
}

const BLOOM_MS: f32 = 250.0;
const BLOOM_FRET_STAGGER_MS: f32 = 18.0;
const BLOOM_ANIMATION_MS: f32 = 700.0;

fn bloom_of(delta: f32, fret: i32) -> f32 {
    let elapsed = delta * BLOOM_ANIMATION_MS - fret as f32 * BLOOM_FRET_STAGGER_MS;
    (elapsed / BLOOM_MS).clamp(0.0, 1.0)
}

pub fn paint_board(
    snapshot: &BoardSnapshot,
    bounds: Bounds<Pixels>,
    window: &mut Window,
    _cx: &mut App,
) {
    let geom = BoardGeom::new(bounds, snapshot.string_count, snapshot.max_fret);
    *snapshot.geom.borrow_mut() = Some(geom);

    let last_y = geom.string_y(geom.string_count - 1);
    let board_w = geom.fret_x(geom.max_fret) - geom.fret_x(0);

    for string_index in 0..geom.string_count {
        let y = geom.string_y(string_index);
        let thickness = if string_index == 0 || string_index + 1 == geom.string_count {
            px(3.)
        } else {
            px(1.6)
        };
        quad_at(
            window,
            geom.fret_x(0),
            y - thickness / 2.,
            board_w,
            thickness,
            string_color(),
        );
    }

    quad_at(
        window,
        geom.fret_x(0) - px(3.),
        geom.string_y(0),
        px(6.),
        last_y - geom.string_y(0),
        nut_color(),
    );

    for fret in 1..=geom.max_fret {
        quad_at(
            window,
            geom.fret_x(fret),
            geom.string_y(0),
            px(1.),
            last_y - geom.string_y(0),
            fret_line_color(),
        );
    }

    if geom.string_count >= 4 {
        for fret in INLAY_FRETS {
            if fret > geom.max_fret {
                break;
            }
            let x = geom.center_x(fret);
            if fret == 12 {
                circle(
                    window,
                    x,
                    geom.string_y(1) + px(geom.sg * 0.5),
                    px(4.),
                    inlay_color(),
                );
                circle(
                    window,
                    x,
                    geom.string_y(3) + px(geom.sg * 0.5),
                    px(4.),
                    inlay_color(),
                );
            } else {
                circle(
                    window,
                    x,
                    geom.string_y(2) + px(geom.sg * 0.5),
                    px(4.),
                    inlay_color(),
                );
            }
        }
    }

    let bloom_delta = *snapshot.bloom_delta.borrow();

    for v in &snapshot.visuals {
        let x = geom.center_x(v.position.fret);
        let y = geom.string_y(v.position.string_index);
        let bloom = bloom_of(bloom_delta, v.position.fret);
        let radius = geom.dot_r * (0.25 + 0.75 * bloom) * v.scale;

        if !v.in_key {
            if radius > 0.5 {
                ring(window, x, y, px(radius), px(1.2), off_key_color());
            }
            continue;
        }

        if bloom > 0.01 {
            circle(window, x, y, px(radius), accent_wave(v.wave, bloom));
            if v.root_halo {
                ring(window, x, y, px(radius + 2.5), px(2.), halo_color());
            }
            if v.selected {
                ring(window, x, y, px(radius + 4.0), px(1.5), gpui::white());
            }
        }
    }
}
