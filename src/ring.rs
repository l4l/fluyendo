use iced::widget::canvas;
use iced::Renderer;

use super::Color;

pub struct RingSemiPending {
    pub ratio: f32,
    pub stroke_width: f32,
    pub padding: f32,
    pub color_background: Color,
    pub color_filled: Color,
    pub color_pending: Color,
}

impl RingSemiPending {
    fn draw(&self, frame: &mut canvas::Frame) {
        let middle = self.ratio * 360.;

        let bounds = frame.size();
        let side = bounds.width.min(bounds.height);

        let radius = side / 2. - self.stroke_width - self.padding;

        let stroke = canvas::Stroke::default()
            .with_color(self.color_background.into())
            .with_width(self.stroke_width);
        let background = canvas::Path::circle(frame.center(), radius);
        frame.stroke(
            &background,
            stroke.with_width(self.stroke_width + self.padding),
        );

        let path1 = canvas::Path::new(|b| {
            b.arc(canvas::path::Arc {
                center: frame.center(),
                radius,
                start_angle: iced::Degrees(0.).into(),
                end_angle: iced::Degrees(middle).into(),
            })
        });
        frame.stroke(&path1, stroke.with_color(self.color_filled.into()));
        let path2 = canvas::Path::new(|b| {
            b.arc(canvas::path::Arc {
                center: frame.center(),
                radius,
                start_angle: iced::Degrees(middle).into(),
                end_angle: iced::Degrees(360.).into(),
            })
        });
        frame.stroke(&path2, stroke.with_color(self.color_pending.into()));
    }
}

impl<E> canvas::Program<E> for RingSemiPending {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &iced_runtime::core::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        self.draw(&mut frame);

        vec![frame.into_geometry()]
    }
}
