use crate::commands::CommandDef;
use crate::customglyph::{BlockAlpha, BlockCoord, Poly, PolyCommand, PolyStyle};
use crate::termwindow::box_model::*;
use crate::termwindow::render::corners::{
    BOTTOM_LEFT_ROUNDED_CORNER, BOTTOM_RIGHT_ROUNDED_CORNER, TOP_LEFT_ROUNDED_CORNER,
    TOP_RIGHT_ROUNDED_CORNER,
};
use crate::termwindow::render::forces_opaque_kaku_tui_window_background;
use crate::termwindow::{
    DimensionContext, OperatorNavItem, RenderFrame, TermWindowNotif, UIItemType,
};
use crate::utilsprites::RenderMetrics;
use ::window::bitmaps::atlas::OutOfTextureSpace;
use ::window::WindowOps;
use anyhow::Context;
use config::Dimension;
use smol::Timer;
use std::time::{Duration, Instant};
use wezterm_font::ClearShapeCache;
use window::color::LinearRgba;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllowImage {
    Yes,
    Scale(usize),
    No,
}

const STATUS_DOT_SIZE: f32 = 14.0;
const BROADCAST_ICON_SIZE: f32 = 24.0;
const OPERATOR_NAV_PANEL_BG: LinearRgba = LinearRgba::with_components(0.050, 0.053, 0.060, 0.988);
const OPERATOR_NAV_BORDER: LinearRgba = LinearRgba::with_components(0.88, 0.88, 0.96, 0.045);
const OPERATOR_NAV_TEXT: LinearRgba = LinearRgba::with_components(0.73, 0.74, 0.79, 1.0);
const OPERATOR_NAV_DIM: LinearRgba = LinearRgba::with_components(0.34, 0.35, 0.40, 1.0);
const OPERATOR_NAV_ACTIVE_BG: LinearRgba = LinearRgba::with_components(0.12, 0.13, 0.17, 0.96);
const OPERATOR_NAV_ACTIVE_TEXT: LinearRgba = LinearRgba::with_components(0.97, 0.97, 1.0, 1.0);
const OPERATOR_NAV_ACCENT: LinearRgba = LinearRgba::with_components(0.49, 0.56, 0.90, 1.0);
const OPERATOR_NAV_HEADER_TEXT: LinearRgba = LinearRgba::with_components(0.91, 0.92, 0.96, 1.0);
const SHORTCUT_STRIP_BG: LinearRgba = LinearRgba::with_components(0.058, 0.060, 0.068, 0.992);
const SHORTCUT_STRIP_BORDER: LinearRgba = LinearRgba::with_components(0.88, 0.88, 0.96, 0.045);
const SHORTCUT_STRIP_TEXT: LinearRgba = LinearRgba::with_components(0.82, 0.84, 0.89, 1.0);
const SHORTCUT_STRIP_DIM: LinearRgba = LinearRgba::with_components(0.53, 0.55, 0.60, 1.0);

static ACTIVE_PANE_INDICATOR_POLY: &[Poly] = &[Poly {
    path: &[PolyCommand::Circle {
        center: (BlockCoord::Frac(1, 2), BlockCoord::Frac(1, 2)),
        radius: BlockCoord::Frac(1, 2),
    }],
    intensity: BlockAlpha::Full,
    style: PolyStyle::Fill,
}];

static BROADCAST_INDICATOR_POLY: &[Poly] = &[
    Poly {
        path: &[PolyCommand::Circle {
            center: (BlockCoord::Frac(1, 4), BlockCoord::Frac(1, 2)),
            radius: BlockCoord::Frac(1, 8),
        }],
        intensity: BlockAlpha::Full,
        style: PolyStyle::Fill,
    },
    Poly {
        path: &[
            PolyCommand::MoveTo(BlockCoord::Frac(5, 12), BlockCoord::Frac(4, 12)),
            PolyCommand::QuadTo {
                control: (BlockCoord::Frac(8, 12), BlockCoord::Frac(1, 2)),
                to: (BlockCoord::Frac(5, 12), BlockCoord::Frac(8, 12)),
            },
        ],
        intensity: BlockAlpha::Full,
        style: PolyStyle::Outline,
    },
    Poly {
        path: &[
            PolyCommand::MoveTo(BlockCoord::Frac(7, 12), BlockCoord::Frac(2, 12)),
            PolyCommand::QuadTo {
                control: (BlockCoord::Frac(11, 12), BlockCoord::Frac(1, 2)),
                to: (BlockCoord::Frac(7, 12), BlockCoord::Frac(10, 12)),
            },
        ],
        intensity: BlockAlpha::Full,
        style: PolyStyle::Outline,
    },
];

fn toast_colors_for_palette(
    palette: &wezterm_term::color::ColorPalette,
    alpha: f32,
) -> (LinearRgba, LinearRgba) {
    if crate::termwindow::is_light_color(&palette.background) {
        let bg_linear = palette.colors.0[3].to_linear();
        (
            LinearRgba(bg_linear.0, bg_linear.1, bg_linear.2, 0.9 * alpha),
            LinearRgba(0.1, 0.1, 0.1, alpha),
        )
    } else {
        let bg_linear = palette.colors.0[13].to_linear();
        (
            LinearRgba(bg_linear.0, bg_linear.1, bg_linear.2, 0.9 * alpha),
            LinearRgba(1.0, 1.0, 1.0, alpha),
        )
    }
}

