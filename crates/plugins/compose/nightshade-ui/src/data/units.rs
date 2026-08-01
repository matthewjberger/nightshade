use nalgebra_glm;
use nightshade_text::contract::data::font_engine::FontEngine;
use std::ops::Add;

use nalgebra_glm::Vec2;

pub trait UiValueType:
    Copy + Default + Add<Output = Self> + PartialEq + std::fmt::Debug + 'static
{
    fn mul_scalar(self, scalar: f32) -> Self;
}

impl UiValueType for f32 {
    fn mul_scalar(self, scalar: f32) -> Self {
        self * scalar
    }
}

impl UiValueType for Vec2 {
    fn mul_scalar(self, scalar: f32) -> Self {
        self * scalar
    }
}

macro_rules! define_unit {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Default, PartialEq)]
        pub struct $name<T>(pub T);
    };
}

define_unit!(Ab);
define_unit!(Rl);
define_unit!(Rw);
define_unit!(Rh);
define_unit!(Em);
define_unit!(Vp);
define_unit!(Vw);
define_unit!(Vh);

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct UiValue<T: UiValueType> {
    pub absolute: Option<T>,
    pub relative: Option<T>,
    pub relative_width: Option<T>,
    pub relative_height: Option<T>,
    pub em: Option<T>,
    pub viewport: Option<T>,
    pub viewport_width: Option<T>,
    pub viewport_height: Option<T>,
}

macro_rules! impl_unit_from {
    ($unit:ident, $field:ident) => {
        impl<T: UiValueType> From<$unit<T>> for UiValue<T> {
            fn from(value: $unit<T>) -> Self {
                let mut result = Self::default();
                result.$field = Some(value.0);
                result
            }
        }
    };
}

impl_unit_from!(Ab, absolute);
impl_unit_from!(Rl, relative);
impl_unit_from!(Rw, relative_width);
impl_unit_from!(Rh, relative_height);
impl_unit_from!(Em, em);
impl_unit_from!(Vp, viewport);
impl_unit_from!(Vw, viewport_width);
impl_unit_from!(Vh, viewport_height);

impl<T: UiValueType> Add for UiValue<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        fn merge<T: UiValueType>(a: Option<T>, b: Option<T>) -> Option<T> {
            match (a, b) {
                (Some(a), Some(b)) => Some(a + b),
                (a @ Some(_), None) => a,
                (None, b @ Some(_)) => b,
                (None, None) => None,
            }
        }

        Self {
            absolute: merge(self.absolute, rhs.absolute),
            relative: merge(self.relative, rhs.relative),
            relative_width: merge(self.relative_width, rhs.relative_width),
            relative_height: merge(self.relative_height, rhs.relative_height),
            em: merge(self.em, rhs.em),
            viewport: merge(self.viewport, rhs.viewport),
            viewport_width: merge(self.viewport_width, rhs.viewport_width),
            viewport_height: merge(self.viewport_height, rhs.viewport_height),
        }
    }
}

macro_rules! impl_uivalue_add_unit {
    ($unit:ident, $field:ident) => {
        impl<T: UiValueType> Add<$unit<T>> for UiValue<T> {
            type Output = UiValue<T>;

            fn add(self, rhs: $unit<T>) -> UiValue<T> {
                let mut result = self;
                result.$field = match result.$field {
                    Some(existing) => Some(existing + rhs.0),
                    None => Some(rhs.0),
                };
                result
            }
        }
    };
}

impl_uivalue_add_unit!(Ab, absolute);
impl_uivalue_add_unit!(Rl, relative);
impl_uivalue_add_unit!(Rw, relative_width);
impl_uivalue_add_unit!(Rh, relative_height);
impl_uivalue_add_unit!(Em, em);
impl_uivalue_add_unit!(Vp, viewport);
impl_uivalue_add_unit!(Vw, viewport_width);
impl_uivalue_add_unit!(Vh, viewport_height);

