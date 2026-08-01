pub enum SystemRunner<W> {
    Exclusive(Box<dyn FnMut(&mut W) + Send>),
    Shared(Box<dyn FnMut(&W) + Send>),
}

impl<W> SystemRunner<W> {
    pub fn run(&mut self, world: &mut W) {
        match self {
            Self::Exclusive(runner) => runner(world),
            Self::Shared(runner) => runner(&*world),
        }
    }
}

struct ScheduleEntry<W> {
    name: String,
    system: SystemRunner<W>,
    access: crate::access::SystemAccess,
    warmer: Option<fn(&mut W)>,
    flusher: Option<fn(&mut W)>,
}

struct Ordering {
    earlier: &'static str,
    later: &'static str,
}

pub struct Schedule<W> {
    entries: Vec<ScheduleEntry<W>>,
    orderings: Vec<Ordering>,
    ordered: bool,
    plan: Option<Vec<std::ops::Range<usize>>>,
}

impl<W> Schedule<W> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            orderings: Vec::new(),
            ordered: true,
            plan: None,
        }
    }

    pub fn order(&mut self, earlier: &'static str, later: &'static str) -> &mut Self {
        self.orderings.push(Ordering { earlier, later });
        self.ordered = false;
        self.plan = None;
        self
    }

    pub fn apply_orderings(&mut self) {
        if self.ordered {
            return;
        }
        self.ordered = true;
        self.plan = None;
        if self.orderings.is_empty() {
            return;
        }

        let index_of = |name: &str| self.entries.iter().position(|entry| entry.name == name);
        let mut successors: Vec<Vec<usize>> = vec![Vec::new(); self.entries.len()];
        let mut incoming = vec![0usize; self.entries.len()];
        for ordering in &self.orderings {
            let (Some(earlier), Some(later)) =
                (index_of(ordering.earlier), index_of(ordering.later))
            else {
                continue;
            };
            if earlier == later || successors[earlier].contains(&later) {
                continue;
            }
            successors[earlier].push(later);
            incoming[later] += 1;
        }

        let mut ready: std::collections::BinaryHeap<std::cmp::Reverse<usize>> = incoming
            .iter()
            .enumerate()
            .filter(|(_, count)| **count == 0)
            .map(|(index, _)| std::cmp::Reverse(index))
            .collect();
        let mut order = Vec::with_capacity(self.entries.len());
        while let Some(std::cmp::Reverse(index)) = ready.pop() {
            order.push(index);
            for successor in successors[index].clone() {
                incoming[successor] -= 1;
                if incoming[successor] == 0 {
                    ready.push(std::cmp::Reverse(successor));
                }
            }
        }

        assert!(
            order.len() == self.entries.len(),
            "Schedule::apply_orderings: these systems order each other in a cycle: {:?}",
            incoming
                .iter()
                .enumerate()
                .filter(|(_, count)| **count > 0)
                .map(|(index, _)| self.entries[index].name.as_str())
                .collect::<Vec<_>>()
        );

        let mut taken: Vec<Option<ScheduleEntry<W>>> = self.entries.drain(..).map(Some).collect();
        self.entries = order
            .into_iter()
            .map(|index| taken[index].take().expect("each entry is taken once"))
            .collect();
    }

    pub fn push<F>(&mut self, name: &str, system: F) -> &mut Self
    where
        F: FnMut(&mut W) + Send + 'static,
    {
        self.push_with_access(
            name,
            crate::access::SystemAccess::exclusive(),
            None,
            None,
            SystemRunner::Exclusive(Box::new(system)),
        )
    }

    pub fn push_with_access(
        &mut self,
        name: &str,
        access: crate::access::SystemAccess,
        warmer: Option<fn(&mut W)>,
        flusher: Option<fn(&mut W)>,
        system: SystemRunner<W>,
    ) -> &mut Self {
        let name = self.unique_name(name);
        self.entries.push(ScheduleEntry {
            name,
            system,
            access,
            warmer,
            flusher,
        });
        self.ordered = false;
        self.plan = None;
        self
    }
    pub fn replace<F>(&mut self, name: &str, system: F) -> &mut Self
    where
        F: FnMut(&mut W) + Send + 'static,
    {
        let index = self
            .index_of(name)
            .unwrap_or_else(|| panic!("Schedule::replace: system \"{name}\" not found"));
        self.entries[index].system = SystemRunner::Exclusive(Box::new(system));
        self.entries[index].access = crate::access::SystemAccess::exclusive();
        self.entries[index].warmer = None;
        self.entries[index].flusher = None;
        self.plan = None;
        self
    }

    pub fn remove(&mut self, name: &str) -> bool {
        let len_before = self.entries.len();
        self.entries.retain(|entry| entry.name != name);
        self.ordered = false;
        self.plan = None;
        self.entries.len() != len_before
    }

    pub fn contains(&self, name: &str) -> bool {
        self.entries.iter().any(|entry| entry.name == name)
    }

    pub fn names(&self) -> impl Iterator<Item = &str> + '_ {
        self.entries.iter().map(|entry| entry.name.as_str())
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn batch_layout(&self) -> BatchLayout<'_, W> {
        BatchLayout { schedule: self }
    }

    pub fn batches(&self) -> Vec<std::ops::Range<usize>> {
        if let Some(plan) = &self.plan {
            return plan.clone();
        }
        self.build_plan()
    }

    fn build_plan(&self) -> Vec<std::ops::Range<usize>> {
        let mut batches = Vec::new();
        let mut start = 0;
        while start < self.entries.len() {
            batches.push(start..self.batch_end(start));
            start = batches[batches.len() - 1].end;
        }
        batches
    }

    #[cfg(all(feature = "parallel", not(target_family = "wasm")))]
    fn plan(&mut self) -> &[std::ops::Range<usize>] {
        if self.plan.is_none() {
            self.plan = Some(self.build_plan());
        }
        self.plan.as_deref().unwrap_or_default()
    }

    fn batch_end(&self, start: usize) -> usize {
        if self.entries[start].access.is_exclusive() || self.entries[start].access.is_main_thread()
        {
            return start + 1;
        }
        let mut end = start + 1;
        while end < self.entries.len() {
            let candidate = &self.entries[end].access;
            if candidate.is_exclusive() || candidate.is_main_thread() {
                break;
            }
            if self.entries[start..end]
                .iter()
                .any(|entry| entry.access.conflicts_with(candidate))
            {
                break;
            }
            end += 1;
        }
        end
    }

    pub fn accesses(&self) -> impl Iterator<Item = (&str, &crate::access::SystemAccess)> {
        self.entries
            .iter()
            .map(|entry| (entry.name.as_str(), &entry.access))
    }

    fn index_of(&self, name: &str) -> Option<usize> {
        self.entries.iter().position(|entry| entry.name == name)
    }

    fn unique_name(&self, base: &str) -> String {
        if !self.contains(base) {
            return base.to_owned();
        }
        let mut suffix = 2;
        loop {
            let candidate = format!("{base}#{suffix}");
            if !self.contains(&candidate) {
                break candidate;
            }
            suffix += 1;
        }
    }
}