impl crate::TermWindow {
    fn paint_shortcut_strip(&mut self) -> anyhow::Result<()> {
        let font = self.fonts.title_font()?;
        let metrics = &self.render_metrics;
        let dimensions = self.dimensions;
        let border = self.get_os_border();
        let bottom_inset = border.bottom.get() as f32
            + if self.show_tab_bar && self.config.tab_bar_at_bottom {
                self.tab_bar_pixel_height().unwrap_or(0.0)
            } else {
                0.0
            };
        let entries = CommandDef::shell_shortcut_strip_entries(&self.config);
        if entries.is_empty() {
            return Ok(());
        }

        let mut children = Vec::with_capacity(entries.len() * 2);
        for (idx, entry) in entries.into_iter().enumerate() {
            if idx > 0 {
                children.push(
                    Element::new(&font, ElementContent::Text(" | ".to_string())).colors(
                        ElementColors {
                            border: BorderColor::default(),
                            bg: LinearRgba::TRANSPARENT.into(),
                            text: SHORTCUT_STRIP_DIM.into(),
                        },
                    ),
                );
            }

            children.push(
                Element::new(
                    &font,
                    ElementContent::Text(format!("{} {}", entry.label, entry.shortcut)),
                )
                .colors(ElementColors {
                    border: BorderColor::default(),
                    bg: LinearRgba::TRANSPARENT.into(),
                    text: SHORTCUT_STRIP_TEXT.into(),
                }),
            );
        }

        let element = Element::new(&font, ElementContent::Children(children))
            .display(DisplayType::Block)
            .min_width(Some(Dimension::Percent(1.0)))
            .colors(ElementColors {
                border: BorderColor::new(SHORTCUT_STRIP_BORDER.into()),
                bg: SHORTCUT_STRIP_BG.into(),
                text: SHORTCUT_STRIP_TEXT.into(),
            })
            .padding(BoxDimension {
                left: Dimension::Cells(0.45),
                right: Dimension::Cells(0.45),
                top: Dimension::Cells(0.12),
                bottom: Dimension::Cells(0.14),
            })
            .border(BoxDimension {
                left: Dimension::Pixels(0.0),
                right: Dimension::Pixels(0.0),
                top: Dimension::Pixels(1.0),
                bottom: Dimension::Pixels(0.0),
            });
        let strip_height = metrics.cell_size.height as f32 + 10.0;
        let bottom_y = dimensions.pixel_height as f32 - strip_height - bottom_inset;
        let computed = self.compute_element(
            &LayoutContext {
                height: DimensionContext {
                    dpi: dimensions.dpi as f32,
                    pixel_max: dimensions.pixel_height as f32,
                    pixel_cell: metrics.cell_size.height as f32,
                },
                width: DimensionContext {
                    dpi: dimensions.dpi as f32,
                    pixel_max: dimensions.pixel_width as f32,
                    pixel_cell: metrics.cell_size.width as f32,
                },
                bounds: euclid::rect(0.0, bottom_y, dimensions.pixel_width as f32, strip_height),
                metrics,
                gl_state: self.render_state.as_ref().unwrap(),
                zindex: 105,
            },
            &element,
        )?;
        let gl_state = self.render_state.as_ref().unwrap();
        self.render_element(&computed, gl_state, None)?;
        Ok(())
    }

    fn operator_nav_workspace(&self) -> Option<String> {
        self.current_workspace_name()
    }

    fn operator_nav_entries(&mut self) -> Vec<mux::task_center::TaskCenterEntry> {
        let workspace = self.operator_nav_workspace();
        let entries = self.task_center_entries();
        match workspace {
            Some(workspace) => entries
                .into_iter()
                .filter(|entry| entry.workspace == workspace)
                .collect(),
            None => entries,
        }
    }

    fn operator_nav_counts(&mut self) -> Vec<(OperatorNavItem, usize)> {
        let entries = self.operator_nav_entries();
        OperatorNavItem::ordered()
            .iter()
            .copied()
            .map(|item| {
                let count = match item {
                    OperatorNavItem::Home => entries.len(),
                    OperatorNavItem::Inbox => entries
                        .iter()
                        .filter(|entry| entry.unread_count > 0)
                        .count(),
                    OperatorNavItem::Running => {
                        entries.iter().filter(|entry| entry.is_running).count()
                    }
                    OperatorNavItem::Failed => {
                        entries.iter().filter(|entry| entry.is_failed).count()
                    }
                    OperatorNavItem::Metadata => entries
                        .iter()
                        .filter(|entry| {
                            entry.workspace_status.as_ref().is_some()
                                || entry.workspace_progress.is_some()
                        })
                        .count(),
                    OperatorNavItem::Tasks => entries
                        .iter()
                        .filter(|entry| {
                            matches!(entry.kind, mux::task_center::TaskCenterKind::Pane)
                        })
                        .count(),
                };
                (item, count)
            })
            .collect()
    }

    fn operator_nav_workspace_status_text(&self, workspace: &str) -> Option<String> {
        let mux = mux::Mux::get();
        let status = mux
            .workspace_status_for_workspace(workspace)
            .map(|record| record.status)
            .or_else(|| self.workspace_status_cache.get(workspace).cloned());
        let progress = mux
            .workspace_progress_for_workspace(workspace)
            .map(|record| record.value)
            .or_else(|| self.workspace_progress_cache.get(workspace).copied());

        match (status, progress) {
            (Some(status), Some(progress)) => {
                Some(format!("{} | {}%", status.to_ascii_uppercase(), progress))
            }
            (Some(status), None) => Some(status.to_ascii_uppercase()),
            (None, Some(progress)) => Some(format!("{}%", progress)),
            (None, None) => None,
        }
    }

