use iced::mouse;
use iced::widget::canvas::{self, Action};
use iced::widget::text::Alignment;
use iced::{Point, Rectangle, Renderer, Size, Theme};

use crate::colors::TabColors;
use crate::model::BASE_FONT_SIZE;

// Design sizes at the BASE_FONT_SIZE base — the column width and row gap
// scale off the font-size setting, so the whole staff grows with the text.
const COLUMN_WIDTH: f32 = 30.0;
const ROW_GAP: f32 = 24.0;
const LABEL_GUTTER: f32 = 48.0;
const RIGHT_PAD: f32 = 24.0;
const TOP_PAD: f32 = 16.0;
const FRET_SIZE: f32 = 12.0;
const LABEL_SIZE: f32 = 11.0;

// The tab canvas is paint-only for now; the event type exists so the screen
// can map the canvas into its own Message — the seam tab interaction will
// plug into later.
#[derive(Debug, Clone)]
pub enum Event {}

// The tab staff: one line per string in tab order, fret numbers in time
// columns, bar lines closing each bar. The screen builds the fret grid; the
// canvas only paints.
pub struct TabCanvas {
    pub string_count: usize,
    // The open-note SPN label per string, drawn at the left of its row.
    pub open_notes: Vec<String>,
    pub columns: usize,
    // The fret grid: [string_index][column], None where the string does not
    // sound in that column — a rest, or a note played on another string.
    pub frets: Vec<Vec<Option<i32>>>,
    pub columns_per_bar: usize,
    // The base font size; the design sizes scale off it.
    pub font_size: f32,
}

struct Geometry {
    left: f32,
    top: f32,
    rows: usize,
    columns: usize,
    columns_per_bar: usize,
    column_width: f32,
    row_gap: f32,
}

impl Geometry {
    fn new(
        bounds: Rectangle,
        string_count: usize,
        columns: usize,
        columns_per_bar: usize,
        scale: f32,
    ) -> Geometry {
        let column_width = COLUMN_WIDTH * scale;
        let row_gap = ROW_GAP * scale;
        let staff_height = string_count.saturating_sub(1) as f32 * row_gap;
        Geometry {
            left: LABEL_GUTTER,
            top: ((bounds.height - staff_height) * 0.5).max(TOP_PAD),
            rows: string_count,
            columns,
            columns_per_bar,
            column_width,
            row_gap,
        }
    }

    // The center of a column's cell.
    fn column_x(&self, column: usize) -> f32 {
        self.left + (column as f32 + 0.5) * self.column_width
    }

    // Tab convention: the highest string is the top row, so [string_index]
    // (numbered low to high) counts the rows from the bottom up.
    fn string_y(&self, string_index: usize) -> f32 {
        self.top + (self.rows - 1 - string_index) as f32 * self.row_gap
    }

    // The bar lines: one at each full bar's left edge, plus the staff's
    // right edge — a bar line closes every bar.
    fn bar_xs(&self) -> Vec<f32> {
        let step = self.columns_per_bar.max(1);
        let mut xs: Vec<f32> = (0..self.columns)
            .step_by(step)
            .map(|bar| self.left + bar as f32 * self.column_width)
            .collect();
        let end = self.left + self.columns as f32 * self.column_width;
        if xs.last() != Some(&end) {
            xs.push(end);
        }
        xs
    }
}

impl TabCanvas {
    fn scaled(&self, size: f32) -> f32 {
        size * self.font_size / BASE_FONT_SIZE
    }

    // The staff's content width. The canvas is sized to this so a staff
    // wider than the tab area scrolls horizontally instead of stretching.
    pub fn content_width(&self) -> f32 {
        let scale = self.font_size / BASE_FONT_SIZE;
        LABEL_GUTTER + self.columns as f32 * COLUMN_WIDTH * scale + RIGHT_PAD
    }

    fn geometry(&self, bounds: &Rectangle) -> Geometry {
        Geometry::new(
            *bounds,
            self.string_count,
            self.columns,
            self.columns_per_bar,
            self.font_size / BASE_FONT_SIZE,
        )
    }

