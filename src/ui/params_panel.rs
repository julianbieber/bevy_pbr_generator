//! The parameter panel, respawned from the active material's parsed fields
//! whenever those fields change.

use bevy::prelude::*;

use crate::material::catalog::MaterialCatalog;
use crate::material::params::{linear_to_srgb, srgb_to_linear, ParamsLayout, Widget};

/// The node the parameter widgets are spawned under.
#[derive(Component, Debug)]
pub struct ParamsPanel;

/// A slider driving one component of one parameter field.
#[derive(Component, Debug, Clone, Copy)]
pub struct ParamSlider {
    pub field: usize,
    pub component: usize,
    /// Whether the slider works in sRGB while the buffer stores linear.
    pub srgb: bool,
    /// Whether the stored value must land on a whole number.
    pub integer: bool,
}

impl ParamSlider {
    /// Converts what the slider shows into what the buffer stores.
    pub fn encode(&self, value: f32) -> f32 {
        if self.srgb {
            srgb_to_linear(value)
        } else if self.integer {
            value.round()
        } else {
            value
        }
    }

    /// Inverse of [`ParamSlider::encode`], for showing a stored value.
    pub fn decode(srgb: bool, value: f32) -> f32 {
        if srgb {
            linear_to_srgb(value)
        } else {
            value
        }
    }
}

/// A button that flips a `toggle` parameter.
#[derive(Component, Debug, Clone, Copy)]
pub struct ParamToggle {
    pub field: usize,
}

/// What the panel was last built for, so it is rebuilt only when it must be.
#[derive(Resource, Debug, Default)]
pub struct PanelState {
    pub material: Option<usize>,
    pub layout: ParamsLayout,
}

/// Respawns the panel when the active material or its parsed fields change.
///
/// The contents are data-driven, so there is nothing to update in place: the
/// widget set itself is what changes when a shader gains or loses a parameter.
pub fn rebuild(
    mut commands: Commands,
    catalog: Res<MaterialCatalog>,
    mut state: ResMut<PanelState>,
    panels: Query<Entity, With<ParamsPanel>>,
) {
    let Some(active) = catalog.active() else {
        return;
    };
    if state.material == Some(catalog.active) && state.layout == active.layout {
        return;
    }
    let Ok(panel) = panels.single() else {
        return;
    };

    state.material = Some(catalog.active);
    state.layout = active.layout.clone();

    commands.entity(panel).despawn_related::<Children>();

    let mut group: Option<String> = None;
    commands.entity(panel).with_children(|panel| {
        for (index, field) in active.layout.fields.iter().enumerate() {
            if field.widget == Widget::Hidden {
                continue;
            }

            if field.group != group {
                group = field.group.clone();
                if let Some(name) = &group {
                    panel.spawn(super::heading(name.to_uppercase()));
                }
            }

            let value = active.values.get(index).copied().unwrap_or(field.default);

            match &field.widget {
                Widget::Slider { min, max, .. } => {
                    let integer = field.ty.is_integer();
                    for component in 0..field.ty.components() {
                        let caption = if field.ty.components() == 1 {
                            field.label.clone()
                        } else {
                            format!("{} {}", field.label, ["x", "y", "z", "w"][component])
                        };
                        panel.spawn(super::labelled_slider(
                            &caption,
                            value[component],
                            *min,
                            *max,
                            ParamSlider {
                                field: index,
                                component,
                                srgb: false,
                                integer,
                            },
                        ));
                    }
                }
                Widget::Color => {
                    for (component, channel) in ["red", "green", "blue"].iter().enumerate() {
                        panel.spawn(super::labelled_slider(
                            &format!("{} {channel}", field.label),
                            ParamSlider::decode(true, value[component]),
                            0.0,
                            1.0,
                            ParamSlider {
                                field: index,
                                component,
                                srgb: true,
                                integer: false,
                            },
                        ));
                    }
                }
                Widget::Toggle => {
                    let state = if value[0] >= 0.5 { "on" } else { "off" };
                    panel.spawn(super::small_button(
                        &format!("{}: {state}", field.label),
                        ParamToggle { field: index },
                    ));
                }
                Widget::Hidden => {}
            }
        }
    });
}
