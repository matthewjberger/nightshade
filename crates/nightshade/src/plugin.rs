use crate::app::App;

pub use crate::app::Need;

pub trait Plugin {
    fn build(&self, app: &mut App);

    fn needs(&self) -> Vec<Need> {
        Vec::new()
    }
}

pub trait PluginGroup: Sized {
    fn build(self) -> PluginGroupBuilder;

    fn set<P: Plugin + 'static>(self, plugin: P) -> PluginGroupBuilder {
        self.build().set(plugin)
    }

    fn disable<P: Plugin + 'static>(self) -> PluginGroupBuilder {
        self.build().disable::<P>()
    }
}

#[derive(Default)]
pub struct PluginGroupBuilder {
    pub(crate) entries: Vec<PluginGroupEntry>,
}

pub(crate) struct PluginGroupEntry {
    pub(crate) id: std::any::TypeId,
    pub(crate) name: &'static str,
    pub(crate) enabled: bool,
    pub(crate) plugin: Box<dyn Plugin>,
}

impl PluginGroupBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_plugin<P: Plugin + 'static>(mut self, plugin: P) -> Self {
        self.entries.push(PluginGroupEntry {
            id: std::any::TypeId::of::<P>(),
            name: std::any::type_name::<P>(),
            enabled: true,
            plugin: Box::new(plugin),
        });
        self
    }

    pub fn set<P: Plugin + 'static>(mut self, plugin: P) -> Self {
        let entry = self.entry_mut::<P>().unwrap_or_else(|| {
            panic!("{} is not in this plugin group", std::any::type_name::<P>())
        });
        entry.plugin = Box::new(plugin);
        self
    }

    pub fn disable<P: Plugin + 'static>(mut self) -> Self {
        let entry = self.entry_mut::<P>().unwrap_or_else(|| {
            panic!("{} is not in this plugin group", std::any::type_name::<P>())
        });
        entry.enabled = false;
        self
    }

    fn entry_mut<P: Plugin + 'static>(&mut self) -> Option<&mut PluginGroupEntry> {
        let id = std::any::TypeId::of::<P>();
        self.entries.iter_mut().find(|entry| entry.id == id)
    }
}

impl PluginGroup for PluginGroupBuilder {
    fn build(self) -> PluginGroupBuilder {
        self
    }
}

macro_rules! impl_plugin_group_for_tuple {
    ($(($plugin_type:ident, $plugin_value:ident)),+) => {
        impl<$($plugin_type: Plugin + 'static),+> PluginGroup for ($($plugin_type,)+) {
            fn build(self) -> PluginGroupBuilder {
                let ($($plugin_value,)+) = self;
                PluginGroupBuilder::new()$(.add_plugin($plugin_value))+
            }
        }
    };
}

