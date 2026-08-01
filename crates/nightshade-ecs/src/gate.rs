use crate::system_param::{IntoSystem, into_standalone_runner};
use std::marker::PhantomData;

pub struct PlainMarker;

pub struct ParamMarker<Marker>(PhantomData<fn() -> Marker>);

pub trait GatedSystem<W, Marker> {
    fn into_gated_runner(self) -> impl FnMut(&mut W) + Send + 'static;
}

impl<W, F> GatedSystem<W, PlainMarker> for F
where
    F: FnMut(&mut W) + Send + 'static,
{
    fn into_gated_runner(self) -> impl FnMut(&mut W) + Send + 'static {
        self
    }
}

impl<W: 'static, Marker, F> GatedSystem<W, ParamMarker<Marker>> for F
where
    F: IntoSystem<W, Marker>,
{
    fn into_gated_runner(self) -> impl FnMut(&mut W) + Send + 'static {
        into_standalone_runner(self)
    }
}

pub struct SingleMarker<Marker>(PhantomData<fn() -> Marker>);

pub struct GroupMarker<Marker>(PhantomData<fn() -> Marker>);

pub trait IntoGroupRunner<W, Marker> {
    fn into_group_runner(self) -> impl FnMut(&mut W) + Send + 'static;
}

impl<W, Marker, S> IntoGroupRunner<W, SingleMarker<Marker>> for S
where
    S: GatedSystem<W, Marker>,
{
    fn into_group_runner(self) -> impl FnMut(&mut W) + Send + 'static {
        self.into_gated_runner()
    }
}

macro_rules! impl_group_runner_tuple {
    ($($system:ident $marker:ident $runner:ident),+) => {
        impl<W, $($system, $marker,)+> IntoGroupRunner<W, GroupMarker<($($marker,)+)>>
            for ($($system,)+)
        where
            $($system: GatedSystem<W, $marker>,)+
        {
            fn into_group_runner(self) -> impl FnMut(&mut W) + Send + 'static {
                let ($($runner,)+) = self;
                $(let mut $runner = $runner.into_gated_runner();)+
                move |world: &mut W| {
                    $($runner(world);)+
                }
            }
        }
    };
}

impl_group_runner_tuple!(S0 M0 r0);
impl_group_runner_tuple!(S0 M0 r0, S1 M1 r1);
impl_group_runner_tuple!(S0 M0 r0, S1 M1 r1, S2 M2 r2);
impl_group_runner_tuple!(S0 M0 r0, S1 M1 r1, S2 M2 r2, S3 M3 r3);
impl_group_runner_tuple!(S0 M0 r0, S1 M1 r1, S2 M2 r2, S3 M3 r3, S4 M4 r4);
impl_group_runner_tuple!(S0 M0 r0, S1 M1 r1, S2 M2 r2, S3 M3 r3, S4 M4 r4, S5 M5 r5);
impl_group_runner_tuple!(S0 M0 r0, S1 M1 r1, S2 M2 r2, S3 M3 r3, S4 M4 r4, S5 M5 r5, S6 M6 r6);
impl_group_runner_tuple!(
    S0 M0 r0, S1 M1 r1, S2 M2 r2, S3 M3 r3, S4 M4 r4, S5 M5 r5, S6 M6 r6, S7 M7 r7
);
impl_group_runner_tuple!(
    S0 M0 r0, S1 M1 r1, S2 M2 r2, S3 M3 r3, S4 M4 r4, S5 M5 r5, S6 M6 r6, S7 M7 r7, S8 M8 r8
);
impl_group_runner_tuple!(
    S0 M0 r0, S1 M1 r1, S2 M2 r2, S3 M3 r3, S4 M4 r4, S5 M5 r5, S6 M6 r6, S7 M7 r7, S8 M8 r8,
    S9 M9 r9
);
impl_group_runner_tuple!(
    S0 M0 r0, S1 M1 r1, S2 M2 r2, S3 M3 r3, S4 M4 r4, S5 M5 r5, S6 M6 r6, S7 M7 r7, S8 M8 r8,
    S9 M9 r9, S10 M10 r10
);
impl_group_runner_tuple!(
    S0 M0 r0, S1 M1 r1, S2 M2 r2, S3 M3 r3, S4 M4 r4, S5 M5 r5, S6 M6 r6, S7 M7 r7, S8 M8 r8,
    S9 M9 r9, S10 M10 r10, S11 M11 r11
);
impl_group_runner_tuple!(
    S0 M0 r0, S1 M1 r1, S2 M2 r2, S3 M3 r3, S4 M4 r4, S5 M5 r5, S6 M6 r6, S7 M7 r7, S8 M8 r8,
    S9 M9 r9, S10 M10 r10, S11 M11 r11, S12 M12 r12
);
impl_group_runner_tuple!(
    S0 M0 r0, S1 M1 r1, S2 M2 r2, S3 M3 r3, S4 M4 r4, S5 M5 r5, S6 M6 r6, S7 M7 r7, S8 M8 r8,
    S9 M9 r9, S10 M10 r10, S11 M11 r11, S12 M12 r12, S13 M13 r13
);
impl_group_runner_tuple!(
    S0 M0 r0, S1 M1 r1, S2 M2 r2, S3 M3 r3, S4 M4 r4, S5 M5 r5, S6 M6 r6, S7 M7 r7, S8 M8 r8,
    S9 M9 r9, S10 M10 r10, S11 M11 r11, S12 M12 r12, S13 M13 r13, S14 M14 r14
);
impl_group_runner_tuple!(
    S0 M0 r0, S1 M1 r1, S2 M2 r2, S3 M3 r3, S4 M4 r4, S5 M5 r5, S6 M6 r6, S7 M7 r7, S8 M8 r8,
    S9 M9 r9, S10 M10 r10, S11 M11 r11, S12 M12 r12, S13 M13 r13, S14 M14 r14, S15 M15 r15
);

pub fn run_if<W, Marker>(
    condition: impl Fn(&W) -> bool + Send + 'static,
    systems: impl IntoGroupRunner<W, Marker>,
) -> impl FnMut(&mut W) + Send + 'static {
    let mut runner = systems.into_group_runner();
    move |world: &mut W| {
        if condition(world) {
            runner(world);
        }
    }
}
