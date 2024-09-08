use crate::renderer::Point;

pub fn shift(idx: u32, w: u32, h: u32) -> Point {
    let x = idx % w;
    let y = idx / w + x;
    Point(x as i32, y as i32)
}

// SPDX-License-Identifier: BSD-2-Clause
// Copyright (c) 2024 abetusk

pub fn gilbert_d2xy(idx: u32, w: u32, h: u32) -> Point {
    if w >= h {
        gilbert_d2xy_r(
            idx as i32,
            0,
            Point(0, 0),
            Point(w as i32, 0),
            Point(0, h as i32),
        )
    } else {
        gilbert_d2xy_r(
            idx as i32,
            0,
            Point(0, 0),
            Point(0, h as i32),
            Point(w as i32, 0),
        )
    }
}

fn gilbert_d2xy_r(dst_idx: i32, cur_idx: i32, p: Point, a: Point, b: Point) -> Point {
    let mut _p: Point;
    let mut _a: Point;
    let mut _b: Point;
    let mut nxt_idx: i32;

    let w = (a.0 + a.1).abs();
    let h = (b.0 + b.1).abs();

    let da = Point(a.0.signum(), a.1.signum());
    let db = Point(b.0.signum(), b.1.signum());
    let i = dst_idx - cur_idx;

    if h == 1 {
        return Point(p.0 + da.0 * i, p.1 + da.1 * i);
    }
    if w == 1 {
        return Point(p.0 + db.0 * i, p.1 + db.1 * i);
    }

    let mut a2 = Point(a.0 / 2, a.1 / 2);
    let mut b2 = Point(b.0 / 2, b.1 / 2);

    let w2 = (a2.0 + a2.1).abs();
    let h2 = (b2.0 + b2.1).abs();

    if 2 * w > 3 * h {
        // prefer even steps
        if w2 % 2 != 0 && w > 2 {
            a2.0 += da.0;
            a2.1 += da.1;
        }

        nxt_idx = cur_idx + ((a2.0 + a2.1) * (b.0 + b.1)).abs();
        if cur_idx <= dst_idx && dst_idx < nxt_idx {
            return gilbert_d2xy_r(dst_idx, cur_idx, p, a2, b);
        }
        let cur_idx = nxt_idx;

        _p = Point(p.0 + a2.0, p.1 + a2.1);
        _a = Point(a.0 - a2.0, a.1 - a2.1);

        return gilbert_d2xy_r(dst_idx, cur_idx, _p, _a, b);
    }

    // prefer even steps
    if h2 % 2 != 0 && h > 2 {
        b2.0 += db.0;
        b2.1 += db.1;
    }

    // standard case: one step up, one long horizontal, one step down
    nxt_idx = cur_idx + ((b2.0 + b2.1) * (a2.0 + a2.1)).abs();
    if cur_idx <= dst_idx && dst_idx < nxt_idx {
        return gilbert_d2xy_r(dst_idx, cur_idx, p, b2, a2);
    }
    let cur_idx = nxt_idx;

    nxt_idx = cur_idx + ((a.0 + a.1) * (b.0 - b2.0 + (b.1 - b2.1))).abs();
    if cur_idx <= dst_idx && dst_idx < nxt_idx {
        _p = Point(p.0 + b2.0, p.1 + b2.1);
        _b = Point(b.0 - b2.0, b.1 - b2.1);
        return gilbert_d2xy_r(dst_idx, cur_idx, _p, a, _b);
    }
    let cur_idx = nxt_idx;

    _p = Point(
        p.0 + (a.0 - da.0) + (b2.0 - db.0),
        p.1 + (a.1 - da.1) + (b2.1 - db.1),
    );
    _a = Point(-b2.0, -b2.1);
    _b = Point(-(a.0 - a2.0), -(a.1 - a2.1));

    gilbert_d2xy_r(dst_idx, cur_idx, _p, _a, _b)
}
