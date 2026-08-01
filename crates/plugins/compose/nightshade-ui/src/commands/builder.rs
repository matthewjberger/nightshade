use nalgebra_glm;
use nalgebra_glm::{Vec2, Vec4};

use crate::components::layer::UiLayer;
use crate::components::layout_types::{
    BoundaryLayout, FlowAlignment, FlowDirection, FlowLayout, ScalingMode, SolidLayout,
    UiLayoutType, WindowLayout,
};
use crate::components::{
    AutoSizeMode, StateTransition, ThemeColor, UiAnimationPhase, UiAnimationType, UiDepthMode,
    UiLayoutNode, UiLayoutRoot, UiNodeAnimation, UiNodeColor, UiNodeContent, UiNodeInteraction,
    UiStateWeights, UiThemeBinding,
};
use crate::data::easing::EasingFunction;
use crate::data::state::UiStateTrait;
use crate::data::units::UiValue;
use crate::resources::theme::UiTheme;
use nightshade_paint::data::Anchor;
use nightshade_text::contract::data::text_data::{TextAlignment, VerticalAlignment};

use crate::prelude::*;
use nightshade::ecs::system_param::Commands;
macro_rules! ui_build_surface {
    (
        write { $($write_field:ident: $write_type:ty,)+ }
        read { $($read_field:ident: $read_type:ty,)+ }
    ) => {
        pub struct UiBuildContext<'a, 'c> {
            pub commands: &'a mut Commands<'c>,
            $(pub $write_field: &'a mut $write_type,)+
            $(pub $read_field: &'a $read_type,)+
        }

        pub struct UiBuild<'a> {
            commands: Commands<'a>,
            $($write_field: &'a mut $write_type,)+
            $($read_field: &'a $read_type,)+
        }

        impl<'a> UiBuild<'a> {
            $(
                pub fn $write_field(&mut self) -> &mut $write_type {
                    self.$write_field
                }
            )+
            $(
                pub fn $read_field(&self) -> &$read_type {
                    self.$read_field
                }
            )+

            pub fn context(&mut self) -> UiBuildContext<'_, 'a> {
                UiBuildContext {
                    commands: &mut self.commands,
                    $($write_field: self.$write_field,)+
                    $($read_field: self.$read_field,)+
                }
            }
        }

        pub fn ui_build_context<'a, 'c>(
            resources: &'a mut nightshade::ecs::ResourceMap,
            commands: &'a mut Commands<'c>,
        ) -> UiBuildContext<'a, 'c> {
            use std::any::TypeId;
            let [$($write_field,)+ $($read_field,)+] = resources.res_disjoint_mut([
                $(TypeId::of::<$write_type>(),)+
                $(TypeId::of::<$read_type>(),)+
            ]);
            UiBuildContext {
                commands,
                $($write_field: $write_field
                    .downcast_mut()
                    .expect(stringify!($write_type)),)+
                $($read_field: $read_field
                    .downcast_ref()
                    .expect(stringify!($read_type)),)+
            }
        }

        impl<W> nightshade::ecs::system_param::ExtractParam<W> for UiBuild<'_>
        where
            W: nightshade::ecs::system_param::EcsHost + nightshade::ecs::ResourceHost,
        {
            type State = ();
            type Owned = nightshade::ecs::system_param::CommandBuffer;
            type Item<'item> = UiBuild<'item>;

            fn init() -> Self::State {}

            fn resources() -> Vec<nightshade::ecs::system_param::ResourceNeed> {
                use nightshade::ecs::system_param::ResourceNeed;
                vec![
                    $(ResourceNeed::write::<$write_type>(),)+
                    $(ResourceNeed::read::<$read_type>(),)+
                ]
            }

            fn extract(
                _state: &mut (),
                host: &W,
            ) -> nightshade::ecs::system_param::CommandBuffer {
                <Commands<'_> as nightshade::ecs::system_param::ExtractParam<W>>::extract(
                    &mut (),
                    host,
                )
            }

            fn build<'a, I: Iterator<Item = Option<nightshade::ecs::ResourceSlot<'a>>>>(
                owned: &'a mut nightshade::ecs::system_param::CommandBuffer,
                slots: &mut I,
                resources: &'a nightshade::ecs::ResourceMap,
            ) -> UiBuild<'a> {
                UiBuild {
                    commands: <Commands<'_> as nightshade::ecs::system_param::ExtractParam<W>>::build(
                        owned,
                        &mut std::iter::empty(),
                        resources,
                    ),
                    $($write_field: next_resource_mut::<$write_type>(slots),)+
                    $($read_field: next_resource_ref::<$read_type>(slots),)+
                }
            }

            fn apply(
                _state: &mut (),
                owned: nightshade::ecs::system_param::CommandBuffer,
                host: &W,
            ) {
                <Commands<'_> as nightshade::ecs::system_param::ExtractParam<W>>::apply(
                    &mut (),
                    owned,
                    host,
                );
            }

            fn flush(host: &mut W) {
                <Commands<'_> as nightshade::ecs::system_param::ExtractParam<W>>::flush(host);
            }

            fn access() -> nightshade::ecs::SystemAccess {
                use nightshade::ecs::{Access, SystemAccess};
                SystemAccess::empty()
                    $(.with_resource(std::any::TypeId::of::<$write_type>(), Access::Write))+
                    $(.with_resource(std::any::TypeId::of::<$read_type>(), Access::Read))+
                    .merge(
                        <Commands<'_> as nightshade::ecs::system_param::ExtractParam<W>>::access(),
                    )
            }
        }
    };
}

