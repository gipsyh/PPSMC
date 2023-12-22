use crate::{Bdd, BddManager};
use fsmbdd::{FsmBdd, Trans};
use std::collections::HashMap;
use sylvan::LaceWorkerContext;

#[derive(Clone, Debug)]
pub struct PartitionedFsmBdd {
    pub symbols: HashMap<String, usize>,
    pub manager: BddManager,
    pub slice: Vec<Bdd>,
    pub init: Vec<Bdd>,
    pub trans: Trans<BddManager>,
    pub justice: Vec<Vec<Bdd>>,
}

impl PartitionedFsmBdd {
    fn slice_bdd(bdd: &Bdd, slice: &[Bdd]) -> Vec<Bdd> {
        let mut res = Vec::new();
        for slice in slice.iter() {
            res.push(bdd & slice);
        }
        res
    }

    fn reslice_bdd(&self, bdd: &[Bdd]) -> Vec<Bdd> {
        let mut context = LaceWorkerContext::get();
        for i in 0..self.slice.len() {
            let slice = self.slice[i].clone();
            let bdd = bdd.to_vec();
            let mut res = self.manager.constant(false);
            context.lace_spawn(move |_| {
                for j in 0..bdd.len() {
                    res |= &bdd[j] & &slice;
                }
                res
            });
        }
        context.lace_sync_multi(self.slice.len())
    }

    pub fn new(fsmbdd: &FsmBdd<BddManager>, partition: &[usize]) -> Self {
        assert!(partition.len() > 0);
        let mut slice = Vec::new();
        for i in 0..(1 << partition.len()) {
            let mut res = fsmbdd.manager.constant(true);
            for j in 0..partition.len() {
                res &= if i & (1 << j) > 0 {
                    fsmbdd.manager.ith_var(partition[j] * 2)
                } else {
                    !fsmbdd.manager.ith_var(partition[j] * 2)
                }
            }
            slice.push(res)
        }
        let init = Self::slice_bdd(&fsmbdd.init, &slice);
        let justice = fsmbdd
            .justice
            .iter()
            .map(|fair| Self::slice_bdd(fair, &slice))
            .collect();
        Self {
            symbols: fsmbdd.symbols.clone(),
            manager: fsmbdd.manager.clone(),
            slice,
            init,
            trans: fsmbdd.trans.clone(),
            justice,
        }
    }

    fn image(&self, state: &[Bdd], forward: bool) -> Vec<Bdd> {
        let mut context = LaceWorkerContext::get();
        state.iter().for_each(|bdd| {
            let bdd = bdd.clone();
            let trans = self.trans.clone();
            context.lace_spawn(move |_| {
                if forward {
                    trans.post_image(&bdd)
                } else {
                    trans.pre_image(&bdd)
                }
            });
        });
        let image: Vec<Bdd> = context.lace_sync_multi(state.len());
        self.reslice_bdd(&image)
    }

    pub fn reachable_with_constrain(
        &self,
        state: &[Bdd],
        forward: bool,
        contain_from: bool,
        constrain: &[Bdd],
    ) -> Vec<Bdd> {
        assert!(state.len() == self.slice.len());
        assert!(constrain.len() == self.slice.len());
        let mut frontier = state.to_vec();
        for i in 0..frontier.len() {
            frontier[i] &= &constrain[i]
        }
        let mut reach = if contain_from {
            frontier.clone()
        } else {
            vec![self.manager.constant(false); self.slice.len()]
        };
        loop {
            let mut new_frontier = self.image(&frontier, forward);
            for i in 0..new_frontier.len() {
                new_frontier[i] &= &constrain[i];
                new_frontier[i] &= !&reach[i];
            }
            if new_frontier
                .iter()
                .all(|bdd| *bdd == self.manager.constant(false))
            {
                break reach;
            }
            for i in 0..reach.len() {
                reach[i] |= &new_frontier[i];
            }
            frontier = new_frontier;
        }
    }

    pub fn reachable(&self, state: &[Bdd], forward: bool, contain_from: bool) -> Vec<Bdd> {
        self.reachable_with_constrain(
            state,
            forward,
            contain_from,
            &vec![self.manager.constant(true); self.slice.len()],
        )
    }

    pub fn reachable_from_init(&self) -> Vec<Bdd> {
        self.reachable(&self.init, true, true)
    }

    pub fn fair_cycle_with_constrain(&self, constrain: &[Bdd]) -> Vec<Bdd> {
        let mut res = constrain.to_vec();
        let mut y = 0;
        loop {
            y += 1;
            dbg!(y);
            let mut new = res.clone();
            for fair in self.justice.iter() {
                let fair: Vec<Bdd> = fair.iter().zip(&res).map(|(f, r)| f & r).collect();
                let backward = self.reachable_with_constrain(&fair, false, false, constrain);
                for i in 0..new.len() {
                    new[i] &= &backward[i];
                }
            }
            if new == res {
                break res;
            }
            res = new
        }
    }
}