impl<W: Send + Sync> Schedule<W> {
    pub fn run(&mut self, world: &mut W) {
        self.apply_orderings();

        #[cfg(not(all(feature = "parallel", not(target_family = "wasm"))))]
        {
            for entry in &mut self.entries {
                run_entry(entry, world);
            }
        }

        #[cfg(all(feature = "parallel", not(target_family = "wasm")))]
        {
            let plan: Vec<std::ops::Range<usize>> = self.plan().to_vec();
            for batch in plan {
                if batch.len() == 1 {
                    run_entry(&mut self.entries[batch.start], world);
                } else {
                    run_batch(&mut self.entries[batch], world);
                }
            }
        }
    }
}

#[inline]
fn run_entry<W>(entry: &mut ScheduleEntry<W>, world: &mut W) {
    if let Some(warm) = entry.warmer {
        warm(world);
    }
    run_system(entry, world);
    flush_entry(entry, world);
}

#[inline]
fn run_system<W>(entry: &mut ScheduleEntry<W>, world: &mut W) {
    #[cfg(feature = "profiling")]
    let _span = tracing::info_span!("system", name = entry.name.as_str()).entered();
    entry.system.run(world);
}

#[inline]
fn flush_entry<W>(entry: &ScheduleEntry<W>, world: &mut W) {
    if let Some(flush) = entry.flusher {
        flush(world);
    }
}