ui_build_surface! {
    write {
        text_state: nightshade_text::contract::resources::TextState,
        theme: crate::resources::ThemeState,
        runtime: crate::resources::RetainedUiRuntime,
        groups: crate::resources::RetainedUiGroups,
        dirty: crate::resources::RetainedUiDirty,
        overlays: crate::resources::RetainedUiOverlays,
        interaction: crate::resources::RetainedUiInteraction,
        accessibility: crate::resources::RetainedUiAccessibility,
        images: crate::resources::UiImageLayerAllocator,
    }
    read {
        timing: crate::resources::RetainedUiTiming,
        frame: crate::resources::RetainedUiFrame,
        hierarchy: nightshade::ecs::HierarchyIndex,
    }
}

fn next_build_slot<'a>(
    slots: &mut impl Iterator<Item = Option<nightshade::ecs::ResourceSlot<'a>>>,
) -> nightshade::ecs::ResourceSlot<'a> {
    slots
        .next()
        .expect("the runner borrows one slot per resource a build names")
        .expect("a build names no resource it can do without")
}

fn next_resource_mut<'a, T: Send + Sync + 'static>(
    slots: &mut impl Iterator<Item = Option<nightshade::ecs::ResourceSlot<'a>>>,
) -> &'a mut T {
    next_build_slot(slots).into_mut::<T>()
}

fn next_resource_ref<'a, T: Send + Sync + 'static>(
    slots: &mut impl Iterator<Item = Option<nightshade::ecs::ResourceSlot<'a>>>,
) -> &'a T {
    next_build_slot(slots).into_ref::<T>()
}

impl<'a, 'c> UiBuildContext<'a, 'c> {
    pub fn edit<T: Clone + Send + Sync + Default + 'static>(
        &mut self,
        entity: nightshade::ecs::Entity,
        change: impl FnOnce(&mut T) + Send + Sync + 'static,
    ) {
        if let Some(value) = self.commands.staged_mut::<T>(entity) {
            change(value);
            self.commands.restage::<T>(entity);
            return;
        }
        self.commands.queue(move |storage, _| {
            if let Some(value) = storage.get_mut::<T>(entity) {
                change(value);
            }
        });
    }

    pub fn component<T: Clone + Send + Sync + Default + 'static>(
        &self,
        entity: nightshade::ecs::Entity,
    ) -> Option<&T> {
        self.commands.staged::<T>(entity)
    }

    pub fn reborrow(&mut self) -> UiBuildContext<'_, 'c> {
        UiBuildContext {
            commands: self.commands,
            text_state: self.text_state,
            theme: self.theme,
            runtime: self.runtime,
            groups: self.groups,
            dirty: self.dirty,
            overlays: self.overlays,
            interaction: self.interaction,
            accessibility: self.accessibility,
            images: self.images,
            timing: self.timing,
            frame: self.frame,
            hierarchy: self.hierarchy,
        }
    }
}

pub struct UiTreeBuilder<'a, 'c> {
    context: UiBuildContext<'a, 'c>,
    root: nightshade::ecs::Entity,
    current_parent: nightshade::ecs::Entity,
    parent_stack: Vec<nightshade::ecs::Entity>,
}

impl<'a, 'c> UiTreeBuilder<'a, 'c> {
    pub fn new(context: UiBuildContext<'a, 'c>) -> Self {
        let root = context.commands.spawn((UiLayoutRoot::default(),));
        ui_mark_children_dirty(context.dirty);

        Self {
            context,
            root,
            current_parent: root,
            parent_stack: Vec::new(),
        }
    }

    pub fn from_parent(
        context: UiBuildContext<'a, 'c>,
        parent_entity: nightshade::ecs::Entity,
    ) -> Self {
        ui_mark_children_dirty(context.dirty);
        Self {
            context,
            root: parent_entity,
            current_parent: parent_entity,
            parent_stack: Vec::new(),
        }
    }

