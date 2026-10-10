use xilem::core::{MessageCtx, MessageResult, Mut, ViewMarker};
use xilem::masonry::accesskit::Role;
use xilem::masonry::core::{
    AccessCtx, ChildrenIds, EventCtx, LayoutCtx, MeasureCtx, PaintCtx, PropertiesMut,
    PropertiesRef, RegisterCtx, Widget, WidgetMut, render_text,
};
use xilem::masonry::imaging::Painter;
use xilem::masonry::kurbo::{Affine, Axis, Circle, Point, Rect, Size, Stroke};
use xilem::masonry::layout::Length;
use xilem::masonry::parley::{Alignment, AlignmentOptions, StyleProperty};
use xilem::masonry::peniko::Color;
use xilem::{Pod, ViewCtx};

use fretboard::fretboard_position::FretboardPosition;

use crate::{FretNot, Visual};

const STRING: Color = Color::from_rgba8(0x99, 0x99, 0xA0, 0xFF);
const NUT: Color = Color::from_rgba8(0xCC, 0xCC, 0xD4, 0xFF);
const FRET_LINE: Color = Color::from_rgba8(0x50, 0x50, 0x5A, 0xFF);
const OFF_KEY: Color = Color::from_rgba8(0x66, 0x66, 0x70, 0xFF);
const INLAY: Color = Color::from_rgba8(0x44, 0x44, 0x4E, 0xFF);
const LABEL_TEXT: Color = Color::WHITE;
const FRET_NUMBER: Color = Color::from_rgba8(0x88, 0x88, 0x90, 0xFF);

const INLAY_FRETS: [i32; 7] = [3, 5, 7, 9, 12, 15, 17];

#[derive(Debug)]
pub enum BoardAction {
    Clicked(FretboardPosition),
    Hovered(Option<FretboardPosition>),
}

struct BoardGeom {
    pad_x: f64,
    pad_top: f64,
    fw: f64,
    sg: f64,
    dot_r: f64,
    string_count: usize,
    max_fret: i32,
}

impl BoardGeom {
    fn new(size: Size, string_count: usize, max_fret: i32) -> BoardGeom {
        let pad_x = 24.0;
        let pad_top = 20.0;
        let pad_bottom = 30.0;
        BoardGeom {
            pad_x,
            pad_top,
            fw: (size.width - 2.0 * pad_x) / (max_fret as f64 + 1.0),
            sg: (size.height - pad_top - pad_bottom) / (string_count as f64 - 1.0),
            dot_r: ((size.width - 2.0 * pad_x) / (max_fret as f64 + 1.0) * 0.16).clamp(6.0, 16.0),
            string_count,
            max_fret,
        }
    }

    fn center_x(&self, fret: i32) -> f64 {
        self.pad_x + (fret as f64 + 0.5) * self.fw
    }

    fn fret_x(&self, fret: i32) -> f64 {
        self.pad_x + fret as f64 * self.fw
    }

    fn string_y(&self, string_index: usize) -> f64 {
        self.pad_top + string_index as f64 * self.sg
    }

