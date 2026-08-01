use nightshade::ecs::system_param::{
    ExtractParam, StorageParam, WorldParam, assert_resources_named_once, declared_resources,
    resource_borrows,
};

pub struct RendererParam<'frame, R> {
    renderer: &'frame mut R,
}

impl<R> std::ops::Deref for RendererParam<'_, R> {
    type Target = R;

    fn deref(&self) -> &R {
        self.renderer
    }
}

impl<R> std::ops::DerefMut for RendererParam<'_, R> {
    fn deref_mut(&mut self) -> &mut R {
        self.renderer
    }
}

pub type PreRenderFn<R> =
    Box<dyn FnMut(&mut R, &mut nightshade::ecs::Storage, &mut nightshade::ecs::ResourceMap)>;

pub trait IntoPreRenderSystem<R, Marker>: Sized {
    fn into_pre_render_runner(self) -> PreRenderFn<R>;
}

pub struct RendererMarker;

pub struct RendererExtractMarker<E>(std::marker::PhantomData<fn() -> E>);

pub struct RendererQueryMarker<E, Q>(std::marker::PhantomData<fn() -> (E, Q)>);

pub struct RendererStorageMarker<E, S>(std::marker::PhantomData<fn() -> (E, S)>);

impl<Func, R> IntoPreRenderSystem<R, RendererMarker> for Func
where
    Func: FnMut(RendererParam<'_, R>) + for<'frame> FnMut(RendererParam<'frame, R>) + 'static,
{
    fn into_pre_render_runner(mut self) -> PreRenderFn<R> {
        Box::new(move |renderer: &mut R, _storage, _resources| {
            self(RendererParam { renderer });
        })
    }
}

impl<Func, R, Q> IntoPreRenderSystem<R, RendererQueryMarker<(), Q>> for Func
where
    Q: WorldParam,
    Func: FnMut(RendererParam<'_, R>, Q)
        + for<'frame> FnMut(RendererParam<'frame, R>, Q::Item<'frame>)
        + 'static,
{
    fn into_pre_render_runner(mut self) -> PreRenderFn<R> {
        Box::new(
            move |renderer: &mut R,
                  storage: &mut nightshade::ecs::Storage,
                  _resources: &mut nightshade::ecs::ResourceMap| {
                let query = Q::build(&*storage);
                self(RendererParam { renderer }, query);
            },
        )
    }
}

impl<Func, R, S> IntoPreRenderSystem<R, RendererStorageMarker<(), S>> for Func
where
    S: StorageParam,
    Func: FnMut(RendererParam<'_, R>, S)
        + for<'frame> FnMut(RendererParam<'frame, R>, S::Item<'frame>)
        + 'static,
{
    fn into_pre_render_runner(mut self) -> PreRenderFn<R> {
        Box::new(
            move |renderer: &mut R,
                  storage: &mut nightshade::ecs::Storage,
                  _resources: &mut nightshade::ecs::ResourceMap| {
                let rows = S::build(storage);
                self(RendererParam { renderer }, rows);
            },
        )
    }
}

macro_rules! impl_renderer_extract {
    ($($param:ident $state:ident $owned:ident),+) => {
        impl<Func, R, $($param,)+> IntoPreRenderSystem<R, RendererExtractMarker<($($param,)+)>> for Func
        where
            $($param: ExtractParam<nightshade::ecs::ResourceMap>,)+
            Func: FnMut(RendererParam<'_, R>, $($param,)+)
                + for<'frame> FnMut(RendererParam<'frame, R>, $($param::Item<'frame>,)+)
                + 'static,
        {
            fn into_pre_render_runner(mut self) -> PreRenderFn<R> {
                assert_resources_named_once::<Func>(&declared_resources(&[
                    $(<$param as ExtractParam<nightshade::ecs::ResourceMap>>::resources(),)+
                ]));
                $(let mut $state = <$param as ExtractParam<nightshade::ecs::ResourceMap>>::init();)+
                Box::new(move |renderer: &mut R,
                               _storage: &mut nightshade::ecs::Storage,
                               resources: &mut nightshade::ecs::ResourceMap| {
                    $(let mut $owned =
                        <$param as ExtractParam<nightshade::ecs::ResourceMap>>::extract(&mut $state, &*resources);)+
                    let declared = declared_resources(&[
                        $(<$param as ExtractParam<nightshade::ecs::ResourceMap>>::resources(),)+
                    ]);
                    {
                        let lent = &*resources;
                        let borrows = resource_borrows(lent, &declared);
                        let mut slots = lent.disjoint_slots(&borrows).into_iter();
                        $(let $owned =
                            <$param as ExtractParam<nightshade::ecs::ResourceMap>>::build(
                                &mut $owned,
                                &mut slots,
                                lent,
                            );)+
                        self(RendererParam { renderer }, $($owned,)+);
                    }
                    $(<$param as ExtractParam<nightshade::ecs::ResourceMap>>::apply(&mut $state, $owned, &*resources);)+
                })
            }
        }
    };
}

macro_rules! impl_renderer_extract_query {
    ($($param:ident $state:ident $owned:ident),+) => {
        impl<Func, R, $($param,)+ Q> IntoPreRenderSystem<R, RendererQueryMarker<($($param,)+), Q>>
            for Func
        where
            $($param: ExtractParam<nightshade::ecs::ResourceMap>,)+
            Q: WorldParam,
            Func: FnMut(RendererParam<'_, R>, $($param,)+ Q)
                + for<'frame> FnMut(RendererParam<'frame, R>, $($param::Item<'frame>,)+ Q::Item<'frame>)
                + 'static,
        {
            fn into_pre_render_runner(mut self) -> PreRenderFn<R> {
                assert_resources_named_once::<Func>(&declared_resources(&[
                    $(<$param as ExtractParam<nightshade::ecs::ResourceMap>>::resources(),)+
                ]));
                $(let mut $state = <$param as ExtractParam<nightshade::ecs::ResourceMap>>::init();)+
                Box::new(move |renderer: &mut R,
                               storage: &mut nightshade::ecs::Storage,
                               resources: &mut nightshade::ecs::ResourceMap| {
                    $(let mut $owned =
                        <$param as ExtractParam<nightshade::ecs::ResourceMap>>::extract(&mut $state, &*resources);)+
                    let declared = declared_resources(&[
                        $(<$param as ExtractParam<nightshade::ecs::ResourceMap>>::resources(),)+
                    ]);
                    {
                        let lent = &*resources;
                        let borrows = resource_borrows(lent, &declared);
                        let mut slots = lent.disjoint_slots(&borrows).into_iter();
                        $(let $owned =
                            <$param as ExtractParam<nightshade::ecs::ResourceMap>>::build(
                                &mut $owned,
                                &mut slots,
                                lent,
                            );)+
                        let query = Q::build(&*storage);
                        self(RendererParam { renderer }, $($owned,)+ query);
                    }
                    $(<$param as ExtractParam<nightshade::ecs::ResourceMap>>::apply(&mut $state, $owned, &*resources);)+
                })
            }
        }
    };
}

macro_rules! impl_renderer_extract_storage {
    ($($param:ident $state:ident $owned:ident),+) => {
        impl<Func, R, $($param,)+ S> IntoPreRenderSystem<R, RendererStorageMarker<($($param,)+), S>>
            for Func
        where
            $($param: ExtractParam<nightshade::ecs::ResourceMap>,)+
            S: StorageParam,
            Func: FnMut(RendererParam<'_, R>, $($param,)+ S)
                + for<'frame> FnMut(RendererParam<'frame, R>, $($param::Item<'frame>,)+ S::Item<'frame>)
                + 'static,
        {
            fn into_pre_render_runner(mut self) -> PreRenderFn<R> {
                assert_resources_named_once::<Func>(&declared_resources(&[
                    $(<$param as ExtractParam<nightshade::ecs::ResourceMap>>::resources(),)+
                ]));
                $(let mut $state = <$param as ExtractParam<nightshade::ecs::ResourceMap>>::init();)+
                Box::new(move |renderer: &mut R,
                               storage: &mut nightshade::ecs::Storage,
                               resources: &mut nightshade::ecs::ResourceMap| {
                    $(let mut $owned =
                        <$param as ExtractParam<nightshade::ecs::ResourceMap>>::extract(&mut $state, &*resources);)+
                    let declared = declared_resources(&[
                        $(<$param as ExtractParam<nightshade::ecs::ResourceMap>>::resources(),)+
                    ]);
                    {
                        let lent = &*resources;
                        let borrows = resource_borrows(lent, &declared);
                        let mut slots = lent.disjoint_slots(&borrows).into_iter();
                        $(let $owned =
                            <$param as ExtractParam<nightshade::ecs::ResourceMap>>::build(
                                &mut $owned,
                                &mut slots,
                                lent,
                            );)+
                        let rows = S::build(storage);
                        self(RendererParam { renderer }, $($owned,)+ rows);
                    }
                    $(<$param as ExtractParam<nightshade::ecs::ResourceMap>>::apply(&mut $state, $owned, &*resources);)+
                })
            }
        }
    };
}

impl_renderer_extract!(P0 state0 owned0);
impl_renderer_extract!(P0 state0 owned0, P1 state1 owned1);
impl_renderer_extract!(P0 state0 owned0, P1 state1 owned1, P2 state2 owned2);
impl_renderer_extract!(P0 state0 owned0, P1 state1 owned1, P2 state2 owned2, P3 state3 owned3);
impl_renderer_extract!(
    P0 state0 owned0, P1 state1 owned1, P2 state2 owned2, P3 state3 owned3, P4 state4 owned4
);
impl_renderer_extract!(
    P0 state0 owned0, P1 state1 owned1, P2 state2 owned2, P3 state3 owned3, P4 state4 owned4,
    P5 state5 owned5
);
impl_renderer_extract!(
    P0 state0 owned0, P1 state1 owned1, P2 state2 owned2, P3 state3 owned3, P4 state4 owned4,
    P5 state5 owned5, P6 state6 owned6
);
impl_renderer_extract!(
    P0 state0 owned0, P1 state1 owned1, P2 state2 owned2, P3 state3 owned3, P4 state4 owned4,
    P5 state5 owned5, P6 state6 owned6, P7 state7 owned7
);

impl_renderer_extract_query!(P0 state0 owned0);
impl_renderer_extract_query!(P0 state0 owned0, P1 state1 owned1);
impl_renderer_extract_query!(P0 state0 owned0, P1 state1 owned1, P2 state2 owned2);
impl_renderer_extract_query!(
    P0 state0 owned0, P1 state1 owned1, P2 state2 owned2, P3 state3 owned3
);
impl_renderer_extract_query!(
    P0 state0 owned0, P1 state1 owned1, P2 state2 owned2, P3 state3 owned3, P4 state4 owned4
);
impl_renderer_extract_query!(
    P0 state0 owned0, P1 state1 owned1, P2 state2 owned2, P3 state3 owned3, P4 state4 owned4,
    P5 state5 owned5
);
impl_renderer_extract_query!(
    P0 state0 owned0, P1 state1 owned1, P2 state2 owned2, P3 state3 owned3, P4 state4 owned4,
    P5 state5 owned5, P6 state6 owned6
);
impl_renderer_extract_query!(
    P0 state0 owned0, P1 state1 owned1, P2 state2 owned2, P3 state3 owned3, P4 state4 owned4,
    P5 state5 owned5, P6 state6 owned6, P7 state7 owned7
);

impl_renderer_extract_storage!(P0 state0 owned0);
impl_renderer_extract_storage!(P0 state0 owned0, P1 state1 owned1);
impl_renderer_extract_storage!(P0 state0 owned0, P1 state1 owned1, P2 state2 owned2);
impl_renderer_extract_storage!(
    P0 state0 owned0, P1 state1 owned1, P2 state2 owned2, P3 state3 owned3
);
impl_renderer_extract_storage!(
    P0 state0 owned0, P1 state1 owned1, P2 state2 owned2, P3 state3 owned3, P4 state4 owned4
);
impl_renderer_extract_storage!(
    P0 state0 owned0, P1 state1 owned1, P2 state2 owned2, P3 state3 owned3, P4 state4 owned4,
    P5 state5 owned5
);
impl_renderer_extract_storage!(
    P0 state0 owned0, P1 state1 owned1, P2 state2 owned2, P3 state3 owned3, P4 state4 owned4,
    P5 state5 owned5, P6 state6 owned6
);
impl_renderer_extract_storage!(
    P0 state0 owned0, P1 state1 owned1, P2 state2 owned2, P3 state3 owned3, P4 state4 owned4,
    P5 state5 owned5, P6 state6 owned6, P7 state7 owned7
);