    pub fn commands(&mut self) -> &mut Commands<'c> {
        self.context.commands
    }

    pub fn text_state_mut(&mut self) -> &mut nightshade_text::contract::resources::TextState {
        self.context.text_state
    }

    pub fn theme_state(&self) -> &crate::resources::ThemeState {
        self.context.theme
    }

    pub fn theme_state_mut(&mut self) -> &mut crate::resources::ThemeState {
        self.context.theme
    }

    pub fn runtime_mut(&mut self) -> &mut crate::resources::RetainedUiRuntime {
        self.context.runtime
    }

    pub fn groups_mut(&mut self) -> &mut crate::resources::RetainedUiGroups {
        self.context.groups
    }

    pub fn dirty_mut(&mut self) -> &mut crate::resources::RetainedUiDirty {
        self.context.dirty
    }

    pub fn hierarchy(&self) -> &nightshade::ecs::HierarchyIndex {
        self.context.hierarchy
    }

    pub fn context_reborrow(&mut self) -> UiBuildContext<'_, 'c> {
        self.context.reborrow()
    }

    pub fn images_and_runtime(
        &mut self,
    ) -> (
        &mut crate::resources::UiImageLayerAllocator,
        &mut crate::resources::RetainedUiRuntime,
    ) {
        (self.context.images, self.context.runtime)
    }

    pub fn dirty_and_commands(
        &mut self,
    ) -> (&mut crate::resources::RetainedUiDirty, &mut Commands<'c>) {
        (self.context.dirty, self.context.commands)
    }

    pub fn overlays_mut(&mut self) -> &mut crate::resources::RetainedUiOverlays {
        self.context.overlays
    }

    pub fn interaction_mut(&mut self) -> &mut crate::resources::RetainedUiInteraction {
        self.context.interaction
    }

    pub fn accessibility_mut(&mut self) -> &mut crate::resources::RetainedUiAccessibility {
        self.context.accessibility
    }

    pub fn accessibility_and_commands(
        &mut self,
    ) -> (
        &mut crate::resources::RetainedUiAccessibility,
        &mut Commands<'c>,
    ) {
        (self.context.accessibility, self.context.commands)
    }

    pub fn text_state_and_commands(
        &mut self,
    ) -> (
        &mut nightshade_text::contract::resources::TextState,
        &mut Commands<'c>,
    ) {
        (self.context.text_state, self.context.commands)
    }

    pub fn images_mut(&mut self) -> &mut crate::resources::UiImageLayerAllocator {
        self.context.images
    }

    pub fn timing(&self) -> &crate::resources::RetainedUiTiming {
        self.context.timing
    }

    pub fn frame(&self) -> &crate::resources::RetainedUiFrame {
        self.context.frame
    }

    pub fn component<T: Send + Sync + 'static>(
        &self,
        entity: nightshade::ecs::Entity,
    ) -> Option<&T> {
        self.context.commands.staged::<T>(entity)
    }

    pub fn edit<T: Clone + Send + Sync + Default + 'static>(
        &mut self,
        entity: nightshade::ecs::Entity,
        change: impl FnOnce(&mut T) + Send + Sync + 'static,
    ) {
        self.context.edit(entity, change);
    }

    pub fn finish_subtree(self) {}

    pub fn assign_tab_index(&mut self, entity: nightshade::ecs::Entity) {
        let tab_index = self.context.runtime.next_tab_index;
        self.context.runtime.next_tab_index += 1;
        self.context.commands.queue(move |storage, _| {
            if let Some(interaction) =
                storage.get_mut::<crate::components::UiNodeInteraction>(entity)
            {
                interaction.tab_index = Some(tab_index);
            }
        });
    }

    pub fn set_absolute_scale(&mut self, scale: f32) {
        let root = self.root;
        self.context.commands.queue(move |storage, _| {
            if let Some(layout_root) = storage.get_mut::<crate::components::UiLayoutRoot>(root) {
                layout_root.absolute_scale = scale;
            }
        });
    }

    pub fn with_absolute_scale(mut self, scale: f32) -> Self {
        self.set_absolute_scale(scale);
        self
    }

    pub fn root_entity(&self) -> nightshade::ecs::Entity {
        self.root
    }

    pub fn current_parent(&self) -> nightshade::ecs::Entity {
        self.current_parent
    }

    pub fn active_theme(&self) -> &UiTheme {
        self.context.theme.active_theme()
    }

    pub fn add_node(&mut self) -> UiNodeBuilder<'_, 'a, 'c> {
        let entity = self.context.commands.reserve_entity();
        ui_mark_layout_dirty(self.context.dirty);
        let parent = self.current_parent;
        UiNodeBuilder {
            tree: self,
            entity,
            payload: Some(NodePayload {
                parent,
                node: UiLayoutNode::default(),
                color: UiNodeColor::default(),
                content: UiNodeContent::default(),
                weights: UiStateWeights::default(),
                interaction: None,
                theme_binding: None,
                transform_states: None,
                shadow: None,
                effect: None,
            }),
        }
    }

    pub fn push_parent(&mut self, entity: nightshade::ecs::Entity) {
        debug_assert!(
            entity != self.current_parent && !self.parent_stack.contains(&entity),
            "push_parent cycle detected: entity {:?} is already in the parent chain",
            entity
        );
        self.parent_stack.push(self.current_parent);
        self.current_parent = entity;
    }

    pub fn pop_parent(&mut self) {
        if let Some(parent) = self.parent_stack.pop() {
            self.current_parent = parent;
        }
    }

    pub fn in_parent<R>(
        &mut self,
        entity: nightshade::ecs::Entity,
        f: impl FnOnce(&mut Self) -> R,
    ) -> R {
        self.push_parent(entity);
        let result = f(self);
        self.pop_parent();
        result
    }

    pub fn children_of_widget<'b>(
        &'b mut self,
        entity: nightshade::ecs::Entity,
    ) -> ParentScope<'b, 'a, 'c> {
        ParentScope::push(self, entity)
    }

    pub fn finish(self) -> nightshade::ecs::Entity {
        self.root
    }
}

