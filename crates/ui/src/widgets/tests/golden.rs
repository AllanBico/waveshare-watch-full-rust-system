//! Golden frame tests for every widget in both modes.

use crate::widgets::WidgetState;

use super::{render_desk, render_watch, SAMPLE_KINDS};

const WIDTH: u16 = 280;
const HEIGHT: u16 = 160;

#[test]
fn component_goldens_are_stable() {
    let expected = [
        (0x3929_4fbf_0645_3049, 0x865f_3f4b_fb3e_c4c5),
        (0xa251_7b91_76fb_b9bb, 0x6e7f_4d0d_8a07_90cd),
        (0x036c_e8f0_2ab2_64f5, 0x4e69_6776_30a9_4957),
        (0xd6b0_8327_9a47_8861, 0xcb31_33ed_0c59_b5f6),
        (0x67cd_6e00_0ac9_e8e8, 0x59fa_0af2_333d_6b97),
        (0x515d_ef55_d63f_67cc, 0xe433_173f_d42a_81b0),
        (0x0ace_4882_3359_3e51, 0x602b_202e_3b8f_518a),
        (0xa7db_f966_a69e_3ce4, 0xeda3_f130_bdfb_4cac),
        (0xda43_fee4_299d_1442, 0x54f5_aa48_efcc_e9f6),
        (0xef2f_a6f6_0747_1665, 0x944a_80ed_3a81_f137),
        (0x3f1a_4d52_16bf_da2b, 0x1eb1_b519_82f8_de3f),
        (0xcbdc_2ebb_82d5_8725, 0xb8c4_61ca_053f_2079),
        (0xe333_3c1e_08e6_e75a, 0x8b5a_dc3d_b72e_fb06),
        (0xbe9f_1642_070d_1c67, 0x5338_aac1_6cdd_cac4),
        (0x65ce_456c_8197_38f4, 0x7634_d49c_af27_6836),
        (0x027b_41f3_8127_4680, 0xe269_6687_0d75_0d9c),
        (0xb533_1ce7_ba56_794c, 0x8e89_4e6f_ef6c_2f16),
        (0x03dc_b279_ccc4_a514, 0x550d_47a9_c4e8_9416),
        (0x106c_0925_3924_ecda, 0x501d_4545_1943_b3ac),
        (0xdcc9_018e_e013_c9bb, 0x8acf_b90d_4607_158e),
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
