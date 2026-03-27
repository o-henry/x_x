use crate::color::LinearRgba;
use crate::customglyph::*;
use crate::tabbar::{TabBarItem, TabEntry};
use crate::termwindow::box_model::*;
use crate::termwindow::render::corners::*;

use crate::termwindow::render::window_buttons::window_button_element;
use crate::termwindow::{UIItem, UIItemType};
use crate::utilsprites::RenderMetrics;
use config::{Dimension, DimensionContext, TabBarColors};
use std::rc::Rc;
use wezterm_font::LoadedFont;
use wezterm_term::color::{ColorAttribute, ColorPalette};
use wezterm_term::Progress;
use window::{IntegratedTitleButtonAlignment, IntegratedTitleButtonStyle};

const X_BUTTON: &[Poly] = &[
    Poly {
        path: &[
            PolyCommand::MoveTo(BlockCoord::One, BlockCoord::Zero),
            PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::One),
        ],
        intensity: BlockAlpha::Full,
        style: PolyStyle::Outline,
    },
    Poly {
        path: &[
            PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Zero),
            PolyCommand::LineTo(BlockCoord::One, BlockCoord::One),
        ],
        intensity: BlockAlpha::Full,
        style: PolyStyle::Outline,
    },
];

const PLUS_BUTTON: &[Poly] = &[
    Poly {
        path: &[
            PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
            PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::One),
        ],
        intensity: BlockAlpha::Full,
        style: PolyStyle::Outline,
    },
    Poly {
        path: &[
            PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
            PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
        ],
        intensity: BlockAlpha::Full,
        style: PolyStyle::Outline,
    },
];