struct NodePayload {
    parent: nightshade::ecs::Entity,
    node: UiLayoutNode,
    color: UiNodeColor,
    content: UiNodeContent,
    weights: UiStateWeights,
    interaction: Option<UiNodeInteraction>,
    theme_binding: Option<UiThemeBinding>,
    transform_states: Option<crate::components::UiNodeTransformStates>,
    shadow: Option<crate::components::UiNodeShadow>,
    effect: Option<crate::components::UiNodeEffect>,
}

pub struct UiNodeBuilder<'b, 'a, 'c> {
    tree: &'b mut UiTreeBuilder<'a, 'c>,
    entity: nightshade::ecs::Entity,
    payload: Option<NodePayload>,
}

impl UiNodeBuilder<'_, '_, '_> {
    fn payload(&mut self) -> &mut NodePayload {
        self.payload
            .as_mut()
            .expect("the node builder was already spent")
    }

    fn node(&mut self) -> &mut UiLayoutNode {
        &mut self.payload().node
    }

    fn color(&mut self) -> &mut UiNodeColor {
        &mut self.payload().color
    }

    fn content(&mut self) -> &mut UiNodeContent {
        &mut self.payload().content
    }

    fn weights(&mut self) -> &mut UiStateWeights {
        &mut self.payload().weights
    }

    fn interaction(&mut self) -> &mut UiNodeInteraction {
        self.payload().interaction.get_or_insert_default()
    }

    fn theme_binding(&mut self) -> &mut UiThemeBinding {
        self.payload().theme_binding.get_or_insert_default()
    }

    fn transform_states(&mut self) -> &mut crate::components::UiNodeTransformStates {
        self.payload().transform_states.get_or_insert_default()
    }

    fn shadow(&mut self) -> &mut crate::components::UiNodeShadow {
        self.payload().shadow.get_or_insert_default()
    }

    fn effect(&mut self) -> &mut crate::components::UiNodeEffect {
        self.payload().effect.get_or_insert_default()
    }

    fn flush(&mut self) {
        let Some(payload) = self.payload.take() else {
            return;
        };
        let entity = self.entity;
        let commands = self.tree.commands();
        commands.spawn_reserved(entity, ());
        commands.set(entity, payload.node);
        commands.set(entity, payload.color);
        commands.set(entity, payload.content);
        commands.set(entity, payload.weights);
        commands.set(entity, nightshade::ecs::ChildOf(payload.parent));
        if let Some(interaction) = payload.interaction {
            commands.set(entity, interaction);
        }
        if let Some(theme_binding) = payload.theme_binding {
            commands.set(entity, theme_binding);
        }
        if let Some(transform_states) = payload.transform_states {
            commands.set(entity, transform_states);
        }
        if let Some(shadow) = payload.shadow {
            commands.set(entity, shadow);
        }
        if let Some(effect) = payload.effect {
            commands.set(entity, effect);
        }
    }
}

impl Drop for UiNodeBuilder<'_, '_, '_> {
    fn drop(&mut self) {
        self.flush();
    }
}

fn default_text_content(
    text_slot: usize,
    font_size: f32,
    font_kind: nightshade_text::contract::data::text_data::FontKind,
) -> UiNodeContent {
    UiNodeContent::Text {
        text_slot,
        font_index: 0,
        font_size_override: Some(font_size),
        outline_color: Vec4::new(0.0, 0.0, 0.0, 0.0),
        outline_width: 0.0,
        alignment: TextAlignment::Center,
        vertical_alignment: VerticalAlignment::Middle,
        overflow: crate::components::TextOverflow::default(),
        monospace_width: None,
        font_kind,
    }
}

impl<'b, 'a, 'c> UiNodeBuilder<'b, 'a, 'c> {
    pub fn entity(mut self) -> nightshade::ecs::Entity {
        self.flush();
        self.entity
    }

    pub fn boundary(
        mut self,
        position_1: impl Into<UiValue<Vec2>>,
        position_2: impl Into<UiValue<Vec2>>,
    ) -> Self {
        let layout = UiLayoutType::Boundary(BoundaryLayout {
            position_1: position_1.into(),
            position_2: position_2.into(),
        });
        {
            let node = self.node();
            node.base_layout = Some(layout);
        }
        self
    }

