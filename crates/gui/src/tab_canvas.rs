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

// The meter: slots are eighth notes, so a 4/4 bar is eight of them and a
// beat is two.
const BEATS_PER_BAR: usize = 4;
// Rhythm-row design sizes, scaled off the font-size setting like the staff.
// The rhythm gap is the notehead line's clearance above the top string —
// the detachment between the rhythm row and the tab staff.
const RHYTHM_GAP: f32 = 26.0;
const STEM_HEIGHT: f32 = 30.0;
const BEAM_THICKNESS: f32 = 2.5;
const NOTEHEAD_RADIUS: f32 = 4.5;
const FLAG_LENGTH: f32 = 9.0;
const FLAG_DROP: f32 = 12.0;
const REST_HEIGHT: f32 = 14.0;
const REST_BALL: f32 = 1.8;
const TIME_SIG_SIZE: f32 = 14.0;

// The tab canvas is paint-only for now; the event type exists so the screen
// can map the canvas into its own Message — the seam tab interaction will
// plug into later.
#[derive(Debug, Clone)]
pub enum Event {}

// The tab staff: one line per string in tab order, fret numbers in time
// columns, bar lines closing each bar, and a rhythm row notating the slots
// above. The screen builds the fret grid; the canvas only paints.
pub struct TabCanvas {
    pub string_count: usize,
    // The open-note SPN label per string, drawn at the left of its row.
    pub open_notes: Vec<String>,
    pub columns: usize,
    // The fret grid: [string_index][column], None where the string does not
    // sound in that column — a rest, or a note played on another string.
    pub frets: Vec<Vec<Option<i32>>>,
    // Per column: true where the slot is a rest — drives the rhythm row.
    pub rests: Vec<bool>,
    pub columns_per_bar: usize,
    // The base font size; the design sizes scale off it.
    pub font_size: f32,
}

// How the rhythm row notates the note columns: beams connect the stems of
// consecutive notes within a beat; a note alone in its beat gets a flag.
struct BeamPlan {
    // Inclusive column ranges whose stems one beam connects.
    beams: Vec<(usize, usize)>,
    // Columns whose single note carries a flag instead of a beam.
    flags: Vec<usize>,
}

// Slots are eighth notes: [slots_per_beat] of them tile the bar into beats,
// and beams group the notes within each beat. A rest breaks the group, so a
// lone note in a beat is flagged rather than beamed, and no beam ever
// crosses a beat (or bar) boundary.
fn beam_plan(rests: &[bool], slots_per_beat: usize) -> BeamPlan {
    let slots_per_beat = slots_per_beat.max(1);
    let mut plan = BeamPlan {
        beams: Vec::new(),
        flags: Vec::new(),
    };
    let mut column = 0;
    while column < rests.len() {
        let beat_end = (column + slots_per_beat).min(rests.len());
        let mut run = None;
        for (offset, &rest) in rests[column..beat_end].iter().enumerate() {
            if rest {
                if let Some(start) = run.take() {
                    close_run(&mut plan, start, column + offset - 1);
                }
            } else if run.is_none() {
                run = Some(column + offset);
            }
        }
        if let Some(start) = run {
            close_run(&mut plan, start, beat_end - 1);
        }
        column = beat_end;
    }
    plan
}

// A run of one note cannot beam — it carries a flag instead.
fn close_run(plan: &mut BeamPlan, start: usize, end: usize) {
    if start == end {
        plan.flags.push(start);
    } else {
        plan.beams.push((start, end));
    }
}

