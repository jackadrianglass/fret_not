use iced::mouse;
use iced::widget::canvas::{self, Action};
use iced::widget::text::Alignment;
use iced::{Point, Rectangle, Renderer, Theme};

use fretboard::fretboard_position::FretboardPosition;

use crate::board;
use crate::colors::CanvasColors;

const INLAY_FRETS: [i32; 7] = [3, 5, 7, 9, 12, 15, 17];

// One drawn position; the board component builds these, the canvas only paints.
pub struct Visual {
    pub position: FretboardPosition,
    pub label: Option<String>,
    pub root_halo: bool,
    pub selected: bool,
    pub emphasized: bool,
}

pub struct FretboardCanvas {
    pub visuals: Vec<Visual>,
    pub string_count: usize,
    pub max_fret: i32,
}

struct Geometry {
    left: f32,
    top: f32,
    fret_width: f32,
    string_gap: f32,
    dot_radius: f32,
}

impl Geometry {
    fn new(bounds: Rectangle, string_count: usize, max_fret: i32) -> Geometry {
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
            fret_width,
            string_gap,
            dot_radius: (fret_width * 0.22).clamp(7.0, 14.0),
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

    fn string_y(&self, string_index: usize) -> f32 {
        self.top + string_index as f32 * self.string_gap
    }

    fn center(&self, position: FretboardPosition) -> Point {
        Point::new(
            self.center_x(position.fret),
            self.string_y(position.string_index),
        )
    }
}

impl FretboardCanvas {
    fn geometry(&self, bounds: &Rectangle) -> Geometry {
        Geometry::new(*bounds, self.string_count, self.max_fret)
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

        let number_y = g.string_y(last) + 18.0;
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
    }

    fn draw_dot(&self, frame: &mut canvas::Frame, g: &Geometry, c: &CanvasColors, v: &Visual) {
        let center = g.center(v.position);
        let radius = g.dot_radius;

        let path = canvas::Path::circle(center, radius);

        if !v.emphasized {
            // A solid fill masks the string under the dot, so the label
            // never has a line striking through it.
            frame.fill(&path, c.background);
            frame.stroke(
                &path,
                canvas::Stroke::default()
                    .with_color(c.field_label)
                    .with_width(1.2),
            );
            if let Some(label) = &v.label {
                frame.fill_text(canvas::Text {
                    content: label.clone(),
                    position: center,
                    color: c.field_label,
                    size: 12.0.into(),
                    align_x: Alignment::Center,
                    align_y: iced::alignment::Vertical::Center,
                    ..canvas::Text::default()
                });
            }
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

        if let Some(label) = &v.label {
            frame.fill_text(canvas::Text {
                content: label.clone(),
                position: center,
                color: c.label,
                size: 12.0.into(),
                align_x: Alignment::Center,
                align_y: iced::alignment::Vertical::Center,
                ..canvas::Text::default()
            });
        }
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
        let g = Geometry::new(bounds, self.string_count, self.max_fret);
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
