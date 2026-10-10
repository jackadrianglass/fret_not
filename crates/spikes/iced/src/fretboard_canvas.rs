use iced::mouse;
use iced::widget::canvas::{self, Action};
use iced::widget::text::Alignment;
use iced::{Color, Point, Rectangle, Renderer, Theme};

use fretboard::fretboard_position::FretboardPosition;

use crate::Message;

pub const ACCENT: Color = Color::from_rgba8(0xFF, 0x8A, 0x3D, 1.0);
const STRING: Color = Color::from_rgba8(0x99, 0x99, 0xA0, 1.0);
const NUT: Color = Color::from_rgba8(0xCC, 0xCC, 0xD4, 1.0);
const FRET_LINE: Color = Color::from_rgba8(0x50, 0x50, 0x5A, 1.0);
const OFF_KEY: Color = Color::from_rgba8(0x66, 0x66, 0x70, 1.0);
const INLAY: Color = Color::from_rgba8(0x44, 0x44, 0x4E, 1.0);
const FRET_NUMBER: Color = Color::from_rgba8(0x88, 0x88, 0x90, 1.0);
const LABEL: Color = Color::WHITE;

const INLAY_FRETS: [i32; 7] = [3, 5, 7, 9, 12, 15, 17];

// One drawn position, with every visual factor pre-interpolated by the view.
// The canvas only paints; the clocks live in the application state.
pub struct Visual {
    pub position: FretboardPosition,
    pub label: Option<String>,
    pub in_key: bool,
    pub root_halo: bool,
    pub selected: bool,
    pub bloom: f32,
    pub wave: f32,
    pub scale: f32,
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
        let pad_x = 24.0;
        let pad_top = 20.0;
        let pad_bottom = 30.0;
        let fret_width = (bounds.width - 2.0 * pad_x) / (max_fret as f32 + 1.0);
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
            dot_radius: (fret_width * 0.16).clamp(5.0, 11.0),
        }
    }

    fn fret_x(&self, fret: i32) -> f32 {
        self.left + fret as f32 * self.fret_width
    }

    fn center_x(&self, fret: i32) -> f32 {
        self.fret_x(fret) + self.fret_width * 0.5
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

fn with_alpha(color: Color, alpha: f32) -> Color {
    Color { a: alpha, ..color }
}

fn dimmed(color: Color, factor: f32) -> Color {
    Color {
        r: color.r * factor,
        g: color.g * factor,
        b: color.b * factor,
        a: color.a,
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

    fn draw_grid(&self, frame: &mut canvas::Frame, g: &Geometry) {
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
                    .with_color(STRING)
                    .with_width(width),
            );
        }

        let nut = canvas::Path::line(
            Point::new(g.fret_x(0), g.string_y(0)),
            Point::new(g.fret_x(0), g.string_y(last)),
        );
        frame.stroke(
            &nut,
            canvas::Stroke::default().with_color(NUT).with_width(6.0),
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
                    .with_color(FRET_LINE)
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
                    frame.fill(&canvas::Path::circle(Point::new(x, upper), 4.0), INLAY);
                    frame.fill(&canvas::Path::circle(Point::new(x, lower), 4.0), INLAY);
                } else {
                    let middle = (g.string_y(2) + g.string_y(3)) * 0.5;
                    frame.fill(&canvas::Path::circle(Point::new(x, middle), 4.0), INLAY);
                }
            }
        }

        let number_y = g.string_y(last) + 18.0;
        for fret in 1..=self.max_fret {
            frame.fill_text(canvas::Text {
                content: fret.to_string(),
                position: Point::new(g.center_x(fret), number_y),
                color: FRET_NUMBER,
                size: 10.0.into(),
                align_x: Alignment::Center,
                align_y: iced::alignment::Vertical::Center,
                ..canvas::Text::default()
            });
        }
    }

    fn draw_dot(&self, frame: &mut canvas::Frame, g: &Geometry, v: &Visual) {
        let center = g.center(v.position);
        let radius = g.dot_radius * (0.25 + 0.75 * v.bloom) * v.scale;
        if radius < 0.5 {
            return;
        }

        let path = canvas::Path::circle(center, radius);

        if !v.in_key {
            frame.stroke(
                &path,
                canvas::Stroke::default()
                    .with_color(with_alpha(OFF_KEY, v.bloom))
                    .with_width(1.2),
            );
            return;
        }

        frame.fill(
            &path,
            with_alpha(dimmed(ACCENT, 0.4 + 0.6 * v.wave), v.bloom),
        );

        if v.root_halo {
            frame.stroke(
                &canvas::Path::circle(center, radius + 2.5),
                canvas::Stroke::default()
                    .with_color(with_alpha(Color::BLACK, v.bloom))
                    .with_width(2.0),
            );
        }

        if v.selected {
            frame.stroke(
                &canvas::Path::circle(center, radius + 4.0),
                canvas::Stroke::default()
                    .with_color(with_alpha(LABEL, 0.9 * v.bloom))
                    .with_width(1.5),
            );
        }

        if let Some(label) = &v.label {
            frame.fill_text(canvas::Text {
                content: label.clone(),
                position: center,
                color: with_alpha(LABEL, v.bloom),
                size: 10.0.into(),
                align_x: Alignment::Center,
                align_y: iced::alignment::Vertical::Center,
                ..canvas::Text::default()
            });
        }
    }
}

impl canvas::Program<Message, Theme, Renderer> for FretboardCanvas {
    type State = ();

    fn update(
        &self,
        _state: &mut (),
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        let hovered = self.hit_test(&bounds, cursor);
        match event {
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                hovered.map(|position| Action::publish(Message::Clicked(position)))
            }
            _ => Some(Action::publish(Message::Hovered(hovered))),
        }
    }

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let g = Geometry::new(bounds, self.string_count, self.max_fret);

        self.draw_grid(&mut frame, &g);
        for v in &self.visuals {
            self.draw_dot(&mut frame, &g, v);
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