    fn paint_operator_nav(&mut self) -> anyhow::Result<()> {
        if !self.operator_nav_enabled() {
            return Ok(());
        }

        let nav_width = self.operator_nav_width_px() as f32;
        let font = self.fonts.title_font()?;
        let metrics = RenderMetrics::with_font_metrics(&font.metrics());
        let border = self.get_os_border();
        let tab_bar_height = if self.show_tab_bar && !self.config.tab_bar_at_bottom {
            self.tab_bar_pixel_height().unwrap_or(0.0)
        } else {
            0.0
        };
        let top_inset = border.top.get() as f32 + tab_bar_height + 4.0;
        let bottom_inset = border.bottom.get() as f32
            + if self.show_tab_bar && self.config.tab_bar_at_bottom {
                self.tab_bar_pixel_height().unwrap_or(0.0)
            } else {
                0.0
            }
            + 8.0;

        let counts = self.operator_nav_counts();
        let workspace = self
            .operator_nav_workspace()
            .unwrap_or_else(|| "workspace".to_string());
        let workspace_title = workspace.to_ascii_uppercase();
        let workspace_meta = self.operator_nav_workspace_status_text(&workspace);
        let inbox_count = counts
            .iter()
            .find_map(|(item, count)| (*item == OperatorNavItem::Inbox).then_some(*count))
            .unwrap_or_else(|| {
                self.operator_nav_entries()
                    .iter()
                    .filter(|entry| entry.unread_count > 0)
                    .count()
            });

        let header = |label: &str| {
            Element::new(&font, ElementContent::Text(label.to_string()))
                .display(DisplayType::Block)
                .min_width(Some(Dimension::Percent(1.0)))
                .colors(ElementColors {
                    border: BorderColor::default(),
                    bg: LinearRgba::TRANSPARENT.into(),
                    text: OPERATOR_NAV_HEADER_TEXT.into(),
                })
                .padding(BoxDimension {
                    left: Dimension::Cells(0.22),
                    right: Dimension::Cells(0.22),
                    top: Dimension::Cells(0.12),
                    bottom: Dimension::Cells(0.08),
                })
                .margin(BoxDimension {
                    left: Dimension::Cells(0.0),
                    right: Dimension::Cells(0.0),
                    top: Dimension::Cells(0.0),
                    bottom: Dimension::Cells(0.02),
                })
        };

        let make_nav_row =
            |item: OperatorNavItem, title: String, count: Option<usize>, is_active: bool| {
                let fg = if is_active {
                    OPERATOR_NAV_ACTIVE_TEXT
                } else {
                    OPERATOR_NAV_TEXT
                };
                let bg = if is_active {
                    OPERATOR_NAV_ACTIVE_BG
                } else {
                    LinearRgba::TRANSPARENT
                };

                let mut row = vec![Element::new(
                    &font,
                    ElementContent::Text(format!("{} {}", item.icon(), title)),
                )
                .colors(ElementColors {
                    border: BorderColor::default(),
                    bg: LinearRgba::TRANSPARENT.into(),
                    text: fg.into(),
                })];

                if let Some(count) = count.filter(|count| *count > 0) {
                    row.push(
                        Element::new(&font, ElementContent::Text(count.to_string()))
                            .float(Float::Right)
                            .colors(ElementColors {
                                border: BorderColor::default(),
                                bg: LinearRgba::TRANSPARENT.into(),
                                text: if is_active {
                                    OPERATOR_NAV_ACCENT.into()
                                } else {
                                    OPERATOR_NAV_DIM.into()
                                },
                            }),
                    );
                }

                Element::new(&font, ElementContent::Children(row))
                    .item_type(UIItemType::OperatorNav(item))
                    .display(DisplayType::Block)
                    .min_width(Some(Dimension::Percent(1.0)))
                    .colors(ElementColors {
                        border: if is_active {
                            BorderColor::new(OPERATOR_NAV_ACCENT.into())
                        } else {
                            BorderColor::default()
                        },
                        bg: bg.into(),
                        text: fg.into(),
                    })
                    .hover_colors(Some(ElementColors {
                        border: BorderColor::new(OPERATOR_NAV_ACCENT.into()),
                        bg: LinearRgba::with_components(0.11, 0.12, 0.16, 1.0).into(),
                        text: OPERATOR_NAV_ACTIVE_TEXT.into(),
                    }))
                    .padding(BoxDimension {
                        left: Dimension::Cells(0.22),
                        right: Dimension::Cells(0.22),
                        top: Dimension::Cells(0.14),
                        bottom: Dimension::Cells(0.14),
                    })
                    .margin(BoxDimension {
                        left: Dimension::Cells(0.0),
                        right: Dimension::Cells(0.0),
                        top: Dimension::Cells(0.03),
                        bottom: Dimension::Cells(0.03),
                    })
                    .border(BoxDimension {
                        left: if is_active {
                            Dimension::Pixels(1.5)
                        } else {
                            Dimension::Pixels(0.0)
                        },
                        right: Dimension::Pixels(0.0),
                        top: Dimension::Pixels(0.0),
                        bottom: Dimension::Pixels(0.0),
                    })
            };

        let mut rows = Vec::new();
        rows.push(header("WORKSPACES"));
        rows.push(make_nav_row(
            OperatorNavItem::Home,
            workspace_title,
            None,
            self.operator_nav_selection == OperatorNavItem::Home,
        ));

        if let Some(workspace_meta) = workspace_meta {
            rows.push(
                Element::new(&font, ElementContent::Text(workspace_meta))
                    .display(DisplayType::Block)
                    .min_width(Some(Dimension::Percent(1.0)))
                    .colors(ElementColors {
                        border: BorderColor::default(),
                        bg: LinearRgba::TRANSPARENT.into(),
                        text: OPERATOR_NAV_DIM.into(),
                    })
                    .padding(BoxDimension {
                        left: Dimension::Cells(0.22),
                        right: Dimension::Cells(0.22),
                        top: Dimension::Cells(0.02),
                        bottom: Dimension::Cells(0.16),
                    }),
            );
        } else {
            rows.push(
                Element::new(&font, ElementContent::Text(String::new()))
                    .display(DisplayType::Block)
                    .min_width(Some(Dimension::Percent(1.0)))
                    .min_height(Some(Dimension::Cells(0.45))),
            );
        }

        for (item, count) in counts {
            rows.push(make_nav_row(
                item,
                item.label().to_ascii_uppercase(),
                Some(count),
                self.operator_nav_selection == item,
            ));
        }

        rows.push(
            Element::new(&font, ElementContent::Text(String::new()))
                .display(DisplayType::Block)
                .min_width(Some(Dimension::Percent(1.0)))
                .min_height(Some(Dimension::Cells(0.6))),
        );
        rows.push(header("INBOX"));
        rows.push(make_nav_row(
            OperatorNavItem::Inbox,
            if inbox_count > 0 {
                format!("{} UNREAD NOTIFICATIONS", inbox_count)
            } else {
                "NO UNREAD NOTIFICATIONS".to_string()
            },
            None,
            self.operator_nav_selection == OperatorNavItem::Inbox,
        ));

        let strip_bounds = euclid::rect(0.0, 0.0, nav_width, self.dimensions.pixel_height as f32);

        let element = Element::new(&font, ElementContent::Children(rows))
            .colors(ElementColors {
                border: BorderColor::new(OPERATOR_NAV_BORDER.into()),
                bg: OPERATOR_NAV_PANEL_BG.into(),
                text: OPERATOR_NAV_TEXT.into(),
            })
            .padding(BoxDimension {
                left: Dimension::Pixels(6.0),
                right: Dimension::Pixels(4.0),
                top: Dimension::Pixels(top_inset),
                bottom: Dimension::Pixels(bottom_inset),
            })
            .border(BoxDimension {
                left: Dimension::Pixels(0.0),
                right: Dimension::Pixels(1.0),
                top: Dimension::Pixels(0.0),
                bottom: Dimension::Pixels(0.0),
            })
            .min_width(Some(Dimension::Pixels(nav_width)))
            .min_height(Some(Dimension::Pixels(self.dimensions.pixel_height as f32)));

        let computed = self.compute_element(
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
                bounds: strip_bounds,
                metrics: &metrics,
                gl_state: self.render_state.as_ref().unwrap(),
                zindex: 30,
            },
            &element,
        )?;