#[cfg(all(feature = "parallel", not(target_family = "wasm")))]
fn run_batch<W: Send + Sync>(entries: &mut [ScheduleEntry<W>], world: &mut W) {
    for entry in entries.iter() {
        if let Some(warm) = entry.warmer {
            warm(world);
        }
    }
    assert_batch_is_disjoint(entries);
    let shared: &W = world;
    rayon::scope(|scope| {
        for entry in entries.iter_mut() {
            let SystemRunner::Shared(runner) = &mut entry.system else {
                unreachable!(
                    "system {:?} takes the whole world, so it must run alone",
                    entry.name
                );
            };
            #[cfg(feature = "profiling")]
            let name = entry.name.clone();
            scope.spawn(move |_| {
                #[cfg(feature = "profiling")]
                let _span = tracing::info_span!("system", name = name.as_str()).entered();
                runner(shared);
            });
        }
    });
    for entry in entries.iter() {
        flush_entry(entry, world);
    }
}

#[cfg(all(feature = "parallel", not(target_family = "wasm")))]
fn assert_batch_is_disjoint<W>(entries: &[ScheduleEntry<W>]) {
    for (index, entry) in entries.iter().enumerate() {
        for later in &entries[index + 1..] {
            assert!(
                !entry.access.conflicts_with(&later.access),
                "systems {:?} and {:?} share a batch, but the access they report overlaps",
                entry.name,
                later.name
            );
        }
    }
}

pub struct BatchLayout<'schedule, W> {
    schedule: &'schedule Schedule<W>,
}

impl<W> std::fmt::Display for BatchLayout<'_, W> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let batches = self.schedule.batches();
        let names: Vec<&str> = self.schedule.names().collect();
        let barriers = self
            .schedule
            .entries
            .iter()
            .filter(|entry| entry.access.is_exclusive())
            .count();

        write!(
            formatter,
            "systems {}, batches {}, barriers {}",
            names.len(),
            batches.len(),
            barriers
        )?;

        for batch in batches {
            let members = &names[batch.clone()];
            match members {
                [] => {}
                [only] => {
                    let shape = if self.schedule.entries[batch.start].access.is_exclusive() {
                        "barrier"
                    } else {
                        "alone"
                    };
                    write!(
                        formatter,
                        "
  {only} ({shape})"
                    )?;
                }
                many => {
                    write!(
                        formatter,
                        "
  {}",
                        many.join(", ")
                    )?;
                }
            }
        }

        Ok(())
    }
}

impl<W> Default for Schedule<W> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Stages<W> {
    pub stages: Vec<(&'static str, Schedule<W>)>,
}

impl<W> Stages<W> {
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    pub fn add_stage(&mut self, name: &'static str) -> &mut Self {
        assert!(
            !self.stages.iter().any(|(existing, _)| *existing == name),
            "stage {name:?} is already declared"
        );
        self.stages.push((name, Schedule::new()));
        self
    }

    pub fn stage_mut(&mut self, name: &str) -> &mut Schedule<W> {
        match self
            .stages
            .iter()
            .position(|(stage_name, _)| *stage_name == name)
        {
            Some(index) => &mut self.stages[index].1,
            None => {
                let declared: Vec<&'static str> = self
                    .stages
                    .iter()
                    .map(|(stage_name, _)| *stage_name)
                    .collect();
                panic!("no stage named {name:?}; declared stages are {declared:?}")
            }
        }
    }
}

impl Stages<crate::World> {
    pub fn run(&mut self, world: &mut crate::World) {
        world.events.update();
        for (_name, schedule) in &mut self.stages {
            schedule.run(world);
        }
    }
}

impl<W: Send + Sync> Stages<W> {
    pub fn apply_orderings(&mut self) {
        for (_name, schedule) in &mut self.stages {
            schedule.apply_orderings();
        }
    }
}

impl<W> Default for Stages<W> {
    fn default() -> Self {
        Self::new()
    }
}
