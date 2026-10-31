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

// The meter is 4/4: four beats per bar. The slot value comes from the
// screen's columns per bar — sixteen columns make the slots sixteenths,
// so a beat is four of them.
const BEATS_PER_BAR: usize = 4;
// Rhythm-row design sizes, scaled off the font-size setting like the staff.
// The rhythm gap is the notehead line's clearance above the top string —
// the detachment between the rhythm row and the tab staff.
const RHYTHM_GAP: f32 = 26.0;
const STEM_HEIGHT: f32 = 30.0;
const BEAM_THICKNESS: f32 = 2.5;
const BEAM_SPACING: f32 = 4.0;
// An edge stub: the short fragment a lone finer note at its group's edge
// carries, pointing away from the group.
const STUB_LENGTH: f32 = 6.0;
const NOTEHEAD_RADIUS: f32 = 4.5;
// A standard notehead is an oval wider than tall, its long axis tilted
// up to the right.
const NOTEHEAD_ASPECT: f32 = 1.2;
const NOTEHEAD_TILT: f32 = 0.35;
// A dotted value's dot sits raised into the space above the line (the
// notehead sits on the line), clear of the stem.
const DOT_GAP: f32 = 4.5;
const DOT_RAISE: f32 = 4.5;
const DOT_RADIUS: f32 = 1.3;
const DOT_SPACING: f32 = 4.5;
const FLAG_LENGTH: f32 = 9.0;
const FLAG_DROP: f32 = 12.0;
const FLAG_GAP: f32 = 5.0;
const REST_HEIGHT: f32 = 14.0;
const REST_HOOK_EXTRA: f32 = 5.0;
const REST_BALL: f32 = 1.8;
const TIME_SIG_SIZE: f32 = 14.0;

// The tab canvas is paint-only for now; the event type exists so the screen
// can map the canvas into its own Message — the seam tab interaction will
// plug into later.
#[derive(Debug, Clone)]
pub enum Event {}

// The tab staff: one line per string in tab order, fret numbers in time
// columns, bar lines closing each bar, and a rhythm row notating each
// beat's rhythm above. The screen builds the fret grid; the canvas only
// paints.
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

// A notated event within a beat: a note that has absorbed the rests
// following it, or a leading run of rests. [value] is the event's length
// in slots — O--- is a quarter (4), O-O- two eighths (2), OOO- a pair of
// sixteenths and an eighth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BeatEvent {
    column: usize,
    value: usize,
    rest: bool,
}

// The notation plan for the staff: the events in time order, and per beat
// the beam segments, edge stubs, and flags.
struct Notation {
    events: Vec<BeatEvent>,
    // Beam segments: (first column, last column, level) — the stems one
    // beam connects; level 0 is the outermost beam.
    beams: Vec<(usize, usize, usize)>,
    // Edge stubs: (column, level, left) — a short fragment a lone finer
    // note at its group's edge carries, pointing away from the group.
    stubs: Vec<(usize, usize, bool)>,
    // Lone notes carry one flag per beam level: (column, flag count).
    flags: Vec<(usize, usize)>,
}

// The events of one beat: an optional leading rest, then each note with
// the rests it absorbs up to the next note or the beat's end.
fn beat_events(beat: &[bool]) -> Vec<BeatEvent> {
    let mut events = Vec::new();
    let mut column = 0;
    while column < beat.len() && beat[column] {
        column += 1;
    }
    if column > 0 {
        events.push(BeatEvent {
            column: 0,
            value: column,
            rest: true,
        });
    }
    while column < beat.len() {
        let start = column;
        column += 1;
        while column < beat.len() && beat[column] {
            column += 1;
        }
        events.push(BeatEvent {
            column: start,
            value: column - start,
            rest: false,
        });
    }
    events
}

// The notation plan for the whole staff. Absorption never crosses a beat
// boundary, so every beat plans independently.
fn notation(rests: &[bool], slots_per_beat: usize) -> Notation {
    let slots_per_beat = slots_per_beat.max(1);
    let mut plan = Notation {
        events: Vec::new(),
        beams: Vec::new(),
        stubs: Vec::new(),
        flags: Vec::new(),
    };
    let mut column = 0;
    while column < rests.len() {
        let beat_end = (column + slots_per_beat).min(rests.len());
        let mut events = beat_events(&rests[column..beat_end]);
        for event in &mut events {
            event.column += column;
        }
        let notes: Vec<BeatEvent> = events.iter().copied().filter(|event| !event.rest).collect();
        beat_beams(&notes, slots_per_beat, &mut plan);
        plan.events.extend(events);
        column = beat_end;
    }
    plan
}