macro_rules! impl_cross_adds {
    ($left:ident, $left_field:ident => $($right:ident, $right_field:ident);+) => {
        $(
            impl<T: UiValueType> Add<$right<T>> for $left<T> {
                type Output = UiValue<T>;

                fn add(self, rhs: $right<T>) -> UiValue<T> {
                    let mut result = UiValue::default();
                    result.$left_field = Some(self.0);
                    result.$right_field = Some(rhs.0);
                    result
                }
            }
        )+
    };
}

impl_cross_adds!(Ab, absolute => Rl, relative; Rw, relative_width; Rh, relative_height; Em, em; Vp, viewport; Vw, viewport_width; Vh, viewport_height);
impl_cross_adds!(Rl, relative => Ab, absolute; Rw, relative_width; Rh, relative_height; Em, em; Vp, viewport; Vw, viewport_width; Vh, viewport_height);
impl_cross_adds!(Rw, relative_width => Ab, absolute; Rl, relative; Rh, relative_height; Em, em; Vp, viewport; Vw, viewport_width; Vh, viewport_height);
impl_cross_adds!(Rh, relative_height => Ab, absolute; Rl, relative; Rw, relative_width; Em, em; Vp, viewport; Vw, viewport_width; Vh, viewport_height);
impl_cross_adds!(Em, em => Ab, absolute; Rl, relative; Rw, relative_width; Rh, relative_height; Vp, viewport; Vw, viewport_width; Vh, viewport_height);
impl_cross_adds!(Vp, viewport => Ab, absolute; Rl, relative; Rw, relative_width; Rh, relative_height; Em, em; Vw, viewport_width; Vh, viewport_height);
impl_cross_adds!(Vw, viewport_width => Ab, absolute; Rl, relative; Rw, relative_width; Rh, relative_height; Em, em; Vp, viewport; Vh, viewport_height);
impl_cross_adds!(Vh, viewport_height => Ab, absolute; Rl, relative; Rw, relative_width; Rh, relative_height; Em, em; Vp, viewport; Vw, viewport_width);

macro_rules! impl_same_unit_add {
    ($unit:ident, $field:ident) => {
        impl<T: UiValueType> Add for $unit<T> {
            type Output = UiValue<T>;

            fn add(self, rhs: Self) -> UiValue<T> {
                let mut result = UiValue::default();
                result.$field = Some(self.0 + rhs.0);
                result
            }
        }
    };
}

impl_same_unit_add!(Ab, absolute);
impl_same_unit_add!(Rl, relative);
impl_same_unit_add!(Rw, relative_width);
impl_same_unit_add!(Rh, relative_height);
impl_same_unit_add!(Em, em);
impl_same_unit_add!(Vp, viewport);
impl_same_unit_add!(Vw, viewport_width);
impl_same_unit_add!(Vh, viewport_height);

