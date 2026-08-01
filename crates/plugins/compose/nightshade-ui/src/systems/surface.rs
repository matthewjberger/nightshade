macro_rules! ui_resource_surface {
    (
        $(#[$meta:meta])*
        $name:ident {
            write { $($write_field:ident: $write_type:ty,)+ }
            read { $($read_field:ident: $read_type:ty,)+ }
        }
    ) => {
        $(#[$meta])*
        pub struct $name<'a> {
            $(pub $write_field: &'a mut $write_type,)+
            $(pub $read_field: &'a $read_type,)+
        }

        impl $name<'_> {
            pub fn reborrow(&mut self) -> $name<'_> {
                $name {
                    $($write_field: self.$write_field,)+
                    $($read_field: self.$read_field,)+
                }
            }
        }

        impl<W> nightshade::ecs::system_param::ExtractParam<W> for $name<'_>
        where
            W: nightshade::ecs::ResourceHost,
        {
            type State = ();
            type Owned = ();
            type Item<'item> = $name<'item>;

            fn init() -> Self::State {}

            fn resources() -> Vec<nightshade::ecs::system_param::ResourceNeed> {
                use nightshade::ecs::system_param::ResourceNeed;
                vec![
                    $(ResourceNeed::write::<$write_type>(),)+
                    $(ResourceNeed::read::<$read_type>(),)+
                ]
            }

            fn extract(_state: &mut (), _host: &W) {}

            fn build<'a, I: Iterator<Item = Option<nightshade::ecs::ResourceSlot<'a>>>>(
                _owned: &'a mut (),
                slots: &mut I,
                _resources: &'a nightshade::ecs::ResourceMap,
            ) -> $name<'a> {
                $name {
                    $($write_field: next_surface_resource_mut::<$write_type>(slots),)+
                    $($read_field: next_surface_resource_ref::<$read_type>(slots),)+
                }
            }

            fn apply(_state: &mut (), _owned: (), _host: &W) {}

            fn access() -> nightshade::ecs::SystemAccess {
                use nightshade::ecs::{Access, SystemAccess};
                SystemAccess::empty()
                    $(.with_resource(std::any::TypeId::of::<$write_type>(), Access::Write))+
                    $(.with_resource(std::any::TypeId::of::<$read_type>(), Access::Read))+
            }
        }
    };
}

fn next_surface_slot<'a>(
    slots: &mut impl Iterator<Item = Option<nightshade::ecs::ResourceSlot<'a>>>,
) -> nightshade::ecs::ResourceSlot<'a> {
    slots
        .next()
        .expect("the runner borrows one slot per resource a surface names")
        .expect("a surface names no resource it can do without")
}

fn next_surface_resource_mut<'a, T: Send + Sync + 'static>(
    slots: &mut impl Iterator<Item = Option<nightshade::ecs::ResourceSlot<'a>>>,
) -> &'a mut T {
    next_surface_slot(slots).into_mut::<T>()
}

fn next_surface_resource_ref<'a, T: Send + Sync + 'static>(
    slots: &mut impl Iterator<Item = Option<nightshade::ecs::ResourceSlot<'a>>>,
) -> &'a T {
    next_surface_slot(slots).into_ref::<T>()
}

ui_resource_surface! {
    RenderSync {
        write {
            frame: crate::resources::RetainedUiFrame,
            dirty: crate::resources::RetainedUiDirty,
            runtime: crate::resources::RetainedUiRuntime,
            interaction: crate::resources::RetainedUiInteraction,
            text_cache: crate::resources::RetainedUiTextCache,
            slots: crate::resources::RenderSlotAllocator,
            text_state: nightshade_text::contract::resources::TextState,
        }
        read {
            theme: crate::resources::ThemeState,
            overlays: crate::resources::RetainedUiOverlays,
            timing: crate::resources::RetainedUiTiming,
            dock: crate::resources::DockState,
            progress_bar: crate::resources::TopProgressBar,
            window: nightshade_platform::contract::window::Window,
            input: nightshade_platform::contract::input::Input,
        }
    }
}