// The beams of one beat: the beat's notes beam as one group — the outer
// beam spans them all, inner beams break at value changes. A lone note
// carries flags instead, one per beam level (none for a quarter).
fn beat_beams(notes: &[BeatEvent], slots_per_beat: usize, plan: &mut Notation) {
    match notes.len() {
        0 => {}
        1 => {
            let count = beams(notes[0].value, slots_per_beat);
            if count > 0 {
                plan.flags.push((notes[0].column, count));
            }
        }
        _ => {
            let max_level = notes
                .iter()
                .map(|note| beams(note.value, slots_per_beat))
                .max()
                .unwrap_or(0);
            for level in 0..max_level {
                let mut run = None;
                for (index, note) in notes.iter().enumerate() {
                    if beams(note.value, slots_per_beat) > level {
                        run.get_or_insert(index);
                    } else if let Some(start) = run.take() {
                        close_run(plan, notes, start, index - 1, level);
                    }
                }
                if let Some(start) = run {
                    close_run(plan, notes, start, notes.len() - 1, level);
                }
            }
        }
    }
}

// A run of one note is an edge stub pointing away from the group; a longer
// run is a beam connecting the stems.
fn close_run(plan: &mut Notation, notes: &[BeatEvent], start: usize, end: usize, level: usize) {
    if start == end {
        plan.stubs.push((notes[start].column, level, start == 0));
    } else {
        plan.beams
            .push((notes[start].column, notes[end].column, level));
    }
}

// The beam level of a note value: how many beams a note of [value] slots
// carries. The largest power of two within [value] sets the level; a
// full-beat note (a quarter) carries none.
fn beams(value: usize, slots_per_beat: usize) -> usize {
    let base = 1 << (usize::BITS - 1 - value.leading_zeros());
    (slots_per_beat / base).trailing_zeros() as usize
}

// The dot count of a note value: every set bit beyond the first is a dot.
fn dots(value: usize) -> usize {
    value.count_ones() as usize - 1
}

// The notehead: a filled oval, wider than tall and tilted like a standard
// notehead, built from four bezier arcs — the canvas has no
// rotated-ellipse primitive. K is the circle-arc bezier constant,
// 4/3 * tan(pi/8).
fn notehead_path(center: Point, radius: f32) -> canvas::Path {
    const K: f32 = 0.5523;
    let a = radius * NOTEHEAD_ASPECT;
    let b = radius;
    let (sin, cos) = NOTEHEAD_TILT.sin_cos();
    let rot =
        |x: f32, y: f32| Point::new(center.x + x * cos + y * sin, center.y - x * sin + y * cos);
    canvas::Path::new(|builder| {
        builder.move_to(rot(a, 0.0));
        builder.bezier_curve_to(rot(a, K * b), rot(K * a, b), rot(0.0, b));
        builder.bezier_curve_to(rot(-K * a, b), rot(-a, K * b), rot(-a, 0.0));
        builder.bezier_curve_to(rot(-a, -K * b), rot(-K * a, -b), rot(0.0, -b));
        builder.bezier_curve_to(rot(K * a, -b), rot(a, -K * b), rot(a, 0.0));
        builder.close();
    })
}