pub struct UiEvalContext {
    pub parent_width: f32,
    pub parent_height: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub font_size: f32,
    pub absolute_scale: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiAxis {
    Horizontal,
    Vertical,
}

impl UiValue<f32> {
    pub fn evaluate(&self, context: &UiEvalContext, axis: UiAxis) -> f32 {
        let mut result = 0.0;
        if let Some(value) = self.absolute {
            result += value * context.absolute_scale;
        }
        if let Some(value) = self.relative {
            result += (value / 100.0)
                * match axis {
                    UiAxis::Horizontal => context.parent_width,
                    UiAxis::Vertical => context.parent_height,
                };
        }
        if let Some(value) = self.relative_width {
            result += (value / 100.0) * context.parent_width;
        }
        if let Some(value) = self.relative_height {
            result += (value / 100.0) * context.parent_height;
        }
        if let Some(value) = self.em {
            result += value * context.font_size;
        }
        if let Some(value) = self.viewport {
            result += (value / 100.0)
                * match axis {
                    UiAxis::Horizontal => context.viewport_width,
                    UiAxis::Vertical => context.viewport_height,
                };
        }
        if let Some(value) = self.viewport_width {
            result += (value / 100.0) * context.viewport_width;
        }
        if let Some(value) = self.viewport_height {
            result += (value / 100.0) * context.viewport_height;
        }
        result
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Length {
    Px(f32),
    Pct(f32),
    Em(f32),
    Vw(f32),
    Vh(f32),
    Vp(f32),
    Auto,
}

impl Length {
    pub fn into_axis(self, axis: UiAxis) -> UiValue<Vec2> {
        let mut value = UiValue::<Vec2>::default();
        let axis_unit = |scalar: f32| match axis {
            UiAxis::Horizontal => Vec2::new(scalar, 0.0),
            UiAxis::Vertical => Vec2::new(0.0, scalar),
        };
        match self {
            Length::Px(amount) => value.absolute = Some(axis_unit(amount)),
            Length::Pct(amount) => value.relative = Some(axis_unit(amount)),
            Length::Em(amount) => value.em = Some(axis_unit(amount)),
            Length::Vw(amount) => value.viewport_width = Some(axis_unit(amount)),
            Length::Vh(amount) => value.viewport_height = Some(axis_unit(amount)),
            Length::Vp(amount) => value.viewport = Some(axis_unit(amount)),
            Length::Auto => {}
        }
        value
    }

    pub fn is_auto(self) -> bool {
        matches!(self, Length::Auto)
    }
}

pub trait LengthExt {
    fn px(self) -> Length;
    fn pct(self) -> Length;
    fn em(self) -> Length;
    fn vw(self) -> Length;
    fn vh(self) -> Length;
    fn vp(self) -> Length;
}

macro_rules! impl_length_ext {
    ($t:ty) => {
        impl LengthExt for $t {
            fn px(self) -> Length {
                Length::Px(self as f32)
            }
            fn pct(self) -> Length {
                Length::Pct(self as f32)
            }
            fn em(self) -> Length {
                Length::Em(self as f32)
            }
            fn vw(self) -> Length {
                Length::Vw(self as f32)
            }
            fn vh(self) -> Length {
                Length::Vh(self as f32)
            }
            fn vp(self) -> Length {
                Length::Vp(self as f32)
            }
        }
    };
}

impl_length_ext!(f32);
impl_length_ext!(i32);
impl_length_ext!(u32);
impl_length_ext!(usize);

impl UiValue<Vec2> {
    pub fn evaluate(&self, context: &UiEvalContext) -> Vec2 {
        let mut result = Vec2::new(0.0, 0.0);
        if let Some(value) = self.absolute {
            result += value * context.absolute_scale;
        }
        if let Some(value) = self.relative {
            result.x += (value.x / 100.0) * context.parent_width;
            result.y += (value.y / 100.0) * context.parent_height;
        }
        if let Some(value) = self.relative_width {
            result.x += (value.x / 100.0) * context.parent_width;
            result.y += (value.y / 100.0) * context.parent_width;
        }
        if let Some(value) = self.relative_height {
            result.x += (value.x / 100.0) * context.parent_height;
            result.y += (value.y / 100.0) * context.parent_height;
        }
        if let Some(value) = self.em {
            result += value * context.font_size;
        }
        if let Some(value) = self.viewport {
            result.x += (value.x / 100.0) * context.viewport_width;
            result.y += (value.y / 100.0) * context.viewport_height;
        }
        if let Some(value) = self.viewport_width {
            result.x += (value.x / 100.0) * context.viewport_width;
            result.y += (value.y / 100.0) * context.viewport_width;
        }
        if let Some(value) = self.viewport_height {
            result.x += (value.x / 100.0) * context.viewport_height;
            result.y += (value.y / 100.0) * context.viewport_height;
        }
        result
    }
}

pub fn measure_text_width_with_kind(
    font_engine: &mut FontEngine,
    text: &str,
    font_size: f32,
    font_kind: nightshade_text::contract::data::text_data::FontKind,
) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    let line_height = font_size * 1.2;
    let buffer = font_engine.shape_buffer(text, font_size, line_height, None, font_kind);
    let mut width = 0.0f32;
    for run in buffer.layout_runs() {
        if run.line_w > width {
            width = run.line_w;
        }
    }
    width
}

pub fn measure_text_width(font_engine: &mut FontEngine, text: &str, font_size: f32) -> f32 {
    measure_text_width_with_kind(
        font_engine,
        text,
        font_size,
        nightshade_text::contract::data::text_data::FontKind::Default,
    )
}
