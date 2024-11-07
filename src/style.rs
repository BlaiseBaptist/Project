#[allow(dead_code)]
pub mod style {
    use iced::{
        widget::{canvas, container},
        Border, Theme,
    };
    pub fn text(theme: &Theme) -> container::Style {
        let palette = theme.extended_palette();
        container::Style {
            text_color: Some(palette.primary.base.text),
            background: Some(palette.primary.base.color.into()),
            border: Border {
                width: 2.0,
                color: palette.secondary.base.color,
                ..Border::default()
            },
            ..Default::default()
        }
    }
    pub fn title(theme: &Theme) -> container::Style {
        let palette = theme.extended_palette();
        container::Style {
            text_color: Some(palette.primary.strong.text),
            background: Some(palette.primary.strong.color.into()),
            border: Border {
                width: 1.0,
                color: palette.secondary.strong.color,
                ..Border::default()
            },
            ..Default::default()
        }
    }
    pub fn graph(theme: &Theme) -> container::Style {
        let palette = theme.extended_palette();
        container::Style {
            text_color: Some(palette.primary.strong.text),
            background: Some(palette.background.weak.color.into()),
            border: Border {
                width: 2.0,
                color: palette.primary.strong.color,
                ..Border::default()
            },
            ..Default::default()
        }
    }
    pub fn app_s(theme: &Theme) -> container::Style {
        let palette = theme.extended_palette();
        container::Style {
            background: Some(palette.background.weak.color.into()),
            border: Border {
                width: 1.0,
                color: palette.background.strong.color,
                ..Border::default()
            },
            ..Default::default()
        }
    }
    pub const THEME: Theme = Theme::Dark;
    pub const STROKE: canvas::Stroke = canvas::Stroke {
        line_cap: canvas::LineCap::Round,
        line_dash: canvas::LineDash {
            offset: 0,
            segments: &[1.0, 0.0],
        },
        line_join: canvas::LineJoin::Bevel,
        width: 1.0,
        style: canvas::Style::Solid(iced::theme::palette::Palette::DARK.text),
    };
}
