use ply_engine::math::BoundingBox;
use ply_engine::prelude::*;

use fretboard::fretboard_position::FretboardPosition;

// The drawable snapshot: every factor is pre-interpolated by the app state
// each frame; the painter only draws.
pub struct Visual {
    pub position: FretboardPosition,
    pub in_key: bool,
    pub root_halo: bool,
    pub selected: bool,
    pub label: Option<String>,
    pub bloom: f32,
    pub wave: f32,
    pub scale: f32,
}

pub struct BoardGeom {
    pub x: f32,
    pub y: f32,
    pad_x: f32,
    pad_top: f32,
    fw: f32,
    sg: f32,
    dot_r: f32,
    string_count: usize,
    max_fret: i32,
}

const INLAY_FRETS: [i32; 7] = [3, 5, 7, 9, 12, 15, 17];

const STRING: MacroquadColor = MacroquadColor::new(0.6, 0.6, 0.63, 1.0);
const NUT: MacroquadColor = MacroquadColor::new(0.8, 0.8, 0.83, 1.0);
const FRET_LINE: MacroquadColor = MacroquadColor::new(0.31, 0.31, 0.35, 1.0);
const OFF_KEY: MacroquadColor = MacroquadColor::new(0.4, 0.4, 0.44, 1.0);
const INLAY: MacroquadColor = MacroquadColor::new(0.27, 0.27, 0.31, 1.0);
const HALO: MacroquadColor = BLACK;
const LABEL: MacroquadColor = WHITE;
const FRET_NUMBER: MacroquadColor = MacroquadColor::new(0.53, 0.53, 0.56, 1.0);

impl BoardGeom {
    pub fn new(bbox: BoundingBox, string_count: usize, max_fret: i32) -> BoardGeom {
        let pad_x = 24.0;
        let pad_top = 20.0;
        let pad_bottom = 30.0;
        let fw = (bbox.width - 2.0 * pad_x) / (max_fret as f32 + 1.0);
        BoardGeom {
            x: bbox.x,
            y: bbox.y,
            pad_x,
            pad_top,
            fw,
            sg: (bbox.height - pad_top - pad_bottom) / (string_count as f32 - 1.0),
            dot_r: (fw * 0.16).clamp(6.0, 16.0),
            string_count,
            max_fret,
        }
    }

    pub fn fret_x(&self, fret: i32) -> f32 {
        self.x + self.pad_x + fret as f32 * self.fw
    }

    pub fn center_x(&self, fret: i32) -> f32 {
        self.fret_x(fret) + self.fw * 0.5
    }

    pub fn string_y(&self, string_index: usize) -> f32 {
        self.y + self.pad_top + string_index as f32 * self.sg
    }

    pub fn hit_test(&self, position: (f32, f32)) -> Option<FretboardPosition> {
        let fret = ((position.0 - self.x - self.pad_x) / self.fw).floor() as i32;
        let string = (position.1 - self.y - self.pad_top + self.sg * 0.5) / self.sg;
        if fret < 0 || string < 0.0 {
            return None;
        }
        let string = string as usize;
        if fret > self.max_fret || string >= self.string_count {
            return None;
        }
        Some(FretboardPosition {
            string_index: string,
            fret,
        })
    }
}

fn accent(wave: f32, alpha: f32) -> MacroquadColor {
    MacroquadColor::new(1.0 * wave, 0.54 * wave, 0.24 * wave, alpha)
}

pub fn draw_board(geom: &BoardGeom, visuals: &[Visual], font: &Font) {
    let last_y = geom.string_y(geom.string_count - 1);
    let board_w = geom.fret_x(geom.max_fret) - geom.fret_x(0);

    for string_index in 0..geom.string_count {
        let y = geom.string_y(string_index);
        let thickness = if string_index == 0 || string_index + 1 == geom.string_count {
            3.0
        } else {
            1.6
        };
        draw_line(
            geom.fret_x(0),
            y,
            geom.fret_x(0) + board_w,
            y,
            thickness,
            STRING,
        );
    }

    draw_rectangle(
        geom.fret_x(0) - 3.0,
        geom.string_y(0),
        6.0,
        last_y - geom.string_y(0),
        NUT,
    );

    for fret in 1..=geom.max_fret {
        let x = geom.fret_x(fret);
        draw_rectangle(
            x,
            geom.string_y(0),
            1.0,
            last_y - geom.string_y(0),
            FRET_LINE,
        );

        let label = fret.to_string();
        let dims = measure_text(&label, Some(font), 12, 1.0);
        draw_text_ex(
            &label,
            geom.center_x(fret) - dims.width * 0.5,
            last_y + 22.0,
            TextParams {
                font: Some(font),
                font_size: 12,
                font_scale: 1.0,
                font_scale_aspect: 1.0,
                rotation: 0.0,
                color: FRET_NUMBER,
            },
        );
    }

    if geom.string_count >= 4 {
        for fret in INLAY_FRETS {
            if fret > geom.max_fret {
                break;
            }
            let x = geom.center_x(fret);
            if fret == 12 {
                draw_circle(x, geom.string_y(1) + geom.sg * 0.5, 4.0, INLAY);
                draw_circle(x, geom.string_y(3) + geom.sg * 0.5, 4.0, INLAY);
            } else {
                draw_circle(x, geom.string_y(2) + geom.sg * 0.5, 4.0, INLAY);
            }
        }
    }

    for visual in visuals {
        let x = geom.center_x(visual.position.fret);
        let y = geom.string_y(visual.position.string_index);
        let radius = geom.dot_r * (0.25 + 0.75 * visual.bloom) * visual.scale;

        if !visual.in_key {
            if radius > 0.5 {
                draw_circle_lines(x, y, radius, 1.2, OFF_KEY);
            }
            continue;
        }

        if visual.bloom > 0.01 {
            let wave = 0.4 + 0.6 * visual.wave;
            draw_circle(x, y, radius, accent(wave, visual.bloom));
            if visual.root_halo {
                draw_circle_lines(x, y, radius + 2.5, 2.0, HALO);
            }
            if visual.selected {
                draw_circle_lines(
                    x,
                    y,
                    radius + 4.0,
                    1.5,
                    MacroquadColor::new(1.0, 1.0, 1.0, 0.9 * visual.bloom),
                );
            }
            if let Some(label) = &visual.label {
                let dims = measure_text(label, Some(font), 13, 1.0);
                draw_text_ex(
                    label,
                    x - dims.width * 0.5,
                    y - dims.height * 0.5,
                    TextParams {
                        font: Some(font),
                        font_size: 13,
                        font_scale: 1.0,
                        font_scale_aspect: 1.0,
                        rotation: 0.0,
                        color: LABEL,
                    },
                );
            }
        }
    }
}