// The notehead's rightmost reach from its center — where the stem meets
// it: the rotated oval's horizontal extent.
fn notehead_extent(radius: f32) -> f32 {
    let (sin, cos) = NOTEHEAD_TILT.sin_cos();
    ((NOTEHEAD_ASPECT * cos).powi(2) + sin.powi(2)).sqrt() * radius
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

    // A stem rises from the right extreme of its column's notehead.
    fn stem_x(&self, column: usize) -> f32 {
        self.column_x(column) + notehead_extent(self.notehead_radius)
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
    // with each beat notated in standard notation — a note's value is the
    // slots it occupies, notes beam per beat with inner beams breaking at
    // value changes, a lone note is flagged, and a rest carries hooks per
    // its value.
    fn draw_rhythm(&self, frame: &mut canvas::Frame, g: &Geometry, c: &TabColors) {
        let right = g.left + self.columns as f32 * g.column_width;
        let slots_per_beat = (self.columns_per_bar / BEATS_PER_BAR).max(1);
        let plan = notation(&self.rests, slots_per_beat);

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

        for event in &plan.events {
            if event.rest {
                self.draw_rest(frame, g, c, event, slots_per_beat);
                continue;
            }
            frame.fill(
                &notehead_path(
                    Point::new(g.column_x(event.column), g.rhythm_y),
                    g.notehead_radius,
                ),
                c.fret_number,
            );
            frame.stroke(
                &canvas::Path::line(
                    Point::new(g.stem_x(event.column), g.rhythm_y),
                    Point::new(g.stem_x(event.column), g.stem_top),
                ),
                canvas::Stroke::default()
                    .with_color(c.fret_number)
                    .with_width(1.2),
            );
            // A dotted value carries a dot per extra slot, raised into the
            // space above the line (the notehead sits on the line).
            let dot_gap = self.scaled(DOT_GAP);
            let dot_raise = self.scaled(DOT_RAISE);
            let dot_spacing = self.scaled(DOT_SPACING);
            for dot in 0..dots(event.value) {
                frame.fill(
                    &canvas::Path::circle(
                        Point::new(
                            g.stem_x(event.column) + dot_gap + dot as f32 * dot_spacing,
                            g.rhythm_y - dot_raise,
                        ),
                        self.scaled(DOT_RADIUS),
                    ),
                    c.fret_number,
                );
            }
        }

        let beam_half = g.beam_thickness * 0.5;
        let beam_spacing = self.scaled(BEAM_SPACING);
        for &(from, to, level) in &plan.beams {
            // The beams stack down from the stem top, one per level — the
            // outermost beam is the stem's top.
            let top = g.stem_top + level as f32 * beam_spacing - beam_half;
            frame.fill(
                &canvas::Path::rectangle(
                    Point::new(g.stem_x(from), top),
                    Size::new(g.stem_x(to) - g.stem_x(from), g.beam_thickness),
                ),
                c.fret_number,
            );
        }
        // An edge stub is a short fragment pointing away from the group.
        let stub_length = self.scaled(STUB_LENGTH);
        for &(column, level, left) in &plan.stubs {
            let top = g.stem_top + level as f32 * beam_spacing - beam_half;
            let stem_x = g.stem_x(column);
            let (from, width) = if left {
                (stem_x - stub_length, stub_length)
            } else {
                (stem_x, stub_length)
            };
            frame.fill(
                &canvas::Path::rectangle(Point::new(from, top), Size::new(width, g.beam_thickness)),
                c.fret_number,
            );
        }
        let flag_length = self.scaled(FLAG_LENGTH);
        let flag_drop = self.scaled(FLAG_DROP);
        let flag_gap = self.scaled(FLAG_GAP);
        for &(column, count) in &plan.flags {
            for flag in 0..count {
                // A lone note carries one flag per beam level, stacked
                // down the stem — each a hook curving off the stem, concave
                // toward the notehead.
                let top = g.stem_top + flag as f32 * flag_gap;
                let stem_x = g.stem_x(column);
                let hook = canvas::Path::new(|builder| {
                    builder.move_to(Point::new(stem_x, top));
                    builder.quadratic_curve_to(
                        Point::new(stem_x + flag_length, top + flag_drop * 0.1),
                        Point::new(stem_x + flag_length * 0.5, top + flag_drop),
                    );
                });
                frame.stroke(
                    &hook,
                    canvas::Stroke::default()
                        .with_color(c.fret_number)
                        .with_width(1.2),
                );
            }
        }
    }

    // The rest glyph: a ball with a curved sweep down to a hook per beam
    // level — one hook for an eighth rest, two for a sixteenth — with a dot
    // per extra slot on a dotted rest. A rest filling the beat is the
    // quarter rest, drawn as its squiggle. Drawn rather than a font glyph —
    // the default font is a platform sans-serif with no guaranteed
    // musical-symbol coverage, and a missing glyph paints as tofu.
    fn draw_rest(
        &self,
        frame: &mut canvas::Frame,
        g: &Geometry,
        c: &TabColors,
        event: &BeatEvent,
        slots_per_beat: usize,
    ) {
        let x = g.column_x(event.column);
        if event.value == slots_per_beat {
            self.draw_quarter_rest(frame, g, c, x);
            return;
        }
        let hooks = beams(event.value, slots_per_beat);
        let height = self.scaled(REST_HEIGHT + (hooks.saturating_sub(1)) as f32 * REST_HOOK_EXTRA);
        let ball = Point::new(x - height * 0.1, g.rhythm_y - height * 0.6);
        frame.fill(
            &canvas::Path::circle(ball, self.scaled(REST_BALL)),
            c.fret_number,
        );
        let sweep = canvas::Path::new(|builder| {
            let bottom = Point::new(x + height * 0.15, g.rhythm_y + height * 0.25);
            builder.move_to(ball);
            builder.quadratic_curve_to(
                Point::new(x + height * 0.4, g.rhythm_y + height * 0.05),
                bottom,
            );
            // Each hook curls up-left off the tail — rising steeply, then
            // flattening out to the left — and the tail drops to the next
            // hook below.
            let mut point = bottom;
            for hook in 0..hooks {
                builder.quadratic_curve_to(
                    Point::new(point.x - height * 0.14, point.y - height * 0.3),
                    Point::new(point.x - height * 0.26, point.y - height * 0.34),
                );
                if hook + 1 < hooks {
                    point = Point::new(point.x, point.y + height * 0.22);
                    builder.line_to(point);
                }
            }
        });
        frame.stroke(
            &sweep,
            canvas::Stroke::default()
                .with_color(c.fret_number)
                .with_width(1.2),
        );
        // A dotted rest carries a dot per extra slot, off to the right.
        let dot_spacing = self.scaled(DOT_SPACING);
        for dot in 0..dots(event.value) {
            frame.fill(
                &canvas::Path::circle(
                    Point::new(
                        x + height * 0.45 + dot as f32 * dot_spacing,
                        g.rhythm_y - height * 0.2,
                    ),
                    self.scaled(REST_BALL),
                ),
                c.fret_number,
            );
        }
    }

    // The quarter rest: the squiggle — a zigzag curling down-left from the
    // top, across to the right, and down to a small hook at the bottom.
    fn draw_quarter_rest(&self, frame: &mut canvas::Frame, g: &Geometry, c: &TabColors, x: f32) {
        let height = self.scaled(REST_HEIGHT + REST_HOOK_EXTRA);
        let top = g.rhythm_y - height * 0.55;
        let bottom = g.rhythm_y + height * 0.45;
        let squiggle = canvas::Path::new(|builder| {
            builder.move_to(Point::new(x + height * 0.11, top));
            builder.line_to(Point::new(x - height * 0.12, top + height * 0.38));
            builder.line_to(Point::new(x + height * 0.14, top + height * 0.68));
            builder.line_to(Point::new(x - height * 0.06, bottom));
            // The bottom hook curls up-left.
            builder.quadratic_curve_to(
                Point::new(x + height * 0.02, bottom + height * 0.07),
                Point::new(x - height * 0.2, bottom - height * 0.06),
            );
        });
        frame.stroke(
            &squiggle,
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

    fn beat_event(column: usize, value: usize, rest: bool) -> BeatEvent {
        BeatEvent {
            column,
            value,
            rest,
        }
    }

    #[test]
    fn a_note_absorbs_the_rests_that_follow_it() {
        assert_eq!(
            beat_events(&[false, true, true, true]),
            vec![beat_event(0, 4, false)],
            "O--- is one quarter note"
        );
        assert_eq!(
            beat_events(&[false, true, false, true]),
            vec![beat_event(0, 2, false), beat_event(2, 2, false)],
            "O-O- is two eighth notes"
        );
        assert_eq!(
            beat_events(&[false, false, false, true]),
            vec![
                beat_event(0, 1, false),
                beat_event(1, 1, false),
                beat_event(2, 2, false)
            ],
            "OOO- is two sixteenths and an eighth"
        );
        assert_eq!(
            beat_events(&[false, true, false, false]),
            vec![
                beat_event(0, 2, false),
                beat_event(2, 1, false),
                beat_event(3, 1, false)
            ],
            "O-OO is an eighth and two sixteenths"
        );
    }

    #[test]
    fn leading_rests_form_a_single_rest() {
        assert_eq!(
            beat_events(&[true, true, false, true]),
            vec![beat_event(0, 2, true), beat_event(2, 2, false)],
            "a leading rest run merges; the note absorbs the trailing rest"
        );
        assert_eq!(
            beat_events(&[true, false, true, true]),
            vec![beat_event(0, 1, true), beat_event(1, 3, false)],
            "a dotted eighth follows its leading sixteenth rest"
        );
        assert_eq!(
            beat_events(&[true, true, true, true]),
            vec![beat_event(0, 4, true)],
            "a silent beat is one rest"
        );
    }

    #[test]
    fn beam_levels_and_dots_follow_the_note_value() {
        assert_eq!(beams(1, 4), 2, "a sixteenth beams twice");
        assert_eq!(beams(2, 4), 1, "an eighth beams once");
        assert_eq!(beams(3, 4), 1, "a dotted eighth beams once");
        assert_eq!(beams(4, 4), 0, "a quarter does not beam");
        assert_eq!(beams(1, 2), 1, "an eighth in an eighth grid beams once");
        assert_eq!(beams(2, 2), 0, "a quarter in an eighth grid does not beam");
        assert_eq!(dots(1), 0);
        assert_eq!(dots(2), 0);
        assert_eq!(dots(3), 1, "a dotted eighth carries one dot");
        assert_eq!(dots(4), 0);
    }

    #[test]
    fn the_four_beat_variations_plan_as_standard_notation() {
        // One bar: O--- | O-O- | OOO- | O-OO.
        let plan = notation(
            &[
                false, true, true, true, false, true, false, true, false, false, false, true,
                false, true, false, false,
            ],
            4,
        );
        assert_eq!(
            plan.events,
            vec![
                beat_event(0, 4, false),
                beat_event(4, 2, false),
                beat_event(6, 2, false),
                beat_event(8, 1, false),
                beat_event(9, 1, false),
                beat_event(10, 2, false),
                beat_event(12, 2, false),
                beat_event(14, 1, false),
                beat_event(15, 1, false),
            ]
        );
        assert_eq!(
            plan.beams,
            vec![(4, 6, 0), (8, 10, 0), (8, 9, 1), (12, 15, 0), (14, 15, 1)],
            "the outer beam spans each beat group; inner beams break at value changes"
        );
        assert!(plan.stubs.is_empty());
        assert!(
            plan.flags.is_empty(),
            "a lone quarter is neither beamed nor flagged"
        );
    }

    #[test]
    fn a_lone_note_is_flagged_per_its_value() {
        // --O- | ---O: a lone eighth, then a lone sixteenth.
        let plan = notation(&[true, true, false, true, true, true, true, false], 4);
        assert_eq!(
            plan.flags,
            vec![(2, 1), (7, 2)],
            "one flag for an eighth, two for a sixteenth"
        );
        assert!(plan.beams.is_empty());
        assert_eq!(
            plan.events,
            vec![
                beat_event(0, 2, true),
                beat_event(2, 2, false),
                beat_event(4, 3, true),
                beat_event(7, 1, false)
            ]
        );
    }

    #[test]
    fn a_finer_note_at_the_group_edge_carries_a_stub() {
        // OO-- | O--O: a sixteenth then a dotted eighth, and the reverse.
        let plan = notation(&[false, false, true, true, false, true, true, false], 4);
        assert_eq!(plan.beams, vec![(0, 1, 0), (4, 7, 0)]);
        assert_eq!(
            plan.stubs,
            vec![(0, 1, true), (7, 1, false)],
            "the lone sixteenth's inner beam points away from the group"
        );
    }

    #[test]
    fn an_inner_beam_breaks_around_a_coarser_note() {
        // OO-O: a sixteenth, an eighth, a sixteenth — the inner beams
        // break around the eighth, each sixteenth carrying an edge stub.
        let plan = notation(&[false, false, true, false], 4);
        assert_eq!(plan.beams, vec![(0, 3, 0)]);
        assert_eq!(plan.stubs, vec![(0, 1, true), (3, 1, false)]);
    }

    #[test]
    fn a_silent_bar_plans_rests_only() {
        let plan = notation(&[true; 8], 4);
        assert_eq!(
            plan.events,
            vec![beat_event(0, 4, true), beat_event(4, 4, true)],
            "a silent beat is one full-beat rest"
        );
        assert!(plan.beams.is_empty());
        assert!(plan.stubs.is_empty());
        assert!(plan.flags.is_empty());
    }

    #[test]
    fn a_partial_final_beat_plans_within_its_slots() {
        // Two sixteenths in a short final beat still beam — a double beam
        // spans the pair, as in a full beat.
        let plan = notation(&[false, false], 4);
        assert_eq!(plan.beams, vec![(0, 1, 0), (0, 1, 1)]);
        assert!(plan.flags.is_empty());
    }

    #[test]
    fn eighths_beam_per_beat_in_an_eighth_grid() {
        let plan = notation(&[false; 8], 2);
        assert_eq!(
            plan.beams,
            vec![(0, 1, 0), (2, 3, 0), (4, 5, 0), (6, 7, 0)],
            "a beam per beat — the bar line is a beat boundary too"
        );
        assert!(plan.flags.is_empty());
    }

    #[test]
    fn a_note_filling_its_beat_is_a_quarter() {
        // O- in an eighth grid: the note absorbs the rest and is a
        // quarter — neither beamed nor flagged.
        let plan = notation(&[false, true, false, true], 2);
        assert_eq!(
            plan.events,
            vec![beat_event(0, 2, false), beat_event(2, 2, false)]
        );
        assert!(plan.beams.is_empty());
        assert!(plan.flags.is_empty());
    }
}