struct Geometry {
    left: f32,
    top: f32,
    // The rhythm row rides above the staff: noteheads on [rhythm_y], stems
    // rising to [stem_top], where the beams sit.
    rhythm_y: f32,
    stem_top: f32,
    notehead_radius: f32,
    beam_thickness: f32,
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
        let rhythm_gap = RHYTHM_GAP * scale;
        let stem_height = STEM_HEIGHT * scale;
        let beam_thickness = BEAM_THICKNESS * scale;
        let notehead_radius = NOTEHEAD_RADIUS * scale;
        let rhythm_block = rhythm_gap + stem_height + beam_thickness;
        let block_top = ((bounds.height - (rhythm_block + staff_height)) * 0.5).max(TOP_PAD);
        let stem_top = block_top + beam_thickness * 0.5;
        let rhythm_y = stem_top + stem_height;
        Geometry {
            left: LABEL_GUTTER,
            top: rhythm_y + rhythm_gap,
            rhythm_y,
            stem_top,
            notehead_radius,
            beam_thickness,
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

    // A stem rises from the right edge of its column's notehead.
    fn stem_x(&self, column: usize) -> f32 {
        self.column_x(column) + self.notehead_radius
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

        // Bar lines span the whole system — the rhythm row down to the
        // bottom string.
        for x in g.bar_xs() {
            frame.stroke(
                &canvas::Path::line(Point::new(x, g.stem_top), Point::new(x, g.string_y(0))),
                canvas::Stroke::default()
                    .with_color(c.bar_line)
                    .with_width(1.5),
            );
        }
    }

    // The rhythm row above the staff: a one-line staff carrying the meter,
    // with each slot's eighth note as a notehead and stem — beamed to its
    // beat-mates, flagged when alone in its beat, and an eighth-rest glyph
    // on a rest.
    fn draw_rhythm(&self, frame: &mut canvas::Frame, g: &Geometry, c: &TabColors) {
        let right = g.left + self.columns as f32 * g.column_width;
        let slots_per_beat = (self.columns_per_bar / BEATS_PER_BAR).max(1);

        frame.stroke(
            &canvas::Path::line(
                Point::new(g.left, g.rhythm_y),
                Point::new(right, g.rhythm_y),
            ),
            canvas::Stroke::default()
                .with_color(c.bar_line)
                .with_width(1.0),
        );
        frame.fill_text(canvas::Text {
            content: format!("{}/4", BEATS_PER_BAR),
            position: Point::new(4.0, g.rhythm_y),
            color: c.open_note,
            size: self.scaled(TIME_SIG_SIZE).into(),
            align_x: Alignment::Left,
            align_y: iced::alignment::Vertical::Center,
            ..canvas::Text::default()
        });

        for column in 0..self.columns {
            if self.rests.get(column).copied().unwrap_or(true) {
                self.draw_rest(frame, g, c, column);
                continue;
            }
            frame.fill(
                &canvas::Path::circle(
                    Point::new(g.column_x(column), g.rhythm_y),
                    g.notehead_radius,
                ),
                c.fret_number,
            );
            frame.stroke(
                &canvas::Path::line(
                    Point::new(g.stem_x(column), g.rhythm_y),
                    Point::new(g.stem_x(column), g.stem_top),
                ),
                canvas::Stroke::default()
                    .with_color(c.fret_number)
                    .with_width(1.2),
            );
        }

        let plan = beam_plan(&self.rests, slots_per_beat);
        let beam_half = g.beam_thickness * 0.5;
        for &(start, end) in &plan.beams {
            frame.fill(
                &canvas::Path::rectangle(
                    Point::new(g.stem_x(start), g.stem_top - beam_half),
                    Size::new(g.stem_x(end) - g.stem_x(start), g.beam_thickness),
                ),
                c.fret_number,
            );
        }
        let flag_length = self.scaled(FLAG_LENGTH);
        let flag_drop = self.scaled(FLAG_DROP);
        for &column in &plan.flags {
            frame.stroke(
                &canvas::Path::line(
                    Point::new(g.stem_x(column), g.stem_top),
                    Point::new(g.stem_x(column) + flag_length, g.stem_top + flag_drop),
                ),
                canvas::Stroke::default()
                    .with_color(c.fret_number)
                    .with_width(1.2),
            );
        }
    }

    // The eighth rest: a ball with a curved sweep down to a small hook. Drawn
    // rather than a font glyph — the default font is a platform sans-serif
    // with no guaranteed musical-symbol coverage, and a missing glyph paints
    // as tofu.
    fn draw_rest(&self, frame: &mut canvas::Frame, g: &Geometry, c: &TabColors, column: usize) {
        let x = g.column_x(column);
        let height = self.scaled(REST_HEIGHT);
        let ball = Point::new(x - height * 0.1, g.rhythm_y - height * 0.6);
        frame.fill(
            &canvas::Path::circle(ball, self.scaled(REST_BALL)),
            c.fret_number,
        );
        let sweep = canvas::Path::new(|builder| {
            builder.move_to(ball);
            builder.quadratic_curve_to(
                Point::new(x + height * 0.4, g.rhythm_y + height * 0.05),
                Point::new(x + height * 0.15, g.rhythm_y + height * 0.3),
            );
            builder.line_to(Point::new(x - height * 0.05, g.rhythm_y + height * 0.42));
        });
        frame.stroke(
            &sweep,
            canvas::Stroke::default()
                .with_color(c.fret_number)
                .with_width(1.2),
        );
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
        self.draw_rhythm(&mut frame, &g, &c);
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
    fn the_rhythm_row_rides_above_the_staff() {
        let g = Geometry::new(bounds(800.0, 200.0), 6, 8, 8, 1.0);
        assert_eq!(
            g.rhythm_y,
            g.top - RHYTHM_GAP,
            "the noteheads sit one gap above the top string"
        );
        assert_eq!(
            g.stem_top,
            g.rhythm_y - STEM_HEIGHT,
            "the stems rise from the noteheads to the beam level"
        );
    }

    #[test]
    fn the_staff_centers_vertically_and_scales_with_the_font_size() {
        let g = Geometry::new(bounds(800.0, 400.0), 6, 4, 4, 1.0);
        let rhythm_block = RHYTHM_GAP + STEM_HEIGHT + BEAM_THICKNESS;
        let content = rhythm_block + 5.0 * g.row_gap;
        assert_eq!(
            g.top,
            (400.0 - content) * 0.5 + rhythm_block - BEAM_THICKNESS * 0.5
        );
        let doubled = Geometry::new(bounds(800.0, 400.0), 6, 4, 4, 2.0);
        assert_eq!(doubled.row_gap, g.row_gap * 2.0);
        assert_eq!(doubled.column_width, g.column_width * 2.0);
        assert_eq!(doubled.rhythm_y - doubled.stem_top, STEM_HEIGHT * 2.0);
    }

    #[test]
    fn eighths_beam_per_beat_and_a_rest_breaks_the_beam() {
        // The demo phrase: 1 2 3 5 (rest) 6 5 3.
        let plan = beam_plan(&[false, false, false, false, true, false, false, false], 2);
        assert_eq!(plan.beams, vec![(0, 1), (2, 3), (6, 7)]);
        assert_eq!(
            plan.flags,
            vec![5],
            "the note alone in its beat carries a flag"
        );
    }

    #[test]
    fn beams_break_at_beat_boundaries_including_bar_lines() {
        let plan = beam_plan(&[false; 12], 2);
        assert_eq!(
            plan.beams,
            vec![(0, 1), (2, 3), (4, 5), (6, 7), (8, 9), (10, 11)],
            "a beam per beat — the bar line at slot 8 is a beat boundary too"
        );
        assert!(plan.flags.is_empty());
    }

    #[test]
    fn rests_at_the_edges_leave_the_inner_notes_beamed() {
        let plan = beam_plan(&[true, false, false, false, false, true], 2);
        assert_eq!(plan.beams, vec![(2, 3)]);
        assert_eq!(plan.flags, vec![1, 4], "each edge rest strands a lone note");
    }

    #[test]
    fn a_partial_final_beat_flags_its_lone_note() {
        let plan = beam_plan(&[false; 5], 2);
        assert_eq!(plan.beams, vec![(0, 1), (2, 3)]);
        assert_eq!(plan.flags, vec![4]);
    }

    #[test]
    fn all_rests_plan_no_notation() {
        let plan = beam_plan(&[true; 8], 2);
        assert!(plan.beams.is_empty());
        assert!(plan.flags.is_empty());
    }

    #[test]
    fn a_single_slot_per_beat_flags_every_note() {
        let plan = beam_plan(&[false, true, false], 1);
        assert!(plan.beams.is_empty());
        assert_eq!(plan.flags, vec![0, 2]);
    }
}