        let mut ui_items = computed.ui_items();
        let gl_state = self.render_state.as_ref().unwrap();
        self.render_element(&computed, gl_state, None)?;
        self.ui_items.append(&mut ui_items);
        Ok(())
    }

    pub fn paint_impl(&mut self, frame: &mut RenderFrame) -> anyhow::Result<()> {
        self.num_frames += 1;
        // If nothing on screen needs animating, then we can avoid
        // invalidating as frequently
        *self.has_animation.borrow_mut() = None;
        // Start with the assumption that we should allow images to render
        self.allow_images = AllowImage::Yes;

        let start = Instant::now();

        {
            let diff = start.duration_since(self.last_fps_check_time);
            if diff > Duration::from_secs(1) {
                let seconds = diff.as_secs_f32();
                self.fps = self.num_frames as f32 / seconds;
                self.num_frames = 0;
                self.last_fps_check_time = start;
            }
        }

        'pass: for pass in 0.. {
            match self.paint_pass() {
                Ok(_) => match self.render_state.as_mut().unwrap().allocated_more_quads() {
                    Ok(allocated) => {
                        if !allocated {
                            break 'pass;
                        }
                        self.invalidate_fancy_tab_bar();
                        self.invalidate_modal();
                    }
                    Err(err) => {
                        log::error!("{:#}", err);
                        break 'pass;
                    }
                },
                Err(err) => {
                    if let Some(&OutOfTextureSpace {
                        size: Some(size),
                        current_size,
                    }) = err.root_cause().downcast_ref::<OutOfTextureSpace>()
                    {
                        let result = if pass == 0 {
                            // Let's try clearing out the atlas and trying again
                            // self.clear_texture_atlas()
                            log::trace!("recreate_texture_atlas");
                            self.recreate_texture_atlas(Some(current_size))
                        } else {
                            log::trace!("grow texture atlas to {}", size);
                            self.recreate_texture_atlas(Some(size))
                        };
                        self.invalidate_fancy_tab_bar();
                        self.invalidate_modal();

                        if let Err(err) = result {
                            self.allow_images = match self.allow_images {
                                AllowImage::Yes => AllowImage::Scale(2),
                                AllowImage::Scale(2) => AllowImage::Scale(4),
                                AllowImage::Scale(4) => AllowImage::Scale(8),
                                AllowImage::Scale(8) => AllowImage::No,
                                AllowImage::No | _ => {
                                    log::error!(
                                        "Failed to {} texture: {}",
                                        if pass == 0 { "clear" } else { "resize" },
                                        err
                                    );
                                    break 'pass;
                                }
                            };

                            log::info!(
                                "Not enough texture space ({:#}); \
                                     will retry render with {:?}",
                                err,
                                self.allow_images,
                            );
                        }
                    } else if err.root_cause().downcast_ref::<ClearShapeCache>().is_some() {
                        self.invalidate_fancy_tab_bar();
                        self.invalidate_modal();
                        self.shape_generation += 1;
                        self.shape_cache.borrow_mut().clear();
                        self.line_to_ele_shape_cache.borrow_mut().clear();
                    } else {
                        log::error!("paint_pass failed: {:#}", err);
                        break 'pass;
                    }
                }
            }
        }

        log::debug!("paint_impl before call_draw elapsed={:?}", start.elapsed());

        self.call_draw(frame)?;
        self.last_frame_duration = start.elapsed();
        log::debug!(
            "paint_impl elapsed={:?}, fps={}",
            self.last_frame_duration,
            self.fps
        );
        metrics::histogram!("gui.paint.impl").record(self.last_frame_duration);
        metrics::histogram!("gui.paint.impl.rate").record(1.);

        // If self.has_animation is some, then the last render detected
        // image attachments with multiple frames, so we also need to
        // invalidate the viewport when the next frame is due
        if self.focused.is_some() {
            if let Some(next_due) = *self.has_animation.borrow() {
                let prior = self.scheduled_animation.borrow_mut().take();
                match prior {
                    Some(prior) if prior <= next_due => {
                        // Already due before that time
                    }
                    _ => {
                        self.scheduled_animation.borrow_mut().replace(next_due);
                        let window = self.window.clone().take().unwrap();
                        promise::spawn::spawn(async move {
                            Timer::at(next_due).await;
                            let win = window.clone();
                            window.notify(TermWindowNotif::Apply(Box::new(move |tw| {
                                tw.scheduled_animation.borrow_mut().take();
                                // Modal content is cached, so blinking carets and other
                                // time-based modal elements must be explicitly reconfigured.
                                tw.invalidate_modal();
                                win.invalidate();
                            })));
                        })
                        .detach();
                    }
                }
            }
        }

        Ok(())
    }

    pub fn paint_modal(&mut self) -> anyhow::Result<()> {
        if let Some(modal) = self.get_modal() {
            for computed in modal.computed_element(self)?.iter() {
                let mut ui_items = computed.ui_items();

                let gl_state = self.render_state.as_ref().unwrap();
                self.render_element(&computed, gl_state, None)?;

                self.ui_items.append(&mut ui_items);
            }
        }

        Ok(())
    }

    pub fn paint_pass(&mut self) -> anyhow::Result<()> {
        {
            let gl_state = self.render_state.as_ref().unwrap();
            for layer in gl_state.layers.borrow().iter() {
                layer.clear_quad_allocation();
            }
        }

        // Clear out UI item positions; we'll rebuild these as we render
        self.ui_items.clear();

        let panes = self.get_panes_to_render();
        let focused = self.focused.is_some();
        let window_is_transparent =
            !self.window_background.is_empty() || self.config.window_background_opacity != 1.0;
        let force_opaque_window_background = forces_opaque_kaku_tui_window_background(&panes);
        let effective_window_is_transparent =
            window_is_transparent && !force_opaque_window_background;

        let start = Instant::now();
        let gl_state = self.render_state.as_ref().unwrap();
        let layer = gl_state
            .layer_for_zindex(0)
            .context("layer_for_zindex(0)")?;
        let mut layers = layer.quad_allocator();
        log::trace!("quad map elapsed {:?}", start.elapsed());
        metrics::histogram!("quad.map").record(start.elapsed());

        let mut paint_terminal_background = false;

        // Render the full window background
        if force_opaque_window_background {
            paint_terminal_background = true;
        } else {
            match (self.window_background.is_empty(), self.allow_images) {
                (false, AllowImage::Yes | AllowImage::Scale(_)) => {
                    let bg_color = self.palette().background.to_linear();

                    let top = panes
                        .iter()
                        .find(|p| p.is_active)
                        .map(|p| match self.effective_viewport(&p.pane) {
                            Some(top) => top,
                            None => p.pane.get_dimensions().physical_top,
                        })
                        .unwrap_or(0);

                    let loaded_any = self
                        .render_backgrounds(bg_color, top)
                        .context("render_backgrounds")?;

                    if !loaded_any {
                        // Either there was a problem loading the background(s)
                        // or they haven't finished loading yet.
                        // Use the regular terminal background until that changes.
                        paint_terminal_background = true;
                    }
                }
                _ if effective_window_is_transparent => {
                    // Avoid doubling up the background color for the main pane area.
                    // We still need to paint strips that are intentionally excluded
                    // from pane background quads.
                    let strip_background = if panes.len() == 1 {
                        panes[0].pane.palette().background
                    } else {
                        self.palette().background
                    }
                    .to_linear()
                    .mul_alpha(self.config.window_background_opacity);
                    let border = self.get_os_border();
                    let tab_bar_height = if self.show_tab_bar {
                        self.tab_bar_pixel_height()
                            .context("tab_bar_pixel_height")?
                    } else {
                        0.0
                    };
                    let (_, padding_bottom) = self.effective_vertical_padding();
                    let padding_bottom = padding_bottom as f32;
                    // Only cover the OS border area; the tab bar paints its own
                    // transparent background in paint_tab_bar(), so including
                    // tab_bar_height here would double-paint that region.
                    // When tab bar is at top, it starts at y=0 and covers the
                    // titlebar area completely, so no top fill needed.
                    let top_fill_height = if self.show_tab_bar && !self.config.tab_bar_at_bottom {
                        0.0
                    } else {
                        border.top.get() as f32
                    };
                    // Same reasoning as top: only cover the OS border + padding.
                    // The tab bar paints its own transparent background.
                    // When tab bar is at bottom, it covers the bottom border area,
                    // so only fill the padding area.
                    let bottom_fill_height = if self.show_tab_bar && self.config.tab_bar_at_bottom {
                        padding_bottom
                    } else {
                        padding_bottom + border.bottom.get() as f32
                    };
                    let right_fill_width = self.effective_right_padding(&self.config) as f32
                        + border.right.get() as f32;
                    // Use content_left_inset() to include content-alignment gap;
                    // the leftmost pane background will start at this boundary so
                    // the two regions don't overlap.
                    let left_fill_width = self.content_left_inset();
                    let window_width = self.dimensions.pixel_width as f32;
                    let window_height = self.dimensions.pixel_height as f32;

                    if top_fill_height > 0.0 {
                        self.filled_rectangle(
                            &mut layers,
                            0,
                            euclid::rect(
                                0.0,
                                0.0,
                                window_width,
                                top_fill_height.min(window_height),
                            ),
                            strip_background,
                        )
                        .context("filled_rectangle for transparent top strip")?;
                    }

                    if bottom_fill_height > 0.0 {
                        let clamped_height = bottom_fill_height.min(window_height);
                        self.filled_rectangle(
                            &mut layers,
                            0,
                            euclid::rect(
                                0.0,
                                (window_height - clamped_height).max(0.0),
                                window_width,
                                clamped_height,
                            ),
                            strip_background,
                        )
                        .context("filled_rectangle for transparent bottom strip")?;
                    }

                    // The tab bar paints its own full-width background, so the
                    // left/right side strips must skip the tab bar region to avoid
                    // double-painting.
                    let tab_bar_top_height = if self.show_tab_bar && !self.config.tab_bar_at_bottom
                    {
                        tab_bar_height
                    } else {
                        0.0
                    };
                    let tab_bar_bottom_height =
                        if self.show_tab_bar && self.config.tab_bar_at_bottom {
                            tab_bar_height
                        } else {
                            0.0
                        };
                    let side_fill_y = (top_fill_height + tab_bar_top_height).min(window_height);
                    // When tab bar is at bottom, side fills should extend to tab bar top,
                    // not to (bottom_fill_height + tab_bar_height) which would leave a gap.
                    let side_fill_height = if self.show_tab_bar && self.config.tab_bar_at_bottom {
                        (window_height - side_fill_y - tab_bar_height).max(0.0)
                    } else {
                        (window_height
                            - side_fill_y
                            - (bottom_fill_height + tab_bar_bottom_height).min(window_height))
                        .max(0.0)
                    };

                    if right_fill_width > 0.0 {
                        let clamped_width = right_fill_width.min(window_width);
                        self.filled_rectangle(
                            &mut layers,
                            0,
                            euclid::rect(
                                window_width - clamped_width,
                                side_fill_y,
                                clamped_width,
                                side_fill_height,
                            ),
                            strip_background,
                        )
                        .context("filled_rectangle for transparent right strip")?;
                    }

                    if left_fill_width > 0.0 {
                        let clamped_width = left_fill_width.min(window_width);
                        self.filled_rectangle(
                            &mut layers,
                            0,
                            euclid::rect(0.0, side_fill_y, clamped_width, side_fill_height),
                            strip_background,
                        )
                        .context("filled_rectangle for transparent left strip")?;
                    }
                }
                _ => {
                    paint_terminal_background = true;
                }
            }
        }

        if paint_terminal_background {
            // Regular window background color
            let background = if panes.len() == 1 {
                // If we're the only pane, use the pane's palette
                // to draw the padding background
                panes[0].pane.palette().background
            } else {
                self.palette().background
            }
            .to_linear()
            .mul_alpha(if force_opaque_window_background {
                1.0
            } else {
                self.config.window_background_opacity
            });

            self.filled_rectangle(
                &mut layers,
                0,
                euclid::rect(
                    0.,
                    0.,
                    self.dimensions.pixel_width as f32,
                    self.dimensions.pixel_height as f32,
                ),
                background,
            )
            .context("filled_rectangle for window background")?;
        }

        let hide_transition_content = self
            .window
            .as_ref()
            .map(|window| window.is_zoom_animation_active())
            .unwrap_or(false);
        if hide_transition_content {
            // During fullscreen transition, keep only a stable background to avoid
            // one-frame text scale pops.
            let hide_background = self.palette().background.to_linear();
            self.filled_rectangle(
                &mut layers,
                0,
                euclid::rect(
                    0.,
                    0.,
                    self.dimensions.pixel_width as f32,
                    self.dimensions.pixel_height as f32,
                ),
                hide_background,
            )
            .context("filled_rectangle for fullscreen transition hide")?;
            drop(layers);
            return Ok(());
        }

        let num_panes = panes.len();
        let broadcast_visual_mode = self.broadcast_input_visual_mode();
        let mut input_target_top_right: Vec<(f32, f32, bool)> = vec![];

        for pos in panes {
            let show_input_target_indicator =
                broadcast_visual_mode || (num_panes > 1 && pos.is_active);
            if show_input_target_indicator {
                let cell_width = self.render_metrics.cell_size.width as f32;
                let cell_height = self.render_metrics.cell_size.height as f32;
                let (_, padding_top) = self.padding_left_top();
                let border = self.get_os_border();
                let tab_bar_height = if self.show_tab_bar {
                    self.tab_bar_pixel_height().unwrap_or(0.)
                } else {
                    0.
                };
                let top_bar_height = if self.config.tab_bar_at_bottom {
                    0.0
                } else {
                    tab_bar_height
                };
                let top_pixel_y = top_bar_height + padding_top + border.top.get() as f32;

                let x = self.content_left_inset() + ((pos.left + pos.width) as f32 * cell_width);
                let y = top_pixel_y + (pos.top as f32 * cell_height);
                let is_top_pane = pos.top == 0;
                input_target_top_right.push((x, y, is_top_pane));
            }
            if pos.is_active {
                if self.get_modal().is_none() {
                    self.update_text_cursor(&pos);
                }
                if focused {
                    pos.pane.advise_focus();
                    mux::Mux::get().record_focus_for_current_identity(pos.pane.pane_id());
                }
                // Bell state cleared in clear_active_tab_bell_state() to ensure badge sync
            }
            self.paint_pane(&pos, &mut layers).context("paint_pane")?;
        }

        const RIGHT_INSET: f32 = 3.0;
        const TOP_PANE_MARGIN_WITH_TAB_BAR: f32 = 24.0;
        const TOP_PANE_MARGIN_NO_TAB_BAR: f32 = 14.0;
        const LOWER_PANE_MARGIN: f32 = 20.0;

        // Draw dot indicator for panes that currently receive input.
        for (dot_x, dot_y, is_top_pane) in input_target_top_right {
            let top_pane_margin = if self.show_tab_bar && !self.config.tab_bar_at_bottom {
                TOP_PANE_MARGIN_WITH_TAB_BAR
            } else {
                TOP_PANE_MARGIN_NO_TAB_BAR
            };
            let margin_top = if is_top_pane {
                top_pane_margin
            } else {
                LOWER_PANE_MARGIN
            };

            const DOT_ALPHA: f32 = 0.5;
            const BROADCAST_ICON_ALPHA: f32 = 0.9;
            let (poly, size, alpha) = if broadcast_visual_mode {
                (
                    BROADCAST_INDICATOR_POLY,
                    BROADCAST_ICON_SIZE,
                    BROADCAST_ICON_ALPHA,
                )
            } else {
                (ACTIVE_PANE_INDICATOR_POLY, STATUS_DOT_SIZE, DOT_ALPHA)
            };
            let dot_color = self.palette().cursor_bg.to_linear().mul_alpha(alpha);

            self.poly_quad(
                &mut layers,
                2,
                euclid::point2(dot_x - size - RIGHT_INSET, dot_y + margin_top),
                poly,
                1,
                euclid::size2(size, size),
                dot_color,
            )
            .context("input target indicator")?;
        }

        if let Some(pane) = self.get_active_pane_or_overlay() {
            let splits = self.get_splits();
            for split in &splits {
                self.paint_split(&mut layers, split, &splits, &pane)
                    .context("paint_split")?;
            }
        }

        // Clear bell state for active tab and update Dock badge (always, regardless of tab bar visibility)
        self.clear_active_tab_bell_state();

        // Draw visual notification dot on inactive tabs with unread bell
        if self.show_tab_bar {
            self.paint_tab_bar(&mut layers).context("paint_tab_bar")?;
            self.paint_tab_bell_indicators(&mut layers)?;
        }

        self.paint_window_borders(&mut layers)
            .context("paint_window_borders")?;
        drop(layers);
        self.paint_operator_nav().context("paint_operator_nav")?;
        self.paint_shortcut_strip()
            .context("paint_shortcut_strip")?;
        self.paint_modal().context("paint_modal")?;
        self.paint_toast().context("paint_toast")?;

        Ok(())
    }

    /// Clear bell state for all panes in the active tab and update global Dock badge.
    /// Called every paint cycle to ensure badge stays in sync regardless of tab bar visibility.
    /// Only clears when window has focus, so Dock badge persists while window is unfocused.
    fn clear_active_tab_bell_state(&mut self) {
        if self.focused.is_none() {
            return;
        }
        // Fast path: skip mux lock if no panes have unread bells
        if !self.pane_state.borrow().values().any(|s| s.has_unread_bell) {
            return;
        }
        let mux = mux::Mux::get();
        let mux_window = match mux.get_window(self.mux_window_id) {
            Some(w) => w,
            None => return,
        };
        let active_tab_idx = mux_window.get_active_idx();

        if let Some(active_tab) = mux_window.get_by_idx(active_tab_idx) {
            let active_tab_panes = active_tab.iter_panes_ignoring_zoom();
            let mut cleared_count: isize = 0;
            for pos in &active_tab_panes {
                let mut state = self.pane_state(pos.pane.pane_id());
                if state.has_unread_bell {
                    state.has_unread_bell = false;
                    cleared_count += 1;
                }
            }
            if cleared_count > 0 {
                crate::frontend::front_end().adjust_unread_bell_count(-cleared_count);
            }
        }
    }

    /// Draw a dot on inactive tabs that have panes with unread bell events.
    fn paint_tab_bell_indicators(
        &mut self,
        _layers: &mut crate::quad::TripleLayerQuadAllocator,
    ) -> anyhow::Result<()> {
        // Kaku renders unread bell state inline in the bundled format-tab-title
        // callback, so the core tab-dot painter would be redundant.
        Ok(())
    }

    /// Render the toast notification
    pub fn paint_toast(&mut self) -> anyhow::Result<()> {
        let (toast_at, message, lifetime) = match &self.toast {
            Some((t, msg, lifetime)) if t.elapsed() < *lifetime => (*t, msg.clone(), *lifetime),
            _ => return Ok(()),
        };

        let font = self.fonts.title_font()?;
        let metrics = RenderMetrics::with_font_metrics(&font.metrics());

        // Fade out during the last 500ms of the configured lifetime.
        let elapsed_ms = toast_at.elapsed().as_millis() as f32;
        let lifetime_ms = lifetime.as_millis() as f32;
        let fade_start_ms = (lifetime_ms - 500.0).max(0.0);
        let alpha = if elapsed_ms > fade_start_ms {
            (1.0 - (elapsed_ms - fade_start_ms) / 500.0).max(0.0)
        } else {
            1.0
        };

        // Match the toast to the currently visible terminal palette so it
        // stays in sync with theme changes and client palette overrides.
        let palette = if let Some(pane) = self.get_active_pane_or_overlay() {
            pane.palette()
        } else {
            self.palette().clone()
        };
        let (bg_color, text_color) = toast_colors_for_palette(&palette, alpha);
        let toast_radius = Dimension::Pixels(8.0);

        let text = Element::new(&font, ElementContent::Text(message.clone()))
            .colors(ElementColors {
                border: BorderColor::default(),
                bg: LinearRgba::TRANSPARENT.into(),
                text: text_color.into(),
            })
            .display(DisplayType::Block);

        let element = Element::new(&font, ElementContent::Children(vec![text]))
            .colors(ElementColors {
                // Rounded corner polys use border colors even if the border
                // width is zero; match bg to avoid corner gaps.
                border: BorderColor::new(bg_color.into()),
                bg: bg_color.into(),
                text: text_color.into(),
            })
            .padding(BoxDimension {
                left: Dimension::Cells(0.75),
                right: Dimension::Cells(0.75),
                top: Dimension::Cells(0.25),
                bottom: Dimension::Cells(0.25),
            })
            .border(BoxDimension::new(Dimension::Pixels(0.0)))
            .border_corners(Some(Corners {
                top_left: SizedPoly {
                    width: toast_radius,
                    height: toast_radius,
                    poly: TOP_LEFT_ROUNDED_CORNER,
                },
                top_right: SizedPoly {
                    width: toast_radius,
                    height: toast_radius,
                    poly: TOP_RIGHT_ROUNDED_CORNER,
                },
                bottom_left: SizedPoly {
                    width: toast_radius,
                    height: toast_radius,
                    poly: BOTTOM_LEFT_ROUNDED_CORNER,
                },
                bottom_right: SizedPoly {
                    width: toast_radius,
                    height: toast_radius,
                    poly: BOTTOM_RIGHT_ROUNDED_CORNER,
                },
            }));

        let dimensions = self.dimensions;
        let border = self.get_os_border();
        // Calculate width based on message length (each char ~cell_width + padding)
        let approx_width = (message.len() as f32 + 1.5) * metrics.cell_size.width as f32;
        let toast_height = metrics.cell_size.height as f32 * 1.5;
        // Use consistent margin based on cell size
        let h_margin = metrics.cell_size.width as f32 * 2.0;
        let v_margin = metrics.cell_size.height as f32 * 2.0;

        // Position at bottom-right with fixed margin from window edge
        let right_x =
            dimensions.pixel_width as f32 - approx_width - h_margin - border.right.get() as f32;
        let bottom_y =
            dimensions.pixel_height as f32 - toast_height - v_margin - border.bottom.get() as f32;

        let computed = self.compute_element(
            &LayoutContext {
                height: DimensionContext {
                    dpi: dimensions.dpi as f32,
                    pixel_max: dimensions.pixel_height as f32,
                    pixel_cell: metrics.cell_size.height as f32,
                },
                width: DimensionContext {
                    dpi: dimensions.dpi as f32,
                    pixel_max: dimensions.pixel_width as f32,
                    pixel_cell: metrics.cell_size.width as f32,
                },
                bounds: euclid::rect(right_x, bottom_y, approx_width, toast_height),
                metrics: &metrics,
                gl_state: self.render_state.as_ref().unwrap(),
                zindex: 120,
            },
            &element,
        )?;

        let gl_state = self.render_state.as_ref().unwrap();
        self.render_element(&computed, gl_state, None)?;

        // Keep redrawing during fade-out
        if elapsed_ms > fade_start_ms {
            let next = Instant::now() + Duration::from_millis(16);
            let mut anim = self.has_animation.borrow_mut();
            match *anim {
                Some(existing) if existing <= next => {}
                _ => {
                    *anim = Some(next);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::toast_colors_for_palette;
    use wezterm_term::color::{ColorPalette, SrgbaTuple};
    use window::color::LinearRgba;

    fn paint_source() -> &'static str {
        include_str!("paint.rs")
    }

    fn operator_nav_source() -> String {
        let source = paint_source();
        let start = source
            .find("fn paint_operator_nav(&mut self) -> anyhow::Result<()>")
            .expect("operator nav renderer should exist");
        let tail = &source[start..];
        let end = tail
            .find("\n    pub fn paint_impl(")
            .expect("operator nav renderer should end before paint_impl");
        tail[..end].to_string()
    }

    #[test]
    fn light_palette_uses_gold_background_and_dark_text() {
        let mut palette = ColorPalette::default();
        palette.background = SrgbaTuple(0.95, 0.95, 0.95, 1.0);
        palette.colors.0[3] = SrgbaTuple(0.8, 0.7, 0.2, 1.0);

        let (bg, text) = toast_colors_for_palette(&palette, 1.0);
        let expected_bg = palette.colors.0[3].to_linear();

        assert_eq!(
            bg,
            LinearRgba(expected_bg.0, expected_bg.1, expected_bg.2, 0.9)
        );
        assert_eq!(text, LinearRgba(0.1, 0.1, 0.1, 1.0));
    }

    #[test]
    fn dark_palette_uses_accent_background_and_light_text() {
        let mut palette = ColorPalette::default();
        palette.background = SrgbaTuple(0.08, 0.08, 0.08, 1.0);
        palette.colors.0[13] = SrgbaTuple(0.5, 0.3, 0.8, 1.0);

        let (bg, text) = toast_colors_for_palette(&palette, 1.0);
        let expected_bg = palette.colors.0[13].to_linear();

        assert_eq!(
            bg,
            LinearRgba(expected_bg.0, expected_bg.1, expected_bg.2, 0.9)
        );
        assert_eq!(text, LinearRgba(1.0, 1.0, 1.0, 1.0));
    }

    #[test]
    fn operator_nav_render_keeps_explicit_hit_targets() {
        let source = operator_nav_source();

        assert!(source.contains(".item_type(UIItemType::OperatorNav(item))"));
        assert!(source.contains("fn paint_operator_nav(&mut self) -> anyhow::Result<()>"));
    }

    #[test]
    fn operator_nav_render_stays_compact_and_avoids_phase_six_panel_headers() {
        let source = operator_nav_source();

        assert!(
            !source.contains("ElementContent::Text(\"KAKU\".to_string())"),
            "operator rail regressed into a branded panel header"
        );
        assert!(
            source.contains("WORKSPACES"),
            "operator rail should expose the persistent workspace section in the current service"
        );
        assert!(
            source.contains("NO UNREAD NOTIFICATIONS"),
            "operator rail should expose the persistent inbox footer copy"
        );
    }

    #[test]
    fn operator_nav_active_state_avoids_rounded_card_treatment() {
        let source = operator_nav_source();

        assert!(
            !source.contains(".border_corners(Some(Corners"),
            "operator rail regressed into rounded card rows instead of quiet edge chrome"
        );
    }

    #[test]
    fn operator_nav_render_keeps_top_inset_close_to_chrome() {
        let source = operator_nav_source();

        assert!(
            source.contains("tab_bar_height + 4.0"),
            "operator rail should begin close to the top chrome instead of floating below it"
        );
        assert!(
            !source.contains("tab_bar_height + 10.0"),
            "operator rail regressed into a large floating inset below the top chrome"
        );
    }

    #[test]
    fn paint_impl_renders_bottom_shortcut_strip() {
        let source = paint_source();

        assert!(source.contains("fn paint_shortcut_strip(&mut self) -> anyhow::Result<()>"));
        assert!(source.contains("self.paint_shortcut_strip()"));
        assert!(source.contains("CommandDef::shell_shortcut_strip_entries(&self.config)"));
    }
}
