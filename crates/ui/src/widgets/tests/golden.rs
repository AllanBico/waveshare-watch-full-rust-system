//! Golden frame tests for every widget in both modes.

use crate::widgets::WidgetState;

use super::{render_desk, render_watch, SAMPLE_KINDS};

const WIDTH: u16 = 280;
const HEIGHT: u16 = 160;

#[test]
fn component_goldens_are_stable() {
    let expected = [
        (0x3929_4fbf_0645_3049, 0x00d1_5bd2_6d8f_ca39),
        (0xa251_7b91_76fb_b9bb, 0xf9a6_adf2_98b0_6495),
        (0x036c_e8f0_2ab2_64f5, 0xd9e6_8b17_a1e1_50c7),
        (0xd6b0_8327_9a47_8861, 0xa230_5b7b_6e66_4970),
        (0x67cd_6e00_0ac9_e8e8, 0xada8_3eda_232d_c472),
        (0x515d_ef55_d63f_67cc, 0x7982_bcc3_5130_30e2),
        (0x0ace_4882_3359_3e51, 0x9981_81df_cf28_f6c4),
        (0xa7db_f966_a69e_3ce4, 0x8e9e_e4df_ab55_0b8c),
        (0xda43_fee4_299d_1442, 0x3e87_1726_5f9e_bd0f),
        (0xef2f_a6f6_0747_1665, 0x7945_4a1b_7f25_56c9),
        (0x3f1a_4d52_16bf_da2b, 0xc38f_1873_d983_6618),
        (0xcbdc_2ebb_82d5_8725, 0xea70_6273_b12c_cc80),
        (0xe333_3c1e_08e6_e75a, 0x7b4e_e638_4f1d_206a),
        (0xbe9f_1642_070d_1c67, 0x8fc1_b918_180e_fae4),
        (0x65ce_456c_8197_38f4, 0xbf72_9fa9_181c_0640),
        (0x027b_41f3_8127_4680, 0x7dc5_708c_a6d0_54d0),
        (0xb533_1ce7_ba56_794c, 0x7629_3cbb_50f1_0b67),
        (0x03dc_b279_ccc4_a514, 0xa1e8_9681_123f_5088),
        (0x3220_6287_fb0a_67fa, 0x950b_7418_32d8_611b),
        (0xeef4_632c_3c00_4b90, 0xc215_6f26_c9d3_c2d0),
        (0x93ac_4348_b72e_922e, 0xc212_3536_061a_f72e),
        (0x58f4_89b5_b790_43c2, 0x5388_c95c_1216_3e22),
        (0x8c6b_18b5_d3fe_dd60, 0xfb5c_17a5_a6b1_de0c),
        (0x052c_49f5_19a7_d535, 0x6ea3_daa2_5c69_8a02),
        (0xbad3_d14e_fc03_4888, 0x3dd7_1425_d804_6cc8),
        (0xeb34_389e_6ce0_c1dc, 0x0276_f6c1_fc55_2394),
        (0x4838_2319_2431_6fc1, 0xc4a1_088f_503e_1487),
        (0x3cbc_5fef_b223_6496, 0x94d5_fdec_abb6_bb11),
        (0x537e_4e90_fd6d_573f, 0xdc55_14e2_b9e6_d317),
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
