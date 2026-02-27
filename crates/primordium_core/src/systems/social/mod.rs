pub mod legend;
pub mod rank;
pub mod reproduction;
pub mod specialization;
pub mod symbiosis;

pub use legend::{archive_if_legend_components, is_legend_worthy_components};
pub use rank::{
    are_same_tribe_components, calculate_social_rank_components, start_tribal_split_components,
};
pub use reproduction::{
    reproduce_asexual_parallel_components_decomposed,
    reproduce_sexual_parallel_components_decomposed,
};
pub use reproduction::{AsexualReproductionContext, ParentData, ReproductionContext};
pub use specialization::increment_spec_meter_components;
pub use symbiosis::{handle_symbiosis_components, PredationContext};