    pub fn window(
        mut self,
        position: impl Into<UiValue<Vec2>>,
        size: impl Into<UiValue<Vec2>>,
        anchor: Anchor,
    ) -> Self {
        let layout = UiLayoutType::Window(WindowLayout {
            position: position.into(),
            size: size.into(),
            anchor,
        });
        {
            let node = self.node();
            node.base_layout = Some(layout);
        }
        self
    }

    pub fn solid(
        mut self,
        size: impl Into<UiValue<Vec2>>,
        scaling: ScalingMode,
        alignment: Vec2,
    ) -> Self {
        let layout = UiLayoutType::Solid(SolidLayout {
            size: size.into(),
            scaling,
            alignment,
        });
        {
            let node = self.node();
            node.base_layout = Some(layout);
        }
        self
    }

    pub fn flow(mut self, direction: FlowDirection, padding: f32, spacing: f32) -> Self {
        {
            let node = self.node();
            node.flow_layout = Some(FlowLayout {
                direction,
                padding,
                spacing,
                alignment: FlowAlignment::Start,
                cross_alignment: FlowAlignment::Start,
                wrap: false,
            });
        }
        self
    }

    pub fn flow_with_alignment(
        mut self,
        direction: FlowDirection,
        padding: f32,
        spacing: f32,
        alignment: FlowAlignment,
        cross_alignment: FlowAlignment,
    ) -> Self {
        {
            let node = self.node();
            node.flow_layout = Some(FlowLayout {
                direction,
                padding,
                spacing,
                alignment,
                cross_alignment,
                wrap: false,
            });
        }
        self
    }

    pub fn flow_wrap(mut self) -> Self {
        let node = self.node();
        if let Some(flow) = &mut node.flow_layout {
            flow.wrap = true;
        }
        self
    }

    pub fn flow_child(mut self, size: impl Into<UiValue<Vec2>>) -> Self {
        {
            let node = self.node();
            node.flow_child_size = Some(size.into());
        }
        self
    }

    pub fn size_px(self, width: f32, height: f32) -> Self {
        self.flow_child(crate::data::units::Ab(nalgebra_glm::vec2(width, height)))
    }

    pub fn fill_width(self) -> Self {
        self.flow_child(crate::data::units::Rl(nalgebra_glm::vec2(100.0, 0.0)))
    }

    pub fn fill(self) -> Self {
        self.flow_child(crate::data::units::Rl(nalgebra_glm::vec2(100.0, 100.0)))
    }

    pub fn auto_height(self) -> Self {
        self.auto_size(AutoSizeMode::Height)
    }

    pub fn size(
        self,
        width: crate::data::units::Length,
        height: crate::data::units::Length,
    ) -> Self {
        use crate::data::units::UiAxis;
        let mut value = UiValue::<Vec2>::default();
        if !width.is_auto() {
            value = value + width.into_axis(UiAxis::Horizontal);
        }
        if !height.is_auto() {
            value = value + height.into_axis(UiAxis::Vertical);
        }
        let auto = match (width.is_auto(), height.is_auto()) {
            (true, true) => Some(AutoSizeMode::Both),
            (true, false) => Some(AutoSizeMode::Width),
            (false, true) => Some(AutoSizeMode::Height),
            (false, false) => None,
        };
        let this = self.flow_child(value);
        match auto {
            Some(mode) => this.auto_size(mode),
            None => this,
        }
    }

    pub fn flow_vertical(self) -> Self {
        self.flow(FlowDirection::Vertical, 0.0, 0.0)
    }

    pub fn flow_horizontal(self) -> Self {
        self.flow(FlowDirection::Horizontal, 0.0, 0.0)
    }

    pub fn padding(mut self, padding: f32) -> Self {
        let node = self.node();
        if let Some(flow) = node.flow_layout.as_mut() {
            flow.padding = padding;
        }
        self
    }

    pub fn gap(mut self, spacing: f32) -> Self {
        let node = self.node();
        if let Some(flow) = node.flow_layout.as_mut() {
            flow.spacing = spacing;
        }
        self
    }

    pub fn align_main(mut self, alignment: FlowAlignment) -> Self {
        let node = self.node();
        if let Some(flow) = node.flow_layout.as_mut() {
            flow.alignment = alignment;
        }
        self
    }

    pub fn align_cross(mut self, alignment: FlowAlignment) -> Self {
        let node = self.node();
        if let Some(flow) = node.flow_layout.as_mut() {
            flow.cross_alignment = alignment;
        }
        self
    }

    pub fn fg(self, role: ThemeColor) -> Self {
        self.with_theme_color::<crate::data::state::UiBase>(role)
    }

    pub fn fg_hover(self, role: ThemeColor) -> Self {
        self.with_theme_color::<crate::data::state::UiHover>(role)
    }

