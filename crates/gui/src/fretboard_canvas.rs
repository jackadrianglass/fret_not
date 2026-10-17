use iced::mouse;
use iced::widget::canvas::{self, Action};
use iced::widget::text::Alignment;
use iced::{Point, Rectangle, Renderer, Theme};

use fretboard::fretboard_position::FretboardPosition;

use crate::board;
use crate::colors::CanvasColors;

const INLAY_FRETS: [i32; 7] = [3, 5, 7, 9, 12, 15, 17];
const LABEL_SIZE: f32 = 12.0;
const SUBSCRIPT_SIZE: f32 = 8.0;
// Where the subscript's center sits relative to the dot's center: at the
// main glyph's lower right, derived from the label's font size.
const SUBSCRIPT_OFFSET: (f32, f32) = (LABEL_SIZE * 0.33, LABEL_SIZE * 0.35);

// The ink box a glyph occupies at [size]: digits and letters render about
// 0.62·size wide and 0.72·size tall.
fn glyph_half_extents(size: f32, chars: f32) -> (f32, f32) {
    (0.31 * size * chars, 0.36 * size)
}

// One drawn position; the board component builds these, the canvas only paints.
pub struct Visual {
    pub position: FretboardPosition,
    pub label: Option<String>,
    // The key-referenced octave count, drawn as a small subscript beside
    // the degree label — one count per cycle of 7 degrees.
    pub octave: Option<i32>,
    // An off-key position: a faint hollow ring until played, labeled with
    // its spelled degree when emphasized.
    pub accidental: bool,
    pub root_halo: bool,
    pub selected: bool,
    pub emphasized: bool,
}

pub struct FretboardCanvas {
    pub visuals: Vec<Visual>,
    pub string_count: usize,
    pub max_fret: i32,
    // The open-note SPN label per string, drawn at the left of its row.
    pub open_notes: Vec<String>,
}

struct Geometry {
    left: f32,
    top: f32,
    rows: usize,
    fret_width: f32,
    string_gap: f32,
    dot_radius: f32,
}

impl Geometry {
    fn new(bounds: Rectangle, string_count: usize, max_fret: i32, content_radius: f32) -> Geometry {
        // The left pad hosts the open-string dots, which sit left of the nut.
        let pad_x = 48.0;
        let pad_right = 24.0;
        let pad_top = 20.0;
        let pad_bottom = 30.0;
        let fret_width = (bounds.width - pad_x - pad_right) / (max_fret as f32 + 1.0);
        let string_gap = if string_count > 1 {
            (bounds.height - pad_top - pad_bottom) / (string_count - 1) as f32
        } else {
            0.0
        };
        Geometry {
            left: pad_x,
            top: pad_top,
            rows: string_count,
            fret_width,
            string_gap,
            // The rim must clear the label stack even when the fret spacing
            // alone would allow a smaller dot.
            dot_radius: (fret_width * 0.22).clamp(7.0, 14.0).max(content_radius),
        }
    }

    fn fret_x(&self, fret: i32) -> f32 {
        self.left + fret as f32 * self.fret_width
    }

    // The open string lives just left of the nut, the way it does past the
    // nut on a real instrument. A fretted note at fret N sounds between the
    // N-1 and N wires — that is the space the finger presses in — so its
    // dot centers between those wires, not the pair to the right.
    fn center_x(&self, fret: i32) -> f32 {
        if fret == 0 {
            self.fret_x(0) - self.dot_radius - 4.0
        } else {
            self.fret_x(fret) - self.fret_width * 0.5
        }
    }

    // Tab convention: the highest string is the top row, so [string_index]
    // (numbered low to high) counts the rows from the bottom up.
    fn string_y(&self, string_index: usize) -> f32 {
        self.top + (self.rows - 1 - string_index) as f32 * self.string_gap
    }

    fn center(&self, position: FretboardPosition) -> Point {
        Point::new(
            self.center_x(position.fret),
            self.string_y(position.string_index),
        )
    }
}

