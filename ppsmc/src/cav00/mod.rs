mod slice;

use crate::{
    automata::BuchiAutomata, cav00::slice::PartitionedFsmBdd, command::Args,
    ltl::ltl_to_automata_preprocess, property_driven::get_ltl, BddManager,
};
use smv::{bdd::SmvBdd, Smv};
use std::time::{Duration, Instant};
use sylvan::lace_run;

pub fn check(manager: BddManager, smv: Smv, args: Args) -> (bool, Duration) {
    let smvbdd = SmvBdd::new(&manager, &smv);
    let mut fsmbdd = smvbdd.to_fsmbdd(args.trans_method.into());
    let ltl = if args.generalize_automata {
        ltl_to_automata_preprocess(&smv, !smv.ltlspecs[0].clone())
    } else {
        fsmbdd.justice.clear();
        get_ltl(&smv, &[], args.flatten_define)
    };
    let ltl_fsmbdd =
        BuchiAutomata::from_ltl(ltl, &manager, &smvbdd.symbols, &smvbdd.defines).to_fsmbdd();
    let product = fsmbdd.product(&ltl_fsmbdd);
    let product = PartitionedFsmBdd::new(&product, &[0, 1, 2, 3, 4]);
    let start = Instant::now();
    let forward = lace_run(|_| product.reachable_from_init());
    let fair_cycle = lace_run(|_| product.fair_cycle_with_constrain(&forward));
    for i in 0..forward.len() {
        if !(&fair_cycle[i] & &forward[i]).is_constant(false) {
            return (false, start.elapsed());
        }
    }
    (true, start.elapsed())
}
