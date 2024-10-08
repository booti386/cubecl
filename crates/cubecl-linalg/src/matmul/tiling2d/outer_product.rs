use cubecl_core as cubecl;
use cubecl_core::prelude::*;

use super::config::CubeTiling2dConfig;

#[cube]
pub(crate) fn tile_outer_product<F: Float>(
    register_m: F,
    register_n: F,
    results: &mut Array<F>,
    #[comptime] config: CubeTiling2dConfig,
) {
    let tile_size = config.tile_size;
    let unroll = config.unroll_tile;

    #[unroll(unroll)]
    for res_idx_m in 0..tile_size {
        let res_pos_base = res_idx_m * tile_size;
        #[unroll(unroll)]
        for res_idx_n in 0..tile_size {
            let mul = register_m[res_idx_m] * register_n[res_idx_n];
            results[res_pos_base + res_idx_n] += mul;
        }
    }
}
