use gpui::{
    Bounds, Hsla, Pixels, Point, Radians, Size, TransformationMatrix, UnderlineStyle, point, px,
    size,
};
use std::ops::Range;

/// Resolve underline colour and preserve GPUI's continuous span across adjacent style runs.
pub(crate) fn coalesce_underline_spans(
    runs: impl IntoIterator<Item = (Range<usize>, UnderlineStyle, Hsla)>,
) -> Vec<(Range<usize>, UnderlineStyle)> {
    let mut spans: Vec<(Range<usize>, UnderlineStyle)> = Vec::new();
    for (range, underline, foreground) in runs {
        let underline = UnderlineStyle {
            color: Some(underline.color.unwrap_or(foreground)),
            ..underline
        };
        if let Some((previous_range, previous_underline)) = spans.last_mut()
            && previous_range.end == range.start
            && *previous_underline == underline
        {
            previous_range.end = range.end;
        } else {
            spans.push((range, underline));
        }
    }
    spans
}

/// Match GPUI's untransformed shaped-line underline baseline placement.
pub(crate) fn underline_origin_top(line_height: Pixels, ascent: Pixels, descent: Pixels) -> Pixels {
    let padding_top = (line_height - ascent - descent) / 2.;
    padding_top + ascent + descent * 0.618
}

/// Snap a logical coordinate to GPUI's nearest device pixel.
pub(crate) fn snap_to_device_pixel(value: Pixels, scale_factor: f32) -> Pixels {
    px(round_half_toward_zero(value.as_f32() * scale_factor) / scale_factor)
}

/// Match `Window::snap_stroke`, returning the snapped width in logical pixels.
pub(crate) fn snap_stroke_width(value: Pixels, scale_factor: f32) -> Pixels {
    if value == px(0.) {
        return px(0.);
    }
    px(round_half_toward_zero(value.as_f32().max(0.) * scale_factor).max(1.) / scale_factor)
}

fn round_half_toward_zero(value: f32) -> f32 {
    (value.abs() - 0.5).ceil().copysign(value)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct WavyUnderlineMetrics {
    pub(crate) thickness: Pixels,
    pub(crate) height: Pixels,
    pub(crate) amplitude: Pixels,
    pub(crate) wavelength: Pixels,
}

/// Resolve GPUI's analytic wavy-underline constants in logical pixels.
pub(crate) fn wavy_underline_metrics(thickness: Pixels, scale_factor: f32) -> WavyUnderlineMetrics {
    let thickness = snap_stroke_width(thickness, scale_factor);
    WavyUnderlineMetrics {
        thickness,
        height: thickness * 3.,
        amplitude: thickness * 0.8,
        wavelength: thickness * 9.,
    }
}

/// Match GPUI's untransformed shaped-line strikethrough baseline placement.
pub(crate) fn strikethrough_band_top(
    line_height: Pixels,
    ascent: Pixels,
    descent: Pixels,
) -> Pixels {
    let padding_top = (line_height - ascent - descent) / 2.;
    ((ascent * 0.5) + padding_top + ascent) * 0.5
}

/// Rotation geometry for an ordinary multi-line text field.
///
/// Text is laid out in `content_size` and rotated clockwise around its centre. The enclosing
/// element remains axis-aligned and uses [`Self::frame_size`] as its layout size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextareaRotation {
    angle: Radians,
    content_size: Size<Pixels>,
}

impl TextareaRotation {
    /// Create centre-based rotation geometry for untransformed text content.
    pub fn new(angle: impl Into<Radians>, content_size: Size<Pixels>) -> Self {
        let angle = angle.into();
        Self {
            angle: Radians(angle.0.rem_euclid(std::f32::consts::TAU)),
            content_size: size(
                content_size.width.max(px(0.)),
                content_size.height.max(px(0.)),
            ),
        }
    }

    /// Clockwise rotation in screen-space radians.
    pub fn angle(&self) -> Radians {
        self.angle
    }

    /// The untransformed text-layout size.
    pub fn content_size(&self) -> Size<Pixels> {
        self.content_size
    }

    /// The smallest axis-aligned size containing the rotated content.
    pub fn frame_size(&self) -> Size<Pixels> {
        let cos = self.angle.0.cos().abs();
        let sin = self.angle.0.sin().abs();
        size(
            self.content_size.width * cos + self.content_size.height * sin,
            self.content_size.width * sin + self.content_size.height * cos,
        )
    }