    fn hit_test(&self, position: Point) -> Option<FretboardPosition> {
        let fret = ((position.x - self.pad_x) / self.fw).floor() as i32;
        let string = (position.y - self.pad_top + self.sg / 2.0) / self.sg;
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

pub struct FretboardWidget {
    visuals: Vec<Visual>,
    string_count: usize,
    max_fret: i32,
    hover: Option<FretboardPosition>,
    size: Size,
    geom: Option<BoardGeom>,
}

impl FretboardWidget {
    pub fn new(state: &FretNot) -> FretboardWidget {
        FretboardWidget {
            visuals: state.visuals(),
            string_count: state.instrument.string_count(),
            max_fret: state.instrument.max_fret(),
            hover: state.hover,
            size: Size::ZERO,
            geom: None,
        }
    }

    pub fn sync(this: &mut WidgetMut<'_, Self>, state: &FretNot) {
        this.widget.visuals = state.visuals();
        this.widget.hover = state.hover;
        this.ctx.request_render();
    }
}

impl Widget for FretboardWidget {
    type Action = BoardAction;

    fn register_children(&mut self, _ctx: &mut RegisterCtx<'_>) {}

    fn on_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &xilem::masonry::core::PointerEvent,
    ) {
        let Some(geom) = &self.geom else {
            return;
        };
        match event {
            xilem::masonry::core::PointerEvent::Move(move_event) => {
                let p = move_event.current.position;
                let hovered = geom.hit_test(Point::new(p.x, p.y));
                if hovered != self.hover {
                    self.hover = hovered;
                    let action = BoardAction::Hovered(hovered);
                    ctx.submit_action::<Self::Action>(action);
                    ctx.request_paint_only();
                }
            }
            xilem::masonry::core::PointerEvent::Down(down_event) => {
                let p = down_event.state.position;
                if let Some(position) = geom.hit_test(Point::new(p.x, p.y)) {
                    ctx.submit_action::<Self::Action>(BoardAction::Clicked(position));
                }
            }
            _ => {}
        }
    }

    fn measure(
        &mut self,
        _ctx: &mut MeasureCtx<'_>,
        _props: &PropertiesRef<'_>,
        _axis: Axis,
        len_req: xilem::masonry::layout::LenReq,
        _cross_length: Option<Length>,
    ) -> Length {
        match len_req {
            xilem::masonry::layout::LenReq::FitContent(space) => space,
            _ => Length::const_px(200.),
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx<'_>, _props: &PropertiesRef<'_>, size: Size) {
        self.size = size;
        self.geom = Some(BoardGeom::new(size, self.string_count, self.max_fret));
        ctx.set_clip_path(size.to_rect());
    }

    fn paint(
        &mut self,
        ctx: &mut PaintCtx<'_>,
        _props: &PropertiesRef<'_>,
        painter: &mut Painter<'_>,
    ) {
        let geom = BoardGeom::new(self.size, self.string_count, self.max_fret);
        let last_y = geom.string_y(self.string_count - 1);
        let board_w = geom.fret_x(self.max_fret) - geom.fret_x(0);

        for string_index in 0..self.string_count {
            let y = geom.string_y(string_index);
            let thickness = if string_index == 0 || string_index + 1 == self.string_count {
                3.0
            } else {
                1.6
            };
            painter
                .fill(
                    Rect::new(
                        geom.fret_x(0),
                        y - thickness / 2.0,
                        geom.fret_x(0) + board_w,
                        y + thickness / 2.0,
                    ),
                    STRING,
                )
                .draw();
        }

        painter
            .fill(
                Rect::new(
                    geom.fret_x(0) - 3.0,
                    geom.string_y(0),
                    geom.fret_x(0) + 3.0,
                    last_y,
                ),
                NUT,
            )
            .draw();

        for fret in 1..=self.max_fret {
            let x = geom.fret_x(fret);
            painter
                .fill(Rect::new(x, geom.string_y(0), x + 1.0, last_y), FRET_LINE)
                .draw();
        }

        for fret in INLAY_FRETS {
            if fret > self.max_fret {
                break;
            }
            let x = geom.center_x(fret);
            if fret == 12 {
                painter
                    .fill(
                        Circle::new((x, geom.string_y(1) + geom.sg / 2.0), 4.0),
                        INLAY,
                    )
                    .draw();
                painter
                    .fill(
                        Circle::new((x, geom.string_y(3) + geom.sg / 2.0), 4.0),
                        INLAY,
                    )
                    .draw();
            } else {
                painter
                    .fill(
                        Circle::new((x, geom.string_y(2) + geom.sg / 2.0), 4.0),
                        INLAY,
                    )
                    .draw();
            }
        }

        let (fcx, lcx) = ctx.text_contexts();

        for visual in &self.visuals {
            let x = geom.center_x(visual.position.fret);
            let y = geom.string_y(visual.position.string_index);
            let radius = geom.dot_r * (0.25 + 0.75 * visual.bloom as f64) * visual.scale as f64;

            if !visual.in_key {
                if radius > 0.5 {
                    painter
                        .stroke(Circle::new((x, y), radius), &Stroke::new(1.2), OFF_KEY)
                        .draw();
                }
                continue;
            }

            if visual.bloom > 0.01 {
                let wave = 0.4 + 0.6 * visual.wave;
                let dimmed = Color::from_rgba8(
                    (255.0 * wave) as u8,
                    (138.0 * wave) as u8,
                    (61.0 * wave) as u8,
                    (visual.bloom * 255.0) as u8,
                );
                painter.fill(Circle::new((x, y), radius), dimmed).draw();
                if visual.root_halo {
                    painter
                        .stroke(
                            Circle::new((x, y), radius + 2.5),
                            &Stroke::new(2.0),
                            Color::BLACK,
                        )
                        .draw();
                }
                if visual.selected {
                    painter
                        .stroke(
                            Circle::new((x, y), radius + 4.0),
                            &Stroke::new(1.5),
                            LABEL_TEXT,
                        )
                        .draw();
                }
                if let Some(label) = &visual.label {
                    let mut builder = lcx.ranged_builder(fcx, label, 1., true);
                    builder.push_default(StyleProperty::FontSize(13.));
                    let mut layout = builder.build(label);
                    layout.break_all_lines(None);
                    layout.align(None, Alignment::Start, AlignmentOptions::default());
                    render_text(
                        painter,
                        Affine::translate((
                            x - layout.width() as f64 / 2.0,
                            y - layout.height() as f64 / 2.0,
                        )),
                        &layout,
                        &[LABEL_TEXT.into()],
                        true,
                    );
                }
            }
        }

        for fret in 1..=self.max_fret {
            let label = fret.to_string();
            let mut builder = lcx.ranged_builder(fcx, &label, 1., true);
            builder.push_default(StyleProperty::FontSize(12.));
            let mut layout = builder.build(&label);
            layout.break_all_lines(None);
            layout.align(None, Alignment::Start, AlignmentOptions::default());
            render_text(
                painter,
                Affine::translate((
                    geom.center_x(fret) - layout.width() as f64 / 2.0,
                    geom.string_y(self.string_count - 1) + 4.0,
                )),
                &layout,
                &[FRET_NUMBER.into()],
                true,
            );
        }
    }

    fn accessibility_role(&self) -> Role {
        Role::Canvas
    }

    fn accessibility(
        &mut self,
        _ctx: &mut AccessCtx<'_>,
        _props: &PropertiesRef<'_>,
        _node: &mut xilem::masonry::accesskit::Node,
    ) {
        // The board is decorative for this spike; the controls carry the state.
    }

    fn children_ids(&self) -> ChildrenIds {
        ChildrenIds::new()
    }
}

pub struct FretboardView;

pub fn fretboard() -> FretboardView {
    FretboardView
}

impl ViewMarker for FretboardView {}

impl xilem::core::View<FretNot, (), ViewCtx> for FretboardView {
    type Element = Pod<FretboardWidget>;
    type ViewState = ();

    fn build(&self, ctx: &mut ViewCtx, state: &mut FretNot) -> (Self::Element, Self::ViewState) {
        (
            ctx.with_action_widget(|ctx| ctx.create_pod(FretboardWidget::new(state))),
            (),
        )
    }

    fn rebuild(
        &self,
        _prev: &Self,
        _view_state: &mut Self::ViewState,
        _ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
        state: &mut FretNot,
    ) {
        FretboardWidget::sync(&mut element, state);
    }

    fn teardown(
        &self,
        _view_state: &mut Self::ViewState,
        _ctx: &mut ViewCtx,
        _element: Mut<'_, Self::Element>,
    ) {
    }

    fn message(
        &self,
        _view_state: &mut Self::ViewState,
        message: &mut MessageCtx,
        _element: Mut<'_, Self::Element>,
        app_state: &mut FretNot,
    ) -> MessageResult<()> {
        if let Some(action) = message.take_message::<BoardAction>() {
            match *action {
                BoardAction::Clicked(position) => app_state.select_at(position),
                BoardAction::Hovered(hover) => {
                    app_state.hover = hover;
                    app_state.hover_since = if hover.is_some() {
                        Some(app_state.now)
                    } else {
                        None
                    };
                }
            }
            MessageResult::RequestRebuild
        } else {
            MessageResult::Stale
        }
    }
}