impl FretboardCanvas {
    // The radius floor that keeps every dot's content inside the rim: the
    // label centered, plus the octave subscript where one is present. The
    // half-extents of the stacked content box, corner to rim.
    fn content_radius(&self) -> f32 {
        self.visuals
            .iter()
            .filter_map(|v| {
                v.label
                    .as_ref()
                    .map(|label| (v.octave.is_some(), label.len() as f32))
            })
            .map(|(subscripted, chars)| {
                let (main_w, main_h) = glyph_half_extents(LABEL_SIZE, chars);
                let (w, h) = match subscripted {
                    true => {
                        let (sub_w, sub_h) = glyph_half_extents(SUBSCRIPT_SIZE, 1.0);
                        (
                            main_w.max(SUBSCRIPT_OFFSET.0 + sub_w),
                            main_h.max(SUBSCRIPT_OFFSET.1 + sub_h),
                        )
                    }
                    false => (main_w, main_h),
                };
                (w * w + h * h).sqrt()
            })
            .fold(0.0, f32::max)
    }

    fn geometry(&self, bounds: &Rectangle) -> Geometry {
        Geometry::new(
            *bounds,
            self.string_count,
            self.max_fret,
            self.content_radius(),
        )
    }

    fn hit_test(&self, bounds: &Rectangle, cursor: mouse::Cursor) -> Option<FretboardPosition> {
        let point = cursor.position_in(*bounds)?;
        let g = self.geometry(bounds);
        self.visuals
            .iter()
            .map(|v| (g.center(v.position), v.position))
            .map(|(center, position)| {
                let dx = center.x - point.x;
                let dy = center.y - point.y;
                (dx * dx + dy * dy, position)
            })
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .filter(|(dist2, _)| *dist2 <= (g.dot_radius * 1.7).powi(2))
            .map(|(_, position)| position)
    }

    fn draw_grid(&self, frame: &mut canvas::Frame, g: &Geometry, c: &CanvasColors) {
        let last = self.string_count.saturating_sub(1);
        for string_index in 0..self.string_count {
            let y = g.string_y(string_index);
            let width = if string_index == 0 || string_index == last {
                3.0
            } else {
                1.6
            };
            frame.stroke(
                &canvas::Path::line(
                    Point::new(g.fret_x(0), y),
                    Point::new(g.fret_x(self.max_fret), y),
                ),
                canvas::Stroke::default()
                    .with_color(c.string)
                    .with_width(width),
            );
        }

        let nut = canvas::Path::line(
            Point::new(g.fret_x(0), g.string_y(0)),
            Point::new(g.fret_x(0), g.string_y(last)),
        );
        frame.stroke(
            &nut,
            canvas::Stroke::default().with_color(c.nut).with_width(6.0),
        );

        for fret in 1..=self.max_fret {
            let x = g.fret_x(fret);
            let line = canvas::Path::line(
                Point::new(x, g.string_y(0)),
                Point::new(x, g.string_y(last)),
            );
            frame.stroke(
                &line,
                canvas::Stroke::default()
                    .with_color(c.fret_line)
                    .with_width(1.0),
            );
        }

        if self.string_count >= 4 {
            for fret in INLAY_FRETS {
                if fret > self.max_fret {
                    break;
                }
                let x = g.center_x(fret);
                if fret == 12 {
                    let upper = (g.string_y(1) + g.string_y(2)) * 0.5;
                    let lower = (g.string_y(3) + g.string_y(4)) * 0.5;
                    frame.fill(&canvas::Path::circle(Point::new(x, upper), 4.0), c.inlay);
                    frame.fill(&canvas::Path::circle(Point::new(x, lower), 4.0), c.inlay);
                } else {
                    let middle = (g.string_y(2) + g.string_y(3)) * 0.5;
                    frame.fill(&canvas::Path::circle(Point::new(x, middle), 4.0), c.inlay);
                }
            }
        }

        // Below the lowest drawn row — string 0 under tab order.
        let number_y = g.string_y(0) + 18.0;
        for fret in 1..=self.max_fret {
            frame.fill_text(canvas::Text {
                content: fret.to_string(),
                position: Point::new(g.center_x(fret), number_y),
                color: c.fret_number,
                size: 11.0.into(),
                align_x: Alignment::Center,
                align_y: iced::alignment::Vertical::Center,
                ..canvas::Text::default()
            });
        }

        for string_index in 0..self.string_count {
            if let Some(note) = self.open_notes.get(string_index) {
                frame.fill_text(canvas::Text {
                    content: note.clone(),
                    position: Point::new(4.0, g.string_y(string_index)),
                    color: c.fret_number,
                    size: 11.0.into(),
                    align_x: Alignment::Left,
                    align_y: iced::alignment::Vertical::Center,
                    ..canvas::Text::default()
                });
            }
        }
    }