    pub(crate) fn resolve(&self, frame_bounds: Bounds<Pixels>) -> ResolvedTextareaRotation {
        let content_size = self.content_size;
        let content_bounds = Bounds::new(
            point(
                frame_bounds.center().x - content_size.width / 2.,
                frame_bounds.center().y - content_size.height / 2.,
            ),
            content_size,
        );
        ResolvedTextareaRotation {
            angle: self.angle,
            content_bounds,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ResolvedTextareaRotation {
    angle: Radians,
    content_bounds: Bounds<Pixels>,
}

impl ResolvedTextareaRotation {
    pub(crate) fn content_bounds(&self) -> Bounds<Pixels> {
        self.content_bounds
    }

    pub(crate) fn to_world(&self, local: Point<Pixels>) -> Point<Pixels> {
        rotate_about(local, self.content_bounds.center(), self.angle.0)
    }

    pub(crate) fn to_local(&self, world: Point<Pixels>) -> Point<Pixels> {
        rotate_about(world, self.content_bounds.center(), -self.angle.0)
    }

    pub(crate) fn bounds_to_world_aabb(&self, bounds: Bounds<Pixels>) -> Bounds<Pixels> {
        let corners = [
            bounds.origin,
            point(bounds.right(), bounds.top()),
            point(bounds.right(), bounds.bottom()),
            point(bounds.left(), bounds.bottom()),
        ]
        .map(|corner| self.to_world(corner));
        let min_x = corners
            .iter()
            .map(|corner| corner.x)
            .fold(corners[0].x, Pixels::min);
        let max_x = corners
            .iter()
            .map(|corner| corner.x)
            .fold(corners[0].x, Pixels::max);
        let min_y = corners
            .iter()
            .map(|corner| corner.y)
            .fold(corners[0].y, Pixels::min);
        let max_y = corners
            .iter()
            .map(|corner| corner.y)
            .fold(corners[0].y, Pixels::max);
        Bounds::from_corners(point(min_x, min_y), point(max_x, max_y))
    }

    /// Build the device-pixel transform used by GPUI's transformed glyph renderer.
    pub(crate) fn device_transformation(&self, scale_factor: f32) -> TransformationMatrix {
        let centre = self
            .content_bounds
            .center()
            .map(|value| value.as_f32() * scale_factor);
        let cos = self.angle.0.cos();
        let sin = self.angle.0.sin();
        TransformationMatrix {
            rotation_scale: [[cos, -sin], [sin, cos]],
            translation: [
                centre.x - cos * centre.x + sin * centre.y,
                centre.y - sin * centre.x - cos * centre.y,
            ],
        }
    }
}

fn rotate_about(position: Point<Pixels>, centre: Point<Pixels>, angle: f32) -> Point<Pixels> {
    let delta = position - centre;
    let cos = angle.cos();
    let sin = angle.sin();
    centre + point(delta.x * cos - delta.y * sin, delta.x * sin + delta.y * cos)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::radians;

    fn assert_close(actual: Pixels, expected: Pixels) {
        assert!(
            (actual - expected).abs() < px(0.001),
            "{actual:?} != {expected:?}"
        );
    }

    fn assert_point_close(actual: Point<Pixels>, expected: Point<Pixels>) {
        assert_close(actual.x, expected.x);
        assert_close(actual.y, expected.y);
    }

    #[test]
    fn strikethrough_band_matches_gpui_shaped_line_placement() {
        assert_close(strikethrough_band_top(px(20.), px(12.), px(4.)), px(10.));
        assert_close(strikethrough_band_top(px(24.), px(15.), px(5.)), px(12.25));
    }

    #[test]
    fn underline_origin_matches_gpui_shaped_line_placement() {
        assert_close(underline_origin_top(px(20.), px(12.), px(4.)), px(16.472));
        assert_close(underline_origin_top(px(24.), px(15.), px(5.)), px(20.09));
    }

    #[test]
    fn wavy_underline_metrics_match_gpui_device_snapping_at_one_and_two_x() {
        let one_x = wavy_underline_metrics(px(1.4), 1.);
        assert_close(one_x.thickness, px(1.));
        assert_close(one_x.height, px(3.));
        assert_close(one_x.amplitude, px(0.8));
        assert_close(one_x.wavelength, px(9.));
        assert_close(snap_to_device_pixel(px(10.5), 1.), px(10.));

        let two_x = wavy_underline_metrics(px(1.4), 2.);
        assert_close(two_x.thickness, px(1.5));
        assert_close(two_x.height, px(4.5));
        assert_close(two_x.amplitude, px(1.2));
        assert_close(two_x.wavelength, px(13.5));
        assert_close(snap_to_device_pixel(px(10.3), 2.), px(10.5));
    }

    #[test]
    fn adjacent_equal_effective_underlines_form_one_continuous_span() {
        let explicit_red = UnderlineStyle {
            thickness: px(1.),
            color: Some(gpui::red()),
            wavy: true,
        };
        let inherited = UnderlineStyle {
            thickness: px(1.),
            color: None,
            wavy: true,
        };
        let spans = coalesce_underline_spans([
            (0..3, explicit_red, gpui::black()),
            (3..7, explicit_red, gpui::white()),
            (7..9, inherited, gpui::blue()),
            (9..11, inherited, gpui::blue()),
            (12..14, inherited, gpui::blue()),
        ]);

        assert_eq!(spans.len(), 3);
        assert_eq!(spans[0], (0..7, explicit_red));
        assert_eq!(
            spans[1],
            (
                7..11,
                UnderlineStyle {
                    color: Some(gpui::blue()),
                    ..inherited
                }
            )
        );
        assert_eq!(spans[2].0, 12..14);
    }

    #[test]
    fn identity_rotation_preserves_size_and_points() {
        let rotation = TextareaRotation::new(radians(0.), size(px(120.), px(40.)));
        assert_eq!(rotation.frame_size(), rotation.content_size());
        let resolved =
            rotation.resolve(Bounds::new(point(px(10.), px(20.)), rotation.frame_size()));
        let sample = point(px(34.), px(47.));
        assert_eq!(
            resolved.content_bounds(),
            Bounds::new(point(px(10.), px(20.)), size(px(120.), px(40.)))
        );
        assert_point_close(resolved.to_world(sample), sample);
        assert_point_close(resolved.to_local(sample), sample);
    }

    #[test]
    fn non_square_rotation_resolves_aabb_and_inverse_geometry() {
        let rotation = TextareaRotation::new(
            radians(std::f32::consts::FRAC_PI_6),
            size(px(120.), px(40.)),
        );
        let frame_size = rotation.frame_size();
        assert_close(frame_size.width, px(123.92305));
        assert_close(frame_size.height, px(94.64102));

        let frame = Bounds::new(point(px(50.), px(70.)), frame_size);
        let resolved = rotation.resolve(frame);
        let content = resolved.content_bounds();
        let local = point(content.left() + px(17.), content.top() + px(9.));
        assert_point_close(resolved.to_local(resolved.to_world(local)), local);
        let aabb = resolved.bounds_to_world_aabb(content);
        assert_point_close(aabb.origin, frame.origin);
        assert_close(aabb.size.width, frame.size.width);
        assert_close(aabb.size.height, frame.size.height);
    }

    #[test]
    fn rotation_normalizes_and_clamps_invalid_negative_size() {
        let rotation = TextareaRotation::new(
            radians(std::f32::consts::TAU + std::f32::consts::FRAC_PI_2),
            size(px(-4.), px(20.)),
        );
        assert_close(px(rotation.angle().0), px(std::f32::consts::FRAC_PI_2));
        assert_eq!(rotation.content_size(), size(px(0.), px(20.)));
        assert_close(rotation.frame_size().width, px(20.));
        assert_close(rotation.frame_size().height, px(0.));
    }

    #[test]
    fn thirty_degree_click_maps_back_to_local_text_position() {
        let rotation = TextareaRotation::new(
            radians(std::f32::consts::FRAC_PI_6),
            size(px(160.), px(60.)),
        );
        let resolved =
            rotation.resolve(Bounds::new(point(px(20.), px(30.)), rotation.frame_size()));
        let local_click = resolved.content_bounds().origin + point(px(41.), px(18.));
        assert_point_close(
            resolved.to_local(resolved.to_world(local_click)),
            local_click,
        );
    }

    #[test]
    fn thirty_degree_selection_and_caret_aabbs_contain_every_rotated_corner() {
        let rotation = TextareaRotation::new(
            radians(std::f32::consts::FRAC_PI_6),
            size(px(180.), px(80.)),
        );
        let resolved =
            rotation.resolve(Bounds::new(point(px(12.), px(24.)), rotation.frame_size()));
        for bounds in [
            Bounds::new(
                resolved.content_bounds().origin + point(px(20.), px(10.)),
                size(px(90.), px(22.)),
            ),
            Bounds::new(
                resolved.content_bounds().origin + point(px(74.), px(35.)),
                size(px(2.), px(17.)),
            ),
        ] {
            let aabb = resolved.bounds_to_world_aabb(bounds);
            for corner in [
                bounds.origin,
                point(bounds.right(), bounds.top()),
                point(bounds.right(), bounds.bottom()),
                point(bounds.left(), bounds.bottom()),
            ] {
                let world = resolved.to_world(corner);
                assert!(world.x >= aabb.left() - px(0.001));
                assert!(world.x <= aabb.right() + px(0.001));
                assert!(world.y >= aabb.top() - px(0.001));
                assert!(world.y <= aabb.bottom() + px(0.001));
            }
        }
    }

    #[test]
    fn empty_bounds_aabb_is_the_single_rotated_corner() {
        let rotation = TextareaRotation::new(
            radians(std::f32::consts::FRAC_PI_6),
            size(px(120.), px(40.)),
        );
        let resolved = rotation.resolve(Bounds::new(point(px(0.), px(0.)), rotation.frame_size()));
        let corner = resolved.content_bounds().origin + point(px(11.), px(7.));
        let aabb = resolved.bounds_to_world_aabb(Bounds::new(corner, size(px(0.), px(0.))));
        assert_point_close(aabb.origin, resolved.to_world(corner));
        assert_eq!(aabb.size, size(px(0.), px(0.)));
    }

    #[test]
    fn empty_frame_aabb_corners_map_outside_rotated_content() {
        let rotation = TextareaRotation::new(
            radians(std::f32::consts::FRAC_PI_6),
            size(px(160.), px(60.)),
        );
        let frame = Bounds::new(point(px(20.), px(30.)), rotation.frame_size());
        let resolved = rotation.resolve(frame);
        for corner in [
            frame.origin,
            point(frame.right(), frame.top()),
            point(frame.right(), frame.bottom()),
            point(frame.left(), frame.bottom()),
        ] {
            assert!(
                !resolved
                    .content_bounds()
                    .contains(&resolved.to_local(corner))
            );
        }
    }
}
