use cubecl_core as cubecl;
use cubecl_core::prelude::*;

use super::super::runtime_info::{Fragments, Ids, SharedMemories};
use crate::matmul::cmma::{
    compute_loop::base::{
        get_smem_position_lhs, get_smem_position_rhs, load_tile_into_fragment, ComputeLoop,
    },
    config::ComptimeCmmaInfo,
};

pub(crate) struct AllBuffersFirstComputeLoop {}

#[cube]
impl ComputeLoop for AllBuffersFirstComputeLoop {
    fn compute_loop<F: Float, FC: Float>(
        shared_memories: SharedMemories<FC>,
        fragments: &mut Fragments<F, FC>,
        compute_ids: Ids,
        #[comptime] comptime_info: ComptimeCmmaInfo,
    ) {
        // Comptime values
        let block_size_k = comptime_info.block_size_k;
        let block_size_n = comptime_info.block_size_n;
        let tile_size = comptime_info.tile_size;
        let unroll = comptime_info.unroll;
        let num_accumulators = comptime_info.num_accumulators;
        let num_buffers = block_size_k / tile_size;
        let num_coop_per_row = (block_size_n / tile_size) / num_accumulators;

        // Runtime values
        let tile_row = compute_ids.coop / num_coop_per_row;
        let tile_col_base = (compute_ids.coop % num_coop_per_row) * num_accumulators;

        #[unroll]
        for accumulator_iter in 0..num_accumulators {
            #[unroll(unroll)]
            for buffer_iter in 0..num_buffers {
                load_tile_into_fragment(
                    get_smem_position_lhs::<F, FC>(tile_row, buffer_iter, comptime_info),
                    shared_memories.lhs,
                    &fragments.lhs,
                    comptime_info,
                );

                load_tile_into_fragment(
                    get_smem_position_rhs::<F, FC>(
                        buffer_iter,
                        tile_col_base + accumulator_iter,
                        comptime_info,
                    ),
                    shared_memories.rhs,
                    &fragments.rhs,
                    comptime_info,
                );

                let accumulator = &fragments.accumulators.index(accumulator_iter);
                cmma::execute::<FC, FC, F, F>(
                    &fragments.lhs,
                    &fragments.rhs,
                    accumulator,
                    accumulator,
                );
            }
        }
    }
}