    // The centered degree label with its octave count drawn as a small
    // subscript beside it — the count of 7-degree cycles above the tonic.
    fn draw_label(&self, frame: &mut canvas::Frame, g: &Geometry, c: &CanvasColors, v: &Visual) {
        let center = g.center(v.position);
        let color = if v.emphasized { c.label } else { c.field_label };
        if let Some(label) = &v.label {
            frame.fill_text(canvas::Text {
                content: label.clone(),
                position: center,
                color,
                size: LABEL_SIZE.into(),
                align_x: Alignment::Center,
                align_y: iced::alignment::Vertical::Center,
                ..canvas::Text::default()
            });
        }
        if let Some(octave) = v.octave {
            frame.fill_text(canvas::Text {
                content: octave.to_string(),
                position: Point::new(center.x + SUBSCRIPT_OFFSET.0, center.y + SUBSCRIPT_OFFSET.1),
                color,
                size: SUBSCRIPT_SIZE.into(),
                align_x: Alignment::Center,
                align_y: iced::alignment::Vertical::Center,
                ..canvas::Text::default()
            });
        }
    }

    fn draw_dot(&self, frame: &mut canvas::Frame, g: &Geometry, c: &CanvasColors, v: &Visual) {
        let center = g.center(v.position);
        let radius = g.dot_radius;

        let path = canvas::Path::circle(center, radius);

        if !v.emphasized {
            if v.accidental {
                // The chromatic field as a whisper: a faint hollow ring,
                // unlabeled, keeping the key's labeled dots the primary
                // display.
                frame.stroke(
                    &path,
                    canvas::Stroke::default()
                        .with_color(c.fret_line)
                        .with_width(1.0),
                );
                return;
            }
            // A solid fill masks the string under the dot, so the label
            // never has a line striking through it.
            frame.fill(&path, c.background);
            frame.stroke(
                &path,
                canvas::Stroke::default()
                    .with_color(c.field_label)
                    .with_width(1.2),
            );
            self.draw_label(frame, g, c, v);
            return;
        }

        frame.fill(&path, c.accent);

        if v.root_halo {
            frame.stroke(
                &canvas::Path::circle(center, radius + 2.5),
                canvas::Stroke::default()
                    .with_color(c.root_halo)
                    .with_width(2.0),
            );
        }

        if v.selected {
            frame.stroke(
                &canvas::Path::circle(center, radius + 4.0),
                canvas::Stroke::default()
                    .with_color(c.label)
                    .with_width(1.5),
            );
        }

        self.draw_label(frame, g, c, v);
    }
}

impl canvas::Program<board::Event, Theme, Renderer> for FretboardCanvas {
    type State = ();

    fn update(
        &self,
        _state: &mut (),
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<board::Event>> {
        match event {
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => self
                .hit_test(&bounds, cursor)
                .map(|position| Action::publish(board::Event::Clicked(position))),
            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let g = self.geometry(&bounds);
        let c = CanvasColors::of(theme);

        self.draw_grid(&mut frame, &g, &c);
        for v in &self.visuals {
            self.draw_dot(&mut frame, &g, &c, v);
        }

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _state: &(),
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if self.hit_test(&bounds, cursor).is_some() {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}
