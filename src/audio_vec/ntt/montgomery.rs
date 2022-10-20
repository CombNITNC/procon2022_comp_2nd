use bytemuck::cast;
use wide::{i32x4, i32x8, i64x2, i64x4, u32x4, u32x8, CmpGt};

#[inline]
pub fn mul_u32x4(a: u32x4, b: u32x4, inv_n: u32x4, m1: u32x4) -> u32x4 {
    cast(
        cast::<_, i32x4>(mul_hi_u32x4(a, b)) + cast::<_, i32x4>(m1)
            - cast::<_, i32x4>(mul_hi_u32x4(a * b * inv_n, m1)),
    )
}

#[inline]
fn mul_hi_u32x4(a: u32x4, b: u32x4) -> u32x4 {
    let a_inner = cast::<_, i32x8>(a).to_array();
    let a13 = i32x4::new([a_inner[1], a_inner[1], a_inner[3], a_inner[3]]);
    let b_inner = cast::<_, i32x8>(b).to_array();
    let b13 = i32x4::new([b_inner[1], b_inner[1], b_inner[3], b_inner[3]]);
    let prod02 = cast::<_, i32x4>(a * b).to_array();
    let prod13 = cast::<_, i32x4>(a13 * b13).to_array();
    let prod_lo =
        cast::<_, i64x2>(i32x4::new([prod02[0], prod13[0], prod02[1], prod13[1]])).to_array();
    let prod_hi =
        cast::<_, i64x2>(i32x4::new([prod02[2], prod13[2], prod02[3], prod13[3]])).to_array();
    cast(i64x2::new([prod_lo[1], prod_hi[1]]))
}

#[inline]
pub fn add_u32x4(a: u32x4, b: u32x4, m2: u32x4, m0: u32x4) -> u32x4 {
    let ret = cast::<_, i32x4>(a) + cast::<_, i32x4>(b) - cast::<_, i32x4>(m2);
    cast(cast::<_, i32x4>(cast::<_, u32x4>(cast::<_, i32x4>(m0).cmp_gt(ret)) & m2) + ret)
}

#[inline]
pub fn sub_u32x4(a: u32x4, b: u32x4, m2: u32x4, m0: u32x4) -> u32x4 {
    let ret = cast::<_, i32x4>(a) - cast::<_, i32x4>(b);
    cast(cast::<_, i32x4>(cast::<_, u32x4>(cast::<_, i32x4>(m0).cmp_gt(ret)) & m2) + ret)
}

#[inline]
pub fn mul_u32x8(a: u32x8, b: u32x8, inv_n: u32x8, m1: u32x8) -> u32x8 {
    cast(
        cast::<_, i32x8>(mul_hi_u32x8(a, b)) + cast::<_, i32x8>(m1)
            - cast::<_, i32x8>(mul_hi_u32x8(a * b * inv_n, m1)),
    )
}

#[inline]
fn mul_hi_u32x8(a: u32x8, b: u32x8) -> u32x8 {
    let a_inner = cast::<_, i32x8>(a).to_array();
    let a13 = i32x8::new([
        a_inner[1], a_inner[1], a_inner[3], a_inner[3], a_inner[5], a_inner[5], a_inner[7],
        a_inner[7],
    ]);
    let b_inner = cast::<_, i32x8>(b).to_array();
    let b13 = i32x8::new([
        b_inner[1], b_inner[1], b_inner[3], b_inner[3], b_inner[5], b_inner[5], b_inner[7],
        b_inner[7],
    ]);
    let prod02 = cast::<_, i32x8>(a * b).to_array();
    let prod13 = cast::<_, i32x8>(a13 * b13).to_array();
    let prod_lo = cast::<_, i64x4>(i32x8::new([
        prod02[0], prod13[0], prod02[1], prod13[1], prod02[2], prod13[2], prod02[3], prod13[3],
    ]))
    .to_array();
    let prod_hi = cast::<_, i64x4>(i32x8::new([
        prod02[4], prod13[4], prod02[5], prod13[5], prod02[6], prod13[6], prod02[7], prod13[7],
    ]))
    .to_array();
    cast(i64x4::new([prod_lo[1], prod_hi[1], prod_lo[3], prod_hi[3]]))
}

#[inline]
pub fn add_u32x8(a: u32x8, b: u32x8, m2: u32x8, m0: u32x8) -> u32x8 {
    let ret = cast::<_, i32x8>(a) + cast::<_, i32x8>(b) - cast::<_, i32x8>(m2);
    cast(cast::<_, i32x8>(cast::<_, u32x8>(cast::<_, i32x8>(m0).cmp_gt(ret)) & m2) + ret)
}

#[inline]
pub fn sub_u32x8(a: u32x8, b: u32x8, m2: u32x8, m0: u32x8) -> u32x8 {
    let ret = cast::<_, i32x8>(a) - cast::<_, i32x8>(b);
    cast(cast::<_, i32x8>(cast::<_, u32x8>(cast::<_, i32x8>(m0).cmp_gt(ret)) & m2) + ret)
}
