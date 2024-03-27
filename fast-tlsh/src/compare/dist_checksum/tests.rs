// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-FileCopyrightText: Copyright (C) 2024 Tsukasa OI <floss_ssdeep@irq.a4lg.com>.

//! Tests: [`crate::compare::dist_checksum`].

#![cfg(test)]

use super::generic;

#[test]
fn test_distance_1() {
    const BINARY: &[u8] = &[0, 1];
    let mut x = [0u8; 1];
    let mut y = [0u8; 1];
    // Only equality is checked
    for &y0 in BINARY {
        y[0] = y0;
        for &x0 in BINARY {
            x[0] = x0;
            assert_eq!(super::distance_1(x, y), generic::distance(x, y));
        }
    }
}

#[test]
fn test_distance_3() {
    const BINARY: &[u8] = &[0, 1];
    let mut x = [0u8; 3];
    let mut y = [0u8; 3];
    // Only equality is checked
    for &y0 in BINARY {
        y[0] = y0;
        for &y1 in BINARY {
            y[1] = y1;
            for &y2 in BINARY {
                y[2] = y2;
                for &x0 in BINARY {
                    x[0] = x0;
                    for &x1 in BINARY {
                        x[1] = x1;
                        for &x2 in BINARY {
                            x[2] = x2;
                            assert_eq!(super::distance_3(x, y), generic::distance(x, y));
                        }
                    }
                }
            }
        }
    }
}
