//! Golden frame tests for every widget in both modes.

use crate::widgets::WidgetState;

use super::{render_desk, render_watch, SAMPLE_KINDS};

const WIDTH: u16 = 280;
const HEIGHT: u16 = 160;

#[test]
fn component_goldens_are_stable() {
    let expected = [
        (0x3df6_c9be_e74e_ea4c, 0x3fc0_e024_0785_c64d),
        (0xa251_7b91_76fb_b9bb, 0xf9a6_adf2_98b0_6495),
        (0x4dbc_b383_1eee_8df9, 0xba90_a4e4_febc_58c3),
        (0xf508_b013_1264_2ce5, 0x847c_5e16_15b9_1c39),
        (0xfab3_2cfa_51bc_9d8a, 0x5955_809e_4b2f_5f9d),
        (0x0d04_c83b_1ff7_7120, 0x6f3c_9fac_cb27_4686),
        (0x64a6_bc5f_611d_7a82, 0xeb03_d80f_7132_65b9),
        (0x2646_4ffb_a553_4fcf, 0x3caf_e5ba_02cb_cf6a),
        (0xcc35_e4b7_1d9b_6562, 0x36b5_5392_0ad5_ae8e),
        (0xfc72_870d_346d_57ee, 0xf216_b58e_99d0_9e88),
        (0x590e_b044_dcfd_30b0, 0x1b26_e6f0_ad86_9bd4),
        (0x0655_82a3_a192_02b4, 0xfb00_b220_7705_8316),
        (0xff7d_4197_cc0c_4d4a, 0x991d_d3c9_0f87_b7ba),
        (0xb5c1_7738_be08_8cb4, 0x66e2_9a7e_9a23_fcd4),
        (0xc964_b241_646f_d217, 0x4270_bec8_194b_20f4),
        (0xdf43_71bb_65f9_2d50, 0x2cd7_c0ac_4435_f1ba),
        (0xc177_a325_ec0b_7b6c, 0x2739_7207_e636_7d41),
        (0xc97e_b1af_cf29_970d, 0xbc7c_d221_79cf_e137),
        (0x2799_ad12_757b_9569, 0x019c_d20f_2ab4_19a4),
        (0x5993_3bec_c83e_1e25, 0x9ce2_fab4_fb43_cb17),
        (0x93ac_4348_b72e_922e, 0xc212_3536_061a_f72e),
        (0x58f4_89b5_b790_43c2, 0x5388_c95c_1216_3e22),
        (0x8c6b_18b5_d3fe_dd60, 0xfb5c_17a5_a6b1_de0c),
        (0xf747_de35_d11a_c261, 0xa487_e312_e396_aede),
        (0xf69b_89c6_c146_dac8, 0x2a9a_3005_b923_d681),
        (0x35c1_c293_cfa0_8e2d, 0x370f_afce_8bc9_61a7),
        (0x4cde_557d_46eb_2e7c, 0x774b_e3a6_9502_c510),
        (0x3cbc_5fef_b223_6496, 0x94d5_fdec_abb6_bb11),
        (0x0eb6_1908_2e81_39db, 0xe8e3_a6ad_f8e7_264d),
        (0xc374_69e8_6033_98f2, 0x631e_de02_f071_340e),
    ];
    for (index, kind) in SAMPLE_KINDS.into_iter().enumerate() {
        let watch = render_watch(kind, WidgetState::NORMAL, WIDTH, HEIGHT).0;
        let desk = render_desk(kind, WidgetState::NORMAL, WIDTH, HEIGHT);
        assert_eq!(
            (watch, desk),
            expected[index],
            "component {index} changed its golden frame"
        );
    }
}
