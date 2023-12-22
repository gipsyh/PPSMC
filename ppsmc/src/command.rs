use clap::{Parser, ValueEnum};
use fsmbdd::TransBddMethod;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum TransMethod {
    Monolithic,
    Partition,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Algorithm {
    PropertyDriven,
    Traditional,
    Cav00,
}

impl Into<TransBddMethod> for TransMethod {
    fn into(self) -> TransBddMethod {
        match self {
            TransMethod::Monolithic => TransBddMethod::Monolithic,
            TransMethod::Partition => TransBddMethod::Partition,
        }
    }
}

#[derive(Parser, Debug, Clone)]
/// Property-driven Symbolic Model Checking
pub struct Args {
    /// input file
    #[arg(short, long, default_value_t)]
    pub file: String,

    /// model checking algorithm
    #[arg(short = 'a', long, value_enum, default_value_t = Algorithm::PropertyDriven)]
    pub algorithm: Algorithm,

    /// trans partition method
    #[arg(short = 'm', long, value_enum, default_value_t = TransMethod::Monolithic)]
    pub trans_method: TransMethod,

    /// parallel
    #[arg(short, long, default_value_t = 1)]
    pub parallel: usize,

    /// extend trans
    #[arg(short = 'e', long)]
    pub ltl_extend_trans: Vec<usize>,

    /// extend vars
    #[arg(long = "ev")]
    pub ltl_extend_vars: Vec<usize>,

    /// verbose
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    /// old_impl
    #[arg(long = "op", default_value_t = false)]
    pub old_impl: bool,

    /// no generalize automata
    #[arg(long = "ga", default_value_t = false)]
    pub generalize_automata: bool,

    /// flatten define
    #[arg(long = "fd", default_value_t = false)]
    pub flatten_define: bool,
}
