use super::*;

#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a component",
    note = "a component is any `Send + Sync + Default + 'static` type",
    note = "a `bundle!` struct is not one: spawn it on its own rather than inside a tuple"
)]
pub trait Component: Send + Sync + Default + 'static {
    fn component_mask(world: &mut Storage) -> Mask {
        world.component_key::<Self>().mask
    }

    fn strict_mask(world: &Storage) -> Option<Mask> {
        world.lookup_key::<Self>().map(|key| key.mask)
    }

    fn lookup_mask(world: &Storage) -> Mask {
        world
            .lookup_key::<Self>()
            .map_or(Mask::EMPTY, |key| key.mask)
    }

    fn write(self, world: &mut Storage, entity: Entity)
    where
        Self: Sized,
    {
        world.set(entity, self);
    }

    fn stage(&self, entity: Entity, staged: &mut StagedComponents)
    where
        Self: Clone,
    {
        staged.insert((TypeId::of::<Self>(), entity), Box::new(self.clone()));
    }
}

impl<T: Send + Sync + Default + 'static> Component for T {}

pub trait Bundle: sealed::SealedBundle {
    fn component_mask(world: &mut Storage) -> Mask;
    fn strict_mask(world: &Storage) -> Option<Mask>;
    fn lookup_mask(world: &Storage) -> Mask;
    fn write(self, world: &mut Storage, entity: Entity);
}

pub trait CloneBundle: Bundle {
    fn stage(&self, entity: Entity, staged: &mut StagedComponents);
}

impl sealed::SealedBundle for () {}

impl Bundle for () {
    fn component_mask(_world: &mut Storage) -> Mask {
        Mask::EMPTY
    }

    fn strict_mask(_world: &Storage) -> Option<Mask> {
        Some(Mask::EMPTY)
    }

    fn lookup_mask(_world: &Storage) -> Mask {
        Mask::EMPTY
    }

    fn write(self, _world: &mut Storage, _entity: Entity) {}
}

impl CloneBundle for () {
    fn stage(&self, _entity: Entity, _staged: &mut StagedComponents) {}
}

macro_rules! impl_bundle {
    ($($element:ident $value:ident),+) => {
        impl<$($element: Component),+> sealed::SealedBundle for ($($element,)+) {}

        impl<$($element: Component),+> Bundle for ($($element,)+) {
            fn component_mask(world: &mut Storage) -> Mask {
                let mut mask = Mask::EMPTY;
                $(
                    let element_mask = <$element as Component>::component_mask(world);
                    assert!(
                        !mask.intersects(element_mask),
                        "bundles must not repeat a component type"
                    );
                    mask |= element_mask;
                )+
                mask
            }

            fn lookup_mask(world: &Storage) -> Mask {
                let mut mask = Mask::EMPTY;
                $(mask |= <$element as Component>::lookup_mask(world);)+
                mask
            }

            fn strict_mask(world: &Storage) -> Option<Mask> {
                let mut mask = Mask::EMPTY;
                $(mask |= <$element as Component>::strict_mask(world)?;)+
                Some(mask)
            }

            fn write(self, world: &mut Storage, entity: Entity) {
                let ($($value,)+) = self;
                $(<$element as Component>::write($value, world, entity);)+
            }

        }

        impl<$($element: Component + Clone),+> CloneBundle for ($($element,)+) {
            fn stage(&self, entity: Entity, staged: &mut StagedComponents) {
                let ($($value,)+) = self;
                $(<$element as Component>::stage($value, entity, staged);)+
            }
        }
    };
}

impl_bundle!(A a);
impl_bundle!(A a, B b);
impl_bundle!(A a, B b, C c);
impl_bundle!(A a, B b, C c, D d);
impl_bundle!(A a, B b, C c, D d, E e);
impl_bundle!(A a, B b, C c, D d, E e, F f);
impl_bundle!(A a, B b, C c, D d, E e, F f, G g);
impl_bundle!(A a, B b, C c, D d, E e, F f, G g, H h);
impl_bundle!(A a, B b, C c, D d, E e, F f, G g, H h, I i);
impl_bundle!(A a, B b, C c, D d, E e, F f, G g, H h, I i, J j);
impl_bundle!(A a, B b, C c, D d, E e, F f, G g, H h, I i, J j, K k);
impl_bundle!(A a, B b, C c, D d, E e, F f, G g, H h, I i, J j, K k, L l);
impl_bundle!(A a, B b, C c, D d, E e, F f, G g, H h, I i, J j, K k, L l, M m);
impl_bundle!(A a, B b, C c, D d, E e, F f, G g, H h, I i, J j, K k, L l, M m, N n);
impl_bundle!(A a, B b, C c, D d, E e, F f, G g, H h, I i, J j, K k, L l, M m, N n, O o);