const PROGRESS_CIRCLE_POLY: &[Poly] = &[Poly {
    path: &[PolyCommand::Circle {
        center: (BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
        radius: BlockCoord::Frac(1, 2),
    }],
    intensity: BlockAlpha::Full,
    style: PolyStyle::Fill,
}];

const PROGRESS_DOT_SIZE: f32 = 14.0;
const QUIET_BAR_LINE_HEIGHT: f64 = 1.42;
const QUIET_TAB_LINE_HEIGHT: f64 = 1.52;
const QUIET_TAB_TOP_MARGIN_CELLS: f32 = 0.08;
const QUIET_TAB_TOP_PADDING_CELLS: f32 = 0.08;
const QUIET_TAB_BOTTOM_PADDING_CELLS: f32 = 0.12;
const QUIET_TAB_CORNER_CELLS: f32 = 0.24;
const QUIET_TAB_HORIZONTAL_PADDING_EXTRA_PX: f32 = 1.0;
const QUIET_OPERATOR_MARKER_MARGIN_LEFT_CELLS: f32 = 0.08;
const QUIET_OPERATOR_MARKER_PADDING_X_CELLS: f32 = 0.28;
const QUIET_OPERATOR_MARKER_PADDING_TOP_CELLS: f32 = 0.08;
const QUIET_STATUS_LEFT_PADDING_CELLS: f32 = 0.35;

fn quiet_tab_bottom_padding() -> Dimension {
    Dimension::Cells(QUIET_TAB_BOTTOM_PADDING_CELLS)
}

impl crate::TermWindow {
    pub fn invalidate_fancy_tab_bar(&mut self) {
        self.fancy_tab_bar.take();
    }

    pub fn build_fancy_tab_bar(&self, palette: &ColorPalette) -> anyhow::Result<ComputedElement> {
        let tab_bar_height = self.tab_bar_pixel_height()?;
        let font = self.fonts.title_font()?;
        let metrics = RenderMetrics::with_font_metrics(&font.metrics());
        let items = self.tab_bar.items();
        let colors = self
            .config
            .colors
            .as_ref()
            .and_then(|c| c.tab_bar.as_ref())
            .cloned()
            .unwrap_or_else(TabBarColors::default);

        let mut left_status = vec![];
        let mut left_eles = vec![];
        let mut right_eles = vec![];
        let bar_colors = ElementColors {
            border: BorderColor::default(),
            bg: if self.focused.is_some() {
                self.config.window_frame.active_titlebar_bg
            } else {
                self.config.window_frame.inactive_titlebar_bg
            }
            .to_linear()
            .into(),
            text: if self.focused.is_some() {
                self.config.window_frame.active_titlebar_fg
            } else {
                self.config.window_frame.inactive_titlebar_fg
            }
            .to_linear()
            .into(),
        };
        let tab_bottom_padding = quiet_tab_bottom_padding();

        let tab_padding_h = Dimension::Pixels(
            (0.32 * metrics.cell_size.width as f32) + QUIET_TAB_HORIZONTAL_PADDING_EXTRA_PX,
        );

        let item_to_elem = |item: &TabEntry| -> Element {
            let element = Element::with_line(&font, &item.title, palette);

            let bg_color = item
                .title
                .get_cell(0)
                .and_then(|c| match c.attrs().background() {
                    ColorAttribute::Default => None,
                    col => Some(palette.resolve_bg(col)),
                });
            let fg_color = item
                .title
                .get_cell(0)
                .and_then(|c| match c.attrs().foreground() {
                    ColorAttribute::Default => None,
                    col => Some(palette.resolve_fg(col)),
                });

            let new_tab = colors.new_tab();
            let new_tab_hover = colors.new_tab_hover();
            let active_tab = colors.active_tab();

            match item.item {
                TabBarItem::RightStatus | TabBarItem::LeftStatus | TabBarItem::None => element
                    .item_type(UIItemType::TabBar(TabBarItem::None))
                    .line_height(Some(QUIET_BAR_LINE_HEIGHT))
                    .margin(BoxDimension {
                        left: Dimension::Cells(0.),
                        right: Dimension::Cells(0.),
                        top: Dimension::Cells(0.0),
                        bottom: Dimension::Cells(0.),
                    })
                    .padding(BoxDimension {
                        left: Dimension::Cells(QUIET_STATUS_LEFT_PADDING_CELLS),
                        right: Dimension::Cells(0.),
                        top: Dimension::Cells(0.),
                        bottom: Dimension::Cells(0.),
                    })
                    .border(BoxDimension::new(Dimension::Pixels(0.)))
                    .colors(bar_colors.clone()),
                TabBarItem::OperatorMarker { .. } => element
                    .item_type(UIItemType::TabBar(item.item.clone()))
                    .line_height(Some(QUIET_BAR_LINE_HEIGHT))
                    .margin(BoxDimension {
                        left: Dimension::Cells(QUIET_OPERATOR_MARKER_MARGIN_LEFT_CELLS),
                        right: Dimension::Cells(0.),
                        top: Dimension::Cells(QUIET_TAB_TOP_MARGIN_CELLS),
                        bottom: Dimension::Cells(0.),
                    })
                    .padding(BoxDimension {
                        left: Dimension::Cells(QUIET_OPERATOR_MARKER_PADDING_X_CELLS),
                        right: Dimension::Cells(QUIET_OPERATOR_MARKER_PADDING_X_CELLS),
                        top: Dimension::Cells(QUIET_OPERATOR_MARKER_PADDING_TOP_CELLS),
                        bottom: tab_bottom_padding,
                    })
                    .border(BoxDimension::new(Dimension::Pixels(0.)))
                    .colors(ElementColors {
                        border: BorderColor::default(),
                        bg: LinearRgba::with_components(1.0, 1.0, 1.0, 0.04).into(),
                        text: bar_colors.text.clone(),
                    }),
                TabBarItem::NewTabButton => Element::new(
                    &font,
                    ElementContent::Poly {
                        line_width: metrics.underline_height.max(2),
                        poly: SizedPoly {
                            poly: PLUS_BUTTON,
                            width: Dimension::Pixels(metrics.cell_size.height as f32 / 2.),
                            height: Dimension::Pixels(metrics.cell_size.height as f32 / 2.),
                        },
                    },
                )
                .vertical_align(VerticalAlign::Middle)
                .item_type(UIItemType::TabBar(item.item.clone()))
                .margin(BoxDimension {
                    left: Dimension::Cells(0.24),
                    right: Dimension::Cells(0.),
                    top: Dimension::Cells(QUIET_TAB_TOP_MARGIN_CELLS),
                    bottom: Dimension::Cells(0.),
                })
                .padding(BoxDimension {
                    left: Dimension::Cells(0.32),
                    right: Dimension::Cells(0.32),
                    top: Dimension::Cells(QUIET_TAB_TOP_PADDING_CELLS),
                    bottom: tab_bottom_padding,
                })
                .border(BoxDimension::new(Dimension::Pixels(0.)))
                .colors(ElementColors {
                    border: BorderColor::default(),
                    bg: new_tab.bg_color.to_linear().into(),
                    text: new_tab.fg_color.to_linear().into(),
                })
                .hover_colors(Some(ElementColors {
                    border: BorderColor::default(),
                    bg: new_tab_hover.bg_color.to_linear().into(),
                    text: new_tab_hover.fg_color.to_linear().into(),
                })),
                TabBarItem::Tab { active, .. } if active => element
                    .vertical_align(VerticalAlign::Bottom)
                    .item_type(UIItemType::TabBar(item.item.clone()))
                    .line_height(Some(QUIET_TAB_LINE_HEIGHT))
                    .margin(BoxDimension {
                        left: Dimension::Cells(0.),
                        right: Dimension::Cells(0.),
                        top: Dimension::Cells(QUIET_TAB_TOP_MARGIN_CELLS),
                        bottom: Dimension::Cells(0.),
                    })
                    .padding(BoxDimension {
                        left: tab_padding_h,
                        right: tab_padding_h,
                        top: Dimension::Cells(QUIET_TAB_TOP_PADDING_CELLS),
                        bottom: tab_bottom_padding,
                    })
                    .border(BoxDimension::new(Dimension::Pixels(0.)))
                    .border_corners(Some(Corners {
                        top_left: SizedPoly {
                            width: Dimension::Cells(QUIET_TAB_CORNER_CELLS),
                            height: Dimension::Cells(QUIET_TAB_CORNER_CELLS),
                            poly: TOP_LEFT_ROUNDED_CORNER,
                        },
                        top_right: SizedPoly {
                            width: Dimension::Cells(QUIET_TAB_CORNER_CELLS),
                            height: Dimension::Cells(QUIET_TAB_CORNER_CELLS),
                            poly: TOP_RIGHT_ROUNDED_CORNER,
                        },
                        bottom_left: SizedPoly::none(),
                        bottom_right: SizedPoly::none(),
                    }))
                    .colors(ElementColors {
                        border: BorderColor::default(),
                        bg: bg_color
                            .unwrap_or_else(|| active_tab.bg_color.into())
                            .to_linear()
                            .into(),
                        text: fg_color
                            .unwrap_or_else(|| active_tab.fg_color.into())
                            .to_linear()
                            .into(),
                    }),
                TabBarItem::Tab { .. } => element
                    .vertical_align(VerticalAlign::Bottom)
                    .item_type(UIItemType::TabBar(item.item.clone()))
                    .line_height(Some(QUIET_TAB_LINE_HEIGHT))
                    .margin(BoxDimension {
                        left: Dimension::Cells(0.),
                        right: Dimension::Cells(0.),
                        top: Dimension::Cells(QUIET_TAB_TOP_MARGIN_CELLS),
                        bottom: Dimension::Cells(0.),
                    })
                    .padding(BoxDimension {
                        left: tab_padding_h,
                        right: tab_padding_h,
                        top: Dimension::Cells(QUIET_TAB_TOP_PADDING_CELLS),
                        bottom: tab_bottom_padding,
                    })
                    .border(BoxDimension::new(Dimension::Pixels(0.)))
                    .border_corners(Some(Corners {
                        top_left: SizedPoly {
                            width: Dimension::Cells(QUIET_TAB_CORNER_CELLS),
                            height: Dimension::Cells(QUIET_TAB_CORNER_CELLS),
                            poly: TOP_LEFT_ROUNDED_CORNER,
                        },
                        top_right: SizedPoly {
                            width: Dimension::Cells(QUIET_TAB_CORNER_CELLS),
                            height: Dimension::Cells(QUIET_TAB_CORNER_CELLS),
                            poly: TOP_RIGHT_ROUNDED_CORNER,
                        },
                        bottom_left: SizedPoly {
                            width: Dimension::Cells(0.),
                            height: Dimension::Cells(0.33),
                            poly: &[],
                        },
                        bottom_right: SizedPoly {
                            width: Dimension::Cells(0.),
                            height: Dimension::Cells(0.33),
                            poly: &[],
                        },
                    }))
                    .colors({
                        let inactive_tab = colors.inactive_tab();
                        let bg = bg_color
                            .unwrap_or_else(|| inactive_tab.bg_color.into())
                            .to_linear();
                        ElementColors {
                            border: BorderColor::default(),
                            bg: bg.into(),
                            text: fg_color
                                .unwrap_or_else(|| inactive_tab.fg_color.into())
                                .to_linear()
                                .into(),
                        }
                    })
                    .hover_colors({
                        let inactive_tab_hover = colors.inactive_tab_hover();
                        Some(ElementColors {
                            border: BorderColor::default(),
                            bg: bg_color
                                .unwrap_or_else(|| inactive_tab_hover.bg_color.into())
                                .to_linear()
                                .into(),
                            text: fg_color
                                .unwrap_or_else(|| inactive_tab_hover.fg_color.into())
                                .to_linear()
                                .into(),
                        })
                    }),
                TabBarItem::WindowButton(button) => window_button_element(
                    button,
                    self.window_state.contains(window::WindowState::MAXIMIZED),
                    &font,
                    &metrics,
                    &self.config,
                ),
            }
        };

        let num_tabs: f32 = items
            .iter()
            .map(|item| match item.item {
                TabBarItem::NewTabButton | TabBarItem::Tab { .. } => 1.,
                _ => 0.,
            })
            .sum();
        let max_tab_width = ((self.dimensions.pixel_width as f32 / num_tabs)
            - (1.5 * metrics.cell_size.width as f32))
            .max(0.);

        // Reserve space for the native titlebar buttons
        if self
            .config
            .window_decorations
            .contains(::window::WindowDecorations::INTEGRATED_BUTTONS)
            && self.config.integrated_title_button_style == IntegratedTitleButtonStyle::MacOsNative
            && !self.layout_is_effective_fullscreen()
        {
            left_status.push(
                Element::new(&font, ElementContent::Text("".to_string())).margin(BoxDimension {
                    left: Dimension::Cells(4.0), // FIXME: determine exact width of macos ... buttons
                    right: Dimension::Cells(0.),
                    top: Dimension::Cells(0.),
                    bottom: Dimension::Cells(0.),
                }),
            );
        }

        for item in items {
            match item.item {
                TabBarItem::LeftStatus => left_status.push(item_to_elem(item)),
                TabBarItem::None | TabBarItem::RightStatus => right_eles.push(item_to_elem(item)),
                TabBarItem::WindowButton(_) => {
                    if self.config.integrated_title_button_alignment
                        == IntegratedTitleButtonAlignment::Left
                    {
                        left_eles.push(item_to_elem(item))
                    } else {
                        right_eles.push(item_to_elem(item))
                    }
                }
                TabBarItem::Tab { tab_idx, active } => {
                    let mut elem = item_to_elem(item);
                    elem.max_width = Some(Dimension::Pixels(max_tab_width));
                    elem.content = match elem.content {
                        ElementContent::Text(_) => unreachable!(),
                        ElementContent::Poly { .. } => unreachable!(),
                        ElementContent::Children(mut kids) => {
                            if item.progress != Progress::None {
                                let dot_color = match &item.progress {
                                    Progress::Error(_) => {
                                        LinearRgba::with_components(1.0, 0.27, 0.27, 0.85)
                                    }
                                    Progress::Percentage(_) | Progress::Indeterminate => {
                                        LinearRgba::with_components(0.27, 0.85, 0.27, 0.85)
                                    }
                                    Progress::None => unreachable!(),
                                };
                                let dot = Element::new(
                                    &font,
                                    ElementContent::Poly {
                                        line_width: 0,
                                        poly: SizedPoly {
                                            poly: PROGRESS_CIRCLE_POLY,
                                            width: Dimension::Pixels(PROGRESS_DOT_SIZE),
                                            height: Dimension::Pixels(PROGRESS_DOT_SIZE),
                                        },
                                    },
                                )
                                .vertical_align(VerticalAlign::Middle)
                                .colors(ElementColors {
                                    border: BorderColor::default(),
                                    bg: LinearRgba::TRANSPARENT.into(),
                                    text: dot_color.into(),
                                })
                                .margin(BoxDimension {
                                    left: Dimension::Cells(0.),
                                    right: Dimension::Pixels(0.),
                                    top: Dimension::Cells(0.),
                                    bottom: Dimension::Cells(0.),
                                });
                                kids.insert(0, dot);
                            }
                            if self.config.show_close_tab_button_in_tabs {
                                kids.push(make_x_button(&font, &metrics, &colors, tab_idx, active));
                            }
                            ElementContent::Children(kids)
                        }
                    };
                    left_eles.push(elem);
                }
                _ => left_eles.push(item_to_elem(item)),
            }
        }

        let mut children = vec![];

        if !left_status.is_empty() {
            children.push(
                Element::new(&font, ElementContent::Children(left_status))
                    .colors(bar_colors.clone()),
            );
        }

        let window_buttons_at_left = self
            .config
            .window_decorations
            .contains(window::WindowDecorations::INTEGRATED_BUTTONS)
            && (self.config.integrated_title_button_alignment
                == IntegratedTitleButtonAlignment::Left
                || self.config.integrated_title_button_style
                    == IntegratedTitleButtonStyle::MacOsNative);

        let is_fullscreen = self.layout_is_effective_fullscreen();
        let left_padding = if is_fullscreen {
            Dimension::Pixels(self.content_left_inset())
        } else if window_buttons_at_left {
            if self.config.integrated_title_button_style == IntegratedTitleButtonStyle::MacOsNative
            {
                Dimension::Pixels(70.0)
            } else {
                Dimension::Pixels(0.0)
            }
        } else {
            Dimension::Cells(0.5)
        };

        children.push(
            Element::new(&font, ElementContent::Children(left_eles))
                .vertical_align(VerticalAlign::Bottom)
                .colors(bar_colors.clone())
                .padding(BoxDimension {
                    left: left_padding,
                    right: Dimension::Cells(0.),
                    top: Dimension::Cells(0.),
                    bottom: Dimension::Cells(0.),
                })
                .zindex(1),
        );
        children.push(
            Element::new(&font, ElementContent::Children(right_eles))
                .colors(bar_colors.clone())
                .float(Float::Right),
        );

        let content = ElementContent::Children(children);

        let tabs = Element::new(&font, content)
            .display(DisplayType::Block)
            .item_type(UIItemType::TabBar(TabBarItem::None))
            .min_width(Some(Dimension::Pixels(self.dimensions.pixel_width as f32)))
            .min_height(Some(Dimension::Pixels(tab_bar_height)))
            .vertical_align(VerticalAlign::Bottom)
            .colors(bar_colors);

        let border = self.get_os_border();
        // In fullscreen, start from 0 since left_padding already handles alignment
        let bounds_left = if is_fullscreen {
            0.0
        } else {
            border.left.get() as f32
        };
        let bounds_width = if is_fullscreen {
            self.dimensions.pixel_width as f32
        } else {
            self.dimensions.pixel_width as f32 - (border.left + border.right).get() as f32
        };

        let mut computed = self.compute_element(
            &LayoutContext {
                height: DimensionContext {
                    dpi: self.dimensions.dpi as f32,
                    pixel_max: self.dimensions.pixel_height as f32,
                    pixel_cell: metrics.cell_size.height as f32,
                },
                width: DimensionContext {
                    dpi: self.dimensions.dpi as f32,
                    pixel_max: self.dimensions.pixel_width as f32,
                    pixel_cell: metrics.cell_size.width as f32,
                },
                bounds: euclid::rect(bounds_left, 0., bounds_width, tab_bar_height),
                metrics: &metrics,
                gl_state: self.render_state.as_ref().unwrap(),
                zindex: 10,
            },
            &tabs,
        )?;

        computed.translate(euclid::vec2(
            0.,
            if self.config.tab_bar_at_bottom {
                self.dimensions.pixel_height as f32
                    - (computed.bounds.height() + border.bottom.get() as f32)
            } else {
                border.top.get() as f32
            },
        ));

        Ok(computed)
    }

    pub fn paint_fancy_tab_bar(&self) -> anyhow::Result<Vec<UIItem>> {
        let computed = self.fancy_tab_bar.as_ref().ok_or_else(|| {
            anyhow::anyhow!("paint_fancy_tab_bar called but fancy_tab_bar is None")
        })?;
        let ui_items = computed.ui_items();

        let gl_state = self.render_state.as_ref().unwrap();
        self.render_element(&computed, gl_state, None)?;

        Ok(ui_items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fancy_tabbar_quiet_chrome_metrics_reduce_tab_density_from_phase_six_values() {
        assert!(QUIET_BAR_LINE_HEIGHT < 1.75);
        assert!(QUIET_TAB_LINE_HEIGHT < 1.75);
        assert!(QUIET_TAB_TOP_MARGIN_CELLS < 0.2);
        assert!(QUIET_TAB_TOP_PADDING_CELLS < 0.2);
        assert!(QUIET_TAB_HORIZONTAL_PADDING_EXTRA_PX < 4.0);
    }

    #[test]
    fn fancy_tabbar_operator_marker_chip_is_flatter_than_the_old_capsule_style() {
        assert!(QUIET_OPERATOR_MARKER_MARGIN_LEFT_CELLS < 0.18);
        assert!(QUIET_OPERATOR_MARKER_PADDING_X_CELLS < 0.45);
        assert!(QUIET_OPERATOR_MARKER_PADDING_TOP_CELLS < 0.15);
        assert!(QUIET_TAB_CORNER_CELLS < 0.5);
    }
}

fn make_x_button(
    font: &Rc<LoadedFont>,
    metrics: &RenderMetrics,
    colors: &TabBarColors,
    tab_idx: usize,
    active: bool,
) -> Element {
    Element::new(
        &font,
        ElementContent::Poly {
            line_width: metrics.underline_height.max(2),
            poly: SizedPoly {
                poly: X_BUTTON,
                width: Dimension::Pixels(metrics.cell_size.height as f32 / 2.),
                height: Dimension::Pixels(metrics.cell_size.height as f32 / 2.),
            },
        },
    )
    // Ensure that we draw our background over the
    // top of the rest of the tab contents
    .zindex(1)
    .vertical_align(VerticalAlign::Middle)
    .float(Float::Right)
    .item_type(UIItemType::CloseTab(tab_idx))
    .hover_colors({
        let inactive_tab_hover = colors.inactive_tab_hover();
        let active_tab = colors.active_tab();

        Some(ElementColors {
            border: BorderColor::default(),
            bg: (if active {
                inactive_tab_hover.bg_color
            } else {
                active_tab.bg_color
            })
            .to_linear()
            .into(),
            text: (if active {
                inactive_tab_hover.fg_color
            } else {
                active_tab.fg_color
            })
            .to_linear()
            .into(),
        })
    })
    .padding(BoxDimension {
        left: Dimension::Cells(0.25),
        right: Dimension::Cells(0.25),
        top: Dimension::Cells(0.25),
        bottom: Dimension::Cells(0.25),
    })
    .margin(BoxDimension {
        left: Dimension::Cells(0.5),
        right: Dimension::Cells(0.),
        top: Dimension::Cells(0.),
        bottom: Dimension::Cells(0.),
    })
}
