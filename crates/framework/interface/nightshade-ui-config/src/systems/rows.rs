use crate::queries::{WidgetRow, row_value};
use crate::resources::ConfigPanel;
use nightshade::prelude::{Query, Res, ResMut};
use nightshade_config::contract::{Config, ConfigSchema};

pub fn read_the_rows<S: ConfigSchema>(
    mut config: ResMut<Config<S>>,
    panel: Res<ConfigPanel<S>>,
    mut widgets: Query<'_, WidgetRow<'_>>,
) {
    for (index, entry) in S::entries().iter().enumerate() {
        let Some(row) = panel.rows.get(index).copied() else {
            continue;
        };
        let mut value = None;
        widgets.for_entity(row, |widgets| {
            value = row_value(entry, widgets);
        });
        if let Some(value) = value {
            config.set_at(index, value);
        }
    }
}