    pub fn fg_pressed(self, role: ThemeColor) -> Self {
        self.with_theme_color::<crate::data::state::UiPressed>(role)
    }

    pub fn label(self, text: &str, size: f32, role: ThemeColor) -> Self {
        self.with_text(text, size).text_left().fg(role)
    }

    pub fn rect(self, corner_radius: f32) -> Self {
        self.with_rect(corner_radius, 0.0, nalgebra_glm::vec4(0.0, 0.0, 0.0, 0.0))
    }

    pub fn window_at(
        self,
        position: impl Into<UiValue<Vec2>>,
        size: impl Into<UiValue<Vec2>>,
    ) -> Self {
        self.window(position, size, nightshade_paint::data::Anchor::TopLeft)
    }

    pub fn card(self) -> Self {
        let theme = self.tree.active_theme();
        let radius = theme.corner_radius;
        let border_color = theme.border_color;
        self.with_rect(radius, 1.0, border_color)
            .with_theme_border_color(ThemeColor::Border)
            .with_theme_color::<crate::data::state::UiBase>(ThemeColor::Panel)
            .with_theme_effect_role(crate::components::ThemeEffect::PanelEffect)
    }

    pub fn flex_grow(mut self, weight: f32) -> Self {
        {
            let node = self.node();
            node.flex_grow = Some(weight);
        }
        self
    }

    pub fn with_min_size(mut self, min: Vec2) -> Self {
        {
            let node = self.node();
            node.min_size = Some(min);
        }
        self
    }

    pub fn auto_size(mut self, mode: AutoSizeMode) -> Self {
        {
            let node = self.node();
            node.auto_size = mode;
        }
        self
    }

    pub fn auto_size_padding(mut self, padding: Vec2) -> Self {
        {
            let node = self.node();
            node.auto_size_padding = padding;
        }
        self
    }

    pub fn responsive_size_at(
        mut self,
        breakpoint: crate::resources::UiBreakpoint,
        size: crate::data::units::UiValue<Vec2>,
    ) -> Self {
        {
            let node = self.node();
            let responsive = node
                .responsive
                .get_or_insert_with(crate::components::UiResponsive::default);
            responsive.override_mut(breakpoint).size = Some(size);
        }
        self
    }

    pub fn responsive_position_1_at(
        mut self,
        breakpoint: crate::resources::UiBreakpoint,
        position: crate::data::units::UiValue<Vec2>,
    ) -> Self {
        {
            let node = self.node();
            let responsive = node
                .responsive
                .get_or_insert_with(crate::components::UiResponsive::default);
            responsive.override_mut(breakpoint).position_1 = Some(position);
        }
        self
    }

    pub fn responsive_position_2_at(
        mut self,
        breakpoint: crate::resources::UiBreakpoint,
        position: crate::data::units::UiValue<Vec2>,
    ) -> Self {
        {
            let node = self.node();
            let responsive = node
                .responsive
                .get_or_insert_with(crate::components::UiResponsive::default);
            responsive.override_mut(breakpoint).position_2 = Some(position);
        }
        self
    }

    pub fn responsive_visible_at(
        mut self,
        breakpoint: crate::resources::UiBreakpoint,
        visible: bool,
    ) -> Self {
        {
            let node = self.node();
            let responsive = node
                .responsive
                .get_or_insert_with(crate::components::UiResponsive::default);
            responsive.override_mut(breakpoint).visible = Some(visible);
        }
        self
    }

    pub fn with_rect(mut self, corner_radius: f32, border_width: f32, border_color: Vec4) -> Self {
        {
            let content = self.content();
            *content = UiNodeContent::Rect {
                corner_radius,
                border_width,
                border_color,
            };
        }
        self
    }

    pub fn with_shadow(
        mut self,
        color: Vec4,
        offset: nalgebra_glm::Vec2,
        blur: f32,
        spread: f32,
    ) -> Self {
        {
            let shadow = self.shadow();
            *shadow = crate::components::UiNodeShadow {
                color,
                offset,
                blur,
                spread,
            };
        }
        self
    }

    pub fn with_text(mut self, text: &str, font_size: f32) -> Self {
        let text_slot = self.tree.text_state_mut().cache.add_text(text);
        {
            let content = self.content();
            *content = default_text_content(
                text_slot,
                font_size,
                nightshade_text::contract::data::text_data::FontKind::Default,
            );
        }
        self
    }

    pub fn with_text_slot(mut self, text_slot: usize, font_size: f32) -> Self {
        {
            let content = self.content();
            *content = default_text_content(
                text_slot,
                font_size,
                nightshade_text::contract::data::text_data::FontKind::Default,
            );
        }
        self
    }

    pub fn with_icon(mut self, icon: crate::data::icons::IconGlyph, font_size: f32) -> Self {
        let mut buf = [0u8; 4];
        let icon_str: &str = icon.codepoint.encode_utf8(&mut buf);
        let text_slot = self.tree.text_state_mut().cache.add_text(icon_str);
        {
            let content = self.content();
            *content = default_text_content(text_slot, font_size, icon.font_kind);
        }
        self
    }