    fn draw_staff(&self, frame: &mut canvas::Frame, g: &Geometry, c: &TabColors) {
        let last = self.string_count.saturating_sub(1);
        let right = g.left + self.columns as f32 * g.column_width;

        for string_index in 0..self.string_count {
            if let Some(note) = self.open_notes.get(string_index) {
                frame.fill_text(canvas::Text {
                    content: note.clone(),
                    position: Point::new(4.0, g.string_y(string_index)),
                    color: c.open_note,
                    size: self.scaled(LABEL_SIZE).into(),
                    align_x: Alignment::Left,
                    align_y: iced::alignment::Vertical::Center,
                    ..canvas::Text::default()
                });
            }
        }

        for string_index in 0..self.string_count {
            let y = g.string_y(string_index);
            // The outer strings read heavier, matching the fretboard.
            let width = if string_index == 0 || string_index == last {
                3.0
            } else {
                1.6
            };
            frame.stroke(
                &canvas::Path::line(Point::new(g.left, y), Point::new(right, y)),
                canvas::Stroke::default()
                    .with_color(c.string)
                    .with_width(width),
            );
        }

        for x in g.bar_xs() {
            frame.stroke(
                &canvas::Path::line(
                    Point::new(x, g.string_y(last)),
                    Point::new(x, g.string_y(0)),
                ),
                canvas::Stroke::default()
                    .with_color(c.bar_line)
                    .with_width(1.5),
            );
        }
    }

    // The fret numbers, centered on their string line in their column. The
    // background knockout behind each glyph keeps the string line from
    // striking through the number — the tab's version of the board's dot
    // fill.
    fn draw_numbers(&self, frame: &mut canvas::Frame, g: &Geometry, c: &TabColors) {
        let size = self.scaled(FRET_SIZE);
        for (string_index, row) in self.frets.iter().enumerate().take(self.string_count) {
            for (column, fret) in row.iter().enumerate() {
                if let Some(fret) = fret {
                    let center = Point::new(g.column_x(column), g.string_y(string_index));
                    let chars = if *fret >= 10 { 2.0 } else { 1.0 };
                    let half_width = 0.31 * size * chars + 2.0;
                    let half_height = 0.36 * size + 2.0;
                    frame.fill(
                        &canvas::Path::rectangle(
                            Point::new(center.x - half_width, center.y - half_height),
                            Size::new(half_width * 2.0, half_height * 2.0),
                        ),
                        c.background,
                    );
                    frame.fill_text(canvas::Text {
                        content: fret.to_string(),
                        position: center,
                        color: c.fret_number,
                        size: size.into(),
                        align_x: Alignment::Center,
                        align_y: iced::alignment::Vertical::Center,
                        ..canvas::Text::default()
                    });
                }
            }
        }
    }
}

impl canvas::Program<Event, Theme, Renderer> for TabCanvas {
    type State = ();

    fn update(
        &self,
        _state: &mut (),
        _event: &canvas::Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<Action<Event>> {
        None
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
        let c = TabColors::of(theme);

        self.draw_staff(&mut frame, &g, &c);
        self.draw_numbers(&mut frame, &g, &c);

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _state: &(),
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        mouse::Interaction::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds(width: f32, height: f32) -> Rectangle {
        Rectangle {
            x: 0.0,
            y: 0.0,
            width,
            height,
        }
    }

    #[test]
    fn the_highest_string_is_the_top_row() {
        let g = Geometry::new(bounds(800.0, 200.0), 6, 4, 4, 1.0);
        assert_eq!(g.string_y(5), g.top, "the high string is the top row");
        assert_eq!(
            g.string_y(0),
            g.top + 5.0 * g.row_gap,
            "the low string is the bottom row"
        );
    }

    #[test]
    fn columns_center_in_their_cells() {
        let g = Geometry::new(bounds(800.0, 200.0), 6, 12, 4, 1.0);
        assert_eq!(g.column_x(0), g.left + g.column_width * 0.5);
        assert_eq!(g.column_x(1), g.left + g.column_width * 1.5);
    }

    #[test]
    fn bar_lines_land_on_bar_boundaries_and_close_the_staff() {
        let g = Geometry::new(bounds(800.0, 200.0), 6, 12, 4, 1.0);
        assert_eq!(
            g.bar_xs(),
            vec![
                g.left,
                g.left + 4.0 * g.column_width,
                g.left + 8.0 * g.column_width,
                g.left + 12.0 * g.column_width,
            ],
            "a bar line per bar edge, the last closing the staff"
        );
        let partial = Geometry::new(bounds(800.0, 200.0), 6, 5, 4, 1.0);
        assert_eq!(
            partial.bar_xs(),
            vec![
                partial.left,
                partial.left + 4.0 * partial.column_width,
                partial.left + 5.0 * partial.column_width,
            ],
            "a partial bar still closes at the right edge"
        );
    }

    #[test]
    fn the_staff_centers_vertically_and_scales_with_the_font_size() {
        let g = Geometry::new(bounds(800.0, 200.0), 6, 4, 4, 1.0);
        assert_eq!(g.top, (200.0 - 5.0 * g.row_gap) * 0.5);
        let doubled = Geometry::new(bounds(800.0, 200.0), 6, 4, 4, 2.0);
        assert_eq!(doubled.row_gap, g.row_gap * 2.0);
        assert_eq!(doubled.column_width, g.column_width * 2.0);
    }
}