impl_plugin_group_for_tuple!((P1, plugin1));
impl_plugin_group_for_tuple!((P1, plugin1), (P2, plugin2));
impl_plugin_group_for_tuple!((P1, plugin1), (P2, plugin2), (P3, plugin3));
impl_plugin_group_for_tuple!((P1, plugin1), (P2, plugin2), (P3, plugin3), (P4, plugin4));
impl_plugin_group_for_tuple!(
    (P1, plugin1),
    (P2, plugin2),
    (P3, plugin3),
    (P4, plugin4),
    (P5, plugin5)
);
impl_plugin_group_for_tuple!(
    (P1, plugin1),
    (P2, plugin2),
    (P3, plugin3),
    (P4, plugin4),
    (P5, plugin5),
    (P6, plugin6)
);
impl_plugin_group_for_tuple!(
    (P1, plugin1),
    (P2, plugin2),
    (P3, plugin3),
    (P4, plugin4),
    (P5, plugin5),
    (P6, plugin6),
    (P7, plugin7)
);
impl_plugin_group_for_tuple!(
    (P1, plugin1),
    (P2, plugin2),
    (P3, plugin3),
    (P4, plugin4),
    (P5, plugin5),
    (P6, plugin6),
    (P7, plugin7),
    (P8, plugin8)
);
impl_plugin_group_for_tuple!(
    (P1, plugin1),
    (P2, plugin2),
    (P3, plugin3),
    (P4, plugin4),
    (P5, plugin5),
    (P6, plugin6),
    (P7, plugin7),
    (P8, plugin8),
    (P9, plugin9)
);
impl_plugin_group_for_tuple!(
    (P1, plugin1),
    (P2, plugin2),
    (P3, plugin3),
    (P4, plugin4),
    (P5, plugin5),
    (P6, plugin6),
    (P7, plugin7),
    (P8, plugin8),
    (P9, plugin9),
    (P10, plugin10)
);
impl_plugin_group_for_tuple!(
    (P1, plugin1),
    (P2, plugin2),
    (P3, plugin3),
    (P4, plugin4),
    (P5, plugin5),
    (P6, plugin6),
    (P7, plugin7),
    (P8, plugin8),
    (P9, plugin9),
    (P10, plugin10),
    (P11, plugin11)
);
impl_plugin_group_for_tuple!(
    (P1, plugin1),
    (P2, plugin2),
    (P3, plugin3),
    (P4, plugin4),
    (P5, plugin5),
    (P6, plugin6),
    (P7, plugin7),
    (P8, plugin8),
    (P9, plugin9),
    (P10, plugin10),
    (P11, plugin11),
    (P12, plugin12)
);
impl_plugin_group_for_tuple!(
    (P1, plugin1),
    (P2, plugin2),
    (P3, plugin3),
    (P4, plugin4),
    (P5, plugin5),
    (P6, plugin6),
    (P7, plugin7),
    (P8, plugin8),
    (P9, plugin9),
    (P10, plugin10),
    (P11, plugin11),
    (P12, plugin12),
    (P13, plugin13)
);
impl_plugin_group_for_tuple!(
    (P1, plugin1),
    (P2, plugin2),
    (P3, plugin3),
    (P4, plugin4),
    (P5, plugin5),
    (P6, plugin6),
    (P7, plugin7),
    (P8, plugin8),
    (P9, plugin9),
    (P10, plugin10),
    (P11, plugin11),
    (P12, plugin12),
    (P13, plugin13),
    (P14, plugin14)
);
impl_plugin_group_for_tuple!(
    (P1, plugin1),
    (P2, plugin2),
    (P3, plugin3),
    (P4, plugin4),
    (P5, plugin5),
    (P6, plugin6),
    (P7, plugin7),
    (P8, plugin8),
    (P9, plugin9),
    (P10, plugin10),
    (P11, plugin11),
    (P12, plugin12),
    (P13, plugin13),
    (P14, plugin14),
    (P15, plugin15)
);
impl_plugin_group_for_tuple!(
    (P1, plugin1),
    (P2, plugin2),
    (P3, plugin3),
    (P4, plugin4),
    (P5, plugin5),
    (P6, plugin6),
    (P7, plugin7),
    (P8, plugin8),
    (P9, plugin9),
    (P10, plugin10),
    (P11, plugin11),
    (P12, plugin12),
    (P13, plugin13),
    (P14, plugin14),
    (P15, plugin15),
    (P16, plugin16)
);

impl App {
    pub fn add_plugin<P: Plugin + 'static>(&mut self, plugin: P) -> &mut Self {
        if self.claim_plugin(std::any::TypeId::of::<P>()) {
            self.hold_needs(std::any::type_name::<P>(), plugin.needs());
            plugin.build(self);
        }
        self
    }

    pub fn add_plugins(&mut self, group: impl PluginGroup) -> &mut Self {
        for entry in group.build().entries {
            if !entry.enabled || !self.claim_plugin(entry.id) {
                continue;
            }
            self.hold_needs(entry.name, entry.plugin.needs());
            entry.plugin.build(self);
        }
        self
    }

    fn claim_plugin(&mut self, id: std::any::TypeId) -> bool {
        if self.added_plugins.contains(&id) {
            return false;
        }
        self.added_plugins.push(id);
        true
    }
}