    pub fn text_left(self) -> Self {
        self.with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
    }

    pub fn text_center(self) -> Self {
        self.with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
    }

    pub fn text_right(self) -> Self {
        self.with_text_alignment(TextAlignment::Right, VerticalAlignment::Middle)
    }

    pub fn absolute_fill(self, inset: f32) -> Self {
        self.boundary(
            crate::data::units::Ab(nalgebra_glm::vec2(inset, inset)),
            crate::data::units::Ab(nalgebra_glm::vec2(-inset, -inset))
                + crate::data::units::Rl(nalgebra_glm::vec2(100.0, 100.0)),
        )
    }

    pub fn with_text_alignment(
        mut self,
        text_alignment: TextAlignment,
        text_vertical_alignment: VerticalAlignment,
    ) -> Self {
        let content = self.content();
        if let UiNodeContent::Text {
            alignment,
            vertical_alignment,
            ..
        } = content
        {
            *alignment = text_alignment;
            *vertical_alignment = text_vertical_alignment;
        }
        self
    }

    pub fn with_text_outline(mut self, color: Vec4, width: f32) -> Self {
        let content = self.content();
        if let UiNodeContent::Text {
            outline_color,
            outline_width,
            ..
        } = content
        {
            *outline_color = color;
            *outline_width = width;
        }
        self
    }

    pub fn with_text_overflow(mut self, mode: crate::components::TextOverflow) -> Self {
        let content = self.content();
        if let UiNodeContent::Text { overflow, .. } = content {
            *overflow = mode;
        }
        self
    }

    pub fn with_text_wrap(self) -> Self {
        self.with_text_overflow(crate::components::TextOverflow::Wrap)
    }

    pub fn color_raw<S: UiStateTrait>(mut self, color: Vec4) -> Self {
        {
            let node_color = self.color();
            node_color.colors[S::INDEX] = Some(color);
        }
        {
            let binding = self.theme_binding();
            binding.color_roles[S::INDEX] = None;
        }
        self
    }

    fn ensure_theme_binding(&mut self) -> &mut UiThemeBinding {
        self.theme_binding()
    }

    pub fn with_theme_color<S: UiStateTrait>(mut self, role: ThemeColor) -> Self {
        let color = {
            let theme = self.tree.active_theme();
            role.resolve(theme)
        };
        {
            let node_color = self.color();
            node_color.colors[S::INDEX] = Some(color);
        }
        self.ensure_theme_binding().color_roles[S::INDEX] = Some(role);
        self
    }

    pub fn with_theme_shadow_role<S: UiStateTrait>(
        mut self,
        role: crate::components::ThemeShadow,
    ) -> Self {
        self.ensure_theme_binding().shadow_roles[S::INDEX] = Some(role);
        self.tree.theme_state_mut().generation += 1;
        self
    }

    pub fn with_theme_effect_role(mut self, role: crate::components::ThemeEffect) -> Self {
        self.ensure_theme_binding().effect_role = Some(role);
        self.tree.theme_state_mut().generation += 1;
        self
    }

    pub fn with_effect(mut self, effect: crate::resources::theme::RectEffect) -> Self {
        let (kind, params) = effect.encode();
        {
            let comp = self.effect();
            comp.kind = kind;
            comp.params = params;
        }
        self
    }

    pub fn with_theme_border_color(mut self, role: ThemeColor) -> Self {
        let color = {
            let theme = self.tree.active_theme();
            role.resolve(theme)
        };
        let content = self.content();
        if let UiNodeContent::Rect { border_color, .. } = content {
            *border_color = color;
        }
        self.ensure_theme_binding().border_color_role = Some(role);
        self
    }

    pub fn with_image_uv(mut self, texture_index: u32, uv_min: Vec2, uv_max: Vec2) -> Self {
        {
            let content = self.content();
            *content = UiNodeContent::Image {
                texture_index,
                uv_min,
                uv_max,
            };
        }
        self
    }

    pub fn with_interaction(mut self) -> Self {
        self.interaction();
        self.node().pointer_events = true;
        self
    }

    pub fn with_cursor_icon(mut self, icon: nightshade_platform::CursorIcon) -> Self {
        {
            let interaction = self.interaction();
            interaction.cursor_icon = Some(icon);
        }
        self
    }

    pub fn with_tooltip(mut self, text: &str) -> Self {
        self.node().pointer_events = true;
        self.interaction().tooltip_text = Some(text.to_string());
        self
    }

    pub fn with_transition<S: UiStateTrait>(mut self, enter_speed: f32, exit_speed: f32) -> Self {
        {
            let weights = self.weights();
            let mut transition = StateTransition::for_state::<S>();
            transition.enter_speed = enter_speed;
            transition.exit_speed = exit_speed;
            weights.transitions[S::INDEX] = Some(transition);
        }
        self
    }

