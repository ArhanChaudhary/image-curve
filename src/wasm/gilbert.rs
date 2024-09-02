// SPDX-License-Identifier: BSD-2-Clause
// Copyright (c) 2024 abetusk

pub struct Point {
    pub x: i32,
    pub y: i32,
}

pub fn gilbert_d2xy(idx: i32, w: i32, h: i32) -> Point {
    let mut _p = Point { x: 0, y: 0 };
    let mut _a = Point { x: 0, y: h };
    let mut _b = Point { x: w, y: 0 };

    if w >= h {
        _a.x = w;
        _a.y = 0;
        _b.x = 0;
        _b.y = h;
    }
    gilbert_d2xy_r(idx, 0, _p, _a, _b)
}

fn gilbert_d2xy_r(dst_idx: i32, cur_idx: i32, p: Point, a: Point, b: Point) -> Point {
    let mut _p: Point;
    let mut _a: Point;
    let mut _b: Point;
    let mut nxt_idx: i32;

    let w = (a.x + a.y).abs();
    let h = (b.x + b.y).abs();

    let da = Point {
        x: a.x.signum(),
        y: a.y.signum(),
    };
    let db = Point {
        x: b.x.signum(),
        y: b.y.signum(),
    };
    let i = dst_idx - cur_idx;

    if h == 1 {
        return Point {
            x: p.x + da.x * i,
            y: p.y + da.y * i,
        };
    }
    if w == 1 {
        return Point {
            x: p.x + db.x * i,
            y: p.y + db.y * i,
        };
    }

    let mut a2 = Point {
        x: a.x / 2,
        y: a.y / 2,
    };
    let mut b2 = Point {
        x: b.x / 2,
        y: b.y / 2,
    };

    let w2 = (a2.x + a2.y).abs();
    let h2 = (b2.x + b2.y).abs();

    if 2 * w > 3 * h {
        // prefer even steps
        if w2 % 2 != 0 && w > 2 {
            a2.x += da.x;
            a2.y += da.y;
        }

        nxt_idx = cur_idx + ((a2.x + a2.y) * (b.x + b.y)).abs();
        if cur_idx <= dst_idx && dst_idx < nxt_idx {
            return gilbert_d2xy_r(dst_idx, cur_idx, p, a2, b);
        }
        let cur_idx = nxt_idx;

        _p = Point {
            x: p.x + a2.x,
            y: p.y + a2.y,
        };
        _a = Point {
            x: a.x - a2.x,
            y: a.y - a2.y,
        };

        return gilbert_d2xy_r(dst_idx, cur_idx, _p, _a, b);
    }

    // prefer even steps
    if h2 % 2 != 0 && h > 2 {
        b2.x += db.x;
        b2.y += db.y;
    }

    // standard case: one step up, one long horizontal, one step down
    nxt_idx = cur_idx + ((b2.x + b2.y) * (a2.x + a2.y)).abs();
    if cur_idx <= dst_idx && dst_idx < nxt_idx {
        return gilbert_d2xy_r(dst_idx, cur_idx, p, b2, a2);
    }
    let cur_idx = nxt_idx;

    nxt_idx = cur_idx + ((a.x + a.y) * (b.x - b2.x + (b.y - b2.y))).abs();
    if cur_idx <= dst_idx && dst_idx < nxt_idx {
        _p = Point {
            x: p.x + b2.x,
            y: p.y + b2.y,
        };
        _b = Point {
            x: b.x - b2.x,
            y: b.y - b2.y,
        };
        return gilbert_d2xy_r(dst_idx, cur_idx, _p, a, _b);
    }
    let cur_idx = nxt_idx;

    _p = Point {
        x: p.x + (a.x - da.x) + (b2.x - db.x),
        y: p.y + (a.y - da.y) + (b2.y - db.y),
    };
    _a = Point { x: -b2.x, y: -b2.y };
    _b = Point {
        x: -(a.x - a2.x),
        y: -(a.y - a2.y),
    };

    gilbert_d2xy_r(dst_idx, cur_idx, _p, _a, _b)
}