    pub fn with_eased_transition<S: UiStateTrait>(
        mut self,
        enter_speed: f32,
        exit_speed: f32,
        easing: EasingFunction,
    ) -> Self {
        {
            let weights = self.weights();
            weights.transitions[S::INDEX] = Some(StateTransition {
                enter_speed,
                exit_speed,
                easing,
                spring: None,
            });
        }
        self
    }

    pub fn with_spring_transition<S: UiStateTrait>(
        mut self,
        spring: crate::components::Spring,
    ) -> Self {
        {
            let weights = self.weights();
            let mut transition = StateTransition::for_state::<S>();
            transition.spring = Some(spring);
            weights.transitions[S::INDEX] = Some(transition);
        }
        self
    }

    pub fn with_state_offset<S: UiStateTrait>(mut self, offset: nalgebra_glm::Vec2) -> Self {
        {
            let states = self.transform_states();
            states.offsets[S::INDEX] = Some(offset);
        }
        self
    }

    pub fn with_state_scale<S: UiStateTrait>(mut self, scale: f32) -> Self {
        {
            let states = self.transform_states();
            states.scales[S::INDEX] = Some(scale);
        }
        self
    }

    pub fn with_depth(mut self, depth: UiDepthMode) -> Self {
        {
            let node = self.node();
            node.depth = depth;
        }

        self
    }

    pub fn with_clip(mut self) -> Self {
        {
            let node = self.node();
            node.clip_content = true;
        }
        self
    }

    pub fn with_visible(mut self, visible: bool) -> Self {
        {
            let node = self.node();
            node.visible = visible;
        }

        self
    }

    pub fn without_pointer_events(mut self) -> Self {
        {
            let node = self.node();
            node.pointer_events = false;
        }
        self
    }

    pub fn with_intro(mut self, animation: UiAnimationType, duration: f32) -> Self {
        {
            let node = self.node();
            let anim = node.animation.get_or_insert(UiNodeAnimation {
                intro: None,
                outro: None,
                duration,
                progress: 0.0,
                phase: UiAnimationPhase::Idle,
            });
            anim.intro = Some(animation);
            anim.duration = duration;
            if node.visible {
                anim.phase = UiAnimationPhase::IntroPlaying;
                anim.progress = 0.0;
            }
        }
        self
    }

    pub fn with_outro(mut self, animation: UiAnimationType, duration: f32) -> Self {
        {
            let node = self.node();
            let anim = node.animation.get_or_insert(UiNodeAnimation {
                intro: None,
                outro: None,
                duration,
                progress: 0.0,
                phase: UiAnimationPhase::Idle,
            });
            anim.outro = Some(animation);
            anim.duration = duration;
        }
        self
    }

    pub fn with_rotation(mut self, radians: f32) -> Self {
        {
            let node = self.node();
            node.rotation = radians;
        }
        self
    }

    pub fn with_layer(mut self, layer: UiLayer) -> Self {
        {
            let node = self.node();
            node.layer = Some(layer);
        }
        self
    }

    pub fn with_children(mut self, child_builder: impl FnOnce(&mut UiTreeBuilder<'a, 'c>)) -> Self {
        self.flush();
        let entity = self.entity;
        self.tree.push_parent(entity);
        child_builder(&mut *self.tree);
        self.tree.pop_parent();
        self
    }
}

pub struct ParentScope<'b, 'a: 'b, 'c> {
    tree: &'b mut UiTreeBuilder<'a, 'c>,
}

impl<'b, 'a: 'b, 'c> ParentScope<'b, 'a, 'c> {
    pub(crate) fn push(
        tree: &'b mut UiTreeBuilder<'a, 'c>,
        parent: nightshade::ecs::Entity,
    ) -> Self {
        tree.push_parent(parent);
        Self { tree }
    }
}

impl<'b, 'a: 'b, 'c> std::ops::Deref for ParentScope<'b, 'a, 'c> {
    type Target = UiTreeBuilder<'a, 'c>;
    fn deref(&self) -> &Self::Target {
        self.tree
    }
}

impl<'b, 'a: 'b, 'c> std::ops::DerefMut for ParentScope<'b, 'a, 'c> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.tree
    }
}

impl Drop for ParentScope<'_, '_, '_> {
    fn drop(&mut self) {
        self.tree.pop_parent();
    }
}

#[cfg(test)]
pub(crate) fn tree_over<R>(
    storage: &mut nightshade::ecs::Storage,
    resources: &mut nightshade::ecs::ResourceMap,
    build: impl FnOnce(&mut UiTreeBuilder<'_, '_>) -> R,
) -> R {
    nightshade::ecs::with_commands_over(storage, resources, |_storage, resources, commands| {
        let mut tree = UiTreeBuilder::new(ui_build_context(resources, commands));
        let result = build(&mut tree);
        tree.finish();
        result
    })
}
